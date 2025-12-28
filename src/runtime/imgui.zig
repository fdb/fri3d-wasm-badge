// ============================================================================
// IMGUI View System for ESP32 128x64 Monochrome Display
// Immediate Mode GUI - no persistent widgets, render and query in same call
// ============================================================================

const platform = @import("platform");
const canvas = platform.canvas;
// Use types from canvas.zig (where the drawing functions expect them)
const Color = canvas.Color;
const Font = canvas.Font;
// Input types are in platform.zig
const InputKey = platform.InputKey;
const InputType = platform.InputType;

// ============================================================================
// Constants
// ============================================================================

pub const SCREEN_WIDTH: i16 = 128;
pub const SCREEN_HEIGHT: i16 = 64;

const MAX_LAYOUT_DEPTH: usize = 8;
const MAX_FOCUSABLE: usize = 32;
const MAX_DEFERRED: usize = 16;

const FONT_HEIGHT_PRIMARY: i16 = 12;
const FONT_HEIGHT_SECONDARY: i16 = 10;
const BUTTON_PADDING_X: i16 = 4;
const BUTTON_PADDING_Y: i16 = 2;
const MENU_ITEM_HEIGHT: i16 = 12;
const FOOTER_HEIGHT: i16 = 12;
const SCROLLBAR_WIDTH: i16 = 3;

// ============================================================================
// Types
// ============================================================================

pub const Align = enum {
    left,
    right,
    center,
};

pub const LayoutDirection = enum {
    vertical,
    horizontal,
};

const LayoutStack = struct {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    direction: LayoutDirection,
    spacing: i16,
    cursor: i16, // Current position in layout direction
    centered: bool, // For hstacks: defer drawing until end_stack for centering
};

const DeferredButton = struct {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    text: [:0]const u8,
    focused: bool,
};

const MenuState = struct {
    active: bool,
    scroll: ?*i16,
    visible: i16,
    total: i16,
    y_start: i16,
    first_item_focus: i16, // Focus index of first menu item
};

const Context = struct {
    // Layout stack
    layout_stack: [MAX_LAYOUT_DEPTH]LayoutStack,
    layout_depth: i8,

    // Focus tracking
    focus_index: i16, // Currently focused widget (-1 = none)
    focus_count: i16, // Focusable widgets this frame
    prev_focus_count: i16, // Focus count from previous frame

    // Input state for current frame
    last_key: InputKey,
    last_type: InputType,
    has_input: bool,
    ok_pressed: bool,
    back_pressed: bool,

    // Menu state
    menu: MenuState,

    // Absolute position override
    use_absolute: bool,
    abs_x: i16,
    abs_y: i16,

    // Deferred drawing for centered hstacks
    deferred_buttons: [MAX_DEFERRED]DeferredButton,
    deferred_count: i8,
};

// ============================================================================
// Global State
// ============================================================================

var g_ctx = Context{
    .layout_stack = undefined,
    .layout_depth = -1,
    .focus_index = 0,
    .focus_count = 0,
    .prev_focus_count = 0,
    .last_key = .up,
    .last_type = .press,
    .has_input = false,
    .ok_pressed = false,
    .back_pressed = false,
    .menu = .{
        .active = false,
        .scroll = null,
        .visible = 0,
        .total = 0,
        .y_start = 0,
        .first_item_focus = 0,
    },
    .use_absolute = false,
    .abs_x = 0,
    .abs_y = 0,
    .deferred_buttons = undefined,
    .deferred_count = 0,
};

// ============================================================================
// Internal Helpers
// ============================================================================

fn getFontHeight(font: Font) i16 {
    return switch (font) {
        .primary => FONT_HEIGHT_PRIMARY,
        else => FONT_HEIGHT_SECONDARY,
    };
}

fn currentLayout() ?*LayoutStack {
    if (g_ctx.layout_depth < 0) return null;
    return &g_ctx.layout_stack[@intCast(g_ctx.layout_depth)];
}

const LayoutResult = struct {
    x: i16,
    y: i16,
    w: i16,
};

