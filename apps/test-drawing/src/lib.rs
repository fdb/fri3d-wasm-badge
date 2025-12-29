//! Test Drawing - Comprehensive drawing primitives test

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const NUM_SCENES: u32 = 12;

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
    // Draw horizontal lines across the screen
    for y in (0..64).step_by(5) {
        canvas::draw_line(0, y, 127, y);
    }
}

fn scene_vertical_lines() {
    // Draw vertical lines across the screen
    for x in (0..128).step_by(8) {
        canvas::draw_line(x, 0, x, 63);
    }
}

fn scene_diagonal_lines() {
    // Draw diagonal lines
    canvas::draw_line(0, 0, 127, 63);
    canvas::draw_line(127, 0, 0, 63);
    canvas::draw_line(0, 32, 64, 0);
    canvas::draw_line(0, 32, 64, 63);
    canvas::draw_line(127, 32, 64, 0);
    canvas::draw_line(127, 32, 64, 63);
}

fn scene_random_pixels() {
    // Draw a pattern of pixels (deterministic for testing)
    for y in 0..64 {
        for x in 0..128 {
            // Create a pseudo-random pattern using simple hash
            let hash = (x * 7 + y * 13) % 17;
            if hash < 3 {
                canvas::draw_pixel(x, y);
            }
        }
    }
}

fn scene_circles() {
    // Draw outline circles - concentric in center, corners
    canvas::draw_circle(64, 32, 28);
    canvas::draw_circle(64, 32, 20);
    canvas::draw_circle(64, 32, 12);

    // Corner circles
    canvas::draw_circle(12, 12, 10);
    canvas::draw_circle(115, 12, 10);
    canvas::draw_circle(12, 51, 10);
    canvas::draw_circle(115, 51, 10);
}

fn scene_filled_circles() {
    // Draw filled circles
    canvas::fill_circle(32, 32, 25);
    canvas::fill_circle(96, 32, 20);
    canvas::fill_circle(64, 32, 10);
}

fn scene_rectangles() {
    // Draw nested rectangles
    canvas::draw_rect(4, 4, 120, 56);
    canvas::draw_rect(12, 10, 104, 44);
    canvas::draw_rect(20, 16, 88, 32);
    canvas::draw_rect(28, 22, 72, 20);
    canvas::draw_rect(36, 28, 56, 8);
}

fn scene_filled_rectangles() {
    // Draw filled rectangles
    canvas::fill_rect(10, 10, 40, 20);
    canvas::fill_rect(60, 10, 30, 25);
    canvas::fill_rect(100, 5, 20, 54);
    canvas::fill_rect(10, 40, 80, 15);
}

fn scene_rounded_rectangles() {
    // Draw rounded rectangles
    canvas::draw_round_rect(10, 10, 50, 20, 5);
    canvas::draw_round_rect(70, 10, 50, 20, 8);
    canvas::fill_round_rect(10, 38, 50, 20, 5);
    canvas::fill_round_rect(70, 38, 50, 20, 10);
}

fn scene_text_rendering() {
    // Draw text at various positions
    canvas::draw_str(0, 0, "Primary Font");
    canvas::draw_str(0, 16, "Secondary Font Test");
    canvas::draw_str(0, 32, "Keyboard: ABCDEF");
    canvas::draw_str(0, 48, "123");
}

fn scene_mixed_primitives() {
    // Mix of different primitives
    canvas::draw_rect(5, 5, 50, 30);
    canvas::draw_circle(80, 20, 15);
    canvas::draw_line(5, 45, 122, 45);
    canvas::fill_rect(100, 35, 25, 25);
    canvas::draw_str(10, 50, "Mixed");
}

fn scene_checkerboard() {
    // Draw a checkerboard pattern
    for y in 0..8 {
        for x in 0..16 {
            if (x + y) % 2 == 0 {
                canvas::fill_rect(x * 8, y * 8, 8, 8);
            }
        }
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
