use crate::font::{FontData, FontDrawTarget};
use crate::types::{Color, Font};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

const DRAW_UPPER_RIGHT: u8 = 0x01;
const DRAW_UPPER_LEFT: u8 = 0x02;
const DRAW_LOWER_LEFT: u8 = 0x04;
const DRAW_LOWER_RIGHT: u8 = 0x08;
const DRAW_ALL: u8 = DRAW_UPPER_RIGHT | DRAW_UPPER_LEFT | DRAW_LOWER_LEFT | DRAW_LOWER_RIGHT;

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
        let mut x1 = x1;
        let mut y1 = y1;
        let mut x2 = x2;
        let mut y2 = y2;

        let mut dx = (x2 - x1).abs();
        let mut dy = (y2 - y1).abs();
        let mut swap_xy = false;

        if dy > dx {
            swap_xy = true;
            std::mem::swap(&mut x1, &mut y1);
            std::mem::swap(&mut x2, &mut y2);
            std::mem::swap(&mut dx, &mut dy);
        }

        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
        }

        let mut err = dx >> 1;
        let ystep = if y2 > y1 { 1 } else { -1 };
        let mut y = y1;

        let mut x = x1;
        while x <= x2 {
            if !swap_xy {
                self.set_pixel_with_color(x, y, self.color);
            } else {
                self.set_pixel_with_color(y, x, self.color);
            }
            err -= dy;
            if err < 0 {
                y += ystep;
                err += dx;
            }
            x += 1;
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

        let mut xl = x + r;
        let mut yu = y + r;
        let xr = x + w - r - 1;
        let yl = y + h - r - 1;

        self.draw_circle_with_option(xl, yu, r, DRAW_UPPER_LEFT, self.color);
        self.draw_circle_with_option(xr, yu, r, DRAW_UPPER_RIGHT, self.color);
        self.draw_circle_with_option(xl, yl, r, DRAW_LOWER_LEFT, self.color);
        self.draw_circle_with_option(xr, yl, r, DRAW_LOWER_RIGHT, self.color);

        let mut ww = w - r - r;
        let mut hh = h - r - r;

        xl += 1;
        yu += 1;

        if ww >= 3 {
            ww -= 2;
            let edge_y = y + h - 1;
            self.draw_hline_color(xl, y, ww as u32, self.color);
            self.draw_hline_color(xl, edge_y, ww as u32, self.color);
        }

        if hh >= 3 {
            hh -= 2;
            let edge_x = x + w - 1;
            self.draw_vline_color(x, yu, hh as u32, self.color);
            self.draw_vline_color(edge_x, yu, hh as u32, self.color);
        }
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

        let mut xl = x + r;
        let mut yu = y + r;
        let xr = x + w - r - 1;
        let yl = y + h - r - 1;

        self.draw_disc_with_option(xl, yu, r, DRAW_UPPER_LEFT, self.color);
        self.draw_disc_with_option(xr, yu, r, DRAW_UPPER_RIGHT, self.color);
        self.draw_disc_with_option(xl, yl, r, DRAW_LOWER_LEFT, self.color);
        self.draw_disc_with_option(xr, yl, r, DRAW_LOWER_RIGHT, self.color);

        let mut ww = w - r - r;
        let mut hh = h - r - r;

        xl += 1;
        yu += 1;

        if ww >= 3 {
            ww -= 2;
            self.draw_box(xl, y, ww as u32, (r + 1) as u32);
            self.draw_box(xl, yl, ww as u32, (r + 1) as u32);
        }

        if hh >= 3 {
            hh -= 2;
            self.draw_box(x, yu, w as u32, hh as u32);
        }
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32) {
        if self.color == Color::Xor {
            self.draw_xor_circle(x, y, radius);
            return;
        }
        self.draw_circle_with_option(x, y, radius as i32, DRAW_ALL, self.color);
    }

    pub fn draw_disc(&mut self, x0: i32, y0: i32, radius: u32) {
        if self.color == Color::Xor {
            self.draw_xor_disc(x0, y0, radius);
            return;
        }
        self.draw_disc_with_option(x0, y0, radius as i32, DRAW_ALL, self.color);
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

    fn draw_circle_section(
        &mut self,
        x: i32,
        y: i32,
        x0: i32,
        y0: i32,
        option: u8,
        color: Color,
    ) {
        if option & DRAW_UPPER_RIGHT != 0 {
            self.set_pixel_with_color(x0 + x, y0 - y, color);
            self.set_pixel_with_color(x0 + y, y0 - x, color);
        }
        if option & DRAW_UPPER_LEFT != 0 {
            self.set_pixel_with_color(x0 - x, y0 - y, color);
            self.set_pixel_with_color(x0 - y, y0 - x, color);
        }
        if option & DRAW_LOWER_RIGHT != 0 {
            self.set_pixel_with_color(x0 + x, y0 + y, color);
            self.set_pixel_with_color(x0 + y, y0 + x, color);
        }
        if option & DRAW_LOWER_LEFT != 0 {
            self.set_pixel_with_color(x0 - x, y0 + y, color);
            self.set_pixel_with_color(x0 - y, y0 + x, color);
        }
    }

    fn draw_circle_with_option(
        &mut self,
        x0: i32,
        y0: i32,
        radius: i32,
        option: u8,
        color: Color,
    ) {
        if radius < 0 {
            return;
        }

        let mut f = 1 - radius;
        let mut dd_f_x = 1;
        let mut dd_f_y = -2 * radius;
        let mut x = 0;
        let mut y = radius;

        self.draw_circle_section(x, y, x0, y0, option, color);

        while x < y {
            if f >= 0 {
                y -= 1;
                dd_f_y += 2;
                f += dd_f_y;
            }
            x += 1;
            dd_f_x += 2;
            f += dd_f_x;

            self.draw_circle_section(x, y, x0, y0, option, color);
        }
    }

    fn draw_disc_section(
        &mut self,
        x: i32,
        y: i32,
        x0: i32,
        y0: i32,
        option: u8,
        color: Color,
    ) {
        if option & DRAW_UPPER_RIGHT != 0 {
            self.draw_vline_color(x0 + x, y0 - y, (y + 1) as u32, color);
            self.draw_vline_color(x0 + y, y0 - x, (x + 1) as u32, color);
        }
        if option & DRAW_UPPER_LEFT != 0 {
            self.draw_vline_color(x0 - x, y0 - y, (y + 1) as u32, color);
            self.draw_vline_color(x0 - y, y0 - x, (x + 1) as u32, color);
        }
        if option & DRAW_LOWER_RIGHT != 0 {
            self.draw_vline_color(x0 + x, y0, (y + 1) as u32, color);
            self.draw_vline_color(x0 + y, y0, (x + 1) as u32, color);
        }
        if option & DRAW_LOWER_LEFT != 0 {
            self.draw_vline_color(x0 - x, y0, (y + 1) as u32, color);
            self.draw_vline_color(x0 - y, y0, (x + 1) as u32, color);
        }
    }

    fn draw_disc_with_option(
        &mut self,
        x0: i32,
        y0: i32,
        radius: i32,
        option: u8,
        color: Color,
    ) {
        if radius < 0 {
            return;
        }

        let mut f = 1 - radius;
        let mut dd_f_x = 1;
        let mut dd_f_y = -2 * radius;
        let mut x = 0;
        let mut y = radius;

        self.draw_disc_section(x, y, x0, y0, option, color);

        while x < y {
            if f >= 0 {
                y -= 1;
                dd_f_y += 2;
                f += dd_f_y;
            }
            x += 1;
            dd_f_x += 2;
            f += dd_f_x;

            self.draw_disc_section(x, y, x0, y0, option, color);
        }
    }

    fn draw_xor_disc(&mut self, x0: i32, y0: i32, radius: u32) {
        if radius == 0 {
            self.set_pixel_with_color(x0, y0, Color::Xor);
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

            let line_y = y0 + dy;
            let line_x = x0 - x_extent;
            let line_w = (2 * x_extent + 1) as u32;

            self.draw_hline_color(line_x, line_y, line_w, Color::Xor);
        }
    }
}

impl FontDrawTarget for Canvas {
    fn draw_hline_with_color(&mut self, x: i32, y: i32, length: u32, color: Color) {
        self.draw_hline_color(x, y, length, color);
    }
}