fn layoutNext(width: i16, height: i16) LayoutResult {
    // Check for absolute position override
    if (g_ctx.use_absolute) {
        const result = LayoutResult{
            .x = g_ctx.abs_x,
            .y = g_ctx.abs_y,
            .w = width,
        };
        g_ctx.use_absolute = false;
        return result;
    }

    const layout = currentLayout() orelse return .{
        .x = 0,
        .y = 0,
        .w = SCREEN_WIDTH,
    };

    if (layout.direction == .vertical) {
        const result = LayoutResult{
            .x = layout.x,
            .y = layout.y + layout.cursor,
            .w = layout.width,
        };
        layout.cursor += height + layout.spacing;
        return result;
    } else {
        const result = LayoutResult{
            .x = layout.x + layout.cursor,
            .y = layout.y,
            .w = width,
        };
        layout.cursor += width + layout.spacing;
        return result;
    }
}

fn registerFocusable() i16 {
    if (g_ctx.focus_count >= MAX_FOCUSABLE) {
        return -1;
    }
    const idx = g_ctx.focus_count;
    g_ctx.focus_count += 1;
    return idx;
}

fn checkFocused(widget_idx: i16) bool {
    return g_ctx.focus_index == widget_idx;
}

fn checkActivated(widget_idx: i16) bool {
    return checkFocused(widget_idx) and g_ctx.ok_pressed;
}

fn inCenteredHstack() bool {
    const layout = currentLayout() orelse return false;
    return layout.direction == .horizontal and layout.centered;
}

fn drawButtonInternal(x: i16, y: i16, w: i16, h: i16, text: [:0]const u8, focused: bool) void {
    canvas.setFont(.secondary);
    if (focused) {
        canvas.setColor(.white);
        canvas.drawRBox(x, y, @intCast(w), @intCast(h), 2);
        canvas.setColor(.black);
        canvas.drawStr(x + BUTTON_PADDING_X, y + h - BUTTON_PADDING_Y, text);
    } else {
        canvas.setColor(.white);
        canvas.drawRFrame(x, y, @intCast(w), @intCast(h), 2);
        canvas.drawStr(x + BUTTON_PADDING_X, y + h - BUTTON_PADDING_Y, text);
    }
}

// ============================================================================
// Context Management
// ============================================================================

/// Begin a new frame - clears canvas, resets widget state
pub fn begin() void {
    // Clear canvas
    canvas.clear();

    // Save previous focus count for navigation bounds
    g_ctx.prev_focus_count = g_ctx.focus_count;

    // Reset widget counter
    g_ctx.focus_count = 0;

    // Reset layout to default full-screen vertical
    g_ctx.layout_depth = 0;
    g_ctx.layout_stack[0] = .{
        .x = 0,
        .y = 0,
        .width = SCREEN_WIDTH,
        .height = SCREEN_HEIGHT,
        .direction = .vertical,
        .spacing = 0,
        .cursor = 0,
        .centered = false,
    };

    // Reset menu state
    g_ctx.menu.active = false;
    g_ctx.menu.scroll = null;

    // Reset position override
    g_ctx.use_absolute = false;

    // Reset deferred drawing
    g_ctx.deferred_count = 0;
}

/// End the frame - commits canvas buffer to display
pub fn end() void {
    // Clamp focus to valid range
    if (g_ctx.focus_count > 0) {
        if (g_ctx.focus_index < 0) {
            g_ctx.focus_index = 0;
        } else if (g_ctx.focus_index >= g_ctx.focus_count) {
            g_ctx.focus_index = g_ctx.focus_count - 1;
        }
    } else {
        g_ctx.focus_index = -1;
    }

    // Clear input state for next frame
    g_ctx.has_input = false;
    g_ctx.ok_pressed = false;
    g_ctx.back_pressed = false;
}

