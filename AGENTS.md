# Fri3d WASM Badge - Agent Guidelines

## Build Commands

### Build Everything

```bash
./build_all.sh
```

### Build Desktop Emulator Only

```bash
./build_emulator.sh
```

### Build WASM Apps

```bash
./build_apps.sh
```

### Run Emulator

```bash
# Show launcher
./target/release/fri3d-emulator

# Run specific app
./target/release/fri3d-emulator build/apps/circles/circles.wasm
```

### Clean Build Artifacts

```bash
rm -rf build target
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

If you add a new app, make sure it is included in the Rust workspace and that
`build_apps.sh` produces its WASM binary so visual/trace tests can find it.

## Project Structure

```
fri3d-runtime/        # Shared runtime (canvas, input, wasm runner, trace)
fri3d-emulator/       # Desktop emulator (minifb)
fri3d-web/            # Web host (wasm32 + JS shell)
fri3d-wasm-api/        # WASM SDK for Rust apps
fri3d-app-*/          # Rust WASM apps
fri3d-trace-harness/  # Trace test harness
specs/                # Specs for the Rust port
```

## Code Style Guidelines

### Rust (Runtime, Emulator, Apps)

- **Edition**: 2021
- **Indentation**: 4 spaces
- **Naming**:
  - Types: `PascalCase`
  - Functions/vars: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`
- Prefer safe Rust; contain `unsafe` behind minimal, audited boundaries.

## Platform-Specific Notes

### Desktop Emulator

- Uses minifb for window/input
- Renders from the Rust framebuffer
- Wasm execution via wasmi

### Web Build

- Host runtime compiled to wasm32
- JS shell loads apps and provides `env` imports

## Input Handling

- **Short press**: < 300ms
- **Long press**: >= 300ms
- **Reset combo**: LEFT + BACK held for 500ms returns to launcher

## Power/Battery Goals

- Default to event-driven rendering (input or explicit `request_render()`).
- Use app timers only when needed (animations/game loops) and stop them when idle.
- Avoid continuous redraws to conserve battery on physical devices.

## WASM App Requirements

Apps must export:
- `void render()` - Called each frame
- `void on_input(InputKey key, InputType type)` - Optional, for input handling
- `uint32_t get_scene_count()` - Optional, for multi-scene test apps
- `void set_scene(uint32_t scene)` - Optional
- `uint32_t get_scene()` - Optional
