#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "frd.h"

typedef enum {
    color_white = 0,
    color_black = 1,
    color_xor = 2,
} color_t;

typedef enum {
    align_left = 0,
    align_right = 1,
    align_center = 2,
    align_top = 3,
    align_bottom = 4,
} align_t;

typedef enum {
    font_primary = 0,
    font_secondary = 1,
    font_keyboard = 2,
    font_big_numbers = 3,
} font_t;

WASM_IMPORT("canvas_width") extern uint32_t canvas_width(void);
WASM_IMPORT("canvas_height") extern uint32_t canvas_height(void);

WASM_IMPORT("canvas_clear") extern void canvas_clear(void);
WASM_IMPORT("canvas_set_color") extern void canvas_set_color(color_t color);
WASM_IMPORT("canvas_set_font") extern void canvas_set_font(font_t font);

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
