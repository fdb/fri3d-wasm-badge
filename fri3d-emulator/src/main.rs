use fri3d_runtime::app_manager::AppManager;
use fri3d_runtime::canvas::Canvas;
use fri3d_runtime::input::{InputBackend, InputEvent, InputManager};
use fri3d_runtime::random::Random;
use fri3d_runtime::types::{InputKey, InputType};
use minifb::{Key, Scale, Window, WindowOptions};
use png::{BitDepth, ColorType, Encoder};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

const SCALE: Scale = Scale::X4;

struct Options {
    wasm_file: Option<String>,
    screenshot_path: Option<PathBuf>,
    test_mode: bool,
    headless: bool,
    test_scene: Option<u32>,
}

struct MinifbInput {
    window: Rc<RefCell<Window>>,
    key_states: [bool; InputKey::COUNT],
    queue: VecDeque<InputEvent>,
}

impl MinifbInput {
    fn new(window: Rc<RefCell<Window>>) -> Self {
        Self {
            window,
            key_states: [false; InputKey::COUNT],
            queue: VecDeque::new(),
        }
    }

    fn poll_keys(&mut self) {
        let (up, down, left, right, ok, back) = {
            let window = self.window.borrow();
            (
                window.is_key_down(Key::Up),
                window.is_key_down(Key::Down),
                window.is_key_down(Key::Left),
                window.is_key_down(Key::Right),
                window.is_key_down(Key::Z) || window.is_key_down(Key::Enter),
                window.is_key_down(Key::X) || window.is_key_down(Key::Backspace),
            )
        };
        self.update_key(InputKey::Up, up);
        self.update_key(InputKey::Down, down);
        self.update_key(InputKey::Left, left);
        self.update_key(InputKey::Right, right);
        self.update_key(InputKey::Ok, ok);
        self.update_key(InputKey::Back, back);
    }

    fn update_key(&mut self, key: InputKey, is_down: bool) {
        let idx = key as usize;
        let was_down = self.key_states[idx];
        if is_down && !was_down {
            self.queue.push_back(InputEvent {
                key,
                kind: InputType::Press,
            });
        } else if !is_down && was_down {
            self.queue.push_back(InputEvent {
                key,
                kind: InputType::Release,
            });
        }
        self.key_states[idx] = is_down;
    }
}

impl InputBackend for MinifbInput {
    fn poll(&mut self) {
        self.poll_keys();
    }

    fn has_event(&self) -> bool {
        !self.queue.is_empty()
    }

    fn next_event(&mut self) -> Option<InputEvent> {
        self.queue.pop_front()
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [options] [wasm_file]\n");
    eprintln!("Options:");
    eprintln!("  --test              Run in test mode (render and exit)");
    eprintln!("  --scene <n>         Set scene number (for test apps)");
    eprintln!("  --screenshot <path> Save screenshot to path (PNG format)");
    eprintln!("  --headless          Run without display (requires --screenshot)");
    eprintln!("  --help              Show this help\n");
    eprintln!("If no wasm_file is specified, runs the launcher.wasm app.");
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        wasm_file: None,
        screenshot_path: None,
        test_mode: false,
        headless: false,
        test_scene: None,
    };

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--test" => {
                options.test_mode = true;
            }
            "--scene" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("Missing value for --scene".to_string());
                }
                let scene = args[idx]
                    .parse::<u32>()
                    .map_err(|_| "Invalid scene value".to_string())?;
                options.test_scene = Some(scene);
            }
            "--screenshot" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("Missing value for --screenshot".to_string());
                }
                options.screenshot_path = Some(PathBuf::from(&args[idx]));
            }
            "--headless" => {
                options.headless = true;
            }
            "--help" => {
                return Err(String::new());
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: {arg}"));
            }
            arg => {
                if options.wasm_file.is_some() {
                    return Err("Multiple wasm files specified".to_string());
                }
                options.wasm_file = Some(arg.to_string());
            }
        }
        idx += 1;
    }

    if options.headless && options.screenshot_path.is_none() {
        return Err("--headless requires --screenshot".to_string());
    }

    Ok(options)
}

