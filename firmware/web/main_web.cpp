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

#include "../src/app_switcher.h"
#include "../src/canvas.h"
#include "../src/screen.h"
#include "../src/input_queue.h"
#include "../src/random.h"
#include "../src/wasm_host.h"
#include "../src/embedded_apps.h"

#include <cstdlib>
#include <cstring>

using fri3d::Canvas;
using fri3d::Random;
using fri3d::Screen;

// Legacy mono palette: design's violet ink on near-black background. Apps
// that don't draw via the new screen_* imports get auto-blitted in this
// scheme so they look like they belong on the new badge OS.
static constexpr uint32_t LEGACY_FG_RGB = 0x7b6cff;
static constexpr uint32_t LEGACY_BG_RGB = 0x05050a;

static Canvas g_canvas;
static Random g_rng(42);
static fri3d::InputQueue g_input;

// 296x240 RGB565 framebuffer. ~142 KB on the heap (Emscripten doesn't
// have PSRAM; the JS side reads from this pointer each frame).
static uint16_t* g_screen_pixels = nullptr;
static Screen*   g_screen        = nullptr;

extern "C" {

EMSCRIPTEN_KEEPALIVE
int web_init() {
    g_screen_pixels = (uint16_t*) std::malloc(fri3d::SCREEN_BYTES);
    if (!g_screen_pixels) {
        std::printf("[web] screen alloc failed (%zu bytes)\n", fri3d::SCREEN_BYTES);
        return 2;
    }
    std::memset(g_screen_pixels, 0, fri3d::SCREEN_BYTES);
    g_screen = new Screen(g_screen_pixels);

    const EmbeddedApp& launcher = EMBEDDED_APPS[0];
    const char* err = wasm_host::init(g_canvas, *g_screen, g_rng, launcher.wasm, launcher.wasm_len);
    if (err) {
        std::printf("[web] wasm init failed: %s\n", err);
        return 1;
    }
    std::printf("[web] wasm ready (%s)\n", launcher.name);
    return 0;
}

EMSCRIPTEN_KEEPALIVE
void web_render() {
    // Drain anything pushed by JS between the last render and this one.
    fri3d::InputEvent ev;
    while (g_input.pop(ev)) {
        wasm_host::on_input(ev.key, ev.type);
    }
    app_switcher::dispatch();
    wasm_host::render();
    app_switcher::dispatch();
    // Same legacy-fallback as main.cpp: if the active app didn't touch the
    // color screen this frame, blit the mono canvas in centred.
    if (g_screen && !g_screen->used()) {
        g_screen->blit_legacy_canvas(g_canvas.buffer(),
                                     fri3d::SCREEN_WIDTH, fri3d::SCREEN_HEIGHT,
                                     LEGACY_FG_RGB, LEGACY_BG_RGB);
    }
}

// JS DOM handlers push here instead of dispatching synchronously — mirrors
// the hardware architecture exactly so behaviour is consistent across
// platforms and a slow render can't swallow a quick tap.
EMSCRIPTEN_KEEPALIVE
void web_push_input(int key, int event) {
    g_input.push({(uint32_t)key, (uint32_t)event});
}

// Kept for backwards compatibility with any callers that want immediate
// synchronous delivery (e.g. Playwright tests that bypass the queue).
EMSCRIPTEN_KEEPALIVE
void web_on_input(int key, int event) {
    wasm_host::on_input((uint32_t)key, (uint32_t)event);
    app_switcher::dispatch();
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

// New full-screen RGB565 framebuffer. JS unpacks 5/6/5 -> RGBA each frame.
EMSCRIPTEN_KEEPALIVE
const uint16_t* web_screen() {
    return g_screen_pixels;
}

EMSCRIPTEN_KEEPALIVE
int web_screen_width() {
    return (int)fri3d::SCREEN_W;
}

EMSCRIPTEN_KEEPALIVE
int web_screen_height() {
    return (int)fri3d::SCREEN_H;
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
