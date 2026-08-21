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
        // Overwrites the entire framebuffer in one host call.
        // `ptr` points to exactly width*height bytes (0 = white, 1 = black).
        // Use for apps that render whole frames at once (Mandelbrot etc) —
        // saves the per-pixel wasm3 boundary crossing.
        pub fn canvas_draw_buffer(ptr: *const u8, len: i32);
        pub fn random_seed(seed: i32);
        pub fn random_get() -> i32;
        pub fn random_range(max: i32) -> i32;
        pub fn get_time_ms() -> i32;
        pub fn start_timer_ms(interval_ms: i32);
        pub fn stop_timer();
        pub fn request_render();
        pub fn exit_to_launcher();
        pub fn start_app(app_id: i32);
        // Draws a 1-bit bitmap (rows of ceil(w/8) bytes, MSB first) in the
        // current color. Clear bits are transparent.
        pub fn canvas_draw_bitmap(x: i32, y: i32, w: i32, h: i32, ptr: *const u8);
        pub fn app_count() -> i32;
        // Copies the 256-byte bundle header of app `index` to `ptr`.
        pub fn app_info(index: i32, ptr: *mut u8, len: i32) -> i32;
        pub fn kernel_version() -> i32;
        pub fn settings_get_u32(ns: *const u8, key: *const u8, default: i32) -> i32;
        pub fn settings_set_u32(ns: *const u8, key: *const u8, value: i32) -> i32;
        pub fn log_str(ptr: *const u8);
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

    pub fn canvas_draw_buffer(_ptr: *const u8, _len: i32) {}

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

    pub fn start_timer_ms(_interval_ms: i32) {}

    pub fn stop_timer() {}

    pub fn request_render() {}

    pub fn exit_to_launcher() {}

    pub fn start_app(_app_id: i32) {}

    pub fn canvas_draw_bitmap(_x: i32, _y: i32, _w: i32, _h: i32, _ptr: *const u8) {}

    pub fn app_count() -> i32 {
        0
    }

    pub fn app_info(_index: i32, _ptr: *mut u8, _len: i32) -> i32 {
        -1
    }

    pub fn kernel_version() -> i32 {
        0
    }

    pub fn settings_get_u32(_ns: *const u8, _key: *const u8, default: i32) -> i32 {
        default
    }

    pub fn settings_set_u32(_ns: *const u8, _key: *const u8, _value: i32) -> i32 {
        0
    }

    pub fn log_str(_ptr: *const u8) {}
}

/// Native canvas size. Hosts upscale (2× on the badge LCD).
pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 120;

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

/// Overwrite the entire 128x64 framebuffer in one host call. `buffer` must
/// be exactly canvas_width() * canvas_height() bytes (SCREEN_WIDTH * SCREEN_HEIGHT),
/// where 0 = white (background) and 1 = black (foreground). Faster than
/// per-pixel canvas_draw_dot for apps that render a full frame every time
/// (Mandelbrot, procedural textures, etc).
pub fn canvas_draw_buffer(buffer: &[u8]) {
    let ptr = buffer.as_ptr();
    let len = buffer.len() as i32;
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_buffer(ptr, len);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_buffer(ptr, len);
    }
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
    // Reinterpret the 32 bits as unsigned — MT19937's full range includes
    // values with the top bit set, which appear negative in i32. Clamping
    // with .max(0) would collapse ~half of the output space to zero.
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::random_get() as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::random_get() as u32
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

pub fn start_timer_ms(interval_ms: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::start_timer_ms(interval_ms as i32);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::start_timer_ms(interval_ms as i32);
    }
}

pub fn stop_timer() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::stop_timer();
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::stop_timer();
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

/// Draw a 1-bit bitmap with the current color. `bits` holds `h` rows of
/// `ceil(w/8)` bytes, MSB = leftmost pixel. Clear bits are transparent.
pub fn canvas_draw_bitmap(x: i32, y: i32, w: u32, h: u32, bits: &[u8]) {
    let row_bytes = (w as usize).div_ceil(8);
    if bits.len() < row_bytes * h as usize {
        return;
    }
    let ptr = bits.as_ptr();
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_bitmap(x, y, w as i32, h as i32, ptr);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_bitmap(x, y, w as i32, h as i32, ptr);
    }
}

/// Number of installed apps (launcher excluded). Index `0..count` is the
/// id accepted by `start_app`.
pub fn app_count() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::app_count().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::app_count().max(0) as u32
    }
}

