use crate::fonts;
use crate::types::{Color, Font};

const FONT_DATA_STRUCT_SIZE: usize = 23;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
struct FontInfo {
    glyph_cnt: u8,
    bbx_mode: u8,
    bits_per_0: u8,
    bits_per_1: u8,
    bits_per_char_width: u8,
    bits_per_char_height: u8,
    bits_per_char_x: u8,
    bits_per_char_y: u8,
    bits_per_delta_x: u8,
    max_char_width: i8,
    max_char_height: i8,
    x_offset: i8,
    y_offset: i8,
    ascent_a: i8,
    descent_g: i8,
    ascent_paren: i8,
    descent_paren: i8,
    start_pos_upper_a: u16,
    start_pos_lower_a: u16,
    start_pos_unicode: u16,
}

impl FontInfo {
    fn from_data(data: &[u8]) -> Self {
        let get = |idx| data.get(idx).copied().unwrap_or(0);
        Self {
            glyph_cnt: get(0),
            bbx_mode: get(1),
            bits_per_0: get(2),
            bits_per_1: get(3),
            bits_per_char_width: get(4),
            bits_per_char_height: get(5),
            bits_per_char_x: get(6),
            bits_per_char_y: get(7),
            bits_per_delta_x: get(8),
            max_char_width: get(9) as i8,
            max_char_height: get(10) as i8,
            x_offset: get(11) as i8,
            y_offset: get(12) as i8,
            ascent_a: get(13) as i8,
            descent_g: get(14) as i8,
            ascent_paren: get(15) as i8,
            descent_paren: get(16) as i8,
            start_pos_upper_a: read_u16(data, 17),
            start_pos_lower_a: read_u16(data, 19),
            start_pos_unicode: read_u16(data, 21),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct GlyphMetrics {
    glyph_width: i32,
    x_offset: i32,
    delta_x: i32,
}

#[derive(Clone, Debug)]
pub struct FontData {
    data: &'static [u8],
    info: FontInfo,
}

impl FontData {
    pub fn from_font(font: Font) -> Self {
        let data = match font {
            Font::Primary => fonts::U8G2_FONT_HELVB08_TR,
            Font::Secondary => fonts::U8G2_FONT_HAXRCORP4089_TR,
            Font::Keyboard => fonts::U8G2_FONT_PROFONT11_MR,
            Font::BigNumbers => fonts::U8G2_FONT_PROFONT22_TN,
        };
        Self::new(data)
    }

    fn new(data: &'static [u8]) -> Self {
        let info = FontInfo::from_data(data);
        Self { data, info }
    }

    pub fn draw_str<T: FontDrawTarget>(
        &self,
        target: &mut T,
        x: i32,
        y: i32,
        text: &str,
        color: Color,
    ) {
        let mut decoder = Utf8Decoder::new();
        let mut cursor_x = x;
        for &byte in text.as_bytes() {
            match decoder.next(byte) {
                Utf8Result::End => break,
                Utf8Result::Continue => continue,
                Utf8Result::Codepoint(code) => {
                    let delta = self.draw_glyph(target, cursor_x, y, code, color);
                    cursor_x += delta;
                }
            }
        }
    }

    pub fn string_width(&self, text: &str) -> u32 {
        let mut decoder = Utf8Decoder::new();
        let mut width = 0i32;
        let mut last_metrics: Option<GlyphMetrics> = None;
        let mut last_delta = 0i32;

        for &byte in text.as_bytes() {
            match decoder.next(byte) {
                Utf8Result::End => break,
                Utf8Result::Continue => continue,
                Utf8Result::Codepoint(code) => {
                    if let Some(metrics) = self.glyph_metrics(code) {
                        last_delta = metrics.delta_x;
                        width += metrics.delta_x;
                        last_metrics = Some(metrics);
                    }
                }
            }
        }

        if let Some(metrics) = last_metrics {
            if metrics.glyph_width != 0 {
                width -= last_delta;
                width += metrics.glyph_width;
                width += metrics.x_offset;
            }
        }

        width.max(0) as u32
    }

    fn glyph_metrics(&self, encoding: u16) -> Option<GlyphMetrics> {
        let glyph_offset = self.glyph_data_offset(encoding)?;
        let mut decoder = FontDecoder::new(self.data, glyph_offset);
        let glyph_width = decoder.get_unsigned_bits(self.info.bits_per_char_width) as i32;
        let _glyph_height = decoder.get_unsigned_bits(self.info.bits_per_char_height);
        let x_offset = decoder.get_signed_bits(self.info.bits_per_char_x) as i32;
        decoder.get_signed_bits(self.info.bits_per_char_y);
        let delta_x = decoder.get_signed_bits(self.info.bits_per_delta_x) as i32;
        Some(GlyphMetrics {
            glyph_width,
            x_offset,
            delta_x,
        })
    }

    fn draw_glyph<T: FontDrawTarget>(
        &self,
        target: &mut T,
        x: i32,
        y: i32,
        encoding: u16,
        color: Color,
    ) -> i32 {
        let glyph_offset = match self.glyph_data_offset(encoding) {
            Some(offset) => offset,
            None => return 0,
        };
        let mut decoder = FontDecoder::new(self.data, glyph_offset);
        decoder.glyph_width = decoder.get_unsigned_bits(self.info.bits_per_char_width);
        decoder.glyph_height = decoder.get_unsigned_bits(self.info.bits_per_char_height);
        let glyph_height = decoder.glyph_height as i32;
        let x_offset = decoder.get_signed_bits(self.info.bits_per_char_x) as i32;
        let y_offset = decoder.get_signed_bits(self.info.bits_per_char_y) as i32;
        let delta_x = decoder.get_signed_bits(self.info.bits_per_delta_x) as i32;

        if decoder.glyph_width > 0 {
            let target_x = x + x_offset;
            let target_y = y - (glyph_height + y_offset);
            decoder.x = 0;
            decoder.y = 0;

            loop {
                let a = decoder.get_unsigned_bits(self.info.bits_per_0);
                let b = decoder.get_unsigned_bits(self.info.bits_per_1);
                loop {
                    decoder.decode_len(target, target_x, target_y, a, false, color);
                    decoder.decode_len(target, target_x, target_y, b, true, color);
                    if decoder.get_unsigned_bits(1) == 0 {
                        break;
                    }
                }
                if decoder.y >= decoder.glyph_height {
                    break;
                }
            }
        }

        delta_x
    }

    fn glyph_data_offset(&self, encoding: u16) -> Option<usize> {
        if self.data.len() < FONT_DATA_STRUCT_SIZE {
            return None;
        }

        if encoding <= 0xff {
            let mut index = FONT_DATA_STRUCT_SIZE;
            if encoding >= b'a' as u16 {
                index = index.saturating_add(self.info.start_pos_lower_a as usize);
            } else if encoding >= b'A' as u16 {
                index = index.saturating_add(self.info.start_pos_upper_a as usize);
            }

            while index + 1 < self.data.len() {
                let glyph_encoding = self.data[index] as u16;
                let glyph_size = self.data[index + 1] as usize;
                if glyph_size == 0 {
                    break;
                }
                if glyph_encoding == encoding {
                    return Some(index + 2);
                }
                index = index.saturating_add(glyph_size);
            }
            return None;
        }

        let unicode_start = self.info.start_pos_unicode as usize;
        if unicode_start == 0 {
            return None;
        }

        let mut font_index = FONT_DATA_STRUCT_SIZE.saturating_add(unicode_start);
        let mut lookup_index = font_index;
        loop {
            let jump = read_u16(self.data, lookup_index) as usize;
            let next_encoding = read_u16(self.data, lookup_index + 2);
            font_index = font_index.saturating_add(jump);
            lookup_index = lookup_index.saturating_add(4);
            if next_encoding >= encoding {
                break;
            }
            if lookup_index + 3 >= self.data.len() {
                return None;
            }
        }

        while font_index + 2 < self.data.len() {
            let high = self.data[font_index] as u16;
            let low = self.data[font_index + 1] as u16;
            let glyph_encoding = (high << 8) | low;
            if glyph_encoding == 0 {
                break;
            }
            let glyph_size = self.data[font_index + 2] as usize;
            if glyph_encoding == encoding {
                return Some(font_index + 3);
            }
            if glyph_size == 0 {
                break;
            }
            font_index = font_index.saturating_add(glyph_size);
        }

        None
    }
}

pub trait FontDrawTarget {
    fn draw_hline_with_color(&mut self, x: i32, y: i32, length: u32, color: Color);
}

struct FontDecoder<'a> {
    data: &'a [u8],
    ptr: usize,
    bit_pos: u8,
    glyph_width: u8,
    glyph_height: u8,
    x: u8,
    y: u8,
}

impl<'a> FontDecoder<'a> {
    fn new(data: &'a [u8], ptr: usize) -> Self {
        Self {
            data,
            ptr,
            bit_pos: 0,
            glyph_width: 0,
            glyph_height: 0,
            x: 0,
            y: 0,
        }
    }

    fn byte_at(&self, idx: usize) -> u8 {
        self.data.get(idx).copied().unwrap_or(0)
    }

    fn get_unsigned_bits(&mut self, cnt: u8) -> u8 {
        if cnt == 0 {
            return 0;
        }
        let mut val = self.byte_at(self.ptr) as u16;
        val >>= self.bit_pos;
        let mut bit_pos_plus_cnt = self.bit_pos as u16 + cnt as u16;
        if bit_pos_plus_cnt >= 8 {
            let shift = 8 - self.bit_pos;
            self.ptr = self.ptr.saturating_add(1);
            val |= (self.byte_at(self.ptr) as u16) << shift;
            bit_pos_plus_cnt = bit_pos_plus_cnt.saturating_sub(8);
        }
        let mask = (1u16 << cnt) - 1;
        val &= mask;
        self.bit_pos = bit_pos_plus_cnt as u8;
        val as u8
    }

    fn get_signed_bits(&mut self, cnt: u8) -> i8 {
        if cnt == 0 {
            return 0;
        }
        let v = self.get_unsigned_bits(cnt) as i16;
        let d = 1i16 << (cnt - 1);
        (v - d) as i8
    }

    fn decode_len<T: FontDrawTarget>(
        &mut self,
        target: &mut T,
        target_x: i32,
        target_y: i32,
        len: u8,
        is_foreground: bool,
        color: Color,
    ) {
        let mut cnt = len;
        let mut lx = self.x;
        let mut ly = self.y;

        loop {
            let rem = self.glyph_width.saturating_sub(lx);
            let current = if cnt < rem { cnt } else { rem };

            let draw_x = target_x + lx as i32;
            let draw_y = target_y + ly as i32;

            if is_foreground {
                target.draw_hline_with_color(draw_x, draw_y, current as u32, color);
            }

            if cnt < rem {
                break;
            }
            cnt = cnt.saturating_sub(rem);
            lx = 0;
            ly = ly.saturating_add(1);
        }

        lx = lx.saturating_add(cnt);
        self.x = lx;
        self.y = ly;
    }
}

struct Utf8Decoder {
    state: u8,
    encoding: u16,
}

impl Utf8Decoder {
    fn new() -> Self {
        Self {
            state: 0,
            encoding: 0,
        }
    }

    fn next(&mut self, byte: u8) -> Utf8Result {
        if byte == 0 || byte == b'\n' {
            self.state = 0;
            return Utf8Result::End;
        }

        if self.state == 0 {
            if byte >= 0xfc {
                self.state = 5;
                self.encoding = (byte & 0x01) as u16;
                return Utf8Result::Continue;
            }
            if byte >= 0xf8 {
                self.state = 4;
                self.encoding = (byte & 0x03) as u16;
                return Utf8Result::Continue;
            }
            if byte >= 0xf0 {
                self.state = 3;
                self.encoding = (byte & 0x07) as u16;
                return Utf8Result::Continue;
            }
            if byte >= 0xe0 {
                self.state = 2;
                self.encoding = (byte & 0x0f) as u16;
                return Utf8Result::Continue;
            }
            if byte >= 0xc0 {
                self.state = 1;
                self.encoding = (byte & 0x1f) as u16;
                return Utf8Result::Continue;
            }
            return Utf8Result::Codepoint(byte as u16);
        }

        self.state = self.state.saturating_sub(1);
        self.encoding <<= 6;
        self.encoding |= (byte & 0x3f) as u16;
        if self.state != 0 {
            return Utf8Result::Continue;
        }

        Utf8Result::Codepoint(self.encoding)
    }
}

enum Utf8Result {
    End,
    Continue,
    Codepoint(u16),
}

fn read_u16(data: &[u8], index: usize) -> u16 {
    let high = data.get(index).copied().unwrap_or(0) as u16;
    let low = data.get(index + 1).copied().unwrap_or(0) as u16;
    (high << 8) | low
}