/// Feed an input event to the UI system (call from on_input)
pub fn input(key: InputKey, input_type: InputType) void {
    g_ctx.last_key = key;
    g_ctx.last_type = input_type;
    g_ctx.has_input = true;

    // Handle navigation on short press or repeat
    // Navigate on short_press (deliberate tap) or repeat (held key for scrolling)
    // Don't use press - that's for immediate visual feedback or games
    const is_nav = input_type == .short_press or input_type == .repeat;
    if (is_nav) {
        switch (key) {
            .up => {
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index -= 1;
                    if (g_ctx.focus_index < 0) {
                        g_ctx.focus_index = g_ctx.prev_focus_count - 1;
                    }
                }
            },
            .down => {
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index += 1;
                    if (g_ctx.focus_index >= g_ctx.prev_focus_count) {
                        g_ctx.focus_index = 0;
                    }
                }
            },
            .left => {
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index -= 1;
                    if (g_ctx.focus_index < 0) {
                        g_ctx.focus_index = g_ctx.prev_focus_count - 1;
                    }
                }
            },
            .right => {
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index += 1;
                    if (g_ctx.focus_index >= g_ctx.prev_focus_count) {
                        g_ctx.focus_index = 0;
                    }
                }
            },
            .ok => {
                g_ctx.ok_pressed = true;
            },
            .back => {
                g_ctx.back_pressed = true;
            },
        }
    }
}

/// Check if Back button was pressed this frame
pub fn backPressed() bool {
    return g_ctx.back_pressed;
}

// ============================================================================
// Layout System
// ============================================================================

/// Begin a vertical stack with specified pixel spacing between items
pub fn vstack(spacing: i16) void {
    if (g_ctx.layout_depth >= MAX_LAYOUT_DEPTH - 1) return;

    const parent = currentLayout();
    const new_x = if (parent) |p| p.x else 0;
    const new_y = if (parent) |p| p.y + p.cursor else 0;
    const new_width = if (parent) |p| p.width else SCREEN_WIDTH;

    g_ctx.layout_depth += 1;
    g_ctx.layout_stack[@intCast(g_ctx.layout_depth)] = .{
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = SCREEN_HEIGHT - new_y,
        .direction = .vertical,
        .spacing = spacing,
        .cursor = 0,
        .centered = false,
    };
}

/// Begin a horizontal stack with specified pixel spacing between items
pub fn hstack(spacing: i16) void {
    if (g_ctx.layout_depth >= MAX_LAYOUT_DEPTH - 1) return;

    const parent = currentLayout();
    const new_x = if (parent) |p| p.x else 0;
    const new_y = if (parent) |p| p.y + p.cursor else 0;
    const new_width = if (parent) |p| p.width else SCREEN_WIDTH;

    g_ctx.layout_depth += 1;
    g_ctx.layout_stack[@intCast(g_ctx.layout_depth)] = .{
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = SCREEN_HEIGHT - new_y,
        .direction = .horizontal,
        .spacing = spacing,
        .cursor = 0,
        .centered = false,
    };
}

/// Begin a centered horizontal stack (contents will be centered in parent width)
pub fn hstackCentered(spacing: i16) void {
    if (g_ctx.layout_depth >= MAX_LAYOUT_DEPTH - 1) return;

    const parent = currentLayout();
    const new_x = if (parent) |p| p.x else 0;
    const new_y = if (parent) |p| p.y + p.cursor else 0;
    const new_width = if (parent) |p| p.width else SCREEN_WIDTH;

    // Reset deferred buffer for this centered stack
    g_ctx.deferred_count = 0;

    g_ctx.layout_depth += 1;
    g_ctx.layout_stack[@intCast(g_ctx.layout_depth)] = .{
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = SCREEN_HEIGHT - new_y,
        .direction = .horizontal,
        .spacing = spacing,
        .cursor = 0,
        .centered = true,
    };
}

