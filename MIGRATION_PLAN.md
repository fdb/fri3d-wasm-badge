# Fri3d Badge: Rust Migration Plan

## Overview

Migrate the entire codebase from C/C++ to Rust, implementing a **Ports & Apps** architecture where:
- **Apps** are WASM modules that export `render()`, `on_input()`, etc.
- **Ports** are platform-specific implementations that own the main loop
- Apps request services via imports (`canvas_*`, `storage_*`, etc.)

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                              APPS                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │  launcher   │ │   circles   │ │ mandelbrot  │ │   test_ui   │   │
│  │   (WASM)    │ │   (WASM)    │ │   (WASM)    │ │   (WASM)    │   │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘   │
│         │               │               │               │           │
│         └───────────────┴───────────────┴───────────────┘           │
│                                 │                                    │
│                         WASM Imports                                 │
│                    (canvas_*, input_*, etc.)                        │
│                                 │                                    │
├─────────────────────────────────┼───────────────────────────────────┤
│                                 ▼                                    │
│                           RUNTIME                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     fri3d_runtime (Rust)                     │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │   │
│  │  │ Canvas  │ │  Input  │ │ Random  │ │  WASM   │            │   │
│  │  │ (128x64)│ │ Manager │ │  (MT)   │ │ Runner  │            │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘            │   │
│  │  ┌─────────────────────────────────────────────────────┐    │   │
│  │  │              App Manager / Launcher                  │    │   │
│  │  └─────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                 │                                    │
│                          Port Trait                                  │
│                   (display, input, time)                            │
│                                 │                                    │
├─────────────────────────────────┼───────────────────────────────────┤
│                                 ▼                                    │
│                             PORTS                                    │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐           │
│  │   Desktop   │     │     Web     │     │    ESP32    │           │
│  │   (minifb)  │     │ (wasm-bind) │     │ (embedded)  │           │
│  │             │     │             │     │             │           │
│  │ Owns loop   │     │ RAF loop    │     │ FreeRTOS    │           │
│  │ SDL2 keybd  │     │ JS events   │     │ GPIO btns   │           │
│  │ PNG export  │     │ Canvas2D    │     │ SPI disp    │           │
│  └─────────────┘     └─────────────┘     └─────────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

## Directory Structure (New)

```
fri3d-wasm-badge/
├── Cargo.toml                    # Workspace root
├── rust-toolchain.toml           # Pin Rust version
│
├── crates/
│   ├── fri3d-runtime/            # Core runtime (no_std compatible)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── canvas.rs         # 128x64 monochrome framebuffer
│   │       ├── input.rs          # Key state, short/long press detection
│   │       ├── random.rs         # Mersenne Twister PRNG
│   │       ├── font.rs           # Bitmap fonts (4 sizes)
│   │       └── runtime.rs        # Coordinates canvas, input, random
│   │
│   ├── fri3d-wasm/               # WASM execution layer
│   │   ├── Cargo.toml            # depends on wasmi
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── host.rs           # Host functions (canvas_*, random_*)
│   │       ├── app.rs            # WasmApp: load, call render/on_input
│   │       └── app_manager.rs    # App lifecycle, switching
│   │
│   ├── fri3d-sdk/                # SDK for writing apps (Rust)
│   │   ├── Cargo.toml            # no_std, wasm32 target
│   │   └── src/
│   │       ├── lib.rs            # Re-exports
│   │       ├── canvas.rs         # canvas_* extern functions
│   │       ├── input.rs          # InputKey, InputType enums
│   │       └── random.rs         # random_* extern functions
│   │
│   └── fri3d-png/                # PNG encoder (no_std, optional)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs            # Minimal PNG encoder for screenshots
│
├── ports/
│   ├── desktop/                  # Desktop emulator
│   │   ├── Cargo.toml            # minifb, fri3d-runtime, fri3d-wasm
│   │   └── src/
│   │       ├── main.rs           # CLI, main loop
│   │       └── input.rs          # Keyboard → InputKey mapping
│   │
│   ├── web/                      # Web port (compiles to WASM)
│   │   ├── Cargo.toml            # wasm-bindgen, fri3d-runtime
│   │   ├── src/
│   │   │   └── lib.rs            # Exports for JS
│   │   ├── www/                  # Static web assets
│   │   │   ├── index.html
│   │   │   ├── main.js           # JS glue, app loader
│   │   │   └── style.css
│   │   └── build.sh              # wasm-pack build
│   │
│   └── esp32/                    # ESP32 port (future)
│       ├── Cargo.toml            # embedded-hal, esp-idf-hal
│       └── src/
│           └── main.rs           # ESP-IDF entry point
│
├── apps/                         # WASM applications
│   ├── launcher/                 # Built-in app launcher
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── circles/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── mandelbrot/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── test-drawing/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── test-ui/
│       ├── Cargo.toml
│       └── src/lib.rs
│
├── tests/
│   └── visual/                   # Keep existing Python tests
│       ├── run_visual_tests.py
│       └── apps/
│
├── build_desktop.sh              # Build desktop port + apps
├── build_web.sh                  # Build web port + apps
└── build_apps.sh                 # Build all WASM apps
```

