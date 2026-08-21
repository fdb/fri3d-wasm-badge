# 016 — Network operations and the Speed Test app

Source: the Wi-Fi split (015), MicroPythonOS's `urequests`-style apps,
and the first IP traffic on the badge.

## What apps get today

Not sockets. One *operation* at a time, host-executed, progress-only:

| Import | Meaning |
| --- | --- |
| `net_probe(ip, port) -> i32` | Open and close a TCP connection. `ip` is a big-endian packed IPv4. |
| `net_download(url) -> i32` | Plain-HTTP GET; the host counts body bytes and discards them. |
| `net_status() -> i32` | 0 Idle, 1 Busy, 2 Done, 3 Failed. |
| `net_bytes() -> i32` | Body bytes received so far. |
| `net_elapsed_ms() -> i32` | Duration of the current or last operation. |
| `net_cancel()` | Abort. Also happens when the app stops. |

The kernel model (`fri3d_kernel::net`) is a request slot plus counters,
mirroring `wifi`: hosts `take_net_request`, then `net_progress(bytes)`
and `net_done(ok)`. Progress marks a visible change every 64 KB, so a
download does not force a frame per packet. Any app may use it; the
operation belongs to the focused app and dies with it.

**Why no socket API yet.** A socket API crosses the fuel and allocation
rules: app-visible buffers, blocking semantics inside a fuel-capped
call, TLS state per connection. That needs its own design
(`specs/015-network-api.md`). The two primitives here answer the first
real question — "is the badge on the internet, and how fast" — without
committing the ABI.

## Hosts

- **Desktop** (`NetDriver` in `hosts/desktop`): a worker thread with std
  sockets; `connect_timeout` 3 s for probes, hand-rolled HTTP/1.0 for
  downloads. Headless runs use `net::Sim` unless `--real-net` is given
  (then frames wait in wall time).
- **Browser**: `net::Sim` (probe 30 ms, 1 MB at 2 MB/s). `fetch` would
  need CORS on the target and cannot do the TCP probe.
- **Badge**: `embassy-net` 0.9 (DHCP, DNS, TCP) over
  `esp_radio::wifi::Interface::station()`. The stack's runner and the
  operation in flight are boxed futures polled from the main loop with
  the no-op waker, like the radio (015). The stack is built on the first
  request after association and never torn down. 16 KB TCP RX buffer,
  10 s socket timeout. `esp-rtos`'s `embassy` feature supplies the time
  driver `embassy-net` needs; no executor is involved — which means
  `embassy-time-queue-utils` must have a `generic-queue-*` feature on,
  otherwise `Timer` wake-ups assert that the waker came from an Embassy
  executor and panic (found on hardware, first probe).

## Speed Test app (`apps/speedtest`)

Three steps in sequence, driven from `render` (the kernel renders on
every network change): TCP to 1.1.1.1:53, TCP to 8.8.8.8:53, then
`http://speedtest.tele2.net/1MB.zip` (Tele2's public mirror; plain
HTTP until TLS exists). Shows milliseconds for the
probes; bytes and KB/s or MB/s for the download, then the final rate and
time. A 250 ms timer refreshes the elapsed counter. Back cancels and
exits. Nothing is written anywhere: the body is counted by the host
and dropped.

## Measured

Desktop (macOS, Wi-Fi): probes 100–150 ms, download limited by the
tele2 mirror. Badge (2026-08-21): both probes pass, DNS resolves,
`HTTP/1.1 200 OK`, but only ~62 KB/s — which is why the file is 1 MB.
The mirror (Sweden) is not the limit; the suspects are on our side:
one poll of the stack per main-loop iteration with a 2 ms idle sleep,
and the 16 KB socket RX buffer. Tuning that is the next step before
OTA downloads.
