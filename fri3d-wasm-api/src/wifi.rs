//! Wi-Fi as apps see it. Reads work for every app; actions (scan, save,
//! connect, forget, enable) only work from apps packed with `system = true`
//! and return false / do nothing otherwise. Passwords are write-only: an
//! app can save one but never read it back.

pub const STATUS_OFF: u32 = 0;
pub const STATUS_IDLE: u32 = 1;
pub const STATUS_CONNECTING: u32 = 2;
pub const STATUS_CONNECTED: u32 = 3;
pub const STATUS_FAILED: u32 = 4;

pub const SSID_LEN: usize = 32;
pub const PASSWORD_LEN: usize = 64;

#[cfg(target_arch = "wasm32")]
mod bindings {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn wifi_status() -> i32;
        pub fn wifi_scanning() -> i32;
        pub fn wifi_enabled() -> i32;
        pub fn wifi_current_ssid(ptr: *mut u8, len: i32) -> i32;
        pub fn wifi_scan_count() -> i32;
        pub fn wifi_scan_ssid(index: i32, ptr: *mut u8, len: i32) -> i32;
        pub fn wifi_scan_rssi(index: i32) -> i32;
        pub fn wifi_scan_secure(index: i32) -> i32;
        pub fn wifi_saved_count() -> i32;
        pub fn wifi_saved_ssid(index: i32, ptr: *mut u8, len: i32) -> i32;
        pub fn wifi_set_enabled(on: i32);
        pub fn wifi_scan() -> i32;
        pub fn wifi_disconnect();
        pub fn wifi_connect(ssid: *const u8) -> i32;
        pub fn wifi_forget(ssid: *const u8) -> i32;
        pub fn wifi_save(ssid: *const u8, password: *const u8) -> i32;
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables)]
mod bindings {
    pub fn wifi_status() -> i32 {
        0
    }
    pub fn wifi_scanning() -> i32 {
        0
    }
    pub fn wifi_enabled() -> i32 {
        0
    }
    pub fn wifi_current_ssid(ptr: *mut u8, len: i32) -> i32 {
        0
    }
    pub fn wifi_scan_count() -> i32 {
        0
    }
    pub fn wifi_scan_ssid(index: i32, ptr: *mut u8, len: i32) -> i32 {
        0
    }
    pub fn wifi_scan_rssi(index: i32) -> i32 {
        -128
    }
    pub fn wifi_scan_secure(index: i32) -> i32 {
        0
    }
    pub fn wifi_saved_count() -> i32 {
        0
    }
    pub fn wifi_saved_ssid(index: i32, ptr: *mut u8, len: i32) -> i32 {
        0
    }
    pub fn wifi_set_enabled(on: i32) {}
    pub fn wifi_scan() -> i32 {
        0
    }
    pub fn wifi_disconnect() {}
    pub fn wifi_connect(ssid: *const u8) -> i32 {
        0
    }
    pub fn wifi_forget(ssid: *const u8) -> i32 {
        0
    }
    pub fn wifi_save(ssid: *const u8, password: *const u8) -> i32 {
        0
    }
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

/// A fixed-capacity SSID. No allocation.
#[derive(Copy, Clone)]
pub struct Ssid {
    buf: [u8; SSID_LEN],
    len: u8,
}

impl Ssid {
    pub const fn empty() -> Self {
        Self { buf: [0; SSID_LEN], len: 0 }
    }

    pub fn from_str(s: &str) -> Self {
        let mut out = Self::empty();
        let n = s.len().min(SSID_LEN);
        out.buf[..n].copy_from_slice(&s.as_bytes()[..n]);
        out.len = n as u8;
        out
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn fill(f: impl FnOnce(*mut u8, i32) -> i32) -> Self {
        let mut out = Self::empty();
        let n = f(out.buf.as_mut_ptr(), SSID_LEN as i32);
        out.len = n.clamp(0, SSID_LEN as i32) as u8;
        out
    }
}

impl PartialEq for Ssid {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

/// One of the `STATUS_*` constants.
pub fn status() -> u32 {
    call!(bindings::wifi_status()) as u32
}

pub fn scanning() -> bool {
    call!(bindings::wifi_scanning()) != 0
}

pub fn enabled() -> bool {
    call!(bindings::wifi_enabled()) != 0
}

/// Network being connected to, connected to, or that failed last.
pub fn current_ssid() -> Ssid {
    Ssid::fill(|p, n| call!(bindings::wifi_current_ssid(p, n)))
}

pub fn scan_count() -> u32 {
    call!(bindings::wifi_scan_count()).max(0) as u32
}

pub fn scan_ssid(index: u32) -> Ssid {
    Ssid::fill(|p, n| call!(bindings::wifi_scan_ssid(index as i32, p, n)))
}

/// Signal strength in dBm (−128 when out of range).
pub fn scan_rssi(index: u32) -> i32 {
    call!(bindings::wifi_scan_rssi(index as i32))
}

pub fn scan_secure(index: u32) -> bool {
    call!(bindings::wifi_scan_secure(index as i32)) != 0
}

pub fn saved_count() -> u32 {
    call!(bindings::wifi_saved_count()).max(0) as u32
}

pub fn saved_ssid(index: u32) -> Ssid {
    Ssid::fill(|p, n| call!(bindings::wifi_saved_ssid(index as i32, p, n)))
}

pub fn is_saved(ssid: &str) -> bool {
    (0..saved_count()).any(|i| saved_ssid(i).as_str() == ssid)
}

/// System apps only. Persists `system.wifi` and starts or stops the radio.
pub fn set_enabled(on: bool) {
    call!(bindings::wifi_set_enabled(on as i32))
}

/// System apps only. Starts a scan; results arrive asynchronously and the
/// kernel re-renders the focused app when they do.
pub fn scan() -> bool {
    call!(bindings::wifi_scan()) != 0
}

pub fn disconnect() {
    call!(bindings::wifi_disconnect())
}

/// System apps only. Connect to a saved network.
pub fn connect(ssid: &str) -> bool {
    with_cstr::<{ SSID_LEN + 1 }, _>(ssid, |p| call!(bindings::wifi_connect(p))) != 0
}

pub fn forget(ssid: &str) -> bool {
    with_cstr::<{ SSID_LEN + 1 }, _>(ssid, |p| call!(bindings::wifi_forget(p))) != 0
}

/// System apps only. Add or update a network. An empty password means an
/// open network.
pub fn save(ssid: &str, password: &str) -> bool {
    with_cstr::<{ SSID_LEN + 1 }, _>(ssid, |s| {
        with_cstr::<{ PASSWORD_LEN + 1 }, _>(password, |p| call!(bindings::wifi_save(s, p)))
    }) != 0
}

fn with_cstr<const N: usize, R>(s: &str, f: impl FnOnce(*const u8) -> R) -> R {
    let mut buf = [0u8; N];
    let n = s.len().min(N - 1);
    buf[..n].copy_from_slice(&s.as_bytes()[..n]);
    f(buf.as_ptr())
}
