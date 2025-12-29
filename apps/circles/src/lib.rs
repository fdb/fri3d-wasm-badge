//! Circles - Random circles demo

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn render() {
    canvas::set_color(canvas::Color::Black);

    for _ in 0..10 {
        let x = random::range(128) as i32;
        let y = random::range(64) as i32;
        let r = random::range(15) + 3;
        canvas::draw_circle(x, y, r);
    }
}

#[no_mangle]
pub extern "C" fn on_input(_key: u32, _event_type: u32) {
    // No input handling needed for this demo
}
