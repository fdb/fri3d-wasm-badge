# WASM toolchain using clang
# No external dependencies required - just clang with WASM target support

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_VERSION 1)
set(CMAKE_SYSTEM_PROCESSOR wasm32)

# Skip compiler checks since we're cross-compiling
set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)

# Use clang directly
find_program(CMAKE_C_COMPILER NAMES clang REQUIRED)

# WASM target flags
set(CMAKE_C_FLAGS_INIT "--target=wasm32 -nostdlib -O3")
set(CMAKE_C_FLAGS "--target=wasm32 -nostdlib -O3" CACHE STRING "" FORCE)

# Linker flags for WASM
set(CMAKE_EXE_LINKER_FLAGS_INIT "-Wl,--no-entry -Wl,--export-all -Wl,--allow-undefined")
set(CMAKE_EXE_LINKER_FLAGS "-Wl,--no-entry -Wl,--export-all -Wl,--allow-undefined" CACHE STRING "" FORCE)

# Output .wasm files
set(CMAKE_EXECUTABLE_SUFFIX ".wasm")

# Disable shared libraries (not supported in WASM)
set(BUILD_SHARED_LIBS OFF)
