#include <frd.h>
#include <canvas.h>
#include <input.h>
#include <random.h>

static uint32_t g_seed = 42;

// Scene control for visual testing (single scene app)
uint32_t get_scene(void) { return 0; }
void set_scene(uint32_t scene) { (void)scene; }
uint32_t get_scene_count(void) { return 1; }

// Called by host each frame for rendering
void render(void) {
    // Use same seed each frame for consistent circles
    random_seed(g_seed);
    canvas_set_color(ColorBlack);

    // Draw 10 random circles
    for (int i = 0; i < 10; i++) {
        int32_t x = (int32_t)random_range(128);
        int32_t y = (int32_t)random_range(64);
        uint32_t r = random_range(15) + 3;
        canvas_draw_circle(x, y, r);
    }
}

// Called by host for input events
void on_input(input_key_t key, input_type_t type) {
    if (type == input_type_press && key == input_key_ok) {
        // Generate new random seed for new circles
        g_seed = random_get();
    }
}

// Required by WASI libc, but not used (host calls render/on_input directly)
int main(void) {
    return 0;
}
