//! Fri3d badge kernel.
//!
//! One crate, three hosts (desktop, browser, badge). The kernel owns every
//! byte of state the badge needs at runtime and allocates it once at boot:
//! the 320x240 framebuffer, the input queue, the app registry, the settings
//! table. Apps are WebAssembly modules run by wasmi; each app gets a fixed
//! fuel budget per call and a capped linear memory, so a broken app cannot
//! stall or exhaust the badge.
//!
//! Hosts drive the kernel with three calls:
//!
//! - [`Kernel::push_raw_input`] when a physical button changes state,
//! - [`Kernel::step`] once per loop iteration with the current clock,
//! - [`Kernel::framebuffer`] to blit when `step` reports a new frame.
//!
//! Nothing in here touches a display, a GPIO or a file. That is the host's
//! job, which is what keeps the three targets byte-identical.
#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

pub mod bundle;
pub mod canvas;
pub mod font;
pub mod fonts;
pub mod host;
pub mod input;
pub mod kernel;
pub mod random;
pub mod registry;
pub mod settings;
pub mod types;
pub mod net;
pub mod palette;
pub mod wifi;

pub use kernel::{Kernel, StepResult};

/// Native canvas: the badge LCD, one byte (a [`palette`] index) per pixel.
pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 240;
pub const FRAMEBUFFER_LEN: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

/// Kernel ABI version reported to apps. Bump when host imports change shape.
pub const KERNEL_VERSION: u32 = 3;

/// Hard limits. Every table in the kernel is sized by one of these.
pub mod limits {
    /// Maximum apps in the registry (launcher excluded).
    pub const MAX_APPS: usize = 32;
    /// Wasm instructions an app may execute per host→app call
    /// (render, on_input, lifecycle). Measured: Mandelbrot at deep zoom
    /// ≈ 1.1M, Snake ≈ 600, launcher ≈ 2k. 40M leaves 35× headroom for
    /// the heaviest app and bounds a runaway loop to ~1–2 s on the badge.
    pub const FUEL_PER_CALL: u64 = 40_000_000;
    /// Linear memory cap per app instance (bytes). Rust apps with a 16 KB
    /// stack use 1–2 pages; this allows a 128 KB framebuffer-sized scratch.
    pub const APP_MEMORY_MAX: usize = 256 * 1024;
    /// Queue depth for synthesized input events between two `step` calls.
    pub const INPUT_QUEUE: usize = 32;
    /// Settings entries across all apps.
    pub const SETTINGS_ENTRIES: usize = 64;
    /// Log lines buffered for the host to drain.
    pub const LOG_LINES: usize = 8;
    pub const LOG_LINE_LEN: usize = 96;
    /// Access points kept from the last Wi-Fi scan.
    pub const WIFI_SCAN_MAX: usize = 16;
    /// Saved Wi-Fi networks (one bit each in the auto-connect round).
    pub const WIFI_SAVED_MAX: usize = 8;
}
