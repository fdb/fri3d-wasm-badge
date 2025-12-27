# Zig Migration Plan: Port-Based Architecture

## Overview

Migrate from C/C++ to Zig with a clean "Port" architecture where:
- **Apps** are WASM modules that export `render()`, `on_input()`, etc.
- **Ports** are platform-specific implementations that own the main loop
- **The Platform Interface** is the contract between apps and ports

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
│  Includes: App logic + SDK (imgui, etc.)                   │
│  Exports: render(), on_input(), scene functions            │
│  Imports: Platform Interface functions                      │
└─────────────────────────┬──────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────┐
│                  Platform Interface                         │
│                                                             │
│  The ABI contract that all ports must implement:           │
│                                                             │
│  ┌─────────────┬─────────────────────────────────────────┐ │
│  │ DISPLAY     │ canvas_clear, canvas_draw_*, present    │ │
│  │ INPUT       │ (port calls app's on_input export)      │ │
│  │ TIME        │ time_get_ms, time_sleep_ms              │ │
│  │ RANDOM      │ random_seed, random_get, random_range   │ │
│  │ STORAGE     │ storage_read, storage_write, storage_*  │ │
│  │ NETWORK     │ http_request, ws_connect, ws_send       │ │
│  │ SYSTEM      │ sys_get_battery, sys_get_info           │ │
│  └─────────────┴─────────────────────────────────────────┘ │
└─────────────────────────┬──────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Desktop Port │  │   Web Port   │  │  ESP32 Port  │
│    (SDL)     │  │    (JS)      │  │   (Native)   │
│              │  │              │  │              │
│ Language:    │  │ Language:    │  │ Language:    │
│ Zig          │  │ JavaScript   │  │ Zig          │
│              │  │              │  │              │
│ WASM Engine: │  │ WASM Engine: │  │ WASM Engine: │
│ WAMR         │  │ Browser      │  │ WAMR         │
│              │  │              │  │              │
│ Main Loop:   │  │ Main Loop:   │  │ Main Loop:   │
│ SDL events   │  │ rAF + events │  │ FreeRTOS     │
└──────────────┘  └──────────────┘  └──────────────┘
```

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

### Display

```zig
// Provided by platform as WASM imports
extern fn canvas_clear() void;
extern fn canvas_width() u32;
extern fn canvas_height() u32;
extern fn canvas_set_color(color: u32) void;
extern fn canvas_set_font(font: u32) void;
extern fn canvas_draw_dot(x: i32, y: i32) void;
extern fn canvas_draw_line(x1: i32, y1: i32, x2: i32, y2: i32) void;
extern fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32) void;
extern fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32) void;
extern fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, r: u32) void;
extern fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, r: u32) void;
extern fn canvas_draw_circle(x: i32, y: i32, r: u32) void;
extern fn canvas_draw_disc(x: i32, y: i32, r: u32) void;
extern fn canvas_draw_str(x: i32, y: i32, str: [*:0]const u8) void;
extern fn canvas_string_width(str: [*:0]const u8) u32;
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

### Phase 1: Foundation (Current Focus)
- [x] Plan architecture
- [ ] Create `build.zig`
- [ ] Define platform interface types in `src/sdk/platform.zig`
- [ ] Port one simple app (circles) to Zig

### Phase 2: SDK Migration
- [ ] Migrate canvas.h → canvas.zig
- [ ] Migrate input.h → input.zig
- [ ] Migrate random.h → random.zig
- [ ] Migrate imgui.c → imgui.zig (biggest piece)

### Phase 3: Desktop Port
- [ ] Create `src/ports/desktop/main.zig`
- [ ] Implement display using SDL + u8g2
- [ ] Implement input from SDL events
- [ ] Wire up WAMR for app loading

### Phase 4: Web Port
- [ ] Create `src/ports/web/platform.js`
- [ ] Implement canvas using HTML5 Canvas API
- [ ] Implement input from keyboard/touch events
- [ ] App loading via fetch() + WebAssembly API
- [ ] Basic app switching

### Phase 5: Polish & Future
- [ ] ESP32 port skeleton
- [ ] Storage interface design
- [ ] Network interface design
- [ ] App store prototype

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
