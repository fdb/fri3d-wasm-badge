#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "canvas.h"
#include "input.h"
#include "random.h"
#include "wasm_runner.h"

typedef struct {
    uint32_t id;
    char* name;
    char* path;
} app_entry_t;

typedef enum {
    app_manager_request_none = 0,
    app_manager_request_exit_to_launcher,
    app_manager_request_start_app,
} app_manager_request_t;

typedef struct {
    canvas_t* canvas;
    random_t* random;
    wasm_runner_t wasm_runner;

    app_entry_t* apps;
    size_t app_count;
    size_t app_capacity;
    bool in_launcher;
    char* launcher_path;
    app_manager_request_t pending_request;
    uint32_t pending_app_id;

    char last_error[256];
} app_manager_t;

bool app_manager_init(app_manager_t* manager, canvas_t* canvas, random_t* random);
void app_manager_deinit(app_manager_t* manager);

bool app_manager_add_app(app_manager_t* manager, const char* name, const char* path);
void app_manager_clear_apps(app_manager_t* manager);

void app_manager_set_launcher_path(app_manager_t* manager, const char* path);
void app_manager_show_launcher(app_manager_t* manager);
bool app_manager_launch_app_by_path(app_manager_t* manager, const char* path);

void app_manager_render(app_manager_t* manager);
void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type);

const char* app_manager_get_last_error(const app_manager_t* manager);

wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager);
