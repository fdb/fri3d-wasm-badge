// Browser entry point for the Fri3d badge firmware. Compiled with Emscripten,
// it wraps the *same* canvas / random / wasm_host modules that run on the
// real badge and exposes them to JavaScript through `extern "C"` shims.
//
// JS drives the loop:
//   1. web_init() — mirrors Arduino setup().
//   2. per animation frame: web_render(), then read web_fb() + web_fb_width()/height()
//      from the module's memory and blit to an HTML <canvas>.
//   3. on keydown/keyup: web_on_input(key, event).
//
// This lets us iterate on C++ logic (MT19937 bugs, canvas semantics, host
// bindings) with sub-second feedback loops via Playwright, without touching
// the badge hardware.

#include <cstdio>
#include <cstdint>
#include <emscripten/emscripten.h>

#include "../src/canvas.h"
#include "../src/random.h"
#include "../src/wasm_host.h"
#include "../src/embedded_apps.h"

using fri3d::Canvas;
using fri3d::Random;

static Canvas g_canvas;
static Random g_rng(42);

extern "C" {

static void dispatch_pending_switches() {
    if (wasm_host::exit_requested()) {
        wasm_host::clear_exit_request();
        const EmbeddedApp& launcher = EMBEDDED_APPS[0];
        std::printf("[web] exit_to_launcher -> %s\n", launcher.name);
        const char* err = wasm_host::reload(launcher.wasm, launcher.wasm_len);
        if (err) std::printf("[web] launcher reload failed: %s\n", err);
    }
    if (wasm_host::start_app_requested()) {
        uint32_t req = wasm_host::requested_app_id();
        wasm_host::clear_start_app_request();
        const EmbeddedApp* target = nullptr;
        for (uint32_t i = 0; i < EMBEDDED_APPS_COUNT; ++i) {
            if (EMBEDDED_APPS[i].app_id == req) { target = &EMBEDDED_APPS[i]; break; }
        }
        if (!target) {
            std::printf("[web] start_app(%u): no such app\n", req);
        } else {
            std::printf("[web] start_app(%u) -> %s\n", req, target->name);
            const char* err = wasm_host::reload(target->wasm, target->wasm_len);
            if (err) std::printf("[web] %s reload failed: %s\n", target->name, err);
        }
    }
}

EMSCRIPTEN_KEEPALIVE
int web_init() {
    const EmbeddedApp& launcher = EMBEDDED_APPS[0];
    const char* err = wasm_host::init(g_canvas, g_rng, launcher.wasm, launcher.wasm_len);
    if (err) {
        std::printf("[web] wasm init failed: %s\n", err);
        return 1;
    }
    std::printf("[web] wasm ready (%s)\n", launcher.name);
    return 0;
}

EMSCRIPTEN_KEEPALIVE
void web_render() {
    wasm_host::render();
    dispatch_pending_switches();
}

EMSCRIPTEN_KEEPALIVE
void web_on_input(int key, int event) {
    wasm_host::on_input((uint32_t)key, (uint32_t)event);
    dispatch_pending_switches();
}

EMSCRIPTEN_KEEPALIVE
const uint8_t* web_fb() {
    return g_canvas.buffer();
}

EMSCRIPTEN_KEEPALIVE
int web_fb_width() {
    return (int)fri3d::SCREEN_WIDTH;
}

EMSCRIPTEN_KEEPALIVE
int web_fb_height() {
    return (int)fri3d::SCREEN_HEIGHT;
}

// Introspection helpers used by Playwright tests to verify deterministic
// behaviour without having to pixel-diff the canvas.
EMSCRIPTEN_KEEPALIVE
uint32_t web_rng_get() {
    return g_rng.get();
}

EMSCRIPTEN_KEEPALIVE
void web_rng_seed(uint32_t s) {
    g_rng.seed(s);
}

// Entry point required by Emscripten when building as an executable.
int main() {
    return 0;
}

} // extern "C"
