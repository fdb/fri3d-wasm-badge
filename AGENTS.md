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

### ⚠️ C-Only Codebase

**IMPORTANT**: This project is strictly C-only (no C++). This is enforced for the following reasons:

1. **Smaller binary size** - No C++ runtime overhead, no STL
2. **Faster compilation** - C compiles significantly faster than C++
3. **Simpler toolchain** - Easier cross-compilation for embedded targets
4. **Library consistency** - All dependencies (u8g2, WAMR, FreeRTOS, lodepng) are pure C
5. **WASM compatibility** - C is the primary target for WASM toolchains
6. **No hidden allocations** - Deterministic memory management without STL
7. **Simpler FFI** - No C++ name mangling issues

**Allowed:**
- C11 standard (with GNU extensions where needed for platform APIs)
- Python scripts for tooling/testing

**Not allowed:**
- C++ source files (.cpp, .cc, .cxx)
- C++ headers (.hpp)
- C++ features: classes, templates, STL, namespaces, exceptions, RTTI
- C++ standard library headers (<iostream>, <string>, <vector>, etc.)

### C (All source files)

- **Standard**: C11
- **Indentation**: 4 spaces
- **Naming**: `snake_case` for functions and variables
- **Prefixes**: Use module prefixes (e.g., `canvas_`, `input_`, `wasm_runner_`)
- **Structs**: Use `typedef struct { ... } type_name_t;` pattern
- **Enums**: Use `typedef enum { ... } type_name_t;` pattern
- **Headers**: Use `#pragma once` and include guards with `extern "C"` for compatibility

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
