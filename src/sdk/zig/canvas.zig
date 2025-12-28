// ============================================================================
// Canvas - Pure Zig Drawing Implementation
// ============================================================================
// All rendering code in one place, compiled into WASM apps.
// The framebuffer is owned here; platform reads it via exported function.

const std = @import("std");

// ============================================================================
// Constants
// ============================================================================

pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 64;
pub const FRAMEBUFFER_SIZE: u32 = SCREEN_WIDTH * SCREEN_HEIGHT;

// ============================================================================
// Types
// ============================================================================

pub const Color = enum(u32) {
    black = 0,
    white = 1,
    xor = 2,
};

pub const Font = enum(u32) {
    primary = 0, // 6x10 (uses 5x7 glyphs)
    secondary = 1, // 5x7
    keyboard = 2, // 5x8 (uses 5x7 glyphs)
    big_numbers = 3, // 10x20 (uses 5x7 glyphs scaled)
};

// ============================================================================
// Framebuffer - Owned by canvas, exported to platform
// ============================================================================
// Platform reads this via canvas_get_framebuffer() export.
// Each byte is a pixel: 0=black, 1=white (not packed bits).

var framebuffer: [FRAMEBUFFER_SIZE]u8 = [_]u8{0} ** FRAMEBUFFER_SIZE;

/// Export framebuffer pointer for platform to read (WASM export)
export fn canvas_get_framebuffer() [*]u8 {
    return &framebuffer;
}

/// Export framebuffer size for platform
export fn canvas_get_framebuffer_size() u32 {
    return FRAMEBUFFER_SIZE;
}

// ============================================================================
// State
// ============================================================================

var current_color: Color = .white;
var current_font: Font = .primary;

// ============================================================================
// Framebuffer Access
// ============================================================================

pub fn width() u32 {
    return SCREEN_WIDTH;
}

pub fn height() u32 {
    return SCREEN_HEIGHT;
}

// ============================================================================
// Basic Drawing
// ============================================================================

pub fn clear() void {
    @memset(&framebuffer, 0);
}

pub fn setColor(color: Color) void {
    current_color = color;
}

pub fn setFont(font: Font) void {
    current_font = font;
}

fn setPixel(x: i32, y: i32) void {
    if (x < 0 or y < 0) return;
    const ux: u32 = @intCast(x);
    const uy: u32 = @intCast(y);
    if (ux >= SCREEN_WIDTH or uy >= SCREEN_HEIGHT) return;

    const idx = uy * SCREEN_WIDTH + ux;

    switch (current_color) {
        .black => framebuffer[idx] = 0,
        .white => framebuffer[idx] = 1,
        .xor => framebuffer[idx] ^= 1,
    }
}

pub fn drawDot(x: i32, y: i32) void {
    setPixel(x, y);
}

// ============================================================================
// Line Drawing (Bresenham's algorithm)
// ============================================================================

pub fn drawLine(x1: i32, y1: i32, x2: i32, y2: i32) void {
    var px = x1;
    var py = y1;
    const dx = if (x2 > x1) x2 - x1 else x1 - x2;
    const dy = if (y2 > y1) y2 - y1 else y1 - y2;
    const sx: i32 = if (x1 < x2) 1 else -1;
    const sy: i32 = if (y1 < y2) 1 else -1;
    var err = dx - dy;

    while (true) {
        setPixel(px, py);
        if (px == x2 and py == y2) break;
        const e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            px += sx;
        }
        if (e2 < dx) {
            err += dx;
            py += sy;
        }
    }
}

// ============================================================================
// Rectangle Drawing
// ============================================================================

pub fn drawFrame(x: i32, y: i32, w: u32, h: u32) void {
    const wi: i32 = @intCast(w);
    const hi: i32 = @intCast(h);

    // Top and bottom
    var i: i32 = 0;
    while (i < wi) : (i += 1) {
        setPixel(x + i, y);
        setPixel(x + i, y + hi - 1);
    }
    // Left and right
    i = 0;
    while (i < hi) : (i += 1) {
        setPixel(x, y + i);
        setPixel(x + wi - 1, y + i);
    }
}

pub fn drawBox(x: i32, y: i32, w: u32, h: u32) void {
    const wi: i32 = @intCast(w);
    const hi: i32 = @intCast(h);

    var py: i32 = 0;
    while (py < hi) : (py += 1) {
        var px: i32 = 0;
        while (px < wi) : (px += 1) {
            setPixel(x + px, y + py);
        }
    }
}

// ============================================================================
// Rounded Rectangle Drawing
// ============================================================================