fn save_screenshot(canvas: &Canvas, path: &Path) -> Result<(), String> {
    let width = canvas.width();
    let height = canvas.height();
    let mut data = Vec::with_capacity((width * height) as usize);
    for &pixel in canvas.buffer() {
        data.push(if pixel == 0 { 0xFF } else { 0x00 });
    }

    let file = File::create(path).map_err(|err| err.to_string())?;
    let mut encoder = Encoder::new(file, width, height);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|err| err.to_string())?;
    writer
        .write_image_data(&data)
        .map_err(|err| err.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args
        .get(0)
        .map(String::as_str)
        .unwrap_or("fri3d-emulator");
    let options = match parse_args(&args) {
        Ok(options) => options,
        Err(message) => {
            if !message.is_empty() {
                eprintln!("Error: {message}");
            }
            print_usage(program);
            return;
        }
    };

    let canvas = Rc::new(RefCell::new(Canvas::new()));
    let random = Rc::new(RefCell::new(Random::new(0)));

    let app_manager = AppManager::new(Rc::clone(&canvas), Rc::clone(&random));
    let app_manager = match app_manager {
        Ok(manager) => Rc::new(RefCell::new(manager)),
        Err(err) => {
            eprintln!("Failed to initialize runtime: {err}");
            return;
        }
    };

    {
        let mut manager = app_manager.borrow_mut();
        manager.set_launcher_path("build/apps/launcher/launcher.wasm");
        manager.add_app("Circles", "build/apps/circles/circles.wasm");
        manager.add_app("Mandelbrot", "build/apps/mandelbrot/mandelbrot.wasm");
        manager.add_app("Test Drawing", "build/apps/test_drawing/test_drawing.wasm");
        manager.add_app("Test UI", "build/apps/test_ui/test_ui.wasm");
        manager.add_app("Snake", "build/apps/snake/snake.wasm");
    }

    if let Some(path) = options.wasm_file.as_ref() {
        if !app_manager.borrow_mut().launch_app_by_path(path) {
            let err = app_manager.borrow().last_error().to_string();
            eprintln!("Failed to load app: {err}");
        }
    } else {
        app_manager.borrow_mut().show_launcher();
    }

    let start = Instant::now();
    {
        let start_for_time = start;
        app_manager
            .borrow_mut()
            .wasm_runner_mut()
            .set_time_provider(move || start_for_time.elapsed().as_millis() as u32);
    }

    if options.test_mode || options.screenshot_path.is_some() {
        if let Some(scene) = options.test_scene {
            app_manager.borrow_mut().wasm_runner_mut().set_scene(scene);
        }

        app_manager.borrow_mut().render();

        if let Some(path) = options.screenshot_path.as_ref() {
            if let Err(err) = save_screenshot(&canvas.borrow(), path) {
                eprintln!("Failed to save screenshot: {err}");
                return;
            }
            println!("Screenshot saved to {}", path.display());
        }

        if !options.headless {
            let width = canvas.borrow().width() as usize;
            let height = canvas.borrow().height() as usize;
            let mut buffer = vec![0u32; width * height];

            if let Ok(mut window) = Window::new(
                "Fri3d Emulator (Rust)",
                width,
                height,
                WindowOptions {
                    resize: false,
                    scale: SCALE,
                    ..WindowOptions::default()
                },
            ) {
                window.limit_update_rate(Some(Duration::from_millis(16)));
                render_canvas(&canvas.borrow(), &mut buffer);
                let _ = window.update_with_buffer(&buffer, width, height);
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        return;
    }

    let mut window = Window::new(
        "Fri3d Emulator (Rust)",
        canvas.borrow().width() as usize,
        canvas.borrow().height() as usize,
        WindowOptions {
            resize: false,
            scale: SCALE,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to open window");
    window.limit_update_rate(Some(Duration::from_millis(16)));

    let window = Rc::new(RefCell::new(window));
    let mut input = MinifbInput::new(Rc::clone(&window));
    let mut input_manager = InputManager::new();

    {
        let manager = Rc::clone(&app_manager);
        input_manager.set_reset_callback(move || {
            manager.borrow_mut().show_launcher();
        });
    }

    let width = canvas.borrow().width() as usize;
    let height = canvas.borrow().height() as usize;
    let mut buffer = vec![0u32; width * height];

    app_manager.borrow_mut().render();
    render_canvas(&canvas.borrow(), &mut buffer);
    if window
        .borrow_mut()
        .update_with_buffer(&buffer, width, height)
        .is_err()
    {
        return;
    }

    let mut needs_render = false;

    while window.borrow().is_open() {
        let time_ms = start.elapsed().as_millis() as u32;
        input_manager.update(&mut input, time_ms);

        let mut had_event = false;
        while let Some(event) = input_manager.next_event() {
            app_manager
                .borrow_mut()
                .handle_input(event.key, event.kind);
            had_event = true;
        }

        needs_render |= had_event;
        if needs_render {
            app_manager.borrow_mut().render();
            render_canvas(&canvas.borrow(), &mut buffer);
            needs_render = false;
        }

        if window
            .borrow_mut()
            .update_with_buffer(&buffer, width, height)
            .is_err()
        {
            break;
        }
    }
}

fn render_canvas(canvas: &Canvas, buffer: &mut [u32]) {
    let pixels = canvas.buffer();
    for (idx, out_pixel) in buffer.iter_mut().enumerate() {
        let value = pixels.get(idx).copied().unwrap_or(0);
        *out_pixel = if value == 0 { 0x00FFFFFF } else { 0x00000000 };
    }
}