## Crate Dependencies

```
fri3d-runtime (no_std)
    └── (no external deps, pure Rust)

fri3d-wasm (std)
    ├── fri3d-runtime
    └── wasmi

fri3d-sdk (no_std, wasm32)
    └── (no deps, just extern declarations)

fri3d-png (no_std, optional std)
    └── miniz_oxide (for deflate)

desktop port
    ├── fri3d-runtime
    ├── fri3d-wasm
    ├── fri3d-png
    └── minifb

web port
    ├── fri3d-runtime
    ├── wasm-bindgen
    └── web-sys

apps/*
    └── fri3d-sdk
```

## Implementation Phases

### Phase 1: Core Runtime (fri3d-runtime)

**Goal**: Pure Rust, no_std compatible runtime core

#### 1.1 Canvas (`crates/fri3d-runtime/src/canvas.rs`)

```rust
#![no_std]

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
    Xor = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Font {
    Primary = 0,      // 6x10
    Secondary = 1,    // 5x7
    Keyboard = 2,     // 5x8
    BigNumbers = 3,   // 10x20
}

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;
pub const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8; // 1024 bytes

pub struct Canvas {
    buffer: [u8; BUFFER_SIZE],  // 1-bit per pixel, row-major
    color: Color,
    font: Font,
}

impl Canvas {
    pub fn new() -> Self { ... }
    pub fn clear(&mut self) { ... }
    pub fn set_color(&mut self, color: Color) { ... }
    pub fn set_font(&mut self, font: Font) { ... }

    // Primitives
    pub fn draw_pixel(&mut self, x: i32, y: i32) { ... }
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) { ... }

    // Shapes
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32) { ... }
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32) { ... }
    pub fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) { ... }
    pub fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32) { ... }
    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: u32) { ... }
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: u32) { ... }

    // Text
    pub fn draw_str(&mut self, x: i32, y: i32, text: &str) { ... }
    pub fn string_width(&self, text: &str) -> u32 { ... }

    // Buffer access (for display output)
    pub fn buffer(&self) -> &[u8; BUFFER_SIZE] { &self.buffer }
    pub fn width(&self) -> u32 { WIDTH as u32 }
    pub fn height(&self) -> u32 { HEIGHT as u32 }
}
```

**Algorithms to implement**:
- Bresenham's line algorithm
- Midpoint circle algorithm (with XOR support)
- Flood fill for shapes (or scanline for filled circles)
- Bitmap font rendering (embed font data as const arrays)

#### 1.2 Input (`crates/fri3d-runtime/src/input.rs`)

```rust
#![no_std]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InputKey {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Ok = 4,
    Back = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InputType {
    Press = 0,      // Key went down (raw)
    Release = 1,    // Key went up (raw)
    Short = 2,      // Released quickly (< 300ms)
    Long = 3,       // Held for 500ms+
    Repeat = 4,     // Held down, repeating
}

pub struct InputEvent {
    pub key: InputKey,
    pub event_type: InputType,
}

const SHORT_PRESS_MAX_MS: u32 = 300;
const LONG_PRESS_MS: u32 = 500;
const RESET_COMBO_MS: u32 = 500;

pub struct InputManager {
    key_states: [KeyState; 6],
    event_queue: [Option<InputEvent>; 8],
    queue_head: usize,
    queue_tail: usize,
    reset_callback: Option<fn()>,
}

struct KeyState {
    pressed: bool,
    press_time: u32,
    long_fired: bool,
}

impl InputManager {
    pub fn new() -> Self { ... }

    /// Call each frame with current time in ms
    pub fn update(&mut self, time_ms: u32) { ... }

    /// Report a raw key press (from port)
    pub fn key_down(&mut self, key: InputKey, time_ms: u32) { ... }

    /// Report a raw key release (from port)
    pub fn key_up(&mut self, key: InputKey, time_ms: u32) { ... }

    /// Pop next event from queue
    pub fn poll_event(&mut self) -> Option<InputEvent> { ... }

    /// Set callback for reset combo (Left + Back held)
    pub fn set_reset_callback(&mut self, callback: fn()) { ... }
}
```

