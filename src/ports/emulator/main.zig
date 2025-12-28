// ============================================================================
// Fri3d Badge Emulator - Zig Implementation
// ============================================================================
// Desktop emulator that runs WASM apps using WAMR runtime and SDL2 display.
//
// Usage:
//   fri3d_emulator                              # Show launcher
//   fri3d_emulator app.wasm                     # Run specific WASM app
//   fri3d_emulator --test app.wasm              # Render one frame and exit
//   fri3d_emulator --headless --scene 0 --screenshot out.png app.wasm

const std = @import("std");

// SDL2 bindings
const c = @cImport({
    @cInclude("SDL2/SDL.h");
});

// WAMR bindings
const wamr = @cImport({
    @cInclude("wasm_export.h");
});

// stb_image_write for PNG output
const stb = @cImport({
    @cInclude("stb_image_write.h");
});

// ============================================================================
// Constants
// ============================================================================

const SCREEN_WIDTH: c_int = 128;
const SCREEN_HEIGHT: c_int = 64;
const SCALE: c_int = 4;
const WINDOW_WIDTH: c_int = SCREEN_WIDTH * SCALE;
const WINDOW_HEIGHT: c_int = SCREEN_HEIGHT * SCALE;
const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
const TARGET_FPS: u32 = 60;
const FRAME_TIME_MS: u32 = 1000 / TARGET_FPS;

// Input timing constants
const LONG_PRESS_MS: u32 = 500; // Threshold for long_press
const REPEAT_INTERVAL_MS: u32 = 100; // Interval between repeat events

// Per-key state for timing detection
const KeyState = struct {
    pressed: bool = false,
    press_time: u32 = 0,
    long_press_sent: bool = false,
    last_repeat_time: u32 = 0,
};

// App registry - maps app IDs to WASM file paths
// ID 0 is always the launcher
const AppEntry = struct {
    id: u32,
    path: []const u8,
};

const APP_REGISTRY = [_]AppEntry{
    .{ .id = 0, .path = "zig-out/bin/launcher.wasm" },
    .{ .id = 1, .path = "zig-out/bin/test_ui.wasm" },
    .{ .id = 2, .path = "zig-out/bin/circles.wasm" },
    .{ .id = 3, .path = "zig-out/bin/mandelbrot.wasm" },
};

// Global state for app launch requests (set by native function, read by main loop)
var g_requested_app_id: ?u32 = null;

// Input key values (must match platform.zig InputKey enum)
const InputKey = enum(u32) {
    up = 0,
    down = 1,
    left = 2,
    right = 3,
    ok = 4,
    back = 5,
};

// Input type values (must match platform.zig InputType enum)
const InputType = enum(u32) {
    press = 0,
    release = 1,
    short_press = 2,
    long_press = 3,
    repeat = 4,
};

// ============================================================================
// Native Functions for WASM Apps
// ============================================================================
// These functions are exported to WASM apps via the "env" module

fn native_request_launch_app(_: wamr.wasm_exec_env_t, app_id: u32) void {
    g_requested_app_id = app_id;
}

fn native_request_exit_to_launcher(_: wamr.wasm_exec_env_t) void {
    g_requested_app_id = 0; // Launcher is always ID 0
}

// Native function symbol table
const native_symbols = [_]wamr.NativeSymbol{
    .{
        .symbol = "platform_request_launch_app",
        .func_ptr = @constCast(@ptrCast(&native_request_launch_app)),
        .signature = "(i)",
        .attachment = null,
    },
    .{
        .symbol = "platform_request_exit_to_launcher",
        .func_ptr = @constCast(@ptrCast(&native_request_exit_to_launcher)),
        .signature = "()",
        .attachment = null,
    },
};

// ============================================================================
// WASM Runtime State
// ============================================================================

