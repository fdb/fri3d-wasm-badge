// 1:1 port of fri3d-runtime/src/font.rs. Drives u8g2 source-format fonts
// (see fonts.h) to render text into a Canvas (or anything implementing the
// FontDrawTarget interface below).
//
// Bit-level behaviour must stay byte-identical with the Rust reference —
// verified by the expanded canvas parity suite at firmware/tools/canvas-parity.

#pragma once

#include <stdint.h>
#include <stddef.h>

namespace fri3d {

// Forward-declared here so font.h stays canvas-free; canvas.h is the
// authoritative source for Color and FontId.
enum class Color : uint8_t;
enum class FontId : uint32_t;

// Callers only need a draw_hline primitive to render glyphs. Canvas already
// exposes this; any other target (e.g. tests that capture draw ops) can
// implement it trivially.
class FontDrawTarget {
public:
    virtual ~FontDrawTarget() = default;
    virtual void draw_hline_with_color(int32_t x, int32_t y, uint32_t length, Color c) = 0;
};

class Font {
public:
    // Construct from a u8g2-format byte array in fonts.h.
    explicit Font(const uint8_t* data, size_t len);

    // Convenience: resolve one of the four built-in FontIds.
    static Font from_id(FontId id);

    void draw_str(FontDrawTarget& target, int32_t x, int32_t y,
                  const char* text, Color color) const;

    uint32_t string_width(const char* text) const;

private:
    struct Info {
        uint8_t glyph_cnt;
        uint8_t bbx_mode;
        uint8_t bits_per_0;
        uint8_t bits_per_1;
        uint8_t bits_per_char_width;
        uint8_t bits_per_char_height;
        uint8_t bits_per_char_x;
        uint8_t bits_per_char_y;
        uint8_t bits_per_delta_x;
        int8_t  max_char_width;
        int8_t  max_char_height;
        int8_t  x_offset;
        int8_t  y_offset;
        int8_t  ascent_a;
        int8_t  descent_g;
        int8_t  ascent_paren;
        int8_t  descent_paren;
        uint16_t start_pos_upper_a;
        uint16_t start_pos_lower_a;
        uint16_t start_pos_unicode;
    };

    struct Metrics {
        int32_t glyph_width;
        int32_t x_offset;
        int32_t delta_x;
    };

    bool glyph_data_offset(uint16_t encoding, size_t& out) const;
    bool glyph_metrics(uint16_t encoding, Metrics& out) const;
    int32_t draw_glyph(FontDrawTarget& target, int32_t x, int32_t y,
                       uint16_t encoding, Color color) const;

    uint8_t byte_at(size_t idx) const;
    uint16_t read_u16(size_t idx) const;

    static Info info_from_data(const uint8_t* data, size_t len);

    const uint8_t* m_data;
    size_t         m_len;
    Info           m_info;
};

} // namespace fri3d
