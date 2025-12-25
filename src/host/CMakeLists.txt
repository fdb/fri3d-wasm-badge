
# Find SDL2
find_package(SDL2 REQUIRED)

set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/wasm-micro-runtime)

# Include WAMR
# We need to set some options BEFORE including it if we want to override defaults not handled by cache
set(WAMR_BUILD_SIMD 0) # Disable SIMD to avoid assembly issues

# Detect platform
if(APPLE)
    add_compile_definitions(BH_PLATFORM_DARWIN)
elseif(UNIX)
    add_compile_definitions(BH_PLATFORM_LINUX)
endif()



add_subdirectory(${WAMR_ROOT_DIR} ${CMAKE_CURRENT_BINARY_DIR}/wamr)

add_executable(host_emulator main.cpp)

target_include_directories(host_emulator PRIVATE
    ${SDL2_INCLUDE_DIRS}
    ${WAMR_ROOT_DIR}/core/iwasm/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc
)

target_link_libraries(host_emulator PRIVATE
    ${SDL2_LIBRARIES}
    vmlib
    u8g2
)
