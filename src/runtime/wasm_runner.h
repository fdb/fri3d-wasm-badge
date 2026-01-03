#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <wasm_export.h>

#include "canvas.h"
#include "random.h"

typedef struct {
    uint8_t* heap_buffer;
    size_t heap_size;
    wasm_module_t module;
    wasm_module_inst_t module_inst;
    wasm_exec_env_t exec_env;

    wasm_function_inst_t func_render;
    wasm_function_inst_t func_on_input;
    wasm_function_inst_t func_set_scene;
    wasm_function_inst_t func_get_scene;
    wasm_function_inst_t func_get_scene_count;

    canvas_t* canvas;
    random_t* random;

    char last_error[256];
    char error_buffer[256];
    bool initialized;
} wasm_runner_t;

typedef void (*wasm_runner_exit_to_launcher_cb)(void* context);
typedef void (*wasm_runner_start_app_cb)(uint32_t app_id, void* context);
typedef uint32_t (*wasm_runner_time_ms_cb)(void* context);

bool wasm_runner_init(wasm_runner_t* runner, size_t heap_size);
void wasm_runner_deinit(wasm_runner_t* runner);

void wasm_runner_set_canvas(wasm_runner_t* runner, canvas_t* canvas);
void wasm_runner_set_random(wasm_runner_t* runner, random_t* random);
void wasm_runner_set_time_provider(wasm_runner_t* runner, wasm_runner_time_ms_cb callback, void* context);
void wasm_runner_set_app_callbacks(wasm_runner_t* runner,
                                   wasm_runner_exit_to_launcher_cb exit_cb,
                                   wasm_runner_start_app_cb start_cb,
                                   void* context);

bool wasm_runner_load_module(wasm_runner_t* runner, const char* path);
bool wasm_runner_load_module_from_memory(wasm_runner_t* runner, const uint8_t* data, size_t size);
void wasm_runner_unload_module(wasm_runner_t* runner);

bool wasm_runner_is_module_loaded(const wasm_runner_t* runner);

void wasm_runner_call_render(wasm_runner_t* runner);
void wasm_runner_call_on_input(wasm_runner_t* runner, uint32_t key, uint32_t type);

uint32_t wasm_runner_get_scene_count(wasm_runner_t* runner);
void wasm_runner_set_scene(wasm_runner_t* runner, uint32_t scene);
uint32_t wasm_runner_get_scene(wasm_runner_t* runner);

bool wasm_runner_has_render_function(const wasm_runner_t* runner);
bool wasm_runner_has_on_input_function(const wasm_runner_t* runner);

const char* wasm_runner_get_last_error(const wasm_runner_t* runner);
