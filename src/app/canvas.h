#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Color enum
typedef enum {
    ColorWhite = 0x00,
    ColorBlack = 0x01,
    ColorXOR = 0x02,
} Color;

// Alignment enum
typedef enum {
    AlignLeft = 0,
    AlignRight = 1,
    AlignCenter = 2,
    AlignTop = 3,
    AlignBottom = 4,
} Align;

// Canvas direction for triangles
typedef enum {
    CanvasDirectionUp,
    CanvasDirectionDown,
    CanvasDirectionLeft,
    CanvasDirectionRight,
} CanvasDirection;

// Font enum (placeholders - actual fonts added later)
typedef enum {
    FontPrimary,
    FontSecondary,
    FontKeyboard,
    FontBigNumbers,
} Font;

// Opaque Canvas struct
typedef struct Canvas Canvas;

// Lifecycle
Canvas* canvas_create(void);
void canvas_destroy(Canvas* canvas);

// Buffer access
uint8_t* canvas_get_buffer(Canvas* canvas);
size_t canvas_get_buffer_size(Canvas* canvas);

// Dimensions
size_t canvas_width(const Canvas* canvas);
size_t canvas_height(const Canvas* canvas);

// Configuration
void canvas_set_color(Canvas* canvas, Color color);
void canvas_invert_color(Canvas* canvas);
void canvas_set_font(Canvas* canvas, Font font);
void canvas_set_custom_u8g2_font(Canvas* canvas, const uint8_t* font);
void canvas_set_bitmap_mode(Canvas* canvas, bool alpha);

// Basic drawing
void canvas_clear(Canvas* canvas);
void canvas_draw_dot(Canvas* canvas, int32_t x, int32_t y);
void canvas_draw_line(Canvas* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2);

// Rectangles
void canvas_draw_frame(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height);
void canvas_draw_box(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height);
void canvas_draw_rframe(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, size_t radius);
void canvas_draw_rbox(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, size_t radius);

// Circles
void canvas_draw_circle(Canvas* canvas, int32_t x, int32_t y, size_t radius);
void canvas_draw_disc(Canvas* canvas, int32_t x, int32_t y, size_t radius);

// Triangles
void canvas_draw_triangle(Canvas* canvas, int32_t x, int32_t y, size_t base, size_t height, CanvasDirection dir);

// Text
void canvas_draw_str(Canvas* canvas, int32_t x, int32_t y, const char* str);
void canvas_draw_str_aligned(Canvas* canvas, int32_t x, int32_t y, Align horizontal, Align vertical, const char* str);
uint16_t canvas_string_width(Canvas* canvas, const char* str);

// Bitmaps
void canvas_draw_xbm(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, const uint8_t* bitmap);
