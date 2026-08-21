//! Launcher — the home screen.
//!
//! Flipper-Zero-style list: a status bar, three rows of 14x14 icon + name,
//! the focused row as an inverted rounded box, a dotted scrollbar. Data
//! comes from the kernel registry (`app_count` / `AppInfo`), so installing
//! an app needs no launcher change.
//!
//! Zero timers: the launcher only renders on input, so an idle badge does
//! no work at all.
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{color, font, input, wifi};

const W: i32 = api::SCREEN_WIDTH as i32;
const H: i32 = api::SCREEN_HEIGHT as i32;
const BAR_H: i32 = 11;
const ROW_H: i32 = 16;
const ROWS: u32 = ((H - LIST_Y) / ROW_H) as u32;
const LIST_Y: i32 = BAR_H + 2;
const LIST_W: i32 = W - 5;
const ICON: u32 = api::AppInfo::ICON_W;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Screen {
    Menu,
    Info,
}

#[derive(Copy, Clone)]
struct State {
    screen: Screen,
    selected: u32,
    scroll: u32,
    count: u32,
}

static STATE: api::AppCell<State> = api::AppCell::new(State {
    screen: Screen::Menu,
    selected: 0,
    scroll: 0,
    count: 0,
});

fn refresh() {
    let mut s = STATE.get();
    s.count = api::app_count();
    if s.count == 0 {
        s.selected = 0;
        s.scroll = 0;
    } else {
        s.selected = s.selected.min(s.count - 1);
        s.scroll = s.scroll.min(s.selected);
    }
    STATE.set(s);
}

fn render_impl() {
    let s = STATE.get();
    match s.screen {
        Screen::Menu => render_menu(&s),
        Screen::Info => render_info(&s),
    }
}

/// 9x7 Wi-Fi mark, MSB-first rows (the `canvas_draw_bitmap` convention).
const WIFI_ICON_W: u32 = 9;
const WIFI_ICON_H: u32 = 7;
const WIFI_ICON: [u8; 14] = [
    0b00111110, 0b00000000, //   #####
    0b01000001, 0b00000000, //  #     #
    0b10011100, 0b10000000, // #  ###  #
    0b00100010, 0b00000000, //   #   #
    0b00001000, 0b00000000, //     #
    0b00000000, 0b00000000, //
    0b00001000, 0b00000000, //     #
];

/// Status bar: the title on the left, a Wi-Fi mark in the top-right
/// corner while connected, `right` just before it, a separator below.
fn render_bar(title: &str, right: &str) {
    api::canvas_set_color(color::BLACK);
    api::canvas_set_font(font::SECONDARY);
    api::canvas_draw_str(2, 8, title);
    let mut edge = W - 2;
    if wifi::status() == wifi::STATUS_CONNECTED {
        edge -= WIFI_ICON_W as i32;
        api::canvas_draw_bitmap(edge, 1, WIFI_ICON_W, WIFI_ICON_H, &WIFI_ICON);
        edge -= 3;
    }
    let rw = api::canvas_string_width(right) as i32;
    api::canvas_draw_str(edge - rw, 8, right);
    api::canvas_draw_line(0, BAR_H - 1, W - 1, BAR_H - 1);
}

fn render_menu(s: &State) {
    render_bar("Fri3d", "");

    if s.count == 0 {
        api::canvas_set_font(font::PRIMARY);
        api::canvas_draw_str(W / 2 - 40, H / 2, "No apps installed");
        return;
    }

    let mut info = api::AppInfo::empty();
    let last = (s.scroll + ROWS).min(s.count);
    for (row, idx) in (s.scroll..last).enumerate() {
        let y = LIST_Y + row as i32 * ROW_H;
        let focused = idx == s.selected;
        if !info.fetch(idx) {
            continue;
        }
        api::canvas_set_color(color::BLACK);
        if focused {
            api::canvas_draw_rbox(0, y, LIST_W as u32, ROW_H as u32, 3);
            api::canvas_set_color(color::WHITE);
        }
        api::canvas_draw_bitmap(3, y + 1, ICON, ICON, info.icon());
        api::canvas_set_font(font::PRIMARY);
        api::canvas_draw_str(3 + ICON as i32 + 5, y + 12, info.name());
    }

    render_scrollbar(s.scroll, ROWS, s.count);
}

