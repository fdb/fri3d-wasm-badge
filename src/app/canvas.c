#include "canvas.h"
#include <u8g2.h>
#include <stdlib.h>

#define CANVAS_WIDTH 128
#define CANVAS_HEIGHT 64
#define CANVAS_BUFFER_SIZE (CANVAS_WIDTH * CANVAS_HEIGHT / 8)

struct Canvas {
    u8g2_t u8g2;
    Color current_color;
};

// Dummy callbacks for u8g2 (we don't use real I2C/SPI)
static uint8_t u8x8_gpio_and_delay_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

static uint8_t u8x8_byte_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

Canvas* canvas_create(void) {
    Canvas* canvas = malloc(sizeof(Canvas));
    if (!canvas) return NULL;

    canvas->current_color = ColorBlack;

    // Initialize u8g2 with SSD1306 128x64 full buffer mode
    u8g2_Setup_ssd1306_128x64_noname_f(&canvas->u8g2, U8G2_R0, u8x8_byte_callback, u8x8_gpio_and_delay_callback);
    u8g2_InitDisplay(&canvas->u8g2);
    u8g2_SetPowerSave(&canvas->u8g2, 0);
    u8g2_ClearBuffer(&canvas->u8g2);

    return canvas;
}

void canvas_destroy(Canvas* canvas) {
    if (canvas) {
        free(canvas);
    }
}

uint8_t* canvas_get_buffer(Canvas* canvas) {
    return u8g2_GetBufferPtr(&canvas->u8g2);
}

size_t canvas_get_buffer_size(Canvas* canvas) {
    (void)canvas;
    return CANVAS_BUFFER_SIZE;
}

size_t canvas_width(const Canvas* canvas) {
    (void)canvas;
    return CANVAS_WIDTH;
}

size_t canvas_height(const Canvas* canvas) {
    (void)canvas;
    return CANVAS_HEIGHT;
}

void canvas_set_color(Canvas* canvas, Color color) {
    canvas->current_color = color;
    u8g2_SetDrawColor(&canvas->u8g2, (uint8_t)color);
}

void canvas_invert_color(Canvas* canvas) {
    if (canvas->current_color == ColorBlack) {
        canvas_set_color(canvas, ColorWhite);
    } else if (canvas->current_color == ColorWhite) {
        canvas_set_color(canvas, ColorBlack);
    }
    // XOR stays XOR
}

void canvas_set_font(Canvas* canvas, Font font) {
    // Map Font enum to u8g2 fonts
    // Using some basic built-in fonts for now
    switch (font) {
    case FontPrimary:
        u8g2_SetFont(&canvas->u8g2, u8g2_font_6x10_tf);
        break;
    case FontSecondary:
        u8g2_SetFont(&canvas->u8g2, u8g2_font_5x7_tf);
        break;
    case FontKeyboard:
        u8g2_SetFont(&canvas->u8g2, u8g2_font_5x8_tf);
        break;
    case FontBigNumbers:
        u8g2_SetFont(&canvas->u8g2, u8g2_font_10x20_tf);
        break;
    default:
        u8g2_SetFont(&canvas->u8g2, u8g2_font_6x10_tf);
        break;
    }
}

void canvas_set_custom_u8g2_font(Canvas* canvas, const uint8_t* font) {
    u8g2_SetFont(&canvas->u8g2, font);
}

void canvas_set_bitmap_mode(Canvas* canvas, bool alpha) {
    u8g2_SetBitmapMode(&canvas->u8g2, alpha ? 1 : 0);
}

void canvas_clear(Canvas* canvas) {
    u8g2_ClearBuffer(&canvas->u8g2);
}

void canvas_draw_dot(Canvas* canvas, int32_t x, int32_t y) {
    u8g2_DrawPixel(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y);
}

void canvas_draw_line(Canvas* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    u8g2_DrawLine(&canvas->u8g2, (u8g2_uint_t)x1, (u8g2_uint_t)y1, (u8g2_uint_t)x2, (u8g2_uint_t)y2);
}

