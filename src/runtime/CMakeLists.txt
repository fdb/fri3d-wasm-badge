# Fri3d Runtime Library
# Shared code for canvas, input, WASM execution, and app management

add_library(fri3d_runtime STATIC
    canvas.cpp
    input.cpp
    random.cpp
    wasm_runner.cpp
    app_manager.cpp
)

target_include_directories(fri3d_runtime PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc
)

# WAMR headers
set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/wasm-micro-runtime)
target_include_directories(fri3d_runtime PUBLIC
    ${WAMR_ROOT_DIR}/core/iwasm/include
)

# Link against u8g2 and WAMR
target_link_libraries(fri3d_runtime PUBLIC
    u8g2
    vmlib
)

# C++14 required
target_compile_features(fri3d_runtime PUBLIC cxx_std_14)
