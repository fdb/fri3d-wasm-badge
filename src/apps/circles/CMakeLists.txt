cmake_minimum_required(VERSION 3.16)
project(circles C)

# SDK headers
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(circles main.c)

# WASM linking options
target_link_options(circles PRIVATE
    "-Wl,--no-entry"
    "-Wl,--export=render"
    "-Wl,--export=on_input"
    "-nostartfiles"
)
