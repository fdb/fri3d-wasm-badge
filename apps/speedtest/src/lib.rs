//! Speed test: reach two public resolvers over TCP, then download a
//! 1 MB file and report the throughput. The body is counted by the
//! host and never stored. Needs Wi-Fi (or a desktop with a network).
#![no_std]
#![deny(unsafe_code)]

use fri3d_wasm_api as api;
use fri3d_wasm_api::{align, color, font, imgui, input, net, wifi};

const URL: &str = "http://speedtest.tele2.net/1MB.zip";
const STEPS: usize = 3;
const LINE_Y0: i32 = 30;
const LINE_H: i32 = 14;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Result {
    Pending,
    Running,
    Ok { ms: u32, bytes: u32 },
    Failed,
}

#[derive(Copy, Clone)]
struct State {
    running: bool,
    step: usize,
    results: [Result; STEPS],
}

static STATE: api::AppCell<State> = api::AppCell::new(State {
    running: false,
    step: 0,
    results: [Result::Pending; STEPS],
});

fn start_step(step: usize) -> bool {
    match step {
        0 => net::probe([1, 1, 1, 1], 53),
        1 => net::probe([8, 8, 8, 8], 53),
        _ => net::download(URL),
    }
}

fn begin() {
    let mut s = STATE.get();
    s.results = [Result::Pending; STEPS];
    s.step = 0;
    s.running = start_step(0);
    s.results[0] = if s.running { Result::Running } else { Result::Failed };
    STATE.set(s);
    api::start_timer_ms(250);
}

/// Advance the sequence from the host's progress. Called from render,
/// which the kernel triggers on every network change.
fn advance() {
    let mut s = STATE.get();
    if !s.running {
        return;
    }
    match net::status() {
        net::STATUS_DONE => {
            s.results[s.step] = Result::Ok { ms: net::elapsed_ms(), bytes: net::bytes() };
            s.step += 1;
            if s.step < STEPS && start_step(s.step) {
                s.results[s.step] = Result::Running;
            } else {
                if s.step < STEPS {
                    s.results[s.step] = Result::Failed;
                }
                s.running = false;
                api::stop_timer();
            }
        }
        net::STATUS_FAILED => {
            s.results[s.step] = Result::Failed;
            s.running = false;
            api::stop_timer();
        }
        _ => {}
    }
    STATE.set(s);
}

fn render_impl() {
    advance();
    let s = STATE.get();
    imgui::ui_begin();
    imgui::ui_label("Speed Test", font::PRIMARY, align::CENTER);
    imgui::ui_separator();
    imgui::ui_end();

    api::canvas_set_color(color::BLACK);
    api::canvas_set_font(font::SECONDARY);
    let labels = ["DNS 1.1.1.1", "DNS 8.8.8.8", "1 MB download"];
    for (i, label) in labels.iter().enumerate() {
        let y = LINE_Y0 + i as i32 * LINE_H;
        api::canvas_draw_str(4, y, label);
        let mut v = Text::new();
        match s.results[i] {
            Result::Pending => v.push_str("-"),
            Result::Running if i == 2 => {
                push_kb(&mut v, net::bytes());
                v.push_str(" ");
                push_rate(&mut v, net::bytes(), net::elapsed_ms());
            }
            Result::Running => {
                v.push(net::elapsed_ms());
                v.push_str(" ms...");
            }
            Result::Ok { ms, bytes } if i == 2 => {
                push_rate(&mut v, bytes, ms);
                v.push_str(" ");
                v.push(ms / 1000);
                v.push_str(".");
                v.push((ms % 1000) / 100);
                v.push_str(" s");
            }
            Result::Ok { ms, .. } => {
                v.push(ms);
                v.push_str(" ms");
            }
            Result::Failed => v.push_str("failed"),
        }
        let w = api::canvas_string_width(v.as_str()) as i32;
        api::canvas_draw_str(api::SCREEN_WIDTH as i32 - 4 - w, y, v.as_str());
    }

    let hint = if s.running {
        "running..."
    } else if wifi::status() != wifi::STATUS_CONNECTED {
        "Wi-Fi not connected"
    } else {
        "OK: start   Back: exit"
    };
    api::canvas_draw_str(4, api::SCREEN_HEIGHT as i32 - 4, hint);
}

fn on_input_impl(key: u32, kind: u32) {
    if kind != input::TYPE_SHORT_PRESS {
        return;
    }
    let s = STATE.get();
    match key {
        input::KEY_OK if !s.running => begin(),
        input::KEY_BACK => {
            if s.running {
                net::cancel();
            }
            api::exit_to_launcher();
        }
        _ => {}
    }
}

fn on_stop_impl() {
    api::stop_timer();
}

/// "123 KB" or "4.5 MB".
fn push_kb(t: &mut Text, bytes: u32) {
    if bytes >= 1024 * 1024 {
        let tenths = bytes as u64 * 10 / (1024 * 1024);
        t.push((tenths / 10) as u32);
        t.push_str(".");
        t.push((tenths % 10) as u32);
        t.push_str(" MB");
    } else {
        t.push(bytes / 1024);
        t.push_str(" KB");
    }
}

/// "850 KB/s" or "1.3 MB/s".
fn push_rate(t: &mut Text, bytes: u32, ms: u32) {
    let per_s = bytes as u64 * 1000 / ms.max(1) as u64;
    push_kb(t, per_s.min(u32::MAX as u64) as u32);
    t.push_str("/s");
}

struct Text {
    buf: [u8; 40],
    len: usize,
}

impl Text {
    const fn new() -> Self {
        Self { buf: [0; 40], len: 0 }
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
            self.push_str(core::str::from_utf8(&[digits[i]]).unwrap_or(""));
        }
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

api::export_render!(render_impl);
api::export_on_input!(on_input_impl);
api::export_on_stop!(on_stop_impl);
api::wasm_panic_handler!();