const WasmApp = struct {
    module: wamr.wasm_module_t = null,
    module_inst: wamr.wasm_module_inst_t = null,
    exec_env: wamr.wasm_exec_env_t = null,

    // Cached function references
    func_render: wamr.wasm_function_inst_t = null,
    func_on_input: wamr.wasm_function_inst_t = null,
    func_get_framebuffer: wamr.wasm_function_inst_t = null,
    func_set_scene: wamr.wasm_function_inst_t = null,

    // Framebuffer pointer (in WASM memory)
    framebuffer_ptr: u32 = 0,

    const Self = @This();

    fn init(wasm_data: []const u8, error_buf: []u8) !Self {
        var self = Self{};

        // Load module
        self.module = wamr.wasm_runtime_load(
            @constCast(wasm_data.ptr),
            @intCast(wasm_data.len),
            error_buf.ptr,
            @intCast(error_buf.len),
        );
        if (self.module == null) {
            std.debug.print("Failed to load WASM module: {s}\n", .{error_buf});
            return error.WasmLoadFailed;
        }

        // Instantiate module
        self.module_inst = wamr.wasm_runtime_instantiate(
            self.module,
            16 * 1024, // stack size
            16 * 1024, // heap size
            error_buf.ptr,
            @intCast(error_buf.len),
        );
        if (self.module_inst == null) {
            std.debug.print("Failed to instantiate WASM module: {s}\n", .{error_buf});
            return error.WasmInstantiateFailed;
        }

        // Create execution environment
        self.exec_env = wamr.wasm_runtime_create_exec_env(self.module_inst, 16 * 1024);
        if (self.exec_env == null) {
            std.debug.print("Failed to create exec environment\n", .{});
            return error.WasmExecEnvFailed;
        }

        // Look up exported functions
        self.func_render = wamr.wasm_runtime_lookup_function(self.module_inst, "render");
        self.func_on_input = wamr.wasm_runtime_lookup_function(self.module_inst, "on_input");
        self.func_get_framebuffer = wamr.wasm_runtime_lookup_function(self.module_inst, "canvas_get_framebuffer");
        self.func_set_scene = wamr.wasm_runtime_lookup_function(self.module_inst, "set_scene");

        if (self.func_render == null) {
            std.debug.print("Warning: WASM module has no render() export\n", .{});
        }

        // Get framebuffer pointer (function returns a pointer, stored in argv[0])
        if (self.func_get_framebuffer != null) {
            var argv: [1]u32 = undefined;
            if (wamr.wasm_runtime_call_wasm(self.exec_env, self.func_get_framebuffer, 0, &argv)) {
                self.framebuffer_ptr = argv[0];
            }
        }

        return self;
    }

    fn deinit(self: *Self) void {
        if (self.exec_env != null) {
            wamr.wasm_runtime_destroy_exec_env(self.exec_env);
        }
        if (self.module_inst != null) {
            wamr.wasm_runtime_deinstantiate(self.module_inst);
        }
        if (self.module != null) {
            wamr.wasm_runtime_unload(self.module);
        }
    }

    fn callRender(self: *Self) void {
        if (self.func_render == null or self.exec_env == null) return;

        if (!wamr.wasm_runtime_call_wasm(self.exec_env, self.func_render, 0, null)) {
            const exception = wamr.wasm_runtime_get_exception(self.module_inst);
            if (exception != null) {
                std.debug.print("WASM exception: {s}\n", .{exception});
                wamr.wasm_runtime_clear_exception(self.module_inst);
            }
        }
    }

    fn callOnInput(self: *Self, key: InputKey, input_type: InputType) void {
        if (self.func_on_input == null or self.exec_env == null) return;

        var args: [2]u32 = .{ @intFromEnum(key), @intFromEnum(input_type) };
        if (!wamr.wasm_runtime_call_wasm(self.exec_env, self.func_on_input, 2, &args)) {
            const exception = wamr.wasm_runtime_get_exception(self.module_inst);
            if (exception != null) {
                std.debug.print("WASM exception in on_input: {s}\n", .{exception});
                wamr.wasm_runtime_clear_exception(self.module_inst);
            }
        }
    }

    fn callSetScene(self: *Self, scene: u32) void {
        if (self.func_set_scene == null or self.exec_env == null) return;

        var args: [1]u32 = .{scene};
        if (!wamr.wasm_runtime_call_wasm(self.exec_env, self.func_set_scene, 1, &args)) {
            const exception = wamr.wasm_runtime_get_exception(self.module_inst);
            if (exception != null) {
                std.debug.print("WASM exception in set_scene: {s}\n", .{exception});
                wamr.wasm_runtime_clear_exception(self.module_inst);
            }
        }
    }

    fn getFramebuffer(self: *Self) ?[]const u8 {
        if (self.framebuffer_ptr == 0 or self.module_inst == null) return null;

        // Convert WASM address to native pointer
        const ptr = wamr.wasm_runtime_addr_app_to_native(self.module_inst, self.framebuffer_ptr);
        if (ptr == null) return null;

        const native_ptr: [*]const u8 = @ptrCast(ptr);
        return native_ptr[0..FRAMEBUFFER_SIZE];
    }
};

