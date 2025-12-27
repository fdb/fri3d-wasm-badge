# Fri3d Emulator
# Desktop SDL-based emulator for running WASM apps

find_package(SDL2 REQUIRED)

# lodepng for PNG screenshot output
set(LODEPNG_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/lodepng)

add_executable(fri3d_emulator
    main.cpp
    display_sdl.cpp
    input_sdl.cpp
    ${LODEPNG_DIR}/lodepng.cpp
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
