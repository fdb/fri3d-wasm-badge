#include <frd.h>
#include <canvas.h>
#include <input.h>
#include <random.h>

// Fixed seed for deterministic random output
#define RANDOM_SEED 12345

// Scene IDs - each tests different drawing primitives
typedef enum {
    SCENE_HORIZONTAL_LINES = 0,
    SCENE_VERTICAL_LINES,
    SCENE_DIAGONAL_LINES,
    SCENE_RANDOM_PIXELS,
    SCENE_CIRCLES,
    SCENE_FILLED_CIRCLES,
    SCENE_RECTANGLES,
    SCENE_FILLED_RECTANGLES,
    SCENE_ROUNDED_RECTANGLES,
    SCENE_TEXT_RENDERING,
    SCENE_MIXED_PRIMITIVES,
    SCENE_CHECKERBOARD,
    SCENE_COUNT  // Total number of scenes
} Scene;

// Current scene (can be set externally for testing)
static Scene g_current_scene = SCENE_HORIZONTAL_LINES;

// Export the current scene for external control
uint32_t get_scene(void) {
    return (uint32_t)g_current_scene;
}

void set_scene(uint32_t scene) {
    if (scene < SCENE_COUNT) {
        g_current_scene = (Scene)scene;
    }
}

uint32_t get_scene_count(void) {
    return SCENE_COUNT;
}

// Scene rendering functions
static void render_horizontal_lines(void) {
    canvas_set_color(ColorBlack);
    // Draw horizontal lines at regular intervals
    for (int y = 0; y < 64; y += 8) {
        canvas_draw_line(0, y, 127, y);
    }
    // Draw some shorter lines
    for (int y = 4; y < 64; y += 16) {
        canvas_draw_line(20, y, 107, y);
    }
}

static void render_vertical_lines(void) {
    canvas_set_color(ColorBlack);
    // Draw vertical lines at regular intervals
    for (int x = 0; x < 128; x += 8) {
        canvas_draw_line(x, 0, x, 63);
    }
    // Draw some shorter lines
    for (int x = 4; x < 128; x += 16) {
        canvas_draw_line(x, 10, x, 53);
    }
}

static void render_diagonal_lines(void) {
    canvas_set_color(ColorBlack);
    // Draw diagonal lines from corners
    canvas_draw_line(0, 0, 127, 63);
    canvas_draw_line(127, 0, 0, 63);
    // Draw parallel diagonal lines
    for (int i = 0; i < 128; i += 16) {
        canvas_draw_line(i, 0, i + 63, 63);
        canvas_draw_line(127 - i, 0, 127 - i - 63, 63);
    }
}

static void render_random_pixels(void) {
    random_seed(RANDOM_SEED);
    canvas_set_color(ColorBlack);
    // Draw 500 random pixels
    for (int i = 0; i < 500; i++) {
        int32_t x = (int32_t)random_range(128);
        int32_t y = (int32_t)random_range(64);
        canvas_draw_dot(x, y);
    }
}

static void render_circles(void) {
    random_seed(RANDOM_SEED);
    canvas_set_color(ColorBlack);
    // Draw circles of various sizes
    canvas_draw_circle(64, 32, 30);  // Large center circle
    canvas_draw_circle(64, 32, 20);
    canvas_draw_circle(64, 32, 10);
    // Draw circles in corners
    canvas_draw_circle(15, 15, 12);
    canvas_draw_circle(112, 15, 12);
    canvas_draw_circle(15, 48, 12);
    canvas_draw_circle(112, 48, 12);
}

static void render_filled_circles(void) {
    random_seed(RANDOM_SEED);
    canvas_set_color(ColorBlack);
    // Draw filled circles (discs)
    canvas_draw_disc(32, 32, 20);
    canvas_draw_disc(96, 32, 20);
    // Smaller discs
    canvas_draw_disc(64, 16, 8);
    canvas_draw_disc(64, 48, 8);
    // Use XOR for overlapping effect
    canvas_set_color(ColorXOR);
    canvas_draw_disc(64, 32, 18);
}

static void render_rectangles(void) {
    canvas_set_color(ColorBlack);
    // Draw concentric rectangles
    canvas_draw_frame(4, 4, 120, 56);
    canvas_draw_frame(14, 10, 100, 44);
    canvas_draw_frame(24, 16, 80, 32);
    canvas_draw_frame(34, 22, 60, 20);
    // Small rectangles in corners
    canvas_draw_frame(0, 0, 20, 15);
    canvas_draw_frame(108, 0, 20, 15);
    canvas_draw_frame(0, 49, 20, 15);
    canvas_draw_frame(108, 49, 20, 15);
}

