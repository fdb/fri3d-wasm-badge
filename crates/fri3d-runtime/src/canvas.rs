//! Canvas - 128x64 monochrome framebuffer with drawing primitives

use crate::font::{self, BitmapFont};

/// Display width in pixels
pub const WIDTH: usize = 128;
/// Display height in pixels
pub const HEIGHT: usize = 64;
/// Buffer size in bytes (1 bit per pixel)
pub const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8;

/// Drawing color
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Color {
    /// White (pixel off)
    White = 0,
    /// Black (pixel on)
    #[default]
    Black = 1,
    /// XOR mode (toggle pixel)
    Xor = 2,
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::White),
            1 => Ok(Color::Black),
            2 => Ok(Color::Xor),
            _ => Err(()),
        }
    }
}

/// Font selection
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Font {
    /// Primary font (6x10)
    #[default]
    Primary = 0,
    /// Secondary font (5x7)
    Secondary = 1,
    /// Keyboard font (5x8)
    Keyboard = 2,
    /// Big numbers font (10x20)
    BigNumbers = 3,
}

impl TryFrom<u8> for Font {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Font::Primary),
            1 => Ok(Font::Secondary),
            2 => Ok(Font::Keyboard),
            3 => Ok(Font::BigNumbers),
            _ => Err(()),
        }
    }
}

