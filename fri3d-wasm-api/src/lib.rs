#![no_std]

#[cfg(target_arch = "wasm32")]
mod bindings {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn canvas_width() -> i32;
        pub fn canvas_height() -> i32;
        pub fn canvas_set_color(color: i32);
        pub fn canvas_draw_dot(x: i32, y: i32);
        pub fn random_range(max: i32) -> i32;
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod bindings {
    pub fn canvas_width() -> i32 {
        0
    }

    pub fn canvas_height() -> i32 {
        0
    }

    pub fn canvas_set_color(_color: i32) {}

    pub fn canvas_draw_dot(_x: i32, _y: i32) {}

    pub fn random_range(_max: i32) -> i32 {
        0
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

pub fn canvas_draw_dot(x: u32, y: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bindings::canvas_draw_dot(x as i32, y as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bindings::canvas_draw_dot(x as i32, y as i32);
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
macro_rules! wasm_panic_handler {
    () => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    };
}