fn drawCornerArc(cx: i32, cy: i32, r: u32, corner: u2) void {
    var px: i32 = @intCast(r);
    var py: i32 = 0;
    var err: i32 = 0;

    while (px >= py) {
        switch (corner) {
            0 => { // top-left
                setPixel(cx - px, cy - py);
                setPixel(cx - py, cy - px);
            },
            1 => { // top-right
                setPixel(cx + px, cy - py);
                setPixel(cx + py, cy - px);
            },
            2 => { // bottom-left
                setPixel(cx - px, cy + py);
                setPixel(cx - py, cy + px);
            },
            3 => { // bottom-right
                setPixel(cx + px, cy + py);
                setPixel(cx + py, cy + px);
            },
        }
        py += 1;
        if (err <= 0) {
            err += 2 * py + 1;
        }
        if (err > 0) {
            px -= 1;
            err -= 2 * px + 1;
        }
    }
}

fn fillCorner(cx: i32, cy: i32, r: u32, corner: u2) void {
    const ri: i32 = @intCast(r);
    var dy: i32 = 0;
    while (dy <= ri) : (dy += 1) {
        const dy_sq = dy * dy;
        const r_sq = ri * ri;
        // Calculate x extent: sqrt(r^2 - dy^2)
        var dx: i32 = 0;
        while ((dx + 1) * (dx + 1) <= r_sq - dy_sq) {
            dx += 1;
        }
        var i: i32 = 0;
        while (i <= dx) : (i += 1) {
            switch (corner) {
                0 => setPixel(cx - i, cy - dy),
                1 => setPixel(cx + i, cy - dy),
                2 => setPixel(cx - i, cy + dy),
                3 => setPixel(cx + i, cy + dy),
            }
        }
    }
}

pub fn drawRFrame(x: i32, y: i32, w: u32, h: u32, r: u32) void {
    const wi: i32 = @intCast(w);
    const hi: i32 = @intCast(h);
    const ri: i32 = @intCast(r);

    if (r == 0 or r > @min(w, h) / 2) {
        drawFrame(x, y, w, h);
        return;
    }

    // Top and bottom (excluding corners)
    var i: i32 = ri;
    while (i < wi - ri) : (i += 1) {
        setPixel(x + i, y);
        setPixel(x + i, y + hi - 1);
    }
    // Left and right (excluding corners)
    i = ri;
    while (i < hi - ri) : (i += 1) {
        setPixel(x, y + i);
        setPixel(x + wi - 1, y + i);
    }
    // Corner arcs
    drawCornerArc(x + ri, y + ri, r, 0);
    drawCornerArc(x + wi - 1 - ri, y + ri, r, 1);
    drawCornerArc(x + ri, y + hi - 1 - ri, r, 2);
    drawCornerArc(x + wi - 1 - ri, y + hi - 1 - ri, r, 3);
}

pub fn drawRBox(x: i32, y: i32, w: u32, h: u32, r: u32) void {
    const wi: i32 = @intCast(w);
    const hi: i32 = @intCast(h);
    const ri: i32 = @intCast(r);

    if (r == 0 or r > @min(w, h) / 2) {
        drawBox(x, y, w, h);
        return;
    }

    // Center rectangle
    drawBox(x, y + ri, w, h - 2 * r);
    // Top and bottom rectangles (excluding corners)
    drawBox(x + ri, y, w - 2 * r, r);
    drawBox(x + ri, y + hi - ri, w - 2 * r, r);
    // Fill corners
    fillCorner(x + ri, y + ri, r, 0);
    fillCorner(x + wi - 1 - ri, y + ri, r, 1);
    fillCorner(x + ri, y + hi - 1 - ri, r, 2);
    fillCorner(x + wi - 1 - ri, y + hi - 1 - ri, r, 3);
}

// ============================================================================
// Circle Drawing (Midpoint algorithm)
// ============================================================================

pub fn drawCircle(cx: i32, cy: i32, r: u32) void {
    if (r == 0) {
        setPixel(cx, cy);
        return;
    }

    var px: i32 = @intCast(r);
    var py: i32 = 0;
    var err: i32 = 0;

    while (px >= py) {
        setPixel(cx + px, cy + py);
        setPixel(cx + py, cy + px);
        setPixel(cx - py, cy + px);
        setPixel(cx - px, cy + py);
        setPixel(cx - px, cy - py);
        setPixel(cx - py, cy - px);
        setPixel(cx + py, cy - px);
        setPixel(cx + px, cy - py);

        py += 1;
        if (err <= 0) {
            err += 2 * py + 1;
        }
        if (err > 0) {
            px -= 1;
            err -= 2 * px + 1;
        }
    }
}