/// End the current stack and return to parent layout
pub fn endStack() void {
    if (g_ctx.layout_depth <= 0) return;

    const ending = &g_ctx.layout_stack[@intCast(g_ctx.layout_depth)];

    // Handle centered hstack: draw deferred buttons with centering offset
    if (ending.centered and ending.direction == .horizontal) {
        var content_width = ending.cursor;
        if (ending.spacing > 0 and content_width > 0) {
            content_width -= ending.spacing; // Remove trailing spacing
        }

        // Calculate centering offset
        const offset = @divTrunc(ending.width - content_width, 2);

        // Draw all deferred buttons with offset
        var i: usize = 0;
        while (i < @as(usize, @intCast(g_ctx.deferred_count))) : (i += 1) {
            const btn = &g_ctx.deferred_buttons[i];
            drawButtonInternal(
                btn.x + offset,
                btn.y,
                btn.width,
                btn.height,
                btn.text,
                btn.focused,
            );
        }

        // Clear deferred buffer
        g_ctx.deferred_count = 0;
    }

    // Get the size used by this stack
    var used_size = ending.cursor;
    if (ending.spacing > 0 and used_size > 0) {
        used_size -= ending.spacing; // Remove trailing spacing
    }

    // For hstacks, track the max height (for now, assume button height)
    const used_height = if (ending.direction == .horizontal)
        FONT_HEIGHT_SECONDARY + BUTTON_PADDING_Y * 2
    else
        used_size;

    g_ctx.layout_depth -= 1;

    // Advance parent cursor by the space this stack used
    if (currentLayout()) |parent| {
        if (parent.direction == .vertical) {
            parent.cursor += used_height + parent.spacing;
        }
    }
}

/// Add empty space in the current stack direction
pub fn spacer(pixels: i16) void {
    if (currentLayout()) |layout| {
        layout.cursor += pixels;
    }
}

/// Set absolute position for next widget (bypasses layout)
pub fn setPosition(x: i16, y: i16) void {
    g_ctx.use_absolute = true;
    g_ctx.abs_x = x;
    g_ctx.abs_y = y;
}

// ============================================================================
// Basic Widgets
// ============================================================================

/// Draw a text label (not focusable)
pub fn label(text: [:0]const u8, font: Font, alignment: Align) void {
    const font_height = getFontHeight(font);
    const pos = layoutNext(0, font_height);

    // Set font
    canvas.setFont(font);
    canvas.setColor(.white);

    // Calculate text position based on alignment
    const text_width: i16 = @intCast(canvas.stringWidth(text));
    const text_x = switch (alignment) {
        .center => pos.x + @divTrunc(pos.w - text_width, 2),
        .right => pos.x + pos.w - text_width,
        .left => pos.x,
    };

    // Draw text (y is baseline, so add font height)
    canvas.drawStr(text_x, pos.y + font_height, text);
}

/// Draw a horizontal separator line
pub fn separator() void {
    const pos = layoutNext(0, 5);
    canvas.setColor(.white);
    canvas.drawLine(pos.x, pos.y + 2, pos.x + pos.w - 1, pos.y + 2);
}

/// Draw a focusable button - returns true when activated
pub fn button(text: [:0]const u8) bool {
    canvas.setFont(.secondary);
    const text_width: i16 = @intCast(canvas.stringWidth(text));

    const btn_width = text_width + BUTTON_PADDING_X * 2;
    const btn_height = FONT_HEIGHT_SECONDARY + BUTTON_PADDING_Y * 2;

    const pos = layoutNext(btn_width, btn_height);

    // Register for focus
    const idx = registerFocusable();
    const focused = checkFocused(idx);
    const activated = checkActivated(idx);

    // Check if we're in a centered hstack - defer drawing
    if (inCenteredHstack()) {
        if (g_ctx.deferred_count < MAX_DEFERRED) {
            g_ctx.deferred_buttons[@intCast(g_ctx.deferred_count)] = .{
                .x = pos.x,
                .y = pos.y,
                .width = btn_width,
                .height = btn_height,
                .text = text,
                .focused = focused,
            };
            g_ctx.deferred_count += 1;
        }
    } else {
        // Center button in layout width for vertical stacks
        const btn_x = pos.x + @divTrunc(pos.w - btn_width, 2);
        drawButtonInternal(btn_x, pos.y, btn_width, btn_height, text, focused);
    }

    return activated;
}

