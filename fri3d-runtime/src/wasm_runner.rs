use crate::canvas::Canvas;
use crate::random::Random;
use crate::trace;
use crate::types::{Color, Font};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use wasmi::{Caller, Engine, Extern, Instance, Linker, Module, Store, TypedFunc};

pub type ExitToLauncherCallback = Rc<dyn Fn()>;
pub type StartAppCallback = Rc<dyn Fn(u32)>;
pub type TimeMsCallback = Rc<dyn Fn() -> u32>;

#[derive(Copy, Clone, Debug, Default)]
struct TimerState {
    interval_ms: Option<u32>,
    next_ms: Option<u32>,
}

impl TimerState {
    fn start(&mut self, now_ms: u32, interval_ms: u32) {
        if interval_ms == 0 {
            self.stop();
            return;
        }
        self.interval_ms = Some(interval_ms);
        self.next_ms = Some(now_ms.saturating_add(interval_ms));
    }

    fn stop(&mut self) {
        self.interval_ms = None;
        self.next_ms = None;
    }

    fn due(&mut self, now_ms: u32) -> bool {
        let (Some(interval), Some(next_ms)) = (self.interval_ms, self.next_ms) else {
            return false;
        };

        if now_ms < next_ms {
            return false;
        }

        let mut next = next_ms;
        while now_ms >= next {
            next = next.saturating_add(interval);
        }
        self.next_ms = Some(next);
        true
    }

    fn next_ms(&self) -> Option<u32> {
        self.next_ms
    }
}

#[derive(Default)]
struct HostState {
    canvas: Option<Rc<RefCell<Canvas>>>,
    random: Option<Rc<RefCell<Random>>>,
    time_ms: Option<TimeMsCallback>,
    exit_to_launcher: Option<ExitToLauncherCallback>,
    start_app: Option<StartAppCallback>,
    render_requested: bool,
    timer: TimerState,
}

pub struct WasmRunner {
    engine: Engine,
    linker: Linker<HostState>,
    store: Store<HostState>,
    instance: Option<Instance>,
    func_render: Option<TypedFunc<(), ()>>,
    func_on_input: Option<TypedFunc<(i32, i32), ()>>,
    func_set_scene: Option<TypedFunc<(i32,), ()>>,
    func_get_scene: Option<TypedFunc<(), i32>>,
    func_get_scene_count: Option<TypedFunc<(), i32>>,
    last_error: String,
}

impl WasmRunner {
    pub fn new() -> Self {
        let engine = Engine::default();
        let store = Store::new(&engine, HostState::default());
        let mut linker = Linker::new(&engine);
        register_host_functions(&mut linker);

        Self {
            engine,
            linker,
            store,
            instance: None,
            func_render: None,
            func_on_input: None,
            func_set_scene: None,
            func_get_scene: None,
            func_get_scene_count: None,
            last_error: String::new(),
        }
    }

    pub fn set_canvas(&mut self, canvas: Rc<RefCell<Canvas>>) {
        self.store.data_mut().canvas = Some(canvas);
    }

    pub fn set_random(&mut self, random: Rc<RefCell<Random>>) {
        self.store.data_mut().random = Some(random);
    }

    pub fn set_time_provider<F>(&mut self, callback: F)
    where
        F: Fn() -> u32 + 'static,
    {
        self.store.data_mut().time_ms = Some(Rc::new(callback));
    }

    pub fn set_app_callbacks(
        &mut self,
        exit_cb: ExitToLauncherCallback,
        start_cb: StartAppCallback,
    ) {
        self.store.data_mut().exit_to_launcher = Some(exit_cb);
        self.store.data_mut().start_app = Some(start_cb);
    }

    pub fn load_module(&mut self, path: &str) -> bool {
        let data = match fs::read(path) {
            Ok(data) => data,
            Err(err) => {
                self.set_error(&format!("Failed to open: {path} ({err})"));
                return false;
            }
        };
        self.load_module_from_bytes(&data)
    }

