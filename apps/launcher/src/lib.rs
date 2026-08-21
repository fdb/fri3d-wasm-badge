//! Launcher — the home screen.
//!
//! A green banner, then a 4×2 grid of app cells: white card, tan rest
//! border, gold border on the focused cell, the 16×16 icon drawn at 2×
//! and the name below. Pages of eight; the banner shows the page. Data
//! comes from the kernel registry (`app_count` / `AppInfo`), so installing
//! an app needs no launcher change. Design doc 017 has the tokens.
//!
//! Zero timers: the launcher only renders on input, so an idle badge does
//! no work at all.
#![no_std]
#![deny(unsafe_code)]

use fri3d_artwork as art;
use fri3d_wasm_api as api;
use fri3d_wasm_api::{color, font, imgui, input, wifi};

const W: i32 = api::SCREEN_WIDTH as i32;
const H: i32 = api::SCREEN_HEIGHT as i32;
const BANNER_H: i32 = 24;
const PAD: i32 = 8;
const COLS: u32 = 4;
const ROWS: u32 = 2;
const PAGE: u32 = COLS * ROWS;
const CELL_W: i32 = (W - 2 * PAD - (COLS as i32 - 1) * PAD) / COLS as i32;
const CELL_H: i32 = 62;
const GRID_Y: i32 = BANNER_H + PAD + 14;
const FOOTER_H: i32 = 18;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Screen {
    Menu,
    Info,
}

#[derive(Copy, Clone)]
struct State {
    screen: Screen,
    selected: u32,
    count: u32,
}

static STATE: api::AppCell<State> = api::AppCell::new(State {
    screen: Screen::Menu,
    selected: 0,
    count: 0,
});

fn refresh() {
    let mut s = STATE.get();
    s.count = api::app_count();
    s.selected = if s.count == 0 { 0 } else { s.selected.min(s.count - 1) };
    STATE.set(s);
}

fn render_impl() {
    let s = STATE.get();
    match s.screen {
        Screen::Menu => render_menu(&s),
        Screen::Info => render_info(&s),
    }
}

fn wifi_mark() -> Option<&'static api::Image> {
    match wifi::status() {
        wifi::STATUS_CONNECTED => Some(&art::WIFI),
        wifi::STATUS_OFF => None,
        _ => Some(&art::WIFI_OFF),
    }
}

fn render_menu(s: &State) {
    imgui::ui_begin();
    imgui::ui_banner("Fri3d", "", wifi_mark());
    imgui::ui_footer_left("Menu");
    imgui::ui_footer_right("Open");
    imgui::ui_end();

    if s.count == 0 {
        api::canvas_set_color(color::INK);
        api::canvas_set_font(font::PRIMARY);
        let msg = "No apps installed";
        let w = api::canvas_string_width(msg) as i32;
        api::canvas_draw_str((W - w) / 2, H / 2, msg);
        return;
    }

    // Section label and page counter.
    let page = s.selected / PAGE;
    let pages = s.count.div_ceil(PAGE);
    api::canvas_set_color(color::MUTED);
    api::canvas_set_font(font::PRIMARY);
    api::canvas_draw_str(PAD, BANNER_H + PAD + 8, "APPS");
    let mut pager = Num::new();
    pager.push_num(page + 1);
    pager.push_str("/");
    pager.push_num(pages);
    let pw = api::canvas_string_width(pager.as_str()) as i32;
    api::canvas_draw_str(W - PAD - pw, BANNER_H + PAD + 8, pager.as_str());

    let mut info = api::AppInfo::empty();
    let first = page * PAGE;
    let last = (first + PAGE).min(s.count);
    for idx in first..last {
        let slot = idx - first;
        let x = PAD + (slot % COLS) as i32 * (CELL_W + PAD);
        let y = GRID_Y + (slot / COLS) as i32 * (CELL_H + PAD);
        if !info.fetch(idx) {
            continue;
        }
        draw_cell(x, y, &info, idx == s.selected);
    }
}

fn draw_cell(x: i32, y: i32, info: &api::AppInfo, focused: bool) {
    api::canvas_set_color(color::CARD);
    api::canvas_draw_rbox(x, y, CELL_W as u32, CELL_H as u32, 2);
    api::canvas_set_color(if focused { color::FOCUS } else { color::REST_BORDER });
    api::canvas_draw_rframe(x, y, CELL_W as u32, CELL_H as u32, 2);
    api::canvas_draw_rframe(x + 1, y + 1, CELL_W as u32 - 2, CELL_H as u32 - 2, 2);

    let icon = api::AppInfo::ICON_W as i32 * 2;
    info.draw_icon(x + (CELL_W - icon) / 2, y + 6, 2);

    api::canvas_set_color(color::INK);
    api::canvas_set_font(font::PRIMARY);
    let name = info.name();
    let w = api::canvas_string_width(name) as i32;
    api::canvas_draw_str(x + (CELL_W - w) / 2, y + CELL_H - 6, name);
}

