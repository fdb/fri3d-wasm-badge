cmake_minimum_required(VERSION 3.16)
project(mandelbrot C)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(mandelbrot main.c)

# WASM linking options for Zig
# Use reactor model and explicitly export functions needed by the host
target_link_options(mandelbrot PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
)
