# Fri3d Runtime Library
# Shared code for canvas, input, WASM execution, and app management

if(EMSCRIPTEN)
    add_library(fri3d_runtime STATIC
        canvas.c
        input.c
        random.c
    )

    target_include_directories(fri3d_runtime PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc
    )

    target_link_libraries(fri3d_runtime PUBLIC
        u8g2
    )
else()
    add_library(fri3d_runtime STATIC
        canvas.c
        input.c
        random.c
        wasm_runner.c
        trace.c
        app_manager.c
    )

    target_include_directories(fri3d_runtime PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/u8g2/csrc
    )

    set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/wasm-micro-runtime)
    target_include_directories(fri3d_runtime PUBLIC
        ${WAMR_ROOT_DIR}/core/iwasm/include
    )

    target_link_libraries(fri3d_runtime PUBLIC
        u8g2
        vmlib
    )
endif()

# C11 required
set_target_properties(fri3d_runtime PROPERTIES
    C_STANDARD 11
    C_STANDARD_REQUIRED YES
)
