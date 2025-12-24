cmake_minimum_required(VERSION 3.16)
project(fri3d-wasm-badge)

set(CMAKE_CXX_STANDARD 14)

# Submodules
add_subdirectory(libs/u8g2)
# WAMR is a bit more complex, often best to include just what we need or use its provided CMake if compatible. 
# For a host build, we usually want to build 'vmlib' (the core runtime).
# We will tackle WAMR inclusion in the host specific CMake or here.
# For now, let's keep it clean and handle WAMR in src/host if possible, or here if it needs global flags.

# WAMR Config
set(WAMR_BUILD_INTERP 1 CACHE BOOL "Enable Interpreter")
set(WAMR_BUILD_FAST_INTERP 1 CACHE BOOL "Enable Fast Interpreter")
set(WAMR_BUILD_JIT 0 CACHE BOOL "Disable JIT for simplicity on host for now")
set(WAMR_BUILD_LIBC_BUILTIN 1 CACHE BOOL "Enable builtin libc")
set(WAMR_BUILD_PLATFORM "linux" CACHE STRING "Platform") # 'linux' works for generic posix/mac usually

# Start with Host Emulator
add_subdirectory(src/host)

# We will handle WASM app build separately or as a custom command because it needs a different toolchain (wasi-sdk or generic wasm clang)
