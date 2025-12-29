# Fri3d WASM Badge

A WebAssembly-based badge platform for the Fri3d camp badge, featuring a 128x64 OLED display. Written entirely in Rust with a **Ports & Apps** architecture:

- **Apps** are WASM modules that export `render()`, `on_input()`, etc.
- **Ports** are platform-specific implementations that own the main loop

## Getting Started

### Prerequisites

- **Rust** (stable toolchain)
- **wasm32-unknown-unknown** target: `rustup target add wasm32-unknown-unknown`

### Building

#### Desktop Emulator + Apps

```bash
./build_desktop.sh
```

This builds the desktop emulator and all WASM apps.

#### WASM Apps Only

```bash
./build_apps.sh
```

#### Web Build

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):

```bash
cargo install wasm-pack
./build_web.sh
```

### Running

#### Desktop Emulator

```bash
# Run with default app (circles)
./target/release/fri3d-emulator

# Run a specific app
./target/release/fri3d-emulator build/apps/mandelbrot.wasm
```

#### Web Emulator

```bash
cd ports/web/www
python3 -m http.server 8080
# Open http://localhost:8080/
```

### Controls

- Arrow keys: Navigate
- Enter: OK / Select
- Backspace / Escape: Back
- S: Screenshot
- Left + Back (hold 500ms): Return to launcher

## Project Structure

```
fri3d-wasm-badge/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── fri3d-runtime/          # Core runtime (no_std compatible)
│   │   └── src/
│   │       ├── canvas.rs       # 128x64 monochrome framebuffer
│   │       ├── input.rs        # Key state, short/long press
│   │       ├── random.rs       # Mersenne Twister PRNG
│   │       └── font.rs         # Bitmap fonts
│   ├── fri3d-wasm/             # WASM execution (wasmi)
│   ├── fri3d-sdk/              # SDK for WASM apps
│   └── fri3d-png/              # PNG encoder for screenshots
├── ports/
│   ├── desktop/                # Desktop emulator (minifb)
│   ├── web/                    # Web runtime (wasm-bindgen)
│   └── esp32/                  # ESP32 port (future)
├── apps/
│   ├── circles/                # Random circles demo
│   ├── mandelbrot/             # Mandelbrot explorer
│   ├── test-drawing/           # Drawing primitives test
│   ├── test-ui/                # UI widget test
│   └── launcher/               # Built-in app launcher
└── tests/visual/               # Visual regression tests
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         APPS (WASM)                          │
│  circles.wasm   mandelbrot.wasm   launcher.wasm   ...       │
│                                                              │
│  Exports: render(), on_input()                              │
│  Imports: canvas_*, random_*                                │
└─────────────────────┬───────────────────────────────────────┘
                      │
              WASM Imports/Exports
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                      RUNTIME (Rust)                          │
│  fri3d-runtime: Canvas, Input, Random, Fonts                │
│  fri3d-wasm: wasmi integration, host functions              │
└─────────────────────┬───────────────────────────────────────┘
                      │
                 Port Trait
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                    PORTS (Platform)                          │
│  Desktop (minifb)    Web (wasm-bindgen)    ESP32 (future)   │
│  - Owns main loop    - JS glue             - GPIO buttons   │
│  - Keyboard input    - Canvas2D            - SPI display    │
└─────────────────────────────────────────────────────────────┘
```

## Writing Apps

Apps are written in Rust with `no_std` and compiled to WASM:

```rust
#![no_std]
#![no_main]

use fri3d_sdk::{canvas, random};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn render() {
    canvas::set_color(canvas::Color::Black);
    let x = random::range(128) as i32;
    let y = random::range(64) as i32;
    canvas::draw_circle(x, y, 10);
}

#[no_mangle]
pub extern "C" fn on_input(key: u32, event_type: u32) {
    // Handle input
}
```

## Running Tests

```bash
# Run all visual tests
uv run tests/visual/run_visual_tests.py

# Update golden images
uv run tests/visual/run_visual_tests.py --update-golden
```

## Platform Support

| Platform          | Directory        | Status      |
| ----------------- | ---------------- | ----------- |
| Desktop Emulator  | `ports/desktop/` | Working     |
| Web Emulator      | `ports/web/`     | Working     |
| ESP32-S3 Firmware | `ports/esp32/`   | Scaffold    |
| WASM Apps         | `apps/`          | Working     |

## Dependencies

All pure Rust, no git submodules required:

- **wasmi** - WASM interpreter (no_std compatible)
- **minifb** - Desktop windowing
- **wasm-bindgen** - Web bindings
- **miniz_oxide** - PNG compression

## License

MIT
