//! Vector UI kit — small set of reusable drawing primitives that match the
//! Fri3d badge OS design (violet/cyan vector terminal). Pure render helpers,
//! no state. Apps call them between `screen::clear` and any custom drawing.

use crate::{font, screen, theme};

/// Screen dimensions, mirrored here for terse call sites.
pub const W: i32 = screen::WIDTH as i32;
pub const H: i32 = screen::HEIGHT as i32;

// ---------------------------------------------------------------------------
// Status bar (top strip).
// ---------------------------------------------------------------------------

/// Top strip with `label` left-aligned and `time` right-aligned, plus a
/// battery + wifi glyph. Always 14 px tall; everything below it is the
/// "content area" for the app to use.
pub fn status_bar(label: &str, time: &str) {
    screen::line(0, 14, W, 14, theme::INK_DEEP);
    screen::text(6, 10, label, theme::WHITE, font::PRIMARY);
    screen::text(W - 30, 10, time, theme::INK, font::PRIMARY);

    // Battery icon: outlined cell + nub on the right + fill bar.
    let bx = W - 60;
    screen::stroke_rect(bx, 4, 18, 8, theme::INK);
    screen::fill_rect(bx + 18, 6, 1, 4, theme::INK);
    screen::fill_rect(bx + 2, 6, 11, 4, theme::INK);

    // Wifi icon: cluster of pixels suggesting concentric arcs.
    let wx = W - 78;
    let dots: &[(i32, i32)] = &[
        (wx + 4, 10), (wx + 3, 10), (wx + 5, 10),
        (wx + 4, 9),
        (wx + 3, 8),  (wx + 5, 8),
        (wx + 2, 7),  (wx + 6, 7),
    ];
    for &(x, y) in dots { screen::pixel(x, y, theme::INK); }
}

// ---------------------------------------------------------------------------
// Hint bar (bottom strip).
// ---------------------------------------------------------------------------

/// Bottom hint strip — typically used to show button mappings ("A: LAUNCH
/// B: BACK"). Drawn at the very bottom of the screen.
pub fn hint_bar(text: &str) {
    let y = H - 14;
    screen::line(0, y, W, y, theme::INK_DEEP);
    screen::text(6, H - 5, text, theme::INK_DIM, font::PRIMARY);
}

// ---------------------------------------------------------------------------
// Card (selectable rect with optional accent corner brackets).
// ---------------------------------------------------------------------------

/// Selection-capable rectangle. When `selected` is true the border switches
/// to the accent color and four corner brackets appear outside the box —
/// the "this is the focused thing" visual idiom from the design.
pub fn card(x: i32, y: i32, w: i32, h: i32, selected: bool) {
    let border = if selected { theme::ACCENT } else { theme::INK_DEEP };
    screen::stroke_rect(x, y, w, h, border);
    if selected {
        let oc = x - 3;
        let oy = y - 3;
        let ow = w + 6;
        let oh = h + 6;
        let l = 6;
        // TL
        screen::line(oc,         oy,         oc + l,     oy,         theme::ACCENT);
        screen::line(oc,         oy,         oc,         oy + l,     theme::ACCENT);
        // TR
        screen::line(oc + ow,    oy,         oc + ow - l, oy,        theme::ACCENT);
        screen::line(oc + ow,    oy,         oc + ow,    oy + l,     theme::ACCENT);
        // BL
        screen::line(oc,         oy + oh,    oc + l,     oy + oh,    theme::ACCENT);
        screen::line(oc,         oy + oh,    oc,         oy + oh - l, theme::ACCENT);
        // BR
        screen::line(oc + ow,    oy + oh,    oc + ow - l, oy + oh,   theme::ACCENT);
        screen::line(oc + ow,    oy + oh,    oc + ow,    oy + oh - l, theme::ACCENT);
    }
}

// ---------------------------------------------------------------------------
// Geometric fox mark — design's logo glyph.
// ---------------------------------------------------------------------------