/// Draw a button at absolute position - returns true when activated
pub fn buttonAt(x: i16, y: i16, text: [:0]const u8) bool {
    setPosition(x, y);
    return button(text);
}

/// Draw a progress bar (value 0.0 to 1.0, width 0 = full layout width)
pub fn progress(value: f32, width: i16) void {
    const clamped_value = @max(0.0, @min(1.0, value));

    const bar_height: i16 = 8;
    const pos = layoutNext(width, bar_height);

    // Use layout width minus margins if width=0
    const bar_width = if (width > 0) width else pos.w - 8; // 4px margin each side
    const bar_x = pos.x + @divTrunc(pos.w - bar_width, 2);

    // Draw outline
    canvas.setColor(.white);
    canvas.drawFrame(bar_x, pos.y, @intCast(bar_width), @intCast(bar_height));

    // Draw filled portion
    const fill_width: i16 = @intFromFloat(clamped_value * @as(f32, @floatFromInt(bar_width - 2)));
    if (fill_width > 0) {
        canvas.drawBox(bar_x + 1, pos.y + 1, @intCast(fill_width), @intCast(bar_height - 2));
    }
}

/// Draw an XBM-format icon
pub fn icon(data: []const u8, icon_width: u8, icon_height: u8) void {
    const pos = layoutNext(@intCast(icon_width), @intCast(icon_height));

    // Center icon in layout width
    const icon_x = pos.x + @divTrunc(pos.w - @as(i16, icon_width), 2);

    // Draw XBM icon pixel by pixel
    canvas.setColor(.white);
    var iy: u8 = 0;
    while (iy < icon_height) : (iy += 1) {
        var ix: u8 = 0;
        while (ix < icon_width) : (ix += 1) {
            const byte_idx: usize = @as(usize, iy) * ((icon_width + 7) / 8) + (ix / 8);
            const bit_idx: u3 = @intCast(ix % 8);
            if (byte_idx < data.len and (data[byte_idx] & (@as(u8, 1) << bit_idx)) != 0) {
                canvas.drawDot(icon_x + ix, pos.y + iy);
            }
        }
    }
}

/// Draw a focusable checkbox - returns true when value changes
pub fn checkbox(text: [:0]const u8, checked: *bool) bool {
    canvas.setFont(.secondary);

    const box_size: i16 = 10;
    var item_height: i16 = FONT_HEIGHT_SECONDARY + 2;
    if (item_height < box_size) item_height = box_size;

    const pos = layoutNext(0, item_height);

    // Register for focus
    const idx = registerFocusable();
    const focused = checkFocused(idx);
    const activated = checkActivated(idx);

    // Toggle on activation
    if (activated) {
        checked.* = !checked.*;
    }

    // Draw checkbox
    const box_x = pos.x + 2;
    const box_y = pos.y + @divTrunc(item_height - box_size, 2);

    canvas.setColor(.white);

    if (focused) {
        // Focused: filled background
        canvas.drawBox(pos.x, pos.y, @intCast(pos.w), @intCast(item_height));
        canvas.setColor(.black);
    }

    // Draw checkbox outline
    canvas.drawFrame(box_x, box_y, @intCast(box_size), @intCast(box_size));

    // Draw checkmark if checked
    if (checked.*) {
        canvas.drawLine(box_x + 2, box_y + 5, box_x + 4, box_y + 7);
        canvas.drawLine(box_x + 4, box_y + 7, box_x + 7, box_y + 2);
    }

    // Draw label
    canvas.drawStr(box_x + box_size + 4, pos.y + item_height - 2, text);

    return activated;
}

// ============================================================================
// Menu System
// ============================================================================

/// Begin a scrollable menu
/// scroll: pointer to scroll position (persists across frames)
/// visible: number of visible items
/// total: total number of items
pub fn menuBegin(scroll: *i16, visible: i16, total: i16) void {
    g_ctx.menu.active = true;
    g_ctx.menu.scroll = scroll;
    g_ctx.menu.visible = visible;
    g_ctx.menu.total = total;
    g_ctx.menu.first_item_focus = g_ctx.focus_count;

    // Store y position where menu starts
    const layout = currentLayout();
    g_ctx.menu.y_start = if (layout) |l| l.y + l.cursor else 0;
}