pub fn drawDisc(cx: i32, cy: i32, r: u32) void {
    if (r == 0) {
        setPixel(cx, cy);
        return;
    }

    const ri: i32 = @intCast(r);

    // Draw horizontal scanlines
    var dy: i32 = -ri;
    while (dy <= ri) : (dy += 1) {
        const dy_sq = dy * dy;
        const r_sq = ri * ri;
        // Calculate x extent
        var dx: i32 = 0;
        while ((dx + 1) * (dx + 1) <= r_sq - dy_sq) {
            dx += 1;
        }
        // Draw horizontal line
        var px: i32 = cx - dx;
        while (px <= cx + dx) : (px += 1) {
            setPixel(px, cy + dy);
        }
    }
}

// ============================================================================
// Font Data - 5x7 Bitmap Font
// ============================================================================
// Each character is 5 columns, 7 rows
// Stored as 5 bytes per character (one byte per column, LSB at top)

const FONT_5X7_WIDTH: u32 = 5;
const FONT_5X7_HEIGHT: u32 = 7;
const FONT_5X7_BASELINE: u32 = 6;

// ASCII 32-126 (95 characters)
const font_5x7_data = [95][5]u8{
    .{ 0x00, 0x00, 0x00, 0x00, 0x00 }, // 32 space
    .{ 0x00, 0x00, 0x5F, 0x00, 0x00 }, // 33 !
    .{ 0x00, 0x07, 0x00, 0x07, 0x00 }, // 34 "
    .{ 0x14, 0x7F, 0x14, 0x7F, 0x14 }, // 35 #
    .{ 0x24, 0x2A, 0x7F, 0x2A, 0x12 }, // 36 $
    .{ 0x23, 0x13, 0x08, 0x64, 0x62 }, // 37 %
    .{ 0x36, 0x49, 0x55, 0x22, 0x50 }, // 38 &
    .{ 0x00, 0x05, 0x03, 0x00, 0x00 }, // 39 '
    .{ 0x00, 0x1C, 0x22, 0x41, 0x00 }, // 40 (
    .{ 0x00, 0x41, 0x22, 0x1C, 0x00 }, // 41 )
    .{ 0x08, 0x2A, 0x1C, 0x2A, 0x08 }, // 42 *
    .{ 0x08, 0x08, 0x3E, 0x08, 0x08 }, // 43 +
    .{ 0x00, 0x50, 0x30, 0x00, 0x00 }, // 44 ,
    .{ 0x08, 0x08, 0x08, 0x08, 0x08 }, // 45 -
    .{ 0x00, 0x60, 0x60, 0x00, 0x00 }, // 46 .
    .{ 0x20, 0x10, 0x08, 0x04, 0x02 }, // 47 /
    .{ 0x3E, 0x51, 0x49, 0x45, 0x3E }, // 48 0
    .{ 0x00, 0x42, 0x7F, 0x40, 0x00 }, // 49 1
    .{ 0x42, 0x61, 0x51, 0x49, 0x46 }, // 50 2
    .{ 0x21, 0x41, 0x45, 0x4B, 0x31 }, // 51 3
    .{ 0x18, 0x14, 0x12, 0x7F, 0x10 }, // 52 4
    .{ 0x27, 0x45, 0x45, 0x45, 0x39 }, // 53 5
    .{ 0x3C, 0x4A, 0x49, 0x49, 0x30 }, // 54 6
    .{ 0x01, 0x71, 0x09, 0x05, 0x03 }, // 55 7
    .{ 0x36, 0x49, 0x49, 0x49, 0x36 }, // 56 8
    .{ 0x06, 0x49, 0x49, 0x29, 0x1E }, // 57 9
    .{ 0x00, 0x36, 0x36, 0x00, 0x00 }, // 58 :
    .{ 0x00, 0x56, 0x36, 0x00, 0x00 }, // 59 ;
    .{ 0x00, 0x08, 0x14, 0x22, 0x41 }, // 60 <
    .{ 0x14, 0x14, 0x14, 0x14, 0x14 }, // 61 =
    .{ 0x41, 0x22, 0x14, 0x08, 0x00 }, // 62 >
    .{ 0x02, 0x01, 0x51, 0x09, 0x06 }, // 63 ?
    .{ 0x32, 0x49, 0x79, 0x41, 0x3E }, // 64 @
    .{ 0x7E, 0x11, 0x11, 0x11, 0x7E }, // 65 A
    .{ 0x7F, 0x49, 0x49, 0x49, 0x36 }, // 66 B
    .{ 0x3E, 0x41, 0x41, 0x41, 0x22 }, // 67 C
    .{ 0x7F, 0x41, 0x41, 0x22, 0x1C }, // 68 D
    .{ 0x7F, 0x49, 0x49, 0x49, 0x41 }, // 69 E
    .{ 0x7F, 0x09, 0x09, 0x01, 0x01 }, // 70 F
    .{ 0x3E, 0x41, 0x41, 0x51, 0x32 }, // 71 G
    .{ 0x7F, 0x08, 0x08, 0x08, 0x7F }, // 72 H
    .{ 0x00, 0x41, 0x7F, 0x41, 0x00 }, // 73 I
    .{ 0x20, 0x40, 0x41, 0x3F, 0x01 }, // 74 J
    .{ 0x7F, 0x08, 0x14, 0x22, 0x41 }, // 75 K
    .{ 0x7F, 0x40, 0x40, 0x40, 0x40 }, // 76 L
    .{ 0x7F, 0x02, 0x04, 0x02, 0x7F }, // 77 M
    .{ 0x7F, 0x04, 0x08, 0x10, 0x7F }, // 78 N
    .{ 0x3E, 0x41, 0x41, 0x41, 0x3E }, // 79 O
    .{ 0x7F, 0x09, 0x09, 0x09, 0x06 }, // 80 P
    .{ 0x3E, 0x41, 0x51, 0x21, 0x5E }, // 81 Q
    .{ 0x7F, 0x09, 0x19, 0x29, 0x46 }, // 82 R
    .{ 0x46, 0x49, 0x49, 0x49, 0x31 }, // 83 S
    .{ 0x01, 0x01, 0x7F, 0x01, 0x01 }, // 84 T
    .{ 0x3F, 0x40, 0x40, 0x40, 0x3F }, // 85 U
    .{ 0x1F, 0x20, 0x40, 0x20, 0x1F }, // 86 V
    .{ 0x7F, 0x20, 0x18, 0x20, 0x7F }, // 87 W
    .{ 0x63, 0x14, 0x08, 0x14, 0x63 }, // 88 X
    .{ 0x03, 0x04, 0x78, 0x04, 0x03 }, // 89 Y
    .{ 0x61, 0x51, 0x49, 0x45, 0x43 }, // 90 Z
    .{ 0x00, 0x00, 0x7F, 0x41, 0x41 }, // 91 [
    .{ 0x02, 0x04, 0x08, 0x10, 0x20 }, // 92 backslash
    .{ 0x41, 0x41, 0x7F, 0x00, 0x00 }, // 93 ]
    .{ 0x04, 0x02, 0x01, 0x02, 0x04 }, // 94 ^
    .{ 0x40, 0x40, 0x40, 0x40, 0x40 }, // 95 _
    .{ 0x00, 0x01, 0x02, 0x04, 0x00 }, // 96 `
    .{ 0x20, 0x54, 0x54, 0x54, 0x78 }, // 97 a
    .{ 0x7F, 0x48, 0x44, 0x44, 0x38 }, // 98 b
    .{ 0x38, 0x44, 0x44, 0x44, 0x20 }, // 99 c
    .{ 0x38, 0x44, 0x44, 0x48, 0x7F }, // 100 d
    .{ 0x38, 0x54, 0x54, 0x54, 0x18 }, // 101 e
    .{ 0x08, 0x7E, 0x09, 0x01, 0x02 }, // 102 f
    .{ 0x08, 0x14, 0x54, 0x54, 0x3C }, // 103 g
    .{ 0x7F, 0x08, 0x04, 0x04, 0x78 }, // 104 h
    .{ 0x00, 0x44, 0x7D, 0x40, 0x00 }, // 105 i
    .{ 0x20, 0x40, 0x44, 0x3D, 0x00 }, // 106 j
    .{ 0x00, 0x7F, 0x10, 0x28, 0x44 }, // 107 k
    .{ 0x00, 0x41, 0x7F, 0x40, 0x00 }, // 108 l
    .{ 0x7C, 0x04, 0x18, 0x04, 0x78 }, // 109 m
    .{ 0x7C, 0x08, 0x04, 0x04, 0x78 }, // 110 n
    .{ 0x38, 0x44, 0x44, 0x44, 0x38 }, // 111 o
    .{ 0x7C, 0x14, 0x14, 0x14, 0x08 }, // 112 p
    .{ 0x08, 0x14, 0x14, 0x18, 0x7C }, // 113 q
    .{ 0x7C, 0x08, 0x04, 0x04, 0x08 }, // 114 r
    .{ 0x48, 0x54, 0x54, 0x54, 0x20 }, // 115 s
    .{ 0x04, 0x3F, 0x44, 0x40, 0x20 }, // 116 t
    .{ 0x3C, 0x40, 0x40, 0x20, 0x7C }, // 117 u
    .{ 0x1C, 0x20, 0x40, 0x20, 0x1C }, // 118 v
    .{ 0x3C, 0x40, 0x30, 0x40, 0x3C }, // 119 w
    .{ 0x44, 0x28, 0x10, 0x28, 0x44 }, // 120 x
    .{ 0x0C, 0x50, 0x50, 0x50, 0x3C }, // 121 y
    .{ 0x44, 0x64, 0x54, 0x4C, 0x44 }, // 122 z
    .{ 0x00, 0x08, 0x36, 0x41, 0x00 }, // 123 {
    .{ 0x00, 0x00, 0x7F, 0x00, 0x00 }, // 124 |
    .{ 0x00, 0x41, 0x36, 0x08, 0x00 }, // 125 }
    .{ 0x10, 0x08, 0x08, 0x10, 0x08 }, // 126 ~
};

