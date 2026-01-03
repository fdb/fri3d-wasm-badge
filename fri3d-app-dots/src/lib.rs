#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use core::sync::atomic::{AtomicU32, Ordering};

const DOT_COUNT: u32 = 200;
const SEED_MAX: u32 = i32::MAX as u32;
static SEED: AtomicU32 = AtomicU32::new(1);

fn render_impl() {
    let width = api::canvas_width();
    let height = api::canvas_height();

    if width == 0 || height == 0 {
        return;
    }

    api::canvas_set_color(1);

    let mut state = SEED.load(Ordering::Relaxed);
    if state == 0 {
        state = 1;
    }

    for _ in 0..DOT_COUNT {
        state = lcg(state);
        let x = state % width;
        state = lcg(state);
        let y = state % height;
        api::canvas_draw_dot(x, y);
    }
}

fn on_input_impl(key: u32, kind: u32) {
    if key == api::input::KEY_OK && kind == api::input::TYPE_PRESS {
        let mut seed = api::random_range(SEED_MAX);
        if seed == 0 {
            seed = 1;
        }
        SEED.store(seed, Ordering::Relaxed);
    }
}

fn lcg(state: u32) -> u32 {
    state
        .wrapping_mul(1664525)
        .wrapping_add(1013904223)
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
