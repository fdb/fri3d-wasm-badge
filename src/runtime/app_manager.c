#include "app_manager.h"
#include <string.h>
#include <stdio.h>

// Helper: min function
static inline size_t min_size(size_t a, size_t b) {
    return a < b ? a : b;
}

// Forward declarations
static void app_manager_render_launcher(app_manager_t* manager);
static void app_manager_launcher_input(app_manager_t* manager, input_key_t key, input_type_t type);

void app_manager_init(app_manager_t* manager) {
    memset(manager, 0, sizeof(app_manager_t));
    manager->in_launcher = true;
    wasm_runner_init(&manager->wasm_runner);
}

void app_manager_cleanup(app_manager_t* manager) {
    wasm_runner_cleanup(&manager->wasm_runner);
}

bool app_manager_start(app_manager_t* manager, canvas_t* canvas, random_t* random) {
    manager->canvas = canvas;
    manager->random = random;

    if (!wasm_runner_start(&manager->wasm_runner, 10 * 1024 * 1024)) {
        snprintf(manager->last_error, APP_MANAGER_ERROR_BUFFER_SIZE, "%s",
                 wasm_runner_get_last_error(&manager->wasm_runner));
        return false;
    }

    wasm_runner_set_canvas(&manager->wasm_runner, canvas);
    wasm_runner_set_random(&manager->wasm_runner, random);

    return true;
}

bool app_manager_add_app(app_manager_t* manager, const char* name, const char* path) {
    if (manager->app_count >= APP_MANAGER_MAX_APPS) {
        return false;
    }

    app_entry_t* entry = &manager->apps[manager->app_count];
    strncpy(entry->name, name, APP_NAME_MAX_LEN - 1);
    entry->name[APP_NAME_MAX_LEN - 1] = '\0';
    strncpy(entry->path, path, APP_PATH_MAX_LEN - 1);
    entry->path[APP_PATH_MAX_LEN - 1] = '\0';

    manager->app_count++;
    return true;
}

void app_manager_clear_apps(app_manager_t* manager) {
    manager->app_count = 0;
    manager->selected_index = 0;
    manager->scroll_offset = 0;
}

size_t app_manager_app_count(const app_manager_t* manager) {
    return manager->app_count;
}

void app_manager_show_launcher(app_manager_t* manager) {
    wasm_runner_unload_module(&manager->wasm_runner);
    manager->in_launcher = true;
}

bool app_manager_launch_app(app_manager_t* manager, size_t index) {
    if (index >= manager->app_count) {
        snprintf(manager->last_error, APP_MANAGER_ERROR_BUFFER_SIZE, "Invalid app index");
        return false;
    }

    return app_manager_launch_app_by_path(manager, manager->apps[index].path);
}

bool app_manager_launch_app_by_path(app_manager_t* manager, const char* path) {
    if (!wasm_runner_load_module(&manager->wasm_runner, path)) {
        snprintf(manager->last_error, APP_MANAGER_ERROR_BUFFER_SIZE, "%s",
                 wasm_runner_get_last_error(&manager->wasm_runner));
        return false;
    }

    manager->in_launcher = false;
    return true;
}

bool app_manager_is_in_launcher(const app_manager_t* manager) {
    return manager->in_launcher;
}

bool app_manager_is_app_running(const app_manager_t* manager) {
    return !manager->in_launcher && wasm_runner_is_module_loaded(&manager->wasm_runner);
}

void app_manager_render(app_manager_t* manager) {
    if (manager->in_launcher) {
        app_manager_render_launcher(manager);
    } else {
        wasm_runner_call_render(&manager->wasm_runner);
    }
}

void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type) {
    if (manager->in_launcher) {
        app_manager_launcher_input(manager, key, type);
    } else {
        // Forward to WASM app (only Press and Release, not ShortPress/LongPress)
        if (type == INPUT_TYPE_PRESS || type == INPUT_TYPE_RELEASE) {
            wasm_runner_call_on_input(&manager->wasm_runner, (uint32_t)key, (uint32_t)type);
        }
    }
}

const char* app_manager_get_last_error(const app_manager_t* manager) {
    return manager->last_error;
}

wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager) {
    return &manager->wasm_runner;
}

static void app_manager_render_launcher(app_manager_t* manager) {
    if (!manager->canvas) return;

    canvas_clear(manager->canvas);
    canvas_set_color(manager->canvas, COLOR_BLACK);
    canvas_set_font(manager->canvas, FONT_PRIMARY);

    // Draw title
    canvas_draw_str(manager->canvas, 2, 10, "Fri3d Apps");
    canvas_draw_line(manager->canvas, 0, 12, 127, 12);

    if (manager->app_count == 0) {
        canvas_draw_str(manager->canvas, 2, 30, "No apps found");
        return;
    }

    // Calculate visible range
    size_t start_idx = manager->scroll_offset;
    size_t end_idx = min_size(start_idx + APP_VISIBLE_ITEMS, manager->app_count);

    // Draw app list
    for (size_t i = start_idx; i < end_idx; i++) {
        int y = APP_START_Y + (int)((i - start_idx) + 1) * APP_ITEM_HEIGHT;

        // Draw selection indicator
        if (i == manager->selected_index) {
            // Filled circle for selected
            canvas_draw_disc(manager->canvas, APP_CIRCLE_X, y - 3, APP_CIRCLE_RADIUS);
        } else {
            // Empty circle for unselected
            canvas_draw_circle(manager->canvas, APP_CIRCLE_X, y - 3, APP_CIRCLE_RADIUS);
        }

        // Draw app name
        canvas_draw_str(manager->canvas, APP_TEXT_X, y, manager->apps[i].name);
    }

    // Draw scroll indicators if needed
    if (manager->scroll_offset > 0) {
        // Up arrow at top
        canvas_draw_str(manager->canvas, 120, 20, "^");
    }
    if (end_idx < manager->app_count) {
        // Down arrow at bottom
        canvas_draw_str(manager->canvas, 120, 60, "v");
    }
}

static void app_manager_launcher_input(app_manager_t* manager, input_key_t key, input_type_t type) {
    // Only handle key presses, not releases
    if (type != INPUT_TYPE_PRESS && type != INPUT_TYPE_SHORT_PRESS) {
        return;
    }

    if (manager->app_count == 0) {
        return;
    }

    switch (key) {
    case INPUT_KEY_UP:
        if (manager->selected_index > 0) {
            manager->selected_index--;
            // Scroll up if needed
            if (manager->selected_index < manager->scroll_offset) {
                manager->scroll_offset = manager->selected_index;
            }
        }
        break;

    case INPUT_KEY_DOWN:
        if (manager->selected_index < manager->app_count - 1) {
            manager->selected_index++;
            // Scroll down if needed
            if (manager->selected_index >= manager->scroll_offset + APP_VISIBLE_ITEMS) {
                manager->scroll_offset = manager->selected_index - APP_VISIBLE_ITEMS + 1;
            }
        }
        break;

    case INPUT_KEY_OK:
        app_manager_launch_app(manager, manager->selected_index);
        break;

    default:
        break;
    }
}
