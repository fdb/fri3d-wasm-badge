
# WASM App build

cmake_minimum_required(VERSION 3.16)
project(circle_app)

# u8g2
# u8g2 is in ../../libs/u8g2
# We need to compile u8g2 sources with the app.
# u8g2 uses a lot of files. Best to glob them or standard style.
file(GLOB U8G2_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc/*.c")

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc)

add_executable(circle_app main.c ${U8G2_SOURCES})

# We need to export on_tick
target_link_options(circle_app PRIVATE 
    "-Wl,--no-entry"
    "-Wl,--export=on_tick"
)
set_target_properties(circle_app PROPERTIES Suffix ".wasm")
