// ============================================================================
// Platform Interface (Minimal)
// ============================================================================
// This module defines the MINIMAL interface between WASM apps and the host.
// Platform only provides: framebuffer, input events, time, random.
// All rendering is done in Zig (see canvas.zig).
//
// The platform owns:
// - The main loop / event loop
// - Framebuffer memory allocation
// - Display presentation
// - Input event dispatch
//
// Apps include:
// - All drawing code (canvas.zig)
// - All UI code (imgui.zig)
// - App-specific logic

const std = @import("std");

// ============================================================================
// Display Constants
// ============================================================================

pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 64;

// ============================================================================
// Types (shared between platform and apps)
// ============================================================================

pub const Color = enum(u32) {
    black = 0,
    white = 1,
    xor = 2,
};

pub const Font = enum(u32) {
    primary = 0, // 6x10
    secondary = 1, // 5x7
    keyboard = 2, // 5x8
    big_numbers = 3, // 10x20
};

pub const InputKey = enum(u32) {
    up = 0,
    down = 1,
    left = 2,
    right = 3,
    ok = 4,
    back = 5,
};

pub const InputType = enum(u32) {
    press = 0,
    release = 1,
    short_press = 2,
    long_press = 3,
    repeat = 4,
};

// ============================================================================
// Time API - Imported from platform (optional)
// ============================================================================

// pub extern "env" fn time_ms() u32;

// ============================================================================
// Random API
// ============================================================================
// Implemented in Zig - no platform import needed

// Zig-side PRNG state
var prng_state: u32 = 42;

/// Random number utilities
pub const random = struct {
    pub fn seed(s: u32) void {
        prng_state = s;
    }

    pub fn get() u32 {
        // Simple LCG (same constants as many implementations)
        prng_state = prng_state *% 1103515245 +% 12345;
        return prng_state;
    }

    pub fn range(max: u32) u32 {
        if (max == 0) return 0;
        return get() % max;
    }
};

// ============================================================================
// Canvas API - Re-export from canvas.zig for convenience
// ============================================================================
// Apps can either:
//   const canvas = @import("canvas");
// Or:
//   const platform = @import("platform");
//   platform.canvas.drawLine(...);

pub const canvas = @import("canvas.zig");

// ============================================================================
// Future: Storage API
// ============================================================================

// pub extern "env" fn storage_read(path: [*:0]const u8, buf: [*]u8, len: u32) i32;
// pub extern "env" fn storage_write(path: [*:0]const u8, data: [*]const u8, len: u32) i32;
// pub extern "env" fn storage_delete(path: [*:0]const u8) i32;

// ============================================================================
// Future: Network API
// ============================================================================

// pub extern "env" fn http_request(method: u32, url: [*:0]const u8, body: [*]const u8, body_len: u32, callback_id: u32) void;
// pub extern "env" fn ws_connect(url: [*:0]const u8, callback_id: u32) u32;
// pub extern "env" fn ws_send(handle: u32, data: [*]const u8, len: u32) void;
// pub extern "env" fn ws_close(handle: u32) void;

// ============================================================================
// Future: System API
// ============================================================================

// pub extern "env" fn sys_get_battery_percent() u32;
// pub extern "env" fn sys_get_wifi_connected() bool;
// pub extern "env" fn sys_vibrate(duration_ms: u32) void;
