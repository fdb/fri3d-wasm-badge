#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

const RANDOM_SEED: u32 = 12345;

#[repr(u32)]
#[derive(Copy, Clone)]
enum Scene {
    HorizontalLines = 0,
    VerticalLines = 1,
    DiagonalLines = 2,
    RandomPixels = 3,
    Circles = 4,
    FilledCircles = 5,
    Rectangles = 6,
    FilledRectangles = 7,
    RoundedRectangles = 8,
    TextRendering = 9,
    MixedPrimitives = 10,
    Checkerboard = 11,
}

const SCENE_COUNT: u32 = 12;

static CURRENT_SCENE: api::AppCell<u32> = api::AppCell::new(Scene::HorizontalLines as u32);

fn render_impl() {
    match scene_from_u32(CURRENT_SCENE.get()) {
        Scene::HorizontalLines => render_horizontal_lines(),
        Scene::VerticalLines => render_vertical_lines(),
        Scene::DiagonalLines => render_diagonal_lines(),
        Scene::RandomPixels => render_random_pixels(),
        Scene::Circles => render_circles(),
        Scene::FilledCircles => render_filled_circles(),
        Scene::Rectangles => render_rectangles(),
        Scene::FilledRectangles => render_filled_rectangles(),
        Scene::RoundedRectangles => render_rounded_rectangles(),
        Scene::TextRendering => render_text(),
        Scene::MixedPrimitives => render_mixed_primitives(),
        Scene::Checkerboard => render_checkerboard(),
    }
}

fn on_input_impl(key: u32, kind: u32) {
    if key == api::input::KEY_BACK
        && (kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_LONG_PRESS)
    {
        api::exit_to_launcher();
        return;
    }

    if kind != api::input::TYPE_PRESS {
        return;
    }

    let current = CURRENT_SCENE.get();
    let next = match key {
        k if k == api::input::KEY_RIGHT || k == api::input::KEY_DOWN => {
            (current + 1) % SCENE_COUNT
        }
        k if k == api::input::KEY_LEFT || k == api::input::KEY_UP => {
            (current + SCENE_COUNT - 1) % SCENE_COUNT
        }
        _ => current,
    };

    CURRENT_SCENE.set(next);
}

fn get_scene_impl() -> u32 {
    CURRENT_SCENE.get()
}

fn set_scene_impl(scene: u32) {
    if scene < SCENE_COUNT {
        CURRENT_SCENE.set(scene);
    }
}

