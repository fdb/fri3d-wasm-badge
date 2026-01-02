# Fri3d Web Emulator
# Emscripten build using browser's native WebAssembly (no WAMR)

set(USE_FLAGS "-sUSE_SDL=2")

set(EMULATOR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../emulator)
set(STB_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/stb)

add_executable(fri3d_web
    main.c
    ${EMULATOR_DIR}/display_sdl.c
    ${EMULATOR_DIR}/input_sdl.c
)

target_include_directories(fri3d_web PRIVATE
    ${EMULATOR_DIR}
    ${STB_DIR}
)

target_link_libraries(fri3d_web PRIVATE
    fri3d_runtime
)

set_target_properties(fri3d_web PROPERTIES
    SUFFIX ".html"
)

target_compile_options(fri3d_web PRIVATE
    ${USE_FLAGS}
)

set(EXPORTED_FUNCTIONS
    "_main"
    "_js_canvas_clear"
    "_js_canvas_width"
    "_js_canvas_height"
    "_js_canvas_set_color"
    "_js_canvas_set_font"
    "_js_canvas_draw_dot"
    "_js_canvas_draw_line"
    "_js_canvas_draw_frame"
    "_js_canvas_draw_box"
    "_js_canvas_draw_rframe"
    "_js_canvas_draw_rbox"
    "_js_canvas_draw_circle"
    "_js_canvas_draw_disc"
    "_js_canvas_draw_str"
    "_js_canvas_string_width"
    "_js_random_seed"
    "_js_random_get"
    "_js_random_range"
    "_js_display_flush"
    "_js_poll_input"
    "_js_has_input_event"
    "_js_get_input_event"
    "_js_get_time_ms"
)
string(REPLACE ";" "," EXPORTED_FUNCTIONS_STR "${EXPORTED_FUNCTIONS}")

target_link_options(fri3d_web PRIVATE
    ${USE_FLAGS}
    -sWASM=1
    -sALLOW_MEMORY_GROWTH=1
    -sTOTAL_MEMORY=67108864
    -sEXIT_RUNTIME=0
    -sEXPORTED_FUNCTIONS=[${EXPORTED_FUNCTIONS_STR}]
    -sEXPORTED_RUNTIME_METHODS=["ccall","cwrap","allocateUTF8","UTF8ToString"]
    --preload-file ${CMAKE_CURRENT_SOURCE_DIR}/../../build/apps@/apps
    --shell-file ${CMAKE_CURRENT_SOURCE_DIR}/shell.html
)

set_target_properties(fri3d_web PROPERTIES
    C_STANDARD 11
    C_STANDARD_REQUIRED YES
)
