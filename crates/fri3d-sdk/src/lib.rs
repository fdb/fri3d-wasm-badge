//! SDK for writing Fri3d Badge WASM apps
//!
//! This crate provides the imports and types needed to write
//! WASM applications for the Fri3d Badge platform.

#![no_std]

pub mod canvas;
pub mod input;
pub mod random;

pub use canvas::{clear, draw_circle, draw_line, draw_pixel, draw_str, set_color, Color, Font};
pub use input::{InputKey, InputType};
pub use random::{range, seed};
