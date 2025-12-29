//! Circles - Random circles demo

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Flag to trigger regeneration of circles
static mut REGENERATE: bool = true;

/// Stored circle data (x, y, radius, filled)
static mut CIRCLES: [(i32, i32, u32, bool); 10] = [(0, 0, 0, false); 10];

fn generate_circles() {
    unsafe {
        for i in 0..10 {
            CIRCLES[i] = (
                random::range(128) as i32,
                random::range(64) as i32,
                random::range(15) + 3,
                random::range(2) == 0,
            );
        }
        REGENERATE = false;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        if REGENERATE {
            generate_circles();
        }
    }

    canvas::set_color(canvas::Color::Black);

    unsafe {
        for (x, y, r, filled) in CIRCLES.iter() {
            if *filled {
                canvas::fill_circle(*x, *y, *r);
            } else {
                canvas::draw_circle(*x, *y, *r);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    // Only respond to short press
    if event_type != InputType::Short as u32 {
        return;
    }

    // Regenerate on OK/Enter press
    if key == InputKey::Ok as u32 {
        unsafe {
            REGENERATE = true;
        }
    }
}
