#pragma once

#include <stdint.h>

// Mirrors fri3d-runtime's Canvas: 128x64 mono, 1 byte per pixel.
// 0 = white (background), 1 = black (foreground), matching canvas.rs semantics.
//
// This lets WASM apps that target the emulator be ported 1:1 to hardware:
// every canvas_draw_* host call the apps make maps onto a method here.

namespace fri3d {

constexpr uint32_t SCREEN_WIDTH = 128;
constexpr uint32_t SCREEN_HEIGHT = 64;

enum class Color : uint8_t {
    White = 0,
    Black = 1,
    Xor   = 2,
};

class Canvas {
public:
    Canvas();

    const uint8_t* buffer() const { return m_buffer; }
    uint32_t width()  const { return SCREEN_WIDTH;  }
    uint32_t height() const { return SCREEN_HEIGHT; }

    void clear();
    void set_color(Color c) { m_color = c; }
    Color color() const { return m_color; }

    void draw_dot(int32_t x, int32_t y);
    void draw_line(int32_t x1, int32_t y1, int32_t x2, int32_t y2);
    void draw_hline(int32_t x, int32_t y, uint32_t length);
    void draw_vline(int32_t x, int32_t y, uint32_t length);
    void draw_frame(int32_t x, int32_t y, uint32_t w, uint32_t h);
    void draw_box  (int32_t x, int32_t y, uint32_t w, uint32_t h);
    void draw_circle(int32_t x0, int32_t y0, uint32_t radius);
    void draw_disc  (int32_t x0, int32_t y0, uint32_t radius);

private:
    void set_pixel(int32_t x, int32_t y, Color c);
    void draw_hline_c(int32_t x, int32_t y, uint32_t length, Color c);
    void draw_vline_c(int32_t x, int32_t y, uint32_t length, Color c);
    void draw_circle_section(int32_t x, int32_t y, int32_t x0, int32_t y0, Color c);
    void draw_disc_section  (int32_t x, int32_t y, int32_t x0, int32_t y0, Color c);

    uint8_t m_buffer[SCREEN_WIDTH * SCREEN_HEIGHT];
    Color   m_color = Color::Black;
};

} // namespace fri3d
