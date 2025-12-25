cmake_minimum_required(VERSION 3.16)
project(mandelbrot C)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../sdk)

add_executable(mandelbrot main.c)

target_link_options(mandelbrot PRIVATE
    "-rdynamic"
)
