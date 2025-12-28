#pragma once

#include "canvas.h"
#include "input.h"
#include "wasm_runner.h"
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Maximum number of apps in the launcher
#define APP_MANAGER_MAX_APPS 16

// Maximum length for app name and path
#define APP_NAME_MAX_LEN 32
#define APP_PATH_MAX_LEN 128

// Error buffer size
#define APP_MANAGER_ERROR_BUFFER_SIZE 256

/**
 * Application entry for the launcher.
 */
typedef struct {
    char name[APP_NAME_MAX_LEN];      // Display name
    char path[APP_PATH_MAX_LEN];      // Path to .wasm file
} app_entry_t;

/**
 * App manager with built-in launcher.
 * Handles app discovery, launching, and switching.
 */
typedef struct {
    canvas_t* canvas;
    random_t* random;
    wasm_runner_t wasm_runner;

    app_entry_t apps[APP_MANAGER_MAX_APPS];
    size_t app_count;
    size_t selected_index;
    size_t scroll_offset;
    bool in_launcher;

    char last_error[APP_MANAGER_ERROR_BUFFER_SIZE];
} app_manager_t;

// Launcher UI constants
#define APP_VISIBLE_ITEMS 4
#define APP_ITEM_HEIGHT 14
#define APP_START_Y 12
#define APP_TEXT_X 16
#define APP_CIRCLE_X 6
#define APP_CIRCLE_RADIUS 3

/**
 * Initialize an app manager instance.
 */
void app_manager_init(app_manager_t* manager);

/**
 * Clean up an app manager instance.
 */
void app_manager_cleanup(app_manager_t* manager);

/**
 * Start the app manager.
 * @param manager App manager
 * @param canvas Canvas for drawing
 * @param random Random number generator
 * @return true on success
 */
bool app_manager_start(app_manager_t* manager, canvas_t* canvas, random_t* random);

/**
 * Add an app to the list.
 * @return true if added successfully
 */
bool app_manager_add_app(app_manager_t* manager, const char* name, const char* path);

/**
 * Clear all apps from the list.
 */
void app_manager_clear_apps(app_manager_t* manager);

/**
 * Get the number of apps.
 */
size_t app_manager_app_count(const app_manager_t* manager);

/**
 * Show the launcher UI.
 * Returns to launcher from any running app.
 */
void app_manager_show_launcher(app_manager_t* manager);

/**
 * Launch an app by index.
 * @param manager App manager
 * @param index App index in the list
 * @return true if launched successfully
 */
bool app_manager_launch_app(app_manager_t* manager, size_t index);

/**
 * Launch an app by path.
 * @param manager App manager
 * @param path Path to .wasm file
 * @return true if launched successfully
 */
bool app_manager_launch_app_by_path(app_manager_t* manager, const char* path);

/**
 * Check if currently in launcher mode.
 */
bool app_manager_is_in_launcher(const app_manager_t* manager);

/**
 * Check if an app is currently running.
 */
bool app_manager_is_app_running(const app_manager_t* manager);

/**
 * Render the current view (launcher or app).
 */
void app_manager_render(app_manager_t* manager);

/**
 * Handle input event.
 * @param manager App manager
 * @param key Input key
 * @param type Input type (use raw Press/Release for apps)
 */
void app_manager_handle_input(app_manager_t* manager, input_key_t key, input_type_t type);

/**
 * Get the last error message.
 */
const char* app_manager_get_last_error(const app_manager_t* manager);

/**
 * Get the WASM runner (for test mode access).
 */
wasm_runner_t* app_manager_get_wasm_runner(app_manager_t* manager);

#ifdef __cplusplus
}
#endif
