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
        // Overwrites the entire 128*64 framebuffer in one host call.
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

        // ---- Full-screen RGB API (296x240) -------------------------------
        // Colors are 24-bit packed 0x00RRGGBB. The host packs to RGB565 once
        // per draw call (the LCD's native format), so apps never see 16-bit
        // values — just standard web-style RGB.
        pub fn screen_width() -> i32;
        pub fn screen_height() -> i32;
        pub fn screen_clear(rgb: i32);
        pub fn screen_pixel(x: i32, y: i32, rgb: i32);
        pub fn screen_line(x1: i32, y1: i32, x2: i32, y2: i32, rgb: i32);
        pub fn screen_fill_rect(x: i32, y: i32, w: i32, h: i32, rgb: i32);
        pub fn screen_stroke_rect(x: i32, y: i32, w: i32, h: i32, rgb: i32);
        pub fn screen_circle(x: i32, y: i32, radius: i32, rgb: i32);
        pub fn screen_disc(x: i32, y: i32, radius: i32, rgb: i32);
        pub fn screen_text(x: i32, y: i32, text: *const u8, rgb: i32, font: i32);
        // Polygon family — `pts` is a packed [x0,y0,x1,y1,...] array of
        // length 2*n_points. Polyline is open; polygon and polygon_fill
        // close to the first point.
        pub fn screen_polyline    (pts: *const i32, n_points: i32, rgb: i32);
        pub fn screen_polygon     (pts: *const i32, n_points: i32, rgb: i32);
        pub fn screen_polygon_fill(pts: *const i32, n_points: i32, rgb: i32);
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

    pub fn screen_width() -> i32 { 296 }
    pub fn screen_height() -> i32 { 240 }
    pub fn screen_clear(_rgb: i32) {}
    pub fn screen_pixel(_x: i32, _y: i32, _rgb: i32) {}
    pub fn screen_line(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _rgb: i32) {}
    pub fn screen_fill_rect(_x: i32, _y: i32, _w: i32, _h: i32, _rgb: i32) {}
    pub fn screen_stroke_rect(_x: i32, _y: i32, _w: i32, _h: i32, _rgb: i32) {}
    pub fn screen_circle(_x: i32, _y: i32, _radius: i32, _rgb: i32) {}
    pub fn screen_disc(_x: i32, _y: i32, _radius: i32, _rgb: i32) {}
    pub fn screen_text(_x: i32, _y: i32, _text: *const u8, _rgb: i32, _font: i32) {}
    pub fn screen_polyline    (_pts: *const i32, _n: i32, _rgb: i32) {}
    pub fn screen_polygon     (_pts: *const i32, _n: i32, _rgb: i32) {}
    pub fn screen_polygon_fill(_pts: *const i32, _n: i32, _rgb: i32) {}
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

/// Overwrite the entire 128x64 framebuffer in one host call. `buffer` must
/// be exactly canvas_width() * canvas_height() bytes (8192 for the badge),
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

/// Vector UI kit — reusable drawing patterns from the Fri3d badge OS
/// design system (status bars, cards, hint bars, fox glyphs). Apps that
/// use these helpers get visual consistency for free instead of every
/// author re-deriving the look. See [`ui`] for what's available.
pub mod ui;

/// Full-screen 296x240 RGB color drawing API. Used by the new launcher and
/// any app written against the design-system aesthetic. Apps that only use
/// the legacy `canvas_*` functions still work — the host blits the 128x64
/// mono canvas centred and 2x-upscaled into the screen automatically.
pub mod screen {
    use super::bindings;

    pub const WIDTH:  u32 = 296;
    pub const HEIGHT: u32 = 240;

    /// Pack r/g/b into the 24-bit format the host expects (0x00RRGGBB).
    pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    fn call<T>(f: impl FnOnce() -> T) -> T { f() }

