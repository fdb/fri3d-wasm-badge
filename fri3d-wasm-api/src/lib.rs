#![no_std]

#[cfg(target_arch = "wasm32")]
mod bindings {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn canvas_clear();
        pub fn canvas_width() -> i32;
        pub fn canvas_height() -> i32;
        pub fn canvas_set_color(color: i32);
        pub fn canvas_set_font(font: i32);
        pub fn canvas_draw_dot(x: i32, y: i32);
        pub fn canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32);
        pub fn canvas_draw_frame(x: i32, y: i32, w: i32, h: i32);
        pub fn canvas_draw_box(x: i32, y: i32, w: i32, h: i32);
        pub fn canvas_draw_rframe(x: i32, y: i32, w: i32, h: i32, radius: i32);
        pub fn canvas_draw_rbox(x: i32, y: i32, w: i32, h: i32, radius: i32);
        pub fn canvas_draw_circle(x: i32, y: i32, radius: i32);
        pub fn canvas_draw_disc(x: i32, y: i32, radius: i32);
        pub fn canvas_draw_str(x: i32, y: i32, text: *const u8);
        pub fn canvas_string_width(text: *const u8) -> i32;
        pub fn random_seed(seed: i32);
        pub fn random_get() -> i32;
        pub fn random_range(max: i32) -> i32;
        pub fn get_time_ms() -> i32;
        pub fn request_render();
        pub fn exit_to_launcher();
        pub fn start_app(app_id: i32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod bindings {
    pub fn canvas_clear() {}

    pub fn canvas_width() -> i32 {
        0
    }

    pub fn canvas_height() -> i32 {
        0
    }

    pub fn canvas_set_color(_color: i32) {}

    pub fn canvas_set_font(_font: i32) {}

    pub fn canvas_draw_dot(_x: i32, _y: i32) {}

    pub fn canvas_draw_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32) {}

    pub fn canvas_draw_frame(_x: i32, _y: i32, _w: i32, _h: i32) {}

    pub fn canvas_draw_box(_x: i32, _y: i32, _w: i32, _h: i32) {}

    pub fn canvas_draw_rframe(_x: i32, _y: i32, _w: i32, _h: i32, _radius: i32) {}

    pub fn canvas_draw_rbox(_x: i32, _y: i32, _w: i32, _h: i32, _radius: i32) {}

    pub fn canvas_draw_circle(_x: i32, _y: i32, _radius: i32) {}

    pub fn canvas_draw_disc(_x: i32, _y: i32, _radius: i32) {}

    pub fn canvas_draw_str(_x: i32, _y: i32, _text: *const u8) {}

    pub fn canvas_string_width(_text: *const u8) -> i32 {
        0
    }

    pub fn random_seed(_seed: i32) {}

    pub fn random_get() -> i32 {
        0
    }

    pub fn random_range(_max: i32) -> i32 {
        0
    }

    pub fn get_time_ms() -> i32 {
        0
    }

    pub fn request_render() {}

    pub fn exit_to_launcher() {}

    pub fn start_app(_app_id: i32) {}
}

const STR_BUFFER_SIZE: usize = 256;

pub fn canvas_clear() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_clear();
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_clear();
    }
}

pub fn canvas_width() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::canvas_width().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_width().max(0) as u32
    }
}

pub fn canvas_height() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::canvas_height().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_height().max(0) as u32
    }
}

pub fn canvas_set_color(color: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_set_color(color as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_set_color(color as i32);
    }
}

pub fn canvas_set_font(font: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_set_font(font as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_set_font(font as i32);
    }
}

pub fn canvas_draw_dot(x: i32, y: i32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_dot(x, y);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_dot(x, y);
    }
}

pub fn canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_line(x1, y1, x2, y2);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_line(x1, y1, x2, y2);
    }
}

pub fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_frame(x, y, w as i32, h as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_frame(x, y, w as i32, h as i32);
    }
}

pub fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_box(x, y, w as i32, h as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_box(x, y, w as i32, h as i32);
    }
}

pub fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, radius: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_rframe(x, y, w as i32, h as i32, radius as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_rframe(x, y, w as i32, h as i32, radius as i32);
    }
}

pub fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, radius: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_rbox(x, y, w as i32, h as i32, radius as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_rbox(x, y, w as i32, h as i32, radius as i32);
    }
}

