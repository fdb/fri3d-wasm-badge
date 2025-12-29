//! Circles - Random circles demo
//!
//! This must match the original C implementation exactly for visual regression testing.

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Current random seed
static mut SEED: u32 = 42;

// Scene control for visual testing
#[no_mangle]
pub extern "C" fn get_scene() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn set_scene(_scene: u32) {}

#[no_mangle]
pub extern "C" fn get_scene_count() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn render() {
    // Use same seed each frame for consistent circles
    random::seed(unsafe { SEED });
    canvas::set_color(canvas::Color::Black);

    // Draw 10 random circles
    for _ in 0..10 {
        let x = random::range(128) as i32;
        let y = random::range(64) as i32;
        let r = random::range(15) + 3;
        canvas::draw_circle(x, y, r);
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    // Only respond to short press
    if event_type != InputType::Short as u32 {
        return;
    }

    // Generate new random seed for new circles on OK press
    if key == InputKey::Ok as u32 {
        unsafe {
            SEED = random::get();
        }
    }
}
