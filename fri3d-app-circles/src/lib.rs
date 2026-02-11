#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

static SEED: api::AppCell<u32> = api::AppCell::new(42);

fn render_impl() {
    api::random_seed(SEED.get());
    api::canvas_set_color(api::color::BLACK);

    for _ in 0..10 {
        let x = api::random_range(128) as i32;
        let y = api::random_range(64) as i32;
        let radius = api::random_range(15) + 3;
        api::canvas_draw_circle(x, y, radius);
    }
}

fn on_input_impl(key: u32, kind: u32) {
    if key == api::input::KEY_BACK
        && (kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_LONG_PRESS)
    {
        api::exit_to_launcher();
        return;
    }

    if key == api::input::KEY_OK && kind == api::input::TYPE_PRESS {
        SEED.set(api::random_get());
    }
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
