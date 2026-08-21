//! System icons, one `Image` per file in `artwork/icons/`. Pixels are
//! DB32 palette indices, `color::TRANSPARENT` for holes. Edit the PNGs
//! (`artwork/db32.gpl` is the palette for Aseprite), then run
//! `cargo run -p fri3d-pack`.
#![no_std]

pub use fri3d_wasm_api::Image;

mod generated;
pub use generated::*;
