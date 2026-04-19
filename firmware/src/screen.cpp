#include "screen.h"
#include "canvas.h"

#include <string.h>

namespace fri3d {

// Adapter that lets Font's draw_str render coloured text into Screen
// without Screen needing to know anything about glyph layout. Font calls
// draw_hline_with_color(x, y, length, Color::Black|White) — we map that
// to "ink" and "no-op" runs.
class ScreenFontTarget final : public FontDrawTarget {
public:
    ScreenFontTarget(Screen& s, uint16_t color565) : m_screen(s), m_color(color565) {}
    void draw_hline_with_color(int32_t x, int32_t y, uint32_t length, Color c) override {
        // Fonts request White/Black runs (the legacy ink convention). We
        // only paint "Black" runs (the actual ink) — White runs would
        // erase background pixels we want to keep.
        if (c == Color::White) return;
        if (length == 0) return;
        for (uint32_t i = 0; i < length; ++i) {
            const int32_t px = x + (int32_t)i;
            if (px < 0 || (uint32_t)px >= SCREEN_W) continue;
            if (y  < 0 || (uint32_t)y  >= SCREEN_H) continue;
            m_screen.pixels()[(size_t)y * SCREEN_W + (size_t)px] = m_color;
        }
    }
private:
    Screen&  m_screen;
    uint16_t m_color;
};

Screen::Screen(uint16_t* pixels) : m_pixels(pixels) {
    if (m_pixels) memset(m_pixels, 0, SCREEN_BYTES);
}

void Screen::clear(uint32_t rgb) {
    if (!m_pixels) return;
    m_used = true;
    const uint16_t c = rgb_to_565(rgb);
    // memset only works for 0; do a 16-bit fill loop. The compiler should
    // unroll this nicely on ESP32-S3.
    for (size_t i = 0; i < SCREEN_PIXELS; ++i) m_pixels[i] = c;
}

void Screen::pixel(int32_t x, int32_t y, uint32_t rgb) {
    if (!m_pixels) return;
    if (x < 0 || y < 0 || (uint32_t)x >= SCREEN_W || (uint32_t)y >= SCREEN_H) return;
    m_used = true;
    m_pixels[(size_t)y * SCREEN_W + (size_t)x] = rgb_to_565(rgb);
}

void Screen::raw_hline(int32_t x, int32_t y, int32_t length, uint16_t color565) {
    if (!m_pixels || length <= 0) return;
    if (y < 0 || (uint32_t)y >= SCREEN_H) return;
    int32_t x0 = x < 0 ? 0 : x;
    int32_t x1 = x + length;
    if (x1 > (int32_t)SCREEN_W) x1 = (int32_t)SCREEN_W;
    if (x0 >= x1) return;
    uint16_t* row = &m_pixels[(size_t)y * SCREEN_W];
    for (int32_t i = x0; i < x1; ++i) row[i] = color565;
}

void Screen::hline(int32_t x, int32_t y, int32_t length, uint32_t rgb) {
    m_used = true;
    raw_hline(x, y, length, rgb_to_565(rgb));
}

void Screen::vline(int32_t x, int32_t y, int32_t length, uint32_t rgb) {
    if (!m_pixels || length <= 0) return;
    if (x < 0 || (uint32_t)x >= SCREEN_W) return;
    m_used = true;
    const uint16_t c = rgb_to_565(rgb);
    int32_t y0 = y < 0 ? 0 : y;
    int32_t y1 = y + length;
    if (y1 > (int32_t)SCREEN_H) y1 = (int32_t)SCREEN_H;
    for (int32_t row = y0; row < y1; ++row) {
        m_pixels[(size_t)row * SCREEN_W + (size_t)x] = c;
    }
}

void Screen::fill_rect(int32_t x, int32_t y, int32_t w, int32_t h, uint32_t rgb) {
    if (w <= 0 || h <= 0) return;
    m_used = true;
    const uint16_t c = rgb_to_565(rgb);
    for (int32_t row = 0; row < h; ++row) raw_hline(x, y + row, w, c);
}

void Screen::stroke_rect(int32_t x, int32_t y, int32_t w, int32_t h, uint32_t rgb) {
    if (w <= 0 || h <= 0) return;
    hline(x,         y,         w, rgb);
    hline(x,         y + h - 1, w, rgb);
    vline(x,         y,         h, rgb);
    vline(x + w - 1, y,         h, rgb);
}

// Bresenham. Same shape as canvas.cpp's draw_line — reused intentionally
// so vector apps look consistent across the two pipelines.
void Screen::line(int32_t x1, int32_t y1, int32_t x2, int32_t y2, uint32_t rgb) {
    int32_t dx = x2 - x1; if (dx < 0) dx = -dx;
    int32_t dy = y2 - y1; if (dy < 0) dy = -dy;
    bool swap_xy = dy > dx;
    if (swap_xy) {
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
    int32_t err   = dx >> 1;
    int32_t ystep = (y2 > y1) ? 1 : -1;
    int32_t y     = y1;
    for (int32_t x = x1; x <= x2; ++x) {
        if (swap_xy) pixel(y, x, rgb);
        else         pixel(x, y, rgb);
        err -= dy;
        if (err < 0) { y += ystep; err += dx; }
    }
}

// Midpoint circle. Don't bother with octant deduplication — at the screen's
// resolution a few extra pixel writes per circle is invisible vs the cost
// of drawing the call.
void Screen::circle(int32_t x0, int32_t y0, int32_t radius, uint32_t rgb) {
    if (radius < 0) return;
    int32_t f      = 1 - radius;
    int32_t ddF_x  = 1;
    int32_t ddF_y  = -2 * radius;
    int32_t x = 0, y = radius;

    pixel(x0,       y0 + y, rgb);
    pixel(x0,       y0 - y, rgb);
    pixel(x0 + y,   y0,     rgb);
    pixel(x0 - y,   y0,     rgb);

    while (x < y) {
        if (f >= 0) { y -= 1; ddF_y += 2; f += ddF_y; }
        x += 1; ddF_x += 2; f += ddF_x;

        pixel(x0 + x, y0 + y, rgb);
        pixel(x0 - x, y0 + y, rgb);
        pixel(x0 + x, y0 - y, rgb);
        pixel(x0 - x, y0 - y, rgb);
        pixel(x0 + y, y0 + x, rgb);
        pixel(x0 - y, y0 + x, rgb);
        pixel(x0 + y, y0 - x, rgb);
        pixel(x0 - y, y0 - x, rgb);
    }
}

void Screen::disc(int32_t x0, int32_t y0, int32_t radius, uint32_t rgb) {
    if (radius < 0) return;
    m_used = true;
    const uint16_t c = rgb_to_565(rgb);
    for (int32_t dy = -radius; dy <= radius; ++dy) {
        const int32_t span = (int32_t)__builtin_sqrt((double)(radius * radius - dy * dy));
        raw_hline(x0 - span, y0 + dy, 2 * span + 1, c);
    }
}

void Screen::polyline(const int32_t* pts, int32_t n, uint32_t rgb) {
    if (!pts || n < 2) return;
    for (int32_t i = 1; i < n; ++i) {
        line(pts[(i - 1) * 2], pts[(i - 1) * 2 + 1],
             pts[ i      * 2], pts[ i      * 2 + 1], rgb);
    }
}

void Screen::polygon(const int32_t* pts, int32_t n, uint32_t rgb) {
    if (!pts || n < 2) return;
    polyline(pts, n, rgb);
    line(pts[(n - 1) * 2], pts[(n - 1) * 2 + 1], pts[0], pts[1], rgb);
}

// Scanline polygon fill (even-odd rule). Doesn't bother with fancy
// edge-table sorting — at the screen's resolution and the small polygons
// apps actually draw, the dumb O(W*H*N) approach is plenty fast.
void Screen::polygon_fill(const int32_t* pts, int32_t n, uint32_t rgb) {
    if (!pts || n < 3) return;
    m_used = true;
    const uint16_t c = rgb_to_565(rgb);

    // Find polygon's vertical bounds, clipped to screen.
    int32_t y_min = pts[1], y_max = pts[1];
    for (int32_t i = 1; i < n; ++i) {
        const int32_t py = pts[i * 2 + 1];
        if (py < y_min) y_min = py;
        if (py > y_max) y_max = py;
    }
    if (y_min < 0) y_min = 0;
    if (y_max >= (int32_t)SCREEN_H) y_max = (int32_t)SCREEN_H - 1;

    // For each scanline, find x-intersections with edges, sort, fill pairs.
    // 32 vertices is enough for any geometric fox / app icon — bigger
    // polygons should be split or precomputed.
    constexpr int32_t MAX_X = 32;
    int32_t xs[MAX_X];

    for (int32_t y = y_min; y <= y_max; ++y) {
        int32_t nx = 0;
        for (int32_t i = 0, j = n - 1; i < n; j = i++) {
            const int32_t y1 = pts[i * 2 + 1];
            const int32_t y2 = pts[j * 2 + 1];
            // Edge crosses this scanline if one endpoint is above and the
            // other below (>=/< asymmetry avoids double-counting vertices).
            const bool crosses = (y1 <= y && y2 > y) || (y2 <= y && y1 > y);
            if (!crosses) continue;
            const int32_t x1 = pts[i * 2];
            const int32_t x2 = pts[j * 2];
            // Linear interpolation; keep integer math by avoiding float.
            const int32_t x_cross = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
            if (nx < MAX_X) xs[nx++] = x_cross;
        }
        // Insertion sort — nx is tiny.
        for (int32_t i = 1; i < nx; ++i) {
            const int32_t v = xs[i];
            int32_t j = i - 1;
            while (j >= 0 && xs[j] > v) { xs[j + 1] = xs[j]; --j; }
            xs[j + 1] = v;
        }
        // Fill between pairs of intersections.
        for (int32_t i = 0; i + 1 < nx; i += 2) {
            const int32_t x0 = xs[i];
            const int32_t x1 = xs[i + 1];
            raw_hline(x0, y, x1 - x0 + 1, c);
        }
    }
}

void Screen::text(int32_t x, int32_t y, const char* str, uint32_t rgb, FontId font) {
    if (!str) return;
    m_used = true;
    ScreenFontTarget target(*this, rgb_to_565(rgb));
    Font::from_id(font).draw_str(target, x, y, str, Color::Black);
}

void Screen::blit_legacy_canvas(const uint8_t* mono_buf,
                                uint32_t mono_w, uint32_t mono_h,
                                uint32_t fg_rgb, uint32_t bg_rgb) {
    if (!m_pixels || !mono_buf) return;
    // Centre + 2x upscale (256x128 lands in the middle of 296x240 with
    // 20px left/right and 56px top/bottom of background).
    const uint16_t fg = rgb_to_565(fg_rgb);
    const uint16_t bg = rgb_to_565(bg_rgb);
    const int32_t scale = 2;
    const int32_t up_w  = (int32_t)mono_w * scale;
    const int32_t up_h  = (int32_t)mono_h * scale;
    const int32_t off_x = ((int32_t)SCREEN_W - up_w) / 2;
    const int32_t off_y = ((int32_t)SCREEN_H - up_h) / 2;
    // Clear the screen background first so legacy apps look intentional
    // rather than landing on whatever stale pixels were there.
    for (size_t i = 0; i < SCREEN_PIXELS; ++i) m_pixels[i] = bg;
    for (uint32_t y = 0; y < mono_h; ++y) {
        const uint8_t* row = &mono_buf[y * mono_w];
        for (uint32_t x = 0; x < mono_w; ++x) {
            const uint16_t c = row[x] ? fg : bg;
            for (int32_t dy = 0; dy < scale; ++dy) {
                for (int32_t dx = 0; dx < scale; ++dx) {
                    const int32_t px = off_x + (int32_t)x * scale + dx;
                    const int32_t py = off_y + (int32_t)y * scale + dy;
                    if (px < 0 || py < 0) continue;
                    if ((uint32_t)px >= SCREEN_W || (uint32_t)py >= SCREEN_H) continue;
                    m_pixels[(size_t)py * SCREEN_W + (size_t)px] = c;
                }
            }
        }
    }
}

} // namespace fri3d
