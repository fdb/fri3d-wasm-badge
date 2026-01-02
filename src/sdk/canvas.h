#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "frd.h"

typedef enum {
    ColorWhite = 0,
    ColorBlack = 1,
    ColorXOR = 2,
} Color;

typedef enum {
    AlignLeft = 0,
    AlignRight = 1,
    AlignTop = 2,
    AlignBottom = 3,
    AlignCenter = 4,
} Align;

typedef enum {
    FontPrimary = 0,
    FontSecondary = 1,
    FontKeyboard = 2,
    FontBigNumbers = 3,
} Font;

WASM_IMPORT("canvas_width") extern uint32_t canvas_width(void);
WASM_IMPORT("canvas_height") extern uint32_t canvas_height(void);

WASM_IMPORT("canvas_clear") extern void canvas_clear(void);
WASM_IMPORT("canvas_set_color") extern void canvas_set_color(Color color);
WASM_IMPORT("canvas_set_font") extern void canvas_set_font(Font font);

WASM_IMPORT("canvas_draw_dot") extern void canvas_draw_dot(int32_t x, int32_t y);
WASM_IMPORT("canvas_draw_line") extern void canvas_draw_line(int32_t x1, int32_t y1, int32_t x2, int32_t y2);

WASM_IMPORT("canvas_draw_frame") extern void canvas_draw_frame(int32_t x, int32_t y, uint32_t width, uint32_t height);
WASM_IMPORT("canvas_draw_box") extern void canvas_draw_box(int32_t x, int32_t y, uint32_t width, uint32_t height);
WASM_IMPORT("canvas_draw_rframe") extern void canvas_draw_rframe(int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius);
WASM_IMPORT("canvas_draw_rbox") extern void canvas_draw_rbox(int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius);

WASM_IMPORT("canvas_draw_circle") extern void canvas_draw_circle(int32_t x, int32_t y, uint32_t radius);
WASM_IMPORT("canvas_draw_disc") extern void canvas_draw_disc(int32_t x, int32_t y, uint32_t radius);

WASM_IMPORT("canvas_draw_str") extern void canvas_draw_str(int32_t x, int32_t y, const char* str);
WASM_IMPORT("canvas_string_width") extern uint32_t canvas_string_width(const char* str);
