// 1:1 port of fri3d-runtime/src/font.rs.

#include "font.h"
#include "fonts.h"
#include "canvas.h" // for Color / FontId definitions (forward-declared in font.h)

#include <cstring>

namespace fri3d {

// ---- FontDecoder: bit-stream reader over packed glyph data ------------

// Must be accessible from free functions below but not part of the public
// header. Kept in an anonymous namespace to avoid ODR collisions.
namespace {

struct FontDecoder {
    const uint8_t* data;
    size_t         len;
    size_t         ptr;
    uint8_t        bit_pos;
    uint8_t        glyph_width;
    uint8_t        glyph_height;
    uint8_t        x;
    uint8_t        y;

    FontDecoder(const uint8_t* d, size_t l, size_t start)
        : data(d), len(l), ptr(start), bit_pos(0),
          glyph_width(0), glyph_height(0), x(0), y(0) {}

    uint8_t byte_at(size_t idx) const {
        return idx < len ? data[idx] : 0;
    }

    uint8_t get_unsigned_bits(uint8_t cnt) {
        if (cnt == 0) return 0;
        uint16_t val = byte_at(ptr);
        val >>= bit_pos;
        uint16_t bit_pos_plus_cnt = (uint16_t)bit_pos + cnt;
        if (bit_pos_plus_cnt >= 8) {
            uint8_t shift = 8 - bit_pos;
            ptr++;
            val |= ((uint16_t)byte_at(ptr)) << shift;
            bit_pos_plus_cnt -= 8;
        }
        uint16_t mask = (uint16_t)((1u << cnt) - 1);
        val &= mask;
        bit_pos = (uint8_t)bit_pos_plus_cnt;
        return (uint8_t)val;
    }

    int8_t get_signed_bits(uint8_t cnt) {
        if (cnt == 0) return 0;
        int16_t v = (int16_t)get_unsigned_bits(cnt);
        int16_t d = (int16_t)(1 << (cnt - 1));
        return (int8_t)(v - d);
    }

    void decode_len(FontDrawTarget& target,
                    int32_t target_x, int32_t target_y,
                    uint8_t len_bits, bool is_foreground, Color color) {
        uint8_t cnt = len_bits;
        uint8_t lx = x;
        uint8_t ly = y;
        while (true) {
            uint8_t rem = (uint8_t)(glyph_width > lx ? (glyph_width - lx) : 0);
            uint8_t current = cnt < rem ? cnt : rem;

            int32_t draw_x = target_x + (int32_t)lx;
            int32_t draw_y = target_y + (int32_t)ly;

            if (is_foreground) {
                target.draw_hline_with_color(draw_x, draw_y, current, color);
            }

            if (cnt < rem) break;
            cnt = (uint8_t)(cnt - rem);
            lx = 0;
            if (ly < 0xFF) ly++;
        }
        lx = (uint8_t)(lx + cnt);
        x = lx;
        y = ly;
    }
};

// ---- UTF-8 decoder ----------------------------------------------------

enum class Utf8Kind { End, Continue, Codepoint };

struct Utf8Decoder {
    uint8_t  state = 0;
    uint16_t encoding = 0;

