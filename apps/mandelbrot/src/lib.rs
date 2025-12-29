//! Mandelbrot Set Explorer

#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut STATE: MandelbrotState = MandelbrotState::new();

struct MandelbrotState {
    center_x: f32,
    center_y: f32,
    zoom: f32,
}

impl MandelbrotState {
    const fn new() -> Self {
        Self {
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    let state = unsafe { &STATE };

    let scale = 3.0 / (state.zoom * 64.0);

    for py in 0..64 {
        for px in 0..128 {
            let x0 = state.center_x + (px as f32 - 64.0) * scale;
            let y0 = state.center_y + (py as f32 - 32.0) * scale;

            let mut x = 0.0f32;
            let mut y = 0.0f32;
            let mut iter = 0u32;

            while x * x + y * y <= 4.0 && iter < 32 {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iter += 1;
            }

            if iter >= 32 {
                canvas::set_color(canvas::Color::Black);
                canvas::draw_pixel(px, py);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    let state = unsafe { &mut STATE };

    // Only respond to Short press
    if event_type != InputType::Short as u32 {
        return;
    }

    match key {
        k if k == InputKey::Ok as u32 => state.zoom *= 2.0,
        k if k == InputKey::Back as u32 => {
            state.zoom /= 2.0;
            if state.zoom < 0.1 {
                state.zoom = 0.1;
            }
        }
        k if k == InputKey::Up as u32 => state.center_y -= 0.5 / state.zoom,
        k if k == InputKey::Down as u32 => state.center_y += 0.5 / state.zoom,
        k if k == InputKey::Left as u32 => state.center_x -= 0.5 / state.zoom,
        k if k == InputKey::Right as u32 => state.center_x += 0.5 / state.zoom,
        _ => {}
    }
}