/// Kernel ABI version.
pub fn kernel_version() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return bindings::kernel_version().max(0) as u32;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::kernel_version().max(0) as u32
    }
}

/// Log a line to the host console. Clipped to 96 bytes by the kernel.
pub fn log(text: &str) {
    with_cstr(text, |ptr| {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            bindings::log_str(ptr);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::log_str(ptr);
        }
    });
}

/// Read a persisted u32 setting. `ns` must be this app's id, or `"system"`
/// for apps packed with `system = true`. Other namespaces return `default`.
pub fn settings_get_u32(ns: &str, key: &str, default: u32) -> u32 {
    with_two_cstr(ns, key, |ns, key| {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            return bindings::settings_get_u32(ns, key, default as i32) as u32;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::settings_get_u32(ns, key, default as i32) as u32
        }
    })
}

/// Persist a u32 setting. Returns false when denied or when the table is
/// full (64 entries across all apps).
pub fn settings_set_u32(ns: &str, key: &str, value: u32) -> bool {
    with_two_cstr(ns, key, |ns, key| {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            return bindings::settings_set_u32(ns, key, value as i32) != 0;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::settings_set_u32(ns, key, value as i32) != 0
        }
    })
}

fn with_two_cstr<R>(a: &str, b: &str, f: impl FnOnce(*const u8, *const u8) -> R) -> R {
    const N: usize = 24;
    let mut ba = [0u8; N];
    let mut bb = [0u8; N];
    let la = a.len().min(N - 1);
    let lb = b.len().min(N - 1);
    ba[..la].copy_from_slice(&a.as_bytes()[..la]);
    bb[..lb].copy_from_slice(&b.as_bytes()[..lb]);
    f(ba.as_ptr(), bb.as_ptr())
}

/// The 256-byte bundle header of an installed app, as copied by the
/// kernel. Field offsets match `fri3d_kernel::bundle`.
pub struct AppInfo {
    header: [u8; AppInfo::LEN],
}

impl AppInfo {
    pub const LEN: usize = 256;
    pub const ICON_W: u32 = 14;
    pub const ICON_H: u32 = 14;

    pub const fn empty() -> Self {
        Self { header: [0; Self::LEN] }
    }

    /// Fetch app `index` from the kernel. Returns false if out of range.
    pub fn fetch(&mut self, index: u32) -> bool {
        let ptr = self.header.as_mut_ptr();
        #[cfg(target_arch = "wasm32")]
        unsafe {
            return bindings::app_info(index as i32, ptr, Self::LEN as i32) == Self::LEN as i32;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            bindings::app_info(index as i32, ptr, Self::LEN as i32) == Self::LEN as i32
        }
    }

    fn field(&self, at: usize, max: usize) -> &str {
        let f = &self.header[at..at + max];
        let end = f.iter().position(|&b| b == 0).unwrap_or(max);
        core::str::from_utf8(&f[..end]).unwrap_or("")
    }

    pub fn id(&self) -> &str {
        self.field(8, 24)
    }
    pub fn name(&self) -> &str {
        self.field(32, 32)
    }
    pub fn version(&self) -> &str {
        self.field(64, 16)
    }
    pub fn author(&self) -> &str {
        self.field(80, 32)
    }
    pub fn description(&self) -> &str {
        self.field(112, 96)
    }
    pub fn is_system(&self) -> bool {
        self.header[6] & 1 != 0
    }
    /// 14x14 1-bit icon, 2 bytes per row.
    pub fn icon(&self) -> &[u8] {
        &self.header[212..212 + 28]
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
    /// Home key. A short press always returns to the launcher; apps still
    /// see the press/release pair and the long press.
    pub const KEY_MENU: u32 = 6;

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
pub mod wifi;
pub mod net;

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

/// Lifecycle exports. All optional; the kernel calls what it finds.
///
/// `on_start` runs once after the module is instantiated, before the first
/// render. `on_resume` runs when the app gains the screen, `on_pause` when
/// it loses it (the launcher sees these around every app launch). `on_stop`
/// runs once before the instance is dropped — persist state here.
#[macro_export]
macro_rules! export_on_start {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn on_start() {
            $func();
        }
    };
}

#[macro_export]
macro_rules! export_on_stop {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn on_stop() {
            $func();
        }
    };
}

#[macro_export]
macro_rules! export_on_pause {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn on_pause() {
            $func();
        }
    };
}

#[macro_export]
macro_rules! export_on_resume {
    ($func:path) => {
        #[no_mangle]
        #[allow(unsafe_code)]
        pub extern "C" fn on_resume() {
            $func();
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
