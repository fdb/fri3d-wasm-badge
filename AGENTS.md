# FRI3D WASM Badge - Agent Guidelines

## Build Commands

### Prerequisites: Git Submodules

This project uses git submodules. Before building, ensure submodules are initialized:

**After cloning the repository:**
```bash
git submodule update --init --recursive
```

**When pulling updates:**
```bash
git pull --recurse-submodules
```

Or manually update submodules:
```bash
git submodule update --init --recursive
```

### Build Host Emulator
```bash
cmake -B build
cmake --build build
```

### Build WASM Application
```bash
cmake -B build-app -DCMAKE_TOOLCHAIN_FILE=cmake/toolchain-wasm.cmake -S src/app
cmake --build build-app
```

### Run Emulator
```bash
./build/host/host_emulator build-app/circle_app.wasm
```

### Clean Build
```bash
rm -rf build build-app
```

## Testing

This project does not currently have automated tests. Manual testing:
1. Build host emulator
2. Build WASM app
3. Run emulator with WASM file
4. Verify display output and input handling (Arrow keys for movement, Z/X for A/B buttons)

## Code Style Guidelines

### C++ (Host Code - src/host/)
- **Standard**: C++14 (CMAKE_CXX_STANDARD=14)
- **Indentation**: 4 spaces
- **Line length**: Prefer under 80 characters
- **Brace style**: Opening brace on same line for control statements, on new line for functions
- **Naming**: 
  - Variables/functions: `snake_case`
  - Native functions: `native_` prefix
  - Classes/types: `PascalCase` (rarely used)
- **Error handling**: Check return values, print errors to `std::cerr` before returning non-zero

### C (WASM App Code - src/app/)
- **Indentation**: 4 spaces (follows u8g2 conventions)
- **Naming**: 
  - Functions: `snake_case`
  - WASM imports: `host_` prefix
  - Use `WASM_IMPORT()` macro for imported functions
- **Globals**: Keep to minimum, mark `static` when possible

### CMakeLists.txt
- **Minimum version**: 3.16
- **Naming**: Variables in uppercase with underscores
- **Structure**: 
  - Use `find_package()` for dependencies (SDL2)
  - Use `add_subdirectory()` for submodules
  - For WASM, use toolchain file cmake/toolchain-wasm.cmake

### Imports and Includes
- **C++**: System includes first (`#include <SDL.h>`), then project includes
- **C**: System headers first (`#include <stdint.h>`), then library headers (`#include <u8g2.h>`)

### WASM-Specific Conventions
- Export `on_tick()` function as main entry point
- Use `__attribute__((import_module("env"), import_name(name)))` for host imports
- For u8g2: use full buffer mode (`u8g2_Setup_ssd1306_128x64_noname_f`)
- Buffer size: 1024 bytes for 128x64 monochrome display

### Error Handling
- **Host code**: Validate WASM memory addresses with `wasm_runtime_validate_app_addr()`
- Check for exceptions after WASM calls with `wasm_runtime_get_exception()`
- Print descriptive error messages to stderr before returning error codes

### Formatting
- Project inherits WAMR's .clang-format (Mozilla style, 4-space indent, 80-char limit)
- Run `clang-format -i <file>` to format if needed

### Platform Notes
- macOS: SDL2 installation via Homebrew required
- WASM compilation uses Zig toolchain via `cmake/toolchain-wasm.cmake`
- WAMR runtime configured for interpreter mode (no JIT)
