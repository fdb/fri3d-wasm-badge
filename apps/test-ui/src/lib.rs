//! Test UI - UI widget test

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut SELECTED: usize = 0;
static mut TOGGLE: bool = false;

const MENU_ITEMS: [&str; 4] = ["Option 1", "Option 2", "Option 3", "Toggle"];

#[no_mangle]
pub extern "C" fn render() {
    let selected = unsafe { SELECTED };
    let toggle = unsafe { TOGGLE };

    canvas::set_color(canvas::Color::Black);

    // Draw menu items
    for (i, _item) in MENU_ITEMS.iter().enumerate() {
        let y = 10 + (i as i32 * 12);

        if i == selected {
            // Highlight selected item
            canvas::fill_rect(0, y - 2, 128, 12);
            canvas::set_color(canvas::Color::White);
        }

        // Draw item text (simplified - just draw a box for each item)
        let w = 50 + (i as u32 * 10);
        canvas::draw_rect(4, y, w, 8);

        if i == selected {
            canvas::set_color(canvas::Color::Black);
        }
    }

    // Draw toggle state
    if toggle {
        canvas::fill_circle(120, 52, 5);
    } else {
        canvas::draw_circle(120, 52, 5);
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    if event_type != InputType::Short as u32 {
        return;
    }

    let selected = unsafe { &mut SELECTED };
    let toggle = unsafe { &mut TOGGLE };

    match key {
        k if k == InputKey::Up as u32 => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        k if k == InputKey::Down as u32 => {
            if *selected < MENU_ITEMS.len() - 1 {
                *selected += 1;
            }
        }
        k if k == InputKey::Ok as u32 => {
            if *selected == 3 {
                *toggle = !*toggle;
            }
        }
        _ => {}
    }
}
