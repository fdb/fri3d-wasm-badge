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
#include "../src/embedded_app.h"

using fri3d::Canvas;
using fri3d::Random;

static Canvas g_canvas;
static Random g_rng(42);

extern "C" {

EMSCRIPTEN_KEEPALIVE
int web_init() {
    const char* err = wasm_host::init(g_canvas, g_rng, embedded_app_wasm, embedded_app_wasm_len);
    if (err) {
        std::printf("[web] wasm init failed: %s\n", err);
        return 1;
    }
    std::printf("[web] wasm ready\n");
    return 0;
}

EMSCRIPTEN_KEEPALIVE
void web_render() {
    wasm_host::render();
}

EMSCRIPTEN_KEEPALIVE
void web_on_input(int key, int event) {
    wasm_host::on_input((uint32_t)key, (uint32_t)event);
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
