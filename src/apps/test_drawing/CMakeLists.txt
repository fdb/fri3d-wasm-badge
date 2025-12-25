cmake_minimum_required(VERSION 3.16)
project(test_drawing C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(test_drawing main.c)

# WASM linking options for Zig
# Use -rdynamic to export all non-static symbols, and let WASI handle startup
target_link_options(test_drawing PRIVATE
    "-rdynamic"
)