pub fn canvas_draw_circle(x: i32, y: i32, radius: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_circle(x, y, radius as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_circle(x, y, radius as i32);
    }
}

pub fn canvas_draw_disc(x: i32, y: i32, radius: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_disc(x, y, radius as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_disc(x, y, radius as i32);
    }
}

pub fn canvas_draw_str(x: i32, y: i32, text: &str) {
    with_cstr(text, |ptr| {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            bindings::canvas_draw_str(x, y, ptr);
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::canvas_draw_str(x, y, ptr);
        }
    });
}

pub fn canvas_string_width(text: &str) -> u32 {
    with_cstr(text, |ptr| {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            return bindings::canvas_string_width(ptr).max(0) as u32;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::canvas_string_width(ptr).max(0) as u32
        }
    })
}

fn with_cstr<R>(text: &str, f: impl FnOnce(*const u8) -> R) -> R {
    let bytes = text.as_bytes();
    let mut buffer = [0u8; STR_BUFFER_SIZE];
    let len = bytes.len().min(STR_BUFFER_SIZE.saturating_sub(1));
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer[len] = 0;
    f(buffer.as_ptr())
}

pub fn random_seed(seed: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::random_seed(seed as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::random_seed(seed as i32);
    }
}

pub fn random_get() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::random_get().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::random_get().max(0) as u32
    }
}

pub fn random_range(max: u32) -> u32 {
    if max == 0 {
        return 0;
    }
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::random_range(max as i32).max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::random_range(max as i32).max(0) as u32
    }
}

pub fn get_time_ms() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::get_time_ms().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::get_time_ms().max(0) as u32
    }
}

pub fn request_render() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::request_render();
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::request_render();
    }
}

pub fn exit_to_launcher() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::exit_to_launcher();
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::exit_to_launcher();
    }
}

pub fn start_app(app_id: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::start_app(app_id as i32);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::start_app(app_id as i32);
    }
}

pub struct AppCell<T: Copy> {
    value: core::cell::Cell<T>,
}

unsafe impl<T: Copy> Sync for AppCell<T> {}

impl<T: Copy> AppCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: core::cell::Cell::new(value),
        }
    }

    pub fn get(&self) -> T {
        self.value.get()
    }

    pub fn set(&self, value: T) {
        self.value.set(value);
    }
}

pub mod color {
    pub const WHITE: u32 = 0;
    pub const BLACK: u32 = 1;
    pub const XOR: u32 = 2;
}

pub mod font {
    pub const PRIMARY: u32 = 0;
    pub const SECONDARY: u32 = 1;
    pub const KEYBOARD: u32 = 2;
    pub const BIG_NUMBERS: u32 = 3;
}

pub mod input {
    pub const KEY_UP: u32 = 0;
    pub const KEY_DOWN: u32 = 1;
    pub const KEY_LEFT: u32 = 2;
    pub const KEY_RIGHT: u32 = 3;
    pub const KEY_OK: u32 = 4;
    pub const KEY_BACK: u32 = 5;

    pub const TYPE_PRESS: u32 = 0;
    pub const TYPE_RELEASE: u32 = 1;
    pub const TYPE_SHORT_PRESS: u32 = 2;
    pub const TYPE_LONG_PRESS: u32 = 3;
    pub const TYPE_REPEAT: u32 = 4;
}

pub mod align {
    pub const LEFT: u32 = 0;
    pub const RIGHT: u32 = 1;
    pub const TOP: u32 = 2;
    pub const BOTTOM: u32 = 3;
    pub const CENTER: u32 = 4;
}

pub mod imgui;

#[macro_export]
macro_rules! export_render {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn render() {
            $func();
        }
    };
}

#[macro_export]
macro_rules! export_on_input {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn on_input(key: u32, kind: u32) {
            $func(key, kind);
        }
    };
}

#[macro_export]
macro_rules! export_get_scene {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn get_scene() -> u32 {
            $func()
        }
    };
}

#[macro_export]
macro_rules! export_set_scene {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn set_scene(scene: u32) {
            $func(scene);
        }
    };
}

#[macro_export]
macro_rules! export_get_scene_count {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn get_scene_count() -> u32 {
            $func()
        }
    };
}

#[macro_export]
macro_rules! wasm_panic_handler {
    () => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    };
}
