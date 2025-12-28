#include "canvas.h"

// XOR-safe circle drawing helpers (forward declarations)
static void canvas_draw_xor_circle(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t r);
static void canvas_draw_xor_disc(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t r);

void canvas_init(canvas_t* canvas, u8g2_t* u8g2) {
    canvas->u8g2 = u8g2;
    canvas->current_color = COLOR_BLACK;
}

u8g2_t* canvas_get_u8g2(canvas_t* canvas) {
    return canvas->u8g2;
}

uint32_t canvas_width(const canvas_t* canvas) {
    (void)canvas;
    return SCREEN_WIDTH;
}

uint32_t canvas_height(const canvas_t* canvas) {
    (void)canvas;
    return SCREEN_HEIGHT;
}

void canvas_clear(canvas_t* canvas) {
    u8g2_ClearBuffer(canvas->u8g2);
}

void canvas_set_color(canvas_t* canvas, canvas_color_t color) {
    canvas->current_color = color;
    u8g2_SetDrawColor(canvas->u8g2, (uint8_t)color);
}

void canvas_set_font(canvas_t* canvas, canvas_font_t font) {
    switch (font) {
    case FONT_PRIMARY:
        u8g2_SetFont(canvas->u8g2, u8g2_font_6x10_tf);
        break;
    case FONT_SECONDARY:
        u8g2_SetFont(canvas->u8g2, u8g2_font_5x7_tf);
        break;
    case FONT_KEYBOARD:
        u8g2_SetFont(canvas->u8g2, u8g2_font_5x8_tf);
        break;
    case FONT_BIG_NUMBERS:
        u8g2_SetFont(canvas->u8g2, u8g2_font_10x20_tf);
        break;
    default:
        u8g2_SetFont(canvas->u8g2, u8g2_font_6x10_tf);
        break;
    }
}

canvas_color_t canvas_get_color(const canvas_t* canvas) {
    return canvas->current_color;
}

void canvas_draw_dot(canvas_t* canvas, int32_t x, int32_t y) {
    u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y);
}

void canvas_draw_line(canvas_t* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    u8g2_DrawLine(canvas->u8g2,
                  (u8g2_uint_t)x1, (u8g2_uint_t)y1,
                  (u8g2_uint_t)x2, (u8g2_uint_t)y2);
}

void canvas_draw_frame(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    u8g2_DrawFrame(canvas->u8g2,
                   (u8g2_uint_t)x, (u8g2_uint_t)y,
                   (u8g2_uint_t)w, (u8g2_uint_t)h);
}

void canvas_draw_box(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    u8g2_DrawBox(canvas->u8g2,
                 (u8g2_uint_t)x, (u8g2_uint_t)y,
                 (u8g2_uint_t)w, (u8g2_uint_t)h);
}

void canvas_draw_rframe(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    u8g2_DrawRFrame(canvas->u8g2,
                    (u8g2_uint_t)x, (u8g2_uint_t)y,
                    (u8g2_uint_t)w, (u8g2_uint_t)h,
                    (u8g2_uint_t)r);
}

void canvas_draw_rbox(canvas_t* canvas, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    u8g2_DrawRBox(canvas->u8g2,
                  (u8g2_uint_t)x, (u8g2_uint_t)y,
                  (u8g2_uint_t)w, (u8g2_uint_t)h,
                  (u8g2_uint_t)r);
}

void canvas_draw_circle(canvas_t* canvas, int32_t x, int32_t y, uint32_t r) {
    // Use XOR-safe version when in XOR mode to avoid ragged edges from duplicate pixels
    if (canvas->current_color == COLOR_XOR) {
        canvas_draw_xor_circle(canvas, x, y, r);
    } else {
        u8g2_DrawCircle(canvas->u8g2,
                        (u8g2_uint_t)x, (u8g2_uint_t)y,
                        (u8g2_uint_t)r, U8G2_DRAW_ALL);
    }
}

void canvas_draw_disc(canvas_t* canvas, int32_t x, int32_t y, uint32_t r) {
    // Use XOR-safe version when in XOR mode to avoid ragged edges from duplicate pixels
    if (canvas->current_color == COLOR_XOR) {
        canvas_draw_xor_disc(canvas, x, y, r);
    } else {
        u8g2_DrawDisc(canvas->u8g2,
                      (u8g2_uint_t)x, (u8g2_uint_t)y,
                      (u8g2_uint_t)r, U8G2_DRAW_ALL);
    }
}

void canvas_draw_str(canvas_t* canvas, int32_t x, int32_t y, const char* str) {
    if (str != NULL) {
        u8g2_DrawUTF8(canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, str);
    }
}

uint32_t canvas_string_width(canvas_t* canvas, const char* str) {
    if (str == NULL) {
        return 0;
    }
    return u8g2_GetStrWidth(canvas->u8g2, str);
}

// ============================================================================
// XOR-Safe Circle Drawing
// ============================================================================

/**
 * XOR-safe circle outline drawing using midpoint algorithm.
 * Ensures no pixel is drawn twice, which would cause incorrect XOR results.
 */
static void canvas_draw_xor_circle(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t r) {
    if (r == 0) {
        u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)y0);
        return;
    }

    int32_t x = 0;
    int32_t y = (int32_t)r;
    int32_t d = 1 - (int32_t)r;

    while (x <= y) {
        // Draw 8 octants, but handle special cases to avoid duplicate pixels:
        // - When x == y (45-degree points), only draw 4 pixels
        // - When x == 0 (axis points), only draw 4 pixels

        if (x == y) {
            // At 45 degrees: only 4 unique pixels
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 - y));
        } else if (x == 0) {
            // On axes: 4 unique pixels
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + y), (u8g2_uint_t)y0);
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - y), (u8g2_uint_t)y0);
        } else {
            // General case: 8 unique pixels
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + y), (u8g2_uint_t)(y0 + x));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - y), (u8g2_uint_t)(y0 + x));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + y), (u8g2_uint_t)(y0 - x));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - y), (u8g2_uint_t)(y0 - x));
        }

        if (d < 0) {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y--;
        }
        x++;
    }
}

/**
 * XOR-safe filled circle (disc) drawing using scanline approach.
 * Each pixel is drawn exactly once, avoiding duplicate pixel issues in XOR mode.
 */
static void canvas_draw_xor_disc(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t r) {
    if (r == 0) {
        u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)y0);
        return;
    }

    int32_t radius = (int32_t)r;

    // Draw horizontal scanlines from top to bottom
    // For each y, calculate x extent using circle equation: x^2 + y^2 = r^2
    for (int32_t dy = -radius; dy <= radius; dy++) {
        int32_t y_sq = dy * dy;
        int32_t r_sq = radius * radius;
        if (y_sq > r_sq) continue;

        // Integer square root approximation to find x extent
        int32_t x_extent = 0;
        while ((x_extent + 1) * (x_extent + 1) <= r_sq - y_sq) {
            x_extent++;
        }

        // Draw horizontal line from x0-x_extent to x0+x_extent
        int32_t line_y = y0 + dy;
        int32_t line_x = x0 - x_extent;
        uint32_t line_w = (uint32_t)(2 * x_extent + 1);

        u8g2_DrawHLine(canvas->u8g2,
                       (u8g2_uint_t)line_x,
                       (u8g2_uint_t)line_y,
                       (u8g2_uint_t)line_w);
    }
}
