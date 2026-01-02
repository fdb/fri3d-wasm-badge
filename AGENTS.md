# Fri3d WASM Badge - Agent Guidelines

## Build Commands

### Prerequisites: Git Submodules

This project uses git submodules. Before building, ensure submodules are initialized:

```bash
git submodule update --init --recursive
```

### Build Everything

```bash
./build_all.sh
```

### Build Desktop Emulator Only

```bash
cmake -B build/emulator
cmake --build build/emulator
```

### Build WASM Apps

Requires Zig for the WASM toolchain:

```bash
cmake -B build/apps/circles -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/apps/circles
cmake --build build/apps/circles
```

### Run Emulator

```bash
# Show launcher
./build/emulator/src/emulator/fri3d_emulator

# Run specific app
./build/emulator/src/emulator/fri3d_emulator build/apps/circles/circles.wasm
```

### Clean Build

```bash
rm -rf build
```

## Testing

### Visual Regression Tests

```bash
# Run all visual tests
uv run tests/visual/run_visual_tests.py

# Update golden images
uv run tests/visual/run_visual_tests.py --update-golden
```

### Trace Integration Tests

Run this after changes that could affect runtime behavior to avoid regressions:

```bash
./build_trace_tests.sh
```

#### Updating Golden Traces

Specs using `expected_mode: "exact"` compare against
`tests/trace/expected/<app>/<spec>.json`. After intentional changes:

```bash
./build_trace_tests.sh
cp tests/trace/output/<app>/<spec>.json tests/trace/expected/<app>/<spec>.json
```

### CI Note (New Apps)

If you add a new app with visual tests, also update the CI build list in
`.github/workflows/visual-tests.yml` so the app's WASM binary is built before
`run_visual_tests.py` runs.

## Project Structure

```
src/
├── sdk/          # WASM SDK headers (API for apps)
│   ├── canvas.h
│   ├── input.h
│   ├── random.h
│   └── frd.h
├── apps/         # WASM applications
│   ├── circles/
│   ├── mandelbrot/
│   └── test_drawing/
├── runtime/      # Shared runtime (canvas, WASM runner, input handling)
├── emulator/     # Desktop SDL emulator
├── firmware/     # ESP32-S3 PlatformIO project
└── web/          # Web/Emscripten build
```

## Code Style Guidelines

### C++ (Runtime & Emulator - src/runtime/, src/emulator/)

- **Standard**: C++14
- **Indentation**: 4 spaces
- **Naming**:
  - Variables/functions: `camelCase` for class members, `snake_case` for C-style
  - Classes: `PascalCase`
  - Private members: `m_` prefix
- **Namespace**: `fri3d`

### C (WASM Apps - src/apps/)

- **Indentation**: 4 spaces
- **Naming**: `snake_case`
- **Use**: `WASM_IMPORT()` macro for host function imports
- **Required exports**: `render()`, optionally `on_input()`

### CMakeLists.txt

- **Minimum version**: 3.16
- For WASM apps: use `cmake/toolchain-wasm.cmake`

## Platform-Specific Notes

### Desktop Emulator

- Uses SDL2 for window/input
- u8g2 for graphics buffer (SSD1306 128x64)
- WAMR for WASM execution

### ESP32 Firmware

- ESP32-S3-WROOM-1 target
- PlatformIO with ESP-IDF framework
- SPI display (SSD1306)
- Button GPIO pins configurable

### Web Build

- Emscripten compilation
- SDL2 via Emscripten
- WASM apps fetched from /apps/ directory

## Input Handling

- **Short press**: < 300ms
- **Long press**: >= 500ms
- **Reset combo**: LEFT + BACK held for 500ms returns to launcher

## WASM App Requirements

Apps must export:
- `void render()` - Called each frame
- `void on_input(InputKey key, InputType type)` - Optional, for input handling
- `uint32_t get_scene_count()` - Optional, for multi-scene test apps
- `void set_scene(uint32_t scene)` - Optional
- `uint32_t get_scene()` - Optional
