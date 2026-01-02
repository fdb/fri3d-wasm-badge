#include "app_manager.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void app_manager_render_launcher_error(app_manager_t* manager);
static void app_manager_process_pending(app_manager_t* manager);
static bool app_manager_launch_app_by_id(app_manager_t* manager, uint32_t app_id);
static void app_manager_request_exit_to_launcher_cb(void* context);
static void app_manager_request_start_app_cb(uint32_t app_id, void* context);

static char* app_manager_strdup(const char* value) {
    if (!value) {
        return NULL;
    }
    size_t len = strlen(value);
    char* copy = (char*)malloc(len + 1);
    if (!copy) {
        return NULL;
    }
    memcpy(copy, value, len + 1);
    return copy;
}

static void app_manager_set_error(app_manager_t* manager, const char* message) {
    if (!manager || !message) {
        return;
    }
    snprintf(manager->last_error, sizeof(manager->last_error), "%s", message);
}

bool app_manager_init(app_manager_t* manager, canvas_t* canvas, random_t* random) {
    if (!manager) {
        return false;
    }

    memset(manager, 0, sizeof(*manager));

    manager->canvas = canvas;
    manager->random = random;
    manager->in_launcher = true;
    manager->launcher_path = NULL;
    manager->pending_request = app_manager_request_none;
    manager->pending_app_id = 0;

    if (!wasm_runner_init(&manager->wasm_runner, 10 * 1024 * 1024)) {
        app_manager_set_error(manager, wasm_runner_get_last_error(&manager->wasm_runner));
        return false;
    }

    wasm_runner_set_canvas(&manager->wasm_runner, canvas);
    wasm_runner_set_random(&manager->wasm_runner, random);
    wasm_runner_set_app_callbacks(&manager->wasm_runner,
                                  app_manager_request_exit_to_launcher_cb,
                                  app_manager_request_start_app_cb,
                                  manager);

    return true;
}

void app_manager_deinit(app_manager_t* manager) {
    if (!manager) {
        return;
    }

    app_manager_clear_apps(manager);
    free(manager->launcher_path);
    manager->launcher_path = NULL;
    wasm_runner_deinit(&manager->wasm_runner);
}

bool app_manager_add_app(app_manager_t* manager, const char* name, const char* path) {
    if (!manager || !name || !path) {
        return false;
    }

    if (manager->app_count >= manager->app_capacity) {
        size_t new_capacity = manager->app_capacity == 0 ? 4 : manager->app_capacity * 2;
        app_entry_t* new_apps = (app_entry_t*)realloc(manager->apps, new_capacity * sizeof(app_entry_t));
        if (!new_apps) {
            app_manager_set_error(manager, "Failed to grow app list");
            return false;
        }
        manager->apps = new_apps;
        manager->app_capacity = new_capacity;
    }

    char* name_copy = app_manager_strdup(name);
    char* path_copy = app_manager_strdup(path);
    if (!name_copy || !path_copy) {
        free(name_copy);
        free(path_copy);
        app_manager_set_error(manager, "Failed to allocate app entry");
        return false;
    }

    manager->apps[manager->app_count].name = name_copy;
    manager->apps[manager->app_count].path = path_copy;
    manager->apps[manager->app_count].id = (uint32_t)(manager->app_count + 1);
    manager->app_count++;

    return true;
}

void app_manager_clear_apps(app_manager_t* manager) {
    if (!manager) {
        return;
    }

    for (size_t i = 0; i < manager->app_count; i++) {
        free(manager->apps[i].name);
        free(manager->apps[i].path);
    }
    free(manager->apps);
    manager->apps = NULL;
    manager->app_count = 0;
    manager->app_capacity = 0;
}

void app_manager_set_launcher_path(app_manager_t* manager, const char* path) {
    if (!manager) {
        return;
    }

    char* path_copy = app_manager_strdup(path);
    if (path && !path_copy) {
        app_manager_set_error(manager, "Failed to allocate launcher path");
        return;
    }

    free(manager->launcher_path);
    manager->launcher_path = path_copy;
}

