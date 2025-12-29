//! Test Drawing - Drawing primitives test

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut SCENE: u32 = 0;

#[no_mangle]
pub extern "C" fn render() {
    let scene = unsafe { SCENE };

    canvas::set_color(canvas::Color::Black);

    match scene {
        0 => {
            // Lines
            canvas::draw_line(0, 0, 127, 63);
            canvas::draw_line(127, 0, 0, 63);
            canvas::draw_line(0, 32, 127, 32);
            canvas::draw_line(64, 0, 64, 63);
        }
        1 => {
            // Rectangles
            canvas::draw_rect(10, 10, 40, 30);
            canvas::fill_rect(60, 10, 40, 30);
            canvas::draw_round_rect(10, 45, 40, 15, 5);
            canvas::fill_round_rect(60, 45, 40, 15, 5);
        }
        2 => {
            // Circles
            canvas::draw_circle(32, 32, 20);
            canvas::fill_circle(96, 32, 20);
            canvas::draw_circle(64, 32, 30);
        }
        3 => {
            // XOR mode test
            canvas::fill_rect(20, 10, 88, 44);
            canvas::set_color(canvas::Color::Xor);
            canvas::fill_circle(64, 32, 25);
        }
        _ => {}
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
            *scene = (*scene + 1) % 4;
        }
        k if k == InputKey::Left as u32 => {
            *scene = if *scene == 0 { 3 } else { *scene - 1 };
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn get_scene_count() -> u32 {
    4
}

#[no_mangle]
pub extern "C" fn get_scene() -> u32 {
    unsafe { SCENE }
}

#[no_mangle]
pub extern "C" fn set_scene(scene: u32) {
    unsafe {
        SCENE = scene % 4;
    }
}
