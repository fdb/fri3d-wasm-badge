# Stage 002: Platform Groundwork (Emulator, Web, ESP32)

This stage describes the host/platform layer that provides display, input, and timing.

## Desktop emulator (Rust minifb)

Reference: `fri3d-emulator/src/main.rs`

### Display backend (minifb)

- Uses the Rust `Canvas` framebuffer (1bpp) from `fri3d-runtime`.
- Framebuffer is expanded to RGBA and displayed in a minifb window.
- Scaling: 4x scale (`Scale::X4`).
- Headless mode: skip window creation; still renders to the framebuffer for screenshots.
- Screenshots: writes 128x64 PNGs via the `png` crate.

### Input backend

- Key mapping:
  - Arrow keys -> up/down/left/right
  - Z or Enter -> ok
  - X or Backspace -> back
- `S` triggers a screenshot saved as `screenshot_<n>.png` in the current working directory.
- OS key repeats are ignored; repeats are synthesized by the runtime input manager.
- Time source: `Instant::now()` (monotonic).

### Emulator main loop

- Composes: canvas -> random -> input -> runtime (wasmi).
- App registry is fixed at startup (launcher + 6 apps, including WiFi Pet).
- Options: `--test`, `--scene`, `--screenshot`, `--headless`.
- Loop: poll input, synthesize events, deliver to app, render on demand, update window.

## Web emulator (Rust wasm host + JS runtime)

References: `fri3d-web/src/lib.rs`, `fri3d-web/shell.html`

### Rust wasm host

- Build a `fri3d-web` wasm module that exposes the runtime building blocks:
  - Canvas drawing + font rendering identical to native (`fri3d-runtime` canvas).
  - Random and input manager logic for parity.
  - Framebuffer access (pointer + size) for JS blitting to `<canvas>`.
- JS bridges app imports to these Rust exports (no `unsafe` in apps).

### JS runtime (shell)

- Loads the Rust host module and app WASM binaries.
- App WASM runs natively in the browser (no Wasmi/WAMR in web).
- Provides `env` imports for apps by forwarding to the Rust host:
  - Canvas operations, random, time, input, and `request_render()`.
  - Timer control via `start_timer_ms()` / `stop_timer()` to schedule renders.
- Event-driven loop:
  - Render on input events, `request_render()`, or when a timer tick is due.
  - Avoid continuous redraws to conserve energy.

### App switching + input delivery

- `exit_to_launcher()` and `start_app()` set a pending request.
- JS swaps the current app WASM and triggers an immediate render.
- Input events are queued in the Rust input manager; JS drains synthesized
  events each frame and forwards them to the app's `on_input()` export.

## ESP32-S3 firmware (hardware target)

### Hardware assumptions

- MCU: ESP32-S3-WROOM-1
- Display: SSD1306 128x64 over SPI
- Buttons: 6 GPIOs, active-low with pull-ups
- Pinout (placeholder):
  - SPI: MOSI=6, MISO=8, SCK=7, CS=5, DC=4, RST=48
  - Buttons: Up=9, Down=10, Left=11, Right=12, Ok=13, Back=14

### Current status

- Rust firmware crate is not yet implemented.
- No on-device runtime integration is present in this repo.

### Porting expectation

- The firmware should reuse the same runtime modules (canvas, input manager, app manager, wasm runner) as the emulator.
- Input GPIO should feed the runtime input manager so long/short/repeat behavior matches other platforms.
- Rust porting plan:
  - Create a `fri3d-firmware` crate (ESP-IDF) that provides display, input, and time backends for the shared runtime.
  - Keep rendering event-driven with optional app timers to conserve battery.
  - Integrate the WASM runner (`wasmi`) once platform memory limits are validated.
