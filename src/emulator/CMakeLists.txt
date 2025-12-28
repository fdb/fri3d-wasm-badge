# Fri3d Emulator
# Desktop SDL-based emulator for running WASM apps

find_package(SDL2 REQUIRED)

# lodepng for PNG screenshot output
# Note: lodepng.cpp is actually C code and can be compiled as C
set(LODEPNG_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/lodepng)

add_executable(fri3d_emulator
    main.c
    display_sdl.c
    input_sdl.c
    lodepng_c.c
)

target_include_directories(fri3d_emulator PRIVATE
    ${SDL2_INCLUDE_DIRS}
    ${LODEPNG_DIR}
)

target_link_libraries(fri3d_emulator PRIVATE
    ${SDL2_LIBRARIES}
    fri3d_runtime
)

# Platform detection for WAMR
if(APPLE)
    target_compile_definitions(fri3d_emulator PRIVATE BH_PLATFORM_DARWIN)
elseif(UNIX)
    target_compile_definitions(fri3d_emulator PRIVATE BH_PLATFORM_LINUX)
endif()