/// Flipper-style: a dotted track with a solid thumb.
fn render_scrollbar(scroll: u32, visible: u32, total: u32) {
    let x = W - 2;
    let y0 = LIST_Y;
    let track = ROWS as i32 * ROW_H;
    api::canvas_set_color(color::BLACK);
    let mut y = y0;
    while y < y0 + track {
        api::canvas_draw_dot(x, y);
        y += 2;
    }
    if total <= visible {
        api::canvas_draw_box(x as i32 - 1, y0, 3, track as u32);
        return;
    }
    let thumb_h = ((track as u32 * visible) / total).max(4) as i32;
    let max_scroll = total - visible;
    let thumb_y = y0 + ((track - thumb_h) * scroll as i32) / max_scroll as i32;
    api::canvas_draw_box(x - 1, thumb_y, 3, thumb_h as u32);
}

fn render_info(s: &State) {
    let mut info = api::AppInfo::empty();
    if !info.fetch(s.selected) {
        render_bar("Fri3d", "");
        return;
    }
    // ui_begin clears the canvas, so the footer goes first.
    api::imgui::ui_begin();
    api::imgui::ui_footer_left("Back");
    api::imgui::ui_footer_right("Open");
    api::imgui::ui_end();
    render_bar(info.name(), info.version());
    api::canvas_set_color(color::BLACK);
    api::canvas_draw_bitmap(2, BAR_H + 3, ICON, ICON, info.icon());
    api::canvas_set_font(font::SECONDARY);
    api::canvas_draw_str(ICON as i32 + 6, BAR_H + 10, info.author());
    let mut ver = Num::new();
    ver.push_str("id: ");
    ver.push_str(info.id());
    api::canvas_draw_str(ICON as i32 + 6, BAR_H + 18, ver.as_str());
    draw_wrapped(2, BAR_H + 29, W - 4, 8, (H - BAR_H - 29 - 12) / 8, info.description());
}

/// Greedy word wrap with the current font. Bounded to `max_lines`.
fn draw_wrapped(x: i32, y: i32, max_w: i32, line_h: i32, max_lines: i32, text: &str) {
    let mut line_start = 0usize;
    let mut line_end = 0usize;
    let mut lines = 0;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i <= bytes.len() && lines < max_lines {
        let at_break = i == bytes.len() || bytes[i] == b' ';
        if at_break {
            let candidate = &text[line_start..i];
            if api::canvas_string_width(candidate) as i32 <= max_w || line_end == line_start {
                line_end = i;
            } else {
                api::canvas_draw_str(x, y + lines * line_h, &text[line_start..line_end]);
                lines += 1;
                line_start = line_end + 1;
                line_end = i;
            }
        }
        i += 1;
    }
    if lines < max_lines && line_start < bytes.len() {
        api::canvas_draw_str(x, y + lines * line_h, &text[line_start..line_end.max(line_start)]);
    }
}

fn on_input_impl(key: u32, kind: u32) {
    let mut s = STATE.get();
    let activate = kind == input::TYPE_SHORT_PRESS || kind == input::TYPE_REPEAT;
    match s.screen {
        Screen::Menu => {
            if s.count == 0 {
                return;
            }
            match (key, kind) {
                (input::KEY_UP, _) if activate => {
                    s.selected = if s.selected == 0 { s.count - 1 } else { s.selected - 1 };
                }
                (input::KEY_DOWN, _) if activate => {
                    s.selected = if s.selected + 1 == s.count { 0 } else { s.selected + 1 };
                }
                (input::KEY_OK, input::TYPE_SHORT_PRESS) => {
                    STATE.set(s);
                    api::start_app(s.selected);
                    return;
                }
                (input::KEY_OK, input::TYPE_LONG_PRESS) | (input::KEY_RIGHT, input::TYPE_SHORT_PRESS) => {
                    s.screen = Screen::Info;
                }
                _ => return,
            }
            if s.selected < s.scroll {
                s.scroll = s.selected;
            } else if s.selected >= s.scroll + ROWS {
                s.scroll = s.selected + 1 - ROWS;
            }
        }
        Screen::Info => match (key, kind) {
            (input::KEY_BACK, input::TYPE_SHORT_PRESS) | (input::KEY_LEFT, input::TYPE_SHORT_PRESS) => {
                s.screen = Screen::Menu;
            }
            (input::KEY_OK, input::TYPE_SHORT_PRESS) => {
                s.screen = Screen::Menu;
                STATE.set(s);
                api::start_app(s.selected);
                return;
            }
            _ => return,
        },
    }
    STATE.set(s);
}

fn on_resume_impl() {
    // Back from an app: keep the cursor on it, show the menu.
    refresh();
    let mut s = STATE.get();
    s.screen = Screen::Menu;
    STATE.set(s);
}

/// Tiny fixed-capacity string for "3/8"-style labels. No allocation.
struct Num {
    buf: [u8; 32],
    len: usize,
}

impl Num {
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
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_on_start!(refresh);
api::export_on_resume!(on_resume_impl);
api::wasm_panic_handler!();
