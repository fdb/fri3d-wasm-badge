//! Launcher - Built-in app launcher

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut SELECTED: usize = 0;

// App names (hardcoded for now)
const APPS: [&str; 4] = ["circles", "mandelbrot", "test_drawing", "test_ui"];

// Host function to launch an app (to be implemented)
extern "C" {
    fn launch_app(index: u32);
}

#[no_mangle]
pub extern "C" fn render() {
    let selected = unsafe { SELECTED };

    canvas::set_color(canvas::Color::Black);

    // Title bar
    canvas::fill_rect(0, 0, 128, 12);
    canvas::set_color(canvas::Color::White);
    // "Fri3d Badge" would go here with text rendering

    canvas::set_color(canvas::Color::Black);

    // Draw app list
    for (i, _app) in APPS.iter().enumerate() {
        let y = 16 + (i as i32 * 12);

        if i == selected {
            // Highlight selected
            canvas::fill_rect(0, y - 2, 128, 12);
            canvas::set_color(canvas::Color::White);
        }

        // Draw app name placeholder (box representing text)
        let w = 40 + (i as u32 * 15);
        canvas::draw_rect(8, y, w, 8);

        if i == selected {
            canvas::set_color(canvas::Color::Black);
        }
    }

    // Draw selection indicator
    let indicator_y = 16 + (selected as i32 * 12);
    canvas::draw_pixel(2, indicator_y + 2);
    canvas::draw_pixel(3, indicator_y + 3);
    canvas::draw_pixel(2, indicator_y + 4);
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    if event_type != InputType::Short as u32 {
        return;
    }

    let selected = unsafe { &mut SELECTED };

    match key {
        k if k == InputKey::Up as u32 => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        k if k == InputKey::Down as u32 => {
            if *selected < APPS.len() - 1 {
                *selected += 1;
            }
        }
        k if k == InputKey::Ok as u32 => {
            // Launch the selected app
            unsafe {
                launch_app(*selected as u32);
            }
        }
        _ => {}
    }
}
