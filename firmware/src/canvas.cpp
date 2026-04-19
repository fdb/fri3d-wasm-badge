// 1:1 port of fri3d-runtime/src/canvas.rs. Verified bit-exact against the
// Rust reference via tools/canvas-parity-gen + firmware/tools/canvas-parity.

#include "canvas.h"
#include "font.h"

#include <string.h>

namespace fri3d {

// Tiny adapter that lets Font's draw_str render into a Canvas without making
// Canvas itself inherit from FontDrawTarget (which would require virtual
// dispatch in a hot path, and a full font.h dependency in canvas.h).
class CanvasFontTarget final : public FontDrawTarget {
public:
    explicit CanvasFontTarget(Canvas& c) : m_canvas(c) {}
    void draw_hline_with_color(int32_t x, int32_t y, uint32_t length, Color c) override {
        m_canvas.draw_hline_with_color(x, y, length, c);
    }
private:
    Canvas& m_canvas;
};

Canvas::Canvas() { clear(); }

void Canvas::clear() { memset(m_buffer, 0, sizeof(m_buffer)); }

void Canvas::fill_from(const uint8_t* src, size_t len) {
    if (!src) return;
    const size_t fb_size = sizeof(m_buffer);
    const size_t n = len < fb_size ? len : fb_size;
    memcpy(m_buffer, src, n);
}

void Canvas::set_pixel(int32_t x, int32_t y, Color c) {
    if (x < 0 || y < 0) return;
    if ((uint32_t)x >= SCREEN_WIDTH || (uint32_t)y >= SCREEN_HEIGHT) return;
    const size_t idx = (size_t)y * SCREEN_WIDTH + (size_t)x;
    switch (c) {
        case Color::White: m_buffer[idx] = 0; break;
        case Color::Black: m_buffer[idx] = 1; break;
        case Color::Xor:   m_buffer[idx] ^= 1; break;
    }
}

void Canvas::draw_hline_c(int32_t x, int32_t y, uint32_t length, Color c) {
    for (uint32_t i = 0; i < length; ++i) set_pixel(x + (int32_t)i, y, c);
}

void Canvas::draw_vline_c(int32_t x, int32_t y, uint32_t length, Color c) {
    for (uint32_t i = 0; i < length; ++i) set_pixel(x, y + (int32_t)i, c);
}

void Canvas::draw_dot(int32_t x, int32_t y)                         { set_pixel(x, y, m_color); }
void Canvas::draw_hline(int32_t x, int32_t y, uint32_t length)      { draw_hline_c(x, y, length, m_color); }
void Canvas::draw_vline(int32_t x, int32_t y, uint32_t length)      { draw_vline_c(x, y, length, m_color); }

// Bresenham, matching canvas.rs:73.
void Canvas::draw_line(int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    int32_t dx = x2 - x1; if (dx < 0) dx = -dx;
    int32_t dy = y2 - y1; if (dy < 0) dy = -dy;
    bool swap_xy = false;

    if (dy > dx) {
        swap_xy = true;
        int32_t t;
        t = x1; x1 = y1; y1 = t;
        t = x2; x2 = y2; y2 = t;
        t = dx; dx = dy; dy = t;
    }
    if (x1 > x2) {
        int32_t t;
        t = x1; x1 = x2; x2 = t;
        t = y1; y1 = y2; y2 = t;
    }

    int32_t err = dx >> 1;
    int32_t ystep = (y2 > y1) ? 1 : -1;
    int32_t y = y1;

    for (int32_t x = x1; x <= x2; ++x) {
        if (!swap_xy) set_pixel(x, y, m_color);
        else          set_pixel(y, x, m_color);
        err -= dy;
        if (err < 0) { y += ystep; err += dx; }
    }
}

void Canvas::draw_frame(int32_t x, int32_t y, uint32_t w, uint32_t h) {
    if (w == 0 || h == 0) return;
    draw_hline_c(x,                   y,                   w, m_color);
    draw_hline_c(x,                   y + (int32_t)h - 1,  w, m_color);
    draw_vline_c(x,                   y,                   h, m_color);
    draw_vline_c(x + (int32_t)w - 1,  y,                   h, m_color);
}

void Canvas::draw_box(int32_t x, int32_t y, uint32_t w, uint32_t h) {
    if (w == 0 || h == 0) return;
    for (uint32_t row = 0; row < h; ++row) {
        draw_hline_c(x, y + (int32_t)row, w, m_color);
    }
}

// ---- Circle / disc with per-quadrant option --------------------------------

void Canvas::draw_circle_section(int32_t x, int32_t y, int32_t x0, int32_t y0, uint8_t option, Color c) {
    if (option & DRAW_UPPER_RIGHT) {
        set_pixel(x0 + x, y0 - y, c);
        set_pixel(x0 + y, y0 - x, c);
    }
    if (option & DRAW_UPPER_LEFT) {
        set_pixel(x0 - x, y0 - y, c);
        set_pixel(x0 - y, y0 - x, c);
    }
    if (option & DRAW_LOWER_RIGHT) {
        set_pixel(x0 + x, y0 + y, c);
        set_pixel(x0 + y, y0 + x, c);
    }
    if (option & DRAW_LOWER_LEFT) {
        set_pixel(x0 - x, y0 + y, c);
        set_pixel(x0 - y, y0 + x, c);
    }
}

void Canvas::draw_circle_with_option(int32_t x0, int32_t y0, int32_t radius, uint8_t option, Color c) {
    if (radius < 0) return;

    int32_t f = 1 - radius;
    int32_t dd_f_x = 1;
    int32_t dd_f_y = -2 * radius;
    int32_t x = 0;
    int32_t y = radius;

    draw_circle_section(x, y, x0, y0, option, c);

    while (x < y) {
        if (f >= 0) { y -= 1; dd_f_y += 2; f += dd_f_y; }
        x += 1; dd_f_x += 2; f += dd_f_x;
        draw_circle_section(x, y, x0, y0, option, c);
    }
}

void Canvas::draw_disc_section(int32_t x, int32_t y, int32_t x0, int32_t y0, uint8_t option, Color c) {
    if (option & DRAW_UPPER_RIGHT) {
        draw_vline_c(x0 + x, y0 - y, (uint32_t)(y + 1), c);
        draw_vline_c(x0 + y, y0 - x, (uint32_t)(x + 1), c);
    }
    if (option & DRAW_UPPER_LEFT) {
        draw_vline_c(x0 - x, y0 - y, (uint32_t)(y + 1), c);
        draw_vline_c(x0 - y, y0 - x, (uint32_t)(x + 1), c);
    }
    if (option & DRAW_LOWER_RIGHT) {
        draw_vline_c(x0 + x, y0, (uint32_t)(y + 1), c);
        draw_vline_c(x0 + y, y0, (uint32_t)(x + 1), c);
    }
    if (option & DRAW_LOWER_LEFT) {
        draw_vline_c(x0 - x, y0, (uint32_t)(y + 1), c);
        draw_vline_c(x0 - y, y0, (uint32_t)(x + 1), c);
    }
}

void Canvas::draw_disc_with_option(int32_t x0, int32_t y0, int32_t radius, uint8_t option, Color c) {
    if (radius < 0) return;

    int32_t f = 1 - radius;
    int32_t dd_f_x = 1;
    int32_t dd_f_y = -2 * radius;
    int32_t x = 0;
    int32_t y = radius;

    draw_disc_section(x, y, x0, y0, option, c);

    while (x < y) {
        if (f >= 0) { y -= 1; dd_f_y += 2; f += dd_f_y; }
        x += 1; dd_f_x += 2; f += dd_f_x;
        draw_disc_section(x, y, x0, y0, option, c);
    }
}

// ---- XOR circle / disc -----------------------------------------------------
// Bresenham visits several pixels twice (at octant boundaries). For solid
// colours that's harmless; for XOR it cancels the pixel. These paths visit
// each pixel exactly once.

void Canvas::draw_xor_circle(int32_t x0, int32_t y0, uint32_t radius) {
    if (radius == 0) { set_pixel(x0, y0, Color::Xor); return; }

    int32_t x = 0;
    int32_t y = (int32_t)radius;
    int32_t d = 1 - y;

    while (x <= y) {
        if (x == y) {
            set_pixel(x0 + x, y0 + y, Color::Xor);
            set_pixel(x0 - x, y0 + y, Color::Xor);
            set_pixel(x0 + x, y0 - y, Color::Xor);
            set_pixel(x0 - x, y0 - y, Color::Xor);
        } else if (x == 0) {
            set_pixel(x0,     y0 + y, Color::Xor);
            set_pixel(x0,     y0 - y, Color::Xor);
            set_pixel(x0 + y, y0,     Color::Xor);
            set_pixel(x0 - y, y0,     Color::Xor);
        } else {
            set_pixel(x0 + x, y0 + y, Color::Xor);
            set_pixel(x0 - x, y0 + y, Color::Xor);
            set_pixel(x0 + x, y0 - y, Color::Xor);
            set_pixel(x0 - x, y0 - y, Color::Xor);
            set_pixel(x0 + y, y0 + x, Color::Xor);
            set_pixel(x0 - y, y0 + x, Color::Xor);
            set_pixel(x0 + y, y0 - x, Color::Xor);
            set_pixel(x0 - y, y0 - x, Color::Xor);
        }

        if (d < 0) {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y -= 1;
        }
        x += 1;
    }
}

void Canvas::draw_xor_disc(int32_t x0, int32_t y0, uint32_t radius) {
    if (radius == 0) { set_pixel(x0, y0, Color::Xor); return; }

    const int32_t r = (int32_t)radius;
    const int32_t r_sq = r * r;

    for (int32_t dy = -r; dy <= r; ++dy) {
        const int32_t y_sq = dy * dy;
        if (y_sq > r_sq) continue;

        int32_t x_extent = 0;
        while ((x_extent + 1) * (x_extent + 1) <= r_sq - y_sq) {
            ++x_extent;
        }

        const int32_t line_y = y0 + dy;
        const int32_t line_x = x0 - x_extent;
        const uint32_t line_w = (uint32_t)(2 * x_extent + 1);

        draw_hline_c(line_x, line_y, line_w, Color::Xor);
    }
}

void Canvas::draw_circle(int32_t x0, int32_t y0, uint32_t radius) {
    if (m_color == Color::Xor) { draw_xor_circle(x0, y0, radius); return; }
    draw_circle_with_option(x0, y0, (int32_t)radius, DRAW_ALL, m_color);
}

void Canvas::draw_disc(int32_t x0, int32_t y0, uint32_t radius) {
    if (m_color == Color::Xor) { draw_xor_disc(x0, y0, radius); return; }
    draw_disc_with_option(x0, y0, (int32_t)radius, DRAW_ALL, m_color);
}

// ---- Rounded frame / box ---------------------------------------------------

void Canvas::draw_rframe(int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius) {
    if (width == 0 || height == 0) return;
    int32_t r = (int32_t)radius;
    const int32_t w = (int32_t)width;
    const int32_t h = (int32_t)height;
    if (r > w / 2) r = w / 2;
    if (r > h / 2) r = h / 2;
    if (r <= 0) { draw_frame(x, y, width, height); return; }

    int32_t xl = x + r;
    int32_t yu = y + r;
    const int32_t xr = x + w - r - 1;
    const int32_t yl = y + h - r - 1;

    draw_circle_with_option(xl, yu, r, DRAW_UPPER_LEFT,  m_color);
    draw_circle_with_option(xr, yu, r, DRAW_UPPER_RIGHT, m_color);
    draw_circle_with_option(xl, yl, r, DRAW_LOWER_LEFT,  m_color);
    draw_circle_with_option(xr, yl, r, DRAW_LOWER_RIGHT, m_color);

    int32_t ww = w - r - r;
    int32_t hh = h - r - r;

    xl += 1;
    yu += 1;

    if (ww >= 3) {
        ww -= 2;
        const int32_t edge_y = y + h - 1;
        draw_hline_c(xl, y,      (uint32_t)ww, m_color);
        draw_hline_c(xl, edge_y, (uint32_t)ww, m_color);
    }
    if (hh >= 3) {
        hh -= 2;
        const int32_t edge_x = x + w - 1;
        draw_vline_c(x,       yu, (uint32_t)hh, m_color);
        draw_vline_c(edge_x,  yu, (uint32_t)hh, m_color);
    }
}

void Canvas::draw_rbox(int32_t x, int32_t y, uint32_t width, uint32_t height, uint32_t radius) {
    if (width == 0 || height == 0) return;
    int32_t r = (int32_t)radius;
    const int32_t w = (int32_t)width;
    const int32_t h = (int32_t)height;
    if (r > w / 2) r = w / 2;
    if (r > h / 2) r = h / 2;
    if (r <= 0) { draw_box(x, y, width, height); return; }

    int32_t xl = x + r;
    int32_t yu = y + r;
    const int32_t xr = x + w - r - 1;
    const int32_t yl = y + h - r - 1;

    draw_disc_with_option(xl, yu, r, DRAW_UPPER_LEFT,  m_color);
    draw_disc_with_option(xr, yu, r, DRAW_UPPER_RIGHT, m_color);
    draw_disc_with_option(xl, yl, r, DRAW_LOWER_LEFT,  m_color);
    draw_disc_with_option(xr, yl, r, DRAW_LOWER_RIGHT, m_color);

    int32_t ww = w - r - r;
    int32_t hh = h - r - r;

    xl += 1;
    yu += 1;

    if (ww >= 3) {
        ww -= 2;
        draw_box(xl, y,  (uint32_t)ww, (uint32_t)(r + 1));
        draw_box(xl, yl, (uint32_t)ww, (uint32_t)(r + 1));
    }
    if (hh >= 3) {
        hh -= 2;
        draw_box(x, yu, (uint32_t)w, (uint32_t)hh);
    }
}

// ---- Text ------------------------------------------------------------------

void Canvas::draw_str(int32_t x, int32_t y, const char* text) {
    CanvasFontTarget target(*this);
    Font::from_id(m_font).draw_str(target, x, y, text, m_color);
}

uint32_t Canvas::string_width(const char* text) const {
    return Font::from_id(m_font).string_width(text);
}

} // namespace fri3d
