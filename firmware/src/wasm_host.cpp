// wasm3 integration: hosts the .wasm apps defined under fri3d-app-*/ on the
// real badge by exposing the same "env" module of 24 host functions that the
// desktop emulator exposes via wasmi.
//
// Scope: host functions that matter for rendering (canvas_*, random_*,
// get_time_ms) are fully implemented. Functions we don't yet need
// (timers, start_app, fonts, draw_str) are wired as no-op stubs so any
// app that references them still links cleanly.

#include "wasm_host.h"

#include <string.h>
#include "host_platform.h"

// wasm3 headers expose a C API. Wrap the include to avoid name leakage.
extern "C" {
#include "wasm3.h"
#include "m3_env.h"
}

#include "canvas.h"
#include "random.h"

// ---------------------------------------------------------------------------
// Module-level state.

static IM3Environment g_env     = nullptr;
static IM3Runtime     g_runtime = nullptr;
static IM3Module      g_module  = nullptr;
static IM3Function    g_fn_render   = nullptr;
static IM3Function    g_fn_on_input = nullptr;

static fri3d::Canvas* g_canvas = nullptr;
static fri3d::Random* g_random = nullptr;

static bool     g_exit_requested = false;
static bool     g_start_app_requested = false;
static uint32_t g_start_app_id = 0;

// Runtime stack size for the wasm interpreter. 8 KB is enough for all
// circles-class apps; bump if deeper recursion is seen.
static constexpr uint32_t WASM_STACK_BYTES = 8 * 1024;

// ---------------------------------------------------------------------------
// Host functions. Each wraps a call from the wasm module into our C++ world.
// The m3ApiRawFunction macro hands us a _sp (stack pointer) + _mem
// (linear-memory base); m3ApiGetArg reads typed args, m3ApiGetArgMem turns
// an i32 offset into a validated host pointer.

m3ApiRawFunction(h_canvas_clear) {
    if (g_canvas) g_canvas->clear();
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_width) {
    m3ApiReturnType(int32_t);
    m3ApiReturn((int32_t)fri3d::SCREEN_WIDTH);
}

m3ApiRawFunction(h_canvas_height) {
    m3ApiReturnType(int32_t);
    m3ApiReturn((int32_t)fri3d::SCREEN_HEIGHT);
}

m3ApiRawFunction(h_canvas_set_color) {
    m3ApiGetArg(int32_t, color);
    if (g_canvas) g_canvas->set_color((fri3d::Color)color);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_set_font) {
    m3ApiGetArg(int32_t, font);
    if (g_canvas) g_canvas->set_font((fri3d::FontId)(uint32_t)font);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_dot) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    if (g_canvas) g_canvas->draw_dot(x, y);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_line) {
    m3ApiGetArg(int32_t, x1);
    m3ApiGetArg(int32_t, y1);
    m3ApiGetArg(int32_t, x2);
    m3ApiGetArg(int32_t, y2);
    if (g_canvas) g_canvas->draw_line(x1, y1, x2, y2);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_frame) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, w);
    m3ApiGetArg(int32_t, h);
    if (g_canvas && w > 0 && h > 0) g_canvas->draw_frame(x, y, (uint32_t)w, (uint32_t)h);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_box) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, w);
    m3ApiGetArg(int32_t, h);
    if (g_canvas && w > 0 && h > 0) g_canvas->draw_box(x, y, (uint32_t)w, (uint32_t)h);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_rframe) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, w);
    m3ApiGetArg(int32_t, h);
    m3ApiGetArg(int32_t, r);
    if (g_canvas && w > 0 && h > 0 && r >= 0)
        g_canvas->draw_rframe(x, y, (uint32_t)w, (uint32_t)h, (uint32_t)r);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_rbox) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, w);
    m3ApiGetArg(int32_t, h);
    m3ApiGetArg(int32_t, r);
    if (g_canvas && w > 0 && h > 0 && r >= 0)
        g_canvas->draw_rbox(x, y, (uint32_t)w, (uint32_t)h, (uint32_t)r);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_circle) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, radius);
    if (g_canvas && radius >= 0) g_canvas->draw_circle(x, y, (uint32_t)radius);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_disc) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArg(int32_t, radius);
    if (g_canvas && radius >= 0) g_canvas->draw_disc(x, y, (uint32_t)radius);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_draw_str) {
    m3ApiGetArg(int32_t, x);
    m3ApiGetArg(int32_t, y);
    m3ApiGetArgMem(const char*, text);
    if (g_canvas && text) g_canvas->draw_str(x, y, text);
    m3ApiSuccess();
}

