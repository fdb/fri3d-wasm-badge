/**
 * Fri3d Web Emulator
 * Uses browser's native WebAssembly API instead of WAMR
 * Canvas operations are exposed to JavaScript which bridges to user WASM apps
 */

#include "display_sdl.h"
#include "input_sdl.h"
#include "canvas.h"
#include "random.h"
#include "input.h"

#include <iostream>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

using namespace fri3d;

// Global state
static DisplaySDL* g_display = nullptr;
static Canvas* g_canvas = nullptr;
static Random* g_random = nullptr;
static InputSDL* g_inputHandler = nullptr;
static InputManager* g_inputManager = nullptr;
static bool g_appRunning = false;

// ============================================================================
// Exported C functions for JavaScript to call
// ============================================================================

extern "C" {

// Canvas functions - called by JavaScript when WASM app calls imports
EMSCRIPTEN_KEEPALIVE
void js_canvas_clear() {
    if (g_canvas) g_canvas->clear();
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_width() {
    return g_canvas ? g_canvas->width() : 0;
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_height() {
    return g_canvas ? g_canvas->height() : 0;
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_set_color(int color) {
    if (g_canvas) g_canvas->setColor(static_cast<Color>(color));
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_set_font(int font) {
    if (g_canvas) g_canvas->setFont(static_cast<Font>(font));
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_dot(int x, int y) {
    if (g_canvas) g_canvas->drawDot(x, y);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_line(int x1, int y1, int x2, int y2) {
    if (g_canvas) g_canvas->drawLine(x1, y1, x2, y2);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_frame(int x, int y, int w, int h) {
    if (g_canvas) g_canvas->drawFrame(x, y, w, h);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_box(int x, int y, int w, int h) {
    if (g_canvas) g_canvas->drawBox(x, y, w, h);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_rframe(int x, int y, int w, int h, int r) {
    if (g_canvas) g_canvas->drawRFrame(x, y, w, h, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_rbox(int x, int y, int w, int h, int r) {
    if (g_canvas) g_canvas->drawRBox(x, y, w, h, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_circle(int x, int y, int r) {
    if (g_canvas) g_canvas->drawCircle(x, y, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_disc(int x, int y, int r) {
    if (g_canvas) g_canvas->drawDisc(x, y, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_str(int x, int y, const char* str) {
    if (g_canvas && str) g_canvas->drawStr(x, y, str);
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_string_width(const char* str) {
    return (g_canvas && str) ? g_canvas->stringWidth(str) : 0;
}

// Random functions
EMSCRIPTEN_KEEPALIVE
void js_random_seed(int seed) {
    if (g_random) g_random->seed(seed);
}

EMSCRIPTEN_KEEPALIVE
int js_random_get() {
    return g_random ? g_random->get() : 0;
}

EMSCRIPTEN_KEEPALIVE
int js_random_range(int max) {
    return g_random ? g_random->range(max) : 0;
}

// Display functions
EMSCRIPTEN_KEEPALIVE
void js_display_flush() {
    if (g_display) g_display->flush();
}

// Input polling - returns bitmask of pressed keys
EMSCRIPTEN_KEEPALIVE
int js_poll_input() {
    if (!g_inputHandler || !g_inputManager) return 0;

    uint32_t timeMs = g_inputHandler->getTimeMs();
    g_inputManager->update(*g_inputHandler, timeMs);

    // Return events as packed data or handle in JS
    return 0;
}

// Check for pending input event
EMSCRIPTEN_KEEPALIVE
int js_has_input_event() {
    return (g_inputManager && g_inputManager->hasEvent()) ? 1 : 0;
}

// Get next input event (returns key << 8 | type)
EMSCRIPTEN_KEEPALIVE
int js_get_input_event() {
    if (!g_inputManager || !g_inputManager->hasEvent()) return -1;
    InputEvent event = g_inputManager->getEvent();
    return (static_cast<int>(event.key) << 8) | static_cast<int>(event.type);
}

// Get current time in milliseconds
EMSCRIPTEN_KEEPALIVE
int js_get_time_ms() {
    return g_inputHandler ? g_inputHandler->getTimeMs() : 0;
}

} // extern "C"

// ============================================================================
// Main Loop
// ============================================================================

static void mainLoop() {
    if (!g_display) return;

    if (g_display->shouldQuit()) {
#ifdef __EMSCRIPTEN__
        emscripten_cancel_main_loop();
#endif
        return;
    }

    // Input is handled by JavaScript calling js_poll_input()
    // Rendering is handled by JavaScript calling the WASM app's render()
    // We just need to flush the display

    // Note: The actual app logic is now in JavaScript
    // This loop just keeps SDL event pump running
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
#ifdef __EMSCRIPTEN__
            emscripten_cancel_main_loop();
#endif
            return;
        }
    }
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    std::cout << "Fri3d Web Emulator starting..." << std::endl;

    // Initialize display
    g_display = new DisplaySDL();
    if (!g_display->init(false)) {
        std::cerr << "Failed to initialize display" << std::endl;
        return 1;
    }

    // Initialize canvas
    g_canvas = new Canvas();
    g_canvas->init(g_display->getU8g2());

    // Initialize random
    g_random = new Random();

    // Initialize input
    g_inputHandler = new InputSDL(*g_display);
    g_inputManager = new InputManager();

    std::cout << "Fri3d Web Emulator ready!" << std::endl;
    std::cout << "Canvas size: " << g_canvas->width() << "x" << g_canvas->height() << std::endl;

#ifdef __EMSCRIPTEN__
    // Use Emscripten's main loop for SDL event handling
    emscripten_set_main_loop(mainLoop, 60, 0);
#else
    while (!g_display->shouldQuit()) {
        mainLoop();
        SDL_Delay(16);
    }
#endif

    return 0;
}