/// Add a menu item - returns true when selected (OK pressed)
pub fn menuItem(text: [:0]const u8, index: i16) bool {
    if (!g_ctx.menu.active) return false;
    const scroll_ptr = g_ctx.menu.scroll orelse return false;

    // Register for focus first (all items, visible or not)
    const idx = registerFocusable();
    const focused = checkFocused(idx);
    const activated = checkActivated(idx);

    // Update scroll to keep focused item visible BEFORE visibility check
    if (focused) {
        if (index < scroll_ptr.*) {
            scroll_ptr.* = index;
        } else if (index >= scroll_ptr.* + g_ctx.menu.visible) {
            scroll_ptr.* = index - g_ctx.menu.visible + 1;
        }
    }

    // Now read scroll (may have been updated)
    const scroll = scroll_ptr.*;

    // Check if item is visible
    if (index < scroll or index >= scroll + g_ctx.menu.visible) {
        return false;
    }

    canvas.setFont(.secondary);

    // Calculate position within visible area
    const visible_index = index - scroll;
    const y = g_ctx.menu.y_start + visible_index * MENU_ITEM_HEIGHT;

    // Calculate width (leave room for scrollbar)
    const item_width = SCREEN_WIDTH - SCROLLBAR_WIDTH - 2;

    // Draw item
    canvas.setColor(.white);

    if (focused) {
        // Focused: filled background with inverted text
        canvas.drawBox(0, y, @intCast(item_width), @intCast(MENU_ITEM_HEIGHT));
        canvas.setColor(.black);
    }

    canvas.drawStr(2, y + MENU_ITEM_HEIGHT - 2, text);

    return activated;
}

/// Add a menu item with right-aligned value - returns true when selected
pub fn menuItemValue(label_text: [:0]const u8, value_text: [:0]const u8, index: i16) bool {
    if (!g_ctx.menu.active) return false;
    const scroll_ptr = g_ctx.menu.scroll orelse return false;

    // Register for focus first (all items, visible or not)
    const idx = registerFocusable();
    const focused = checkFocused(idx);
    const activated = checkActivated(idx);

    // Update scroll to keep focused item visible BEFORE visibility check
    if (focused) {
        if (index < scroll_ptr.*) {
            scroll_ptr.* = index;
        } else if (index >= scroll_ptr.* + g_ctx.menu.visible) {
            scroll_ptr.* = index - g_ctx.menu.visible + 1;
        }
    }

    // Now read scroll (may have been updated)
    const scroll = scroll_ptr.*;

    // Check if item is visible
    if (index < scroll or index >= scroll + g_ctx.menu.visible) {
        return false;
    }

    canvas.setFont(.secondary);

    // Calculate position within visible area
    const visible_index = index - scroll;
    const y = g_ctx.menu.y_start + visible_index * MENU_ITEM_HEIGHT;

    // Calculate width (leave room for scrollbar)
    const item_width = SCREEN_WIDTH - SCROLLBAR_WIDTH - 2;

    // Draw item
    canvas.setColor(.white);

    if (focused) {
        // Focused: filled background with inverted text
        canvas.drawBox(0, y, @intCast(item_width), @intCast(MENU_ITEM_HEIGHT));
        canvas.setColor(.black);
    }

    // Draw label on left
    canvas.drawStr(2, y + MENU_ITEM_HEIGHT - 2, label_text);

    // Draw value on right
    const value_width: i16 = @intCast(canvas.stringWidth(value_text));
    canvas.drawStr(item_width - value_width - 2, y + MENU_ITEM_HEIGHT - 2, value_text);

    return activated;
}

