//! Settings — brightness, sound, about. A system app: it may write the
//! `system` settings namespace, which hosts read (brightness drives the
//! LCD backlight on the badge, the amber tint on desktop and web).
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{align, color, font, imgui, input};

const NS: &str = "system";
const KEY_BRIGHTNESS: &str = "brightness";
const KEY_SOUND: &str = "sound";

const ITEM_BRIGHTNESS: i16 = 0;
const ITEM_SOUND: i16 = 1;
const ITEM_ABOUT: i16 = 2;
const ITEM_COUNT: i16 = 3;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Screen {
    Menu,
    About,
}

#[derive(Copy, Clone)]
struct State {
    screen: Screen,
    scroll: i16,
    brightness: u32,
    sound: bool,
}

static STATE: api::AppCell<State> = api::AppCell::new(State {
    screen: Screen::Menu,
    scroll: 0,
    brightness: 100,
    sound: true,
});

fn on_start_impl() {
    let mut s = STATE.get();
    s.brightness = api::settings_get_u32(NS, KEY_BRIGHTNESS, 100).clamp(10, 100);
    s.sound = api::settings_get_u32(NS, KEY_SOUND, 1) != 0;
    STATE.set(s);
}

fn render_impl() {
    let mut s = STATE.get();
    imgui::ui_begin();
    match s.screen {
        Screen::Menu => render_menu(&mut s),
        Screen::About => render_about(),
    }
    imgui::ui_end();
    STATE.set(s);
}

fn render_menu(s: &mut State) {
    imgui::ui_label("Settings", font::PRIMARY, align::CENTER);
    imgui::ui_separator();

    let mut pct = Small::new();
    pct.push(s.brightness);
    pct.push_str("%");

    imgui::ui_menu_begin(&mut s.scroll, 4, ITEM_COUNT);
    imgui::ui_menu_item_value("Brightness", pct.as_str(), ITEM_BRIGHTNESS);
    imgui::ui_menu_item_value("Sound", if s.sound { "On" } else { "Off" }, ITEM_SOUND);
    if imgui::ui_menu_item("About", ITEM_ABOUT) {
        s.screen = Screen::About;
    }
    imgui::ui_menu_end();
}

fn render_about() {
    imgui::ui_label("About", font::PRIMARY, align::CENTER);
    imgui::ui_separator();
    api::canvas_set_color(color::BLACK);
    api::canvas_set_font(font::SECONDARY);
    let mut line = Small::new();
    line.push_str("Fri3d WASM badge");
    api::canvas_draw_str(2, 26, line.as_str());
    let mut line = Small::new();
    line.push_str("kernel ABI v");
    line.push(api::kernel_version());
    api::canvas_draw_str(2, 36, line.as_str());
    let mut line = Small::new();
    line.push(api::app_count());
    line.push_str(" apps installed");
    api::canvas_draw_str(2, 46, line.as_str());
    imgui::ui_footer_left("Back");
}

fn on_input_impl(key: u32, kind: u32) {
    let mut s = STATE.get();
    let short = kind == input::TYPE_SHORT_PRESS;
    let step = kind == input::TYPE_SHORT_PRESS || kind == input::TYPE_REPEAT;
    match s.screen {
        Screen::About => {
            if (key == input::KEY_BACK || key == input::KEY_LEFT) && short {
                s.screen = Screen::Menu;
                STATE.set(s);
                api::request_render();
                return;
            }
        }
        Screen::Menu => {
            if key == input::KEY_BACK && short {
                api::exit_to_launcher();
                return;
            }
            let focus = imgui::ui_get_focus();
            match (focus, key) {
                (ITEM_BRIGHTNESS, input::KEY_LEFT) if step => {
                    s.brightness = s.brightness.saturating_sub(10).max(10);
                    api::settings_set_u32(NS, KEY_BRIGHTNESS, s.brightness);
                }
                (ITEM_BRIGHTNESS, input::KEY_RIGHT) if step => {
                    s.brightness = (s.brightness + 10).min(100);
                    api::settings_set_u32(NS, KEY_BRIGHTNESS, s.brightness);
                }
                (ITEM_SOUND, input::KEY_LEFT) | (ITEM_SOUND, input::KEY_RIGHT) | (ITEM_SOUND, input::KEY_OK)
                    if short =>
                {
                    s.sound = !s.sound;
                    api::settings_set_u32(NS, KEY_SOUND, s.sound as u32);
                }
                _ => {}
            }
        }
    }
    STATE.set(s);
    imgui::ui_input(key as u8, kind as u8);
}

/// Fixed-capacity label builder. No allocation.
struct Small {
    buf: [u8; 32],
    len: usize,
}

impl Small {
    const fn new() -> Self {
        Self { buf: [0; 32], len: 0 }
    }
    fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            if self.len < self.buf.len() {
                self.buf[self.len] = b;
                self.len += 1;
            }
        }
    }
    fn push(&mut self, mut n: u32) {
        let mut digits = [0u8; 10];
        let mut i = 0;
        loop {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
            if n == 0 {
                break;
            }
        }
        while i > 0 {
            i -= 1;
            if self.len < self.buf.len() {
                self.buf[self.len] = digits[i];
                self.len += 1;
            }
        }
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_on_start!(on_start_impl);
api::wasm_panic_handler!();
