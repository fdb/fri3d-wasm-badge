// ============================================================================
// Mandelbrot - Interactive fractal explorer
// ============================================================================
// Navigate with arrow keys, zoom with OK/Back buttons.

const platform = @import("platform");

// Global state
var g_x_offset: f32 = 2.5;
var g_y_offset: f32 = 1.12;
var g_x_zoom: f32 = 3.5;
var g_y_zoom: f32 = 2.24;

// ============================================================================
// Mandelbrot calculation
// ============================================================================

fn mandelbrotPixel(x: i32, y: i32) bool {
    const x0 = @as(f32, @floatFromInt(x)) / 128.0 * g_x_zoom - g_x_offset;
    const y0 = @as(f32, @floatFromInt(y)) / 64.0 * g_y_zoom - g_y_offset;
    var x1: f32 = 0.0;
    var y1: f32 = 0.0;
    var x2: f32 = 0.0;
    var y2: f32 = 0.0;
    var iter: u32 = 0;

    while (x2 + y2 <= 4.0 and iter < 50) {
        y1 = 2.0 * x1 * y1 + y0;
        x1 = x2 - y2 + x0;
        x2 = x1 * x1;
        y2 = y1 * y1;
        iter += 1;
    }

    return iter > 49;
}

// ============================================================================
// WASM Exports - called by platform runtime
// ============================================================================

export fn render() void {
    // Clear screen to black
    platform.canvas.setColor(.black);
    platform.canvas.drawBox(0, 0, platform.SCREEN_WIDTH, platform.SCREEN_HEIGHT);

    // Draw Mandelbrot set in white
    platform.canvas.setColor(.white);

    var y: i32 = 0;
    while (y < 64) : (y += 1) {
        var x: i32 = 0;
        while (x < 128) : (x += 1) {
            if (mandelbrotPixel(x, y)) {
                platform.canvas.drawDot(x, y);
            }
        }
    }
}

export fn on_input(key: u32, input_type: u32) void {
    const key_enum: platform.InputKey = @enumFromInt(key);
    const type_enum: platform.InputType = @enumFromInt(input_type);

    // Long-press back exits to launcher
    if (type_enum == .long_press and key_enum == .back) {
        platform.app.exitToLauncher();
        return;
    }

    if (type_enum != .press) return;

    // Step is 10% of current view size
    const x_step = g_x_zoom * 0.1;
    const y_step = g_y_zoom * 0.1;

    switch (key_enum) {
        .up => g_y_offset += y_step,
        .down => g_y_offset -= y_step,
        .left => g_x_offset += x_step,
        .right => g_x_offset -= x_step,
        .ok => {
            // Zoom in, keeping center fixed
            g_x_offset -= g_x_zoom * 0.05;
            g_y_offset -= g_y_zoom * 0.05;
            g_x_zoom *= 0.9;
            g_y_zoom *= 0.9;
        },
        .back => {
            // Zoom out, keeping center fixed
            g_x_offset += g_x_zoom * 0.05;
            g_y_offset += g_y_zoom * 0.05;
            g_x_zoom *= 1.1;
            g_y_zoom *= 1.1;
        },
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