m3ApiRawFunction(h_canvas_string_width) {
    m3ApiReturnType(int32_t);
    m3ApiGetArgMem(const char*, text);
    uint32_t w = g_canvas && text ? g_canvas->string_width(text) : 0;
    m3ApiReturn((int32_t)w);
}

m3ApiRawFunction(h_random_seed) {
    m3ApiGetArg(int32_t, seed);
    if (g_random) g_random->seed((uint32_t)seed);
    m3ApiSuccess();
}

m3ApiRawFunction(h_random_get) {
    m3ApiReturnType(int32_t);
    uint32_t v = g_random ? g_random->get() : 0;
    m3ApiReturn((int32_t)v);
}

m3ApiRawFunction(h_random_range) {
    m3ApiReturnType(int32_t);
    m3ApiGetArg(int32_t, max);
    uint32_t v = (g_random && max > 0) ? g_random->range((uint32_t)max) : 0;
    m3ApiReturn((int32_t)v);
}

m3ApiRawFunction(h_get_time_ms) {
    m3ApiReturnType(int32_t);
    m3ApiReturn((int32_t)host_millis());
}

m3ApiRawFunction(h_start_timer_ms) {
    m3ApiGetArg(int32_t, interval);
    (void)interval;
    // Timers are not yet wired. The main loop renders every frame anyway.
    m3ApiSuccess();
}

m3ApiRawFunction(h_stop_timer) {
    m3ApiSuccess();
}

m3ApiRawFunction(h_request_render) {
    // Our main loop already calls render() each frame; nothing to schedule.
    m3ApiSuccess();
}

m3ApiRawFunction(h_exit_to_launcher) {
    g_exit_requested = true;
    m3ApiSuccess();
}

m3ApiRawFunction(h_start_app) {
    m3ApiGetArg(int32_t, app_id);
    g_start_app_requested = true;
    g_start_app_id = (uint32_t)app_id;
    m3ApiSuccess();
}

// ---------------------------------------------------------------------------
// Lifecycle.

static const char* link_host_functions() {
    // Signature legend:
    //   v()       void, no args
    //   i()       returns i32, no args
    //   v(i)      void, one i32 arg
    //   v(ii)     two i32 args; (iii) three; (iiii) four; (iiiii) five
    //   i(i)      returns i32, takes one i32
    //
    // A missing import in the module yields m3Err_functionLookupFailed which
    // is benign — we treat it as success so stubs don't cause init to fail.

#define LINK(sig, name) do { \
        M3Result r = m3_LinkRawFunction(g_module, "env", #name, sig, &h_##name); \
        if (r && r != m3Err_functionLookupFailed) return r; \
    } while (0)

    LINK("v()",       canvas_clear);
    LINK("i()",       canvas_width);
    LINK("i()",       canvas_height);
    LINK("v(i)",      canvas_set_color);
    LINK("v(i)",      canvas_set_font);
    LINK("v(ii)",     canvas_draw_dot);
    LINK("v(iiii)",   canvas_draw_line);
    LINK("v(iiii)",   canvas_draw_frame);
    LINK("v(iiii)",   canvas_draw_box);
    LINK("v(iiiii)",  canvas_draw_rframe);
    LINK("v(iiiii)",  canvas_draw_rbox);
    LINK("v(iii)",    canvas_draw_circle);
    LINK("v(iii)",    canvas_draw_disc);
    LINK("v(iii)",    canvas_draw_str);
    LINK("i(i)",      canvas_string_width);
    LINK("v(i)",      random_seed);
    LINK("i()",       random_get);
    LINK("i(i)",      random_range);
    LINK("i()",       get_time_ms);
    LINK("v(i)",      start_timer_ms);
    LINK("v()",       stop_timer);
    LINK("v()",       request_render);
    LINK("v()",       exit_to_launcher);
    LINK("v(i)",      start_app);

#undef LINK
    return nullptr;
}

