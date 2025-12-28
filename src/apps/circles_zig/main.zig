// ============================================================================
// Circles - A simple app demonstrating the Zig SDK
// ============================================================================
// Draws random circles on screen. Press OK to regenerate.
// Works on both WASM (web) and native (desktop) platforms.

const std = @import("std");
const builtin = @import("builtin");
const platform = @import("platform");

const is_wasm = builtin.cpu.arch == .wasm32;

// Global state
var g_seed: u32 = 42;

// ============================================================================
// App Interface - called by platform (both WASM and native)
// ============================================================================

/// Initialize the app (called once at startup)
pub fn init() void {
    // No initialization needed for this simple app
}

/// Called each frame for rendering
pub fn render() void {
    // Use same seed each frame for consistent circles
    platform.random.seed(g_seed);
    platform.canvas.setColor(.black);

    // Draw 10 random circles
    var i: u32 = 0;
    while (i < 10) : (i += 1) {
        const x: i32 = @intCast(platform.random.range(128));
        const y: i32 = @intCast(platform.random.range(64));
        const r: u32 = platform.random.range(15) + 3;
        platform.canvas.drawCircle(x, y, r);
    }
}

/// Called for input events
pub fn on_input(key: platform.InputKey, input_type: platform.InputType) void {
    if (input_type == .press and key == .ok) {
        // Generate new random seed for new circles
        g_seed = platform.random.get();
    }
}

// ============================================================================
// WASM Exports - only compiled for WASM target
// ============================================================================

pub usingnamespace if (is_wasm) struct {
    export fn render() void {
        @import("root").render();
    }

    export fn on_input(key: u32, input_type: u32) void {
        @import("root").on_input(@enumFromInt(key), @enumFromInt(input_type));
    }

    export fn get_scene() u32 {
        return 0;
    }

    export fn set_scene(scene: u32) void {
        _ = scene;
    }

    export fn get_scene_count() u32 {
        return 1;
    }
} else struct {};