void app_manager_show_launcher(app_manager_t* manager) {
    if (!manager) {
        return;
    }
    wasm_runner_unload_module(&manager->wasm_runner);
    manager->in_launcher = true;

    if (!manager->launcher_path) {
        app_manager_set_error(manager, "Launcher path not set");
        return;
    }

    if (!wasm_runner_load_module(&manager->wasm_runner, manager->launcher_path)) {
        app_manager_set_error(manager, wasm_runner_get_last_error(&manager->wasm_runner));
    }
}

bool app_manager_launch_app_by_path(app_manager_t* manager, const char* path) {
    if (!manager || !path) {
        return false;
    }

    if (!wasm_runner_load_module(&manager->wasm_runner, path)) {
        app_manager_set_error(manager, wasm_runner_get_last_error(&manager->wasm_runner));
        return false;
    }

    manager->in_launcher = false;
    return true;
}

void app_manager_render(app_manager_t* manager) {
    if (!manager) {
        return;
    }

    if (manager->in_launcher) {
        if (wasm_runner_is_module_loaded(&manager->wasm_runner)) {
            wasm_runner_call_render(&manager->wasm_runner);
        } else {
            app_manager_render_launcher_error(manager);
        }
    } else {
        wasm_runner_call_render(&manager->wasm_runner);
    }

    app_manager_process_pending(manager);
}

void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type) {
    if (!manager) {
        return;
    }

    if (manager->in_launcher) {
        if (wasm_runner_is_module_loaded(&manager->wasm_runner)) {
            wasm_runner_call_on_input(&manager->wasm_runner, (uint32_t)key, (uint32_t)type);
        }
    } else {
        wasm_runner_call_on_input(&manager->wasm_runner, (uint32_t)key, (uint32_t)type);
    }

    app_manager_process_pending(manager);
}

const char* app_manager_get_last_error(const app_manager_t* manager) {
    return manager ? manager->last_error : "";
}

wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager) {
    return manager ? &manager->wasm_runner : NULL;
}

static void app_manager_render_launcher_error(app_manager_t* manager) {
    if (!manager || !manager->canvas) {
        return;
    }

    canvas_clear(manager->canvas);
    canvas_set_color(manager->canvas, ColorBlack);
    canvas_set_font(manager->canvas, FontPrimary);
    canvas_draw_str(manager->canvas, 2, 12, "Error loading launcher");
}

static bool app_manager_launch_app_by_id(app_manager_t* manager, uint32_t app_id) {
    if (!manager) {
        return false;
    }

    if (app_id == 0) {
        app_manager_show_launcher(manager);
        return true;
    }

    for (size_t i = 0; i < manager->app_count; i++) {
        if (manager->apps[i].id == app_id) {
            return app_manager_launch_app_by_path(manager, manager->apps[i].path);
        }
    }

    app_manager_set_error(manager, "Invalid app id");
    return false;
}

static void app_manager_process_pending(app_manager_t* manager) {
    if (!manager || manager->pending_request == app_manager_request_none) {
        return;
    }

    app_manager_request_t request = manager->pending_request;
    uint32_t app_id = manager->pending_app_id;

    manager->pending_request = app_manager_request_none;
    manager->pending_app_id = 0;

    if (request == app_manager_request_exit_to_launcher) {
        app_manager_show_launcher(manager);
    } else if (request == app_manager_request_start_app) {
        app_manager_launch_app_by_id(manager, app_id);
    }
}

static void app_manager_request_exit_to_launcher_cb(void* context) {
    app_manager_t* manager = (app_manager_t*)context;
    if (!manager) {
        return;
    }
    manager->pending_request = app_manager_request_exit_to_launcher;
}

static void app_manager_request_start_app_cb(uint32_t app_id, void* context) {
    app_manager_t* manager = (app_manager_t*)context;
    if (!manager) {
        return;
    }
    manager->pending_request = app_manager_request_start_app;
    manager->pending_app_id = app_id;
}