const char* wasm_host::init(fri3d::Canvas& canvas,
                            fri3d::Random& random,
                            const uint8_t* wasm_bytes,
                            uint32_t wasm_len) {
    g_canvas = &canvas;
    g_random = &random;

    host_log("%s\n","[wasm] m3_NewEnvironment");
    g_env = m3_NewEnvironment();
    if (!g_env) return "m3_NewEnvironment failed";

    host_log("%s\n","[wasm] m3_NewRuntime");
    g_runtime = m3_NewRuntime(g_env, WASM_STACK_BYTES, nullptr);
    if (!g_runtime) return "m3_NewRuntime failed";

    host_log("%s\n","[wasm] m3_ParseModule");
    M3Result r = m3_ParseModule(g_env, &g_module, wasm_bytes, wasm_len);
    if (r) return r;

    host_log("%s\n","[wasm] m3_LoadModule");
    r = m3_LoadModule(g_runtime, g_module);
    if (r) return r;
    // m3_LoadModule transfers ownership of the module to the runtime;
    // don't free g_module ourselves.

    host_log("%s\n","[wasm] link_host_functions");
    r = link_host_functions();
    if (r) return r;

    host_log("%s\n","[wasm] find render");
    r = m3_FindFunction(&g_fn_render, g_runtime, "render");
    if (r) return r;

    // on_input is optional (test apps that don't take input still work).
    M3Result ri = m3_FindFunction(&g_fn_on_input, g_runtime, "on_input");
    if (ri && ri != m3Err_functionLookupFailed) {
        host_log("[wasm] warn: on_input lookup: %s\n", ri);
    }

    return nullptr;
}

void wasm_host::render() {
    if (!g_fn_render) return;
    // Matches the emulator's wasm_runner.rs behaviour: the canvas is cleared
    // before each render call, so apps don't need to call canvas_clear()
    // themselves (and indeed fri3d-app-circles doesn't).
    if (g_canvas) g_canvas->clear();
    M3Result r = m3_CallV(g_fn_render);
    if (r) {
        host_log("[wasm] render trap: %s\n", r);
    }
}

void wasm_host::on_input(uint32_t key, uint32_t kind) {
    if (!g_fn_on_input) return;
    M3Result r = m3_CallV(g_fn_on_input, (int32_t)key, (int32_t)kind);
    if (r) {
        host_log("[wasm] on_input trap: %s\n", r);
    }
}

bool wasm_host::exit_requested()      { return g_exit_requested; }
void wasm_host::clear_exit_request()  { g_exit_requested = false; }

bool     wasm_host::start_app_requested()   { return g_start_app_requested; }
uint32_t wasm_host::requested_app_id()      { return g_start_app_id; }
void     wasm_host::clear_start_app_request() {
    g_start_app_requested = false;
    g_start_app_id = 0;
}

static void teardown() {
    if (g_runtime) { m3_FreeRuntime(g_runtime); g_runtime = nullptr; }
    if (g_env)     { m3_FreeEnvironment(g_env); g_env = nullptr; }
    g_module       = nullptr;   // owned by runtime, freed above
    g_fn_render    = nullptr;
    g_fn_on_input  = nullptr;
    g_exit_requested = false;
    g_start_app_requested = false;
    g_start_app_id = 0;
}

const char* wasm_host::reload(const uint8_t* wasm_bytes, uint32_t wasm_len) {
    if (!g_canvas || !g_random) return "reload before init";
    teardown();
    // Re-enter the same init flow with the previously-registered canvas/rng.
    return init(*g_canvas, *g_random, wasm_bytes, wasm_len);
}
