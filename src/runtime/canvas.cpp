#include "canvas.h"
#include "display.h"

namespace fri3d {

void Canvas::init(u8g2_t* u8g2) {
    m_u8g2 = u8g2;
    m_currentColor = Color::Black;
}

uint32_t Canvas::width() const {
    return SCREEN_WIDTH;
}

uint32_t Canvas::height() const {
    return SCREEN_HEIGHT;
}

void Canvas::clear() {
    u8g2_ClearBuffer(m_u8g2);
}

void Canvas::setColor(Color color) {
    m_currentColor = color;
    u8g2_SetDrawColor(m_u8g2, static_cast<uint8_t>(color));
}

void Canvas::setFont(Font font) {
    switch (font) {
    case Font::Primary:
        u8g2_SetFont(m_u8g2, u8g2_font_6x10_tf);
        break;
    case Font::Secondary:
        u8g2_SetFont(m_u8g2, u8g2_font_5x7_tf);
        break;
    case Font::Keyboard:
        u8g2_SetFont(m_u8g2, u8g2_font_5x8_tf);
        break;
    case Font::BigNumbers:
        u8g2_SetFont(m_u8g2, u8g2_font_10x20_tf);
        break;
    default:
        u8g2_SetFont(m_u8g2, u8g2_font_6x10_tf);
        break;
    }
}

void Canvas::drawDot(int32_t x, int32_t y) {
    u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y));
}

void Canvas::drawLine(int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    u8g2_DrawLine(m_u8g2,
                  static_cast<u8g2_uint_t>(x1), static_cast<u8g2_uint_t>(y1),
                  static_cast<u8g2_uint_t>(x2), static_cast<u8g2_uint_t>(y2));
}

void Canvas::drawFrame(int32_t x, int32_t y, uint32_t w, uint32_t h) {
    u8g2_DrawFrame(m_u8g2,
                   static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                   static_cast<u8g2_uint_t>(w), static_cast<u8g2_uint_t>(h));
}

void Canvas::drawBox(int32_t x, int32_t y, uint32_t w, uint32_t h) {
    u8g2_DrawBox(m_u8g2,
                 static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                 static_cast<u8g2_uint_t>(w), static_cast<u8g2_uint_t>(h));
}

void Canvas::drawRFrame(int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    u8g2_DrawRFrame(m_u8g2,
                    static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                    static_cast<u8g2_uint_t>(w), static_cast<u8g2_uint_t>(h),
                    static_cast<u8g2_uint_t>(r));
}

void Canvas::drawRBox(int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    u8g2_DrawRBox(m_u8g2,
                  static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                  static_cast<u8g2_uint_t>(w), static_cast<u8g2_uint_t>(h),
                  static_cast<u8g2_uint_t>(r));
}

void Canvas::drawCircle(int32_t x, int32_t y, uint32_t r) {
    // Use XOR-safe version when in XOR mode to avoid ragged edges from duplicate pixels
    if (m_currentColor == Color::XOR) {
        drawXorCircle(x, y, r);
    } else {
        u8g2_DrawCircle(m_u8g2,
                        static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                        static_cast<u8g2_uint_t>(r), U8G2_DRAW_ALL);
    }
}

void Canvas::drawDisc(int32_t x, int32_t y, uint32_t r) {
    // Use XOR-safe version when in XOR mode to avoid ragged edges from duplicate pixels
    if (m_currentColor == Color::XOR) {
        drawXorDisc(x, y, r);
    } else {
        u8g2_DrawDisc(m_u8g2,
                      static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y),
                      static_cast<u8g2_uint_t>(r), U8G2_DRAW_ALL);
    }
}

void Canvas::drawStr(int32_t x, int32_t y, const char* str) {
    if (str != nullptr) {
        u8g2_DrawUTF8(m_u8g2, static_cast<u8g2_uint_t>(x), static_cast<u8g2_uint_t>(y), str);
    }
}

uint32_t Canvas::stringWidth(const char* str) {
    if (str == nullptr) {
        return 0;
    }
    return u8g2_GetStrWidth(m_u8g2, str);
}

// ============================================================================
// XOR-Safe Circle Drawing
// ============================================================================

/**
 * XOR-safe circle outline drawing using midpoint algorithm.
 * Ensures no pixel is drawn twice, which would cause incorrect XOR results.
 */
void Canvas::drawXorCircle(int32_t x0, int32_t y0, uint32_t r) {
    if (r == 0) {
        u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0), static_cast<u8g2_uint_t>(y0));
        return;
    }

    int32_t x = 0;
    int32_t y = static_cast<int32_t>(r);
    int32_t d = 1 - static_cast<int32_t>(r);

    while (x <= y) {
        // Draw 8 octants, but handle special cases to avoid duplicate pixels:
        // - When x == y (45-degree points), only draw 4 pixels
        // - When x == 0 (axis points), only draw 4 pixels

        if (x == y) {
            // At 45 degrees: only 4 unique pixels
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + x), static_cast<u8g2_uint_t>(y0 + y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - x), static_cast<u8g2_uint_t>(y0 + y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + x), static_cast<u8g2_uint_t>(y0 - y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - x), static_cast<u8g2_uint_t>(y0 - y));
        } else if (x == 0) {
            // On axes: 4 unique pixels
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0), static_cast<u8g2_uint_t>(y0 + y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0), static_cast<u8g2_uint_t>(y0 - y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + y), static_cast<u8g2_uint_t>(y0));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - y), static_cast<u8g2_uint_t>(y0));
        } else {
            // General case: 8 unique pixels
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + x), static_cast<u8g2_uint_t>(y0 + y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - x), static_cast<u8g2_uint_t>(y0 + y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + x), static_cast<u8g2_uint_t>(y0 - y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - x), static_cast<u8g2_uint_t>(y0 - y));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + y), static_cast<u8g2_uint_t>(y0 + x));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - y), static_cast<u8g2_uint_t>(y0 + x));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 + y), static_cast<u8g2_uint_t>(y0 - x));
            u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0 - y), static_cast<u8g2_uint_t>(y0 - x));
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
void Canvas::drawXorDisc(int32_t x0, int32_t y0, uint32_t r) {
    if (r == 0) {
        u8g2_DrawPixel(m_u8g2, static_cast<u8g2_uint_t>(x0), static_cast<u8g2_uint_t>(y0));
        return;
    }

    int32_t radius = static_cast<int32_t>(r);

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
        uint32_t line_w = static_cast<uint32_t>(2 * x_extent + 1);

        u8g2_DrawHLine(m_u8g2,
                       static_cast<u8g2_uint_t>(line_x),
                       static_cast<u8g2_uint_t>(line_y),
                       static_cast<u8g2_uint_t>(line_w));
    }
}

} // namespace fri3d