    pub fn load_module_from_bytes(&mut self, data: &[u8]) -> bool {
        self.unload_module();

        let module = match Module::new(&self.engine, data) {
            Ok(module) => module,
            Err(err) => {
                self.set_error(&format!("Failed to load module: {err}"));
                return false;
            }
        };

        let instance = match self
            .linker
            .instantiate(&mut self.store, &module)
            .and_then(|pre| pre.start(&mut self.store))
        {
            Ok(instance) => instance,
            Err(err) => {
                self.set_error(&format!("Failed to instantiate module: {err}"));
                return false;
            }
        };

        let render = match instance.get_typed_func::<(), ()>(&mut self.store, "render") {
            Ok(func) => func,
            Err(_) => {
                self.set_error("Module missing required 'render' function");
                return false;
            }
        };

        self.func_render = Some(render);
        self.func_on_input = instance
            .get_typed_func::<(i32, i32), ()>(&mut self.store, "on_input")
            .ok();
        self.func_set_scene = instance
            .get_typed_func::<(i32,), ()>(&mut self.store, "set_scene")
            .ok();
        self.func_get_scene = instance
            .get_typed_func::<(), i32>(&mut self.store, "get_scene")
            .ok();
        self.func_get_scene_count = instance
            .get_typed_func::<(), i32>(&mut self.store, "get_scene_count")
            .ok();

        self.instance = Some(instance);
        true
    }

    pub fn unload_module(&mut self) {
        self.instance = None;
        self.func_render = None;
        self.func_on_input = None;
        self.func_set_scene = None;
        self.func_get_scene = None;
        self.func_get_scene_count = None;
        let state = self.store.data_mut();
        state.render_requested = false;
        state.timer.stop();
    }

    pub fn is_module_loaded(&self) -> bool {
        self.instance.is_some()
    }

    pub fn call_render(&mut self) {
        let Some(func) = self.func_render else {
            return;
        };
        if let Some(canvas) = &self.store.data().canvas {
            canvas.borrow_mut().clear();
        }
        if let Err(err) = func.call(&mut self.store, ()) {
            self.set_error(&format!("WASM error in render: {err}"));
            eprintln!("WASM error in render: {err}");
        }
    }

    pub fn call_on_input(&mut self, key: u32, kind: u32) {
        let Some(func) = self.func_on_input else {
            return;
        };

        trace::trace_call(
            "on_input",
            &[trace::TraceArg::Int(key as i64), trace::TraceArg::Int(kind as i64)],
        );

        if let Err(err) = func.call(&mut self.store, (key as i32, kind as i32)) {
            self.set_error(&format!("WASM error in on_input: {err}"));
            eprintln!("WASM error in on_input: {err}");
        }
    }

    pub fn get_scene_count(&mut self) -> u32 {
        let Some(func) = self.func_get_scene_count else {
            return 0;
        };
        match func.call(&mut self.store, ()) {
            Ok(result) => result as u32,
            Err(_) => 0,
        }
    }

    pub fn set_scene(&mut self, scene: u32) {
        let Some(func) = self.func_set_scene else {
            return;
        };
        let _ = func.call(&mut self.store, (scene as i32,));
    }

    pub fn get_scene(&mut self) -> u32 {
        let Some(func) = self.func_get_scene else {
            return 0;
        };
        match func.call(&mut self.store, ()) {
            Ok(result) => result as u32,
            Err(_) => 0,
        }
    }

    pub fn take_render_request(&mut self) -> bool {
        let state = self.store.data_mut();
        let requested = state.render_requested;
        state.render_requested = false;
        requested
    }

    pub fn timer_due(&mut self, now_ms: u32) -> bool {
        let state = self.store.data_mut();
        state.timer.due(now_ms)
    }

    pub fn timer_next_ms(&self) -> Option<u32> {
        self.store.data().timer.next_ms()
    }

