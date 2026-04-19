// 1:1 port of fri3d-runtime/src/canvas.rs primitives.
// Preserves exact pixel semantics so the hardware rendering matches the emulator
// byte-for-byte for the same app inputs.

#include "canvas.h"

#include <string.h>

namespace fri3d {

Canvas::Canvas() {
    clear();
}

void Canvas::clear() {
    memset(m_buffer, 0, sizeof(m_buffer));
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

void Canvas::draw_dot(int32_t x, int32_t y) {
    set_pixel(x, y, m_color);
}

void Canvas::draw_hline(int32_t x, int32_t y, uint32_t length) {
    draw_hline_c(x, y, length, m_color);
}

void Canvas::draw_vline(int32_t x, int32_t y, uint32_t length) {
    draw_vline_c(x, y, length, m_color);
}

// Bresenham, matching canvas.rs:73 line-by-line.
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
    draw_hline_c(x,             y,               w, m_color);
    draw_hline_c(x,             y + (int32_t)h - 1, w, m_color);
    draw_vline_c(x,             y,               h, m_color);
    draw_vline_c(x + (int32_t)w - 1, y,           h, m_color);
}

void Canvas::draw_box(int32_t x, int32_t y, uint32_t w, uint32_t h) {
    if (w == 0 || h == 0) return;
    for (uint32_t row = 0; row < h; ++row) {
        draw_hline_c(x, y + (int32_t)row, w, m_color);
    }
}

void Canvas::draw_circle_section(int32_t x, int32_t y, int32_t x0, int32_t y0, Color c) {
    set_pixel(x0 + x, y0 - y, c);
    set_pixel(x0 + y, y0 - x, c);
    set_pixel(x0 - x, y0 - y, c);
    set_pixel(x0 - y, y0 - x, c);
    set_pixel(x0 + x, y0 + y, c);
    set_pixel(x0 + y, y0 + x, c);
    set_pixel(x0 - x, y0 + y, c);
    set_pixel(x0 - y, y0 + x, c);
}

void Canvas::draw_circle(int32_t x0, int32_t y0, uint32_t radius) {
    int32_t r = (int32_t)radius;
    if (r < 0) return;

    int32_t f = 1 - r;
    int32_t dd_f_x = 1;
    int32_t dd_f_y = -2 * r;
    int32_t x = 0;
    int32_t y = r;

    draw_circle_section(x, y, x0, y0, m_color);

    while (x < y) {
        if (f >= 0) { y -= 1; dd_f_y += 2; f += dd_f_y; }
        x += 1; dd_f_x += 2; f += dd_f_x;
        draw_circle_section(x, y, x0, y0, m_color);
    }
}

void Canvas::draw_disc_section(int32_t x, int32_t y, int32_t x0, int32_t y0, Color c) {
    draw_vline_c(x0 + x, y0 - y, (uint32_t)(y + 1), c);
    draw_vline_c(x0 + y, y0 - x, (uint32_t)(x + 1), c);
    draw_vline_c(x0 - x, y0 - y, (uint32_t)(y + 1), c);
    draw_vline_c(x0 - y, y0 - x, (uint32_t)(x + 1), c);
    draw_vline_c(x0 + x, y0,     (uint32_t)(y + 1), c);
    draw_vline_c(x0 + y, y0,     (uint32_t)(x + 1), c);
    draw_vline_c(x0 - x, y0,     (uint32_t)(y + 1), c);
    draw_vline_c(x0 - y, y0,     (uint32_t)(x + 1), c);
}

void Canvas::draw_disc(int32_t x0, int32_t y0, uint32_t radius) {
    int32_t r = (int32_t)radius;
    if (r < 0) return;

    int32_t f = 1 - r;
    int32_t dd_f_x = 1;
    int32_t dd_f_y = -2 * r;
    int32_t x = 0;
    int32_t y = r;

    draw_disc_section(x, y, x0, y0, m_color);

    while (x < y) {
        if (f >= 0) { y -= 1; dd_f_y += 2; f += dd_f_y; }
        x += 1; dd_f_x += 2; f += dd_f_x;
        draw_disc_section(x, y, x0, y0, m_color);
    }
}

} // namespace fri3d