void canvas_draw_frame(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height) {
    u8g2_DrawFrame(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)width, (u8g2_uint_t)height);
}

void canvas_draw_box(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height) {
    u8g2_DrawBox(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)width, (u8g2_uint_t)height);
}

void canvas_draw_rframe(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, size_t radius) {
    u8g2_DrawRFrame(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)width, (u8g2_uint_t)height, (u8g2_uint_t)radius);
}

void canvas_draw_rbox(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, size_t radius) {
    u8g2_DrawRBox(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)width, (u8g2_uint_t)height, (u8g2_uint_t)radius);
}

void canvas_draw_circle(Canvas* canvas, int32_t x, int32_t y, size_t radius) {
    u8g2_DrawCircle(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)radius, U8G2_DRAW_ALL);
}

void canvas_draw_disc(Canvas* canvas, int32_t x, int32_t y, size_t radius) {
    u8g2_DrawDisc(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)radius, U8G2_DRAW_ALL);
}

void canvas_draw_triangle(Canvas* canvas, int32_t x, int32_t y, size_t base, size_t height, CanvasDirection dir) {
    int32_t x0, y0, x1, y1, x2, y2;

    switch (dir) {
    case CanvasDirectionUp:
        x0 = x;
        y0 = y;
        x1 = x - (int32_t)(base / 2);
        y1 = y + (int32_t)height;
        x2 = x + (int32_t)(base / 2);
        y2 = y + (int32_t)height;
        break;
    case CanvasDirectionDown:
        x0 = x;
        y0 = y + (int32_t)height;
        x1 = x - (int32_t)(base / 2);
        y1 = y;
        x2 = x + (int32_t)(base / 2);
        y2 = y;
        break;
    case CanvasDirectionLeft:
        x0 = x;
        y0 = y;
        x1 = x + (int32_t)height;
        y1 = y - (int32_t)(base / 2);
        x2 = x + (int32_t)height;
        y2 = y + (int32_t)(base / 2);
        break;
    case CanvasDirectionRight:
        x0 = x + (int32_t)height;
        y0 = y;
        x1 = x;
        y1 = y - (int32_t)(base / 2);
        x2 = x;
        y2 = y + (int32_t)(base / 2);
        break;
    default:
        return;
    }

    u8g2_DrawTriangle(&canvas->u8g2, x0, y0, x1, y1, x2, y2);
}

void canvas_draw_str(Canvas* canvas, int32_t x, int32_t y, const char* str) {
    u8g2_DrawUTF8(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, str);
}

void canvas_draw_str_aligned(Canvas* canvas, int32_t x, int32_t y, Align horizontal, Align vertical, const char* str) {
    uint16_t width = u8g2_GetStrWidth(&canvas->u8g2, str);
    int8_t height = u8g2_GetAscent(&canvas->u8g2) - u8g2_GetDescent(&canvas->u8g2);

    int32_t draw_x = x;
    int32_t draw_y = y;

    // Horizontal alignment
    switch (horizontal) {
    case AlignLeft:
        break;
    case AlignCenter:
        draw_x = x - width / 2;
        break;
    case AlignRight:
        draw_x = x - width;
        break;
    default:
        break;
    }

    // Vertical alignment
    switch (vertical) {
    case AlignTop:
        draw_y = y + u8g2_GetAscent(&canvas->u8g2);
        break;
    case AlignCenter:
        draw_y = y + height / 2;
        break;
    case AlignBottom:
        draw_y = y - u8g2_GetDescent(&canvas->u8g2);
        break;
    default:
        break;
    }

    u8g2_DrawUTF8(&canvas->u8g2, (u8g2_uint_t)draw_x, (u8g2_uint_t)draw_y, str);
}

uint16_t canvas_string_width(Canvas* canvas, const char* str) {
    return u8g2_GetStrWidth(&canvas->u8g2, str);
}

void canvas_draw_xbm(Canvas* canvas, int32_t x, int32_t y, size_t width, size_t height, const uint8_t* bitmap) {
    u8g2_DrawXBM(&canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)width, (u8g2_uint_t)height, bitmap);
}
