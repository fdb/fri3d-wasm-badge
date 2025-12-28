// ============================================================================
// Test UI Demo - Pure Zig IMGUI Demo
// ============================================================================

const platform = @import("platform");
const imgui = @import("imgui");

// App state
var current_screen: Screen = .main_menu;
var menu_scroll: i16 = 0;
var settings_scroll: i16 = 0;
var wifi_enabled: bool = true;
var bluetooth_enabled: bool = false;
var progress_value: f32 = 0.0;
var counter: i32 = 0;

const Screen = enum {
    main_menu,
    buttons_demo,
    progress_demo,
    settings,
    about,
};

// ============================================================================
// Main Menu
// ============================================================================

fn renderMainMenu() void {
    imgui.menuBegin(&menu_scroll, 5, 5);

    if (imgui.menuItem("Buttons Demo", 0)) {
        current_screen = .buttons_demo;
    }
    if (imgui.menuItem("Progress Demo", 1)) {
        current_screen = .progress_demo;
    }
    if (imgui.menuItem("Settings", 2)) {
        current_screen = .settings;
    }
    if (imgui.menuItem("About", 3)) {
        current_screen = .about;
    }
    if (imgui.menuItem("Exit", 4)) {
        platform.app.exitToLauncher();
    }

    imgui.menuEnd();
}

// ============================================================================
// Buttons Demo
// ============================================================================

fn renderButtonsDemo() void {
    imgui.spacer(8);

    // Counter display
    var buf: [16]u8 = undefined;
    const counter_str = formatInt(&buf, counter);
    imgui.label(counter_str, .secondary, .center);

    imgui.spacer(4);

    // Row of buttons
    imgui.hstackCentered(8);
    if (imgui.button("-")) {
        counter -= 1;
    }
    if (imgui.button("Reset")) {
        counter = 0;
    }
    if (imgui.button("+")) {
        counter += 1;
    }
    imgui.endStack();
}

// ============================================================================
// Progress Demo
// ============================================================================

fn renderProgressDemo() void {
    imgui.spacer(8);

    // Progress bar
    imgui.progress(progress_value, 0);
    imgui.spacer(4);

    // Percentage display
    var buf: [8]u8 = undefined;
    const percent: i32 = @intFromFloat(progress_value * 100.0);
    const pct_str = formatPercent(&buf, percent);
    imgui.label(pct_str, .secondary, .center);

    imgui.spacer(8);

    // Control buttons
    imgui.hstackCentered(8);
    if (imgui.button("0%")) {
        progress_value = 0.0;
    }
    if (imgui.button("+10%")) {
        progress_value = @min(1.0, progress_value + 0.1);
    }
    if (imgui.button("100%")) {
        progress_value = 1.0;
    }
    imgui.endStack();
}

// ============================================================================
// Settings
// ============================================================================

fn renderSettings() void {
    imgui.menuBegin(&settings_scroll, 5, 4);

    const wifi_val: [:0]const u8 = if (wifi_enabled) "On" else "Off";
    if (imgui.menuItemValue("WiFi", wifi_val, 0)) {
        wifi_enabled = !wifi_enabled;
    }

    const bt_val: [:0]const u8 = if (bluetooth_enabled) "On" else "Off";
    if (imgui.menuItemValue("Bluetooth", bt_val, 1)) {
        bluetooth_enabled = !bluetooth_enabled;
    }

    if (imgui.menuItemValue("Brightness", "80%", 2)) {
        // Would open brightness control
    }

    if (imgui.menuItemValue("Volume", "60%", 3)) {
        // Would open volume control
    }

    imgui.menuEnd();
}

// ============================================================================
// About Screen
// ============================================================================

fn renderAbout() void {
    imgui.spacer(12);

    imgui.label("Fri3d Badge", .primary, .center);
    imgui.spacer(2);
    imgui.label("IMGUI Demo", .secondary, .center);
    imgui.spacer(2);
    imgui.label("Pure Zig!", .secondary, .center);

    imgui.spacer(8);

    if (imgui.button("OK")) {
        current_screen = .main_menu;
    }
}

// ============================================================================
// WASM Exports - called by platform runtime
// ============================================================================

export fn render() void {
    imgui.begin();

    switch (current_screen) {
        .main_menu => renderMainMenu(),
        .buttons_demo => renderButtonsDemo(),
        .progress_demo => renderProgressDemo(),
        .settings => renderSettings(),
        .about => renderAbout(),
    }

    // Handle global back button
    if (imgui.backPressed()) {
        if (current_screen != .main_menu) {
            current_screen = .main_menu;
        } else {
            // At main menu, exit to launcher
            platform.app.exitToLauncher();
        }
    }

    imgui.end();
}

export fn on_input(key: u32, input_type: u32) void {
    const k: platform.InputKey = @enumFromInt(key);
    const t: platform.InputType = @enumFromInt(input_type);
    imgui.input(k, t);
}

// Scene API (for visual testing)
export fn get_scene() u32 {
    return 0;
}

export fn set_scene(_: u32) void {}

export fn get_scene_count() u32 {
    return 1;
}

// ============================================================================
// Helper Functions
// ============================================================================

fn formatInt(buf: *[16]u8, value: i32) [:0]const u8 {
    var v = value;
    var neg = false;
    if (v < 0) {
        neg = true;
        v = -v;
    }

    var i: usize = buf.len - 1;
    buf[i] = 0;
    i -= 1;

    if (v == 0) {
        buf[i] = '0';
        return buf[i .. buf.len - 1 :0];
    }

    while (v > 0 and i > 0) {
        buf[i] = @intCast('0' + @as(u8, @intCast(@mod(v, 10))));
        v = @divTrunc(v, 10);
        i -= 1;
    }

    if (neg and i > 0) {
        buf[i] = '-';
        return buf[i .. buf.len - 1 :0];
    }

    return buf[i + 1 .. buf.len - 1 :0];
}

fn formatPercent(buf: *[8]u8, value: i32) [:0]const u8 {
    var v = value;
    if (v < 0) v = 0;
    if (v > 100) v = 100;

    var i: usize = buf.len - 1;
    buf[i] = 0;
    i -= 1;
    buf[i] = '%';
    i -= 1;

    if (v == 0) {
        buf[i] = '0';
        return buf[i .. buf.len - 1 :0];
    }

    while (v > 0 and i > 0) {
        buf[i] = @intCast('0' + @as(u8, @intCast(@mod(v, 10))));
        v = @divTrunc(v, 10);
        i -= 1;
    }

    return buf[i + 1 .. buf.len - 1 :0];
}