    pub fn width()  -> u32 { call(|| {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_width().max(0)  as u32 }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_width().max(0)  as u32 }
    })}
    pub fn height() -> u32 { call(|| {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_height().max(0) as u32 }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_height().max(0) as u32 }
    })}
    pub fn clear(rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_clear(rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_clear(rgb as i32); }
    }
    pub fn pixel(x: i32, y: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_pixel(x, y, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_pixel(x, y, rgb as i32); }
    }
    pub fn line(x1: i32, y1: i32, x2: i32, y2: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_line(x1, y1, x2, y2, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_line(x1, y1, x2, y2, rgb as i32); }
    }
    pub fn fill_rect(x: i32, y: i32, w: i32, h: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_fill_rect(x, y, w, h, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_fill_rect(x, y, w, h, rgb as i32); }
    }
    pub fn stroke_rect(x: i32, y: i32, w: i32, h: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_stroke_rect(x, y, w, h, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_stroke_rect(x, y, w, h, rgb as i32); }
    }
    pub fn circle(x: i32, y: i32, radius: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_circle(x, y, radius, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_circle(x, y, radius, rgb as i32); }
    }
    pub fn disc(x: i32, y: i32, radius: i32, rgb: u32) {
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_disc(x, y, radius, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_disc(x, y, radius, rgb as i32); }
    }
    pub fn text(x: i32, y: i32, text: &str, rgb: u32, font: u32) {
        super::with_cstr(text, |ptr| {
            #[cfg(target_arch = "wasm32")] unsafe {
                bindings::screen_text(x, y, ptr, rgb as i32, font as i32);
            }
            #[cfg(not(target_arch = "wasm32"))] {
                bindings::screen_text(x, y, ptr, rgb as i32, font as i32);
            }
        });
    }

    /// Draw a connected line through `pts` (interleaved x,y pairs). Open;
    /// last point doesn't connect back to first.
    pub fn polyline(pts: &[i32], rgb: u32) {
        let n = (pts.len() / 2) as i32;
        if n < 2 { return; }
        let ptr = pts.as_ptr();
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_polyline(ptr, n, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_polyline(ptr, n, rgb as i32); }
    }
    /// Closed outline through `pts`.
    pub fn polygon(pts: &[i32], rgb: u32) {
        let n = (pts.len() / 2) as i32;
        if n < 2 { return; }
        let ptr = pts.as_ptr();
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_polygon(ptr, n, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_polygon(ptr, n, rgb as i32); }
    }
    /// Filled polygon (even-odd rule). Up to 32 vertices per scanline.
    pub fn polygon_fill(pts: &[i32], rgb: u32) {
        let n = (pts.len() / 2) as i32;
        if n < 3 { return; }
        let ptr = pts.as_ptr();
        #[cfg(target_arch = "wasm32")] unsafe { bindings::screen_polygon_fill(ptr, n, rgb as i32); }
        #[cfg(not(target_arch = "wasm32"))] { bindings::screen_polygon_fill(ptr, n, rgb as i32); }
    }
}

/// Fri3d badge OS palette — violet/cyan-on-black "vector terminal" aesthetic
/// from the design system. Pulled directly from the design's CSS custom
/// properties so apps and the launcher stay visually consistent.
pub mod theme {
    use super::screen::rgb;
    pub const BG:        u32 = rgb(0x05, 0x05, 0x0a);  // near-black
    pub const BG_PANEL:  u32 = rgb(0x0a, 0x0a, 0x14);  // panel inset
    pub const INK:       u32 = rgb(0x7b, 0x6c, 0xff);  // violet primary
    pub const INK_DIM:   u32 = rgb(0x4a, 0x3f, 0xb0);
    pub const INK_DEEP:  u32 = rgb(0x2a, 0x24, 0x70);
    pub const ACCENT:    u32 = rgb(0x29, 0xe0, 0xff);  // electric cyan
    pub const ACCENT_DIM:u32 = rgb(0x0f, 0x8a, 0xa8);
    pub const WARN:      u32 = rgb(0xff, 0x5e, 0xa8);  // pink warn
    pub const WHITE:     u32 = rgb(0xe8, 0xe6, 0xff);  // soft white
}

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
