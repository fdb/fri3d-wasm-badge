#![deny(unsafe_code)]

use fri3d_runtime::canvas::Canvas;
use fri3d_runtime::input::{InputBackend, InputEvent, InputManager};
use fri3d_runtime::random::Random;
use fri3d_runtime::types::{Color, Font, InputKey, InputType};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

const STRING_BUFFER_SIZE: usize = 256;

// Shared scratch buffer for JS to write null-terminated strings.
static mut STRING_BUFFER: [u8; STRING_BUFFER_SIZE] = [0; STRING_BUFFER_SIZE];

struct WebInputBackend {
    queue: VecDeque<InputEvent>,
}

impl WebInputBackend {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push_event(&mut self, key: InputKey, kind: InputType) {
        self.queue.push_back(InputEvent { key, kind });
    }
}

impl InputBackend for WebInputBackend {
    fn poll(&mut self) {}

    fn has_event(&self) -> bool {
        !self.queue.is_empty()
    }

    fn next_event(&mut self) -> Option<InputEvent> {
        self.queue.pop_front()
    }
}

thread_local! {
    static CANVAS: RefCell<Canvas> = RefCell::new(Canvas::new());
    static RANDOM: RefCell<Random> = RefCell::new(Random::new(0));
    static INPUT_MANAGER: RefCell<InputManager> = RefCell::new(InputManager::new());
    static INPUT_BACKEND: RefCell<WebInputBackend> = RefCell::new(WebInputBackend::new());
    static RESET_REQUESTED: Cell<bool> = Cell::new(false);
    static INIT_DONE: Cell<bool> = Cell::new(false);
}

fn ensure_init() {
    INIT_DONE.with(|done| {
        if done.get() {
            return;
        }
        INPUT_MANAGER.with(|manager| {
            manager.borrow_mut().set_reset_callback(|| {
                RESET_REQUESTED.with(|flag| flag.set(true));
            });
        });
        done.set(true);
    });
}

#[allow(unsafe_code)]
fn read_c_string(ptr: u32) -> String {
    if ptr == 0 {
        return String::new();
    }
    let mut len = 0usize;
    let base = ptr as *const u8;
    unsafe {
        while len < STRING_BUFFER_SIZE {
            let byte = *base.add(len);
            if byte == 0 {
                break;
            }
            len += 1;
        }
        let slice = std::slice::from_raw_parts(base, len);
        String::from_utf8_lossy(slice).to_string()
    }
}

#[allow(unsafe_code)]
mod exports {
use super::*;

#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn frd_string_buffer_ptr() -> u32 {
    core::ptr::addr_of_mut!(STRING_BUFFER) as u32
}

#[no_mangle]
pub extern "C" fn frd_canvas_clear() {
    CANVAS.with(|canvas| canvas.borrow_mut().clear());
}

#[no_mangle]
pub extern "C" fn frd_canvas_width() -> i32 {
    CANVAS.with(|canvas| canvas.borrow().width() as i32)
}

#[no_mangle]
pub extern "C" fn frd_canvas_height() -> i32 {
    CANVAS.with(|canvas| canvas.borrow().height() as i32)
}

#[no_mangle]
pub extern "C" fn frd_canvas_set_color(color: i32) {
    let value = match color {
        0 => Color::White,
        1 => Color::Black,
        _ => Color::Xor,
    };
    CANVAS.with(|canvas| canvas.borrow_mut().set_color(value));
}

