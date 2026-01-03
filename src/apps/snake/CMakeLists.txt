cmake_minimum_required(VERSION 3.16)
project(snake C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(snake main.c)

# Output with .wasm extension
set_target_properties(snake PROPERTIES SUFFIX ".wasm")

# WASM linking options for Zig
# Use reactor model and explicitly export functions needed by the host
target_link_options(snake PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
)
