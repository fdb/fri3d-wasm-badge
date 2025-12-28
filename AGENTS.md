# Fri3d WASM Badge - Agent Guidelines

## Build Commands

### Prerequisites

1. **Git Submodules** (for WAMR runtime):
   ```bash
   git submodule update --init --recursive
   ```

2. **Zig** (for apps and emulator): Install from https://ziglang.org/download/

3. **SDL2** (for emulator): Install via package manager (brew, apt, etc.)

### Build Zig Apps (WASM)

Apps are always compiled to WASM. The same .wasm file runs on all platforms.

```bash
# Build all WASM apps
zig build

# Build and copy to www/ for web testing
zig build web

# Output location
ls zig-out/bin/*.wasm
```

### Build Desktop Emulator (Zig)

The Zig emulator loads and runs WASM apps using WAMR runtime and SDL2.

```bash
# First build WAMR (one time, via CMake)
cmake -B build/emulator
cmake --build build/emulator

# Build the Zig emulator
zig build emulator
```

### Build Legacy C Apps (WASM)

For apps written in C (not Zig):

```bash
cmake -B build/apps/circles -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/apps/circles
cmake --build build/apps/circles
```

### Run Emulator

```bash
# Show launcher (defaults to test_ui_zig.wasm)
./zig-out/bin/fri3d_emulator

# Run specific app
./zig-out/bin/fri3d_emulator zig-out/bin/circles_zig.wasm

# Or use zig build run
zig build run -- zig-out/bin/circles_zig.wasm
```

### Run Web Version

```bash
zig build web
cd www && python3 -m http.server 8000
# Open http://localhost:8000
```

### Clean Build

```bash
rm -rf build zig-out zig-cache
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     WASM App (.wasm)                        │
│  Written in: Zig (src/apps/*_zig/) or C (src/apps/*)       │
│  Exports: render(), on_input(), get_scene(), etc.          │
│  Includes: All drawing code (canvas.zig compiled in)       │
└─────────────────────────────────────────────────────────────┘
                              │
                    canvas_get_framebuffer()
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Platform Runtime                          │
│  ┌─────────────┐  ┌─────────────────┐  ┌────────────────┐  │
│  │     Web     │  │ Desktop (Zig)   │  │   ESP32        │  │
│  │ platform.js │  │ WAMR + SDL2     │  │ WAMR + SPI     │  │
│  └─────────────┘  └─────────────────┘  └────────────────┘  │
│  Reads framebuffer from WASM memory, displays it           │
│  Sends input events to WASM app via on_input()             │
└─────────────────────────────────────────────────────────────┘
```

**Key principle**: Apps are always WASM. All drawing code (canvas.zig) is compiled INTO the WASM app. Platform runtimes are thin - they just read the framebuffer and handle input/display.

## Project Structure

```
src/
├── sdk/zig/         # Zig SDK (canvas, platform, imgui)
│   ├── canvas.zig   # All drawing code (compiled into WASM apps)
│   ├── platform.zig # Platform interface
│   └── imgui.zig    # UI widget library
├── sdk/             # C SDK headers (legacy)
│   ├── canvas.h
│   ├── input.h
│   └── random.h
├── apps/            # WASM applications
│   ├── circles_zig/ # Zig app
│   ├── test_ui_zig/ # Zig app with IMGUI
│   ├── circles/     # C app (legacy)
│   └── mandelbrot/  # C app (legacy)
├── ports/
│   ├── web/         # Web platform (JS + HTML)
│   └── emulator/    # Desktop emulator (Zig + SDL2)
├── runtime/         # Shared C++ runtime (for WAMR build)
└── firmware/        # ESP32-S3 PlatformIO project
```

## Testing

### Visual Regression Tests

```bash
# Run all visual tests
uv run tests/visual/run_visual_tests.py

# Update golden images
uv run tests/visual/run_visual_tests.py --update-golden
```

## Code Style Guidelines

### Zig (Apps, SDK, Emulator)

- **Naming**: `camelCase` for functions/variables, `PascalCase` for types
- **Exports**: Use `export fn` for WASM exports
- **Required exports**: `render()`, optionally `on_input(key: u32, type: u32)`

### C++ (Runtime - src/runtime/)

- **Standard**: C++14
- **Indentation**: 4 spaces
- **Naming**: `camelCase` for members, `snake_case` for C-style, `PascalCase` for classes
- **Namespace**: `fri3d`

### C (Legacy WASM Apps - src/apps/)

- **Indentation**: 4 spaces
- **Naming**: `snake_case`
- **Use**: `WASM_IMPORT()` macro for host function imports

## Platform-Specific Notes

### Desktop Emulator (Zig)

- Uses SDL2 for window/input (128x64 scaled 4x = 512x256)
- WAMR for WASM execution
- Reads framebuffer via `canvas_get_framebuffer()` export
- Keyboard: Arrow keys, Enter/Z (OK), Backspace/X/Esc (Back)

### Web Platform (JavaScript)

- Loads WASM directly in browser
- Reads framebuffer from WASM memory
- Canvas 2D API for display (scaled 4x)

### ESP32 Firmware

- ESP32-S3-WROOM-1 target
- PlatformIO with ESP-IDF framework
- SPI display (SSD1306)
- WAMR for WASM execution

## Input Handling

- **Keys**: up, down, left, right, ok, back (as u32: 0-5)
- **Types**: press, release, short_press, long_press, repeat (as u32: 0-4)
- **Short press**: < 300ms
- **Long press**: >= 500ms
- **Reset combo**: LEFT + BACK held for 500ms returns to launcher

## WASM App Requirements

Apps must export:
- `void render()` - Called each frame (~60fps)
- `void on_input(u32 key, u32 type)` - For input handling
- `[*]u8 canvas_get_framebuffer()` - Returns pointer to 128x64 framebuffer
- `u32 canvas_get_framebuffer_size()` - Returns 8192

Optional exports (for visual testing):
- `u32 get_scene_count()` - Number of test scenes
- `void set_scene(u32 scene)` - Switch to scene
- `u32 get_scene()` - Get current scene
