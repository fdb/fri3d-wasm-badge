
# Find SDL2
find_package(SDL2 REQUIRED)

set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/wasm-micro-runtime)

# Include WAMR
# We need to set some options BEFORE including it if we want to override defaults not handled by cache
# set(WAMR_BUILD_PLATFORM "linux") # Let WAMR detect Darwin
set(WAMR_BUILD_SIMD 0) # Disable SIMD to avoid assembly issues on Mac for now
add_compile_definitions(BH_PLATFORM_DARWIN)



add_subdirectory(${WAMR_ROOT_DIR} ${CMAKE_CURRENT_BINARY_DIR}/wamr)

add_executable(host_emulator main.cpp)

target_include_directories(host_emulator PRIVATE 
    ${SDL2_INCLUDE_DIRS}
    ${WAMR_ROOT_DIR}/core/iwasm/include
)

target_link_libraries(host_emulator PRIVATE 
    ${SDL2_LIBRARIES}
    vmlib
)
