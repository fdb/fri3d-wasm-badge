#include "frd.h"
#include "canvas.h"
#include "input.h"
#include "viewport.h"
#include "random.h"
#include <stdlib.h>
#include <stdbool.h>

typedef struct {
    uint32_t seed;
} AppState;

static void render_callback(Canvas* canvas, void* ctx) {
    frd_assert(ctx);
    AppState* state = ctx;

    canvas_set_color(canvas, ColorBlack);

    // Reset seed for consistent rendering of same circles
    random_seed(state->seed);

    // Draw 10 random circles
    for (int i = 0; i < 10; i++) {
        int32_t x = (int32_t)random_range(128);
        int32_t y = (int32_t)random_range(64);
        size_t r = random_range(15) + 3;  // radius 3-17
        canvas_draw_circle(canvas, x, y, r);
    }
}

static void input_callback(InputEvent* event, void* ctx) {
    frd_assert(ctx);
    AppState* state = ctx;

    if (event->type == InputTypePress) {
        switch (event->key) {
        case InputKeyOk:
            // Generate new random seed for new circles
            state->seed = random_get();
            break;
        default:
            break;
        }
    }
}

// App entry point - called once at startup by the OS
void app_init(void) {
    AppState* state = malloc(sizeof(AppState));
    state->seed = 42;  // Initial seed

    random_seed(12345);  // Seed for generating new seeds

    ViewPort* view_port = view_port_alloc();
    view_port_draw_callback_set(view_port, render_callback, state);
    view_port_input_callback_set(view_port, input_callback, state);
}
