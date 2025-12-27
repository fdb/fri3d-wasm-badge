// ============================================================================
// Platform Interface
// ============================================================================
// This module defines the interface between WASM apps and the host platform.
// Apps import functions from "env" module; platforms provide implementations.
//
// The platform owns:
// - The main loop / event loop
// - All I/O and async operations
// - Resource lifecycle management
//
// Apps only:
// - Respond to calls (render(), on_input())
// - Request services via imports (canvas_*, storage_*, etc.)
// - Return quickly (no blocking)

const std = @import("std");

// ============================================================================
// Display Constants
// ============================================================================

pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 64;

// ============================================================================
// Color
// ============================================================================

pub const Color = enum(u32) {
    black = 0,
    white = 1,
    xor = 2,
};

// ============================================================================
// Font
// ============================================================================

pub const Font = enum(u32) {
    primary = 0, // 6x10
    secondary = 1, // 5x7
    keyboard = 2, // 5x8
    big_numbers = 3, // 10x20
};

// ============================================================================
// Input
// ============================================================================

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
// Canvas API - Imported from platform
// ============================================================================

pub const canvas = struct {
    // These are imported from the host platform
    pub extern "env" fn canvas_clear() void;
    pub extern "env" fn canvas_width() u32;
    pub extern "env" fn canvas_height() u32;
    pub extern "env" fn canvas_set_color(color: u32) void;
    pub extern "env" fn canvas_set_font(font: u32) void;
    pub extern "env" fn canvas_draw_dot(x: i32, y: i32) void;
    pub extern "env" fn canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32) void;
    pub extern "env" fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32) void;
    pub extern "env" fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32) void;
    pub extern "env" fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, r: u32) void;
    pub extern "env" fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, r: u32) void;
    pub extern "env" fn canvas_draw_circle(x: i32, y: i32, r: u32) void;
    pub extern "env" fn canvas_draw_disc(x: i32, y: i32, r: u32) void;
    pub extern "env" fn canvas_draw_str(x: i32, y: i32, str: [*:0]const u8) void;
    pub extern "env" fn canvas_string_width(str: [*:0]const u8) u32;

    // Convenience wrappers with Zig types
    pub fn clear() void {
        canvas_clear();
    }

    pub fn width() u32 {
        return canvas_width();
    }

    pub fn height() u32 {
        return canvas_height();
    }

    pub fn setColor(color: Color) void {
        canvas_set_color(@intFromEnum(color));
    }

    pub fn setFont(font: Font) void {
        canvas_set_font(@intFromEnum(font));
    }

    pub fn drawDot(x: i32, y: i32) void {
        canvas_draw_dot(x, y);
    }

    pub fn drawLine(x1: i32, y1: i32, x2: i32, y2: i32) void {
        canvas_draw_line(x1, y1, x2, y2);
    }

    pub fn drawFrame(x: i32, y: i32, w: u32, h: u32) void {
        canvas_draw_frame(x, y, w, h);
    }

    pub fn drawBox(x: i32, y: i32, w: u32, h: u32) void {
        canvas_draw_box(x, y, w, h);
    }

    pub fn drawRFrame(x: i32, y: i32, w: u32, h: u32, r: u32) void {
        canvas_draw_rframe(x, y, w, h, r);
    }

    pub fn drawRBox(x: i32, y: i32, w: u32, h: u32, r: u32) void {
        canvas_draw_rbox(x, y, w, h, r);
    }

    pub fn drawCircle(x: i32, y: i32, r: u32) void {
        canvas_draw_circle(x, y, r);
    }

    pub fn drawDisc(x: i32, y: i32, r: u32) void {
        canvas_draw_disc(x, y, r);
    }

    pub fn drawStr(x: i32, y: i32, str: [:0]const u8) void {
        canvas_draw_str(x, y, str.ptr);
    }

    pub fn stringWidth(str: [:0]const u8) u32 {
        return canvas_string_width(str.ptr);
    }
};

// ============================================================================
// Random API - Imported from platform
// ============================================================================

pub const random = struct {
    pub extern "env" fn random_seed(seed: u32) void;
    pub extern "env" fn random_get() u32;
    pub extern "env" fn random_range(max: u32) u32;

    pub fn seed(s: u32) void {
        random_seed(s);
    }

    pub fn get() u32 {
        return random_get();
    }

    pub fn range(max: u32) u32 {
        return random_range(max);
    }
};

// ============================================================================
// Future: Storage API
// ============================================================================

// pub const storage = struct {
//     pub extern "env" fn storage_read(path: [*:0]const u8, callback_id: u32) void;
//     pub extern "env" fn storage_write(path: [*:0]const u8, data: [*]const u8, len: u32, callback_id: u32) void;
//     pub extern "env" fn storage_delete(path: [*:0]const u8, callback_id: u32) void;
//     pub extern "env" fn storage_list(path: [*:0]const u8, callback_id: u32) void;
// };

// ============================================================================
// Future: Network API
// ============================================================================

// pub const network = struct {
//     pub extern "env" fn http_request(method: u32, url: [*:0]const u8, body: [*]const u8, body_len: u32, callback_id: u32) void;
//     pub extern "env" fn ws_connect(url: [*:0]const u8, callback_id: u32) u32;
//     pub extern "env" fn ws_send(handle: u32, data: [*]const u8, len: u32) void;
//     pub extern "env" fn ws_close(handle: u32) void;
// };

// ============================================================================
// Future: System API
// ============================================================================

// pub const system = struct {
//     pub extern "env" fn sys_get_battery_percent() u32;
//     pub extern "env" fn sys_get_wifi_connected() bool;
//     pub extern "env" fn sys_get_time_unix() u64;
//     pub extern "env" fn sys_vibrate(duration_ms: u32) void;
// };
