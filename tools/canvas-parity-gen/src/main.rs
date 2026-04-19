//! Generate pixel-perfect golden framebuffers from the Rust Canvas reference
//! implementation. The C++ port in firmware/src/canvas.cpp must produce
//! byte-identical output for each test case.
//!
//! Each test runs on a fresh 128×64 Canvas, performs the named drawing op,
//! then writes the raw 8192-byte framebuffer to tests/canvas-golden/<name>.bin.
//!
//! Convention: each byte is 0 (white / background) or 1 (black / foreground),
//! matching fri3d-runtime/src/canvas.rs. This is not a PNG — it's the raw
//! buffer so C++ can mmap-compare in O(n) without decoding.

use fri3d_runtime::canvas::Canvas;
use fri3d_runtime::types::{Color, Font};
use std::fs;
use std::path::Path;

fn dump(name: &str, canvas: &Canvas) {
    let path = Path::new("tests/canvas-golden").join(format!("{name}.bin"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, canvas.buffer()).expect("write golden");
    println!("  wrote {}", path.display());
}

/// Helper: run `f` on a fresh canvas and dump the result.
fn run<F: FnOnce(&mut Canvas)>(name: &str, f: F) {
    let mut c = Canvas::new();
    c.clear();
    f(&mut c);
    dump(name, &c);
}

fn main() {
    println!("generating canvas goldens...");

    // ---- Basic primitives ----------------------------------------------
    run("clear_empty", |_c| {});

    run("dot_single", |c| {
        c.set_color(Color::Black);
        c.draw_dot(3, 4);
    });
    run("dot_oob", |c| {
        // Out-of-bounds dots must be silently clipped, not panic.
        c.set_color(Color::Black);
        c.draw_dot(-1, -1);
        c.draw_dot(128, 64);
        c.draw_dot(200, 200);
    });

    run("line_horizontal", |c| {
        c.set_color(Color::Black);
        c.draw_line(10, 20, 60, 20);
    });
    run("line_vertical", |c| {
        c.set_color(Color::Black);
        c.draw_line(40, 5, 40, 50);
    });
    run("line_diagonal_down_right", |c| {
        c.set_color(Color::Black);
        c.draw_line(0, 0, 127, 63);
    });
    run("line_diagonal_up_left", |c| {
        c.set_color(Color::Black);
        c.draw_line(127, 63, 0, 0);
    });

    run("frame_full", |c| {
        c.set_color(Color::Black);
        c.draw_frame(0, 0, 128, 64);
    });
    run("frame_small_centered", |c| {
        c.set_color(Color::Black);
        c.draw_frame(40, 16, 48, 32);
    });

    run("box_full", |c| {
        c.set_color(Color::Black);
        c.draw_box(0, 0, 128, 64);
    });
    run("box_small", |c| {
        c.set_color(Color::Black);
        c.draw_box(20, 10, 30, 20);
    });

    run("circle_center", |c| {
        c.set_color(Color::Black);
        c.draw_circle(64, 32, 15);
    });
    run("circle_r0", |c| {
        c.set_color(Color::Black);
        c.draw_circle(64, 32, 0);
    });

    run("disc_center", |c| {
        c.set_color(Color::Black);
        c.draw_disc(64, 32, 15);
    });
    run("disc_r0", |c| {
        c.set_color(Color::Black);
        c.draw_disc(64, 32, 0);
    });

    // ---- Rounded variants (to be ported) -------------------------------
    run("rframe_5r", |c| {
        c.set_color(Color::Black);
        c.draw_rframe(10, 10, 60, 40, 5);
    });
    run("rframe_r0_equals_frame", |c| {
        c.set_color(Color::Black);
        c.draw_rframe(10, 10, 60, 40, 0);
    });
    run("rframe_large_radius_clamps", |c| {
        c.set_color(Color::Black);
        c.draw_rframe(10, 10, 20, 20, 100);
    });

    run("rbox_5r", |c| {
        c.set_color(Color::Black);
        c.draw_rbox(10, 10, 60, 40, 5);
    });
    run("rbox_r0_equals_box", |c| {
        c.set_color(Color::Black);
        c.draw_rbox(10, 10, 60, 40, 0);
    });

    // ---- XOR mode ------------------------------------------------------
    run("xor_disc_over_box", |c| {
        c.set_color(Color::Black);
        c.draw_box(20, 10, 60, 40);
        c.set_color(Color::Xor);
        c.draw_disc(50, 30, 12);
    });
    run("xor_circle_over_box", |c| {
        c.set_color(Color::Black);
        c.draw_box(20, 10, 60, 40);
        c.set_color(Color::Xor);
        c.draw_circle(50, 30, 12);
    });

    // ---- Text rendering ------------------------------------------------
    // Guards the u8g2 decoder port: bit-stream reader, UTF-8 decoder, glyph
    // blit. If any of these drift, every app that renders text will regress.
    run("text_hello_primary", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::Primary);
        c.draw_str(5, 20, "Hello");
    });
    run("text_ascii_primary", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::Primary);
        c.draw_str(2, 20, "The quick brown fox");
    });
    run("text_digits_bignumbers", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::BigNumbers);
        c.draw_str(10, 40, "01234");
    });
    run("text_secondary_line", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::Secondary);
        c.draw_str(0, 10, "ABCabc 0-9");
    });
    run("text_keyboard_row", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::Keyboard);
        c.draw_str(0, 10, "qwertyuiop");
    });
    run("text_multiline", |c| {
        c.set_color(Color::Black);
        c.set_font(Font::Primary);
        c.draw_str(2, 12, "Fri3d");
        c.draw_str(2, 28, "WASM");
        c.draw_str(2, 44, "Badge");
    });

    println!("done.");
}
