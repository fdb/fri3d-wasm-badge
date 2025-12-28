// ============================================================================
// Desktop Platform (SDL2)
// ============================================================================
// Native desktop emulator using SDL2 for display and input.
// Uses the same canvas.zig as web and embedded platforms.

const std = @import("std");
const platform = @import("platform");
const canvas = platform.canvas;
const app = @import("app");

// SDL2 C bindings
const c = @cImport({
    @cInclude("SDL2/SDL.h");
});

// Display settings
const SCALE: c_int = 4;
const WINDOW_WIDTH: c_int = canvas.SCREEN_WIDTH * SCALE;
const WINDOW_HEIGHT: c_int = canvas.SCREEN_HEIGHT * SCALE;
const TARGET_FPS: u32 = 60;
const FRAME_TIME_MS: u32 = 1000 / TARGET_FPS;

// SDL state
var window: ?*c.SDL_Window = null;
var renderer: ?*c.SDL_Renderer = null;
var texture: ?*c.SDL_Texture = null;
var running: bool = true;

// Timing
var last_frame_time: u32 = 0;

pub fn main() !void {
    try init();
    defer deinit();

    // Initialize the app
    app.init();

    // Main loop
    while (running) {
        const frame_start = c.SDL_GetTicks();

        // Handle events
        handleEvents();

        // Render frame
        canvas.clear();
        app.render();

        // Present to screen
        present();

        // Frame timing
        const frame_time = c.SDL_GetTicks() - frame_start;
        if (frame_time < FRAME_TIME_MS) {
            c.SDL_Delay(FRAME_TIME_MS - frame_time);
        }
    }
}

fn init() !void {
    // Initialize SDL
    if (c.SDL_Init(c.SDL_INIT_VIDEO) < 0) {
        std.debug.print("SDL_Init failed: {s}\n", .{c.SDL_GetError()});
        return error.SDLInitFailed;
    }

    // Create window
    window = c.SDL_CreateWindow(
        "Fri3d Badge Emulator",
        c.SDL_WINDOWPOS_CENTERED,
        c.SDL_WINDOWPOS_CENTERED,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        c.SDL_WINDOW_SHOWN,
    );
    if (window == null) {
        std.debug.print("SDL_CreateWindow failed: {s}\n", .{c.SDL_GetError()});
        return error.WindowCreationFailed;
    }

    // Create renderer
    renderer = c.SDL_CreateRenderer(window, -1, c.SDL_RENDERER_ACCELERATED);
    if (renderer == null) {
        std.debug.print("SDL_CreateRenderer failed: {s}\n", .{c.SDL_GetError()});
        return error.RendererCreationFailed;
    }

    // Set scaling quality
    _ = c.SDL_SetHint(c.SDL_HINT_RENDER_SCALE_QUALITY, "0"); // Nearest neighbor

    // Create texture for framebuffer
    texture = c.SDL_CreateTexture(
        renderer,
        c.SDL_PIXELFORMAT_RGB332,
        c.SDL_TEXTUREACCESS_STREAMING,
        canvas.SCREEN_WIDTH,
        canvas.SCREEN_HEIGHT,
    );
    if (texture == null) {
        std.debug.print("SDL_CreateTexture failed: {s}\n", .{c.SDL_GetError()});
        return error.TextureCreationFailed;
    }
}

fn deinit() void {
    if (texture) |t| c.SDL_DestroyTexture(t);
    if (renderer) |r| c.SDL_DestroyRenderer(r);
    if (window) |w| c.SDL_DestroyWindow(w);
    c.SDL_Quit();
}

fn handleEvents() void {
    var event: c.SDL_Event = undefined;
    while (c.SDL_PollEvent(&event) != 0) {
        switch (event.type) {
            c.SDL_QUIT => {
                running = false;
            },
            c.SDL_KEYDOWN => {
                if (event.key.repeat == 0) {
                    if (mapKey(event.key.keysym.sym)) |key| {
                        app.on_input(key, .press);
                    }
                }
            },
            c.SDL_KEYUP => {
                if (mapKey(event.key.keysym.sym)) |key| {
                    app.on_input(key, .release);
                }
            },
            else => {},
        }
    }
}

fn mapKey(sdl_key: c.SDL_Keycode) ?platform.InputKey {
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

fn present() void {
    const tex = texture orelse return;
    const ren = renderer orelse return;

    // Get framebuffer from canvas
    const fb = canvas.getFramebuffer();

    // Update texture with framebuffer data
    // Convert 1-bit monochrome to RGB332 format
    var pixels: [canvas.SCREEN_WIDTH * canvas.SCREEN_HEIGHT]u8 = undefined;
    for (0..canvas.SCREEN_HEIGHT) |y| {
        for (0..canvas.SCREEN_WIDTH) |x| {
            const pixel = fb[y * canvas.SCREEN_WIDTH + x];
            // Map: 0 = black (0x00), 1 = white (0xFF in RGB332)
            pixels[y * canvas.SCREEN_WIDTH + x] = if (pixel != 0) 0xFF else 0x00;
        }
    }

    _ = c.SDL_UpdateTexture(tex, null, &pixels, canvas.SCREEN_WIDTH);

    // Clear and render
    _ = c.SDL_RenderClear(ren);
    _ = c.SDL_RenderCopy(ren, tex, null, null);
    c.SDL_RenderPresent(ren);
}
