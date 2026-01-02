cmake_minimum_required(VERSION 3.16)
project(launcher C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

# Include imgui implementation from SDK
add_executable(launcher
    main.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../../sdk/imgui.c
)

# Output with .wasm extension
set_target_properties(launcher PROPERTIES SUFFIX ".wasm")

# WASM linking options for Zig
# Use reactor model and explicitly export functions needed by the host
target_link_options(launcher PRIVATE
    "-mexec-model=reactor"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
)
