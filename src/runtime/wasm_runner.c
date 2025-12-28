#include "wasm_runner.h"
#include "canvas.h"
#include "random.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

// Global pointers for native function callbacks
// WAMR uses C-style callbacks that can't capture state
static canvas_t* g_canvas = NULL;
static random_t* g_random = NULL;

// ============================================================================
// Native Function Implementations
// ============================================================================

// Canvas functions
static void native_canvas_clear(wasm_exec_env_t exec_env) {
    (void)exec_env;
    if (g_canvas) canvas_clear(g_canvas);
}

static uint32_t native_canvas_width(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_canvas ? canvas_width(g_canvas) : 0;
}

static uint32_t native_canvas_height(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_canvas ? canvas_height(g_canvas) : 0;
}

static void native_canvas_set_color(wasm_exec_env_t exec_env, uint32_t color) {
    (void)exec_env;
    if (g_canvas) canvas_set_color(g_canvas, (canvas_color_t)color);
}

static void native_canvas_set_font(wasm_exec_env_t exec_env, uint32_t font) {
    (void)exec_env;
    if (g_canvas) canvas_set_font(g_canvas, (canvas_font_t)font);
}

static void native_canvas_draw_dot(wasm_exec_env_t exec_env, int32_t x, int32_t y) {
    (void)exec_env;
    if (g_canvas) canvas_draw_dot(g_canvas, x, y);
}

static void native_canvas_draw_line(wasm_exec_env_t exec_env, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    (void)exec_env;
    if (g_canvas) canvas_draw_line(g_canvas, x1, y1, x2, y2);
}

static void native_canvas_draw_frame(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    if (g_canvas) canvas_draw_frame(g_canvas, x, y, w, h);
}

static void native_canvas_draw_box(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    if (g_canvas) canvas_draw_box(g_canvas, x, y, w, h);
}

static void native_canvas_draw_rframe(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    if (g_canvas) canvas_draw_rframe(g_canvas, x, y, w, h, r);
}

static void native_canvas_draw_rbox(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    if (g_canvas) canvas_draw_rbox(g_canvas, x, y, w, h, r);
}

static void native_canvas_draw_circle(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    if (g_canvas) canvas_draw_circle(g_canvas, x, y, r);
}

static void native_canvas_draw_disc(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    if (g_canvas) canvas_draw_disc(g_canvas, x, y, r);
}

static void native_canvas_draw_str(wasm_exec_env_t exec_env, int32_t x, int32_t y, const char* str) {
    (void)exec_env;
    if (g_canvas) canvas_draw_str(g_canvas, x, y, str);
}

static uint32_t native_canvas_string_width(wasm_exec_env_t exec_env, const char* str) {
    (void)exec_env;
    return g_canvas ? canvas_string_width(g_canvas, str) : 0;
}

// Random functions
static void native_random_seed(wasm_exec_env_t exec_env, uint32_t seed) {
    (void)exec_env;
    if (g_random) random_seed(g_random, seed);
}

static uint32_t native_random_get(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_random ? random_get(g_random) : 0;
}

static uint32_t native_random_range(wasm_exec_env_t exec_env, uint32_t max) {
    (void)exec_env;
    return g_random ? random_range(g_random, max) : 0;
}

// Native symbols table
static NativeSymbol g_native_symbols[] = {
    // Canvas
    { "canvas_clear", (void*)native_canvas_clear, "()", NULL },
    { "canvas_width", (void*)native_canvas_width, "()i", NULL },
    { "canvas_height", (void*)native_canvas_height, "()i", NULL },
    { "canvas_set_color", (void*)native_canvas_set_color, "(i)", NULL },
    { "canvas_set_font", (void*)native_canvas_set_font, "(i)", NULL },
    { "canvas_draw_dot", (void*)native_canvas_draw_dot, "(ii)", NULL },
    { "canvas_draw_line", (void*)native_canvas_draw_line, "(iiii)", NULL },
    { "canvas_draw_frame", (void*)native_canvas_draw_frame, "(iiii)", NULL },
    { "canvas_draw_box", (void*)native_canvas_draw_box, "(iiii)", NULL },
    { "canvas_draw_rframe", (void*)native_canvas_draw_rframe, "(iiiii)", NULL },
    { "canvas_draw_rbox", (void*)native_canvas_draw_rbox, "(iiiii)", NULL },
    { "canvas_draw_circle", (void*)native_canvas_draw_circle, "(iii)", NULL },
    { "canvas_draw_disc", (void*)native_canvas_draw_disc, "(iii)", NULL },
    { "canvas_draw_str", (void*)native_canvas_draw_str, "(ii*)", NULL },
    { "canvas_string_width", (void*)native_canvas_string_width, "(*)i", NULL },
    // Random
    { "random_seed", (void*)native_random_seed, "(i)", NULL },
    { "random_get", (void*)native_random_get, "()i", NULL },
    { "random_range", (void*)native_random_range, "(i)i", NULL },
};

