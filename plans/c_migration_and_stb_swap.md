# C Migration + stb_image_write Swap (Planned Work)

## Goal
Move the project from mixed C/C++ to C-only with consistent snake_case naming, and replace PNG encoding via lodepng with stb_image_write headers. Ensure visual regression tests still pass after the change.

## Planned Steps
1) Inventory existing C++ sources and APIs\n\
   - Locate all `.cpp`/C++ headers in runtime, emulator, web, and firmware.\n\
   - Identify C++-only constructs (classes, namespaces, STL usage) and map them to C equivalents.

2) Convert runtime/emulator/web/firmware sources to C\n\
   - Rename `.cpp` files to `.c`.\n\
   - Replace classes with structs + functions.\n\
   - Remove namespaces and update includes.\n\
   - Convert `std::string`/`std::vector` to C memory management.\n\
   - Implement a C-compatible RNG equivalent (MT19937) to preserve visuals.

3) Normalize naming to snake_case\n\
   - Update runtime/public APIs to `snake_case` types and functions.\n\
   - Update SDK headers (canvas/input/imgui/random) to match.\n\
   - Update in-tree apps to use the new enum/type names (e.g. `ColorBlack`, `input_key_up`, `input_type_press`).

4) Update build system for C-only\n\
   - Adjust `CMakeLists.txt` to `project(... C)` and set C standard.\n\
   - Update runtime/emulator/web CMake targets to compile `.c` sources only.\n\
   - Ensure WAMR/u8g2 integration remains intact.

5) Replace lodepng with stb_image_write\n\
   - Add `libs/stb/stb_image_write.h` include path to emulator/web targets.\n\
   - Update screenshot encoding in `display_sdl.c` to use `stbi_write_png`.\n\
   - Remove lodepng sources from builds.

6) Remove lodepng submodule\n\
   - Remove entry from `.gitmodules`.\n\
   - Remove submodule from git index and delete `libs/lodepng` directory.\n\
   - Clean any lingering references to lodepng in docs and build files.

7) Update documentation\n\
   - Update README to reflect stb usage and removal of lodepng.\n\
   - Update any firmware/emulator docs referencing `*.cpp` filenames.

8) Validate with visual tests\n\
   - Run `uv run tests/visual/run_visual_tests.py` and fix regressions if any.

## Expected Files to Touch (high level)
- C core/runtime: `src/runtime/*.{c,h}`
- Emulator: `src/emulator/*.{c,h}`
- Web: `src/web/main.c`, `src/web/CMakeLists.txt`
- Firmware: `src/firmware/src/*.c`, `src/firmware/include/input_gpio.h`
- SDK/apps: `src/sdk/*`, `src/apps/*/main.c`
- Build: `CMakeLists.txt`, `src/runtime/CMakeLists.txt`, `src/emulator/CMakeLists.txt`
- Docs: `README.md`, `src/firmware/README.md`
- Submodules: `.gitmodules` (remove lodepng)

## Test Plan
- `uv run tests/visual/run_visual_tests.py`
