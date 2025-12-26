# Fri3d Web Emulator
# Emscripten build of the desktop emulator for browsers

cmake_minimum_required(VERSION 3.16)

# This file is used with emcmake/emmake:
# emcmake cmake -B build/web -S src/web
# cmake --build build/web

# SDL2 is provided by Emscripten
set(USE_FLAGS "-s USE_SDL=2")

# lodepng for PNG screenshot output
set(LODEPNG_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/lodepng)

add_executable(fri3d_web
    main.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/../emulator/display_sdl.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/../emulator/input_sdl.cpp
    ${LODEPNG_DIR}/lodepng.cpp
)

target_include_directories(fri3d_web PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../emulator
    ${LODEPNG_DIR}
)

target_link_libraries(fri3d_web PRIVATE
    fri3d_runtime
)

# Emscripten-specific settings
set_target_properties(fri3d_web PROPERTIES
    SUFFIX ".html"
)

target_compile_options(fri3d_web PRIVATE
    ${USE_FLAGS}
)

target_link_options(fri3d_web PRIVATE
    ${USE_FLAGS}
    -s WASM=1
    -s ALLOW_MEMORY_GROWTH=1
    -s TOTAL_MEMORY=67108864  # 64MB
    -s EXIT_RUNTIME=0
    -s ASYNCIFY=1  # For proper main loop handling
    --preload-file ${CMAKE_CURRENT_SOURCE_DIR}/../../build/apps@/apps  # Embed WASM apps
    --shell-file ${CMAKE_CURRENT_SOURCE_DIR}/shell.html
)

# WAMR platform for Emscripten
target_compile_definitions(fri3d_web PRIVATE
    BH_PLATFORM_LINUX  # WAMR uses Linux platform layer with Emscripten
)
