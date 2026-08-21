# 009 — App ABI reference

The contract between a `.fab` app and the kernel. Everything an app can
import from module `env`, everything it may export, and the rules the
kernel enforces. `fri3d-wasm-api` wraps all of this in safe Rust; this
document is the ground truth the wrapper follows.

ABI version: `kernel_version() == 1`. The version bumps when a signature
changes or a function is removed. Adding a function is not a bump: an
app that does not import it never notices.

## Conventions

- All values are `i32` on the wire. Unsigned quantities are passed as
  their bit pattern; negative lengths and sizes are treated as zero.
- Strings are NUL-terminated UTF-8 in app memory, at most 256 bytes.
  The kernel clips longer strings and treats invalid UTF-8 as empty. A
  pointer outside memory draws nothing. **No import traps on bad input.**
- Coordinates are canvas pixels, origin top-left, `x` right, `y` down.
  Text `y` is the baseline.
- Every call from the kernel into the app (`render`, `on_input`,
  lifecycle) runs with `limits::FUEL_PER_CALL` (40M) wasm instructions.
  Exceeding it traps the app; the kernel drops it and returns to the
  launcher. The app cannot catch this.
- Requests (`start_app`, `exit_to_launcher`) take effect after the
  current call returns. An app never observes its own teardown.

## Imports (`env`)

### Canvas

| Signature | Semantics |
| --- | --- |
| `canvas_clear()` | Fill with white. The kernel already clears before every `render`. |
| `canvas_width() -> i32`, `canvas_height() -> i32` | 160, 120. Prefer the SDK constants. |
| `canvas_set_color(c)` | 0 white, 1 black, 2 XOR. Other values = black. |
| `canvas_set_font(f)` | 0 Primary (helvB08, bold), 1 Secondary (haxrcorp4089), 2 Keyboard (profont11), 3 BigNumbers (profont22, digits only). |
| `canvas_draw_dot(x, y)` | One pixel. Out of range: ignored. |
| `canvas_draw_line(x1, y1, x2, y2)` | Bresenham, inclusive ends. |
| `canvas_draw_frame(x, y, w, h)` / `canvas_draw_box(x, y, w, h)` | Outline / filled rectangle. |
| `canvas_draw_rframe(x, y, w, h, r)` / `canvas_draw_rbox(...)` | Rounded variants. |
| `canvas_draw_circle(x, y, r)` / `canvas_draw_disc(x, y, r)` | Outline / filled. XOR-safe: each pixel touched once. |
| `canvas_draw_str(x, y, ptr)` | Text at baseline `y`, current font and colour. |
| `canvas_string_width(ptr) -> i32` | Advance width in pixels for the current font. |
| `canvas_draw_buffer(ptr, len)` | Replace the framebuffer with `len` bytes (0 white, 1 black), row-major. `len` clipped to `width*height`. One call per frame for full-frame renderers. |
| `canvas_draw_bitmap(x, y, w, h, ptr)` | 1-bit bitmap, rows of `ceil(w/8)` bytes, MSB = leftmost pixel. Set bits drawn in the current colour, clear bits transparent. `w ≤ 160`, `h ≤ 120`. |

Known wart: the SDK's `imgui::ui_icon` reads bitmaps LSB-first, the opposite of `canvas_draw_bitmap`. Icons in `.fab` headers are MSB-first; draw them with `canvas_draw_bitmap`.

### Random and time

| Signature | Semantics |
| --- | --- |
| `random_seed(seed)` | Reseed the shared MT19937. Shared with the launcher: reseed in `on_start` if you need determinism. |
| `random_get() -> i32` | 32 random bits. |
| `random_range(max) -> i32` | `[0, max)`. `max ≤ 0` returns 0. |
| `get_time_ms() -> i32` | Host clock in ms, monotonic, wraps at 2³². Sampled once per `step`, so it is constant within a frame. |

### Rendering control

