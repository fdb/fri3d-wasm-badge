#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "canvas.h"
#include "input.h"
#include "random.h"
#include "wasm_runner.h"

typedef struct {
    char* name;
    char* path;
} app_entry_t;

typedef struct {
    canvas_t* canvas;
    random_t* random;
    wasm_runner_t wasm_runner;

    app_entry_t* apps;
    size_t app_count;
    size_t app_capacity;
    size_t selected_index;
    size_t scroll_offset;
    bool in_launcher;

    char last_error[256];
} app_manager_t;

bool app_manager_init(app_manager_t* manager, canvas_t* canvas, random_t* random);
void app_manager_deinit(app_manager_t* manager);

bool app_manager_add_app(app_manager_t* manager, const char* name, const char* path);
void app_manager_clear_apps(app_manager_t* manager);
size_t app_manager_get_app_count(const app_manager_t* manager);

void app_manager_show_launcher(app_manager_t* manager);

bool app_manager_launch_app(app_manager_t* manager, size_t index);
bool app_manager_launch_app_by_path(app_manager_t* manager, const char* path);

bool app_manager_is_in_launcher(const app_manager_t* manager);
bool app_manager_is_app_running(const app_manager_t* manager);

void app_manager_render(app_manager_t* manager);
void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type);

const char* app_manager_get_last_error(const app_manager_t* manager);

wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager);
