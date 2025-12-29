//! Canvas drawing functions for WASM apps

/// Drawing color
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
    Xor = 2,
}

/// Font selection
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Font {
    Primary = 0,
    Secondary = 1,
    Keyboard = 2,
    BigNumbers = 3,
}

// External functions provided by the host
extern "C" {
    fn canvas_clear();
    fn canvas_width() -> u32;
    fn canvas_height() -> u32;
    fn canvas_set_color(color: u32);
    fn canvas_set_font(font: u32);
    fn canvas_draw_dot(x: i32, y: i32);
    fn canvas_draw_line(x0: i32, y0: i32, x1: i32, y1: i32);
    fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32);
    fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32);
    fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, r: u32);
    fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, r: u32);
    fn canvas_draw_circle(cx: i32, cy: i32, r: u32);
    fn canvas_draw_disc(cx: i32, cy: i32, r: u32);
    fn canvas_draw_str(x: i32, y: i32, ptr: u32, len: u32);
}

/// Clear the canvas to white
#[inline]
pub fn clear() {
    unsafe { canvas_clear() }
}

/// Get canvas width
#[inline]
pub fn width() -> u32 {
    unsafe { canvas_width() }
}

/// Get canvas height
#[inline]
pub fn height() -> u32 {
    unsafe { canvas_height() }
}

/// Set the drawing color
#[inline]
pub fn set_color(color: Color) {
    unsafe { canvas_set_color(color as u32) }
}

/// Set the font for text drawing
#[inline]
pub fn set_font(font: Font) {
    unsafe { canvas_set_font(font as u32) }
}

/// Draw a single pixel
#[inline]
pub fn draw_pixel(x: i32, y: i32) {
    unsafe { canvas_draw_dot(x, y) }
}

/// Draw a line
#[inline]
pub fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32) {
    unsafe { canvas_draw_line(x0, y0, x1, y1) }
}

/// Draw a rectangle outline
#[inline]
pub fn draw_rect(x: i32, y: i32, w: u32, h: u32) {
    unsafe { canvas_draw_frame(x, y, w, h) }
}

/// Draw a filled rectangle
#[inline]
pub fn fill_rect(x: i32, y: i32, w: u32, h: u32) {
    unsafe { canvas_draw_box(x, y, w, h) }
}

/// Draw a rounded rectangle outline
#[inline]
pub fn draw_round_rect(x: i32, y: i32, w: u32, h: u32, r: u32) {
    unsafe { canvas_draw_rframe(x, y, w, h, r) }
}

/// Draw a filled rounded rectangle
#[inline]
pub fn fill_round_rect(x: i32, y: i32, w: u32, h: u32, r: u32) {
    unsafe { canvas_draw_rbox(x, y, w, h, r) }
}

/// Draw a circle outline
#[inline]
pub fn draw_circle(x: i32, y: i32, r: u32) {
    unsafe { canvas_draw_circle(x, y, r) }
}

/// Draw a filled circle (disc)
#[inline]
pub fn fill_circle(x: i32, y: i32, r: u32) {
    unsafe { canvas_draw_disc(x, y, r) }
}

/// Draw a string at the given position
#[inline]
pub fn draw_str(x: i32, y: i32, text: &str) {
    unsafe {
        canvas_draw_str(x, y, text.as_ptr() as u32, text.len() as u32);
    }
}
