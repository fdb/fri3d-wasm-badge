use crate::font::{FontData, FontDrawTarget};
use crate::types::{Color, Font};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Canvas {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    color: Color,
    font: Font,
    font_data: FontData,
}

impl Canvas {
    pub fn new() -> Self {
        let width = SCREEN_WIDTH;
        let height = SCREEN_HEIGHT;
        let font = Font::Primary;
        let font_data = FontData::from_font(font);
        Self {
            width,
            height,
            buffer: vec![0; (width * height) as usize],
            color: Color::Black,
            font,
            font_data,
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
        self.font_data = FontData::from_font(font);
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn font(&self) -> Font {
        self.font
    }

    pub fn draw_dot(&mut self, x: i32, y: i32) {
        self.set_pixel_with_color(x, y, self.color);
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x0 = x1;
        let mut y0 = y1;
        let dx = (x2 - x1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let dy = -(y2 - y1).abs();
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel_with_color(x0, y0, self.color);
            if x0 == x2 && y0 == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_frame(&mut self, x: i32, y: i32, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let w = width as i32;
        let h = height as i32;
        self.draw_hline_color(x, y, width, self.color);
        self.draw_hline_color(x, y + h - 1, width, self.color);
        self.draw_vline_color(x, y, height, self.color);
        self.draw_vline_color(x + w - 1, y, height, self.color);
    }

    pub fn draw_box(&mut self, x: i32, y: i32, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        for row in 0..height {
            self.draw_hline_color(x, y + row as i32, width, self.color);
        }
    }

    pub fn draw_rframe(&mut self, x: i32, y: i32, width: u32, height: u32, radius: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let mut r = radius as i32;
        let w = width as i32;
        let h = height as i32;
        r = r.min(w / 2).min(h / 2);
        if r <= 0 {
            self.draw_frame(x, y, width, height);
            return;
        }

        self.draw_hline_color(x + r, y, (w - 2 * r) as u32, self.color);
        self.draw_hline_color(x + r, y + h - 1, (w - 2 * r) as u32, self.color);
        self.draw_vline_color(x, y + r, (h - 2 * r) as u32, self.color);
        self.draw_vline_color(x + w - 1, y + r, (h - 2 * r) as u32, self.color);

        self.draw_circle_quadrants(x + r, y + r, r, true, false, false, false);
        self.draw_circle_quadrants(x + w - 1 - r, y + r, r, false, true, false, false);
        self.draw_circle_quadrants(x + r, y + h - 1 - r, r, false, false, true, false);
        self.draw_circle_quadrants(x + w - 1 - r, y + h - 1 - r, r, false, false, false, true);
    }

    pub fn draw_rbox(&mut self, x: i32, y: i32, width: u32, height: u32, radius: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let mut r = radius as i32;
        let w = width as i32;
        let h = height as i32;
        r = r.min(w / 2).min(h / 2);
        if r <= 0 {
            self.draw_box(x, y, width, height);
            return;
        }

        self.draw_box(x + r, y, (w - 2 * r) as u32, height);
        self.draw_box(x, y + r, r as u32, (h - 2 * r) as u32);
        self.draw_box(x + w - r, y + r, r as u32, (h - 2 * r) as u32);

        self.draw_disc(x + r, y + r, r as u32);
        self.draw_disc(x + w - 1 - r, y + r, r as u32);
        self.draw_disc(x + r, y + h - 1 - r, r as u32);
        self.draw_disc(x + w - 1 - r, y + h - 1 - r, r as u32);
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32) {
        if self.color == Color::Xor {
            self.draw_xor_circle(x, y, radius);
            return;
        }
        let r = radius as i32;
        if r <= 0 {
            self.set_pixel_with_color(x, y, self.color);
            return;
        }
        let mut dx = 0;
        let mut dy = r;
        let mut d = 1 - r;
        while dx <= dy {
            self.draw_circle_points(x, y, dx, dy, self.color);
            if d < 0 {
                d += 2 * dx + 3;
            } else {
                d += 2 * (dx - dy) + 5;
                dy -= 1;
            }
            dx += 1;
        }
    }

    pub fn draw_disc(&mut self, x0: i32, y0: i32, radius: u32) {
        if radius == 0 {
            self.set_pixel_with_color(x0, y0, self.color);
            return;
        }
        let r = radius as i32;
        let r_sq = r * r;
        for dy in -r..=r {
            let y_sq = dy * dy;
            if y_sq > r_sq {
                continue;
            }
            let mut x_extent = 0;
            while (x_extent + 1) * (x_extent + 1) <= r_sq - y_sq {
                x_extent += 1;
            }
            let line_x = x0 - x_extent;
            let line_y = y0 + dy;
            let line_w = (2 * x_extent + 1) as u32;
            self.draw_hline_color(line_x, line_y, line_w, self.color);
        }
    }

    pub fn draw_str(&mut self, x: i32, y: i32, text: &str) {
        let font_data = self.font_data.clone();
        font_data.draw_str(self, x, y, text, self.color);
    }

    pub fn string_width(&self, text: &str) -> u32 {
        self.font_data.string_width(text)
    }

    fn set_pixel_with_color(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        match color {
            Color::White => self.buffer[idx] = 0,
            Color::Black => self.buffer[idx] = 1,
            Color::Xor => self.buffer[idx] ^= 1,
        }
    }

    fn draw_hline_color(&mut self, x: i32, y: i32, length: u32, color: Color) {
        if length == 0 {
            return;
        }
        for offset in 0..length {
            self.set_pixel_with_color(x + offset as i32, y, color);
        }
    }

    fn draw_vline_color(&mut self, x: i32, y: i32, length: u32, color: Color) {
        if length == 0 {
            return;
        }
        for offset in 0..length {
            self.set_pixel_with_color(x, y + offset as i32, color);
        }
    }

    fn draw_circle_points(&mut self, x0: i32, y0: i32, dx: i32, dy: i32, color: Color) {
        self.set_pixel_with_color(x0 + dx, y0 + dy, color);
        self.set_pixel_with_color(x0 - dx, y0 + dy, color);
        self.set_pixel_with_color(x0 + dx, y0 - dy, color);
        self.set_pixel_with_color(x0 - dx, y0 - dy, color);
        self.set_pixel_with_color(x0 + dy, y0 + dx, color);
        self.set_pixel_with_color(x0 - dy, y0 + dx, color);
        self.set_pixel_with_color(x0 + dy, y0 - dx, color);
        self.set_pixel_with_color(x0 - dy, y0 - dx, color);
    }

    fn draw_xor_circle(&mut self, x0: i32, y0: i32, radius: u32) {
        if radius == 0 {
            self.set_pixel_with_color(x0, y0, Color::Xor);
            return;
        }

        let mut x = 0i32;
        let mut y = radius as i32;
        let mut d = 1 - y;

        while x <= y {
            if x == y {
                self.set_pixel_with_color(x0 + x, y0 + y, Color::Xor);
                self.set_pixel_with_color(x0 - x, y0 + y, Color::Xor);
                self.set_pixel_with_color(x0 + x, y0 - y, Color::Xor);
                self.set_pixel_with_color(x0 - x, y0 - y, Color::Xor);
            } else if x == 0 {
                self.set_pixel_with_color(x0, y0 + y, Color::Xor);
                self.set_pixel_with_color(x0, y0 - y, Color::Xor);
                self.set_pixel_with_color(x0 + y, y0, Color::Xor);
                self.set_pixel_with_color(x0 - y, y0, Color::Xor);
            } else {
                self.set_pixel_with_color(x0 + x, y0 + y, Color::Xor);
                self.set_pixel_with_color(x0 - x, y0 + y, Color::Xor);
                self.set_pixel_with_color(x0 + x, y0 - y, Color::Xor);
                self.set_pixel_with_color(x0 - x, y0 - y, Color::Xor);
                self.set_pixel_with_color(x0 + y, y0 + x, Color::Xor);
                self.set_pixel_with_color(x0 - y, y0 + x, Color::Xor);
                self.set_pixel_with_color(x0 + y, y0 - x, Color::Xor);
                self.set_pixel_with_color(x0 - y, y0 - x, Color::Xor);
            }

            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }
    }

    fn draw_circle_quadrants(
        &mut self,
        x0: i32,
        y0: i32,
        radius: i32,
        top_left: bool,
        top_right: bool,
        bottom_left: bool,
        bottom_right: bool,
    ) {
        if radius <= 0 {
            return;
        }
        let mut dx = 0;
        let mut dy = radius;
        let mut d = 1 - radius;

        while dx <= dy {
            if top_left {
                self.set_pixel_with_color(x0 - dx, y0 - dy, self.color);
                self.set_pixel_with_color(x0 - dy, y0 - dx, self.color);
            }
            if top_right {
                self.set_pixel_with_color(x0 + dx, y0 - dy, self.color);
                self.set_pixel_with_color(x0 + dy, y0 - dx, self.color);
            }
            if bottom_left {
                self.set_pixel_with_color(x0 - dx, y0 + dy, self.color);
                self.set_pixel_with_color(x0 - dy, y0 + dx, self.color);
            }
            if bottom_right {
                self.set_pixel_with_color(x0 + dx, y0 + dy, self.color);
                self.set_pixel_with_color(x0 + dy, y0 + dx, self.color);
            }

            if d < 0 {
                d += 2 * dx + 3;
            } else {
                d += 2 * (dx - dy) + 5;
                dy -= 1;
            }
            dx += 1;
        }
    }
}

impl FontDrawTarget for Canvas {
    fn draw_hline_with_color(&mut self, x: i32, y: i32, length: u32, color: Color) {
        self.draw_hline_color(x, y, length, color);
    }
}
