# Stage 012: ESP32-S3 Rust Port - Next Steps

This spec describes the concrete next steps to bring the Rust runtime to the ESP32-S3
firmware target, with a focus on build requirements, platform integration, and the
info we still need to proceed.

## Scope

- Target: ESP32-S3 (WROOM-1), SSD1306 128x64 display, 6 buttons.
- Goal: reuse `fri3d-runtime` (canvas/input/app manager/wasmi) on-device.
- Preserve behavior parity with desktop/web (event-driven rendering + timers).

## Deliverables

1) `fri3d-firmware` Rust crate (ESP-IDF based).
2) Platform backends:
   - Display driver wrapper (SSD1306 via SPI).
   - Input backend (GPIO + debouncing).
   - Time source (monotonic ms).
3) Integration:
   - Wire `InputManager` + `WasmRunner` + `AppManager`.
   - Load app registry + launcher.
4) Build + flash docs and scripts.

## Build Requirements (What’s needed to compile)

### Toolchain

- ESP-IDF install (version pinned; see “Info needed” below).
- Rust toolchain for ESP32-S3:
  - `espup` or `rustup` with the ESP target (`xtensa-esp32s3-espidf`).
- `ldproxy` / `esptool.py` (for flashing).
- CMake/Ninja (used by ESP-IDF).

### Host OS Dependencies

- Python 3.8+ (ESP-IDF tooling).
- `cmake`, `ninja`.
- `pkg-config` (if using `esp-idf-sys` + `esp-idf-hal`).

### Build System Choice (must be finalized)

Option A (recommended): **esp-idf-sys + esp-idf-hal**
- Rust crate uses the ESP-IDF SDK and can keep `std` enabled.
- Simplifies networking, filesystem, and driver bindings.

Option B: **no_std + esp-hal**
- Smaller binaries, but more manual driver work.
- Wasm runtime integration may be harder depending on memory limits.

## Integration Steps (Proposed)

1) **Create crate**
   - Add `fri3d-firmware/` with Cargo manifest.
   - Set target to `xtensa-esp32s3-espidf`.

2) **Display backend**
   - Implement SSD1306 SPI driver wrapper.
   - Provide a framebuffer flush method compatible with `fri3d-runtime::Canvas`.
   - Ensure 128x64, 1bpp → SPI write; match u8g2 output layout.

3) **Input backend**
   - GPIO input + debouncing.
   - Feed raw press/release events into `InputManager`.
   - Preserve timing constants (300/150/500ms).

4) **Time source**
   - Provide monotonic `now_ms()` based on `esp_timer_get_time()` or equivalent.

5) **WASM runtime**
   - Integrate `wasmi` (or alternative if memory constrained).
   - Load WASM binaries from flash (SPIFFS/LittleFS) or from embedded assets.

6) **App registry + launcher**
   - Mirror the Rust desktop/web registry (IDs 1..N, launcher as 0).
   - Confirm storage path or embed strategy for WASM binaries.

7) **Event-driven loop**
   - Main loop:
     - Poll input → `InputManager`.
     - Deliver events to app.
     - Render only when input, timer due, or `request_render`.

8) **Testing hooks**
   - Optional: enable trace feature for on-device validation.
   - Add a simple “smoke” test app build for hardware validation.

## Info Needed From You

Please confirm or provide:

1) **ESP-IDF version** to target (and whether you prefer PlatformIO or pure ESP-IDF + cargo).
2) **Board details**: exact module/board, flash size, PSRAM availability.
3) **Display wiring**: SPI pins, reset pin, DC pin, CS pin, and SPI clock target.
4) **Button wiring**: GPIO pins, active-low/active-high, external pull-ups?
5) **Storage strategy for WASM**:
   - Embed in firmware? (binary size constraints)
   - Load from SPIFFS/LittleFS?
6) **Memory budget constraints** (RAM/PSRAM) for the WASM runtime.
7) **Power/refresh policy**:
   - Any strict power limits or watchdog constraints?
8) **Required peripherals** beyond display/buttons (battery monitoring, LEDs, etc.).
9) **Preferred rust crates** (esp-idf-hal vs esp-hal) if you already decided.

Once these are confirmed, we can draft the concrete `fri3d-firmware` crate structure,
`build_firmware.sh`, and the first hardware bring-up checklist.
