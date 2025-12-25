#include "frd.h"
#include "canvas.h"
#include "input.h"
#include "viewport.h"
#include <stdint.h>
#include <stdbool.h>

// Host imports
#define WASM_IMPORT(name) __attribute__((import_module("env"), import_name(name)))
WASM_IMPORT("host_lcd_update") extern void host_lcd_update(uint8_t* buffer, int size);
WASM_IMPORT("host_get_input") extern uint32_t host_get_input(void);

// OS state
static Canvas* g_canvas = NULL;
static uint32_t g_prev_input = 0;

void frd_crash(const char* message) {
    // In WASM we can't do much - just loop forever
    (void)message;
    while (1) {}
}

// App entry point (defined in main.c)
extern void app_init(void);

// Called by WASM runtime each frame
void on_tick(void) {
    // Initialize on first call
    static bool initialized = false;
    if (!initialized) {
        g_canvas = canvas_create();
        app_init();
        initialized = true;
    }

    ViewPort* viewport = view_port_get_current();

    // Process input - detect changes
    uint32_t input = host_get_input();
    if (viewport) {
        // Check each key for press/release
        for (int i = 0; i < 6; i++) {
            uint32_t mask = 1 << i;
            bool was_pressed = (g_prev_input & mask) != 0;
            bool is_pressed = (input & mask) != 0;

            if (is_pressed && !was_pressed) {
                InputEvent event = { .key = (InputKey)i, .type = InputTypePress };
                view_port_handle_input(viewport, &event);
            } else if (!is_pressed && was_pressed) {
                InputEvent event = { .key = (InputKey)i, .type = InputTypeRelease };
                view_port_handle_input(viewport, &event);
            }
        }
    }
    g_prev_input = input;

    // Render if viewport exists and is enabled
    if (view_port_is_enabled(viewport)) {
        canvas_clear(g_canvas);
        view_port_render(viewport, g_canvas);
        host_lcd_update(canvas_get_buffer(g_canvas), (int)canvas_get_buffer_size(g_canvas));
    }
}

int main(void) {
    return 0;
}
