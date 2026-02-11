use crate::canvas::Canvas;
use crate::random::Random;
use crate::types::{InputKey, InputType};
use crate::wasm_runner::{StartAppCallback, WasmRunner};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
struct AppEntry {
    id: u32,
    path: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AppRequest {
    None,
    ExitToLauncher,
    StartApp,
}

#[derive(Copy, Clone, Debug)]
struct PendingRequest {
    request: AppRequest,
    app_id: u32,
}

pub struct AppManager {
    apps: Vec<AppEntry>,
    launcher_path: Option<String>,
    in_launcher: bool,
    last_error: String,
    canvas: Rc<RefCell<Canvas>>,
    _random: Rc<RefCell<Random>>,
    wasm_runner: WasmRunner,
    pending: Rc<RefCell<PendingRequest>>,
}

impl AppManager {
    pub fn new(canvas: Rc<RefCell<Canvas>>, random: Rc<RefCell<Random>>) -> Result<Self, String> {
        let mut wasm_runner = WasmRunner::new();
        wasm_runner.set_canvas(Rc::clone(&canvas));
        wasm_runner.set_random(Rc::clone(&random));

        let pending = Rc::new(RefCell::new(PendingRequest {
            request: AppRequest::None,
            app_id: 0,
        }));

        let exit_cb = {
            let pending = Rc::clone(&pending);
            Rc::new(move || {
                let mut state = pending.borrow_mut();
                state.request = AppRequest::ExitToLauncher;
                state.app_id = 0;
            })
        };
        let start_cb: StartAppCallback = {
            let pending = Rc::clone(&pending);
            Rc::new(move |app_id| {
                let mut state = pending.borrow_mut();
                state.request = AppRequest::StartApp;
                state.app_id = app_id;
            })
        };
        wasm_runner.set_app_callbacks(exit_cb, start_cb);

        Ok(Self {
            apps: Vec::new(),
            launcher_path: None,
            in_launcher: true,
            last_error: String::new(),
            canvas,
            _random: random,
            wasm_runner,
            pending,
        })
    }

    pub fn add_app(&mut self, _name: &str, path: &str) -> bool {
        let id = (self.apps.len() + 1) as u32;
        self.apps.push(AppEntry {
            id,
            path: path.to_string(),
        });
        true
    }

    pub fn set_launcher_path(&mut self, path: &str) {
        self.launcher_path = Some(path.to_string());
    }

    pub fn show_launcher(&mut self) {
        self.wasm_runner.unload_module();
        self.in_launcher = true;
        let Some(path) = self.launcher_path.clone() else {
            self.set_error("Launcher path not set");
            return;
        };

        if !self.wasm_runner.load_module(&path) {
            let err = self.wasm_runner.last_error().to_string();
            self.set_error(&err);
        }
    }

    pub fn launch_app_by_path(&mut self, path: &str) -> bool {
        if !self.wasm_runner.load_module(path) {
            let err = self.wasm_runner.last_error().to_string();
            self.set_error(&err);
            return false;
        }
        self.in_launcher = false;
        true
    }

    pub fn render(&mut self) {
        let mut passes = 0;
        loop {
            if self.in_launcher {
                if self.wasm_runner.is_module_loaded() {
                    self.wasm_runner.call_render();
                } else {
                    self.render_launcher_error();
                }
            } else {
                self.wasm_runner.call_render();
            }

            let switched = self.process_pending();
            passes += 1;

            if passes >= 2 {
                break;
            }
            let rerender = switched || self.wasm_runner.take_render_request();
            if !rerender {
                break;
            }
            if !self.wasm_runner.is_module_loaded() && !self.in_launcher {
                break;
            }
        }
    }

    pub fn handle_input(&mut self, key: InputKey, kind: InputType) {
        if self.in_launcher {
            if self.wasm_runner.is_module_loaded() {
                self.wasm_runner.call_on_input(key as u32, kind as u32);
            }
        } else {
            self.wasm_runner.call_on_input(key as u32, kind as u32);
        }

        self.process_pending();
    }

    pub fn poll_timer(&mut self, now_ms: u32) -> bool {
        if !self.wasm_runner.is_module_loaded() {
            return false;
        }
        self.wasm_runner.timer_due(now_ms)
    }

    pub fn last_error(&self) -> &str {
        &self.last_error
    }

    pub fn wasm_runner(&self) -> &WasmRunner {
        &self.wasm_runner
    }

    pub fn wasm_runner_mut(&mut self) -> &mut WasmRunner {
        &mut self.wasm_runner
    }

    fn process_pending(&mut self) -> bool {
        let (request, app_id) = {
            let mut pending = self.pending.borrow_mut();
            let request = pending.request;
            let app_id = pending.app_id;
            pending.request = AppRequest::None;
            pending.app_id = 0;
            (request, app_id)
        };

        match request {
            AppRequest::None => false,
            AppRequest::ExitToLauncher => {
                self.show_launcher();
                true
            }
            AppRequest::StartApp => self.launch_app_by_id(app_id),
        }
    }

    fn launch_app_by_id(&mut self, app_id: u32) -> bool {
        if app_id == 0 {
            self.show_launcher();
            return true;
        }
        let path = self
            .apps
            .iter()
            .find(|entry| entry.id == app_id)
            .map(|entry| entry.path.clone());
        if let Some(path) = path {
            return self.launch_app_by_path(&path);
        }
        self.set_error("Invalid app id");
        false
    }

    fn render_launcher_error(&mut self) {
        let mut canvas = self.canvas.borrow_mut();
        canvas.clear();
        canvas.set_color(crate::types::Color::Black);
        canvas.set_font(crate::types::Font::Primary);
        canvas.draw_str(2, 12, "Error loading launcher");
    }

    fn set_error(&mut self, message: &str) {
        self.last_error = message.to_string();
    }
}
