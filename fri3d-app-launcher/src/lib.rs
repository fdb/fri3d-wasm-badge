#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;

struct LauncherEntry {
    name: &'static str,
    id: u32,
}

const APPS: [LauncherEntry; 6] = [
    LauncherEntry { name: "Circles", id: 1 },
    LauncherEntry { name: "Mandelbrot", id: 2 },
    LauncherEntry { name: "Test Drawing", id: 3 },
    LauncherEntry { name: "Test UI", id: 4 },
    LauncherEntry { name: "Snake", id: 5 },
    LauncherEntry { name: "WiFi Pet", id: 6 },
];

static MENU_SCROLL: api::AppCell<i16> = api::AppCell::new(0);

fn render_impl() {
    api::imgui::ui_begin();

    api::imgui::ui_label("Fri3d Apps", api::font::PRIMARY, api::align::CENTER);
    api::imgui::ui_separator();

    let mut scroll = MENU_SCROLL.get();
    api::imgui::ui_menu_begin(&mut scroll, 4, APPS.len() as i16);
    for (idx, entry) in APPS.iter().enumerate() {
        if api::imgui::ui_menu_item(entry.name, idx as i16) {
            api::start_app(entry.id);
        }
    }
    api::imgui::ui_menu_end();
    MENU_SCROLL.set(scroll);

    api::imgui::ui_end();
}

fn on_input_impl(key: u32, kind: u32) {
    api::imgui::ui_input(key as u8, kind as u8);
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::wasm_panic_handler!();
