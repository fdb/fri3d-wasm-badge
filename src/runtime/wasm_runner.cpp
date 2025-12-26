#include "wasm_runner.h"
#include "canvas.h"
#include "random.h"
#include <cstring>
#include <fstream>
#include <iostream>

namespace fri3d {

// Global pointers for native function callbacks
// WAMR uses C-style callbacks that can't capture state
static Canvas* g_canvas = nullptr;
static Random* g_random = nullptr;

// ============================================================================
// Native Function Implementations
// ============================================================================

extern "C" {

// Canvas functions
static void native_canvas_clear(wasm_exec_env_t exec_env) {
    (void)exec_env;
    if (g_canvas) g_canvas->clear();
}

static uint32_t native_canvas_width(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_canvas ? g_canvas->width() : 0;
}

static uint32_t native_canvas_height(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_canvas ? g_canvas->height() : 0;
}

static void native_canvas_set_color(wasm_exec_env_t exec_env, uint32_t color) {
    (void)exec_env;
    if (g_canvas) g_canvas->setColor(static_cast<Color>(color));
}

static void native_canvas_set_font(wasm_exec_env_t exec_env, uint32_t font) {
    (void)exec_env;
    if (g_canvas) g_canvas->setFont(static_cast<Font>(font));
}

static void native_canvas_draw_dot(wasm_exec_env_t exec_env, int32_t x, int32_t y) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawDot(x, y);
}

static void native_canvas_draw_line(wasm_exec_env_t exec_env, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawLine(x1, y1, x2, y2);
}

static void native_canvas_draw_frame(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawFrame(x, y, w, h);
}

static void native_canvas_draw_box(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawBox(x, y, w, h);
}

static void native_canvas_draw_rframe(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawRFrame(x, y, w, h, r);
}

static void native_canvas_draw_rbox(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawRBox(x, y, w, h, r);
}

static void native_canvas_draw_circle(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawCircle(x, y, r);
}

static void native_canvas_draw_disc(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawDisc(x, y, r);
}

static void native_canvas_draw_str(wasm_exec_env_t exec_env, int32_t x, int32_t y, const char* str) {
    (void)exec_env;
    if (g_canvas) g_canvas->drawStr(x, y, str);
}

static uint32_t native_canvas_string_width(wasm_exec_env_t exec_env, const char* str) {
    (void)exec_env;
    return g_canvas ? g_canvas->stringWidth(str) : 0;
}

// Random functions
static void native_random_seed(wasm_exec_env_t exec_env, uint32_t seed) {
    (void)exec_env;
    if (g_random) g_random->seed(seed);
}

static uint32_t native_random_get(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_random ? g_random->get() : 0;
}

static uint32_t native_random_range(wasm_exec_env_t exec_env, uint32_t max) {
    (void)exec_env;
    return g_random ? g_random->range(max) : 0;
}

} // extern "C"

// Native symbols table
static NativeSymbol g_nativeSymbols[] = {
    // Canvas
    { "canvas_clear", (void*)native_canvas_clear, "()", nullptr },
    { "canvas_width", (void*)native_canvas_width, "()i", nullptr },
    { "canvas_height", (void*)native_canvas_height, "()i", nullptr },
    { "canvas_set_color", (void*)native_canvas_set_color, "(i)", nullptr },
    { "canvas_set_font", (void*)native_canvas_set_font, "(i)", nullptr },
    { "canvas_draw_dot", (void*)native_canvas_draw_dot, "(ii)", nullptr },
    { "canvas_draw_line", (void*)native_canvas_draw_line, "(iiii)", nullptr },
    { "canvas_draw_frame", (void*)native_canvas_draw_frame, "(iiii)", nullptr },
    { "canvas_draw_box", (void*)native_canvas_draw_box, "(iiii)", nullptr },
    { "canvas_draw_rframe", (void*)native_canvas_draw_rframe, "(iiiii)", nullptr },
    { "canvas_draw_rbox", (void*)native_canvas_draw_rbox, "(iiiii)", nullptr },
    { "canvas_draw_circle", (void*)native_canvas_draw_circle, "(iii)", nullptr },
    { "canvas_draw_disc", (void*)native_canvas_draw_disc, "(iii)", nullptr },
    { "canvas_draw_str", (void*)native_canvas_draw_str, "(ii*)", nullptr },
    { "canvas_string_width", (void*)native_canvas_string_width, "(*)i", nullptr },
    // Random
    { "random_seed", (void*)native_random_seed, "(i)", nullptr },
    { "random_get", (void*)native_random_get, "()i", nullptr },
    { "random_range", (void*)native_random_range, "(i)i", nullptr },
};

// ============================================================================
// WasmRunner Implementation
// ============================================================================

WasmRunner::WasmRunner() = default;

WasmRunner::~WasmRunner() {
    unloadModule();

    if (m_initialized) {
        wasm_runtime_destroy();
    }

    if (m_heapBuffer) {
        free(m_heapBuffer);
        m_heapBuffer = nullptr;
    }
}

bool WasmRunner::init(size_t heapSize) {
    if (m_initialized) {
        return true;
    }

    // Allocate heap buffer
    m_heapBuffer = static_cast<uint8_t*>(malloc(heapSize));
    if (!m_heapBuffer) {
        m_lastError = "Failed to allocate WAMR heap";
        return false;
    }

    // Initialize WAMR runtime
    RuntimeInitArgs initArgs;
    memset(&initArgs, 0, sizeof(initArgs));
    initArgs.mem_alloc_type = Alloc_With_Pool;
    initArgs.mem_alloc_option.pool.heap_buf = m_heapBuffer;
    initArgs.mem_alloc_option.pool.heap_size = heapSize;

    if (!wasm_runtime_full_init(&initArgs)) {
        m_lastError = "Failed to initialize WAMR runtime";
        free(m_heapBuffer);
        m_heapBuffer = nullptr;
        return false;
    }

    // Register native functions
    registerNativeFunctions();

    m_initialized = true;
    return true;
}

