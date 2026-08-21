# 007 — API guidelines: fast by default

The kernel cannot make a slow app fast. It can make the fast path the
obvious path. These are the rules the SDK (`fri3d-wasm-api`) is built
around, and the ones to keep when it grows.

## Rules for the SDK

1. **No allocator in apps.** The SDK is `no_std` without `alloc`. State
   lives in `static` `AppCell<T>` values with `const` initialisers; the
   wasm data segment is the constructor. A Rust app is 1–7 KB of wasm and
   one 64 KB memory page.
2. **Render only when something changed.** The kernel renders on input,
   on a timer tick, or on `request_render()`. Apps that animate call
   `start_timer_ms(n)` and `stop_timer()` when the animation ends. An app
   without a timer costs zero CPU at rest.
3. **One crossing per frame where possible.** Every host import is a
   wasm→host call with fixed overhead. `canvas_draw_buffer` blits a whole
   frame in one call; `canvas_draw_bitmap` draws an icon in one call;
   `ui_*` widgets draw with a handful of calls. Prefer these to per-pixel
   `canvas_draw_dot` loops.
4. **Strings are borrowed, bounded and NUL-terminated.** The SDK copies
   into a 256-byte stack buffer; the kernel reads at most 256 bytes.
   Nothing is allocated on either side.
5. **Fixed-size records across the boundary.** `AppInfo` is a 256-byte
   header; settings are `u32`. No JSON, no varints, no length-prefixed
   anything.
6. **Integer math.** The canvas API is `i32`; apps that need fractions
   use fixed point (see Mandelbrot's Q16.16). The wasm32 target has
   floats, but the ESP32-S3 has no double-precision FPU and wasmi
   interprets them slowly.
7. **Budgets are visible.** `fri3d --headless ... ` prints fuel per
   render and memory per app. An app author can see "1.1M fuel" and know
   where they stand against the 40M cap before touching hardware.
8. **Lifecycle is optional and cheap.** `on_start` / `on_stop` /
   `on_pause` / `on_resume` exist for apps that need them (persist a
   high score in `on_stop`, pause a game in `on_pause`). Apps that do
   not export them pay nothing.

## What an app author writes

```rust
#![no_std]
use fri3d_wasm_api as api;

static COUNT: api::AppCell<u32> = api::AppCell::new(0);

fn render() {
    api::canvas_set_font(api::font::PRIMARY);
    api::canvas_draw_str(2, 12, "Hello badge");
}

fn on_input(key: u32, kind: u32) {
    if key == api::input::KEY_BACK && kind == api::input::TYPE_SHORT_PRESS {
        api::exit_to_launcher();
    }
    COUNT.set(COUNT.get() + 1);
}

fn on_stop() {
    api::settings_set_u32("hello", "presses", COUNT.get());
}

api::export_render!(render);
api::export_on_input!(on_input);
api::export_on_stop!(on_stop);
api::wasm_panic_handler!();
```

Plus `manifest.toml` and `icon.png`. `cargo run -p fri3d-pack` and the
app is in every host.

## Things we deliberately do not offer (yet)

- Dynamic memory from the host. Apps that need a buffer declare a
  `static`.
- Callbacks from host to app other than the fixed exports. No function
  pointers cross the boundary.
- Floating-point host imports.
- Blocking calls. Every import returns immediately; the app never waits
  on the host.