#### 1.3 Random (`crates/fri3d-runtime/src/random.rs`)

```rust
#![no_std]

/// Mersenne Twister PRNG
pub struct Random {
    state: [u32; 624],
    index: usize,
}

impl Random {
    pub fn new(seed: u32) -> Self { ... }
    pub fn seed(&mut self, seed: u32) { ... }
    pub fn next(&mut self) -> u32 { ... }
    pub fn range(&mut self, max: u32) -> u32 { ... }
}
```

#### 1.4 Fonts (`crates/fri3d-runtime/src/font.rs`)

```rust
#![no_std]

pub struct BitmapFont {
    pub width: u8,
    pub height: u8,
    pub baseline: u8,
    pub first_char: u8,
    pub last_char: u8,
    pub data: &'static [u8],
}

// Embed font bitmaps as const data
pub static FONT_PRIMARY: BitmapFont = BitmapFont {
    width: 6,
    height: 10,
    baseline: 8,
    first_char: 32,  // space
    last_char: 126,  // ~
    data: include_bytes!("../data/font_6x10.bin"),
};

// ... similar for other fonts
```

**Font data**: Extract from u8g2 or create from scratch. Simple 1-bit-per-pixel bitmaps.

---

### Phase 2: WASM Layer (fri3d-wasm)

**Goal**: wasmi-based WASM execution with host function bindings

#### 2.1 Host Functions (`crates/fri3d-wasm/src/host.rs`)

```rust
use wasmi::{Caller, Linker};
use fri3d_runtime::{Canvas, Random, Color, Font};

pub struct HostState {
    pub canvas: Canvas,
    pub random: Random,
}

pub fn register_host_functions(linker: &mut Linker<HostState>) -> Result<(), Error> {
    // Canvas functions
    linker.func_wrap("env", "canvas_clear", |caller: Caller<'_, HostState>| {
        caller.data_mut().canvas.clear();
    })?;

    linker.func_wrap("env", "canvas_width", |caller: Caller<'_, HostState>| -> u32 {
        caller.data().canvas.width()
    })?;

    linker.func_wrap("env", "canvas_set_color", |caller: Caller<'_, HostState>, color: u32| {
        let color = Color::try_from(color as u8).unwrap_or(Color::Black);
        caller.data_mut().canvas.set_color(color);
    })?;

    linker.func_wrap("env", "canvas_draw_circle",
        |caller: Caller<'_, HostState>, x: i32, y: i32, r: u32| {
            caller.data_mut().canvas.draw_circle(x, y, r);
        }
    )?;

    // ... all other canvas_* functions

    // Random functions
    linker.func_wrap("env", "random_seed", |caller: Caller<'_, HostState>, seed: u32| {
        caller.data_mut().random.seed(seed);
    })?;

    linker.func_wrap("env", "random_get", |caller: Caller<'_, HostState>| -> u32 {
        caller.data_mut().random.next()
    })?;

    linker.func_wrap("env", "random_range", |caller: Caller<'_, HostState>, max: u32| -> u32 {
        caller.data_mut().random.range(max)
    })?;

    Ok(())
}
```

#### 2.2 WASM App (`crates/fri3d-wasm/src/app.rs`)

