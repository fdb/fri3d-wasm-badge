//! Test Drawing - Comprehensive drawing primitives test
//!
//! This must match the original C implementation exactly for visual regression testing.

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const NUM_SCENES: u32 = 12;
const RANDOM_SEED: u32 = 12345;

static mut SCENE: u32 = 0;

#[no_mangle]
pub extern "C" fn render() {
    let scene = unsafe { SCENE };

    canvas::set_color(canvas::Color::Black);

    match scene {
        0 => scene_horizontal_lines(),
        1 => scene_vertical_lines(),
        2 => scene_diagonal_lines(),
        3 => scene_random_pixels(),
        4 => scene_circles(),
        5 => scene_filled_circles(),
        6 => scene_rectangles(),
        7 => scene_filled_rectangles(),
        8 => scene_rounded_rectangles(),
        9 => scene_text_rendering(),
        10 => scene_mixed_primitives(),
        11 => scene_checkerboard(),
        _ => {}
    }
}

fn scene_horizontal_lines() {
    // Draw horizontal lines at regular intervals
    let mut y = 0i32;
    while y < 64 {
        canvas::draw_line(0, y, 127, y);
        y += 8;
    }
    // Draw some shorter lines
    y = 4;
    while y < 64 {
        canvas::draw_line(20, y, 107, y);
        y += 16;
    }
}

fn scene_vertical_lines() {
    // Draw vertical lines at regular intervals
    let mut x = 0i32;
    while x < 128 {
        canvas::draw_line(x, 0, x, 63);
        x += 8;
    }
    // Draw some shorter lines
    x = 4;
    while x < 128 {
        canvas::draw_line(x, 10, x, 53);
        x += 16;
    }
}

fn scene_diagonal_lines() {
    // Draw diagonal lines from corners
    canvas::draw_line(0, 0, 127, 63);
    canvas::draw_line(127, 0, 0, 63);
    // Draw parallel diagonal lines
    let mut i = 0i32;
    while i < 128 {
        canvas::draw_line(i, 0, i + 63, 63);
        canvas::draw_line(127 - i, 0, 127 - i - 63, 63);
        i += 16;
    }
}

fn scene_random_pixels() {
    random::seed(RANDOM_SEED);
    // Draw 500 random pixels
    for _ in 0..500 {
        let x = random::range(128) as i32;
        let y = random::range(64) as i32;
        canvas::draw_pixel(x, y);
    }
}

fn scene_circles() {
    random::seed(RANDOM_SEED);
    // Draw circles of various sizes
    canvas::draw_circle(64, 32, 30); // Large center circle
    canvas::draw_circle(64, 32, 20);
    canvas::draw_circle(64, 32, 10);
    // Draw circles in corners
    canvas::draw_circle(15, 15, 12);
    canvas::draw_circle(112, 15, 12);
    canvas::draw_circle(15, 48, 12);
    canvas::draw_circle(112, 48, 12);
}

fn scene_filled_circles() {
    random::seed(RANDOM_SEED);
    // Draw filled circles (discs)
    canvas::fill_circle(32, 32, 20);
    canvas::fill_circle(96, 32, 20);
    // Smaller discs
    canvas::fill_circle(64, 16, 8);
    canvas::fill_circle(64, 48, 8);
    // Use XOR for overlapping effect
    canvas::set_color(canvas::Color::Xor);
    canvas::fill_circle(64, 32, 18);
}

fn scene_rectangles() {
    // Draw concentric rectangles
    canvas::draw_rect(4, 4, 120, 56);
    canvas::draw_rect(14, 10, 100, 44);
    canvas::draw_rect(24, 16, 80, 32);
    canvas::draw_rect(34, 22, 60, 20);
    // Small rectangles in corners
    canvas::draw_rect(0, 0, 20, 15);
    canvas::draw_rect(108, 0, 20, 15);
    canvas::draw_rect(0, 49, 20, 15);
    canvas::draw_rect(108, 49, 20, 15);
}

fn scene_filled_rectangles() {
    // Draw filled rectangles
    canvas::fill_rect(10, 10, 30, 20);
    canvas::fill_rect(88, 10, 30, 20);
    canvas::fill_rect(10, 34, 30, 20);
    canvas::fill_rect(88, 34, 30, 20);
    // Center box with XOR
    canvas::set_color(canvas::Color::Xor);
    canvas::fill_rect(30, 20, 68, 24);
}

fn scene_rounded_rectangles() {
    // Draw rounded rectangles with various corner radii
    canvas::draw_round_rect(5, 5, 50, 25, 3);
    canvas::draw_round_rect(73, 5, 50, 25, 8);
    // Filled rounded rectangles
    canvas::fill_round_rect(5, 34, 50, 25, 5);
    canvas::fill_round_rect(73, 34, 50, 25, 10);
}

fn scene_text_rendering() {
    // Primary font
    canvas::set_font(canvas::Font::Primary);
    canvas::draw_str(5, 12, "Primary Font");
    // Secondary font
    canvas::set_font(canvas::Font::Secondary);
    canvas::draw_str(5, 24, "Secondary Font Test");
    // Keyboard font
    canvas::set_font(canvas::Font::Keyboard);
    canvas::draw_str(5, 36, "Keyboard: ABCDEF");
    // Big numbers
    canvas::set_font(canvas::Font::BigNumbers);
    canvas::draw_str(5, 58, "123");
}

fn scene_mixed_primitives() {
    random::seed(RANDOM_SEED);

    // Background grid
    let mut x = 0i32;
    while x < 128 {
        canvas::draw_line(x, 0, x, 63);
        x += 16;
    }
    let mut y = 0i32;
    while y < 64 {
        canvas::draw_line(0, y, 127, y);
        y += 16;
    }

    // Circles
    canvas::draw_circle(32, 32, 15);
    canvas::fill_circle(96, 32, 10);

    // Rectangles
    canvas::draw_rect(50, 10, 28, 20);
    canvas::fill_rect(52, 38, 24, 16);

    // Random pixels
    for _ in 0..50 {
        let rx = random::range(128) as i32;
        let ry = random::range(64) as i32;
        canvas::draw_pixel(rx, ry);
    }

    // Text
    canvas::set_font(canvas::Font::Secondary);
    canvas::draw_str(2, 8, "Mix");
}

fn scene_checkerboard() {
    // Draw 8x8 pixel checkerboard pattern
    let mut y = 0i32;
    while y < 64 {
        let mut x = 0i32;
        while x < 128 {
            // Alternate filled boxes
            if ((x / 8) + (y / 8)) % 2 == 0 {
                canvas::fill_rect(x, y, 8, 8);
            }
            x += 8;
        }
        y += 8;
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    if event_type != InputType::Short as u32 {
        return;
    }

    let scene = unsafe { &mut SCENE };

    match key {
        k if k == InputKey::Right as u32 || k == InputKey::Ok as u32 => {
            *scene = (*scene + 1) % NUM_SCENES;
        }
        k if k == InputKey::Left as u32 => {
            *scene = if *scene == 0 { NUM_SCENES - 1 } else { *scene - 1 };
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn get_scene_count() -> u32 {
    NUM_SCENES
}

#[no_mangle]
pub extern "C" fn get_scene() -> u32 {
    unsafe { SCENE }
}

#[no_mangle]
pub extern "C" fn set_scene(scene: u32) {
    unsafe {
        SCENE = scene % NUM_SCENES;
    }
}
