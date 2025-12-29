//! Fri3d Badge Runtime
//!
//! Core runtime for the Fri3d Badge platform, providing:
//! - Canvas: 128x64 monochrome framebuffer with drawing primitives
//! - Input: Key state tracking with short/long press detection
//! - Random: Mersenne Twister PRNG
//!
//! This crate is `no_std` compatible for embedded targets.

#![cfg_attr(not(feature = "std"), no_std)]

mod canvas;
mod font;
mod input;
mod random;

pub use canvas::{Canvas, Color, Font, BUFFER_SIZE, HEIGHT, WIDTH};
pub use input::{InputEvent, InputKey, InputManager, InputType};
pub use random::Random;