#[no_mangle]
pub extern "C" fn frd_canvas_set_font(font: i32) {
    let value = match font {
        0 => Font::Primary,
        1 => Font::Secondary,
        2 => Font::Keyboard,
        _ => Font::BigNumbers,
    };
    CANVAS.with(|canvas| canvas.borrow_mut().set_font(value));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_dot(x: i32, y: i32) {
    CANVAS.with(|canvas| canvas.borrow_mut().draw_dot(x, y));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32) {
    CANVAS.with(|canvas| canvas.borrow_mut().draw_line(x1, y1, x2, y2));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_frame(x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    CANVAS.with(|canvas| canvas.borrow_mut().draw_frame(x, y, w as u32, h as u32));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_box(x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    CANVAS.with(|canvas| canvas.borrow_mut().draw_box(x, y, w as u32, h as u32));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_rframe(x: i32, y: i32, w: i32, h: i32, r: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    CANVAS.with(|canvas| {
        canvas
            .borrow_mut()
            .draw_rframe(x, y, w as u32, h as u32, r.max(0) as u32)
    });
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_rbox(x: i32, y: i32, w: i32, h: i32, r: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    CANVAS.with(|canvas| {
        canvas
            .borrow_mut()
            .draw_rbox(x, y, w as u32, h as u32, r.max(0) as u32)
    });
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_circle(x: i32, y: i32, r: i32) {
    if r <= 0 {
        return;
    }
    CANVAS.with(|canvas| canvas.borrow_mut().draw_circle(x, y, r as u32));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_disc(x: i32, y: i32, r: i32) {
    if r <= 0 {
        return;
    }
    CANVAS.with(|canvas| canvas.borrow_mut().draw_disc(x, y, r as u32));
}

#[no_mangle]
pub extern "C" fn frd_canvas_draw_str(x: i32, y: i32, ptr: u32) {
    let text = read_c_string(ptr);
    CANVAS.with(|canvas| canvas.borrow_mut().draw_str(x, y, &text));
}

#[no_mangle]
pub extern "C" fn frd_canvas_string_width(ptr: u32) -> i32 {
    let text = read_c_string(ptr);
    CANVAS.with(|canvas| canvas.borrow().string_width(&text) as i32)
}

#[no_mangle]
pub extern "C" fn frd_random_seed(seed: i32) {
    RANDOM.with(|random| random.borrow_mut().seed(seed as u32));
}

#[no_mangle]
pub extern "C" fn frd_random_get() -> i32 {
    RANDOM.with(|random| random.borrow_mut().get() as i32)
}

#[no_mangle]
pub extern "C" fn frd_random_range(max: i32) -> i32 {
    if max <= 0 {
        return 0;
    }
    RANDOM.with(|random| random.borrow_mut().range(max as u32) as i32)
}

#[no_mangle]
pub extern "C" fn frd_input_push_event(key: i32, kind: i32) {
    ensure_init();
    let key = match key {
        0 => InputKey::Up,
        1 => InputKey::Down,
        2 => InputKey::Left,
        3 => InputKey::Right,
        4 => InputKey::Ok,
        5 => InputKey::Back,
        _ => return,
    };
    let kind = match kind {
        0 => InputType::Press,
        1 => InputType::Release,
        _ => return,
    };
    INPUT_BACKEND.with(|backend| backend.borrow_mut().push_event(key, kind));
}

#[no_mangle]
pub extern "C" fn frd_input_update(time_ms: u32) {
    ensure_init();
    INPUT_BACKEND.with(|backend| {
        let mut backend = backend.borrow_mut();
        INPUT_MANAGER.with(|manager| manager.borrow_mut().update(&mut *backend, time_ms));
    });
}

#[no_mangle]
pub extern "C" fn frd_input_has_event() -> i32 {
    INPUT_MANAGER.with(|manager| if manager.borrow().has_event() { 1 } else { 0 })
}

#[no_mangle]
pub extern "C" fn frd_input_next_event() -> i32 {
    INPUT_MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        let Some(event) = manager.next_event() else {
            return -1;
        };
        ((event.key as i32) << 8) | (event.kind as i32)
    })
}

#[no_mangle]
pub extern "C" fn frd_input_take_reset_request() -> i32 {
    RESET_REQUESTED.with(|flag| {
        if flag.get() {
            flag.set(false);
            1
        } else {
            0
        }
    })
}

#[no_mangle]
pub extern "C" fn frd_framebuffer_ptr() -> u32 {
    CANVAS.with(|canvas| canvas.borrow().buffer().as_ptr() as u32)
}

#[no_mangle]
pub extern "C" fn frd_framebuffer_len() -> u32 {
    CANVAS.with(|canvas| canvas.borrow().buffer().len() as u32)
}
}
