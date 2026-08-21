# 015 — Wi-Fi: kernel model, host radio, settings flow

Source: MicroPythonOS `WifiService` + `wifi_settings.py`, and our own
host split (design doc 010).

## The split

The kernel owns the Wi-Fi *model*. Hosts own the *radio*. The model is:

- the saved-network store (SSID + password, `limits::WIFI_SAVED_MAX` = 8),
- the last scan (`WIFI_SCAN_MAX` = 16, strongest first),
- the link state (`WifiStatus`: Off, Idle, Connecting, Connected, Failed),
- the auto-connect state machine.

A host driver only executes three primitives and reports back:

| Kernel → host (`take_wifi_request`) | Host → kernel |
| --- | --- |
| `WifiRequest::Scan` | `wifi_scan_done(&[ScanEntry])` |
| `WifiRequest::Connect { ssid, password }` | `wifi_connect_done(ok)` |
| `WifiRequest::Disconnect` | — |
| | `wifi_link_lost()` when an association drops |

One request slot; a newer request replaces an unserviced one. The host
polls the slot once per loop iteration, after it blitted the frame, so
"scanning..." is on screen before a blocking step starts.

**Why in the kernel.** MicroPythonOS keeps the policy (try saved networks
strongest-first, fall through on failure, hidden networks last) in a
Python service. Ours lives in `fri3d_kernel::wifi`, so the badge, the
desktop and the browser behave the same and the whole flow is tested in
`wifi.rs` at millisecond cost. The host code stays small and boring.

## Auto-connect

`set_enabled(true)` (boot with `system.wifi = 1`, or the toggle) starts a
round: scan, then connect to the strongest *visible* saved network, then
the next on failure, until `Failed` (or `Idle` when none was visible).
A lost link schedules a new round after 10 s. A manual `connect(ssid)`
never falls through to other networks: the user asked for that one, and
"failed" next to it is the answer.

Hidden networks are not tried blind. Typing them via "Add network" saves
and connects them once; they reconnect only when the scan shows them.

## Policy

- Reads (`wifi_status`, scan and saved lists, current SSID) are open to
  every app: the launcher draws the status icon.
- Actions (`wifi_scan/save/forget/connect/disconnect/set_enabled`) are
  **system-only**. A user app calling them gets 0 and nothing happens
  (`lifecycle.rs: wifi_actions_are_system_only_reads_are_open`).
- Passwords never leave the kernel. Apps can list saved SSIDs; only a
  `WifiRequest::Connect` carries a password, and only the host sees it.

## Persistence

Saved networks are a separate image (`wifi::IMAGE_LEN` = 776 bytes,
magic `FWF1`), dirty-tracked like settings: `take_wifi_image`. Badge:
sector 1 of the `settings` partition (`FWIF`). Desktop:
`~/.fri3d-badge/wifi.bin`. Browser: `localStorage["fri3d.wifi"]`.
Passwords are stored in clear, as MicroPythonOS does; the flash is not
encrypted and the badge has no secure element.

## The simulated radio

`wifi::Sim` is a deterministic driver used by the desktop host, the
browser host and the tests: four fixed networks (`SIM_NETWORKS`), a
1.5 s scan, a 2 s connect, success only with the right password or on
the open network. `fri3d --headless` drives it with the scripted clock,
so screenshots of "scanning...", "connecting" and "connected" are
reproducible.

## Badge driver (esp-radio 1.0.0-beta.0)

- esp-radio's scan/connect are `async` only. The main loop is not. Each
  request becomes one boxed future polled once per loop iteration with
  `Waker::noop()`; the driver's own tasks run under `esp-rtos`, which
  `main` starts right after the allocators (`esp_rtos::start(timg0,
  sw_interrupt0)`). No executor, no second task for us.
- The controller is created on the first request and leaked to
  `'static`, so the future can own `&'static mut WifiController` and hand
  it back on completion. Wi-Fi off means the radio is never powered.
- `esp-radio` needs the `unstable` feature for `is_connected`, used to
  detect link loss between requests.
- The driver's task stacks and buffers must come from internal RAM
  (`esp_alloc::InternalMemory`). esp-alloc serves the *first registered
  region that fits*, so with the internal heap registered before PSRAM
  the kernel and wasmi had eaten it before the first scan, and
  `WifiController::new` panicked with "Failed to allocate stack". The
  badge now registers PSRAM first and a 160 KB internal region second:
  ordinary allocations go to PSRAM, internal RAM is reserved for
  capability-tagged requests.
- Image grows from 1.1 MB to 1.58 MB of the 3.5 MB slot.

## Settings app flow

Settings → Wi-Fi: `Wi-Fi On/Off`, `Scan`, `Add network`, then the list
(scan results strongest first, then saved networks out of range).
Row values: `connecting` / `connected` / `failed` / `saved` / `open`.
OK on a row: saved → Connect / Change password / Forget; unknown secured
→ password keyboard (min 8 chars) → save + connect; open → connect at
once. The on-screen keyboard has no cancel key; long-press Back cancels.
The launcher draws a Wi-Fi mark top-left while connected (it replaced
the `n/m` counter; the scrollbar shows position).

No timers: the kernel sets `needs_render` whenever the Wi-Fi model
changes (`Wifi::take_changed`), so the focused app redraws when a scan
finishes or the link changes.

## Not done

- No IP stack. Association only; `embassy-net` on the badge is the next
  step for OTA and NTP.
- No signal bars in the list; the order carries that information.
- WPA3 / enterprise: `AuthenticationMethod::Wpa2Personal` is assumed
  for any non-empty password.
