# Input system

The kernel turns button edges into one event stream for every host.

## Keys

`up`, `down`, `left`, `right`, `ok` (A), `back` (X), `menu` (MENU).
SDK constants: `fri3d_wasm_api::input::KEY_*` (0–6).

## Event types

Delivered to `on_input(key, kind)`:

| Kind | When |
| --- | --- |
| `TYPE_PRESS` | key down |
| `TYPE_SHORT_PRESS` | released before 300 ms (sent before the release) |
| `TYPE_LONG_PRESS` | held 300 ms (sent while still held) |
| `TYPE_REPEAT` | every 150 ms after a long press, while held |
| `TYPE_RELEASE` | key up |

Hosts push raw edges; the kernel synthesises the rest, so timings are
identical on desktop, browser and badge. Queue depth is 32 events per
`step`; overflow drops, never blocks.

## Kernel-owned input

- **Menu short press** returns to the launcher. The app still receives
  `PRESS`, `LONG_PRESS`, `REPEAT` and `RELEASE` for Menu, so a long press
  is free for app use.
- **Left + Back held 500 ms** returns to the launcher without any event
  reaching the app.

## Conventions

- `Back` short press exits to the launcher in most apps; apps that use
  Back for something else (Mandelbrot: zoom out) exit on Back long press.
- Up/Down repeat is how long lists scroll fast; handle `TYPE_REPEAT` like
  `TYPE_SHORT_PRESS` for navigation.
- IMGUI apps forward every event to `imgui::ui_input(key, kind)`.
