# Zig Migration Plan: Port-Based Architecture

## Overview

Migrate from C/C++ to Zig with a clean "Port" architecture where:
- **Apps** are WASM modules that export `render()`, `on_input()`, etc.
- **Ports** are platform-specific implementations that own the main loop
- **The Platform Interface** is a **minimal** contract (just framebuffer + input)
- **All rendering code lives in Zig SDK** (compiled into apps, not platform)

### Key Architectural Decision: Zig-Native Rendering

**All drawing code (fonts, lines, circles, boxes, text) is implemented in Zig and compiles INTO the WASM apps.** The platform only provides:
1. A framebuffer to draw into
2. Input events
3. Time/random utilities

This means:
- **No u8g2 dependency** - pure Zig rendering
- **Pixel-perfect consistency** across all platforms
- **One implementation** to maintain
- **Platforms become thin** - just framebuffer display + input handling

### Key Principle: "Don't Call Us, We'll Call You"

The platform (port) owns:
- The main loop / event loop
- All I/O and async operations
- Resource lifecycle management
- When to call into the app

The app only:
- Responds to calls (`render()`, `on_input()`)
- Requests services via imports (`canvas_*`, `storage_*`)
- Returns quickly (no blocking)

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                      WASM Apps                              │
│           (Zig source → compiled to .wasm)                 │
│                                                             │
│  Includes: App logic + FULL SDK                            │
│    - canvas.zig (all drawing: lines, circles, fonts, etc.) │
│    - imgui.zig (UI framework)                              │
│    - App-specific code                                      │
│                                                             │
│  Exports: render(), on_input(), scene functions            │
│  Imports: MINIMAL platform interface (framebuffer only)    │
└─────────────────────────┬──────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────┐
│              Platform Interface (MINIMAL)                   │
│                                                             │
│  Only what CANNOT be done in pure Zig:                     │
│                                                             │
│  ┌─────────────┬─────────────────────────────────────────┐ │
│  │ FRAMEBUFFER │ fb_ptr, fb_width, fb_height, fb_present │ │
│  │ INPUT       │ (port calls app's on_input export)      │ │
│  │ TIME        │ time_get_ms                             │ │
│  │ RANDOM      │ random_get (hardware RNG if available)  │ │
│  │ STORAGE     │ storage_read, storage_write (future)    │ │
│  │ NETWORK     │ http_request, ws_* (future)             │ │
│  └─────────────┴─────────────────────────────────────────┘ │
└─────────────────────────┬──────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Desktop Port │  │   Web Port   │  │  ESP32 Port  │
│    (SDL)     │  │    (JS)      │  │   (Native)   │
│              │  │              │  │              │
│ Provides:    │  │ Provides:    │  │ Provides:    │
│ - Framebuffer│  │ - Framebuffer│  │ - Framebuffer│
│ - SDL events │  │ - KB/touch   │  │ - GPIO btns  │
│ - Display FB │  │ - Canvas2D   │  │ - SPI OLED   │
│              │  │              │  │              │
│ NO drawing   │  │ NO drawing   │  │ NO drawing   │
│ code here!   │  │ code here!   │  │ code here!   │
└──────────────┘  └──────────────┘  └──────────────┘
```

### What Lives Where

| Component | Location | Why |
|-----------|----------|-----|
| Line drawing | Zig SDK (canvas.zig) | Pure math, no platform dependency |
| Circle/disc | Zig SDK (canvas.zig) | Pure math |
| Rectangle/box | Zig SDK (canvas.zig) | Pure math |
| Font data | Zig SDK (font.zig) | Static data |
| Text rendering | Zig SDK (canvas.zig) | Uses font data |
| IMGUI widgets | Zig SDK (imgui.zig) | Uses canvas |
| Framebuffer memory | Platform | Platform allocates, shares ptr |
| Display to screen | Platform | Hardware-specific |
| Input handling | Platform | Hardware-specific |

## Why This Architecture?

### 1. Web Port Doesn't Need Emscripten

The browser already has a WASM engine. We just need:
- JavaScript to load .wasm files
- JavaScript to provide imports (canvas, storage, etc.)
- JavaScript to call exports (render, on_input)

No C/C++ compilation to WASM needed. Pure JS platform layer.

### 2. Ports Don't Share Code, They Share Interface

Each port can be implemented in the most natural way:
- Desktop: Zig + SDL + WAMR
- Web: Pure JavaScript + Browser APIs
- ESP32: Zig + bare metal + WAMR
- Future Android: Kotlin/Java + Android APIs + WAMR

The interface (imports/exports) is what's standardized, not the implementation.

### 3. Easy to Add Capabilities

Want to add file storage?
1. Define the interface: `storage_read(path, buf, len) -> i32`
2. Each port implements it their way:
   - Desktop: filesystem
   - Web: IndexedDB
   - ESP32: SPIFFS/LittleFS

### 4. App Store Ready

The network interface enables:
- Fetching app catalog from server
- Downloading .wasm files
- Checking for updates
- User authentication (future)

## File Structure After Migration

```
fri3d-wasm-badge/
├── build.zig                    # Single build file for everything
├── src/
│   ├── sdk/                     # SDK for WASM apps (compiled INTO apps)
│   │   ├── platform.zig         # Platform interface declarations
│   │   ├── canvas.zig           # Canvas drawing API
│   │   ├── input.zig            # Input types and helpers
│   │   ├── random.zig           # Random number API
│   │   ├── storage.zig          # Storage API (future)
│   │   ├── network.zig          # Network API (future)
│   │   └── imgui/               # IMGUI system
│   │       ├── imgui.zig        # Main IMGUI module
│   │       ├── widgets.zig      # Button, checkbox, etc.
│   │       ├── layout.zig       # VStack, HStack, etc.
│   │       └── menu.zig         # Scrollable menus
│   │
│   ├── apps/                    # WASM applications
│   │   ├── circles/
│   │   │   └── main.zig
│   │   ├── mandelbrot/
│   │   │   └── main.zig
│   │   ├── test_ui/
│   │   │   └── main.zig
│   │   └── launcher/            # Built-in launcher app
│   │       └── main.zig
│   │
│   └── ports/                   # Platform implementations
│       ├── desktop/             # SDL Desktop Port
│       │   ├── main.zig         # Entry point, main loop
│       │   ├── display.zig      # SDL window/rendering
│       │   ├── input.zig        # SDL keyboard handling
│       │   └── storage.zig      # Filesystem storage
│       │
│       ├── web/                 # Pure JavaScript Web Port
│       │   ├── index.html       # HTML shell
│       │   ├── platform.js      # Platform implementation
│       │   ├── display.js       # Canvas rendering
│       │   ├── input.js         # Keyboard/touch handling
│       │   └── storage.js       # IndexedDB wrapper
│       │
│       └── esp32/               # ESP32 Firmware Port
│           ├── main.zig         # Entry point, FreeRTOS
│           ├── display.zig      # SPI OLED driver
│           ├── input.zig        # GPIO buttons
│           └── storage.zig      # SPIFFS/LittleFS
│
├── libs/                        # External C libraries
│   ├── u8g2/                    # Graphics library (used by native ports)
│   └── wasm-micro-runtime/      # WAMR (used by native ports)
│
└── www/                         # Built web files (generated)
    ├── index.html
    ├── platform.js
    └── apps/
        ├── circles.wasm
        └── ...
```

## Platform Interface Specification

### Framebuffer (MINIMAL - platform provides)

```zig
// Platform imports - ONLY these are provided by the platform
extern fn fb_ptr() [*]u8;      // Pointer to framebuffer (128*64 bytes for 128x64 1bpp)
extern fn fb_width() u32;       // Framebuffer width (128)
extern fn fb_height() u32;      // Framebuffer height (64)
extern fn fb_present() void;    // Tell platform to display the framebuffer

// Time
extern fn time_ms() u32;        // Milliseconds since start

// Random (optional - can use Zig PRNG if not provided)
extern fn random_hw() u32;      // Hardware random number (if available)
```

### Canvas (Zig SDK - compiled into app)

```zig
// src/sdk/zig/canvas.zig - ALL drawing implemented in Zig
const canvas = @import("canvas");

// These are NOT platform imports - they're Zig functions that write to framebuffer
canvas.clear();
canvas.setColor(.white);  // .black, .white, .xor
canvas.setFont(.primary); // .primary, .secondary, .keyboard, .big_numbers
canvas.drawDot(x, y);
canvas.drawLine(x1, y1, x2, y2);
canvas.drawFrame(x, y, w, h);
canvas.drawBox(x, y, w, h);
canvas.drawRFrame(x, y, w, h, r);
canvas.drawRBox(x, y, w, h, r);
canvas.drawCircle(x, y, r);
canvas.drawDisc(x, y, r);
canvas.drawStr(x, y, "Hello");
canvas.stringWidth("Hello");
```

### Input

```zig
// Input key codes
pub const Key = enum(u32) {
    up = 0,
    down = 1,
    left = 2,
    right = 3,
    ok = 4,
    back = 5,
};

// Input event types
pub const EventType = enum(u32) {
    press = 0,
    release = 1,
    short_press = 2,
    long_press = 3,
    repeat = 4,
};

// App exports this, platform calls it
export fn on_input(key: u32, event_type: u32) void;
```

### Random

```zig
extern fn random_seed(seed: u32) void;
extern fn random_get() u32;
extern fn random_range(max: u32) u32;
```

### Storage (Future)

```zig
// Async storage API using callbacks
extern fn storage_read(path: [*:0]const u8, callback_id: u32) void;
extern fn storage_write(path: [*:0]const u8, data: [*]const u8, len: u32, callback_id: u32) void;
extern fn storage_delete(path: [*:0]const u8, callback_id: u32) void;
extern fn storage_list(path: [*:0]const u8, callback_id: u32) void;

// Platform calls back with results
export fn on_storage_result(callback_id: u32, success: bool, data: [*]const u8, len: u32) void;
```

### Network (Future)

```zig
// HTTP requests
extern fn http_request(
    method: u32,           // GET=0, POST=1, etc.
    url: [*:0]const u8,
    body: [*]const u8,
    body_len: u32,
    callback_id: u32
) void;

// Platform calls back with response
export fn on_http_response(callback_id: u32, status: u32, body: [*]const u8, len: u32) void;

// WebSocket (for real-time features)
extern fn ws_connect(url: [*:0]const u8, callback_id: u32) u32;  // returns handle
extern fn ws_send(handle: u32, data: [*]const u8, len: u32) void;
extern fn ws_close(handle: u32) void;

export fn on_ws_message(handle: u32, data: [*]const u8, len: u32) void;
export fn on_ws_close(handle: u32) void;
```

### System Info (Future)

```zig
extern fn sys_get_battery_percent() u32;
extern fn sys_get_wifi_connected() bool;
extern fn sys_get_time_unix() u64;
extern fn sys_vibrate(duration_ms: u32) void;
```

## Migration Phases

### Phase 1: Foundation ✅ DONE
- [x] Plan architecture
- [x] Create `build.zig`
- [x] Define platform interface types in `src/sdk/zig/platform.zig`
- [x] Port circles app to Zig
- [x] Basic web port with JS rendering

### Phase 2: Zig-Native Rendering ✅ DONE
- [x] Migrate imgui.c → imgui.zig
- [x] **Create canvas.zig with full drawing implementation**
  - [x] Framebuffer management (owned by canvas, exported to platform)
  - [x] setPixel with color modes (black/white/xor)
  - [x] drawLine (Bresenham)
  - [x] drawFrame, drawBox
  - [x] drawRFrame, drawRBox (rounded)
  - [x] drawCircle, drawDisc (midpoint algorithm)
  - [x] Font data (5x7 bitmap font)
  - [x] drawStr, stringWidth
- [x] **Update platform.zig to minimal framebuffer interface**
- [x] **Web platform.js reads framebuffer from WASM memory**
- [x] Update existing Zig apps to use new canvas module

### Phase 3: Desktop Port ✅ DONE
- [x] Create `src/ports/emulator/main.zig`
- [x] Implement framebuffer display using SDL (just blit pixels)
- [x] Implement input from SDL events
- [x] Wire up WAMR for app loading (built from source via Zig)
- [x] No u8g2 dependency needed!
- [x] Add screenshot functionality (--headless, --scene, --screenshot)

### Phase 4: Polish & Testing (CURRENT)
- [x] Visual regression testing with Zig apps (7 tests passing)
- [ ] ESP32 port skeleton
- [ ] Storage interface design
- [ ] Network interface design

## Benefits Summary

| Aspect | Before | After |
|--------|--------|-------|
| Languages | C + C++ + CMake | Zig + JS (web only) |
| Toolchain | CMake + GCC + Clang + Zig | Just Zig |
| Web build | Emscripten (complex) | Pure JS (simple) |
| Cross-compile | Complex CMake configs | `zig build -Dtarget=...` |
| Safety | Manual | Compile-time checked |
| Portability | OK | Excellent (clean ports) |
| Extensibility | Add to each platform | Define interface, implement per-port |
| **Graphics lib** | **u8g2 (C, 3rd party)** | **Pure Zig (no dependency)** |
| **Rendering** | **Per-platform impl** | **One Zig impl for all** |
| **Platform code** | **Complex (drawing)** | **Thin (just framebuffer)** |

## Open Questions

1. **Should the launcher be a WASM app or built into ports?**
   - Pro app: Same on all platforms, easy to update
   - Pro built-in: Faster startup, guaranteed to work

2. **How to handle async operations in WASM?**
   - Callback-based (shown above)
   - Or: WASM components/threads (future)

3. **Should we support hot-reload for development?**
   - Web: Easy (just reload WASM)
   - Desktop: Need file watching + WAMR reload

4. **Memory management for storage/network data?**
   - Platform allocates, passes pointer
   - App must copy if needed, platform frees after callback
