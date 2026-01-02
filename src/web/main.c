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

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

static display_sdl_t g_display;
static canvas_t g_canvas;
static random_t g_random;
static input_sdl_t g_input_sdl;
static input_handler_t g_input_handler;
static input_manager_t g_input_manager;

// ============================================================================
// Exported C functions for JavaScript to call
// ============================================================================

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_clear(void) {
    canvas_clear(&g_canvas);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_canvas_width(void) {
    return (int)canvas_width(&g_canvas);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_canvas_height(void) {
    return (int)canvas_height(&g_canvas);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_set_color(int color) {
    canvas_set_color(&g_canvas, (Color)color);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_set_font(int font) {
    canvas_set_font(&g_canvas, (Font)font);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_dot(int x, int y) {
    canvas_draw_dot(&g_canvas, x, y);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_line(int x1, int y1, int x2, int y2) {
    canvas_draw_line(&g_canvas, x1, y1, x2, y2);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_frame(int x, int y, int w, int h) {
    canvas_draw_frame(&g_canvas, x, y, (uint32_t)w, (uint32_t)h);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_box(int x, int y, int w, int h) {
    canvas_draw_box(&g_canvas, x, y, (uint32_t)w, (uint32_t)h);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_rframe(int x, int y, int w, int h, int r) {
    canvas_draw_rframe(&g_canvas, x, y, (uint32_t)w, (uint32_t)h, (uint32_t)r);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_rbox(int x, int y, int w, int h, int r) {
    canvas_draw_rbox(&g_canvas, x, y, (uint32_t)w, (uint32_t)h, (uint32_t)r);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_circle(int x, int y, int r) {
    canvas_draw_circle(&g_canvas, x, y, (uint32_t)r);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_disc(int x, int y, int r) {
    canvas_draw_disc(&g_canvas, x, y, (uint32_t)r);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_canvas_draw_str(int x, int y, const char* str) {
    if (str) {
        canvas_draw_str(&g_canvas, x, y, str);
    }
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_canvas_string_width(const char* str) {
    if (!str) {
        return 0;
    }
    return (int)canvas_string_width(&g_canvas, str);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_random_seed(int seed) {
    random_seed(&g_random, (uint32_t)seed);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_random_get(void) {
    return (int)random_get(&g_random);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_random_range(int max) {
    return (int)random_range(&g_random, (uint32_t)max);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
void js_display_flush(void) {
    display_sdl_flush(&g_display);
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_poll_input(void) {
    uint32_t time_ms = input_handler_get_time_ms(&g_input_handler);
    input_manager_update(&g_input_manager, &g_input_handler, time_ms);
    return 0;
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_has_input_event(void) {
    return input_manager_has_event(&g_input_manager) ? 1 : 0;
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_get_input_event(void) {
    if (!input_manager_has_event(&g_input_manager)) {
        return -1;
    }
    input_event_t event = input_manager_get_event(&g_input_manager);
    return ((int)event.key << 8) | (int)event.type;
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
#endif
int js_get_time_ms(void) {
    return (int)input_handler_get_time_ms(&g_input_handler);
}

// ============================================================================
// Main Loop
// ============================================================================

static void main_loop(void) {
    if (display_sdl_should_quit(&g_display)) {
#ifdef __EMSCRIPTEN__
        emscripten_cancel_main_loop();
#endif
        return;
    }

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

    if (!display_sdl_init(&g_display, false)) {
        fprintf(stderr, "Failed to initialize display\n");
        return 1;
    }

    canvas_init(&g_canvas, display_sdl_get_u8g2(&g_display));
    random_init(&g_random);

    input_sdl_init(&g_input_sdl, &g_display);
    input_sdl_bind_handler(&g_input_sdl, &g_input_handler);
    input_manager_init(&g_input_manager);

    printf("Fri3d Web Emulator ready!\n");
    printf("Canvas size: %ux%u\n", (unsigned)canvas_width(&g_canvas), (unsigned)canvas_height(&g_canvas));

#ifdef __EMSCRIPTEN__
    emscripten_set_main_loop(main_loop, 60, 0);
#else
    while (!display_sdl_should_quit(&g_display)) {
        main_loop();
        SDL_Delay(16);
    }
#endif

    display_sdl_deinit(&g_display);
    return 0;
}