fn getGlyph(char_code: u8) ?*const [5]u8 {
    if (char_code < 32 or char_code > 126) return null;
    return &font_5x7_data[char_code - 32];
}

fn getFontMetrics() struct { char_width: u32, spacing: u32, baseline: u32 } {
    // char_width = glyph width (5) + spacing (1) = 6 for all fonts
    return switch (current_font) {
        .primary => .{ .char_width = 6, .spacing = 1, .baseline = 8 },
        .secondary => .{ .char_width = 6, .spacing = 1, .baseline = 6 },
        .keyboard => .{ .char_width = 6, .spacing = 1, .baseline = 7 },
        .big_numbers => .{ .char_width = 6, .spacing = 1, .baseline = 8 },
    };
}

// ============================================================================
// Text Rendering
// ============================================================================

fn drawChar(x: i32, y: i32, char_code: u8) void {
    const glyph = getGlyph(char_code) orelse {
        // Draw placeholder box for unknown characters
        const metrics = getFontMetrics();
        const old_color = current_color;
        current_color = .white;
        drawFrame(x, y - @as(i32, @intCast(metrics.baseline)) + 1, 5, 7);
        current_color = old_color;
        return;
    };

    const metrics = getFontMetrics();
    const y_offset = y - @as(i32, @intCast(metrics.baseline));

    // Draw the glyph (5 columns, 7 rows)
    for (glyph, 0..) |col_data, col| {
        var row: u3 = 0;
        while (row < 7) : (row += 1) {
            if (col_data & (@as(u8, 1) << row) != 0) {
                setPixel(x + @as(i32, @intCast(col)), y_offset + @as(i32, row));
            }
        }
    }
}