// ============================================================================
// Helper Functions
// ============================================================================

static void register_native_functions(void) {
    wasm_runtime_register_natives("env", g_native_symbols,
                                  sizeof(g_native_symbols) / sizeof(NativeSymbol));
}

static void lookup_functions(wasm_runner_t* runner) {
    runner->func_render = wasm_runtime_lookup_function(runner->module_inst, "render");
    runner->func_on_input = wasm_runtime_lookup_function(runner->module_inst, "on_input");
    runner->func_set_scene = wasm_runtime_lookup_function(runner->module_inst, "set_scene");
    runner->func_get_scene = wasm_runtime_lookup_function(runner->module_inst, "get_scene");
    runner->func_get_scene_count = wasm_runtime_lookup_function(runner->module_inst, "get_scene_count");
}

// ============================================================================
// Public API Implementation
// ============================================================================

void wasm_runner_init(wasm_runner_t* runner) {
    memset(runner, 0, sizeof(wasm_runner_t));
}

void wasm_runner_cleanup(wasm_runner_t* runner) {
    wasm_runner_unload_module(runner);

    if (runner->initialized) {
        wasm_runtime_destroy();
    }

    if (runner->heap_buffer) {
        free(runner->heap_buffer);
        runner->heap_buffer = NULL;
    }
}

bool wasm_runner_start(wasm_runner_t* runner, size_t heap_size) {
    if (runner->initialized) {
        return true;
    }

    // Allocate heap buffer
    runner->heap_buffer = (uint8_t*)malloc(heap_size);
    if (!runner->heap_buffer) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to allocate WAMR heap");
        return false;
    }

    // Initialize WAMR runtime
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(init_args));
    init_args.mem_alloc_type = Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = runner->heap_buffer;
    init_args.mem_alloc_option.pool.heap_size = heap_size;

    if (!wasm_runtime_full_init(&init_args)) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to initialize WAMR runtime");
        free(runner->heap_buffer);
        runner->heap_buffer = NULL;
        return false;
    }

    // Register native functions
    register_native_functions();

    runner->initialized = true;
    return true;
}

void wasm_runner_set_canvas(wasm_runner_t* runner, canvas_t* canvas) {
    runner->canvas = canvas;
    g_canvas = canvas;
}

void wasm_runner_set_random(wasm_runner_t* runner, random_t* random) {
    runner->random = random;
    g_random = random;
}

bool wasm_runner_load_module(wasm_runner_t* runner, const char* path) {
    // Read file
    FILE* file = fopen(path, "rb");
    if (!file) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to open: %s", path);
        return false;
    }

    // Get file size
    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    fseek(file, 0, SEEK_SET);

    if (size <= 0) {
        fclose(file);
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Invalid file size: %s", path);
        return false;
    }

    // Read file content
    uint8_t* buffer = (uint8_t*)malloc((size_t)size);
    if (!buffer) {
        fclose(file);
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to allocate buffer for: %s", path);
        return false;
    }

    size_t bytes_read = fread(buffer, 1, (size_t)size, file);
    fclose(file);

    if (bytes_read != (size_t)size) {
        free(buffer);
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to read: %s", path);
        return false;
    }

    bool result = wasm_runner_load_module_from_memory(runner, buffer, (size_t)size);
    free(buffer);
    return result;
}

