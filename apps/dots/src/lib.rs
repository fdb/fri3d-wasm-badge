#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

const DOT_COUNT: u32 = 200;
const SEED_MAX: u32 = i32::MAX as u32;
static SEED: api::AppCell<u32> = api::AppCell::new(1);

fn render_impl() {
    let width = api::canvas_width();
    let height = api::canvas_height();

    if width == 0 || height == 0 {
        return;
    }

    api::canvas_set_color(api::color::BLACK);

    let mut state = SEED.get();
    if state == 0 {
        state = 1;
    }

    for _ in 0..DOT_COUNT {
        state = lcg(state);
        let x = (state % width) as i32;
        state = lcg(state);
        let y = (state % height) as i32;
        api::canvas_draw_dot(x, y);
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();

fn on_input_impl(key: u32, kind: u32) {
    if key == api::input::KEY_BACK
        && (kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_LONG_PRESS)
    {
        api::exit_to_launcher();
        return;
    }

    if key == api::input::KEY_OK && kind == api::input::TYPE_PRESS {
        let mut seed = api::random_range(SEED_MAX);
        if seed == 0 {
            seed = 1;
        }
        SEED.set(seed);
    }
}

fn lcg(state: u32) -> u32 {
    state
        .wrapping_mul(1664525)
        .wrapping_add(1013904223)
}