    pub fn last_error(&self) -> &str {
        &self.last_error
    }

    fn set_error(&mut self, message: &str) {
        self.last_error = message.to_string();
    }
}

fn register_host_functions(linker: &mut Linker<HostState>) {
    linker
        .func_wrap("env", "canvas_clear", |caller: Caller<'_, HostState>| {
            trace::trace_call("canvas_clear", &[]);
            if let Some(canvas) = &caller.data().canvas {
                canvas.borrow_mut().clear();
            }
        })
        .expect("register canvas_clear");

    linker
        .func_wrap("env", "canvas_width", |caller: Caller<'_, HostState>| -> i32 {
            trace::trace_call("canvas_width", &[]);
            let width = caller
                .data()
                .canvas
                .as_ref()
                .map(|canvas| canvas.borrow().width() as i32)
                .unwrap_or(0);
            trace::trace_result(width as i64);
            width
        })
        .expect("register canvas_width");

    linker
        .func_wrap("env", "canvas_height", |caller: Caller<'_, HostState>| -> i32 {
            trace::trace_call("canvas_height", &[]);
            let height = caller
                .data()
                .canvas
                .as_ref()
                .map(|canvas| canvas.borrow().height() as i32)
                .unwrap_or(0);
            trace::trace_result(height as i64);
            height
        })
        .expect("register canvas_height");

    linker
        .func_wrap("env", "canvas_set_color", |caller: Caller<'_, HostState>, color: i32| {
            trace::trace_call("canvas_set_color", &[trace::TraceArg::Int(color as i64)]);
            if let Some(canvas) = &caller.data().canvas {
                canvas.borrow_mut().set_color(match color {
                    0 => Color::White,
                    1 => Color::Black,
                    _ => Color::Xor,
                });
            }
        })
        .expect("register canvas_set_color");

    linker
        .func_wrap("env", "canvas_set_font", |caller: Caller<'_, HostState>, font: i32| {
            trace::trace_call("canvas_set_font", &[trace::TraceArg::Int(font as i64)]);
            if let Some(canvas) = &caller.data().canvas {
                canvas.borrow_mut().set_font(match font {
                    0 => Font::Primary,
                    1 => Font::Secondary,
                    2 => Font::Keyboard,
                    _ => Font::BigNumbers,
                });
            }
        })
        .expect("register canvas_set_font");

    linker
        .func_wrap(
            "env",
            "canvas_draw_dot",
            |caller: Caller<'_, HostState>, x: i32, y: i32| {
                trace::trace_call(
                    "canvas_draw_dot",
                    &[trace::TraceArg::Int(x as i64), trace::TraceArg::Int(y as i64)],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_dot(x, y);
                }
            },
        )
        .expect("register canvas_draw_dot");

    linker
        .func_wrap(
            "env",
            "canvas_draw_line",
            |caller: Caller<'_, HostState>, x1: i32, y1: i32, x2: i32, y2: i32| {
                trace::trace_call(
                    "canvas_draw_line",
                    &[
                        trace::TraceArg::Int(x1 as i64),
                        trace::TraceArg::Int(y1 as i64),
                        trace::TraceArg::Int(x2 as i64),
                        trace::TraceArg::Int(y2 as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_line(x1, y1, x2, y2);
                }
            },
        )
        .expect("register canvas_draw_line");

    linker
        .func_wrap(
            "env",
            "canvas_draw_frame",
            |caller: Caller<'_, HostState>, x: i32, y: i32, width: i32, height: i32| {
                trace::trace_call(
                    "canvas_draw_frame",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(width as i64),
                        trace::TraceArg::Int(height as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_frame(x, y, width as u32, height as u32);
                }
            },
        )
        .expect("register canvas_draw_frame");

    linker
        .func_wrap(
            "env",
            "canvas_draw_box",
            |caller: Caller<'_, HostState>, x: i32, y: i32, width: i32, height: i32| {
                trace::trace_call(
                    "canvas_draw_box",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(width as i64),
                        trace::TraceArg::Int(height as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_box(x, y, width as u32, height as u32);
                }
            },
        )
        .expect("register canvas_draw_box");

    linker
        .func_wrap(
            "env",
            "canvas_draw_rframe",
            |caller: Caller<'_, HostState>,
             x: i32,
             y: i32,
             width: i32,
             height: i32,
             radius: i32| {
                trace::trace_call(
                    "canvas_draw_rframe",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(width as i64),
                        trace::TraceArg::Int(height as i64),
                        trace::TraceArg::Int(radius as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas
                        .borrow_mut()
                        .draw_rframe(x, y, width as u32, height as u32, radius as u32);
                }
            },
        )
        .expect("register canvas_draw_rframe");

    linker
        .func_wrap(
            "env",
            "canvas_draw_rbox",
            |caller: Caller<'_, HostState>,
             x: i32,
             y: i32,
             width: i32,
             height: i32,
             radius: i32| {
                trace::trace_call(
                    "canvas_draw_rbox",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(width as i64),
                        trace::TraceArg::Int(height as i64),
                        trace::TraceArg::Int(radius as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas
                        .borrow_mut()
                        .draw_rbox(x, y, width as u32, height as u32, radius as u32);
                }
            },
        )
        .expect("register canvas_draw_rbox");

    linker
        .func_wrap(
            "env",
            "canvas_draw_circle",
            |caller: Caller<'_, HostState>, x: i32, y: i32, radius: i32| {
                trace::trace_call(
                    "canvas_draw_circle",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(radius as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_circle(x, y, radius as u32);
                }
            },
        )
        .expect("register canvas_draw_circle");

    linker
        .func_wrap(
            "env",
            "canvas_draw_disc",
            |caller: Caller<'_, HostState>, x: i32, y: i32, radius: i32| {
                trace::trace_call(
                    "canvas_draw_disc",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Int(radius as i64),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_disc(x, y, radius as u32);
                }
            },
        )
        .expect("register canvas_draw_disc");

    linker
        .func_wrap(
            "env",
            "canvas_draw_str",
            |mut caller: Caller<'_, HostState>, x: i32, y: i32, ptr: i32| {
                let text = read_c_string(&mut caller, ptr);
                trace::trace_call(
                    "canvas_draw_str",
                    &[
                        trace::TraceArg::Int(x as i64),
                        trace::TraceArg::Int(y as i64),
                        trace::TraceArg::Str(text.clone()),
                    ],
                );
                if let Some(canvas) = &caller.data().canvas {
                    canvas.borrow_mut().draw_str(x, y, &text);
                }
            },
        )
        .expect("register canvas_draw_str");

    linker
        .func_wrap(
            "env",
            "canvas_string_width",
            |mut caller: Caller<'_, HostState>, ptr: i32| -> i32 {
                let text = read_c_string(&mut caller, ptr);
                trace::trace_call("canvas_string_width", &[trace::TraceArg::Str(text.clone())]);
                let width = caller
                    .data()
                    .canvas
                    .as_ref()
                    .map(|canvas| canvas.borrow().string_width(&text) as i32)
                    .unwrap_or(0);
                trace::trace_result(width as i64);
                width
            },
        )
        .expect("register canvas_string_width");

    linker
        .func_wrap("env", "random_seed", |caller: Caller<'_, HostState>, seed: i32| {
            trace::trace_call("random_seed", &[trace::TraceArg::Int(seed as i64)]);
            if let Some(random) = &caller.data().random {
                random.borrow_mut().seed(seed as u32);
            }
        })
        .expect("register random_seed");

    linker
        .func_wrap("env", "random_get", |caller: Caller<'_, HostState>| -> i32 {
            trace::trace_call("random_get", &[]);
            let value = caller
                .data()
                .random
                .as_ref()
                .map(|random| random.borrow_mut().get() as i32)
                .unwrap_or(0);
            trace::trace_result(value as i64);
            value
        })
        .expect("register random_get");

    linker
        .func_wrap(
            "env",
            "random_range",
            |caller: Caller<'_, HostState>, max: i32| -> i32 {
                trace::trace_call("random_range", &[trace::TraceArg::Int(max as i64)]);
                let value = caller
                    .data()
                    .random
                    .as_ref()
                    .map(|random| random.borrow_mut().range(max.max(0) as u32) as i32)
                    .unwrap_or(0);
                trace::trace_result(value as i64);
                value
            },
        )
        .expect("register random_range");

    linker
        .func_wrap("env", "get_time_ms", |caller: Caller<'_, HostState>| -> i32 {
            trace::trace_call("get_time_ms", &[]);
            let value = caller
                .data()
                .time_ms
                .as_ref()
                .map(|cb| cb() as i32)
                .unwrap_or(0);
            trace::trace_result(value as i64);
            value
        })
        .expect("register get_time_ms");

    linker
        .func_wrap(
            "env",
            "start_timer_ms",
            |mut caller: Caller<'_, HostState>, interval_ms: i32| {
                trace::trace_call(
                    "start_timer_ms",
                    &[trace::TraceArg::Int(interval_ms as i64)],
                );
                let interval = interval_ms.max(0) as u32;
                let now = caller
                    .data()
                    .time_ms
                    .as_ref()
                    .map(|cb| cb())
                    .unwrap_or(0);
                caller.data_mut().timer.start(now, interval);
            },
        )
        .expect("register start_timer_ms");

    linker
        .func_wrap("env", "stop_timer", |mut caller: Caller<'_, HostState>| {
            trace::trace_call("stop_timer", &[]);
            caller.data_mut().timer.stop();
        })
        .expect("register stop_timer");

    linker
        .func_wrap("env", "request_render", |mut caller: Caller<'_, HostState>| {
            trace::trace_call("request_render", &[]);
            caller.data_mut().render_requested = true;
        })
        .expect("register request_render");

    linker
        .func_wrap("env", "exit_to_launcher", |caller: Caller<'_, HostState>| {
            trace::trace_call("exit_to_launcher", &[]);
            if let Some(cb) = &caller.data().exit_to_launcher {
                cb();
            }
        })
        .expect("register exit_to_launcher");

    linker
        .func_wrap("env", "start_app", |caller: Caller<'_, HostState>, app_id: i32| {
            trace::trace_call("start_app", &[trace::TraceArg::Int(app_id as i64)]);
            if let Some(cb) = &caller.data().start_app {
                cb(app_id as u32);
            }
        })
        .expect("register start_app");
}

fn read_c_string(caller: &mut Caller<'_, HostState>, ptr: i32) -> String {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => memory,
        _ => return String::new(),
    };

    let data = memory.data(caller);
    let start = ptr.max(0) as usize;
    if start >= data.len() {
        return String::new();
    }

    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }

    String::from_utf8_lossy(&data[start..end]).to_string()
}

#[cfg(test)]
mod tests {
    use super::TimerState;

    #[test]
    fn timer_not_due_before_interval() {
        let mut timer = TimerState::default();
        timer.start(1000, 100);
        assert!(!timer.due(1000));
        assert!(!timer.due(1099));
        assert!(timer.due(1100));
    }

    #[test]
    fn timer_catches_up_on_large_jump() {
        let mut timer = TimerState::default();
        timer.start(1000, 100);
        assert!(timer.due(1350));
        assert!(!timer.due(1399));
        assert!(timer.due(1400));
    }

    #[test]
    fn timer_stop_disables_due() {
        let mut timer = TimerState::default();
        timer.start(500, 50);
        timer.stop();
        assert!(!timer.due(1000));
    }
}