pub fn drawStr(x: i32, y: i32, str: []const u8) void {
    const metrics = getFontMetrics();
    const char_width: i32 = @intCast(metrics.char_width);

    var cur_x = x;
    for (str) |c| {
        drawChar(cur_x, y, c);
        cur_x += char_width;
    }
}

pub fn drawStrZ(x: i32, y: i32, str: [*:0]const u8) void {
    const metrics = getFontMetrics();
    const char_width: i32 = @intCast(metrics.char_width);

    var cur_x = x;
    var i: usize = 0;
    while (str[i] != 0) : (i += 1) {
        drawChar(cur_x, y, str[i]);
        cur_x += char_width;
    }
}

pub fn stringWidth(str: []const u8) u32 {
    const metrics = getFontMetrics();
    return @intCast(str.len * metrics.char_width);
}

pub fn stringWidthZ(str: [*:0]const u8) u32 {
    const metrics = getFontMetrics();
    var len: u32 = 0;
    var i: usize = 0;
    while (str[i] != 0) : (i += 1) {
        len += 1;
    }
    return len * metrics.char_width;
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Draw text centered horizontally on screen
pub fn drawStrCentered(y: i32, str: []const u8) void {
    const w = stringWidth(str);
    const x: i32 = @intCast((SCREEN_WIDTH - w) / 2);
    drawStr(x, y, str);
}

/// Draw text right-aligned
pub fn drawStrRight(x: i32, y: i32, str: []const u8) void {
    const w = stringWidth(str);
    drawStr(x - @as(i32, @intCast(w)), y, str);
}

/// Get raw framebuffer slice (for advanced use)
pub fn getFramebuffer() []u8 {
    return &framebuffer;
}
