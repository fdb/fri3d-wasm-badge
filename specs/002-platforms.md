# Stage 002: Platform Groundwork (Emulator, Web, ESP32)

This stage describes the host/platform layer that provides display, input, and timing.

## Desktop emulator (SDL)

Reference: `src/emulator/`

### Display backend (SDL + u8g2)

- `display_sdl_t` owns SDL window/renderer/texture and a u8g2 framebuffer.
- Buffer format: u8g2 SSD1306 128x64, full framebuffer.
- `display_sdl_flush()` reads the u8g2 buffer and converts each bit into RGBA pixels for SDL.
- Scaling: window uses 4x scaling (`DISPLAY_SDL_SCALE_FACTOR` in `src/emulator/display_sdl.c`).
- Headless mode: skip SDL window/renderer; still uses u8g2 buffer.
- Screenshots: `display_sdl_save_screenshot()` encodes 128x64 RGBA to PNG using `stb_image_write`.

### Input backend (SDL)

- `input_sdl.c` maps keyboard input to logical keys:
  - Arrow keys -> up/down/left/right
  - Z or Enter -> ok
  - X or Backspace -> back
- `S` triggers screenshot (emulator only).
- Key repeats are ignored at the SDL event layer; repeats are synthesized by the runtime input manager.
- Time source: `SDL_GetTicks()`.

### Emulator main loop

- `src/emulator/main.c` composes: display -> canvas -> random -> input -> runtime.
- App registry is fixed at startup (launcher + 5 apps).
- Options: `--test`, `--scene`, `--screenshot`, `--headless`.
- Loop: poll input, synthesize events, deliver to app, render, flush, ~60fps delay.

## Web emulator (Rust wasm host + JS runtime)

References: `src/web/main.c`, `src/web/shell.html`

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

References: `src/firmware/README.md`, `src/firmware/src/*.c`

### Hardware assumptions

- MCU: ESP32-S3-WROOM-1
- Display: SSD1306 128x64 over SPI
- Buttons: 6 GPIOs, active-low with pull-ups
- Pinout (placeholder):
  - SPI: MOSI=6, MISO=8, SCK=7, CS=5, DC=4, RST=48
  - Buttons: Up=9, Down=10, Left=11, Right=12, Ok=13, Back=14

### Current firmware skeleton

- `display_spi.c` uses u8g2 over ESP32 SPI to render into the SSD1306.
- `input_gpio.c` debounces GPIO and queues press/release events only.
- `main.c` initializes display + input and polls inputs in a loop.
- WAMR + shared runtime not integrated yet (noted in `src/firmware/README.md`).

### Porting expectation

- The firmware should eventually reuse the same runtime modules (canvas, input manager, app manager, wasm runner) as the emulator.
- Input GPIO should feed the runtime input manager so long/short/repeat behavior matches other platforms.
- Rust porting plan:
  - Create a `fri3d-firmware` crate (ESP-IDF/PlatformIO) that provides display,
    input, and time backends for the shared runtime.
  - Keep rendering event-driven with optional app timers to conserve battery.
  - Integrate the WASM runner (Wasmi) once platform memory limits are validated.
