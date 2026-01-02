# Fri3d Emulator
# Desktop SDL-based emulator for running WASM apps

find_package(SDL2 REQUIRED)

set(STB_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/stb)

add_executable(fri3d_emulator
    main.c
    display_sdl.c
    input_sdl.c
)

target_include_directories(fri3d_emulator PRIVATE
    ${SDL2_INCLUDE_DIRS}
    ${STB_DIR}
)

target_link_libraries(fri3d_emulator PRIVATE
    ${SDL2_LIBRARIES}
    fri3d_runtime
)

if(APPLE)
    target_compile_definitions(fri3d_emulator PRIVATE BH_PLATFORM_DARWIN)
elseif(UNIX)
    target_compile_definitions(fri3d_emulator PRIVATE BH_PLATFORM_LINUX)
endif()

set_target_properties(fri3d_emulator PROPERTIES
    C_STANDARD 11
    C_STANDARD_REQUIRED YES
)
