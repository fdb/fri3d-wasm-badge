pub mod canvas;
pub mod font;
pub mod fonts;
pub mod input;
pub mod random;
pub mod trace;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod app_manager;
#[cfg(not(target_arch = "wasm32"))]
pub mod wasm_runner;

pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 64;
