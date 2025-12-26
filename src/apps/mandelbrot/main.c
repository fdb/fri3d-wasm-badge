#include <frd.h>
#include <canvas.h>
#include <input.h>

static float g_x_offset = 2.5f;
static float g_y_offset = 1.12f;
static float g_x_zoom = 3.5f;
static float g_y_zoom = 2.24f;
static float g_zoom = 1.0f;

// Scene control for visual testing (single scene app)
uint32_t get_scene(void) { return 0; }
void set_scene(uint32_t scene) { (void)scene; }
uint32_t get_scene_count(void) { return 1; }

static bool mandelbrot_pixel(int x, int y) {
    float x0 = (x / 128.0f) * g_x_zoom - g_x_offset;
    float y0 = (y / 64.0f) * g_y_zoom - g_y_offset;
    float x1 = 0.0f, y1 = 0.0f;
    float x2 = 0.0f, y2 = 0.0f;
    int iter = 0;

    while (x2 + y2 <= 4.0f && iter < 50) {
        y1 = 2.0f * x1 * y1 + y0;
        x1 = x2 - y2 + x0;
        x2 = x1 * x1;
        y2 = y1 * y1;
        iter++;
    }

    return iter > 49;
}

void render(void) {
    canvas_set_color(ColorBlack);

    for (int y = 0; y < 64; y++) {
        for (int x = 0; x < 128; x++) {
            if (mandelbrot_pixel(x, y)) {
                canvas_draw_dot(x, y);
            }
        }
    }
}

void on_input(InputKey key, InputType type) {
    if (type != InputTypePress) return;

    float step = 0.1f / g_zoom;

    switch (key) {
        case InputKeyUp:
            g_y_offset += step;
            break;
        case InputKeyDown:
            g_y_offset -= step;
            break;
        case InputKeyLeft:
            g_x_offset += step;
            break;
        case InputKeyRight:
            g_x_offset -= step;
            break;
        case InputKeyOk:
            g_x_zoom *= 0.9f;
            g_y_zoom *= 0.9f;
            g_zoom += 0.15f;
            break;
        case InputKeyBack:
            g_x_zoom *= 1.1f;
            g_y_zoom *= 1.1f;
            g_zoom -= 0.15f;
            if (g_zoom < 1.0f) g_zoom = 1.0f;
            break;
    }
}

int main(void) {
    return 0;
}
