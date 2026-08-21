//! The DB32 palette (Richard "DawnBringer" Fhager) and the roles the badge
//! UI gives its entries. Every framebuffer byte is an index into [`RGB`].
//!
//! Design doc 017 explains the roles. `artwork/db32.gpl` is the same
//! palette for Aseprite.

use crate::types::Color;

/// Packed `0xRRGGBB`, in the order of the published `.gpl` file.
pub const RGB: [u32; 32] = [
    0x000000, 0x222034, 0x45283c, 0x663931, 0x8f563b, 0xdf7126, 0xd9a066, 0xeec39a, //  0..7
    0xfbf236, 0x99e550, 0x6abe30, 0x37946e, 0x4b692f, 0x524b24, 0x323c39, 0x3f3f74, //  8..15
    0x306082, 0x5b6ee1, 0x639bff, 0x5fcde4, 0xcbdbfc, 0xffffff, 0x9badb7, 0x847e87, // 16..23
    0x696a6a, 0x595652, 0x76428a, 0xac3232, 0xd95763, 0xd77bba, 0x8f974a, 0x8a6f30, // 24..31
];

/// Image pixel value that is not drawn.
pub const TRANSPARENT: u8 = 255;

pub const BLACK: Color = Color(0);
pub const INK: Color = Color(1);
pub const PLUM: Color = Color(2);
pub const BROWN_DARK: Color = Color(3);
pub const BROWN: Color = Color(4);
pub const TERRA: Color = Color(5);
pub const TAN: Color = Color(6);
pub const PAPER: Color = Color(7);
pub const GOLD: Color = Color(8);
pub const GREEN_LIGHT: Color = Color(9);
pub const GREEN: Color = Color(10);
pub const TEAL: Color = Color(11);
pub const GREEN_DARK: Color = Color(12);
pub const OLIVE: Color = Color(13);
pub const FOREST: Color = Color(14);
pub const INDIGO: Color = Color(15);
pub const BLUE_DARK: Color = Color(16);
pub const BLUE_MID: Color = Color(17);
pub const BLUE: Color = Color(18);
pub const CYAN: Color = Color(19);
pub const SKY: Color = Color(20);
pub const WHITE: Color = Color(21);
pub const GREY_LIGHT: Color = Color(22);
pub const GREY: Color = Color(23);
pub const GREY_DARK: Color = Color(24);
pub const CHARCOAL: Color = Color(25);
pub const PURPLE: Color = Color(26);
pub const RED: Color = Color(27);
pub const ROSE: Color = Color(28);
pub const PINK: Color = Color(29);
pub const SAGE: Color = Color(30);
pub const GOLD_DARK: Color = Color(31);

// UI roles.
pub const CARD: Color = WHITE;
pub const MUTED: Color = BROWN;
pub const FOCUS: Color = GOLD;
pub const REST_BORDER: Color = TAN;

/// `0xRRGGBB` for a framebuffer byte. Out-of-range bytes map to ink.
pub const fn rgb(index: u8) -> u32 {
    RGB[Color::from_index(index).0 as usize]
}

/// RGB565, big-endian on the wire as the ST7789 expects it.
pub const fn rgb565(index: u8) -> u16 {
    let c = rgb(index);
    let r = ((c >> 16) & 0xff) as u16;
    let g = ((c >> 8) & 0xff) as u16;
    let b = (c & 0xff) as u16;
    ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
}
