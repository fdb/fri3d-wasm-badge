//! Bitmap text. Fonts are static glyph tables generated from the BDF
//! sources in `artwork/fonts` by `tools/fri3d-fontgen`.

use crate::fonts;
use crate::types::{Color, Font};

const FIRST: u32 = 32;
const LAST: u32 = 126;

/// One glyph: its bounding box as rows of `ceil(w/8)` bytes, MSB =
/// leftmost pixel. `x_off` shifts the box right of the pen; `y_off` is
/// the box bottom relative to the baseline (negative for descenders).
pub struct Glyph {
    pub advance: u8,
    pub w: u8,
    pub h: u8,
    pub x_off: i8,
    pub y_off: i8,
    pub bits: &'static [u8],
}

impl Glyph {
    pub const EMPTY: Glyph = Glyph { advance: 0, w: 0, h: 0, x_off: 0, y_off: 0, bits: &[] };
}

/// ASCII 32..=126.
pub struct BitmapFont {
    pub ascent: u8,
    pub descent: u8,
    pub glyphs: [Glyph; (LAST - FIRST + 1) as usize],
}

impl BitmapFont {
    fn glyph(&self, code: u32) -> Option<&Glyph> {
        if (FIRST..=LAST).contains(&code) {
            Some(&self.glyphs[(code - FIRST) as usize])
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct FontData {
    font: &'static BitmapFont,
}

impl core::fmt::Debug for BitmapFont {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BitmapFont(ascent {}, descent {})", self.ascent, self.descent)
    }
}

impl FontData {
    pub fn from_font(font: Font) -> Self {
        let font = match font {
            Font::Primary | Font::Secondary | Font::Keyboard => &fonts::PIXELIFY_R11,
            Font::Title | Font::BigNumbers => &fonts::PIXELIFY_B22,
        };
        Self { font }
    }

    pub fn ascent(&self) -> u32 {
        self.font.ascent as u32
    }

    pub fn descent(&self) -> u32 {
        self.font.descent as u32
    }

    pub fn draw_str<T: FontDrawTarget>(&self, target: &mut T, x: i32, y: i32, text: &str, color: Color) {
        let mut pen = x;
        for ch in text.chars() {
            let Some(g) = self.font.glyph(ch as u32) else { continue };
            let row_bytes = (g.w as usize).div_ceil(8);
            let top = y - (g.h as i32 + g.y_off as i32);
            let left = pen + g.x_off as i32;
            for row in 0..g.h as usize {
                let Some(line) = g.bits.get(row * row_bytes..(row + 1) * row_bytes) else { break };
                let mut col = 0usize;
                while col < g.w as usize {
                    if line[col / 8] & (0x80 >> (col % 8)) == 0 {
                        col += 1;
                        continue;
                    }
                    let start = col;
                    while col < g.w as usize && line[col / 8] & (0x80 >> (col % 8)) != 0 {
                        col += 1;
                    }
                    target.draw_hline_with_color(left + start as i32, top + row as i32, (col - start) as u32, color);
                }
            }
            pen += g.advance as i32;
        }
    }

    /// Advance width of `text`: the pen position after the last glyph.
    pub fn string_width(&self, text: &str) -> u32 {
        text.chars()
            .filter_map(|ch| self.font.glyph(ch as u32))
            .map(|g| g.advance as u32)
            .sum()
    }
}

pub trait FontDrawTarget {
    fn draw_hline_with_color(&mut self, x: i32, y: i32, length: u32, color: Color);
}
