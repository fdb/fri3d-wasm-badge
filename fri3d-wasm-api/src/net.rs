//! Network operations for apps: start one, poll its progress. There is
//! no socket API; the host runs the operation and only counts bytes.

pub const STATUS_IDLE: u32 = 0;
pub const STATUS_BUSY: u32 = 1;
pub const STATUS_DONE: u32 = 2;
pub const STATUS_FAILED: u32 = 3;

pub const URL_LEN: usize = 96;

#[cfg(target_arch = "wasm32")]
mod bindings {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn net_status() -> i32;
        pub fn net_bytes() -> i32;
        pub fn net_elapsed_ms() -> i32;
        pub fn net_probe(ip: i32, port: i32) -> i32;
        pub fn net_download(url: *const u8) -> i32;
        pub fn net_cancel();
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables)]
mod bindings {
    pub fn net_status() -> i32 {
        0
    }
    pub fn net_bytes() -> i32 {
        0
    }
    pub fn net_elapsed_ms() -> i32 {
        0
    }
    pub fn net_probe(ip: i32, port: i32) -> i32 {
        0
    }
    pub fn net_download(url: *const u8) -> i32 {
        0
    }
    pub fn net_cancel() {}
}

macro_rules! call {
    ($e:expr) => {{
        #[cfg(target_arch = "wasm32")]
        unsafe {
            $e
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            $e
        }
    }};
}

/// One of the `STATUS_*` constants.
pub fn status() -> u32 {
    call!(bindings::net_status()) as u32
}

/// Bytes received so far by the current or last operation.
pub fn bytes() -> u32 {
    call!(bindings::net_bytes()) as u32
}

/// Milliseconds the current or last operation took (so far).
pub fn elapsed_ms() -> u32 {
    call!(bindings::net_elapsed_ms()) as u32
}

/// Open and close a TCP connection to `ip:port`. False when busy.
pub fn probe(ip: [u8; 4], port: u16) -> bool {
    let packed = u32::from_be_bytes(ip) as i32;
    call!(bindings::net_probe(packed, port as i32)) != 0
}

/// HTTP GET `url` (plain http), counting and discarding the body.
/// False when busy or the URL is longer than `URL_LEN`.
pub fn download(url: &str) -> bool {
    let mut buf = [0u8; URL_LEN + 1];
    let n = url.len().min(URL_LEN);
    buf[..n].copy_from_slice(&url.as_bytes()[..n]);
    call!(bindings::net_download(buf.as_ptr())) != 0
}

pub fn cancel() {
    call!(bindings::net_cancel())
}