    Utf8Kind next(uint8_t byte, uint16_t& out_code) {
        if (byte == 0 || byte == (uint8_t)'\n') {
            state = 0;
            return Utf8Kind::End;
        }
        if (state == 0) {
            if (byte >= 0xfc) { state = 5; encoding = byte & 0x01; return Utf8Kind::Continue; }
            if (byte >= 0xf8) { state = 4; encoding = byte & 0x03; return Utf8Kind::Continue; }
            if (byte >= 0xf0) { state = 3; encoding = byte & 0x07; return Utf8Kind::Continue; }
            if (byte >= 0xe0) { state = 2; encoding = byte & 0x0f; return Utf8Kind::Continue; }
            if (byte >= 0xc0) { state = 1; encoding = byte & 0x1f; return Utf8Kind::Continue; }
            out_code = byte;
            return Utf8Kind::Codepoint;
        }
        state = (uint8_t)(state - 1);
        encoding = (uint16_t)(encoding << 6);
        encoding |= (uint16_t)(byte & 0x3f);
        if (state != 0) return Utf8Kind::Continue;
        out_code = encoding;
        return Utf8Kind::Codepoint;
    }
};

} // anonymous namespace

// ---- Font ------------------------------------------------------------

static constexpr size_t FONT_DATA_STRUCT_SIZE = 23;

Font::Info Font::info_from_data(const uint8_t* data, size_t len) {
    auto get = [&](size_t i) -> uint8_t { return i < len ? data[i] : 0; };
    auto rd16 = [&](size_t i) -> uint16_t {
        return (uint16_t)((get(i) << 8) | get(i + 1));
    };
    Info info{};
    info.glyph_cnt           = get(0);
    info.bbx_mode            = get(1);
    info.bits_per_0          = get(2);
    info.bits_per_1          = get(3);
    info.bits_per_char_width = get(4);
    info.bits_per_char_height= get(5);
    info.bits_per_char_x     = get(6);
    info.bits_per_char_y     = get(7);
    info.bits_per_delta_x    = get(8);
    info.max_char_width      = (int8_t)get(9);
    info.max_char_height     = (int8_t)get(10);
    info.x_offset            = (int8_t)get(11);
    info.y_offset            = (int8_t)get(12);
    info.ascent_a            = (int8_t)get(13);
    info.descent_g           = (int8_t)get(14);
    info.ascent_paren        = (int8_t)get(15);
    info.descent_paren       = (int8_t)get(16);
    info.start_pos_upper_a   = rd16(17);
    info.start_pos_lower_a   = rd16(19);
    info.start_pos_unicode   = rd16(21);
    return info;
}

Font::Font(const uint8_t* data, size_t len)
    : m_data(data), m_len(len), m_info(info_from_data(data, len)) {}

const Font& Font::from_id(FontId id) {
    // Parsed once at first access, reused forever. 4 × sizeof(Font) is small
    // and avoids the 23-byte header parse that would otherwise run on every
    // Canvas::draw_str call.
    static const Font FONTS[4] = {
        Font(U8G2_FONT_HELVB08_TR,      sizeof(U8G2_FONT_HELVB08_TR)),
        Font(U8G2_FONT_HAXRCORP4089_TR, sizeof(U8G2_FONT_HAXRCORP4089_TR)),
        Font(U8G2_FONT_PROFONT11_MR,    sizeof(U8G2_FONT_PROFONT11_MR)),
        Font(U8G2_FONT_PROFONT22_TN,    sizeof(U8G2_FONT_PROFONT22_TN)),
    };
    uint32_t idx = (uint32_t)id;
    if (idx > 3) idx = 0;
    return FONTS[idx];
}

uint8_t Font::byte_at(size_t idx) const {
    return idx < m_len ? m_data[idx] : 0;
}

uint16_t Font::read_u16(size_t idx) const {
    return (uint16_t)((byte_at(idx) << 8) | byte_at(idx + 1));
}

bool Font::glyph_data_offset(uint16_t encoding, size_t& out) const {
    if (m_len < FONT_DATA_STRUCT_SIZE) return false;

    if (encoding <= 0xff) {
        size_t index = FONT_DATA_STRUCT_SIZE;
        if (encoding >= (uint16_t)'a')      index += m_info.start_pos_lower_a;
        else if (encoding >= (uint16_t)'A') index += m_info.start_pos_upper_a;

        while (index + 1 < m_len) {
            uint16_t glyph_encoding = m_data[index];
            size_t   glyph_size     = m_data[index + 1];
            if (glyph_size == 0) break;
            if (glyph_encoding == encoding) { out = index + 2; return true; }
            index += glyph_size;
        }
        return false;
    }

    size_t unicode_start = m_info.start_pos_unicode;
    if (unicode_start == 0) return false;

    size_t font_index   = FONT_DATA_STRUCT_SIZE + unicode_start;
    size_t lookup_index = font_index;
    while (true) {
        uint16_t jump          = read_u16(lookup_index);
        uint16_t next_encoding = read_u16(lookup_index + 2);
        font_index   += jump;
        lookup_index += 4;
        if (next_encoding >= encoding) break;
        if (lookup_index + 3 >= m_len) return false;
    }

    while (font_index + 2 < m_len) {
        uint16_t high = m_data[font_index];
        uint16_t low  = m_data[font_index + 1];
        uint16_t glyph_encoding = (uint16_t)((high << 8) | low);
        if (glyph_encoding == 0) break;
        size_t glyph_size = m_data[font_index + 2];
        if (glyph_encoding == encoding) { out = font_index + 3; return true; }
        if (glyph_size == 0) break;
        font_index += glyph_size;
    }

    return false;
}

bool Font::glyph_metrics(uint16_t encoding, Metrics& out) const {
    size_t off = 0;
    if (!glyph_data_offset(encoding, off)) return false;
    FontDecoder dec(m_data, m_len, off);
    out.glyph_width = (int32_t)dec.get_unsigned_bits(m_info.bits_per_char_width);
    (void)dec.get_unsigned_bits(m_info.bits_per_char_height);
    out.x_offset = (int32_t)dec.get_signed_bits(m_info.bits_per_char_x);
    (void)dec.get_signed_bits(m_info.bits_per_char_y);
    out.delta_x = (int32_t)dec.get_signed_bits(m_info.bits_per_delta_x);
    return true;
}

int32_t Font::draw_glyph(FontDrawTarget& target, int32_t x, int32_t y,
                         uint16_t encoding, Color color) const {
    size_t off = 0;
    if (!glyph_data_offset(encoding, off)) return 0;
    FontDecoder dec(m_data, m_len, off);
    dec.glyph_width  = dec.get_unsigned_bits(m_info.bits_per_char_width);
    dec.glyph_height = dec.get_unsigned_bits(m_info.bits_per_char_height);
    int32_t gh = (int32_t)dec.glyph_height;
    int32_t x_offset = (int32_t)dec.get_signed_bits(m_info.bits_per_char_x);
    int32_t y_offset = (int32_t)dec.get_signed_bits(m_info.bits_per_char_y);
    int32_t delta_x  = (int32_t)dec.get_signed_bits(m_info.bits_per_delta_x);

    if (dec.glyph_width > 0) {
        int32_t target_x = x + x_offset;
        int32_t target_y = y - (gh + y_offset);
        dec.x = 0;
        dec.y = 0;

        while (true) {
            uint8_t a = dec.get_unsigned_bits(m_info.bits_per_0);
            uint8_t b = dec.get_unsigned_bits(m_info.bits_per_1);
            while (true) {
                dec.decode_len(target, target_x, target_y, a, false, color);
                dec.decode_len(target, target_x, target_y, b, true,  color);
                if (dec.get_unsigned_bits(1) == 0) break;
            }
            if (dec.y >= dec.glyph_height) break;
        }
    }

    return delta_x;
}

void Font::draw_str(FontDrawTarget& target, int32_t x, int32_t y,
                    const char* text, Color color) const {
    if (!text) return;
    Utf8Decoder dec;
    int32_t cursor_x = x;
    for (size_t i = 0; ; ++i) {
        uint8_t byte = (uint8_t)text[i];
        uint16_t code = 0;
        Utf8Kind k = dec.next(byte, code);
        if (k == Utf8Kind::End) break;
        if (k == Utf8Kind::Continue) continue;
        cursor_x += draw_glyph(target, cursor_x, y, code, color);
    }
}

uint32_t Font::string_width(const char* text) const {
    if (!text) return 0;
    Utf8Decoder dec;
    int32_t width = 0;
    bool have_last = false;
    Metrics last{};
    int32_t last_delta = 0;

    for (size_t i = 0; ; ++i) {
        uint8_t byte = (uint8_t)text[i];
        uint16_t code = 0;
        Utf8Kind k = dec.next(byte, code);
        if (k == Utf8Kind::End) break;
        if (k == Utf8Kind::Continue) continue;

        Metrics m{};
        if (glyph_metrics(code, m)) {
            last_delta = m.delta_x;
            width += m.delta_x;
            last = m;
            have_last = true;
        }
    }

    if (have_last && last.glyph_width != 0) {
        width -= last_delta;
        width += last.glyph_width;
        width += last.x_offset;
    }

    return width < 0 ? 0u : (uint32_t)width;
}

} // namespace fri3d
