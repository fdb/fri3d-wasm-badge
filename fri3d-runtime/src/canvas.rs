use crate::types::{Color, Font};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Canvas {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    color: Color,
    font: Font,
}

impl Canvas {
    pub fn new() -> Self {
        let width = SCREEN_WIDTH;
        let height = SCREEN_HEIGHT;
        Self {
            width,
            height,
            buffer: vec![0; (width * height) as usize],
            color: Color::Black,
            font: Font::Primary,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn font(&self) -> Font {
        self.font
    }

    fn set_pixel(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        match self.color {
            Color::White => self.buffer[idx] = 0,
            Color::Black => self.buffer[idx] = 1,
            Color::Xor => self.buffer[idx] ^= 1,
        }
    }

    pub fn draw_dot(&mut self, x: i32, y: i32) {
        self.set_pixel(x, y);
    }

    // Drawing primitives are stubs until the dedicated graphics stage.
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let _ = (x1, y1, x2, y2);
    }

    pub fn draw_frame(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let _ = (x, y, width, height);
    }

    pub fn draw_box(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let _ = (x, y, width, height);
    }

    pub fn draw_rframe(&mut self, x: i32, y: i32, width: u32, height: u32, radius: u32) {
        let _ = (x, y, width, height, radius);
    }

    pub fn draw_rbox(&mut self, x: i32, y: i32, width: u32, height: u32, radius: u32) {
        let _ = (x, y, width, height, radius);
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32) {
        let _ = (x, y, radius);
    }

    pub fn draw_disc(&mut self, x: i32, y: i32, radius: u32) {
        let _ = (x, y, radius);
    }

    pub fn draw_str(&mut self, x: i32, y: i32, text: &str) {
        let _ = (x, y, text);
    }

    pub fn string_width(&self, text: &str) -> u32 {
        text.len() as u32
    }
}
