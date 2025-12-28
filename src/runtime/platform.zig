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
// Canvas API - Re-export from canvas.zig
// ============================================================================
// Canvas owns the framebuffer and all drawing code.
// Re-export here for convenience so apps can use platform.canvas.*

pub const canvas = @import("canvas.zig");

// Re-export display types from canvas for convenience
pub const Color = canvas.Color;
pub const Font = canvas.Font;
pub const SCREEN_WIDTH = canvas.SCREEN_WIDTH;
pub const SCREEN_HEIGHT = canvas.SCREEN_HEIGHT;

// ============================================================================
// Input Types (platform-specific, not in canvas)
// ============================================================================

pub const InputKey = enum(u32) {
    up = 0,
    down = 1,
    left = 2,
    right = 3,
    ok = 4,
    back = 5,
};

/// Input event types sent by the platform to apps via on_input()
///
/// Event timing semantics:
/// - `press`: Key went down. Sent immediately on keydown. Use for visual feedback
///   or games needing continuous movement while key is held.
/// - `release`: Key went up. Sent immediately on keyup.
/// - `short_press`: Key released before LONG_PRESS threshold. Sent on release.
///   Mutually exclusive with long_press for a given press. Use for menu navigation.
/// - `long_press`: Key held >= LONG_PRESS threshold (500ms). Sent once while held.
///   After this, repeat events begin. short_press is NOT sent on release.
///   Use for alternate actions (e.g., uppercase letters on keyboard).
/// - `repeat`: Key continues to be held after long_press. Fires periodically (100ms).
///   Use for scrolling through long lists.
///
/// Typical usage patterns:
/// - Menu navigation: React to `short_press` for single steps, `repeat` for scrolling
/// - Keyboard app: Show feedback on `press`, lowercase on `short_press`, uppercase on `long_press`
/// - Game (Arkanoid): Use `press`/`release` for continuous paddle movement
/// - Exit gesture: `long_press` on BACK to exit app
pub const InputType = enum(u32) {
    press = 0,
    release = 1,
    short_press = 2,
    long_press = 3,
    repeat = 4,
};

/// Platform timing constants (used by platform implementations)
pub const input_timing = struct {
    /// Time threshold for long_press detection (milliseconds)
    pub const LONG_PRESS_MS: u32 = 500;
    /// Interval between repeat events (milliseconds)
    pub const REPEAT_INTERVAL_MS: u32 = 100;
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
// App Lifecycle API
// ============================================================================
// Request the platform to launch another app. The current app will be unloaded.
// The app_id is an index into the platform's app registry.

extern "env" fn platform_request_launch_app(app_id: u32) void;
extern "env" fn platform_request_exit_to_launcher() void;

pub const app = struct {
    /// Request to launch an app by ID. Current app will be unloaded.
    pub fn launchApp(app_id: u32) void {
        platform_request_launch_app(app_id);
    }

    /// Request to exit back to the launcher.
    pub fn exitToLauncher() void {
        platform_request_exit_to_launcher();
    }
};

// ============================================================================
// Future: System API
// ============================================================================

// pub extern "env" fn sys_get_battery_percent() u32;
// pub extern "env" fn sys_get_wifi_connected() bool;
// pub extern "env" fn sys_vibrate(duration_ms: u32) void;