/// Variant of the fox mark — different eyes per emotional state.
#[derive(Copy, Clone)]
pub enum FoxState {
    /// Default — small bright dots for eyes.
    Idle,
    /// Closed-eye sleep — thin horizontal lines.
    Sleep,
    /// Cyan accent eyes for "alert / on" states.
    Alert,
}

/// The geometric Fri3d fox mark — inverted diamond face with two
/// triangular ears (matches the camp logo). Drawn in a 28x30 box centred
/// horizontally on `(x_center, y_top)` (y is the top of the ears).
/// `color` is the stroke color; eyes always use WHITE or ACCENT depending
/// on `state`.
pub fn fox_mark(x_center: i32, y_top: i32, color: u32, state: FoxState) {
    let x = x_center - 14;
    let y = y_top + 8; // y_top is the ear tip; the face starts 8px below
    // Face: inverted diamond
    screen::line(x,      y,      x + 28, y,      color);
    screen::line(x,      y,      x + 14, y + 22, color);
    screen::line(x + 28, y,      x + 14, y + 22, color);
    // Snout detail
    screen::line(x,      y,      x + 14, y + 10, color);
    screen::line(x + 28, y,      x + 14, y + 10, color);
    screen::line(x + 14, y + 10, x + 14, y + 22, color);
    // Ears (point upward)
    screen::line(x,      y,      x + 4,  y - 8,  color);
    screen::line(x + 4,  y - 8,  x + 10, y,      color);
    screen::line(x + 28, y,      x + 24, y - 8,  color);
    screen::line(x + 24, y - 8,  x + 18, y,      color);
    // Eyes
    match state {
        FoxState::Sleep => {
            screen::line(x + 5,  y + 6, x + 9,  y + 6, theme::WHITE);
            screen::line(x + 19, y + 6, x + 23, y + 6, theme::WHITE);
        }
        FoxState::Alert => {
            screen::pixel(x + 7,  y + 6, theme::ACCENT);
            screen::pixel(x + 21, y + 6, theme::ACCENT);
            screen::pixel(x + 7,  y + 7, theme::ACCENT);
            screen::pixel(x + 21, y + 7, theme::ACCENT);
        }
        FoxState::Idle => {
            screen::pixel(x + 7,  y + 6, theme::WHITE);
            screen::pixel(x + 21, y + 6, theme::WHITE);
            screen::pixel(x + 7,  y + 7, theme::WHITE);
            screen::pixel(x + 21, y + 7, theme::WHITE);
        }
    }
}

/// A small "filled" version of the fox face — useful for in-line glyphs
/// (foxmojis, status icons). Just the inverted diamond + ears, no snout
/// detail. `size` is the width/height of the face triangle (typical 12).
pub fn fox_glyph(cx: i32, cy: i32, size: i32, color: u32) {
    let half = size / 2;
    // Inverted-diamond face
    let face: [i32; 6] = [
        cx - half, cy - half / 2,
        cx + half, cy - half / 2,
        cx,        cy + half,
    ];
    screen::polygon_fill(&face, color);
    // Ears
    let h = half / 2;
    let ear_l: [i32; 6] = [
        cx - half,     cy - half / 2,
        cx - half + h, cy - half - h,
        cx - h,        cy - half / 2,
    ];
    let ear_r: [i32; 6] = [
        cx + half,     cy - half / 2,
        cx + half - h, cy - half - h,
        cx + h,        cy - half / 2,
    ];
    screen::polygon_fill(&ear_l, color);
    screen::polygon_fill(&ear_r, color);
}

// ---------------------------------------------------------------------------
// Centre helpers.
// ---------------------------------------------------------------------------

/// Approximate text width for the PRIMARY font — useful for centring
/// without an extra host call. Each glyph is ~6px wide on average.
pub fn text_width_approx(text: &str) -> i32 {
    text.len() as i32 * 6
}

/// Draw `text` centred horizontally between `x0` and `x1`, baseline at `y`.
pub fn text_centered(x0: i32, x1: i32, y: i32, text: &str, rgb: u32, font_id: u32) {
    let tw = text_width_approx(text);
    let x = x0 + ((x1 - x0) - tw) / 2;
    screen::text(x, y, text, rgb, font_id);
}
