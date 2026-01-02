#include "app_manager.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define APP_MANAGER_VISIBLE_ITEMS 4
#define APP_MANAGER_ITEM_HEIGHT 14
#define APP_MANAGER_START_Y 12
#define APP_MANAGER_TEXT_X 16
#define APP_MANAGER_CIRCLE_X 6
#define APP_MANAGER_CIRCLE_RADIUS 3

static void app_manager_render_launcher(app_manager_t* manager);
static void app_manager_launcher_input(app_manager_t* manager, input_key_t key, input_type_t type);

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

    if (!wasm_runner_init(&manager->wasm_runner, 10 * 1024 * 1024)) {
        app_manager_set_error(manager, wasm_runner_get_last_error(&manager->wasm_runner));
        return false;
    }

    wasm_runner_set_canvas(&manager->wasm_runner, canvas);
    wasm_runner_set_random(&manager->wasm_runner, random);

    return true;
}

void app_manager_deinit(app_manager_t* manager) {
    if (!manager) {
        return;
    }

    app_manager_clear_apps(manager);
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
    manager->selected_index = 0;
    manager->scroll_offset = 0;
}

size_t app_manager_get_app_count(const app_manager_t* manager) {
    return manager ? manager->app_count : 0;
}

void app_manager_show_launcher(app_manager_t* manager) {
    if (!manager) {
        return;
    }
    wasm_runner_unload_module(&manager->wasm_runner);
    manager->in_launcher = true;
}

bool app_manager_launch_app(app_manager_t* manager, size_t index) {
    if (!manager) {
        return false;
    }

    if (index >= manager->app_count) {
        app_manager_set_error(manager, "Invalid app index");
        return false;
    }

    return app_manager_launch_app_by_path(manager, manager->apps[index].path);
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

bool app_manager_is_in_launcher(const app_manager_t* manager) {
    return manager ? manager->in_launcher : true;
}

bool app_manager_is_app_running(const app_manager_t* manager) {
    if (!manager) {
        return false;
    }
    return !manager->in_launcher && wasm_runner_is_module_loaded(&manager->wasm_runner);
}

void app_manager_render(app_manager_t* manager) {
    if (!manager) {
        return;
    }

    if (manager->in_launcher) {
        app_manager_render_launcher(manager);
    } else {
        wasm_runner_call_render(&manager->wasm_runner);
    }
}

void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type) {
    if (!manager) {
        return;
    }

    if (manager->in_launcher) {
        app_manager_launcher_input(manager, key, type);
    } else {
        if (type == input_type_press || type == input_type_release) {
            wasm_runner_call_on_input(&manager->wasm_runner, (uint32_t)key, (uint32_t)type);
        }
    }
}

const char* app_manager_get_last_error(const app_manager_t* manager) {
    return manager ? manager->last_error : "";
}

wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager) {
    return manager ? &manager->wasm_runner : NULL;
}

static void app_manager_render_launcher(app_manager_t* manager) {
    if (!manager || !manager->canvas) {
        return;
    }

    canvas_clear(manager->canvas);
    canvas_set_color(manager->canvas, color_black);
    canvas_set_font(manager->canvas, font_primary);

    canvas_draw_str(manager->canvas, 2, 10, "Fri3d Apps");
    canvas_draw_line(manager->canvas, 0, 12, 127, 12);

    if (manager->app_count == 0) {
        canvas_draw_str(manager->canvas, 2, 30, "No apps found");
        return;
    }

    size_t start_idx = manager->scroll_offset;
    size_t end_idx = start_idx + APP_MANAGER_VISIBLE_ITEMS;
    if (end_idx > manager->app_count) {
        end_idx = manager->app_count;
    }

    for (size_t i = start_idx; i < end_idx; i++) {
        int y = APP_MANAGER_START_Y + (int)((i - start_idx) + 1) * APP_MANAGER_ITEM_HEIGHT;

        if (i == manager->selected_index) {
            canvas_draw_disc(manager->canvas, APP_MANAGER_CIRCLE_X, y - 3, APP_MANAGER_CIRCLE_RADIUS);
        } else {
            canvas_draw_circle(manager->canvas, APP_MANAGER_CIRCLE_X, y - 3, APP_MANAGER_CIRCLE_RADIUS);
        }

        canvas_draw_str(manager->canvas, APP_MANAGER_TEXT_X, y, manager->apps[i].name);
    }

    if (manager->scroll_offset > 0) {
        canvas_draw_str(manager->canvas, 120, 20, "^");
    }
    if (end_idx < manager->app_count) {
        canvas_draw_str(manager->canvas, 120, 60, "v");
    }
}

static void app_manager_launcher_input(app_manager_t* manager, input_key_t key, input_type_t type) {
    if (!manager) {
        return;
    }

    if (type != input_type_press && type != input_type_short_press) {
        return;
    }

    if (manager->app_count == 0) {
        return;
    }

    switch (key) {
        case input_key_up:
            if (manager->selected_index > 0) {
                manager->selected_index--;
                if (manager->selected_index < manager->scroll_offset) {
                    manager->scroll_offset = manager->selected_index;
                }
            }
            break;
        case input_key_down:
            if (manager->selected_index + 1 < manager->app_count) {
                manager->selected_index++;
                if (manager->selected_index >= manager->scroll_offset + APP_MANAGER_VISIBLE_ITEMS) {
                    manager->scroll_offset = manager->selected_index - APP_MANAGER_VISIBLE_ITEMS + 1;
                }
            }
            break;
        case input_key_ok:
            app_manager_launch_app(manager, manager->selected_index);
            break;
        default:
            break;
    }
}
