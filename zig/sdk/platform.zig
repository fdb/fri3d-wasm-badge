pub const Key = enum(u32) {
    up = 0,
    down = 1,
    left = 2,
    right = 3,
    ok = 4,
    back = 5,
};

pub const EventType = enum(u32) {
    press = 0,
    release = 1,
    short_press = 2,
    long_press = 3,
    repeat = 4,
};

pub extern fn canvas_clear() void;
pub extern fn canvas_width() u32;
pub extern fn canvas_height() u32;
pub extern fn canvas_set_color(color: u32) void;
pub extern fn canvas_set_font(font: u32) void;
pub extern fn canvas_draw_dot(x: i32, y: i32) void;
pub extern fn canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32) void;
pub extern fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32) void;
pub extern fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32) void;
pub extern fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, r: u32) void;
pub extern fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, r: u32) void;
pub extern fn canvas_draw_circle(x: i32, y: i32, r: u32) void;
pub extern fn canvas_draw_disc(x: i32, y: i32, r: u32) void;
pub extern fn canvas_draw_str(x: i32, y: i32, str: [*:0]const u8) void;
pub extern fn canvas_string_width(str: [*:0]const u8) u32;

pub extern fn random_seed(seed: u32) void;
pub extern fn random_get() u32;
pub extern fn random_range(max: u32) u32;

pub inline fn canvas_size() struct { width: u32, height: u32 } {
    return .{ .width = canvas_width(), .height = canvas_height() };
}
