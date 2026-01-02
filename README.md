# Fri3d WASM Badge

A WebAssembly-based badge platform for the Fri3d camp badge, featuring a 128x64 OLED display. Includes a desktop emulator (SDL), web emulator (Emscripten), and ESP32-S3 firmware.

## Getting Started

### Cloning the Repository

This project uses git submodules for dependencies. When cloning:

```bash
git clone --recursive https://github.com/fdb/fri3d-wasm-badge.git
cd fri3d-wasm-badge
```

Or clone first, then initialize submodules:

```bash
git clone https://github.com/fdb/fri3d-wasm-badge.git
cd fri3d-wasm-badge
git submodule update --init --recursive
```

## Prerequisites

### macOS

```
brew install cmake zig sdl2
```

## Building

### Quick Build (All Components)

```bash
./build_all.sh
```

This builds the desktop emulator and all WASM apps.

### Desktop Emulator Only

```bash
cmake -B build/emulator
cmake --build build/emulator
```

### WASM Apps Only

Requires [Zig](https://ziglang.org/) for the WASM toolchain:

```bash
cmake -B build/apps/circles -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/apps/circles
cmake --build build/apps/circles
```

### Running the Emulator

```bash
# Run launcher.wasm
./build/emulator/src/emulator/fri3d_emulator

# Run a specific app
./build/emulator/src/emulator/fri3d_emulator build/apps/circles/circles.wasm
```

### Controls

- Arrow keys: Navigate
- Z / Enter: OK / Select
- X / Backspace: Back
- S: Screenshot
- Left + X (hold 500ms): Return to launcher

## Running Tests

```bash
# Run all visual tests
uv run tests/visual/run_visual_tests.py

# Update golden images
uv run tests/visual/run_visual_tests.py --update-golden
```

### Trace Integration Tests

```bash
# Build trace harness + WASM apps and run trace tests
./build_trace_tests.sh
```

#### Updating Golden Traces

Some specs use `expected_mode: "exact"` and compare against golden traces in
`tests/trace/expected/<app>/<spec>.json`. To update them after intentional app changes:

```bash
# Run trace tests to generate new outputs
./build_trace_tests.sh

# Copy a specific trace to the expected location
cp tests/trace/output/<app>/<spec>.json tests/trace/expected/<app>/<spec>.json
```

## Project Structure

```
src/
├── sdk/          # WASM SDK headers (API for apps)
├── apps/         # WASM applications
│   ├── circles/
│   ├── mandelbrot/
│   └── test_drawing/
├── runtime/      # Shared runtime code (canvas, WASM runner, input handling)
├── emulator/     # Desktop SDL emulator
├── firmware/     # ESP32-S3 PlatformIO project
└── web/          # Web/Emscripten build

libs/             # Git submodules (WAMR, u8g2) + headers (stb)
cmake/            # CMake toolchain files
tests/            # Visual regression tests
```

### Platform Targets

| Platform          | Directory       | Build System | Status   |
| ----------------- | --------------- | ------------ | -------- |
| Desktop Emulator  | `src/emulator/` | CMake + SDL2 | Working  |
| Web Emulator      | `src/web/`      | Emscripten   | Skeleton |
| ESP32-S3 Firmware | `src/firmware/` | PlatformIO   | Skeleton |
| WASM Apps         | `src/apps/`     | CMake + Zig  | Working  |

## Dependencies

- **CMake** 3.16+
- **SDL2** (for desktop emulator)
- **Zig** (for WASM compilation)
- **PlatformIO** (for ESP32 firmware, optional)
- **Emscripten** (for web build, optional)

Git submodules:

- WASM Micro Runtime (WAMR)
- u8g2 graphics library
- stb_image_write (PNG encoding header in `libs/stb`)

## License

See individual submodule licenses in `libs/` directory.