/// 128x64 monochrome canvas with drawing primitives
pub struct Canvas {
    /// Framebuffer (1 bit per pixel, row-major, MSB first)
    buffer: [u8; BUFFER_SIZE],
    /// Current drawing color
    color: Color,
    /// Current font
    font: Font,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Create a new canvas (cleared to white)
    pub fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            color: Color::Black,
            font: Font::Primary,
        }
    }

    /// Clear the canvas to white
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    /// Set the drawing color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Get the current drawing color
    pub fn color(&self) -> Color {
        self.color
    }

    /// Set the font for text drawing
    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    /// Get the current font
    pub fn font(&self) -> Font {
        self.font
    }

    /// Get the framebuffer
    pub fn buffer(&self) -> &[u8; BUFFER_SIZE] {
        &self.buffer
    }

    /// Get mutable access to the framebuffer
    pub fn buffer_mut(&mut self) -> &mut [u8; BUFFER_SIZE] {
        &mut self.buffer
    }

    /// Get display width
    pub fn width(&self) -> u32 {
        WIDTH as u32
    }

    /// Get display height
    pub fn height(&self) -> u32 {
        HEIGHT as u32
    }

    /// Draw a single pixel
    pub fn draw_pixel(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            return;
        }

        let x = x as usize;
        let y = y as usize;
        let byte_index = (y * WIDTH + x) / 8;
        let bit_index = 7 - (x % 8);

        match self.color {
            Color::White => self.buffer[byte_index] &= !(1 << bit_index),
            Color::Black => self.buffer[byte_index] |= 1 << bit_index,
            Color::Xor => self.buffer[byte_index] ^= 1 << bit_index,
        }
    }

    /// Draw a horizontal line (optimized)
    pub fn draw_hline(&mut self, x: i32, y: i32, w: u32) {
        if y < 0 || y >= HEIGHT as i32 {
            return;
        }

        let x_start = x.max(0) as usize;
        let x_end = ((x + w as i32) as usize).min(WIDTH);

        for px in x_start..x_end {
            self.draw_pixel(px as i32, y);
        }
    }

    /// Draw a vertical line (optimized)
    pub fn draw_vline(&mut self, x: i32, y: i32, h: u32) {
        if x < 0 || x >= WIDTH as i32 {
            return;
        }

        let y_start = y.max(0) as usize;
        let y_end = ((y + h as i32) as usize).min(HEIGHT);

        for py in y_start..y_end {
            self.draw_pixel(x, py as i32);
        }
    }

    /// Draw a line using Bresenham's algorithm
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        // Handle horizontal and vertical lines specially
        if y0 == y1 {
            let (x_start, x_end) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
            self.draw_hline(x_start, y0, (x_end - x_start + 1) as u32);
            return;
        }
        if x0 == x1 {
            let (y_start, y_end) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
            self.draw_vline(x0, y_start, (y_end - y_start + 1) as u32);
            return;
        }

        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_pixel(x, y);

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                if x == x1 {
                    break;
                }
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == y1 {
                    break;
                }
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }

        self.draw_hline(x, y, w);
        self.draw_hline(x, y + h as i32 - 1, w);
        self.draw_vline(x, y + 1, h.saturating_sub(2));
        self.draw_vline(x + w as i32 - 1, y + 1, h.saturating_sub(2));
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32) {
        for py in 0..h as i32 {
            self.draw_hline(x, y + py, w);
        }
    }

    /// Draw a rounded rectangle outline
    pub fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) {
        if w == 0 || h == 0 {
            return;
        }

        let r = r.min(w / 2).min(h / 2);

        if r == 0 {
            self.draw_rect(x, y, w, h);
            return;
        }

        // Horizontal lines
        self.draw_hline(x + r as i32, y, w - 2 * r);
        self.draw_hline(x + r as i32, y + h as i32 - 1, w - 2 * r);

        // Vertical lines
        self.draw_vline(x, y + r as i32, h - 2 * r);
        self.draw_vline(x + w as i32 - 1, y + r as i32, h - 2 * r);

        // Corners
        self.draw_circle_quarter(x + r as i32, y + r as i32, r, 0b0001);
        self.draw_circle_quarter(x + w as i32 - 1 - r as i32, y + r as i32, r, 0b0010);
        self.draw_circle_quarter(x + r as i32, y + h as i32 - 1 - r as i32, r, 0b0100);
        self.draw_circle_quarter(
            x + w as i32 - 1 - r as i32,
            y + h as i32 - 1 - r as i32,
            r,
            0b1000,
        );
    }

    /// Draw a filled rounded rectangle
    pub fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) {
        if w == 0 || h == 0 {
            return;
        }

        let r = r.min(w / 2).min(h / 2);

        if r == 0 {
            self.fill_rect(x, y, w, h);
            return;
        }

        // Center rectangle
        self.fill_rect(x, y + r as i32, w, h - 2 * r);

        // Top and bottom strips
        self.fill_rect(x + r as i32, y, w - 2 * r, r);
        self.fill_rect(x + r as i32, y + h as i32 - r as i32, w - 2 * r, r);

        // Filled corners
        self.fill_circle_quarter(x + r as i32, y + r as i32, r, 0b0001);
        self.fill_circle_quarter(x + w as i32 - 1 - r as i32, y + r as i32, r, 0b0010);
        self.fill_circle_quarter(x + r as i32, y + h as i32 - 1 - r as i32, r, 0b0100);
        self.fill_circle_quarter(
            x + w as i32 - 1 - r as i32,
            y + h as i32 - 1 - r as i32,
            r,
            0b1000,
        );
    }

    /// Draw a circle outline using midpoint algorithm
    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: u32) {
        if r == 0 {
            self.draw_pixel(cx, cy);
            return;
        }

        let mut x = r as i32;
        let mut y = 0i32;
        let mut err = 1 - x;

        while x >= y {
            self.draw_pixel(cx + x, cy + y);
            self.draw_pixel(cx + y, cy + x);
            self.draw_pixel(cx - y, cy + x);
            self.draw_pixel(cx - x, cy + y);
            self.draw_pixel(cx - x, cy - y);
            self.draw_pixel(cx - y, cy - x);
            self.draw_pixel(cx + y, cy - x);
            self.draw_pixel(cx + x, cy - y);

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x + 1);
            }
        }
    }

    /// Draw a filled circle (disc)
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: u32) {
        if r == 0 {
            self.draw_pixel(cx, cy);
            return;
        }

        let mut x = r as i32;
        let mut y = 0i32;
        let mut err = 1 - x;

        while x >= y {
            self.draw_hline(cx - x, cy + y, (2 * x + 1) as u32);
            self.draw_hline(cx - x, cy - y, (2 * x + 1) as u32);
            self.draw_hline(cx - y, cy + x, (2 * y + 1) as u32);
            self.draw_hline(cx - y, cy - x, (2 * y + 1) as u32);

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x + 1);
            }
        }
    }

    /// Draw a quarter of a circle outline
    /// Quadrants: 0b0001=TL, 0b0010=TR, 0b0100=BL, 0b1000=BR
    fn draw_circle_quarter(&mut self, cx: i32, cy: i32, r: u32, quadrant: u8) {
        if r == 0 {
            self.draw_pixel(cx, cy);
            return;
        }

        let mut x = r as i32;
        let mut y = 0i32;
        let mut err = 1 - x;

        while x >= y {
            if quadrant & 0b0001 != 0 {
                // Top-left
                self.draw_pixel(cx - x, cy - y);
                self.draw_pixel(cx - y, cy - x);
            }
            if quadrant & 0b0010 != 0 {
                // Top-right
                self.draw_pixel(cx + x, cy - y);
                self.draw_pixel(cx + y, cy - x);
            }
            if quadrant & 0b0100 != 0 {
                // Bottom-left
                self.draw_pixel(cx - x, cy + y);
                self.draw_pixel(cx - y, cy + x);
            }
            if quadrant & 0b1000 != 0 {
                // Bottom-right
                self.draw_pixel(cx + x, cy + y);
                self.draw_pixel(cx + y, cy + x);
            }

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x + 1);
            }
        }
    }

    /// Draw a filled quarter of a circle
    fn fill_circle_quarter(&mut self, cx: i32, cy: i32, r: u32, quadrant: u8) {
        if r == 0 {
            self.draw_pixel(cx, cy);
            return;
        }

        let mut x = r as i32;
        let mut y = 0i32;
        let mut err = 1 - x;

        while x >= y {
            if quadrant & 0b0001 != 0 {
                // Top-left
                self.draw_hline(cx - x, cy - y, x as u32 + 1);
                self.draw_hline(cx - y, cy - x, y as u32 + 1);
            }
            if quadrant & 0b0010 != 0 {
                // Top-right
                self.draw_hline(cx, cy - y, x as u32 + 1);
                self.draw_hline(cx, cy - x, y as u32 + 1);
            }
            if quadrant & 0b0100 != 0 {
                // Bottom-left
                self.draw_hline(cx - x, cy + y, x as u32 + 1);
                self.draw_hline(cx - y, cy + x, y as u32 + 1);
            }
            if quadrant & 0b1000 != 0 {
                // Bottom-right
                self.draw_hline(cx, cy + y, x as u32 + 1);
                self.draw_hline(cx, cy + x, y as u32 + 1);
            }

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x + 1);
            }
        }
    }

    /// Draw a string at the given position (y is top of text)
    pub fn draw_str(&mut self, x: i32, y: i32, text: &str) {
        let font = self.get_font();
        let mut cursor_x = x;

        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(ch) {
                self.draw_glyph(cursor_x, y, glyph, font);
                cursor_x += font.width as i32;
            }
        }
    }

    /// Calculate the width of a string in pixels
    pub fn string_width(&self, text: &str) -> u32 {
        let font = self.get_font();
        (text.chars().count() * font.width as usize) as u32
    }

    /// Get the current font data
    fn get_font(&self) -> &'static BitmapFont {
        match self.font {
            Font::Primary => &font::FONT_PRIMARY,
            Font::Secondary => &font::FONT_SECONDARY,
            Font::Keyboard => &font::FONT_KEYBOARD,
            Font::BigNumbers => &font::FONT_BIG_NUMBERS,
        }
    }

    /// Draw a single glyph (row-based format)
    fn draw_glyph(&mut self, x: i32, y: i32, glyph: &[u8], font: &BitmapFont) {
        for row in 0..font.height as usize {
            if row >= glyph.len() {
                break;
            }
            let row_data = glyph[row];
            for col in 0..font.width as usize {
                // Bits are stored from bit 6 down to bit 1 for 6-pixel width
                // (bit 7 is unused, bits 6-1 are pixels left to right)
                let bit_index = 6 - col;
                if (row_data >> bit_index) & 1 != 0 {
                    self.draw_pixel(x + col as i32, y + row as i32);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_new() {
        let canvas = Canvas::new();
        assert_eq!(canvas.width(), 128);
        assert_eq!(canvas.height(), 64);
        assert!(canvas.buffer().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_draw_pixel() {
        let mut canvas = Canvas::new();
        canvas.draw_pixel(0, 0);
        assert_eq!(canvas.buffer()[0], 0b1000_0000);

        canvas.draw_pixel(7, 0);
        assert_eq!(canvas.buffer()[0], 0b1000_0001);
    }

    #[test]
    fn test_draw_pixel_xor() {
        let mut canvas = Canvas::new();
        canvas.draw_pixel(0, 0);
        assert_eq!(canvas.buffer()[0], 0b1000_0000);

        canvas.set_color(Color::Xor);
        canvas.draw_pixel(0, 0);
        assert_eq!(canvas.buffer()[0], 0b0000_0000);

        canvas.draw_pixel(0, 0);
        assert_eq!(canvas.buffer()[0], 0b1000_0000);
    }

    #[test]
    fn test_clear() {
        let mut canvas = Canvas::new();
        canvas.draw_pixel(10, 10);
        canvas.draw_pixel(20, 20);
        canvas.clear();
        assert!(canvas.buffer().iter().all(|&b| b == 0));
    }
}
