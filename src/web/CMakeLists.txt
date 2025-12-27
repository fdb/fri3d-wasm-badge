# Fri3d Web Emulator
# Emscripten build using browser's native WebAssembly (no WAMR)

# SDL2 is provided by Emscripten (no space after -s for modern Emscripten)
set(USE_FLAGS "-sUSE_SDL=2")

# Paths
set(EMULATOR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../emulator)
set(LODEPNG_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/lodepng)

add_executable(fri3d_web
    main.cpp
    # Emulator display/input
    ${EMULATOR_DIR}/display_sdl.cpp
    ${EMULATOR_DIR}/input_sdl.cpp
    # lodepng
    ${LODEPNG_DIR}/lodepng.cpp
)

target_include_directories(fri3d_web PRIVATE
    ${EMULATOR_DIR}
    ${LODEPNG_DIR}
)

# Link against fri3d_runtime (minimal version for Emscripten)
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

# Exported functions for JavaScript to call
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
    -sTOTAL_MEMORY=67108864  # 64MB
    -sEXIT_RUNTIME=0
    -sEXPORTED_FUNCTIONS=[${EXPORTED_FUNCTIONS_STR}]
    -sEXPORTED_RUNTIME_METHODS=["ccall","cwrap","allocateUTF8","UTF8ToString"]
    --preload-file ${CMAKE_CURRENT_SOURCE_DIR}/../../build/apps@/apps
    --shell-file ${CMAKE_CURRENT_SOURCE_DIR}/shell.html
)

# C++14 required
target_compile_features(fri3d_web PUBLIC cxx_std_14)
