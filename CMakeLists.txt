cmake_minimum_required(VERSION 3.16)
project(fri3d-wasm-badge C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Build options
option(BUILD_EMULATOR "Build the desktop SDL emulator" ON)
option(BUILD_WEB "Build the web/Emscripten version" OFF)
option(BUILD_TRACE_TESTS "Build trace test harness" OFF)

# ============================================================================
# External Libraries
# ============================================================================

add_subdirectory(libs/u8g2)

if(NOT EMSCRIPTEN)
    set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/libs/wasm-micro-runtime)

    set(WAMR_BUILD_INTERP 1 CACHE BOOL "Enable Interpreter")
    set(WAMR_BUILD_FAST_INTERP 1 CACHE BOOL "Enable Fast Interpreter")
    set(WAMR_BUILD_JIT 0 CACHE BOOL "Disable JIT")
    set(WAMR_BUILD_AOT 0 CACHE BOOL "Disable AOT")
    set(WAMR_BUILD_LIBC_BUILTIN 1 CACHE BOOL "Enable builtin libc")
    set(WAMR_BUILD_SIMD 0 CACHE BOOL "Disable SIMD")

    if(APPLE)
        set(WAMR_BUILD_PLATFORM "darwin" CACHE STRING "Platform")
    else()
        set(WAMR_BUILD_PLATFORM "linux" CACHE STRING "Platform")
    endif()

    add_subdirectory(${WAMR_ROOT_DIR} ${CMAKE_CURRENT_BINARY_DIR}/wamr)
endif()

# ============================================================================
# Shared Runtime Library
# ============================================================================

add_subdirectory(src/runtime)

# ============================================================================
# Platform-Specific Builds
# ============================================================================

if(BUILD_EMULATOR)
    add_subdirectory(src/emulator)
endif()

if(BUILD_WEB)
    add_subdirectory(src/web)
endif()

if(BUILD_TRACE_TESTS)
    target_compile_definitions(fri3d_runtime PUBLIC FRD_TRACE)
    add_subdirectory(tests/trace/harness)
endif()

# Note: WASM apps are built separately with a WASM toolchain
# See cmake/toolchain-wasm.cmake and build_all.sh
