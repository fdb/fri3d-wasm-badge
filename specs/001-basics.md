# Stage 001: Basics (System Overview)

Purpose: describe the Fri3d WASM Badge system in a language-agnostic way so it can be reimplemented (e.g., in Rust) while preserving behavior.

## Core goals and constraints

- Display: 128x64 monochrome framebuffer (SSD1306 class). Treat as 1bpp, origin (0,0) top-left.
- Input: 6 logical keys (up/down/left/right/ok/back) with press/release + synthesized short/long/repeat.
- Apps: compiled to WebAssembly and invoked by a host runtime (render per frame, input events delivered asynchronously).
- Immediate redraw: full-screen redraw each frame; no dirty rectangles.
- Share the same runtime + graphics engine across platforms; avoid re-implementing drawing with platform-native APIs (e.g., web canvas 2D) because it changes pixel output.
- Deterministic tests: some apps expose scene switching and use fixed RNG seeds for visual tests.

## High-level architecture

- Host platform layer (per target):
  - Display backend (SDL/web canvas/ESP32 SPI).
  - Input backend (SDL events/web events/ESP32 GPIO).
  - Time source (millisecond ticks).
- Shared runtime layer:
  - `app_manager`: loads and switches apps, handles launcher, forwards input/render.
  - `wasm_runner`: loads WASM, registers imports, calls exports.
  - `input_manager`: converts raw presses/releases into short/long/repeat and reset combo.
  - `random`: deterministic PRNG.
  - `canvas`: drawing API backed by the display buffer.
- App layer (WASM):
  - Uses SDK imports from `env` and exports `render()` + optional `on_input()`.
  - Apps are pure draw + input logic with no direct host resources.

## Data flow (conceptual frame loop)

1) Poll raw input events from platform backend.
2) Input manager synthesizes short/long/repeat and queues events.
3) For each event, call the active app's `on_input()` if exported.
4) Call the active app's `render()` each frame.
5) Flush the framebuffer to the platform display.
6) Process any app-requested switch to launcher or another app.

## Source-of-truth references

- Platform overview + build layout: `README.md`
- Input model: `docs/input_system.md`
- IMGUI high-level API: `docs/imgui.md`
- Runtime modules: `src/runtime/`
- SDK headers used by apps: `src/sdk/`
- Emulator implementation: `src/emulator/`
- Web emulator implementation: `src/web/` and `src/web/shell.html`
- Firmware skeleton: `src/firmware/`

## Porting note

Keep the following invariants across a reimplementation:
- Screen dimensions and coordinate system.
- Input timings for short/long/repeat and reset combo (see `src/runtime/input.h`).
- WASM import module name `env` and function signatures (see `src/sdk/*.h` and `src/runtime/wasm_runner.c`).
- App IDs and launcher behavior (see `src/apps/launcher/main.c`).
