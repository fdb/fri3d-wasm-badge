#pragma once

#include <u8g2.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Screen dimensions (matching SSD1306 128x64)
#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 64

// Color values matching the SDK
typedef enum {
    COLOR_WHITE = 0,
    COLOR_BLACK = 1,
    COLOR_XOR = 2
} canvas_color_t;

// Font values matching the SDK
typedef enum {
    FONT_PRIMARY = 0,
    FONT_SECONDARY = 1,
    FONT_KEYBOARD = 2,
    FONT_BIG_NUMBERS = 3
} canvas_font_t;

/**
 * Canvas state structure.
 * Provides drawing operations backed by u8g2.
 */
typedef struct canvas {
    u8g2_t* u8g2;
    canvas_color_t current_color;
} canvas_t;

/**
 * Initialize the canvas with a u8g2 instance.
 * The u8g2 instance is owned by the Display.
 */
void canvas_init(canvas_t* canvas, u8g2_t* u8g2);

/**
 * Get the u8g2 instance (for direct access if needed).
 */
u8g2_t* canvas_get_u8g2(canvas_t* canvas);

// Screen dimensions
uint32_t canvas_width(const canvas_t* canvas);
uint32_t canvas_height(const canvas_t* canvas);

// Configuration
void canvas_clear(canvas_t* canvas);
void canvas_set_color(canvas_t* canvas, canvas_color_t color);
void canvas_set_font(canvas_t* canvas, canvas_font_t font);
canvas_color_t canvas_get_color(const canvas_t* canvas);

// Basic drawing
void canvas_draw_dot(canvas_t* canvas, int32_t x, int32_t y);
void canvas_draw_line(canvas_t* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2);

// Rectangles
void canvas_draw_frame(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h);
void canvas_draw_box(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h);
void canvas_draw_rframe(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r);
void canvas_draw_rbox(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r);

// Circles (with XOR-safe implementations)
void canvas_draw_circle(canvas_t* canvas, int32_t x, int32_t y, uint32_t r);
void canvas_draw_disc(canvas_t* canvas, int32_t x, int32_t y, uint32_t r);

// Text
void canvas_draw_str(canvas_t* canvas, int32_t x, int32_t y, const char* str);
uint32_t canvas_string_width(canvas_t* canvas, const char* str);

#ifdef __cplusplus
}
#endif