| Signature | Semantics |
| --- | --- |
| `start_timer_ms(interval)` | Render every `interval` ms while the app is focused. `0` stops. Missed periods coalesce: one render, never a burst. |
| `stop_timer()` | Stop the timer. The kernel also stops it when the app loses focus or is dropped. |
| `request_render()` | Render once more after the current call. Inside `render`, at most one extra pass. |

### Apps and kernel

| Signature | Semantics |
| --- | --- |
| `exit_to_launcher()` | Leave the app. No-op for the launcher. |
| `start_app(index)` | Start registry app `index`. From an app: that app stops first. Invalid index: kernel error, focus unchanged. |
| `app_count() -> i32` | Installed apps (launcher excluded). |
| `app_info(index, ptr, len) -> i32` | Copy the app's 256-byte bundle header to `ptr`. Returns 256, or -1 if `index` is out of range or `len < 256` or the range does not fit. Offsets: see 006. |
| `kernel_version() -> i32` | This document's version. |

### Settings

| Signature | Semantics |
| --- | --- |
| `settings_get_u32(ns, key, default) -> i32` | Read `ns/key` or `default`. |
| `settings_set_u32(ns, key, value) -> i32` | Write. Returns 1 on success, 0 when denied or the table (64 entries) is full. |

Policy: `ns` must equal the app's `id`. Apps packed with `system = true`
may also use `ns = "system"`. Keys and namespaces are at most 23 bytes.
The kernel persists the table through the host when it changes.

Reserved `system` keys: `brightness` (10–100), `sound` (0/1).

### Debug

| Signature | Semantics |
| --- | --- |
| `log_str(ptr)` | Append one line (≤ 96 bytes) to the kernel log ring (8 lines). Hosts drain it to the console. Dropped silently when full. |

## Exports

| Export | Required | Called |
| --- | --- | --- |
| `memory` | yes | The kernel reads and writes app memory through it. |
| `render()` | yes | After the kernel cleared the canvas, whenever a frame is due. |
| `on_input(key, kind)` | no | Per input event. Keys 0–6 (Up, Down, Left, Right, Ok, Back, Menu); kinds 0–4 (Press, Release, ShortPress, LongPress, Repeat). |
| `on_start()` | no | Once, right after instantiation. |
| `on_resume()` | no | After `on_start`, and whenever the app regains the screen. |
| `on_pause()` | no | Whenever the app loses the screen. |
| `on_stop()` | no | Once, before the instance is dropped. Persist state here. |
| `get_scene_count() -> i32`, `get_scene() -> i32`, `set_scene(i32)` | no | Test hooks for deterministic screenshots. |

A `_start` function (wasm start section) runs during instantiation with
the same fuel budget; Rust `cdylib`s do not emit one.

### Input sequencing

For one key: `Press`, then either `ShortPress` (held < 300 ms) or
`LongPress` (at 300 ms while held), then `Repeat` every 150 ms while
held after a long press, then `Release`. A short press is delivered
*before* the release. Holding Left + Back for 500 ms returns to the
launcher without any event reaching the app.

`Menu` short press goes home before `on_input` sees it. The app does see
`Menu` press/release/long press/repeat.

## Memory and limits

- One linear memory, at most `limits::APP_MEMORY_MAX` (256 KB).
  `memory.grow` past the cap traps.
- Rust apps: 16 KB wasm stack (`.cargo/config.toml`), one 64 KB page in
  practice.
- No tables beyond one, no multi-memory, no SIMD.

## Lifecycle, precisely

```
launch:   load → on_start → on_resume → render
home:     on_pause → on_stop → drop → launcher.on_resume → render
trap:     (same as home; last_error set; stats.app_traps += 1)
launcher: load → on_start → on_resume; per app launch: on_pause … on_resume
```

A lifecycle export that traps is treated like a trap in `render`: the app
is dropped. `on_pause`/`on_stop` traps during teardown are ignored.

## Versioning policy

- Add imports freely; document them here and in `fri3d-wasm-api`.
- Never change an existing signature. Add `name2` instead.
- Bump `KERNEL_VERSION` only when removing or changing semantics, and
  keep the old function working for one camp season.
- Bundle header changes bump `bundle::FORMAT_VERSION` separately.
