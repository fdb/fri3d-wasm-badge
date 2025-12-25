cmake_minimum_required(VERSION 3.16)
project(circles C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(circles main.c)

# WASM linking options for Zig
# Use -rdynamic to export all non-static symbols, and let WASI handle startup
target_link_options(circles PRIVATE
    "-rdynamic"
)
