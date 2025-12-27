cmake_minimum_required(VERSION 3.16)
project(circles C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(circles main.c)

# Output with .wasm extension
set_target_properties(circles PROPERTIES SUFFIX ".wasm")

# WASM linking options for Zig
# Use reactor model and explicitly export functions needed by the host
target_link_options(circles PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
)
