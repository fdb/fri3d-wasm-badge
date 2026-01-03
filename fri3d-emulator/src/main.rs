use fri3d_runtime::app_manager::AppManager;
use fri3d_runtime::canvas::Canvas;
use fri3d_runtime::input::{InputBackend, InputEvent, InputManager};
use fri3d_runtime::random::Random;
use fri3d_runtime::types::{InputKey, InputType};
use minifb::{Key, Scale, Window, WindowOptions};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

const SCALE: Scale = Scale::X4;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let app_path = args.get(1).cloned();

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

    if let Some(path) = app_path {
        if !app_manager.borrow_mut().launch_app_by_path(&path) {
            let err = app_manager.borrow().last_error().to_string();
            eprintln!("Failed to load app: {err}");
        }
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

    let start = Instant::now();
    {
        let start_for_time = start;
        app_manager
            .borrow_mut()
            .wasm_runner_mut()
            .set_time_provider(move || start_for_time.elapsed().as_millis() as u32);
    }

    let mut buffer = vec![0u32; (canvas.borrow().width() * canvas.borrow().height()) as usize];
    let mut needs_render = true;

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

            if window
                .borrow_mut()
                .update_with_buffer(
                    &buffer,
                    canvas.borrow().width() as usize,
                    canvas.borrow().height() as usize,
                )
                .is_err()
            {
                break;
            }
        } else {
            window.borrow_mut().update();
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
