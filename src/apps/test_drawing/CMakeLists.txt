cmake_minimum_required(VERSION 3.16)
project(test_drawing C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(test_drawing main.c)

# Output with .wasm extension
set_target_properties(test_drawing PROPERTIES SUFFIX ".wasm")

# WASM linking options for Zig
# Explicitly export functions needed by the host emulator
target_link_options(test_drawing PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
    "-Wl,--export=get_scene"
    "-Wl,--export=set_scene"
    "-Wl,--export=get_scene_count"
)