bool wasm_runner_load_module_from_memory(wasm_runner_t* runner, const uint8_t* data, size_t size) {
    // Unload any existing module
    wasm_runner_unload_module(runner);

    // Load module
    runner->module = wasm_runtime_load((uint8_t*)data, (uint32_t)size,
                                        runner->error_buffer, sizeof(runner->error_buffer));
    if (!runner->module) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to load module: %s", runner->error_buffer);
        return false;
    }

    // Instantiate module
    runner->module_inst = wasm_runtime_instantiate(runner->module, 16 * 1024, 16 * 1024,
                                                    runner->error_buffer, sizeof(runner->error_buffer));
    if (!runner->module_inst) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to instantiate module: %s", runner->error_buffer);
        wasm_runtime_unload(runner->module);
        runner->module = NULL;
        return false;
    }

    // Create execution environment
    runner->exec_env = wasm_runtime_create_exec_env(runner->module_inst, 16 * 1024);
    if (!runner->exec_env) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Failed to create execution environment");
        wasm_runtime_deinstantiate(runner->module_inst);
        wasm_runtime_unload(runner->module);
        runner->module_inst = NULL;
        runner->module = NULL;
        return false;
    }

    // Look up exported functions
    lookup_functions(runner);

    if (!runner->func_render) {
        snprintf(runner->last_error, WASM_ERROR_BUFFER_SIZE, "Module missing required 'render' function");
        wasm_runner_unload_module(runner);
        return false;
    }

    return true;
}

void wasm_runner_unload_module(wasm_runner_t* runner) {
    runner->func_render = NULL;
    runner->func_on_input = NULL;
    runner->func_set_scene = NULL;
    runner->func_get_scene = NULL;
    runner->func_get_scene_count = NULL;

    if (runner->exec_env) {
        wasm_runtime_destroy_exec_env(runner->exec_env);
        runner->exec_env = NULL;
    }

    if (runner->module_inst) {
        wasm_runtime_deinstantiate(runner->module_inst);
        runner->module_inst = NULL;
    }

    if (runner->module) {
        wasm_runtime_unload(runner->module);
        runner->module = NULL;
    }
}

bool wasm_runner_is_module_loaded(const wasm_runner_t* runner) {
    return runner->module_inst != NULL;
}

void wasm_runner_call_render(wasm_runner_t* runner) {
    if (!runner->func_render || !runner->exec_env) return;

    // Clear canvas before rendering
    if (runner->canvas) {
        canvas_clear(runner->canvas);
    }

    wasm_runtime_call_wasm(runner->exec_env, runner->func_render, 0, NULL);

    if (wasm_runtime_get_exception(runner->module_inst)) {
        fprintf(stderr, "WASM Exception in render: %s\n",
                wasm_runtime_get_exception(runner->module_inst));
        wasm_runtime_clear_exception(runner->module_inst);
    }
}

void wasm_runner_call_on_input(wasm_runner_t* runner, uint32_t key, uint32_t type) {
    if (!runner->func_on_input || !runner->exec_env) return;

    uint32_t args[2] = { key, type };
    wasm_runtime_call_wasm(runner->exec_env, runner->func_on_input, 2, args);

    if (wasm_runtime_get_exception(runner->module_inst)) {
        fprintf(stderr, "WASM Exception in on_input: %s\n",
                wasm_runtime_get_exception(runner->module_inst));
        wasm_runtime_clear_exception(runner->module_inst);
    }
}

uint32_t wasm_runner_get_scene_count(wasm_runner_t* runner) {
    if (!runner->func_get_scene_count || !runner->exec_env) return 0;

    uint32_t argv[1] = { 0 };
    if (wasm_runtime_call_wasm(runner->exec_env, runner->func_get_scene_count, 0, argv)) {
        return argv[0];
    }
    return 0;
}

void wasm_runner_set_scene(wasm_runner_t* runner, uint32_t scene) {
    if (!runner->func_set_scene || !runner->exec_env) return;

    uint32_t args[1] = { scene };
    wasm_runtime_call_wasm(runner->exec_env, runner->func_set_scene, 1, args);
}

uint32_t wasm_runner_get_scene(wasm_runner_t* runner) {
    if (!runner->func_get_scene || !runner->exec_env) return 0;

    uint32_t argv[1] = { 0 };
    if (wasm_runtime_call_wasm(runner->exec_env, runner->func_get_scene, 0, argv)) {
        return argv[0];
    }
    return 0;
}

bool wasm_runner_has_render_function(const wasm_runner_t* runner) {
    return runner->func_render != NULL;
}

bool wasm_runner_has_on_input_function(const wasm_runner_t* runner) {
    return runner->func_on_input != NULL;
}

const char* wasm_runner_get_last_error(const wasm_runner_t* runner) {
    return runner->last_error;
}
