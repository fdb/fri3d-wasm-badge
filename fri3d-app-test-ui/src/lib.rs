#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

const SCENE_COUNT: u32 = 7;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
enum Scene {
    Counter = 0,
    Menu = 1,
    Layout = 2,
    Progress = 3,
    Checkbox = 4,
    Footer = 5,
    Keyboard = 6,
}

static CURRENT_SCENE: api::AppCell<Scene> = api::AppCell::new(Scene::Counter);
static COUNTER: api::AppCell<i32> = api::AppCell::new(0);
static MENU_SCROLL: api::AppCell<i16> = api::AppCell::new(0);
static BRIGHTNESS: api::AppCell<i32> = api::AppCell::new(5);
static SOUND: api::AppCell<bool> = api::AppCell::new(true);
static VIBRATION: api::AppCell<bool> = api::AppCell::new(true);
static WIFI: api::AppCell<bool> = api::AppCell::new(false);
static PROGRESS: api::AppCell<f32> = api::AppCell::new(0.0);
static CHECK1: api::AppCell<bool> = api::AppCell::new(false);
static CHECK2: api::AppCell<bool> = api::AppCell::new(true);
static CHECK3: api::AppCell<bool> = api::AppCell::new(false);
static LEFT_COUNT: api::AppCell<i32> = api::AppCell::new(0);
static RIGHT_COUNT: api::AppCell<i32> = api::AppCell::new(0);
static KEYBOARD: api::AppCell<api::imgui::UiVirtualKeyboard<32>> =
    api::AppCell::new(api::imgui::UiVirtualKeyboard::new());
static KEYBOARD_SAVED: api::AppCell<bool> = api::AppCell::new(false);
static KEYBOARD_SAVED_UNTIL: api::AppCell<u32> = api::AppCell::new(0);
static KEYBOARD_SAVED_TEXT: api::AppCell<[u8; 32]> = api::AppCell::new([0u8; 32]);

fn get_scene_impl() -> u32 {
    CURRENT_SCENE.get() as u32
}

fn set_scene_impl(scene: u32) {
    if scene >= SCENE_COUNT {
        return;
    }

    let new_scene = match scene {
        0 => Scene::Counter,
        1 => Scene::Menu,
        2 => Scene::Layout,
        3 => Scene::Progress,
        4 => Scene::Checkbox,
        5 => Scene::Footer,
        _ => Scene::Keyboard,
    };

    CURRENT_SCENE.set(new_scene);
    MENU_SCROLL.set(0);

    if new_scene == Scene::Keyboard {
        init_keyboard();
    }
}

fn get_scene_count_impl() -> u32 {
    SCENE_COUNT
}

fn init_keyboard() {
    let mut keyboard = KEYBOARD.get();
    api::imgui::ui_virtual_keyboard_init(&mut keyboard, "guest");
    api::imgui::ui_virtual_keyboard_set_min_length(&mut keyboard, 3);
    keyboard.clear_default_text = true;
    KEYBOARD.set(keyboard);
    KEYBOARD_SAVED.set(false);
    KEYBOARD_SAVED_UNTIL.set(0);
    KEYBOARD_SAVED_TEXT.set([0u8; 32]);
}

fn write_str(buf: &mut [u8], mut idx: usize, text: &str) -> usize {
    for byte in text.as_bytes() {
        if idx + 1 >= buf.len() {
            break;
        }
        buf[idx] = *byte;
        idx += 1;
    }
    idx
}

