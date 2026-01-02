#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <u8g2.h>

#include "display.h"

// Color values matching the SDK
typedef enum {
    color_white = 0,
    color_black = 1,
    color_xor = 2
} color_t;

// Font values matching the SDK
typedef enum {
    font_primary = 0,
    font_secondary = 1,
    font_keyboard = 2,
    font_big_numbers = 3
} font_t;

typedef struct {
    u8g2_t* u8g2;
    color_t current_color;
} canvas_t;

/**
 * Initialize the canvas with a u8g2 instance.
 * The u8g2 instance is owned by the display implementation.
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
void canvas_set_color(canvas_t* canvas, color_t color);
void canvas_set_font(canvas_t* canvas, font_t font);
color_t canvas_get_color(const canvas_t* canvas);

// Basic drawing
void canvas_draw_dot(canvas_t* canvas, int32_t x, int32_t y);
void canvas_draw_line(canvas_t* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2);

// Rectangles
void canvas_draw_frame(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height);
void canvas_draw_box(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height);
void canvas_draw_rframe(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius);
void canvas_draw_rbox(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius);

// Circles (with XOR-safe implementations)
void canvas_draw_circle(canvas_t* canvas, int32_t x, int32_t y, uint32_t radius);
void canvas_draw_disc(canvas_t* canvas, int32_t x, int32_t y, uint32_t radius);

// Text
void canvas_draw_str(canvas_t* canvas, int32_t x, int32_t y, const char* str);
uint32_t canvas_string_width(canvas_t* canvas, const char* str);
