#pragma once

#include <stdint.h>

// 1:1 port of fri3d-runtime/src/canvas.rs semantics. 128x64 mono, 1 byte per
// pixel: 0 = white (background), 1 = black (foreground). Kept bit-exact with
// the Rust reference — verified by tools/canvas-parity-gen + the C++ parity
// runner under firmware/tools/canvas-parity/.

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
    void draw_frame (int32_t x, int32_t y, uint32_t w, uint32_t h);
    void draw_box   (int32_t x, int32_t y, uint32_t w, uint32_t h);
    void draw_rframe(int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t radius);
    void draw_rbox  (int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t radius);
    void draw_circle(int32_t x0, int32_t y0, uint32_t radius);
    void draw_disc  (int32_t x0, int32_t y0, uint32_t radius);

private:
    // Quadrant bits for rounded-corner helpers. Matches canvas.rs.
    static constexpr uint8_t DRAW_UPPER_RIGHT = 0x01;
    static constexpr uint8_t DRAW_UPPER_LEFT  = 0x02;
    static constexpr uint8_t DRAW_LOWER_LEFT  = 0x04;
    static constexpr uint8_t DRAW_LOWER_RIGHT = 0x08;
    static constexpr uint8_t DRAW_ALL = DRAW_UPPER_RIGHT | DRAW_UPPER_LEFT
                                      | DRAW_LOWER_LEFT  | DRAW_LOWER_RIGHT;

    void set_pixel(int32_t x, int32_t y, Color c);
    void draw_hline_c(int32_t x, int32_t y, uint32_t length, Color c);
    void draw_vline_c(int32_t x, int32_t y, uint32_t length, Color c);

    void draw_circle_section(int32_t x, int32_t y, int32_t x0, int32_t y0, uint8_t option, Color c);
    void draw_circle_with_option(int32_t x0, int32_t y0, int32_t radius, uint8_t option, Color c);
    void draw_disc_section(int32_t x, int32_t y, int32_t x0, int32_t y0, uint8_t option, Color c);
    void draw_disc_with_option(int32_t x0, int32_t y0, int32_t radius, uint8_t option, Color c);

    // XOR-mode circle/disc need dedicated paths because Bresenham visits some
    // pixels twice; a naive XOR of the doubled pixels cancels them out.
    void draw_xor_circle(int32_t x0, int32_t y0, uint32_t radius);
    void draw_xor_disc  (int32_t x0, int32_t y0, uint32_t radius);

    uint8_t m_buffer[SCREEN_WIDTH * SCREEN_HEIGHT];
    Color   m_color = Color::Black;
};

} // namespace fri3d