// ============================================================================
// SDL Display
// ============================================================================

const Display = struct {
    window: ?*c.SDL_Window = null,
    renderer: ?*c.SDL_Renderer = null,
    texture: ?*c.SDL_Texture = null,
    running: bool = true,

    // Per-key timing state
    key_states: [6]KeyState = [_]KeyState{.{}} ** 6,

    // Key state tracking for BACK+LEFT combo detection
    back_pressed: bool = false,
    left_pressed: bool = false,
    combo_start_time: u32 = 0, // SDL ticks when both keys first held

    const Self = @This();

    fn init() !Self {
        var self = Self{};

        if (c.SDL_Init(c.SDL_INIT_VIDEO) < 0) {
            std.debug.print("SDL_Init failed: {s}\n", .{c.SDL_GetError()});
            return error.SDLInitFailed;
        }

        self.window = c.SDL_CreateWindow(
            "Fri3d Badge Emulator",
            c.SDL_WINDOWPOS_CENTERED,
            c.SDL_WINDOWPOS_CENTERED,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            c.SDL_WINDOW_SHOWN,
        );
        if (self.window == null) {
            std.debug.print("SDL_CreateWindow failed: {s}\n", .{c.SDL_GetError()});
            return error.WindowCreationFailed;
        }

        self.renderer = c.SDL_CreateRenderer(self.window, -1, c.SDL_RENDERER_ACCELERATED);
        if (self.renderer == null) {
            std.debug.print("SDL_CreateRenderer failed: {s}\n", .{c.SDL_GetError()});
            return error.RendererCreationFailed;
        }

        // Use nearest neighbor scaling for crisp pixels
        _ = c.SDL_SetHint(c.SDL_HINT_RENDER_SCALE_QUALITY, "0");

        self.texture = c.SDL_CreateTexture(
            self.renderer,
            c.SDL_PIXELFORMAT_RGBA8888,
            c.SDL_TEXTUREACCESS_STREAMING,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        );
        if (self.texture == null) {
            std.debug.print("SDL_CreateTexture failed: {s}\n", .{c.SDL_GetError()});
            return error.TextureCreationFailed;
        }

        return self;
    }

    fn deinit(self: *Self) void {
        if (self.texture) |t| c.SDL_DestroyTexture(t);
        if (self.renderer) |r| c.SDL_DestroyRenderer(r);
        if (self.window) |w| c.SDL_DestroyWindow(w);
        c.SDL_Quit();
    }

    fn present(self: *Self, framebuffer: []const u8) void {
        const tex = self.texture orelse return;
        const ren = self.renderer orelse return;

        var pixels: ?*anyopaque = null;
        var pitch: c_int = 0;

        if (c.SDL_LockTexture(tex, null, &pixels, &pitch) != 0) {
            return;
        }

        // Convert 1-byte-per-pixel framebuffer to RGBA
        const dst: [*]u32 = @ptrCast(@alignCast(pixels));
        for (0..SCREEN_HEIGHT) |y| {
            for (0..SCREEN_WIDTH) |x| {
                const src_idx = y * @as(usize, @intCast(SCREEN_WIDTH)) + x;
                const dst_idx = y * @as(usize, @intCast(@divTrunc(pitch, 4))) + x;
                // 0 = black, 1 = white
                dst[dst_idx] = if (framebuffer[src_idx] != 0) 0xFFFFFFFF else 0x000000FF;
            }
        }

        c.SDL_UnlockTexture(tex);
        _ = c.SDL_RenderClear(ren);
        _ = c.SDL_RenderCopy(ren, tex, null, null);
        c.SDL_RenderPresent(ren);
    }

    fn handleEvents(self: *Self, app: *WasmApp) void {
        var event: c.SDL_Event = undefined;
        while (c.SDL_PollEvent(&event) != 0) {
            switch (event.type) {
                c.SDL_QUIT => {
                    self.running = false;
                },
                c.SDL_KEYDOWN => {
                    if (event.key.repeat == 0) {
                        if (mapKey(event.key.keysym.sym)) |key| {
                            const key_idx = @intFromEnum(key);
                            const now = c.SDL_GetTicks();

                            // Initialize per-key timing state
                            self.key_states[key_idx] = .{
                                .pressed = true,
                                .press_time = now,
                                .long_press_sent = false,
                                .last_repeat_time = 0,
                            };

                            // Track for BACK+LEFT combo detection
                            if (key == .back) self.back_pressed = true;
                            if (key == .left) self.left_pressed = true;
                            if (self.back_pressed and self.left_pressed and self.combo_start_time == 0) {
                                self.combo_start_time = now;
                            }

                            // Send immediate press event
                            app.callOnInput(key, .press);
                        }
                    }
                },
                c.SDL_KEYUP => {
                    if (mapKey(event.key.keysym.sym)) |key| {
                        const key_idx = @intFromEnum(key);
                        const key_state = &self.key_states[key_idx];

                        if (key_state.pressed) {
                            // If long_press was NOT sent, this is a short_press
                            if (!key_state.long_press_sent) {
                                app.callOnInput(key, .short_press);
                            }

                            // Always send release
                            app.callOnInput(key, .release);

                            // Reset per-key state
                            key_state.pressed = false;
                        }

                        // Track for BACK+LEFT combo detection
                        if (key == .back) self.back_pressed = false;
                        if (key == .left) self.left_pressed = false;
                        if (!self.back_pressed or !self.left_pressed) {
                            self.combo_start_time = 0;
                        }
                    }
                },
                else => {},
            }
        }
    }

    /// Check key timing and generate long_press/repeat events
    fn checkKeyTiming(self: *Self, app: *WasmApp) void {
        const now = c.SDL_GetTicks();

        for (0..6) |i| {
            const key_state = &self.key_states[i];
            if (!key_state.pressed) continue;

            const held_time = now - key_state.press_time;
            const key: InputKey = @enumFromInt(i);

            // Check for long_press (fires once after threshold)
            if (!key_state.long_press_sent and held_time >= LONG_PRESS_MS) {
                app.callOnInput(key, .long_press);
                key_state.long_press_sent = true;
                key_state.last_repeat_time = now;
            }

            // Check for repeat (fires periodically after long_press)
            if (key_state.long_press_sent) {
                const time_since_repeat = now - key_state.last_repeat_time;
                if (time_since_repeat >= REPEAT_INTERVAL_MS) {
                    app.callOnInput(key, .repeat);
                    key_state.last_repeat_time = now;
                }
            }
        }
    }

    /// Check if BACK+LEFT has been held for LONG_PRESS_MS
    fn checkExitCombo(self: *Self) bool {
        if (self.back_pressed and self.left_pressed and self.combo_start_time > 0) {
            const held_time = c.SDL_GetTicks() - self.combo_start_time;
            if (held_time >= LONG_PRESS_MS) {
                // Reset to prevent repeated triggers
                self.combo_start_time = 0;
                return true;
            }
        }
        return false;
    }

    fn mapKey(sdl_key: c.SDL_Keycode) ?InputKey {
        return switch (sdl_key) {
            c.SDLK_UP => .up,
            c.SDLK_DOWN => .down,
            c.SDLK_LEFT => .left,
            c.SDLK_RIGHT => .right,
            c.SDLK_RETURN, c.SDLK_z => .ok,
            c.SDLK_BACKSPACE, c.SDLK_x, c.SDLK_ESCAPE => .back,
            else => null,
        };
    }
};

