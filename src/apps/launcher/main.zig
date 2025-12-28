// ============================================================================
// Launcher - Main app selection menu
// ============================================================================
// The default app that runs when the device/emulator starts.
// Displays a list of available apps that can be launched.

const platform = @import("platform");
const imgui = @import("imgui");

// App state
var menu_scroll: i16 = 0;

// App registry - hardcoded list of available apps
// App IDs must match the order in the emulator's app list
const AppInfo = struct {
    name: [:0]const u8,
    description: [:0]const u8,
    id: u32,
};

const apps = [_]AppInfo{
    .{ .name = "Test UI", .description = "IMGUI Demo", .id = 1 },
    .{ .name = "Circles", .description = "Random circles", .id = 2 },
    .{ .name = "Mandelbrot", .description = "Fractal explorer", .id = 3 },
};

// ============================================================================
// WASM Exports - called by platform runtime
// ============================================================================

export fn render() void {
    imgui.begin();

    // App menu (full screen)
    imgui.menuBegin(&menu_scroll, 5, apps.len);

    for (apps, 0..) |app, i| {
        if (imgui.menuItem(app.name, @intCast(i))) {
            platform.app.launchApp(app.id);
        }
    }

    imgui.menuEnd();

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
