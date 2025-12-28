# Developing Apps for Fri3d Badge

This guide covers how to create apps for the Fri3d Badge platform. Apps are compiled to WebAssembly (WASM) and run on all platforms: desktop emulator, web browser, and ESP32 hardware.

## Quick Start

1. Create a new directory under `src/apps/yourapp/`
2. Create `main.zig` with required exports
3. Add your app to `build.zig`
4. Build with `zig build`
5. Test with `./zig-out/bin/fri3d_emulator zig-out/bin/yourapp.wasm`

## Minimal App Template

```zig
const platform = @import("platform");

export fn render() void {
    // Clear screen
    platform.canvas.setColor(.white);
    platform.canvas.drawBox(0, 0, platform.SCREEN_WIDTH, platform.SCREEN_HEIGHT);

    // Draw something
    platform.canvas.setColor(.black);
    platform.canvas.drawStr(10, 32, "Hello World!");
}

export fn on_input(key: u32, input_type: u32) void {
    const key_enum: platform.InputKey = @enumFromInt(key);
    const type_enum: platform.InputType = @enumFromInt(input_type);

    if (type_enum == .press and key_enum == .back) {
        platform.app.exitToLauncher();
    }
}

// Required for visual testing
export fn get_scene() u32 { return 0; }
export fn set_scene(_: u32) void {}
export fn get_scene_count() u32 { return 1; }
```

## Platform API

### Display

- Screen size: 128x64 pixels, monochrome (1-bit)
- Colors: `platform.canvas.Color.black` and `.white`
- Fonts: `.primary` (12px) and `.secondary` (10px)

### Drawing Functions

```zig
platform.canvas.clear();                          // Clear to black
platform.canvas.setColor(.white);                 // Set draw color
platform.canvas.setFont(.primary);                // Set font

// Primitives
platform.canvas.drawDot(x, y);
platform.canvas.drawLine(x1, y1, x2, y2);
platform.canvas.drawBox(x, y, w, h);              // Filled rectangle
platform.canvas.drawFrame(x, y, w, h);            // Rectangle outline
platform.canvas.drawRBox(x, y, w, h, r);          // Filled rounded rect
platform.canvas.drawRFrame(x, y, w, h, r);        // Rounded rect outline
platform.canvas.drawCircle(x, y, r);              // Circle outline
platform.canvas.drawDisc(x, y, r);                // Filled circle

// Text
platform.canvas.drawStr(x, y, "text");            // y is baseline
platform.canvas.stringWidth("text");              // Get text width
```

### Input

Keys: `up`, `down`, `left`, `right`, `ok`, `back`

Input types:
- `press` - Key just pressed
- `release` - Key released
- `short_press` - Quick tap (< 300ms)
- `long_press` - Held for >= 500ms
- `repeat` - Key held, repeating

### App Lifecycle

```zig
platform.app.launchApp(app_id);     // Launch another app by ID
platform.app.exitToLauncher();       // Return to launcher
```

### Random Numbers

```zig
platform.random.seed(42);            // Set seed
platform.random.get();               // Get random u32
platform.random.range(100);          // Get 0..99
```

## Using IMGUI

For menu-driven apps, use the IMGUI module:

```zig
const platform = @import("platform");
const imgui = @import("imgui");

var menu_scroll: i16 = 0;

export fn render() void {
    imgui.begin();

    imgui.label("My App", .primary, .center);
    imgui.spacer(4);

    imgui.menuBegin(&menu_scroll, 4, 3);
    if (imgui.menuItem("Option 1", 0)) { /* handle */ }
    if (imgui.menuItem("Option 2", 1)) { /* handle */ }
    if (imgui.menuItem("Option 3", 2)) { /* handle */ }
    imgui.menuEnd();

    if (imgui.backPressed()) {
        platform.app.exitToLauncher();
    }

    imgui.end();
}

export fn on_input(key: u32, input_type: u32) void {
    const k: platform.InputKey = @enumFromInt(key);
    const t: platform.InputType = @enumFromInt(input_type);
    imgui.input(k, t);
}
```

### IMGUI Widgets

- `label(text, font, align)` - Text label
- `button(text)` - Returns true when activated
- `checkbox(text, &checked)` - Toggle checkbox
- `progress(value, width)` - Progress bar (0.0-1.0)
- `menuItem(text, index)` - Menu item
- `menuItemValue(text, value, index)` - Menu item with right-aligned value
- `separator()` - Horizontal line
- `spacer(pixels)` - Vertical spacing

