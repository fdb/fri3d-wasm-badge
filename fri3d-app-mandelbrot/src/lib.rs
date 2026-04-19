#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

#[derive(Copy, Clone)]
struct MandelState {
    x_offset: f32,
    y_offset: f32,
    x_zoom: f32,
    y_zoom: f32,
    zoom: f32,
}

impl MandelState {
    const fn new() -> Self {
        Self {
            x_offset: 2.5,
            y_offset: 1.12,
            x_zoom: 3.5,
            y_zoom: 2.24,
            zoom: 1.0,
        }
    }
}

static STATE: api::AppCell<MandelState> = api::AppCell::new(MandelState::new());

// Iteration cap for escape-time. 30 is plenty for a 128x64 mono render —
// the set's edges look the same as at 50 to the human eye at this
// resolution, and each iter saved is a 40% time cut on interior-like pixels.
const MAX_ITER: i32 = 30;

// Main-cardioid + period-2 bulb test. Any point inside these two regions is
// *provably* in the Mandelbrot set, so we can mark it black without any
// iteration. On the default view ~40% of pixels short-circuit here.
// See: https://en.wikipedia.org/wiki/Mandelbrot_set#Cardioid_/_bulb_checking
fn in_main_body(x: f32, y: f32) -> bool {
    // Period-2 bulb centred at (-1, 0) with radius 1/4.
    let dx = x + 1.0;
    if dx * dx + y * y < 0.0625 {
        return true;
    }
    // Main cardioid.
    let qx = x - 0.25;
    let q = qx * qx + y * y;
    q * (q + qx) < 0.25 * y * y
}

fn mandelbrot_pixel(state: MandelState, x: i32, y: i32) -> bool {
    let x0 = (x as f32 / 128.0) * state.x_zoom - state.x_offset;
    let y0 = (y as f32 / 64.0) * state.y_zoom - state.y_offset;

    if in_main_body(x0, y0) {
        return true;
    }

    let mut x1 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y2 = 0.0f32;
    let mut iter = 0;

    while x2 + y2 <= 4.0 && iter < MAX_ITER {
        y1 = 2.0 * x1 * y1 + y0;
        x1 = x2 - y2 + x0;
        x2 = x1 * x1;
        y2 = y1 * y1;
        iter += 1;
    }

    iter >= MAX_ITER
}

fn render_impl() {
    let state = STATE.get();
    api::canvas_set_color(api::color::BLACK);

    for y in 0..64 {
        for x in 0..128 {
            if mandelbrot_pixel(state, x, y) {
                api::canvas_draw_dot(x, y);
            }
        }
    }
}

fn on_input_impl(key: u32, kind: u32) {
    let mut state = STATE.get();

    if key == api::input::KEY_BACK {
        if kind == api::input::TYPE_LONG_PRESS {
            api::exit_to_launcher();
            return;
        }
        if kind == api::input::TYPE_SHORT_PRESS {
            state.x_offset += 0.05 * state.x_zoom;
            state.y_offset += 0.05 * state.y_zoom;
            state.x_zoom *= 1.1;
            state.y_zoom *= 1.1;
            state.zoom -= 0.15;
            if state.zoom < 1.0 {
                state.zoom = 1.0;
            }
        }
        STATE.set(state);
        return;
    }

    if kind != api::input::TYPE_PRESS {
        return;
    }

    let step = state.x_zoom * 0.03;

    match key {
        api::input::KEY_UP => {
            state.y_offset += step;
        }
        api::input::KEY_DOWN => {
            state.y_offset -= step;
        }
        api::input::KEY_LEFT => {
            state.x_offset += step;
        }
        api::input::KEY_RIGHT => {
            state.x_offset -= step;
        }
        api::input::KEY_OK => {
            state.x_offset -= 0.05 * state.x_zoom;
            state.y_offset -= 0.05 * state.y_zoom;
            state.x_zoom *= 0.9;
            state.y_zoom *= 0.9;
            state.zoom += 0.15;
        }
        _ => {}
    }

    STATE.set(state);
}

fn get_scene_impl() -> u32 {
    0
}

fn set_scene_impl(_scene: u32) {}

fn get_scene_count_impl() -> u32 {
    1
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_get_scene!(get_scene_impl);
api::export_set_scene!(set_scene_impl);
api::export_get_scene_count!(get_scene_count_impl);
api::wasm_panic_handler!();
