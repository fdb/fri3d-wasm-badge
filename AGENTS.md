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
WAMR is built from source by Zig's build system (no CMake needed).

```bash
# Build the Zig emulator (includes WAMR compilation)
# Use ReleaseFast for best performance (Debug mode has alignment issues with WAMR)
zig build emulator -Doptimize=ReleaseFast
```

Note: Debug builds may crash due to WAMR's fast interpreter doing unaligned memory access (valid in C but caught by Zig's debug checks). Use `-Doptimize=ReleaseFast` for a working build.

### Run Emulator

```bash
# Show launcher (defaults to test_ui.wasm)
./zig-out/bin/fri3d_emulator

# Run specific app
./zig-out/bin/fri3d_emulator zig-out/bin/circles.wasm

# Or use zig build run
zig build run -- zig-out/bin/circles.wasm
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
│  Written in: Zig (src/apps/*/)                             │
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
├── runtime/         # Zig runtime (canvas, platform, imgui)
│   ├── canvas.zig   # All drawing code (compiled into WASM apps)
│   ├── platform.zig # Platform interface
│   └── imgui.zig    # UI widget library
├── sdk/             # C SDK headers (for C apps wanting to compile to WASM)
│   ├── canvas.h
│   ├── input.h
│   └── random.h
├── apps/            # WASM applications (output: circles.wasm, test_ui.wasm, etc.)
│   ├── circles/     # Circles demo
│   ├── test_ui/     # UI test with IMGUI
│   └── mandelbrot/  # Mandelbrot fractal explorer
├── ports/
│   ├── web/         # Web platform (JS + HTML)
│   └── emulator/    # Desktop emulator (Zig + SDL2 + WAMR)
└── firmware/        # ESP32-S3 PlatformIO project

libs/
└── wasm-micro-runtime/  # WAMR submodule (built by Zig)
```

## Testing

### Visual Regression Tests

```bash
# Run all visual tests
uv run tests/visual/run_visual_tests.py

# Update golden images
uv run tests/visual/run_visual_tests.py --update-golden
```

## Zig 0.15 Build API Notes

This project uses Zig 0.15.x. Key API changes from earlier versions:

### Static Libraries (addLibrary replaces addStaticLibrary)

In Zig 0.15, `addStaticLibrary()` was removed. Use `addLibrary()` with `.linkage`:

```zig
// OLD (Zig 0.14 and earlier):
const lib = b.addStaticLibrary(.{
    .name = "mylib",
    .target = target,
    .optimize = optimize,
});

// NEW (Zig 0.15+):
const lib = b.addLibrary(.{
    .linkage = .static,  // or .dynamic
    .name = "mylib",
    .root_module = b.createModule(.{
        .target = target,
        .optimize = optimize,
    }),
});

// For C-only libraries (no Zig root file):
const lib = b.addLibrary(.{
    .linkage = .static,
    .name = "mylib",
    .root_module = b.createModule(.{
        .root_source_file = null,
        .target = target,
        .optimize = optimize,
    }),
});
```

### Executables with root_module

```zig
const exe = b.addExecutable(.{
    .name = "myapp",
    .root_module = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    }),
});
```

Sources:
- [Zig 0.15.1 Release Notes](https://ziglang.org/download/0.15.1/release-notes.html)
- [Ziggit: How to convert addStaticLibrary to addLibrary](https://ziggit.dev/t/how-to-convert-addstaticlibrary-to-addlibrary/12753)

## Code Style Guidelines

### Zig (Apps, Runtime, Emulator)

- **Naming**: `camelCase` for functions/variables, `PascalCase` for types
- **Exports**: Use `export fn` for WASM exports
- **Required exports**: `render()`, optionally `on_input(key: u32, type: u32)`

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

## App Development Guidelines

### Launcher System

The emulator starts with the launcher app by default (`zig-out/bin/launcher.wasm`). Apps can:
- Launch other apps via `platform.app.launchApp(app_id)`
- Return to launcher via `platform.app.exitToLauncher()`

App IDs are defined in the emulator's app registry (see `src/ports/emulator/main.zig`):
- 0: Launcher (always)
- 1: Test UI
- 2: Circles
- 3: Mandelbrot

### Exit Behavior Patterns

Different apps use different patterns for exiting:

1. **Simple apps** (e.g., Circles): Press Back to exit immediately
   ```zig
   if (key_enum == .back) {
       platform.app.exitToLauncher();
   }
   ```

2. **Menu-based apps** (e.g., Test UI): Press Back at top-level menu to exit
   ```zig
   if (imgui.backPressed()) {
       if (current_screen == .main_menu) {
           platform.app.exitToLauncher();
       } else {
           current_screen = .main_menu;
       }
   }
   ```

3. **Apps using Back for navigation** (e.g., Mandelbrot): Long-press Back to exit
   ```zig
   if (type_enum == .long_press and key_enum == .back) {
       platform.app.exitToLauncher();
   }
   ```

### Global Exit Combo

The platform always handles **LEFT + BACK held for 500ms** to return to the launcher. This works regardless of what the app is doing, providing a guaranteed escape hatch.

### Adding New Apps

1. Create `src/apps/yourapp/main.zig`
2. Implement required exports (`render`, `on_input`, etc.)
3. Add to `build.zig` with platform module import
4. Add to emulator's `APP_REGISTRY` with a unique ID
5. Add to launcher's app list

See `docs/developing_apps.md` for more details.
