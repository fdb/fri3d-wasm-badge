//! Fri3d Badge Desktop Emulator

use fri3d_png::encode_monochrome;
use fri3d_runtime::{Canvas, InputKey, InputManager, InputType, HEIGHT, WIDTH};
use fri3d_wasm::WasmApp;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;
use wasmi::Engine;

const SCALE: usize = 4;

/// Convert keyboard key to InputKey
fn key_to_input(key: Key) -> Option<InputKey> {
    match key {
        Key::Up => Some(InputKey::Up),
        Key::Down => Some(InputKey::Down),
        Key::Left => Some(InputKey::Left),
        Key::Right => Some(InputKey::Right),
        Key::Enter => Some(InputKey::Ok),
        Key::Backspace | Key::Escape => Some(InputKey::Back),
        _ => None,
    }
}

/// Convert 1-bit buffer to RGBA for minifb
fn canvas_to_buffer(canvas: &Canvas, buffer: &mut [u32]) {
    let src = canvas.buffer();

    // Colors (warm badge colors)
    const WHITE: u32 = 0xFFE7D396;
    const BLACK: u32 = 0xFF1A1A2E;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let byte_index = (y * WIDTH + x) / 8;
            let bit_index = 7 - (x % 8);
            let pixel = (src[byte_index] >> bit_index) & 1;
            let color = if pixel == 1 { BLACK } else { WHITE };

            // Scale up the pixel
            for sy in 0..SCALE {
                for sx in 0..SCALE {
                    let dest_x = x * SCALE + sx;
                    let dest_y = y * SCALE + sy;
                    buffer[dest_y * (WIDTH * SCALE) + dest_x] = color;
                }
            }
        }
    }
}

/// Save a screenshot
fn save_screenshot(canvas: &Canvas, path: &str) -> std::io::Result<()> {
    let png_data = encode_monochrome(WIDTH as u32, HEIGHT as u32, canvas.buffer());
    std::fs::write(path, png_data)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let mut test_mode = false;
    let mut screenshot_path: Option<String> = None;
    let mut scene: Option<u32> = None;
    let mut app_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--test" => test_mode = true,
            "--screenshot" => {
                i += 1;
                if i < args.len() {
                    screenshot_path = Some(args[i].clone());
                }
            }
            "--scene" => {
                i += 1;
                if i < args.len() {
                    scene = args[i].parse().ok();
                }
            }
            arg if !arg.starts_with('-') => {
                app_path = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    // Create WASM engine
    let engine = Engine::default();

    // Load app
    let wasm_bytes = if let Some(path) = &app_path {
        std::fs::read(path).expect("Failed to read WASM file")
    } else {
        // Try to load a default app
        let default_paths = [
            "build/apps/circles.wasm",
            "apps/circles/target/wasm32-unknown-unknown/release/circles.wasm",
        ];
        default_paths
            .iter()
            .find_map(|p| std::fs::read(p).ok())
            .expect("No WASM app found. Specify a path or build an app first.")
    };

    let mut app = WasmApp::load(&engine, &wasm_bytes).expect("Failed to load WASM app");

    // Set scene if specified
    if let Some(s) = scene {
        app.set_scene(s).expect("Failed to set scene");
    }

    // Test mode: render once, save screenshot, exit
    if test_mode {
        app.clear_canvas();
        app.render().expect("Render failed");

        if let Some(path) = &screenshot_path {
            save_screenshot(app.canvas(), path).expect("Failed to save screenshot");
            println!("Screenshot saved to {}", path);
        }
        return;
    }

    // Create window
    let mut window = Window::new(
        "Fri3d Badge Emulator",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    // Limit to ~60fps
    window.set_target_fps(60);

    let mut buffer = vec![0u32; WIDTH * SCALE * HEIGHT * SCALE];
    let mut input_manager = InputManager::new();
    let start_time = Instant::now();
    let mut prev_keys = Vec::new();

    // Set reset callback
    input_manager.set_reset_callback(|| {
        println!("Reset combo detected!");
        // In a full implementation, this would switch back to the launcher
    });

    while window.is_open() && !window.is_key_down(Key::Q) {
        let time_ms = start_time.elapsed().as_millis() as u32;

        // Process keyboard input
        let current_keys: Vec<Key> = window.get_keys_pressed(KeyRepeat::No);

        // Key down events
        for key in &current_keys {
            if let Some(input_key) = key_to_input(*key) {
                if !prev_keys.contains(key) {
                    input_manager.key_down(input_key, time_ms);
                }
            }
        }

        // Key up events
        for key in &prev_keys {
            if !window.is_key_down(*key) {
                if let Some(input_key) = key_to_input(*key) {
                    input_manager.key_up(input_key, time_ms);
                }
            }
        }

        prev_keys = window.get_keys();

        // Update input state
        input_manager.update(time_ms);

        // Process input events
        while let Some(event) = input_manager.poll_event() {
            // Only pass Short, Long, and Repeat to the app
            match event.event_type {
                InputType::Short | InputType::Long | InputType::Repeat => {
                    let _ = app.on_input(event.key, event.event_type);
                }
                _ => {}
            }
        }

        // Clear and render
        app.clear_canvas();
        app.render().expect("Render failed");

        // Convert canvas to display buffer
        canvas_to_buffer(app.canvas(), &mut buffer);

        // Update window
        window
            .update_with_buffer(&buffer, WIDTH * SCALE, HEIGHT * SCALE)
            .expect("Failed to update window");

        // Screenshot on 'S' key
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let path = format!("screenshot_{}.png", time_ms);
            if let Err(e) = save_screenshot(app.canvas(), &path) {
                eprintln!("Failed to save screenshot: {}", e);
            } else {
                println!("Screenshot saved to {}", path);
            }
        }
    }
}