// ============================================================================
// WAMR Initialization
// ============================================================================

var g_heap_buf: [10 * 1024 * 1024]u8 = undefined; // 10MB heap

fn initWasmRuntime() bool {
    var init_args: wamr.RuntimeInitArgs = std.mem.zeroes(wamr.RuntimeInitArgs);
    init_args.mem_alloc_type = wamr.Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = &g_heap_buf;
    init_args.mem_alloc_option.pool.heap_size = g_heap_buf.len;

    // Register native functions
    init_args.n_native_symbols = native_symbols.len;
    init_args.native_symbols = @constCast(&native_symbols);
    init_args.native_module_name = "env";

    return wamr.wasm_runtime_full_init(&init_args);
}

fn deinitWasmRuntime() void {
    wamr.wasm_runtime_destroy();
}

/// Look up app path by ID in the registry
fn getAppPathById(id: u32) ?[]const u8 {
    for (APP_REGISTRY) |entry| {
        if (entry.id == id) {
            return entry.path;
        }
    }
    return null;
}

// ============================================================================
// Screenshot Saving (PNG via stb_image_write)
// ============================================================================

fn saveScreenshot(framebuffer: []const u8, path: []const u8) bool {
    // Convert 1bpp framebuffer to 24-bit RGB for PNG
    var rgb_buffer: [SCREEN_WIDTH * SCREEN_HEIGHT * 3]u8 = undefined;

    for (0..@intCast(SCREEN_HEIGHT)) |y| {
        for (0..@intCast(SCREEN_WIDTH)) |x| {
            const src_idx = y * @as(usize, @intCast(SCREEN_WIDTH)) + x;
            const dst_idx = (y * @as(usize, @intCast(SCREEN_WIDTH)) + x) * 3;
            const pixel: u8 = if (framebuffer[src_idx] != 0) 0xFF else 0x00;
            rgb_buffer[dst_idx] = pixel; // R
            rgb_buffer[dst_idx + 1] = pixel; // G
            rgb_buffer[dst_idx + 2] = pixel; // B
        }
    }

    // Null-terminate the path for C function
    var path_buf: [256]u8 = undefined;
    if (path.len >= path_buf.len) return false;
    @memcpy(path_buf[0..path.len], path);
    path_buf[path.len] = 0;

    // Save as PNG (stbi_write_png returns non-zero on success)
    const result = stb.stbi_write_png(
        &path_buf,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        3, // RGB components
        &rgb_buffer,
        SCREEN_WIDTH * 3, // stride
    );

    return result != 0;
}