```rust
use wasmi::{Engine, Module, Store, Instance, TypedFunc};

pub struct WasmApp {
    store: Store<HostState>,
    instance: Instance,
    render_func: TypedFunc<(), ()>,
    on_input_func: Option<TypedFunc<(u32, u32), ()>>,
    get_scene_count_func: Option<TypedFunc<(), u32>>,
    set_scene_func: Option<TypedFunc<(u32,), ()>>,
}

impl WasmApp {
    pub fn load(engine: &Engine, wasm_bytes: &[u8]) -> Result<Self, Error> {
        let module = Module::new(engine, wasm_bytes)?;
        let mut store = Store::new(engine, HostState::new());

        let mut linker = Linker::new(engine);
        register_host_functions(&mut linker)?;

        let instance = linker.instantiate(&mut store, &module)?
            .start(&mut store)?;

        // Get required exports
        let render_func = instance.get_typed_func::<(), ()>(&store, "render")?;

        // Get optional exports
        let on_input_func = instance
            .get_typed_func::<(u32, u32), ()>(&store, "on_input")
            .ok();

        Ok(Self { store, instance, render_func, on_input_func, ... })
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.render_func.call(&mut self.store, ())?;
        Ok(())
    }

    pub fn on_input(&mut self, key: InputKey, event_type: InputType) -> Result<(), Error> {
        if let Some(func) = &self.on_input_func {
            func.call(&mut self.store, (key as u32, event_type as u32))?;
        }
        Ok(())
    }

    pub fn canvas(&self) -> &Canvas {
        &self.store.data().canvas
    }
}
```

#### 2.3 App Manager (`crates/fri3d-wasm/src/app_manager.rs`)

```rust
pub struct AppManager {
    engine: Engine,
    apps: Vec<AppInfo>,
    current_app: Option<WasmApp>,
    launcher: WasmApp,
}

pub struct AppInfo {
    pub name: String,
    pub wasm_bytes: Vec<u8>,
}

impl AppManager {
    pub fn new(launcher_wasm: &[u8], apps: Vec<AppInfo>) -> Result<Self, Error> {
        let engine = Engine::default();
        let launcher = WasmApp::load(&engine, launcher_wasm)?;

        Ok(Self {
            engine,
            apps,
            current_app: None,
            launcher,
        })
    }

    pub fn switch_to_app(&mut self, index: usize) -> Result<(), Error> {
        let app = WasmApp::load(&self.engine, &self.apps[index].wasm_bytes)?;
        self.current_app = Some(app);
        Ok(())
    }

    pub fn back_to_launcher(&mut self) {
        self.current_app = None;
    }

    pub fn render(&mut self) -> Result<&Canvas, Error> {
        if let Some(app) = &mut self.current_app {
            app.render()?;
            Ok(app.canvas())
        } else {
            self.launcher.render()?;
            Ok(self.launcher.canvas())
        }
    }

    pub fn on_input(&mut self, key: InputKey, event_type: InputType) -> Result<(), Error> {
        // Handle reset combo separately (in port)
        if let Some(app) = &mut self.current_app {
            app.on_input(key, event_type)?;
        } else {
            self.launcher.on_input(key, event_type)?;
        }
        Ok(())
    }
}
```

---

### Phase 3: SDK (fri3d-sdk)

**Goal**: Minimal no_std crate for writing WASM apps in Rust

```rust
// crates/fri3d-sdk/src/lib.rs
#![no_std]

mod canvas;
mod input;
mod random;

pub use canvas::*;
pub use input::*;
pub use random::*;

// Panic handler for WASM
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

```rust
// crates/fri3d-sdk/src/canvas.rs
#![no_std]

#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
    Xor = 2,
}

#[repr(u8)]
pub enum Font {
    Primary = 0,
    Secondary = 1,
    Keyboard = 2,
    BigNumbers = 3,
}

extern "C" {
    pub fn canvas_clear();
    pub fn canvas_width() -> u32;
    pub fn canvas_height() -> u32;
    pub fn canvas_set_color(color: u8);
    pub fn canvas_set_font(font: u8);
    pub fn canvas_draw_dot(x: i32, y: i32);
    pub fn canvas_draw_line(x0: i32, y0: i32, x1: i32, y1: i32);
    pub fn canvas_draw_frame(x: i32, y: i32, w: u32, h: u32);
    pub fn canvas_draw_box(x: i32, y: i32, w: u32, h: u32);
    pub fn canvas_draw_rframe(x: i32, y: i32, w: u32, h: u32, r: u32);
    pub fn canvas_draw_rbox(x: i32, y: i32, w: u32, h: u32, r: u32);
    pub fn canvas_draw_circle(x: i32, y: i32, r: u32);
    pub fn canvas_draw_disc(x: i32, y: i32, r: u32);
    fn canvas_draw_str_raw(x: i32, y: i32, ptr: *const u8, len: u32);
    fn canvas_string_width_raw(ptr: *const u8, len: u32) -> u32;
}

