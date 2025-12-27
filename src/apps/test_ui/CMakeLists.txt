cmake_minimum_required(VERSION 3.16)
project(test_ui C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

# Include imgui implementation from SDK
add_executable(test_ui
    main.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../../sdk/imgui.c
)

# Output with .wasm extension
set_target_properties(test_ui PROPERTIES SUFFIX ".wasm")

# WASM linking options for Zig
# Explicitly export functions needed by the host emulator
target_link_options(test_ui PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
    "-Wl,--export=get_scene"
    "-Wl,--export=set_scene"
    "-Wl,--export=get_scene_count"
)
