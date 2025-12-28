// ============================================================================
// Fri3d Badge Emulator - Zig Implementation
// ============================================================================
// Desktop emulator that runs WASM apps using WAMR runtime and SDL2 display.
//
// Usage:
//   fri3d_emulator                    # Show launcher (TODO)
//   fri3d_emulator app.wasm           # Run specific WASM app
//   fri3d_emulator --test app.wasm    # Render one frame and exit
//   fri3d_emulator --screenshot out.png app.wasm  # Save screenshot

const std = @import("std");

// SDL2 bindings
const c = @cImport({
    @cInclude("SDL2/SDL.h");
});

// WAMR bindings
const wamr = @cImport({
    @cInclude("wasm_export.h");
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
                            app.callOnInput(key, .press);
                        }
                    }
                },
                c.SDL_KEYUP => {
                    if (mapKey(event.key.keysym.sym)) |key| {
                        app.callOnInput(key, .release);
                    }
                },
                else => {},
            }
        }
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

    return wamr.wasm_runtime_full_init(&init_args);
}

fn deinitWasmRuntime() void {
    wamr.wasm_runtime_destroy();
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

    var i: usize = 1;
    while (i < args.len) : (i += 1) {
        const arg = args[i];
        if (std.mem.eql(u8, arg, "--test")) {
            test_mode = true;
        } else if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
            std.debug.print("Usage: fri3d_emulator [options] [wasm_file]\n\n", .{});
            std.debug.print("Options:\n", .{});
            std.debug.print("  --test    Render one frame and exit\n", .{});
            std.debug.print("  --help    Show this help\n", .{});
            return;
        } else {
            wasm_path = arg;
        }
    }

    // Default to launcher app if no WASM file specified
    if (wasm_path == null) {
        // Try common locations for the launcher
        const launcher_paths = [_][]const u8{
            "zig-out/bin/test_ui_zig.wasm",
            "www/test_ui_zig.wasm",
            "build/apps/launcher/launcher.wasm",
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

    // Load WASM file
    const wasm_data = try std.fs.cwd().readFileAlloc(allocator, wasm_path.?, 10 * 1024 * 1024);
    defer allocator.free(wasm_data);

    // Initialize WASM app
    var error_buf: [256]u8 = undefined;
    var app = WasmApp.init(wasm_data, &error_buf) catch |err| {
        std.debug.print("Failed to initialize WASM app: {}\n", .{err});
        return;
    };
    defer app.deinit();

    // Initialize display
    var display = try Display.init();
    defer display.deinit();

    // Main loop
    while (display.running) {
        const frame_start = c.SDL_GetTicks();

        // Handle input
        display.handleEvents(&app);

        // Render
        app.callRender();

        // Get framebuffer and display
        if (app.getFramebuffer()) |fb| {
            display.present(fb);
        }

        // Test mode: exit after one frame
        if (test_mode) {
            break;
        }

        // Frame timing
        const frame_time = c.SDL_GetTicks() - frame_start;
        if (frame_time < FRAME_TIME_MS) {
            c.SDL_Delay(FRAME_TIME_MS - frame_time);
        }
    }

    std.debug.print("Emulator exited cleanly\n", .{});
}