/// End menu and draw scrollbar
pub fn menuEnd() void {
    if (!g_ctx.menu.active) return;

    // Draw scrollbar if needed (Flipper Zero style: dotted track + solid thumb)
    if (g_ctx.menu.total > g_ctx.menu.visible) {
        const scroll = if (g_ctx.menu.scroll) |s| s.* else 0;
        // Scrollbar extends from menu start to bottom of screen
        const scrollbar_height = SCREEN_HEIGHT - g_ctx.menu.y_start;
        const scrollbar_x = SCREEN_WIDTH - 2; // Single pixel column near edge

        // Calculate thumb position and size
        var thumb_height = @divTrunc(scrollbar_height * g_ctx.menu.visible, g_ctx.menu.total);
        if (thumb_height < 4) thumb_height = 4;

        const thumb_y = g_ctx.menu.y_start +
            @divTrunc((scrollbar_height - thumb_height) * scroll, g_ctx.menu.total - g_ctx.menu.visible);

        canvas.setColor(.white);

        // Draw dotted track line
        var y = g_ctx.menu.y_start;
        while (y < SCREEN_HEIGHT) : (y += 2) {
            canvas.drawDot(scrollbar_x, y);
        }

        // Draw solid thumb (thicker, 3px wide)
        canvas.drawBox(scrollbar_x - 1, thumb_y, 3, @intCast(thumb_height));
    }

    // Advance layout cursor
    if (currentLayout()) |layout| {
        var visible_count = g_ctx.menu.visible;
        if (g_ctx.menu.total < visible_count) {
            visible_count = g_ctx.menu.total;
        }
        layout.cursor += visible_count * MENU_ITEM_HEIGHT + layout.spacing;
    }

    g_ctx.menu.active = false;
}

// ============================================================================
// Footer Buttons
// ============================================================================

/// Left footer hint (Left arrow + text) - purely visual, use backPressed() for back navigation
pub fn footerLeft(text: [:0]const u8) void {
    canvas.setFont(.secondary);

    const y = SCREEN_HEIGHT - FOOTER_HEIGHT;

    canvas.setColor(.white);

    // Draw left arrow
    canvas.drawLine(2, y + 5, 6, y + 2);
    canvas.drawLine(2, y + 5, 6, y + 8);

    // Draw text
    canvas.drawStr(9, y + FOOTER_HEIGHT - 2, text);
}

/// Center footer hint (OK + text) - purely visual
pub fn footerCenter(text: [:0]const u8) void {
    canvas.setFont(.secondary);

    const y = SCREEN_HEIGHT - FOOTER_HEIGHT;

    // Calculate center position
    const text_width: i16 = @intCast(canvas.stringWidth(text));
    const total_width = text_width + 12; // text + OK symbol + spacing
    const x = @divTrunc(SCREEN_WIDTH - total_width, 2);

    canvas.setColor(.white);

    // Draw OK symbol (small filled circle)
    canvas.drawDisc(x + 4, y + 5, 3);

    // Draw text
    canvas.drawStr(x + 12, y + FOOTER_HEIGHT - 2, text);
}

/// Right footer hint (Right arrow + text) - purely visual
pub fn footerRight(text: [:0]const u8) void {
    canvas.setFont(.secondary);

    const y = SCREEN_HEIGHT - FOOTER_HEIGHT;

    // Calculate position from right
    const text_width: i16 = @intCast(canvas.stringWidth(text));
    const x = SCREEN_WIDTH - text_width - 10;

    canvas.setColor(.white);

    // Draw text
    canvas.drawStr(x, y + FOOTER_HEIGHT - 2, text);

    // Draw right arrow
    const arrow_x = SCREEN_WIDTH - 7;
    canvas.drawLine(arrow_x + 4, y + 5, arrow_x, y + 2);
    canvas.drawLine(arrow_x + 4, y + 5, arrow_x, y + 8);
}

// ============================================================================
// Focus Management
// ============================================================================

/// Get current focus index (-1 if no focusable widgets)
pub fn getFocus() i16 {
    return g_ctx.focus_index;
}

/// Set focus to specific widget index
pub fn setFocus(index: i16) void {
    g_ctx.focus_index = index;
}

/// Check if specific widget index is focused
pub fn isFocused(index: i16) bool {
    return g_ctx.focus_index == index;
}
