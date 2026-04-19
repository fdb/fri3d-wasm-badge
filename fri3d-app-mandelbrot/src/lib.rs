#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

// ---------------------------------------------------------------------------
// Fixed-point (Q16.16) Mandelbrot.
//
// wasm3's interpreter pays a sizeable tax per f32 op, while i32 ops run at
// its full speed. For a 128×64 escape-time render that's a 5-10× speedup on
// ESP32-S3 hardware — from ~1100 ms/frame down to ~150-200 ms.
//
// Q16.16 gives us ±32k range with 1/65536 precision. The Mandelbrot inner
// loop never takes |x|, |y| above 2 before bailing (|z|² > 4), and the
// viewport coordinates fit well inside ±8, so 16 integer bits is overkill
// but buys cheap headroom without risking squared-overflow.

const FP_SHIFT: i32 = 16;
const FP_ONE: i32 = 1 << FP_SHIFT;
const FP_FOUR: i32 = 4 << FP_SHIFT;

#[inline]
fn fp_from_f32(v: f32) -> i32 {
    // Compile-time float -> fixed conversion (still f32 at compile time,
    // but const_eval happens once per STATE field update in on_input).
    (v * (FP_ONE as f32)) as i32
}

#[inline]
fn fp_mul(a: i32, b: i32) -> i32 {
    // Widen to i64 to avoid overflow on the multiply, shift back.
    (((a as i64) * (b as i64)) >> FP_SHIFT) as i32
}

// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct MandelState {
    // Viewport stored as f32 for readability in on_input; converted to
    // fixed-point per frame at render time.
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

// Iteration cap for escape-time. 30 is plenty for a 128x64 mono render.
const MAX_ITER: i32 = 30;

// Main-cardioid + period-2 bulb test (fixed-point). ~40% of default-view
// pixels short-circuit here without any iteration at all.
#[inline]
fn in_main_body(x: i32, y: i32) -> bool {
    // Period-2 bulb centred at (-1, 0) with radius 1/4:
    //   (x + 1)^2 + y^2  <  1/16   (= 0.0625 in fixed-point)
    let dx = x + FP_ONE;
    let bulb_lhs = fp_mul(dx, dx) + fp_mul(y, y);
    if bulb_lhs < (FP_ONE / 16) {
        return true;
    }
    // Main cardioid:
    //   q = (x - 0.25)^2 + y^2
    //   q * (q + (x - 0.25))  <  0.25 * y^2
    let qx = x - (FP_ONE / 4);
    let q = fp_mul(qx, qx) + fp_mul(y, y);
    fp_mul(q, q + qx) < fp_mul(FP_ONE / 4, fp_mul(y, y))
}

// Build a full 128x64 framebuffer in WASM memory using fixed-point math,
// then ship it to the host in one canvas_draw_buffer call.
fn render_impl() {
    static mut FB: [u8; 128 * 64] = [0; 128 * 64];

    let state = STATE.get();

    // Per-frame derived fixed-point viewport.
    let x_zoom = fp_from_f32(state.x_zoom);
    let y_zoom = fp_from_f32(state.y_zoom);
    let x_offset = fp_from_f32(state.x_offset);
    let y_offset = fp_from_f32(state.y_offset);

    // Per-pixel step in world space (fixed-point).
    let dx = x_zoom / 128;
    let dy = y_zoom / 64;

    // SAFETY: single-threaded WASM app; the static mut is only touched here.
    #[allow(unsafe_code)]
    let fb = unsafe { &mut FB };

    let mut y0 = -y_offset;
    for y in 0..64 {
        let mut x0 = -x_offset;
        for x in 0..128 {
            let inside = if in_main_body(x0, y0) {
                true
            } else {
                let mut x1: i32 = 0;
                let mut y1: i32 = 0;
                let mut x2: i32 = 0;
                let mut y2: i32 = 0;
                let mut iter = 0;
                while x2 + y2 <= FP_FOUR && iter < MAX_ITER {
                    // y1 = 2*x1*y1 + y0  (2*a*b in fixed-point is just a*b << 1 after fp_mul)
                    y1 = (fp_mul(x1, y1) << 1) + y0;
                    x1 = x2 - y2 + x0;
                    x2 = fp_mul(x1, x1);
                    y2 = fp_mul(y1, y1);
                    iter += 1;
                }
                iter >= MAX_ITER
            };
            fb[(y * 128 + x) as usize] = if inside { 1 } else { 0 };
            x0 += dx;
        }
        y0 += dy;
    }

    api::canvas_draw_buffer(fb);
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
