# Stage 015: Network API for apps (DNS, TCP, UDP, HTTP, HTTPS)

Status: draft. Builds on design docs 015 (Wi-Fi) and 016 (network ops).
Nothing here is implemented yet; `net_probe` / `net_download` are the
only network imports today.

## Goals

- Apps can resolve names, open TCP and UDP sockets, and fetch over HTTP
  and HTTPS, on all three hosts.
- Every call stays fuel-capped and non-blocking; no app call waits for
  the network.
- No allocation in the call path; fixed socket and buffer tables with
  constants in `fri3d_kernel::limits`.
- TLS with real certificate validation, using a modern, pure-Rust stack;
  no TLS 1.0/1.1, no `verify = false` escape hatch for apps.
- Secrets (Wi-Fi passwords, client keys) stay host-side.

## Non-goals

- Servers / listening sockets on the badge (v1 is client-only).
- IPv6 (smoltcp supports it; enable later without ABI change — addresses
  are passed as 16-byte buffers from day one).
- Streaming bodies larger than RAM into app memory; apps read in chunks.

## Model

Everything is a **handle in a kernel table**, owned by the app instance
that created it and closed when the app stops (same rule as timers and
the current `net` operation). The host executes; the kernel only tracks
state, copies bytes between app memory and host buffers, and enforces
quotas. All calls return immediately with a status; apps poll, and the
kernel re-renders the app when a handle changes state (`take_changed`,
as `wifi` and `net` do today).

```
limits::NET_SOCKETS      = 4    // per app, TCP + UDP + TLS combined
limits::NET_DNS_QUERIES  = 2
limits::NET_RX_BUF       = 4096 // host-side bytes buffered per socket
limits::NET_TX_BUF       = 2048
limits::NET_HTTP         = 1    // HTTP(S) requests in flight per app
```

### Status codes (shared by every handle)

| Value | Meaning |
| --- | --- |
| 0 `IDLE` | Created, nothing started. |
| 1 `BUSY` | Operation in flight (resolving, connecting, handshaking, sending). |
| 2 `READY` | Connected / data available / request complete. |
| 3 `CLOSED` | Peer closed; drain remaining bytes then close. |
| −1 `ERR_*` | Negative: error class (`-1` generic, `-2` timeout, `-3` refused, `-4` dns, `-5` tls-cert, `-6` tls-other, `-7` quota, `-8` no-link). |

### DNS

```
net_dns_resolve(name_ptr, out_addr_ptr /*16 B*/) -> handle | ERR
net_dns_status(handle) -> status        // READY: out_addr is filled
```

Host: `embassy-net` DNS on the badge (servers from DHCP), `getaddrinfo`
on desktop, browser: not possible → `ERR_DNS` unless the name is an IP
literal.

### TCP

```
net_tcp_open() -> handle | ERR_QUOTA
net_tcp_connect(h, addr_ptr, port) -> status    // starts; poll status
net_tcp_send(h, ptr, len) -> bytes_taken | ERR   // copies into host TX buffer; 0 = full, try later
net_tcp_recv(h, ptr, len) -> bytes_copied | ERR  // 0 = nothing yet; CLOSED status when peer closed
net_tcp_close(h)
net_status(h) -> status
```

Buffers are fixed per socket (`NET_RX_BUF` / `NET_TX_BUF`); the host's
stack (smoltcp / std) does the real buffering. `send`/`recv` are pure
memcpy between app memory and the host buffer — bounded, fuel-cheap, no
blocking.

### UDP

```
net_udp_open(local_port) -> handle | ERR
net_udp_send_to(h, addr_ptr, port, ptr, len) -> status
net_udp_recv_from(h, ptr, len, out_addr_ptr, out_port_ptr) -> bytes | 0
```

One datagram per call, bounded by `NET_RX_BUF`. Enables NTP, mDNS
discovery, simple game protocols.

### TLS (over TCP)

```
net_tls_start(h, server_name_ptr) -> status   // upgrade a connected TCP handle
```

After `READY`, `net_tcp_send/recv` on the same handle carry plaintext;
the host encrypts. Implementation:

- **Badge**: `embedded-tls` (TLS 1.3 only, pure Rust, no_std, `rustls`
  lineage) over `embassy-net` TCP; ~16 KB RX + ~16 KB TX record buffers
  per session, allocated from PSRAM once per session. Certificate
  validation with `webpki` against a **bundled root store** (Mozilla CA
  list trimmed to the roots used by the hosts the badge talks to, plus
  the Fri3d update server). Time for validity checks comes from NTP
  (UDP, above) or, before NTP, a build-time floor date.
- **Desktop**: `rustls` + `webpki-roots`, TLS 1.2/1.3 (1.3 preferred).
- **Browser**: only via `fetch` (see HTTP); raw TLS sockets do not exist.

Apps cannot disable validation. A host flag (`--insecure-tls`, desktop
only) exists for debugging captive portals and is logged loudly.

### HTTP / HTTPS (convenience layer)

Most apps want "GET this URL into a buffer". A kernel-side request
object wraps DNS + TCP + TLS + HTTP/1.1 so the app never parses headers:

```
net_http_request(method, url_ptr, body_ptr, body_len) -> handle | ERR
net_http_header(h, name_ptr, value_ptr)          // before start; ≤ 8 headers
net_http_start(h) -> status
net_http_status_code(h) -> code | 0
net_http_read(h, ptr, len) -> bytes | 0          // body, chunked-decoded
net_http_close(h)
```

The host implements it as a single async task (badge) or thread
(desktop) and exposes a byte stream; the browser maps it to `fetch`
(subject to CORS). Redirects: followed up to 3 times for GET. Bodies
are streamed; `net_http_read` is the only way to get bytes, so a 10 MB
download never needs 10 MB of app memory.

## Policy and safety

- **Quota per app**, not global; a misbehaving app cannot starve the
  settings app's OTA download.
- **Host-only allow-list hook**: the kernel asks the host
  `net_policy(app_id, host, port) -> bool` before connecting. Default
  allow; the badge can later ship a deny-list (e.g. block port 25).
- **Wi-Fi off or no link** → every open returns `ERR_NO_LINK` at once;
  apps should show "connect Wi-Fi in Settings".
- **Stopping an app** closes all its handles; the host aborts in-flight
  work (same as `net_cancel` today).
- **Fuel**: each import is O(len) memcpy at most; `len` is clamped to
  `NET_RX_BUF`. No import loops on app-controlled input otherwise.

## Host implementation notes (badge)

- Keep the current "boxed futures polled from the main loop" model.
  Each socket gets one future for the operation in flight; `recv` just
  reads from the socket's smoltcp buffer, which needs no future.
- `embassy-net` `StackResources<N>`: N must cover DNS + all sockets;
  size it from `limits`.
- TLS memory is the budget item: 2 × 16 KB per session. Allow
  `NET_SOCKETS` TLS sessions only if PSRAM-backed.
- Firmware size: `embedded-tls` + `webpki` + root store ≈ 250–400 KB.
  Fits the 3.5 MB slot.

## Migration

`net_probe` / `net_download` stay as thin wrappers over this API (probe
= TCP open + connect + close; download = HTTP request + read + discard),
so the Speed Test app does not change.

## Open questions

- Root store refresh: ship with firmware only, or also allow the update
  server to push a new store? (Firmware-only for v1.)
- Whether to expose `net_http_request` at all in the browser host given
  CORS, or make the browser return `ERR_GENERIC` with a console hint.
- Per-app bandwidth accounting for the launcher's status bar.