### IMGUI Layout

```zig
imgui.vstack(spacing);           // Vertical layout
imgui.hstack(spacing);           // Horizontal layout
imgui.hstackCentered(spacing);   // Centered horizontal layout
imgui.endStack();                // End layout
```

## Exit Behavior Patterns

Choose the appropriate exit behavior for your app:

### Pattern 1: Simple Apps

For apps with no navigation, exit immediately on Back:

```zig
if (key_enum == .back) {
    platform.app.exitToLauncher();
}
```

Examples: Circles

### Pattern 2: Menu-Based Apps

For apps with screens/menus, exit only from top-level:

```zig
if (imgui.backPressed()) {
    if (current_screen == .main_menu) {
        platform.app.exitToLauncher();
    } else {
        current_screen = .main_menu;
    }
}
```

Examples: Test UI

### Pattern 3: Back Used for App Function

If Back is used for in-app functionality (like zoom out), use long-press:

```zig
if (type_enum == .long_press and key_enum == .back) {
    platform.app.exitToLauncher();
}
```

Examples: Mandelbrot

### Global Exit Combo

Users can always exit by holding **LEFT + BACK for 500ms**. This is handled by the platform and works regardless of app state.

## Adding Your App to the Launcher

1. Add to `build.zig`:
   ```zig
   const yourapp = b.addExecutable(.{
       .name = "yourapp",
       .root_module = b.createModule(.{
           .root_source_file = b.path("src/apps/yourapp/main.zig"),
           .target = zig_wasm_target,
           .optimize = .ReleaseSmall,
           .imports = &.{
               .{ .name = "platform", .module = platform_module },
               // Add imgui_module if using IMGUI
           },
       }),
   });
   yourapp.rdynamic = true;
   yourapp.entry = .disabled;
   b.installArtifact(yourapp);
   ```

2. Add to default install step:
   ```zig
   b.getInstallStep().dependOn(&b.addInstallArtifact(yourapp, .{}).step);
   ```

3. Add to emulator's `APP_REGISTRY` in `src/ports/emulator/main.zig`:
   ```zig
   .{ .id = 4, .path = "zig-out/bin/yourapp.wasm" },
   ```

4. Add to launcher's app list in `src/apps/launcher/main.zig`:
   ```zig
   .{ .name = "Your App", .description = "Description", .id = 4 },
   ```

## Testing

### Desktop Emulator

```bash
# Build all apps and emulator
zig build
zig build emulator -Doptimize=ReleaseFast

# Run launcher (default)
./zig-out/bin/fri3d_emulator

# Run specific app
./zig-out/bin/fri3d_emulator zig-out/bin/yourapp.wasm
```

### Visual Regression Tests

Implement scene API for testing different app states:

```zig
var current_scene: u32 = 0;

export fn get_scene() u32 { return current_scene; }
export fn set_scene(scene: u32) void { current_scene = scene; }
export fn get_scene_count() u32 { return 3; } // Number of test scenes
```

Run visual tests:
```bash
uv run tests/visual/run_visual_tests.py
```

### Keyboard Controls (Emulator)

- Arrow keys: Navigation (up/down/left/right)
- Enter or Z: OK button
- Backspace, X, or Escape: Back button

## Lessons Learned

### Memory Management

- WASM apps have limited stack (16KB) and heap (16KB)
- Avoid deep recursion
- Use fixed-size buffers instead of dynamic allocation
- Global variables are fine for app state

### Number Formatting

Zig's std.fmt requires allocators; for simple formatting, use manual conversion:

```zig
fn formatInt(buf: *[16]u8, value: i32) [:0]const u8 {
    var v = value;
    var neg = false;
    if (v < 0) { neg = true; v = -v; }

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
```

### String Literals

Use `[:0]const u8` for null-terminated strings (required by canvas API):

```zig
const my_text: [:0]const u8 = "Hello";
platform.canvas.drawStr(0, 12, my_text);
```

### Frame Rate

- Target: 60fps (~16ms per frame)
- Keep render() fast - avoid heavy computation
- For expensive operations, spread work across frames

### Coordinate System

- Origin (0,0) is top-left
- Y increases downward
- Text y-coordinate is baseline (bottom of text)