// ============================================================================
// Main
// ============================================================================

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Parse command line
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    var wasm_path: ?[]const u8 = null;
    var test_mode = false;
    var headless_mode = false;
    var screenshot_path: ?[]const u8 = null;
    var scene_index: ?u32 = null;

    var i: usize = 1;
    while (i < args.len) : (i += 1) {
        const arg = args[i];
        if (std.mem.eql(u8, arg, "--test")) {
            test_mode = true;
        } else if (std.mem.eql(u8, arg, "--headless")) {
            headless_mode = true;
        } else if (std.mem.eql(u8, arg, "--screenshot")) {
            i += 1;
            if (i < args.len) {
                screenshot_path = args[i];
            }
        } else if (std.mem.eql(u8, arg, "--scene")) {
            i += 1;
            if (i < args.len) {
                scene_index = std.fmt.parseInt(u32, args[i], 10) catch null;
            }
        } else if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
            std.debug.print("Usage: fri3d_emulator [options] [wasm_file]\n\n", .{});
            std.debug.print("Options:\n", .{});
            std.debug.print("  --test              Render one frame and exit\n", .{});
            std.debug.print("  --headless          Run without display window\n", .{});
            std.debug.print("  --scene N           Set scene index before rendering\n", .{});
            std.debug.print("  --screenshot FILE   Save screenshot and exit\n", .{});
            std.debug.print("  --help              Show this help\n", .{});
            return;
        } else {
            wasm_path = arg;
        }
    }

    // Default to launcher app if no WASM file specified
    if (wasm_path == null) {
        // Try launcher first, then fall back to test_ui for compatibility
        const launcher_paths = [_][]const u8{
            "zig-out/bin/launcher.wasm",
            "www/launcher.wasm",
            "zig-out/bin/test_ui.wasm",
            "www/test_ui.wasm",
        };
        for (launcher_paths) |path| {
            if (std.fs.cwd().access(path, .{})) |_| {
                wasm_path = path;
                break;
            } else |_| {}
        }
        if (wasm_path == null) {
            std.debug.print("No WASM file specified and no launcher found.\n", .{});
            std.debug.print("Usage: fri3d_emulator [options] <wasm_file>\n", .{});
            std.debug.print("Run with --help for more options\n", .{});
            return;
        }
    }

    // Initialize WAMR
    if (!initWasmRuntime()) {
        std.debug.print("Failed to initialize WASM runtime\n", .{});
        return;
    }
    defer deinitWasmRuntime();

    // Track current app path (may change during runtime)
    var current_path: []const u8 = wasm_path.?;

    // For headless mode, just run once without display
    if (headless_mode or screenshot_path != null) {
        const wasm_data = try std.fs.cwd().readFileAlloc(allocator, current_path, 10 * 1024 * 1024);
        defer allocator.free(wasm_data);

        var error_buf: [256]u8 = undefined;
        var app = WasmApp.init(wasm_data, &error_buf) catch |err| {
            std.debug.print("Failed to initialize WASM app: {}\n", .{err});
            return;
        };
        defer app.deinit();

        if (scene_index) |scene| {
            app.callSetScene(scene);
        }

        app.callRender();

        if (screenshot_path) |path| {
            if (app.getFramebuffer()) |fb| {
                if (!saveScreenshot(fb, path)) {
                    std.debug.print("Failed to save screenshot to {s}\n", .{path});
                    std.process.exit(1);
                }
            } else {
                std.debug.print("Failed to get framebuffer\n", .{});
                std.process.exit(1);
            }
        }
        return;
    }

    // Initialize display for interactive mode (once, persists across app switches)
    var display = try Display.init();
    defer display.deinit();

    // Main app loop - supports app switching
    app_loop: while (display.running) {
        // Load WASM file
        const wasm_data = std.fs.cwd().readFileAlloc(allocator, current_path, 10 * 1024 * 1024) catch |err| {
            std.debug.print("Failed to load {s}: {}\n", .{ current_path, err });
            // If we failed to load a requested app, try to go back to launcher
            if (!std.mem.eql(u8, current_path, "zig-out/bin/launcher.wasm")) {
                current_path = "zig-out/bin/launcher.wasm";
                continue :app_loop;
            }
            return;
        };
        defer allocator.free(wasm_data);

        // Initialize WASM app
        var error_buf: [256]u8 = undefined;
        var app = WasmApp.init(wasm_data, &error_buf) catch |err| {
            std.debug.print("Failed to initialize WASM app: {}\n", .{err});
            // If we failed to init a requested app, try to go back to launcher
            if (!std.mem.eql(u8, current_path, "zig-out/bin/launcher.wasm")) {
                current_path = "zig-out/bin/launcher.wasm";
                continue :app_loop;
            }
            return;
        };
        defer app.deinit();

        // Clear any pending launch request
        g_requested_app_id = null;

        // Reset key state on app switch
        display.back_pressed = false;
        display.left_pressed = false;
        display.combo_start_time = 0;
        display.key_states = [_]KeyState{.{}} ** 6;

        // Set scene if requested (only for initial app)
        if (scene_index) |scene| {
            app.callSetScene(scene);
            scene_index = null; // Only apply once
        }

        // Frame loop for current app
        while (display.running) {
            const frame_start = c.SDL_GetTicks();

            // Handle input events
            display.handleEvents(&app);

            // Check key timing for long_press and repeat events
            display.checkKeyTiming(&app);

            // Check for BACK+LEFT long-press combo to exit to launcher
            if (display.checkExitCombo()) {
                // Don't exit if we're already on the launcher
                if (!std.mem.eql(u8, current_path, "zig-out/bin/launcher.wasm")) {
                    current_path = "zig-out/bin/launcher.wasm";
                    break; // Exit frame loop to load new app
                }
            }

            // Check for app launch request
            if (g_requested_app_id) |app_id| {
                if (getAppPathById(app_id)) |new_path| {
                    current_path = new_path;
                    g_requested_app_id = null;
                    break; // Exit frame loop to load new app
                } else {
                    std.debug.print("Unknown app ID: {}\n", .{app_id});
                    g_requested_app_id = null;
                }
            }

            // Render
            app.callRender();

            // Get framebuffer and display
            if (app.getFramebuffer()) |fb| {
                display.present(fb);
            }

            // Test mode: exit after one frame
            if (test_mode) {
                return;
            }

            // Frame timing
            const frame_time = c.SDL_GetTicks() - frame_start;
            if (frame_time < FRAME_TIME_MS) {
                c.SDL_Delay(FRAME_TIME_MS - frame_time);
            }
        }
    }

    std.debug.print("Emulator exited cleanly\n", .{});
}
