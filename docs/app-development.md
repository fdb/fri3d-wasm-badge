# Writing an app

An app is a Rust crate compiled to `wasm32-unknown-unknown`, packed with
its manifest and icon into a `.fab` bundle, and run by the kernel on
every host. This guide is the short path; the exact ABI is in
[design_docs/009-app-abi.md](../design_docs/009-app-abi.md).

## 1. Create the folder

```
apps/hello/
  Cargo.toml
  manifest.toml
  icon.png
  src/lib.rs
```

`Cargo.toml`:

```toml
[package]
name = "fri3d-app-hello"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
fri3d-wasm-api = { path = "../../fri3d-wasm-api" }
```

`manifest.toml`:

```toml
id = "hello"                 # [a-z0-9_], unique; also your settings namespace
name = "Hello"
version = "0.1.0"
author = "Your name"
description = "Says hello."  # shown on the launcher's info page
category = "Demo"
icon = "icon.png"            # 14x14; dark pixels are drawn
```

`icon.png`: any PNG, 14×14. Dark = set pixel, light or transparent =
clear. Draw it in any editor, or as ASCII and convert with Pillow.

Add `"apps/hello"` to `members` in the root `Cargo.toml`.

## 2. Write the app

```rust
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{color, font, input};

static PRESSES: api::AppCell<u32> = api::AppCell::new(0);

fn on_start() {
    PRESSES.set(api::settings_get_u32("hello", "presses", 0));
}

fn render() {
    api::canvas_set_color(color::BLACK);
    api::canvas_set_font(font::PRIMARY);
    api::canvas_draw_str(4, 16, "Hello, badge");
    api::canvas_set_font(font::SECONDARY);
    api::canvas_draw_str(4, 30, "A: count   B: home");
    api::canvas_draw_frame(0, 0, api::SCREEN_WIDTH, api::SCREEN_HEIGHT);
}

fn on_input(key: u32, kind: u32) {
    match (key, kind) {
        (input::KEY_OK, input::TYPE_SHORT_PRESS) => PRESSES.set(PRESSES.get() + 1),
        (input::KEY_BACK, input::TYPE_SHORT_PRESS) => api::exit_to_launcher(),
        _ => {}
    }
}

fn on_stop() {
    api::settings_set_u32("hello", "presses", PRESSES.get());
}

api::export_render!(render);
api::export_on_input!(on_input);
api::export_on_start!(on_start);
api::export_on_stop!(on_stop);
api::wasm_panic_handler!();
```

Rules the kernel enforces, so design around them:

- **No allocator.** Keep state in `static AppCell<T>` with `const`
  initialisers. Arrays, not `Vec`.
- **Render only when asked.** The kernel calls `render` after input, on a
  timer, or after `request_render()`. For animation call
  `start_timer_ms(50)` and `stop_timer()` when done. An app without a
  timer costs nothing at rest.
- **Each call has a fuel budget** (40M wasm instructions). A loop that
  never returns is killed and the badge goes home. `fri3d --headless`
  prints fuel per frame; Mandelbrot uses ~2.5M.
- **One memory, 256 KB max.** One 64 KB page is typical.
- **`Menu` goes home.** You cannot override it. `Back` is yours.
- **Settings** are `u32` under your own `id`. Write them in `on_stop`.
- **Wi-Fi** is read-only for apps: `api::wifi::status()`,
  `current_ssid()`, the scan and saved lists. Only the Settings app
  (system) scans, saves and connects. There is no socket API yet; apps
  can run one host-side operation at a time via `api::net`
  (`probe(ip, port)`, `download(url)`, then poll `status()`, `bytes()`,
  `elapsed_ms()`). See `apps/speedtest`.

## 3. Pack and look at it

```bash
cargo run -q -p fri3d-pack
cargo run -q --release -p fri3d-host-desktop -- \
    --headless --app hello --keys ok,ok --frames 1 --screenshot hello.png
```

`--keys` takes `up,down,left,right,ok,back,menu` (prefix `long` for a
long press, e.g. `longok`). The PNG is the exact framebuffer. The same
command prints fuel and memory.

Interactive: `cargo run --release -p fri3d-host-desktop`, then find your
app in the launcher.

## 4. Lifecycle

```
load → on_start → on_resume → render … → on_pause → on_stop → drop
```

All four are optional. `on_pause`/`on_resume` bracket every moment the
app is not on screen (today that only happens to the launcher; apps are
stopped, not paused, when the user leaves).

## 5. Drawing

Canvas: 160×120, 1 bit. Colours: `WHITE`, `BLACK`, `XOR`. Fonts:
`PRIMARY` (bold 8 px), `SECONDARY` (small 8 px), `KEYBOARD` (mono 11),
`BIG_NUMBERS` (digits, 22 px). Text `y` is the baseline.

For full-frame renderers keep a `static [u8; 160*120]` and push it with
`canvas_draw_buffer` once per frame. For icons use `canvas_draw_bitmap`.
For menus, labels, buttons and a keyboard use [imgui.md](imgui.md).

## 6. Testing

- Deterministic: seed with `random_seed` in `on_start`, or run the host
  with `--seed`.
- Scenes: export `get_scene_count` / `get_scene` / `set_scene` and use
  `--scene N` to screenshot each screen.
- Browser: `hosts/web/build.sh`, open `test.html`, extend
  `hosts/web/tests.js` with `fri3d.startApp(i)` / `fri3d.tap(key)` /
  `fri3d.readFb()`.
