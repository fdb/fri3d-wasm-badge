#include "canvas.h"

static void canvas_draw_xor_circle(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t radius);
static void canvas_draw_xor_disc(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t radius);

void canvas_init(canvas_t* canvas, u8g2_t* u8g2) {
    if (!canvas) {
        return;
    }

    canvas->u8g2 = u8g2;
    canvas->current_color = ColorBlack;
}

u8g2_t* canvas_get_u8g2(canvas_t* canvas) {
    return canvas ? canvas->u8g2 : NULL;
}

uint32_t canvas_width(const canvas_t* canvas) {
    (void)canvas;
    return (uint32_t)FRI3D_SCREEN_WIDTH;
}

uint32_t canvas_height(const canvas_t* canvas) {
    (void)canvas;
    return (uint32_t)FRI3D_SCREEN_HEIGHT;
}

void canvas_clear(canvas_t* canvas) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_ClearBuffer(canvas->u8g2);
}

void canvas_set_color(canvas_t* canvas, Color color) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    canvas->current_color = color;
    u8g2_SetDrawColor(canvas->u8g2, (uint8_t)color);
}

void canvas_set_font(canvas_t* canvas, Font font) {
    if (!canvas || !canvas->u8g2) {
        return;
    }

    u8g2_SetFontMode(canvas->u8g2, 1);

    switch (font) {
        case FontPrimary:
            u8g2_SetFont(canvas->u8g2, u8g2_font_helvB08_tr);
            break;
        case FontSecondary:
            u8g2_SetFont(canvas->u8g2, u8g2_font_haxrcorp4089_tr);
            break;
        case FontKeyboard:
            u8g2_SetFont(canvas->u8g2, u8g2_font_profont11_mr);
            break;
        case FontBigNumbers:
            u8g2_SetFont(canvas->u8g2, u8g2_font_profont22_tn);
            break;
        default:
            u8g2_SetFont(canvas->u8g2, u8g2_font_helvB08_tr);
            break;
    }
}

Color canvas_get_color(const canvas_t* canvas) {
    return canvas ? canvas->current_color : ColorBlack;
}

void canvas_draw_dot(canvas_t* canvas, int32_t x, int32_t y) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y);
}

void canvas_draw_line(canvas_t* canvas, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawLine(canvas->u8g2,
                  (u8g2_uint_t)x1, (u8g2_uint_t)y1,
                  (u8g2_uint_t)x2, (u8g2_uint_t)y2);
}

void canvas_draw_frame(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawFrame(canvas->u8g2,
                   (u8g2_uint_t)x, (u8g2_uint_t)y,
                   (u8g2_uint_t)width, (u8g2_uint_t)height);
}

void canvas_draw_box(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawBox(canvas->u8g2,
                 (u8g2_uint_t)x, (u8g2_uint_t)y,
                 (u8g2_uint_t)width, (u8g2_uint_t)height);
}

void canvas_draw_rframe(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawRFrame(canvas->u8g2,
                    (u8g2_uint_t)x, (u8g2_uint_t)y,
                    (u8g2_uint_t)width, (u8g2_uint_t)height,
                    (u8g2_uint_t)radius);
}

void canvas_draw_rbox(canvas_t* canvas, int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }
    u8g2_DrawRBox(canvas->u8g2,
                  (u8g2_uint_t)x, (u8g2_uint_t)y,
                  (u8g2_uint_t)width, (u8g2_uint_t)height,
                  (u8g2_uint_t)radius);
}

void canvas_draw_circle(canvas_t* canvas, int32_t x, int32_t y, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }

    if (canvas->current_color == ColorXOR) {
        canvas_draw_xor_circle(canvas, x, y, radius);
    } else {
        u8g2_DrawCircle(canvas->u8g2,
                        (u8g2_uint_t)x, (u8g2_uint_t)y,
                        (u8g2_uint_t)radius, U8G2_DRAW_ALL);
    }
}

void canvas_draw_disc(canvas_t* canvas, int32_t x, int32_t y, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }

    if (canvas->current_color == ColorXOR) {
        canvas_draw_xor_disc(canvas, x, y, radius);
    } else {
        u8g2_DrawDisc(canvas->u8g2,
                      (u8g2_uint_t)x, (u8g2_uint_t)y,
                      (u8g2_uint_t)radius, U8G2_DRAW_ALL);
    }
}

void canvas_draw_str(canvas_t* canvas, int32_t x, int32_t y, const char* str) {
    if (!canvas || !canvas->u8g2 || !str) {
        return;
    }
    u8g2_DrawUTF8(canvas->u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, str);
}

uint32_t canvas_string_width(canvas_t* canvas, const char* str) {
    if (!canvas || !canvas->u8g2 || !str) {
        return 0;
    }
    return (uint32_t)u8g2_GetUTF8Width(canvas->u8g2, str);
}

// ============================================================================
// XOR-Safe Circle Drawing
// ============================================================================

static void canvas_draw_xor_circle(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }

    if (radius == 0) {
        u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)y0);
        return;
    }

    int32_t x = 0;
    int32_t y = (int32_t)radius;
    int32_t d = 1 - (int32_t)radius;

    while (x <= y) {
        if (x == y) {
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + x), (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - x), (u8g2_uint_t)(y0 - y));
        } else if (x == 0) {
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)(y0 + y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)(y0 - y));
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 + y), (u8g2_uint_t)y0);
            u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)(x0 - y), (u8g2_uint_t)y0);
        } else {
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

static void canvas_draw_xor_disc(canvas_t* canvas, int32_t x0, int32_t y0, uint32_t radius) {
    if (!canvas || !canvas->u8g2) {
        return;
    }

    if (radius == 0) {
        u8g2_DrawPixel(canvas->u8g2, (u8g2_uint_t)x0, (u8g2_uint_t)y0);
        return;
    }

    int32_t radius_i = (int32_t)radius;

    for (int32_t dy = -radius_i; dy <= radius_i; dy++) {
        int32_t y_sq = dy * dy;
        int32_t r_sq = radius_i * radius_i;
        if (y_sq > r_sq) {
            continue;
        }

        int32_t x_extent = 0;
        while ((x_extent + 1) * (x_extent + 1) <= r_sq - y_sq) {
            x_extent++;
        }

        int32_t line_y = y0 + dy;
        int32_t line_x = x0 - x_extent;
        uint32_t line_w = (uint32_t)(2 * x_extent + 1);

        u8g2_DrawHLine(canvas->u8g2,
                       (u8g2_uint_t)line_x,
                       (u8g2_uint_t)line_y,
                       (u8g2_uint_t)line_w);
    }
}
