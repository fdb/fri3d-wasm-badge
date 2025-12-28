#pragma once

#include <wasm_export.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct canvas canvas_t;
typedef struct random random_t;

// Error buffer size
#define WASM_ERROR_BUFFER_SIZE 512

/**
 * WASM module runner using WAMR (WebAssembly Micro Runtime).
 * Handles module loading, native function registration, and execution.
 */
typedef struct {
    // WAMR state
    uint8_t* heap_buffer;
    wasm_module_t module;
    wasm_module_inst_t module_inst;
    wasm_exec_env_t exec_env;

    // Function handles
    wasm_function_inst_t func_render;
    wasm_function_inst_t func_on_input;
    wasm_function_inst_t func_set_scene;
    wasm_function_inst_t func_get_scene;
    wasm_function_inst_t func_get_scene_count;

    // Canvas and Random for native functions (not owned)
    canvas_t* canvas;
    random_t* random;

    // Error handling
    char last_error[WASM_ERROR_BUFFER_SIZE];
    char error_buffer[256];

    bool initialized;
} wasm_runner_t;

/**
 * Initialize a WASM runner instance.
 */
void wasm_runner_init(wasm_runner_t* runner);

/**
 * Clean up a WASM runner instance.
 */
void wasm_runner_cleanup(wasm_runner_t* runner);

/**
 * Initialize the WAMR runtime.
 * Must be called once before loading any modules.
 * @param runner WASM runner instance
 * @param heap_size Size of the WAMR heap in bytes
 * @return true on success
 */
bool wasm_runner_start(wasm_runner_t* runner, size_t heap_size);

/**
 * Set the canvas instance for native functions.
 * Must be called before loading a module.
 */
void wasm_runner_set_canvas(wasm_runner_t* runner, canvas_t* canvas);

/**
 * Set the random instance for native functions.
 * Must be called before loading a module.
 */
void wasm_runner_set_random(wasm_runner_t* runner, random_t* random);

/**
 * Load and instantiate a WASM module from file.
 * Unloads any previously loaded module.
 * @param runner WASM runner instance
 * @param path Path to the .wasm file
 * @return true on success
 */
bool wasm_runner_load_module(wasm_runner_t* runner, const char* path);

/**
 * Load and instantiate a WASM module from memory.
 * Unloads any previously loaded module.
 * @param runner WASM runner instance
 * @param data WASM binary data
 * @param size Size of the data
 * @return true on success
 */
bool wasm_runner_load_module_from_memory(wasm_runner_t* runner, const uint8_t* data, size_t size);

/**
 * Unload the current module.
 */
void wasm_runner_unload_module(wasm_runner_t* runner);

/**
 * Check if a module is currently loaded.
 */
bool wasm_runner_is_module_loaded(const wasm_runner_t* runner);

/**
 * Call the module's render() function.
 */
void wasm_runner_call_render(wasm_runner_t* runner);

/**
 * Call the module's on_input(key, type) function.
 */
void wasm_runner_call_on_input(wasm_runner_t* runner, uint32_t key, uint32_t type);

/**
 * Get the number of scenes (for test apps).
 */
uint32_t wasm_runner_get_scene_count(wasm_runner_t* runner);

/**
 * Set the current scene.
 */
void wasm_runner_set_scene(wasm_runner_t* runner, uint32_t scene);

/**
 * Get the current scene.
 */
uint32_t wasm_runner_get_scene(wasm_runner_t* runner);

/**
 * Check if render function exists.
 */
bool wasm_runner_has_render_function(const wasm_runner_t* runner);

/**
 * Check if on_input function exists.
 */
bool wasm_runner_has_on_input_function(const wasm_runner_t* runner);

/**
 * Get the last error message.
 */
const char* wasm_runner_get_last_error(const wasm_runner_t* runner);

#ifdef __cplusplus
}
#endif