fn get_scene_count_impl() -> u32 {
    SCENE_COUNT
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_get_scene!(get_scene_impl);
api::export_set_scene!(set_scene_impl);
api::export_get_scene_count!(get_scene_count_impl);
api::wasm_panic_handler!();

fn render_horizontal_lines() {
    api::canvas_set_color(api::color::BLACK);
    for y in (0..64).step_by(8) {
        api::canvas_draw_line(0, y, 127, y);
    }
    for y in (4..64).step_by(16) {
        api::canvas_draw_line(20, y, 107, y);
    }
}

fn render_vertical_lines() {
    api::canvas_set_color(api::color::BLACK);
    for x in (0..128).step_by(8) {
        api::canvas_draw_line(x, 0, x, 63);
    }
    for x in (4..128).step_by(16) {
        api::canvas_draw_line(x, 10, x, 53);
    }
}

fn render_diagonal_lines() {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_line(0, 0, 127, 63);
    api::canvas_draw_line(127, 0, 0, 63);
    for i in (0..128).step_by(16) {
        api::canvas_draw_line(i, 0, i + 63, 63);
        api::canvas_draw_line(127 - i, 0, 127 - i - 63, 63);
    }
}

fn render_random_pixels() {
    api::random_seed(RANDOM_SEED);
    api::canvas_set_color(api::color::BLACK);
    for _ in 0..500 {
        let x = api::random_range(128) as i32;
        let y = api::random_range(64) as i32;
        api::canvas_draw_dot(x, y);
    }
}

fn render_circles() {
    api::random_seed(RANDOM_SEED);
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_circle(64, 32, 30);
    api::canvas_draw_circle(64, 32, 20);
    api::canvas_draw_circle(64, 32, 10);
    api::canvas_draw_circle(15, 15, 12);
    api::canvas_draw_circle(112, 15, 12);
    api::canvas_draw_circle(15, 48, 12);
    api::canvas_draw_circle(112, 48, 12);
}

fn render_filled_circles() {
    api::random_seed(RANDOM_SEED);
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_disc(32, 32, 20);
    api::canvas_draw_disc(96, 32, 20);
    api::canvas_draw_disc(64, 16, 8);
    api::canvas_draw_disc(64, 48, 8);
    api::canvas_set_color(api::color::XOR);
    api::canvas_draw_disc(64, 32, 18);
}

fn render_rectangles() {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_frame(4, 4, 120, 56);
    api::canvas_draw_frame(14, 10, 100, 44);
    api::canvas_draw_frame(24, 16, 80, 32);
    api::canvas_draw_frame(34, 22, 60, 20);
    api::canvas_draw_frame(0, 0, 20, 15);
    api::canvas_draw_frame(108, 0, 20, 15);
    api::canvas_draw_frame(0, 49, 20, 15);
    api::canvas_draw_frame(108, 49, 20, 15);
}

fn render_filled_rectangles() {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_box(10, 10, 30, 20);
    api::canvas_draw_box(88, 10, 30, 20);
    api::canvas_draw_box(10, 34, 30, 20);
    api::canvas_draw_box(88, 34, 30, 20);
    api::canvas_set_color(api::color::XOR);
    api::canvas_draw_box(30, 20, 68, 24);
}

fn render_rounded_rectangles() {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_draw_rframe(5, 5, 50, 25, 3);
    api::canvas_draw_rframe(73, 5, 50, 25, 8);
    api::canvas_draw_rbox(5, 34, 50, 25, 5);
    api::canvas_draw_rbox(73, 34, 50, 25, 10);
}

fn render_text() {
    api::canvas_set_color(api::color::BLACK);
    api::canvas_set_font(api::font::PRIMARY);
    api::canvas_draw_str(5, 12, "Primary Font");
    api::canvas_set_font(api::font::SECONDARY);
    api::canvas_draw_str(5, 24, "Secondary Font Test");
    api::canvas_set_font(api::font::KEYBOARD);
    api::canvas_draw_str(5, 36, "Keyboard: ABCDEF");
    api::canvas_set_font(api::font::BIG_NUMBERS);
    api::canvas_draw_str(5, 58, "123");
}

fn render_mixed_primitives() {
    api::random_seed(RANDOM_SEED);
    api::canvas_set_color(api::color::BLACK);

    for x in (0..128).step_by(16) {
        api::canvas_draw_line(x, 0, x, 63);
    }
    for y in (0..64).step_by(16) {
        api::canvas_draw_line(0, y, 127, y);
    }

    api::canvas_draw_circle(32, 32, 15);
    api::canvas_draw_disc(96, 32, 10);

    api::canvas_draw_frame(50, 10, 28, 20);
    api::canvas_draw_box(52, 38, 24, 16);

    for _ in 0..50 {
        let x = api::random_range(128) as i32;
        let y = api::random_range(64) as i32;
        api::canvas_draw_dot(x, y);
    }

    api::canvas_set_font(api::font::SECONDARY);
    api::canvas_draw_str(2, 8, "Mix");
}

fn render_checkerboard() {
    api::canvas_set_color(api::color::BLACK);
    for y in (0..64).step_by(8) {
        for x in (0..128).step_by(8) {
            if ((x / 8) + (y / 8)) % 2 == 0 {
                api::canvas_draw_box(x, y, 8, 8);
            }
        }
    }
}

fn scene_from_u32(scene: u32) -> Scene {
    match scene {
        0 => Scene::HorizontalLines,
        1 => Scene::VerticalLines,
        2 => Scene::DiagonalLines,
        3 => Scene::RandomPixels,
        4 => Scene::Circles,
        5 => Scene::FilledCircles,
        6 => Scene::Rectangles,
        7 => Scene::FilledRectangles,
        8 => Scene::RoundedRectangles,
        9 => Scene::TextRendering,
        10 => Scene::MixedPrimitives,
        _ => Scene::Checkerboard,
    }
}