pub fn clear() {
    unsafe { canvas_clear() }
}

pub fn width() -> u32 {
    unsafe { canvas_width() }
}

pub fn height() -> u32 {
    unsafe { canvas_height() }
}

pub fn set_color(color: Color) {
    unsafe { canvas_set_color(color as u8) }
}

pub fn draw_circle(x: i32, y: i32, r: u32) {
    unsafe { canvas_draw_circle(x, y, r) }
}

pub fn draw_str(x: i32, y: i32, text: &str) {
    unsafe { canvas_draw_str_raw(x, y, text.as_ptr(), text.len() as u32) }
}

// ... etc
```

---

### Phase 4: Desktop Port

**Goal**: Working desktop emulator with minifb

```rust
// ports/desktop/src/main.rs
use minifb::{Key, Window, WindowOptions};
use fri3d_runtime::{Canvas, InputKey, InputManager, WIDTH, HEIGHT};
use fri3d_wasm::AppManager;

const SCALE: usize = 4;

fn main() {
    let args = parse_args();

    // Load apps
    let launcher_wasm = include_bytes!("../../../apps/launcher/target/wasm32-unknown-unknown/release/launcher.wasm");
    let apps = load_apps(&args.apps_dir);

    let mut app_manager = AppManager::new(launcher_wasm, apps).unwrap();
    let mut input_manager = InputManager::new();

    // Create window
    let mut window = Window::new(
        "Fri3d Badge Emulator",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    ).unwrap();

    let mut buffer = vec![0u32; WIDTH * SCALE * HEIGHT * SCALE];
    let mut last_time = std::time::Instant::now();

    // Set reset callback
    input_manager.set_reset_callback(|| {
        // Will be handled in main loop
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let time_ms = now.duration_since(last_time).as_millis() as u32;

        // Process input
        process_keyboard(&window, &mut input_manager, time_ms);
        input_manager.update(time_ms);

        // Handle events
        while let Some(event) = input_manager.poll_event() {
            app_manager.on_input(event.key, event.event_type).unwrap();
        }

        // Render
        let canvas = app_manager.render().unwrap();

        // Convert 1-bit buffer to RGBA
        canvas_to_buffer(canvas, &mut buffer);

        // Display
        window.update_with_buffer(&buffer, WIDTH * SCALE, HEIGHT * SCALE).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn process_keyboard(window: &Window, input: &mut InputManager, time_ms: u32) {
    let key_map = [
        (Key::Up, InputKey::Up),
        (Key::Down, InputKey::Down),
        (Key::Left, InputKey::Left),
        (Key::Right, InputKey::Right),
        (Key::Enter, InputKey::Ok),
        (Key::Backspace, InputKey::Back),
    ];

    for (minifb_key, input_key) in key_map {
        if window.is_key_down(minifb_key) {
            input.key_down(input_key, time_ms);
        } else {
            input.key_up(input_key, time_ms);
        }
    }
}

fn canvas_to_buffer(canvas: &Canvas, buffer: &mut [u32]) {
    // Convert 1-bit monochrome to scaled RGBA
    // White = 0xFFE7D396 (warm badge color)
    // Black = 0xFF1A1A2E (dark)
    // ...
}
```

---

### Phase 5: Web Port

**Goal**: Runtime as WASM + JS glue for loading apps

#### 5.1 Rust Side (`ports/web/src/lib.rs`)

```rust
use wasm_bindgen::prelude::*;
use fri3d_runtime::{Canvas, InputKey, InputType, InputManager, Random};

#[wasm_bindgen]
pub struct Runtime {
    canvas: Canvas,
    random: Random,
    input_manager: InputManager,
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            random: Random::new(42),
            input_manager: InputManager::new(),
        }
    }

    // Canvas API for apps to call
    pub fn canvas_clear(&mut self) { self.canvas.clear(); }
    pub fn canvas_width(&self) -> u32 { self.canvas.width() }
    pub fn canvas_height(&self) -> u32 { self.canvas.height() }
    pub fn canvas_set_color(&mut self, color: u8) { ... }
    pub fn canvas_draw_circle(&mut self, x: i32, y: i32, r: u32) { ... }
    // ... all canvas functions

    // Random API
    pub fn random_seed(&mut self, seed: u32) { self.random.seed(seed); }
    pub fn random_get(&mut self) -> u32 { self.random.next() }
    pub fn random_range(&mut self, max: u32) -> u32 { self.random.range(max) }

    // Input API
    pub fn key_down(&mut self, key: u8, time_ms: u32) {
        if let Ok(k) = InputKey::try_from(key) {
            self.input_manager.key_down(k, time_ms);
        }
    }

    pub fn key_up(&mut self, key: u8, time_ms: u32) {
        if let Ok(k) = InputKey::try_from(key) {
            self.input_manager.key_up(k, time_ms);
        }
    }

    pub fn update_input(&mut self, time_ms: u32) {
        self.input_manager.update(time_ms);
    }

    pub fn poll_event(&mut self) -> Option<u32> {
        self.input_manager.poll_event().map(|e| {
            ((e.key as u32) << 8) | (e.event_type as u32)
        })
    }

    // Get buffer as Uint8Array for rendering to canvas
    pub fn get_buffer(&self) -> Vec<u8> {
        self.canvas.buffer().to_vec()
    }
}
```

#### 5.2 JS Glue (`ports/web/www/main.js`)

```javascript
import init, { Runtime } from './pkg/fri3d_web.js';

let runtime;
let currentApp;
let apps = {};

async function initRuntime() {
    await init();
    runtime = new Runtime();
}

async function loadApp(name) {
    const response = await fetch(`/apps/${name}.wasm`);
    const bytes = await response.arrayBuffer();

    // Create imports object that calls into runtime
    const imports = {
        env: {
            canvas_clear: () => runtime.canvas_clear(),
            canvas_width: () => runtime.canvas_width(),
            canvas_height: () => runtime.canvas_height(),
            canvas_set_color: (c) => runtime.canvas_set_color(c),
            canvas_draw_circle: (x, y, r) => runtime.canvas_draw_circle(x, y, r),
            // ... all imports
            random_seed: (s) => runtime.random_seed(s),
            random_get: () => runtime.random_get(),
            random_range: (m) => runtime.random_range(m),
        }
    };

    const module = await WebAssembly.instantiate(bytes, imports);
    return module.instance.exports;
}

async function main() {
    await initRuntime();

    // Load launcher
    currentApp = await loadApp('launcher');

    // Setup canvas
    const canvas = document.getElementById('display');
    const ctx = canvas.getContext('2d');

    // Main loop
    function frame(timestamp) {
        // Update input
        runtime.update_input(timestamp);

        // Handle events
        let event;
        while ((event = runtime.poll_event()) !== undefined) {
            const key = (event >> 8) & 0xFF;
            const type = event & 0xFF;
            currentApp.on_input?.(key, type);
        }

        // Render
        currentApp.render();

        // Draw to canvas
        const buffer = runtime.get_buffer();
        drawBuffer(ctx, buffer);

        requestAnimationFrame(frame);
    }

    requestAnimationFrame(frame);
}

function drawBuffer(ctx, buffer) {
    const imageData = ctx.createImageData(128, 64);
    for (let y = 0; y < 64; y++) {
        for (let x = 0; x < 128; x++) {
            const byteIndex = (y * 128 + x) >> 3;
            const bitIndex = 7 - (x & 7);
            const pixel = (buffer[byteIndex] >> bitIndex) & 1;

            const i = (y * 128 + x) * 4;
            const color = pixel ? 0x1A : 0xE7;
            imageData.data[i] = color;
            imageData.data[i + 1] = color;
            imageData.data[i + 2] = color;
            imageData.data[i + 3] = 255;
        }
    }
    ctx.putImageData(imageData, 0, 0);
}

// Keyboard handling
document.addEventListener('keydown', (e) => {
    const key = keyMap[e.code];
    if (key !== undefined) {
        runtime.key_down(key, performance.now());
        e.preventDefault();
    }
});

document.addEventListener('keyup', (e) => {
    const key = keyMap[e.code];
    if (key !== undefined) {
        runtime.key_up(key, performance.now());
    }
});

const keyMap = {
    'ArrowUp': 0,
    'ArrowDown': 1,
    'ArrowLeft': 2,
    'ArrowRight': 3,
    'Enter': 4,
    'Backspace': 5,
};

main();
```

---

### Phase 6: Apps Migration

**Goal**: Convert all apps to Rust no_std

#### Example: Circles App

```rust
// apps/circles/src/lib.rs
#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random, InputKey, InputType};

#[no_mangle]
pub extern "C" fn render() {
    canvas::set_color(canvas::Color::Black);

    for _ in 0..10 {
        let x = random::range(128) as i32;
        let y = random::range(64) as i32;
        let r = random::range(15) + 3;
        canvas::draw_circle(x, y, r);
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    let key = InputKey::try_from(key as u8).ok();
    let event_type = InputType::try_from(event_type as u8).ok();

    match (key, event_type) {
        (Some(InputKey::Ok), Some(InputType::Short)) => {
            // Do something on OK press
        }
        _ => {}
    }
}
```

#### Example: Mandelbrot App

```rust
// apps/mandelbrot/src/lib.rs
#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

static mut STATE: MandelbrotState = MandelbrotState::new();

struct MandelbrotState {
    center_x: f32,
    center_y: f32,
    zoom: f32,
}

impl MandelbrotState {
    const fn new() -> Self {
        Self {
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    let state = unsafe { &STATE };

    let scale = 3.0 / (state.zoom * 64.0);

    for py in 0..64 {
        for px in 0..128 {
            let x0 = state.center_x + (px as f32 - 64.0) * scale;
            let y0 = state.center_y + (py as f32 - 32.0) * scale;

            let mut x = 0.0f32;
            let mut y = 0.0f32;
            let mut iter = 0u32;

            while x * x + y * y <= 4.0 && iter < 32 {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iter += 1;
            }

            if iter >= 32 {
                canvas::set_color(canvas::Color::Black);
                canvas::draw_dot(px, py);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    let state = unsafe { &mut STATE };

    if event_type != InputType::Short as u32 {
        return;
    }

    match key {
        k if k == InputKey::Ok as u32 => state.zoom *= 2.0,
        k if k == InputKey::Back as u32 => state.zoom /= 2.0,
        k if k == InputKey::Up as u32 => state.center_y -= 0.1 / state.zoom,
        k if k == InputKey::Down as u32 => state.center_y += 0.1 / state.zoom,
        k if k == InputKey::Left as u32 => state.center_x -= 0.1 / state.zoom,
        k if k == InputKey::Right as u32 => state.center_x += 0.1 / state.zoom,
        _ => {}
    }
}
```

#### Launcher App

```rust
// apps/launcher/src/lib.rs
#![no_std]
#![no_main]

use fri3d_sdk::{canvas, InputKey, InputType};

const APPS: &[&str] = &["circles", "mandelbrot", "test_drawing", "test_ui"];

static mut SELECTED: usize = 0;

#[no_mangle]
pub extern "C" fn render() {
    canvas::clear();
    canvas::set_color(canvas::Color::Black);

    canvas::draw_str(4, 10, "Fri3d Badge");
    canvas::draw_line(0, 14, 128, 14);

    let selected = unsafe { SELECTED };

    for (i, app) in APPS.iter().enumerate() {
        let y = 22 + (i as i32 * 10);

        if i == selected {
            canvas::fill_rect(0, y - 8, 128, 10);
            canvas::set_color(canvas::Color::White);
        }

        canvas::draw_str(4, y, app);
        canvas::set_color(canvas::Color::Black);
    }
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    if event_type != InputType::Short as u32 {
        return;
    }

    let selected = unsafe { &mut SELECTED };

    match key {
        k if k == InputKey::Up as u32 => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        k if k == InputKey::Down as u32 => {
            if *selected < APPS.len() - 1 {
                *selected += 1;
            }
        }
        k if k == InputKey::Ok as u32 => {
            // Signal to host to launch app at index *selected
            // (Need a host function for this: launch_app(index))
        }
        _ => {}
    }
}
```

---

### Phase 7: ESP32 Port (Scaffold)

**Goal**: Prepare architecture for ESP32, don't fully implement yet

```rust
// ports/esp32/src/main.rs
#![no_std]
#![no_main]

use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::spi::*;
use fri3d_runtime::{Canvas, InputKey, InputManager};
use fri3d_wasm::AppManager;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // TODO: Initialize SPI display (SSD1306)
    // TODO: Initialize GPIO buttons
    // TODO: Load apps from SPIFFS/SD card
    // TODO: Initialize wasmi runtime

    let mut canvas = Canvas::new();
    let mut input_manager = InputManager::new();

    loop {
        // TODO: Poll buttons
        // TODO: Update input manager
        // TODO: Handle events
        // TODO: Render app
        // TODO: Push buffer to display

        // ~60 FPS
        FreeRtos::delay_ms(16);
    }
}
```

---

## Build Scripts

### `build_desktop.sh`

```bash
#!/bin/bash
set -e

# Build all apps first
./build_apps.sh

# Build desktop port
cargo build --release -p fri3d-desktop
```

### `build_apps.sh`

```bash
#!/bin/bash
set -e

APPS="launcher circles mandelbrot test-drawing test-ui"

for app in $APPS; do
    echo "Building $app..."
    cargo build --release \
        --target wasm32-unknown-unknown \
        -p $app \
        --manifest-path apps/$app/Cargo.toml
done

# Copy to output directory
mkdir -p build/apps
for app in $APPS; do
    cp apps/$app/target/wasm32-unknown-unknown/release/*.wasm build/apps/
done
```

### `build_web.sh`

```bash
#!/bin/bash
set -e

# Build apps
./build_apps.sh

# Build web port with wasm-pack
cd ports/web
wasm-pack build --target web --release

# Copy to www
cp -r pkg www/
cp ../../build/apps/*.wasm www/apps/

echo "Web build complete! Serve from ports/web/www/"
```

---

## Migration Checklist

### Phase 1: Core Runtime
- [ ] Create Cargo.toml workspace
- [ ] Implement `fri3d-runtime` crate
  - [ ] Canvas with all drawing primitives
  - [ ] Input manager with short/long press
  - [ ] Random (Mersenne Twister)
  - [ ] Bitmap fonts (extract from u8g2 or create)
- [ ] Unit tests for canvas algorithms

### Phase 2: WASM Layer
- [ ] Implement `fri3d-wasm` crate
  - [ ] wasmi integration
  - [ ] Host function bindings
  - [ ] WasmApp loader
  - [ ] AppManager

### Phase 3: SDK
- [ ] Implement `fri3d-sdk` crate
  - [ ] Canvas bindings
  - [ ] Input types
  - [ ] Random bindings

### Phase 4: Desktop Port
- [ ] Implement desktop port
  - [ ] minifb window
  - [ ] Keyboard input
  - [ ] CLI args (--test, --screenshot)
  - [ ] PNG export

### Phase 5: Apps
- [ ] Migrate launcher
- [ ] Migrate circles
- [ ] Migrate mandelbrot
- [ ] Migrate test_drawing
- [ ] Migrate test_ui

### Phase 6: Web Port
- [ ] Implement web port
  - [ ] wasm-bindgen exports
  - [ ] JS glue code
  - [ ] HTML/CSS

### Phase 7: Cleanup
- [ ] Remove C/C++ code
- [ ] Remove CMake files
- [ ] Update documentation
- [ ] Run visual tests

### Phase 8: ESP32 (Future)
- [ ] Scaffold ESP32 port
- [ ] Document hardware integration

---

## Key Design Decisions

1. **wasmi everywhere**: Consistent behavior, no_std compatible, simpler than wasmtime
2. **no_std runtime**: Can run on ESP32 without modification
3. **Ports own the loop**: Platform-specific timing, input, display
4. **Apps are reactive**: No blocking, no main loop in apps
5. **Launcher is an app**: Dogfooding the app architecture
6. **Web uses native WASM**: No WASM-in-WASM overhead

## Estimated Complexity

| Component | LOC (Rust) | Difficulty |
|-----------|------------|------------|
| fri3d-runtime | ~800 | Medium (canvas algorithms) |
| fri3d-wasm | ~400 | Medium (wasmi API) |
| fri3d-sdk | ~150 | Easy |
| Desktop port | ~300 | Easy |
| Web port | ~200 + ~150 JS | Medium |
| Apps (5 total) | ~500 | Easy |
| **Total** | ~2500 | Medium |

## Questions Resolved

1. wasmi for all platforms
2. minifb for desktop
3. Runtime.wasm + app.wasm for web
4. Single app at a time, no isolation
5. ESP32 architecture ready, implementation later
6. Keep Python visual tests
7. Pure Cargo, remove CMake

---

**Ready to implement! Start with Phase 1: Core Runtime.**