static void render_filled_rectangles(void) {
    canvas_set_color(ColorBlack);
    // Draw filled rectangles
    canvas_draw_box(10, 10, 30, 20);
    canvas_draw_box(88, 10, 30, 20);
    canvas_draw_box(10, 34, 30, 20);
    canvas_draw_box(88, 34, 30, 20);
    // Center box with XOR
    canvas_set_color(ColorXOR);
    canvas_draw_box(30, 20, 68, 24);
}

static void render_rounded_rectangles(void) {
    canvas_set_color(ColorBlack);
    // Draw rounded rectangles with various corner radii
    canvas_draw_rframe(5, 5, 50, 25, 3);
    canvas_draw_rframe(73, 5, 50, 25, 8);
    // Filled rounded rectangles
    canvas_draw_rbox(5, 34, 50, 25, 5);
    canvas_draw_rbox(73, 34, 50, 25, 10);
}

static void render_text(void) {
    canvas_set_color(ColorBlack);
    // Primary font
    canvas_set_font(FontPrimary);
    canvas_draw_str(5, 12, "Primary Font");
    // Secondary font
    canvas_set_font(FontSecondary);
    canvas_draw_str(5, 24, "Secondary Font Test");
    // Keyboard font
    canvas_set_font(FontKeyboard);
    canvas_draw_str(5, 36, "Keyboard: ABCDEF");
    // Big numbers
    canvas_set_font(FontBigNumbers);
    canvas_draw_str(5, 58, "123");
}

static void render_mixed_primitives(void) {
    random_seed(RANDOM_SEED);
    canvas_set_color(ColorBlack);

    // Background grid
    for (int x = 0; x < 128; x += 16) {
        canvas_draw_line(x, 0, x, 63);
    }
    for (int y = 0; y < 64; y += 16) {
        canvas_draw_line(0, y, 127, y);
    }

    // Circles
    canvas_draw_circle(32, 32, 15);
    canvas_draw_disc(96, 32, 10);

    // Rectangles
    canvas_draw_frame(50, 10, 28, 20);
    canvas_draw_box(52, 38, 24, 16);

    // Random pixels
    for (int i = 0; i < 50; i++) {
        int32_t x = (int32_t)random_range(128);
        int32_t y = (int32_t)random_range(64);
        canvas_draw_dot(x, y);
    }

    // Text
    canvas_set_font(FontSecondary);
    canvas_draw_str(2, 8, "Mix");
}

static void render_checkerboard(void) {
    canvas_set_color(ColorBlack);
    // Draw 8x8 pixel checkerboard pattern
    for (int y = 0; y < 64; y += 8) {
        for (int x = 0; x < 128; x += 8) {
            // Alternate filled boxes
            if (((x / 8) + (y / 8)) % 2 == 0) {
                canvas_draw_box(x, y, 8, 8);
            }
        }
    }
}

// Main render function - called by host each frame
void render(void) {
    switch (g_current_scene) {
        case SCENE_HORIZONTAL_LINES:
            render_horizontal_lines();
            break;
        case SCENE_VERTICAL_LINES:
            render_vertical_lines();
            break;
        case SCENE_DIAGONAL_LINES:
            render_diagonal_lines();
            break;
        case SCENE_RANDOM_PIXELS:
            render_random_pixels();
            break;
        case SCENE_CIRCLES:
            render_circles();
            break;
        case SCENE_FILLED_CIRCLES:
            render_filled_circles();
            break;
        case SCENE_RECTANGLES:
            render_rectangles();
            break;
        case SCENE_FILLED_RECTANGLES:
            render_filled_rectangles();
            break;
        case SCENE_ROUNDED_RECTANGLES:
            render_rounded_rectangles();
            break;
        case SCENE_TEXT_RENDERING:
            render_text();
            break;
        case SCENE_MIXED_PRIMITIVES:
            render_mixed_primitives();
            break;
        case SCENE_CHECKERBOARD:
            render_checkerboard();
            break;
        default:
            break;
    }
}

// Handle input - allows cycling through scenes manually
void on_input(InputKey key, InputType type) {
    if (type != InputTypePress) return;

    switch (key) {
        case InputKeyRight:
        case InputKeyDown:
            g_current_scene = (Scene)((g_current_scene + 1) % SCENE_COUNT);
            break;
        case InputKeyLeft:
        case InputKeyUp:
            g_current_scene = (Scene)((g_current_scene + SCENE_COUNT - 1) % SCENE_COUNT);
            break;
        default:
            break;
    }
}

// Required by WASI libc
int main(void) {
    return 0;
}
