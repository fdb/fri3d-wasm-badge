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

#include <stdio.h>
#include <stdlib.h>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

// Global state
static display_sdl_t* g_display = NULL;
static canvas_t* g_canvas = NULL;
static random_t* g_random = NULL;
static input_sdl_t* g_input_handler = NULL;
static input_manager_t* g_input_manager = NULL;

// ============================================================================
// Exported C functions for JavaScript to call
// ============================================================================

// Canvas functions - called by JavaScript when WASM app calls imports
EMSCRIPTEN_KEEPALIVE
void js_canvas_clear(void) {
    if (g_canvas) canvas_clear(g_canvas);
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_width(void) {
    return g_canvas ? (int)canvas_width(g_canvas) : 0;
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_height(void) {
    return g_canvas ? (int)canvas_height(g_canvas) : 0;
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_set_color(int color) {
    if (g_canvas) canvas_set_color(g_canvas, (canvas_color_t)color);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_set_font(int font) {
    if (g_canvas) canvas_set_font(g_canvas, (canvas_font_t)font);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_dot(int x, int y) {
    if (g_canvas) canvas_draw_dot(g_canvas, x, y);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_line(int x1, int y1, int x2, int y2) {
    if (g_canvas) canvas_draw_line(g_canvas, x1, y1, x2, y2);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_frame(int x, int y, int w, int h) {
    if (g_canvas) canvas_draw_frame(g_canvas, x, y, w, h);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_box(int x, int y, int w, int h) {
    if (g_canvas) canvas_draw_box(g_canvas, x, y, w, h);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_rframe(int x, int y, int w, int h, int r) {
    if (g_canvas) canvas_draw_rframe(g_canvas, x, y, w, h, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_rbox(int x, int y, int w, int h, int r) {
    if (g_canvas) canvas_draw_rbox(g_canvas, x, y, w, h, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_circle(int x, int y, int r) {
    if (g_canvas) canvas_draw_circle(g_canvas, x, y, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_disc(int x, int y, int r) {
    if (g_canvas) canvas_draw_disc(g_canvas, x, y, r);
}

EMSCRIPTEN_KEEPALIVE
void js_canvas_draw_str(int x, int y, const char* str) {
    if (g_canvas && str) canvas_draw_str(g_canvas, x, y, str);
}

EMSCRIPTEN_KEEPALIVE
int js_canvas_string_width(const char* str) {
    return (g_canvas && str) ? (int)canvas_string_width(g_canvas, str) : 0;
}

// Random functions
EMSCRIPTEN_KEEPALIVE
void js_random_seed(int seed) {
    if (g_random) random_seed(g_random, (uint32_t)seed);
}

EMSCRIPTEN_KEEPALIVE
int js_random_get(void) {
    return g_random ? (int)random_get(g_random) : 0;
}

EMSCRIPTEN_KEEPALIVE
int js_random_range(int max) {
    return g_random ? (int)random_range(g_random, (uint32_t)max) : 0;
}

// Display functions
EMSCRIPTEN_KEEPALIVE
void js_display_flush(void) {
    if (g_display) display_sdl_flush(g_display);
}

// Input polling - returns bitmask of pressed keys
EMSCRIPTEN_KEEPALIVE
int js_poll_input(void) {
    if (!g_input_handler || !g_input_manager) return 0;

    uint32_t time_ms = input_sdl_get_time_ms(g_input_handler);
    input_manager_update(g_input_manager, input_sdl_get_handler(g_input_handler), time_ms);

    // Return events as packed data or handle in JS
    return 0;
}

// Check for pending input event
EMSCRIPTEN_KEEPALIVE
int js_has_input_event(void) {
    return (g_input_manager && input_manager_has_event(g_input_manager)) ? 1 : 0;
}

// Get next input event (returns key << 8 | type)
EMSCRIPTEN_KEEPALIVE
int js_get_input_event(void) {
    if (!g_input_manager || !input_manager_has_event(g_input_manager)) return -1;
    input_event_t event = input_manager_get_event(g_input_manager);
    return ((int)event.key << 8) | (int)event.type;
}

// Get current time in milliseconds
EMSCRIPTEN_KEEPALIVE
int js_get_time_ms(void) {
    return g_input_handler ? (int)input_sdl_get_time_ms(g_input_handler) : 0;
}

// ============================================================================
// Main Loop
// ============================================================================

static void main_loop(void) {
    if (!g_display) return;

    if (display_sdl_should_quit(g_display)) {
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

    printf("Fri3d Web Emulator starting...\n");

    // Initialize display
    g_display = (display_sdl_t*)malloc(sizeof(display_sdl_t));
    if (!g_display) {
        fprintf(stderr, "Failed to allocate display\n");
        return 1;
    }
    display_sdl_init(g_display);

    if (!display_sdl_start(g_display, false)) {
        fprintf(stderr, "Failed to initialize display\n");
        free(g_display);
        return 1;
    }

    // Initialize canvas
    g_canvas = (canvas_t*)malloc(sizeof(canvas_t));
    if (!g_canvas) {
        fprintf(stderr, "Failed to allocate canvas\n");
        display_sdl_cleanup(g_display);
        free(g_display);
        return 1;
    }
    canvas_init(g_canvas, display_sdl_get_u8g2(g_display));

    // Initialize random
    g_random = (random_t*)malloc(sizeof(random_t));
    if (!g_random) {
        fprintf(stderr, "Failed to allocate random\n");
        free(g_canvas);
        display_sdl_cleanup(g_display);
        free(g_display);
        return 1;
    }
    random_init(g_random);

    // Initialize input
    g_input_handler = (input_sdl_t*)malloc(sizeof(input_sdl_t));
    g_input_manager = (input_manager_t*)malloc(sizeof(input_manager_t));
    if (!g_input_handler || !g_input_manager) {
        fprintf(stderr, "Failed to allocate input handlers\n");
        free(g_input_manager);
        free(g_input_handler);
        free(g_random);
        free(g_canvas);
        display_sdl_cleanup(g_display);
        free(g_display);
        return 1;
    }
    input_sdl_init(g_input_handler, g_display);
    input_manager_init(g_input_manager);

    printf("Fri3d Web Emulator ready!\n");
    printf("Canvas size: %dx%d\n", (int)canvas_width(g_canvas), (int)canvas_height(g_canvas));

#ifdef __EMSCRIPTEN__
    // Use Emscripten's main loop for SDL event handling
    emscripten_set_main_loop(main_loop, 60, 0);
#else
    while (!display_sdl_should_quit(g_display)) {
        main_loop();
        SDL_Delay(16);
    }
#endif

    return 0;
}
