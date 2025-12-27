const std = @import("std");
const platform = @import("../../sdk/platform.zig");

var frame: u32 = 0;

fn drawMovingCircle() void {
    const dims = platform.canvas_size();
    const radius: u32 = 10;

    // Move horizontally across the screen and wrap around.
    const x_pos = @as(i32, @intCast(frame % @max(dims.width, 1)));
    const y_pos = @as(i32, @intCast(dims.height / 2));

    platform.canvas_set_color(1);
    platform.canvas_draw_disc(x_pos, y_pos, radius);
}

pub export fn render() void {
    platform.canvas_clear();

    drawMovingCircle();

    const dims = platform.canvas_size();
    const label = "Hello from Zig!\x00";
    const text_x: i32 = 2;
    const text_y: i32 = @as(i32, @intCast(dims.height)) - 6;
    platform.canvas_set_color(1);
    platform.canvas_draw_str(text_x, text_y, label);

    frame +%= 1;
}

pub export fn on_input(key: u32, event_type: u32) void {
    _ = key;
    _ = event_type;
    // This demo app doesn't handle input yet, but the export is present
    // so that ports can verify the call path.
}