fn write_i32(buf: &mut [u8], mut idx: usize, value: i32) -> usize {
    let mut v = value;
    if v < 0 {
        if idx + 1 < buf.len() {
            buf[idx] = b'-';
            idx += 1;
        }
        v = -v;
    }

    let mut digits = [0u8; 12];
    let mut len = 0usize;
    let mut n = v as u32;
    if n == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while n > 0 {
            digits[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
    }

    for i in (0..len).rev() {
        if idx + 1 >= buf.len() {
            break;
        }
        buf[idx] = digits[i];
        idx += 1;
    }

    idx
}

fn buf_to_str(buf: &[u8], len: usize) -> &str {
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

fn cstr_to_str(buf: &[u8]) -> &str {
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

fn copy_str(buf: &mut [u8], text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len().min(buf.len().saturating_sub(1));
    buf[..len].copy_from_slice(&bytes[..len]);
    if len < buf.len() {
        buf[len] = 0;
    }
}

fn render_counter() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Counter Demo", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(8);

    let mut buf = [0u8; 32];
    let mut idx = write_str(&mut buf, 0, "Count: ");
    idx = write_i32(&mut buf, idx, COUNTER.get());
    api::imgui::ui_label(buf_to_str(&buf, idx), api::font::SECONDARY, api::align::CENTER);
    api::imgui::ui_spacer(8);

    api::imgui::ui_hstack_centered(4);
    if api::imgui::ui_button("+") {
        COUNTER.set(COUNTER.get() + 1);
    }
    if api::imgui::ui_button("-") {
        COUNTER.set(COUNTER.get() - 1);
    }
    if api::imgui::ui_button("Reset") {
        COUNTER.set(0);
    }
    api::imgui::ui_end_stack();

    api::imgui::ui_end();
}

fn render_menu() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Settings Menu", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(4);

    let mut scroll = MENU_SCROLL.get();
    api::imgui::ui_menu_begin(&mut scroll, 4, 6);

    let mut bright_buf = [0u8; 8];
    let bright_len = write_i32(&mut bright_buf, 0, BRIGHTNESS.get());
    let bright_text = buf_to_str(&bright_buf, bright_len);
    if api::imgui::ui_menu_item_value("Brightness", bright_text, 0) {
        // no-op
    }

    if api::imgui::ui_menu_item_value("Sound", if SOUND.get() { "On" } else { "Off" }, 1) {
        SOUND.set(!SOUND.get());
    }

    if api::imgui::ui_menu_item_value("Vibration", if VIBRATION.get() { "On" } else { "Off" }, 2) {
        VIBRATION.set(!VIBRATION.get());
    }

    if api::imgui::ui_menu_item_value("WiFi", if WIFI.get() { "On" } else { "Off" }, 3) {
        WIFI.set(!WIFI.get());
    }

    if api::imgui::ui_menu_item("About", 4) {
        // no-op
    }

    if api::imgui::ui_menu_item("Reset All", 5) {
        BRIGHTNESS.set(5);
        SOUND.set(true);
        VIBRATION.set(true);
        WIFI.set(false);
    }

    api::imgui::ui_menu_end();
    MENU_SCROLL.set(scroll);

    api::imgui::ui_end();
}

fn render_layout() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Layout Demo", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(2);

    api::imgui::ui_label("Left", api::font::SECONDARY, api::align::LEFT);
    api::imgui::ui_label("Center", api::font::SECONDARY, api::align::CENTER);
    api::imgui::ui_label("Right", api::font::SECONDARY, api::align::RIGHT);

    api::imgui::ui_separator();

    api::imgui::ui_hstack_centered(4);
    api::imgui::ui_button("A");
    api::imgui::ui_button("B");
    api::imgui::ui_button("C");
    api::imgui::ui_end_stack();

    api::imgui::ui_end();
}

fn render_progress() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Progress Demo", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(4);

    api::imgui::ui_label("Loading:", api::font::SECONDARY, api::align::LEFT);
    api::imgui::ui_spacer(2);
    api::imgui::ui_progress(PROGRESS.get(), 0);
    api::imgui::ui_spacer(4);

    api::imgui::ui_label("25%:", api::font::SECONDARY, api::align::LEFT);
    api::imgui::ui_spacer(2);
    api::imgui::ui_progress(0.25, 0);

    api::imgui::ui_label("50%:", api::font::SECONDARY, api::align::LEFT);
    api::imgui::ui_spacer(2);
    api::imgui::ui_progress(0.50, 0);

    api::imgui::ui_label("75%:", api::font::SECONDARY, api::align::LEFT);
    api::imgui::ui_spacer(2);
    api::imgui::ui_progress(0.75, 0);

    let mut progress = PROGRESS.get() + 0.02;
    if progress > 1.0 {
        progress = 0.0;
    }
    PROGRESS.set(progress);

    api::imgui::ui_end();
}

fn render_checkbox() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Checkbox Demo", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(8);

    let mut check1 = CHECK1.get();
    if api::imgui::ui_checkbox("Option 1", &mut check1) {
        // no-op
    }
    CHECK1.set(check1);

    let mut check2 = CHECK2.get();
    if api::imgui::ui_checkbox("Option 2 (default on)", &mut check2) {
        // no-op
    }
    CHECK2.set(check2);

    let mut check3 = CHECK3.get();
    if api::imgui::ui_checkbox("Option 3", &mut check3) {
        // no-op
    }
    CHECK3.set(check3);

    api::imgui::ui_spacer(8);

    let mut buf = [0u8; 64];
    let mut idx = write_str(&mut buf, 0, "State: ");
    idx = write_i32(&mut buf, idx, if check1 { 1 } else { 0 });
    idx = write_str(&mut buf, idx, " ");
    idx = write_i32(&mut buf, idx, if check2 { 1 } else { 0 });
    idx = write_str(&mut buf, idx, " ");
    idx = write_i32(&mut buf, idx, if check3 { 1 } else { 0 });
    api::imgui::ui_label(buf_to_str(&buf, idx), api::font::SECONDARY, api::align::CENTER);

    api::imgui::ui_end();
}

