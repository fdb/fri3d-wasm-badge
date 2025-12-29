//! Fri3d Badge Web Runtime
//!
//! This crate compiles to WASM and provides the runtime for web-based apps.
//! Apps are loaded separately by the JavaScript glue code.

use fri3d_runtime::{Canvas, Color, Font, InputKey, InputManager, Random};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Runtime {
    canvas: Canvas,
    random: Random,
    input_manager: InputManager,
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            random: Random::new(42),
            input_manager: InputManager::new(),
        }
    }

    // Canvas API
    pub fn canvas_clear(&mut self) {
        self.canvas.clear();
    }

    pub fn canvas_width(&self) -> u32 {
        self.canvas.width()
    }

    pub fn canvas_height(&self) -> u32 {
        self.canvas.height()
    }

    pub fn canvas_set_color(&mut self, color: u8) {
        if let Ok(c) = Color::try_from(color) {
            self.canvas.set_color(c);
        }
    }

    pub fn canvas_set_font(&mut self, font: u8) {
        if let Ok(f) = Font::try_from(font) {
            self.canvas.set_font(f);
        }
    }

    pub fn canvas_draw_dot(&mut self, x: i32, y: i32) {
        self.canvas.draw_pixel(x, y);
    }

    pub fn canvas_draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.canvas.draw_line(x0, y0, x1, y1);
    }

    pub fn canvas_draw_frame(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.canvas.draw_rect(x, y, w, h);
    }

    pub fn canvas_draw_box(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.canvas.fill_rect(x, y, w, h);
    }

    pub fn canvas_draw_rframe(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) {
        self.canvas.draw_round_rect(x, y, w, h, r);
    }

    pub fn canvas_draw_rbox(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) {
        self.canvas.fill_round_rect(x, y, w, h, r);
    }

    pub fn canvas_draw_circle(&mut self, x: i32, y: i32, r: u32) {
        self.canvas.draw_circle(x, y, r);
    }

    pub fn canvas_draw_disc(&mut self, x: i32, y: i32, r: u32) {
        self.canvas.fill_circle(x, y, r);
    }

    // Random API
    pub fn random_seed(&mut self, seed: u32) {
        self.random.seed(seed);
    }

    pub fn random_get(&mut self) -> u32 {
        self.random.next()
    }

    pub fn random_range(&mut self, max: u32) -> u32 {
        self.random.range(max)
    }

    // Input API
    pub fn key_down(&mut self, key: u8, time_ms: u32) {
        if let Ok(k) = InputKey::try_from(key) {
            self.input_manager.key_down(k, time_ms);
        }
    }

    pub fn key_up(&mut self, key: u8, time_ms: u32) {
        if let Ok(k) = InputKey::try_from(key) {
            self.input_manager.key_up(k, time_ms);
        }
    }

    pub fn update_input(&mut self, time_ms: u32) {
        self.input_manager.update(time_ms);
    }

    /// Poll next event, returns packed u32: (key << 8) | type, or 0xFFFFFFFF if no event
    pub fn poll_event(&mut self) -> u32 {
        self.input_manager
            .poll_event()
            .map(|e| ((e.key as u32) << 8) | (e.event_type as u32))
            .unwrap_or(0xFFFFFFFF)
    }

    /// Get buffer as bytes for rendering
    pub fn get_buffer(&self) -> Vec<u8> {
        self.canvas.buffer().to_vec()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