fn render_info(s: &State) {
    let mut info = api::AppInfo::empty();
    if !info.fetch(s.selected) {
        render_menu(s);
        return;
    }
    imgui::ui_begin();
    imgui::ui_banner(info.name(), info.version(), None);
    imgui::ui_footer_left("Back");
    imgui::ui_footer_right("Open");
    imgui::ui_end();

    // Icon card and the name block beside it.
    let card = 56;
    let y = BANNER_H + PAD;
    api::canvas_set_color(color::CARD);
    api::canvas_draw_rbox(PAD, y, card, card, 2);
    api::canvas_set_color(color::INK);
    api::canvas_draw_rframe(PAD, y, card, card, 2);
    api::canvas_draw_rframe(PAD + 1, y + 1, card - 2, card - 2, 2);
    info.draw_icon(PAD + 4, y + 4, 3);

    let tx = PAD + card as i32 + PAD;
    api::canvas_set_font(font::TITLE);
    api::canvas_draw_str(tx, y + 18, info.name());
    api::canvas_set_font(font::PRIMARY);
    api::canvas_set_color(color::MUTED);
    let mut by = Num::new();
    by.push_str("by ");
    by.push_str(info.author());
    api::canvas_draw_str(tx, y + 36, by.as_str());
    let mut id = Num::new();
    id.push_str("id: ");
    id.push_str(info.id());
    api::canvas_draw_str(tx, y + 50, id.as_str());

    // Description panel.
    let py = y + card as i32 + PAD;
    let ph = H - FOOTER_H - PAD - py;
    api::canvas_set_color(color::CARD);
    api::canvas_draw_rbox(PAD, py, (W - 2 * PAD) as u32, ph as u32, 2);
    api::canvas_set_color(color::INK);
    api::canvas_draw_rframe(PAD, py, (W - 2 * PAD) as u32, ph as u32, 2);
    api::canvas_draw_rframe(PAD + 1, py + 1, (W - 2 * PAD) as u32 - 2, ph as u32 - 2, 2);
    draw_wrapped(2 * PAD, py + 18, W - 4 * PAD, 14, (ph - 12) / 14, info.description());
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
            let n = s.count;
            match (key, kind) {
                (input::KEY_LEFT, _) if activate => s.selected = (s.selected + n - 1) % n,
                (input::KEY_RIGHT, _) if activate => s.selected = (s.selected + 1) % n,
                (input::KEY_UP, _) if activate => s.selected = (s.selected + n - COLS.min(n)) % n,
                (input::KEY_DOWN, _) if activate => s.selected = (s.selected + COLS.min(n)) % n,
                (input::KEY_OK, input::TYPE_SHORT_PRESS) => {
                    STATE.set(s);
                    api::start_app(s.selected);
                    return;
                }
                (input::KEY_OK, input::TYPE_LONG_PRESS) => s.screen = Screen::Info,
                _ => return,
            }
        }
        Screen::Info => match (key, kind) {
            (input::KEY_BACK, input::TYPE_SHORT_PRESS) | (input::KEY_LEFT, input::TYPE_SHORT_PRESS) => {
                s.screen = Screen::Menu;
            }
            (input::KEY_OK, input::TYPE_SHORT_PRESS) | (input::KEY_RIGHT, input::TYPE_SHORT_PRESS) => {
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

/// Tiny fixed-capacity string for "1/2"-style labels. No allocation.
struct Num {
    buf: [u8; 48],
    len: usize,
}

impl Num {
    const fn new() -> Self {
        Self { buf: [0; 48], len: 0 }
    }
    fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            if self.len < self.buf.len() {
                self.buf[self.len] = b;
                self.len += 1;
            }
        }
    }
    fn push_num(&mut self, mut v: u32) {
        let mut digits = [0u8; 10];
        let mut n = 0;
        loop {
            digits[n] = b'0' + (v % 10) as u8;
            n += 1;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        while n > 0 {
            n -= 1;
            if self.len < self.buf.len() {
                self.buf[self.len] = digits[n];
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