fn render_footer() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Footer Demo", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_spacer(8);

    let mut buf = [0u8; 32];
    let mut idx = write_str(&mut buf, 0, "Left: ");
    idx = write_i32(&mut buf, idx, LEFT_COUNT.get());
    idx = write_str(&mut buf, idx, "   Right: ");
    idx = write_i32(&mut buf, idx, RIGHT_COUNT.get());
    api::imgui::ui_label(buf_to_str(&buf, idx), api::font::SECONDARY, api::align::CENTER);

    api::imgui::ui_spacer(4);
    api::imgui::ui_label("Press </> to change", api::font::SECONDARY, api::align::CENTER);

    if api::imgui::ui_footer_left("Dec") {
        LEFT_COUNT.set(LEFT_COUNT.get() - 1);
    }

    api::imgui::ui_footer_center("OK");

    if api::imgui::ui_footer_right("Inc") {
        RIGHT_COUNT.set(RIGHT_COUNT.get() + 1);
    }

    api::imgui::ui_end();
}

fn keyboard_validator(text: &str, message: &mut [u8], _context: usize) -> bool {
    let len = text.as_bytes().len();
    if len < 3 {
        copy_str(message, "Min 3 chars");
        return false;
    }

    for byte in text.as_bytes() {
        if *byte == b' ' {
            copy_str(message, "No spaces");
            return false;
        }
    }

    true
}

fn render_keyboard() {
    api::imgui::ui_begin();

    let now_ms = api::get_time_ms();
    if KEYBOARD_SAVED.get() && now_ms >= KEYBOARD_SAVED_UNTIL.get() {
        KEYBOARD_SAVED.set(false);
    }

    let mut keyboard = KEYBOARD.get();
    api::imgui::ui_virtual_keyboard_set_validator(&mut keyboard, keyboard_validator, 0);

    let mut header_buf = [0u8; 32];
    let header_text = if KEYBOARD_SAVED.get() {
        let saved = KEYBOARD_SAVED_TEXT.get();
        let saved_text = cstr_to_str(&saved);
        let mut idx = write_str(&mut header_buf, 0, "Saved: ");
        idx = write_str(&mut header_buf, idx, saved_text);
        buf_to_str(&header_buf, idx)
    } else {
        "Enter Name"
    };

    if api::imgui::ui_virtual_keyboard(&mut keyboard, header_text, now_ms) {
        KEYBOARD_SAVED.set(true);
        KEYBOARD_SAVED_UNTIL.set(now_ms + 2000);
        let mut saved = KEYBOARD_SAVED_TEXT.get();
        copy_str(&mut saved, keyboard.text());
        KEYBOARD_SAVED_TEXT.set(saved);
    }

    KEYBOARD.set(keyboard);
    api::imgui::ui_end();
}

fn render_impl() {
    match CURRENT_SCENE.get() {
        Scene::Counter => render_counter(),
        Scene::Menu => render_menu(),
        Scene::Layout => render_layout(),
        Scene::Progress => render_progress(),
        Scene::Checkbox => render_checkbox(),
        Scene::Footer => render_footer(),
        Scene::Keyboard => render_keyboard(),
    }
}

fn on_input_impl(key: u32, kind: u32) {
    api::imgui::ui_input(key as u8, kind as u8);

    if key == api::input::KEY_BACK && kind == api::input::TYPE_LONG_PRESS {
        api::exit_to_launcher();
        return;
    }

    if key == api::input::KEY_BACK
        && (kind == api::input::TYPE_SHORT_PRESS || kind == api::input::TYPE_REPEAT)
        && api::imgui::ui_back_pressed()
    {
        let scene = CURRENT_SCENE.get() as u32;
        if scene == 0 {
            set_scene_impl(SCENE_COUNT - 1);
        } else {
            set_scene_impl(scene - 1);
        }
    }

    if kind == api::input::TYPE_PRESS {
        if CURRENT_SCENE.get() == Scene::Menu && api::imgui::ui_get_focus() == 0 {
            if key == api::input::KEY_LEFT && BRIGHTNESS.get() > 0 {
                BRIGHTNESS.set(BRIGHTNESS.get() - 1);
            }
            if key == api::input::KEY_RIGHT && BRIGHTNESS.get() < 10 {
                BRIGHTNESS.set(BRIGHTNESS.get() + 1);
            }
        }
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_get_scene!(get_scene_impl);
api::export_set_scene!(set_scene_impl);
api::export_get_scene_count!(get_scene_count_impl);
api::wasm_panic_handler!();
