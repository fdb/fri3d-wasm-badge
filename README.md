# FRI3D WASM Badge

A WebAssembly-based badge emulator for the FRI3D camp badge, featuring a 128x64 OLED display simulation.

## Getting Started

### Cloning the Repository

This project uses git submodules for dependencies (WAMR, u8g2, lodepng). When cloning, use one of these methods:

**Option 1: Clone with submodules in one step**
```bash
git clone --recursive https://github.com/fdb/fri3d-wasm-badge.git
```

**Option 2: Clone first, then initialize submodules**
```bash
git clone https://github.com/fdb/fri3d-wasm-badge.git
cd fri3d-wasm-badge
git submodule update --init --recursive
```

### Updating Submodules

If you've already cloned the repository and need to update the submodules:

```bash
git pull --recurse-submodules
```

Or manually update submodules:

```bash
git submodule update --init --recursive
```

## Building

See [AGENTS.md](AGENTS.md) for detailed build instructions and development guidelines.

### Quick Build

```bash
# Build host emulator
cmake -B build
cmake --build build

# Build WASM application
cmake -B build-app -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/app
cmake --build build-app

# Run
./build/host/host_emulator build-app/circle_app.wasm
```

## Running tests

```bash
# Run all visual tests and produce a report
uv run tests/visual/run_visual_tests.py

# Update the "golden master" visual outputs
uv run tests/visual/run_visual_tests.py --update-golden
```

## Project Structure

- `src/host/` - Host emulator (C++ with SDL2)
- `src/app/` - WASM applications (C)
- `libs/` - Git submodules (WAMR, u8g2, lodepng)
- `cmake/` - CMake toolchain files
- `tests/` - Visual regression tests

## Dependencies

- CMake 3.16+
- SDL2
- Zig (for WASM compilation)
- Git submodules:
  - WASM Micro Runtime (WAMR)
  - u8g2 graphics library
  - lodepng

## License

See individual submodule licenses in `libs/` directory.