void WasmRunner::setCanvas(Canvas* canvas) {
    m_canvas = canvas;
    g_canvas = canvas;
}

void WasmRunner::setRandom(Random* random) {
    m_random = random;
    g_random = random;
}

void WasmRunner::registerNativeFunctions() {
    wasm_runtime_register_natives("env", g_nativeSymbols,
                                  sizeof(g_nativeSymbols) / sizeof(NativeSymbol));
}

bool WasmRunner::loadModule(const std::string& path) {
    // Read file
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        m_lastError = "Failed to open: " + path;
        return false;
    }

    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<char> buffer(size);
    if (!file.read(buffer.data(), size)) {
        m_lastError = "Failed to read: " + path;
        return false;
    }

    return loadModuleFromMemory(reinterpret_cast<uint8_t*>(buffer.data()), size);
}

bool WasmRunner::loadModuleFromMemory(const uint8_t* data, size_t size) {
    // Unload any existing module
    unloadModule();

    // Load module
    m_module = wasm_runtime_load(const_cast<uint8_t*>(data), size,
                                  m_errorBuffer, sizeof(m_errorBuffer));
    if (!m_module) {
        m_lastError = std::string("Failed to load module: ") + m_errorBuffer;
        return false;
    }

    // Instantiate module
    m_moduleInst = wasm_runtime_instantiate(m_module, 16 * 1024, 16 * 1024,
                                             m_errorBuffer, sizeof(m_errorBuffer));
    if (!m_moduleInst) {
        m_lastError = std::string("Failed to instantiate module: ") + m_errorBuffer;
        wasm_runtime_unload(m_module);
        m_module = nullptr;
        return false;
    }

    // Create execution environment
    m_execEnv = wasm_runtime_create_exec_env(m_moduleInst, 16 * 1024);
    if (!m_execEnv) {
        m_lastError = "Failed to create execution environment";
        wasm_runtime_deinstantiate(m_moduleInst);
        wasm_runtime_unload(m_module);
        m_moduleInst = nullptr;
        m_module = nullptr;
        return false;
    }

    // Look up exported functions
    lookupFunctions();

    if (!m_funcRender) {
        m_lastError = "Module missing required 'render' function";
        unloadModule();
        return false;
    }

    return true;
}

void WasmRunner::unloadModule() {
    m_funcRender = nullptr;
    m_funcOnInput = nullptr;
    m_funcSetScene = nullptr;
    m_funcGetScene = nullptr;
    m_funcGetSceneCount = nullptr;

    if (m_execEnv) {
        wasm_runtime_destroy_exec_env(m_execEnv);
        m_execEnv = nullptr;
    }

    if (m_moduleInst) {
        wasm_runtime_deinstantiate(m_moduleInst);
        m_moduleInst = nullptr;
    }

    if (m_module) {
        wasm_runtime_unload(m_module);
        m_module = nullptr;
    }
}

void WasmRunner::lookupFunctions() {
    m_funcRender = wasm_runtime_lookup_function(m_moduleInst, "render");
    m_funcOnInput = wasm_runtime_lookup_function(m_moduleInst, "on_input");
    m_funcSetScene = wasm_runtime_lookup_function(m_moduleInst, "set_scene");
    m_funcGetScene = wasm_runtime_lookup_function(m_moduleInst, "get_scene");
    m_funcGetSceneCount = wasm_runtime_lookup_function(m_moduleInst, "get_scene_count");
}

void WasmRunner::callRender() {
    if (!m_funcRender || !m_execEnv) return;

    // Clear canvas before rendering
    if (m_canvas) {
        m_canvas->clear();
    }

    wasm_runtime_call_wasm(m_execEnv, m_funcRender, 0, nullptr);

    if (wasm_runtime_get_exception(m_moduleInst)) {
        std::cerr << "WASM Exception in render: "
                  << wasm_runtime_get_exception(m_moduleInst) << std::endl;
        wasm_runtime_clear_exception(m_moduleInst);
    }
}

void WasmRunner::callOnInput(uint32_t key, uint32_t type) {
    if (!m_funcOnInput || !m_execEnv) return;

    uint32_t args[2] = { key, type };
    wasm_runtime_call_wasm(m_execEnv, m_funcOnInput, 2, args);

    if (wasm_runtime_get_exception(m_moduleInst)) {
        std::cerr << "WASM Exception in on_input: "
                  << wasm_runtime_get_exception(m_moduleInst) << std::endl;
        wasm_runtime_clear_exception(m_moduleInst);
    }
}

uint32_t WasmRunner::getSceneCount() {
    if (!m_funcGetSceneCount || !m_execEnv) return 0;

    uint32_t argv[1] = { 0 };
    if (wasm_runtime_call_wasm(m_execEnv, m_funcGetSceneCount, 0, argv)) {
        return argv[0];
    }
    return 0;
}

void WasmRunner::setScene(uint32_t scene) {
    if (!m_funcSetScene || !m_execEnv) return;

    uint32_t args[1] = { scene };
    wasm_runtime_call_wasm(m_execEnv, m_funcSetScene, 1, args);
}

uint32_t WasmRunner::getScene() {
    if (!m_funcGetScene || !m_execEnv) return 0;

    uint32_t argv[1] = { 0 };
    if (wasm_runtime_call_wasm(m_execEnv, m_funcGetScene, 0, argv)) {
        return argv[0];
    }
    return 0;
}

} // namespace fri3d
