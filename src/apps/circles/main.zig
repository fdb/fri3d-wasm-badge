// ============================================================================
// Circles - A simple app demonstrating the Zig SDK
// ============================================================================
// Draws random circles on screen. Press OK to regenerate.

const platform = @import("platform");

// Global state
var g_seed: u32 = 42;

// ============================================================================
// WASM Exports - called by platform runtime
// ============================================================================

export fn render() void {
    // Use same seed each frame for consistent circles
    platform.random.seed(g_seed);

    // Clear screen to white background
    platform.canvas.setColor(.white);
    platform.canvas.drawBox(0, 0, platform.SCREEN_WIDTH, platform.SCREEN_HEIGHT);

    // Draw black circles
    platform.canvas.setColor(.black);
    var i: u32 = 0;
    while (i < 10) : (i += 1) {
        const x: i32 = @intCast(platform.random.range(128));
        const y: i32 = @intCast(platform.random.range(64));
        const r: u32 = platform.random.range(15) + 3;
        platform.canvas.drawCircle(x, y, r);
    }
}

export fn on_input(key: u32, input_type: u32) void {
    const key_enum: platform.InputKey = @enumFromInt(key);
    const type_enum: platform.InputType = @enumFromInt(input_type);

    if (type_enum == .press and key_enum == .ok) {
        // Generate new random seed for new circles
        g_seed = platform.random.get();
    }
}

// Scene API (for visual testing)
export fn get_scene() u32 {
    return 0;
}

export fn set_scene(_: u32) void {}

export fn get_scene_count() u32 {
    return 1;
}
