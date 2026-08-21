//! wasmi host: one `AppInstance` per running module, the `env` imports it
//! sees, and the shared state those imports touch.
//!
//! Every host import follows the same shape: validate arguments, borrow
//! the shared state, do the work on slices that already exist. No import
//! allocates.

use crate::bundle::{Bundle, HEADER_LEN};
use crate::canvas::Canvas;
use crate::limits::{APP_MEMORY_MAX, FUEL_PER_CALL, LOG_LINES, LOG_LINE_LEN};
use crate::random::Random;
use crate::registry::Registry;
use crate::settings::{Settings, SYSTEM_NS};
use crate::types::{Color, Font};
use crate::KERNEL_VERSION;
use alloc::rc::Rc;
use core::cell::RefCell;
use heapless::{Deque, String};
use wasmi::{
    Caller, CompilationMode, Config, Engine, Error as WasmiError, Extern, Instance, Linker, Memory,
    Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc,
};

/// State shared by every instance: the one canvas, the one RNG, the
/// registry, settings, and the mailbox apps use to talk to the kernel.
pub struct Shared {
    pub canvas: Canvas,
    pub random: Random,
    pub registry: Registry,
    pub settings: Settings,
    pub now_ms: u32,
    pub request: AppRequest,
    pub log: Deque<String<LOG_LINE_LEN>, LOG_LINES>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            random: Random::new(42),
            registry: Registry::new(),
            settings: Settings::new(),
            now_ms: 0,
            request: AppRequest::None,
            log: Deque::new(),
        }
    }

    pub fn log_line(&mut self, line: &str) {
        let mut s: String<LOG_LINE_LEN> = String::new();
        let _ = s.push_str(&line[..line.len().min(LOG_LINE_LEN)]);
        if self.log.is_full() {
            self.log.pop_front();
        }
        let _ = self.log.push_back(s);
    }
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedRef = Rc<RefCell<Shared>>;

/// Requests an app can make of the kernel. Applied after the current call
/// returns, so the app never observes itself being torn down.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AppRequest {
    None,
    ExitToLauncher,
    StartApp(u32),
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TimerState {
    interval_ms: Option<u32>,
    next_ms: Option<u32>,
}

impl TimerState {
    pub fn start(&mut self, now_ms: u32, interval_ms: u32) {
        if interval_ms == 0 {
            self.stop();
            return;
        }
        self.interval_ms = Some(interval_ms);
        self.next_ms = Some(now_ms.wrapping_add(interval_ms));
    }

    pub fn stop(&mut self) {
        self.interval_ms = None;
        self.next_ms = None;
    }

    /// True when the timer fired. Skips missed periods instead of
    /// bursting, so a slow frame never produces a backlog of renders.
    pub fn due(&mut self, now_ms: u32) -> bool {
        let (Some(interval), Some(next)) = (self.interval_ms, self.next_ms) else {
            return false;
        };
        if (now_ms.wrapping_sub(next) as i32) < 0 {
            return false;
        }
        let mut next = next;
        let mut guard = 0u32;
        while (now_ms.wrapping_sub(next) as i32) >= 0 && guard < 1024 {
            next = next.wrapping_add(interval);
            guard += 1;
        }
        if guard == 1024 {
            next = now_ms.wrapping_add(interval);
        }
        self.next_ms = Some(next);
        true
    }

    pub fn next_ms(&self) -> Option<u32> {
        self.next_ms
    }
}

/// Per-store host data.
pub struct HostState {
    shared: SharedRef,
    app_id: String<24>,
    is_system: bool,
    limits: StoreLimits,
    pub timer: TimerState,
    pub render_requested: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallError {
    OutOfFuel,
    Trap(String<96>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    Compile(String<96>),
    Instantiate(String<96>),
    MissingRender,
    MissingMemory,
}

fn clip(msg: &str) -> String<96> {
    let mut s = String::new();
    let mut end = msg.len().min(96);
    while end > 0 && !msg.is_char_boundary(end) {
        end -= 1;
    }
    let _ = s.push_str(&msg[..end]);
    s
}

fn describe(err: &WasmiError) -> String<96> {
    let mut s: String<160> = String::new();
    let _ = core::fmt::write(&mut s, format_args!("{err}"));
    clip(&s)
}

pub fn make_engine() -> Engine {
    let mut config = Config::default();
    config.consume_fuel(true);
    config.compilation_mode(CompilationMode::Eager);
    // Rust-on-wasm32 never needs these; disabling them keeps apps on the
    // portable subset.
    config.wasm_multi_memory(false);
    Engine::new(&config)
}

/// One running wasm module with its own store, memory and fuel budget.
pub struct AppInstance {
    store: Store<HostState>,
    _instance: Instance,
    memory: Memory,
    render: TypedFunc<(), ()>,
    on_input: Option<TypedFunc<(i32, i32), ()>>,
    on_start: Option<TypedFunc<(), ()>>,
    on_stop: Option<TypedFunc<(), ()>>,
    on_pause: Option<TypedFunc<(), ()>>,
    on_resume: Option<TypedFunc<(), ()>>,
    set_scene: Option<TypedFunc<(i32,), ()>>,
    get_scene: Option<TypedFunc<(), i32>>,
    get_scene_count: Option<TypedFunc<(), i32>>,
}

impl AppInstance {
    pub fn load(
        engine: &Engine,
        shared: &SharedRef,
        bundle: Bundle<'static>,
    ) -> Result<Self, LoadError> {
        let module = Module::new(engine, bundle.payload())
            .map_err(|e| LoadError::Compile(describe(&e)))?;

        let mut app_id: String<24> = String::new();
        let _ = app_id.push_str(bundle.id());
        let state = HostState {
            shared: Rc::clone(shared),
            app_id,
            is_system: bundle.is_system(),
            limits: StoreLimitsBuilder::new()
                .memory_size(APP_MEMORY_MAX)
                .memories(1)
                .tables(1)
                .instances(1)
                .trap_on_grow_failure(true)
                .build(),
            timer: TimerState::default(),
            render_requested: false,
        };
        let mut store = Store::new(engine, state);
        store.limiter(|s| &mut s.limits);
        store
            .set_fuel(FUEL_PER_CALL)
            .map_err(|e| LoadError::Instantiate(describe(&e)))?;

        let mut linker = Linker::<HostState>::new(engine);
        register_imports(&mut linker).map_err(|e| LoadError::Instantiate(describe(&e)))?;

        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|e| LoadError::Instantiate(describe(&e)))?;

        let memory = instance
            .get_export(&store, "memory")
            .and_then(Extern::into_memory)
            .ok_or(LoadError::MissingMemory)?;

        let render = instance
            .get_typed_func::<(), ()>(&store, "render")
            .map_err(|_| LoadError::MissingRender)?;

        let opt0 = |name: &str| instance.get_typed_func::<(), ()>(&store, name).ok();
        Ok(Self {
            on_input: instance.get_typed_func::<(i32, i32), ()>(&store, "on_input").ok(),
            on_start: opt0("on_start"),
            on_stop: opt0("on_stop"),
            on_pause: opt0("on_pause"),
            on_resume: opt0("on_resume"),
            set_scene: instance.get_typed_func::<(i32,), ()>(&store, "set_scene").ok(),
            get_scene: instance.get_typed_func::<(), i32>(&store, "get_scene").ok(),
            get_scene_count: instance.get_typed_func::<(), i32>(&store, "get_scene_count").ok(),
            store,
            _instance: instance,
            memory,
            render,
        })
    }

    fn refuel(&mut self) {
        // Fuel cannot fail to set on an engine built with consume_fuel.
        let _ = self.store.set_fuel(FUEL_PER_CALL);
    }

    fn map_err(err: WasmiError) -> CallError {
        if matches!(err.as_trap_code(), Some(wasmi::core::TrapCode::OutOfFuel)) {
            CallError::OutOfFuel
        } else {
            CallError::Trap(describe(&err))
        }
    }

    fn call0(&mut self, func: Option<TypedFunc<(), ()>>) -> Result<(), CallError> {
        let Some(func) = func else {
            return Ok(());
        };
        self.refuel();
        func.call(&mut self.store, ()).map_err(Self::map_err)
    }

    pub fn render(&mut self) -> Result<(), CallError> {
        self.refuel();
        self.render.call(&mut self.store, ()).map_err(Self::map_err)
    }

    pub fn on_input(&mut self, key: u32, kind: u32) -> Result<(), CallError> {
        let Some(func) = self.on_input else {
            return Ok(());
        };
        self.refuel();
        func.call(&mut self.store, (key as i32, kind as i32))
            .map_err(Self::map_err)
    }

    pub fn on_start(&mut self) -> Result<(), CallError> {
        self.call0(self.on_start)
    }
    pub fn on_stop(&mut self) -> Result<(), CallError> {
        self.call0(self.on_stop)
    }
    pub fn on_pause(&mut self) -> Result<(), CallError> {
        self.call0(self.on_pause)
    }
    pub fn on_resume(&mut self) -> Result<(), CallError> {
        self.call0(self.on_resume)
    }

    pub fn scene_count(&mut self) -> u32 {
        let Some(f) = self.get_scene_count else { return 1 };
        self.refuel();
        f.call(&mut self.store, ()).map(|v| v.max(0) as u32).unwrap_or(1)
    }

    pub fn scene(&mut self) -> u32 {
        let Some(f) = self.get_scene else { return 0 };
        self.refuel();
        f.call(&mut self.store, ()).map(|v| v.max(0) as u32).unwrap_or(0)
    }

    pub fn set_scene(&mut self, scene: u32) {
        let Some(f) = self.set_scene else { return };
        self.refuel();
        let _ = f.call(&mut self.store, (scene as i32,));
    }

    pub fn timer_due(&mut self, now_ms: u32) -> bool {
        self.store.data_mut().timer.due(now_ms)
    }

    pub fn timer_next_ms(&self) -> Option<u32> {
        self.store.data().timer.next_ms()
    }

    pub fn take_render_request(&mut self) -> bool {
        core::mem::replace(&mut self.store.data_mut().render_requested, false)
    }

    /// Bytes of linear memory currently reserved by this instance.
    pub fn memory_bytes(&self) -> usize {
        self.memory.data_size(&self.store)
    }

    /// Fuel consumed by the most recent call.
    pub fn last_call_fuel(&self) -> u64 {
        FUEL_PER_CALL.saturating_sub(self.store.get_fuel().unwrap_or(FUEL_PER_CALL))
    }
}

// ---------------------------------------------------------------------------
// Host imports
// ---------------------------------------------------------------------------

type C<'a> = Caller<'a, HostState>;

fn memory_of(caller: &mut C<'_>) -> Option<Memory> {
    caller.get_export("memory").and_then(Extern::into_memory)
}

/// Borrow a NUL-terminated string out of app memory. Bounded by
/// `max_len`; a missing terminator yields the clipped prefix. Invalid
/// UTF-8 yields "" — drawing nothing beats trapping the app.
fn cstr(mem: &[u8], ptr: i32, max_len: usize) -> &str {
    let start = ptr.max(0) as usize;
    if start >= mem.len() {
        return "";
    }
    let end = mem.len().min(start + max_len);
    let slice = &mem[start..end];
    let len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    core::str::from_utf8(&slice[..len]).unwrap_or("")
}

const STR_MAX: usize = 256;

fn with_canvas(caller: &mut C<'_>, f: impl FnOnce(&mut Canvas)) {
    let shared = Rc::clone(&caller.data().shared);
    f(&mut shared.borrow_mut().canvas);
}

fn with_str(caller: &mut C<'_>, ptr: i32, f: impl FnOnce(&mut Shared, &str)) {
    let Some(memory) = memory_of(caller) else { return };
    let (mem, state) = memory.data_and_store_mut(caller);
    let text = cstr(mem, ptr, STR_MAX);
    f(&mut state.shared.borrow_mut(), text);
}

fn color_from(v: i32) -> Color {
    match v {
        0 => Color::White,
        2 => Color::Xor,
        _ => Color::Black,
    }
}

fn font_from(v: i32) -> Font {
    match v {
        1 => Font::Secondary,
        2 => Font::Keyboard,
        3 => Font::BigNumbers,
        _ => Font::Primary,
    }
}

/// An app owns its own namespace. System apps (launcher, settings) may
/// also use `system`. Nobody touches another app's settings.
fn settings_ns_allowed(state: &HostState, ns: &str) -> bool {
    ns == state.app_id.as_str() || (state.is_system && ns == SYSTEM_NS)
}

fn register_imports(linker: &mut Linker<HostState>) -> Result<(), WasmiError> {
    // -- canvas --------------------------------------------------------
    linker.func_wrap("env", "canvas_clear", |mut c: C<'_>| {
        with_canvas(&mut c, |cv| cv.clear());
    })?;
    linker.func_wrap("env", "canvas_width", |_: C<'_>| -> i32 { crate::SCREEN_WIDTH as i32 })?;
    linker.func_wrap("env", "canvas_height", |_: C<'_>| -> i32 { crate::SCREEN_HEIGHT as i32 })?;
    linker.func_wrap("env", "canvas_set_color", |mut c: C<'_>, color: i32| {
        with_canvas(&mut c, |cv| cv.set_color(color_from(color)));
    })?;
    linker.func_wrap("env", "canvas_set_font", |mut c: C<'_>, font: i32| {
        with_canvas(&mut c, |cv| cv.set_font(font_from(font)));
    })?;
    linker.func_wrap("env", "canvas_draw_dot", |mut c: C<'_>, x: i32, y: i32| {
        with_canvas(&mut c, |cv| cv.draw_dot(x, y));
    })?;
    linker.func_wrap(
        "env",
        "canvas_draw_line",
        |mut c: C<'_>, x1: i32, y1: i32, x2: i32, y2: i32| {
            with_canvas(&mut c, |cv| cv.draw_line(x1, y1, x2, y2));
        },
    )?;
    linker.func_wrap(
        "env",
        "canvas_draw_frame",
        |mut c: C<'_>, x: i32, y: i32, w: i32, h: i32| {
            with_canvas(&mut c, |cv| cv.draw_frame(x, y, w.max(0) as u32, h.max(0) as u32));
        },
    )?;
    linker.func_wrap(
        "env",
        "canvas_draw_box",
        |mut c: C<'_>, x: i32, y: i32, w: i32, h: i32| {
            with_canvas(&mut c, |cv| cv.draw_box(x, y, w.max(0) as u32, h.max(0) as u32));
        },
    )?;
    linker.func_wrap(
        "env",
        "canvas_draw_rframe",
        |mut c: C<'_>, x: i32, y: i32, w: i32, h: i32, r: i32| {
            with_canvas(&mut c, |cv| {
                cv.draw_rframe(x, y, w.max(0) as u32, h.max(0) as u32, r.max(0) as u32)
            });
        },
    )?;
    linker.func_wrap(
        "env",
        "canvas_draw_rbox",
        |mut c: C<'_>, x: i32, y: i32, w: i32, h: i32, r: i32| {
            with_canvas(&mut c, |cv| {
                cv.draw_rbox(x, y, w.max(0) as u32, h.max(0) as u32, r.max(0) as u32)
            });
        },
    )?;
    linker.func_wrap("env", "canvas_draw_circle", |mut c: C<'_>, x: i32, y: i32, r: i32| {
        with_canvas(&mut c, |cv| cv.draw_circle(x, y, r.max(0) as u32));
    })?;
    linker.func_wrap("env", "canvas_draw_disc", |mut c: C<'_>, x: i32, y: i32, r: i32| {
        with_canvas(&mut c, |cv| cv.draw_disc(x, y, r.max(0) as u32));
    })?;
    linker.func_wrap("env", "canvas_draw_str", |mut c: C<'_>, x: i32, y: i32, ptr: i32| {
        with_str(&mut c, ptr, |s, text| s.canvas.draw_str(x, y, text));
    })?;
    linker.func_wrap("env", "canvas_string_width", |mut c: C<'_>, ptr: i32| -> i32 {
        let mut width = 0;
        with_str(&mut c, ptr, |s, text| width = s.canvas.string_width(text) as i32);
        width
    })?;
    linker.func_wrap("env", "canvas_draw_buffer", |mut c: C<'_>, ptr: i32, len: i32| {
        let Some(memory) = memory_of(&mut c) else { return };
        let (mem, state) = memory.data_and_store_mut(&mut c);
        let start = ptr.max(0) as usize;
        let len = (len.max(0) as usize).min(crate::FRAMEBUFFER_LEN);
        if start.saturating_add(len) > mem.len() {
            return;
        }
        state.shared.borrow_mut().canvas.fill_from(&mem[start..start + len]);
    })?;
    linker.func_wrap(
        "env",
        "canvas_draw_bitmap",
        |mut c: C<'_>, x: i32, y: i32, w: i32, h: i32, ptr: i32| {
            let Some(memory) = memory_of(&mut c) else { return };
            let (mem, state) = memory.data_and_store_mut(&mut c);
            let (w, h) = (
                w.clamp(0, crate::SCREEN_WIDTH as i32) as usize,
                h.clamp(0, crate::SCREEN_HEIGHT as i32) as usize,
            );
            let row_bytes = w.div_ceil(8);
            let start = ptr.max(0) as usize;
            let Some(bits) = mem.get(start..start + row_bytes * h) else { return };
            state.shared.borrow_mut().canvas.draw_bitmap(x, y, w as u32, h as u32, bits);
        },
    )?;

    // -- random / time ------------------------------------------------
    linker.func_wrap("env", "random_seed", |c: C<'_>, seed: i32| {
        c.data().shared.borrow_mut().random.seed(seed as u32);
    })?;
    linker.func_wrap("env", "random_get", |c: C<'_>| -> i32 {
        c.data().shared.borrow_mut().random.get() as i32
    })?;
    linker.func_wrap("env", "random_range", |c: C<'_>, max: i32| -> i32 {
        c.data().shared.borrow_mut().random.range(max.max(0) as u32) as i32
    })?;
    linker.func_wrap("env", "get_time_ms", |c: C<'_>| -> i32 {
        c.data().shared.borrow().now_ms as i32
    })?;

    // -- timers / rendering -------------------------------------------
    linker.func_wrap("env", "start_timer_ms", |mut c: C<'_>, interval_ms: i32| {
        let now = c.data().shared.borrow().now_ms;
        c.data_mut().timer.start(now, interval_ms.max(0) as u32);
    })?;
    linker.func_wrap("env", "stop_timer", |mut c: C<'_>| {
        c.data_mut().timer.stop();
    })?;
    linker.func_wrap("env", "request_render", |mut c: C<'_>| {
        c.data_mut().render_requested = true;
    })?;

    // -- app control ---------------------------------------------------
    linker.func_wrap("env", "exit_to_launcher", |c: C<'_>| {
        c.data().shared.borrow_mut().request = AppRequest::ExitToLauncher;
    })?;
    linker.func_wrap("env", "start_app", |c: C<'_>, index: i32| {
        if index >= 0 {
            c.data().shared.borrow_mut().request = AppRequest::StartApp(index as u32);
        }
    })?;
    linker.func_wrap("env", "app_count", |c: C<'_>| -> i32 {
        c.data().shared.borrow().registry.len() as i32
    })?;
    // Copies the 256-byte bundle header of app `index` into app memory.
    // Returns bytes written, or -1.
    linker.func_wrap("env", "app_info", |mut c: C<'_>, index: i32, ptr: i32, len: i32| -> i32 {
        let Some(memory) = memory_of(&mut c) else { return -1 };
        let (mem, state) = memory.data_and_store_mut(&mut c);
        let shared = state.shared.borrow();
        let Some(bundle) = shared.registry.get(index.max(0) as usize) else { return -1 };
        let start = ptr.max(0) as usize;
        if len < HEADER_LEN as i32 || start.saturating_add(HEADER_LEN) > mem.len() {
            return -1;
        }
        mem[start..start + HEADER_LEN].copy_from_slice(bundle.header());
        HEADER_LEN as i32
    })?;
    linker.func_wrap("env", "kernel_version", |_: C<'_>| -> i32 { KERNEL_VERSION as i32 })?;

    // -- settings ------------------------------------------------------
    linker.func_wrap(
        "env",
        "settings_get_u32",
        |mut c: C<'_>, ns_ptr: i32, key_ptr: i32, default: i32| -> i32 {
            let Some(memory) = memory_of(&mut c) else { return default };
            let (mem, state) = memory.data_and_store_mut(&mut c);
            let ns = cstr(mem, ns_ptr, 24);
            let key = cstr(mem, key_ptr, 24);
            if !settings_ns_allowed(state, ns) {
                return default;
            }
            state.shared.borrow().settings.get_or(ns, key, default as u32) as i32
        },
    )?;
    linker.func_wrap(
        "env",
        "settings_set_u32",
        |mut c: C<'_>, ns_ptr: i32, key_ptr: i32, value: i32| -> i32 {
            let Some(memory) = memory_of(&mut c) else { return 0 };
            let (mem, state) = memory.data_and_store_mut(&mut c);
            let ns = cstr(mem, ns_ptr, 24);
            let key = cstr(mem, key_ptr, 24);
            if !settings_ns_allowed(state, ns) || key.is_empty() {
                return 0;
            }
            state.shared.borrow_mut().settings.set(ns, key, value as u32) as i32
        },
    )?;

    // -- logging -------------------------------------------------------
    linker.func_wrap("env", "log_str", |mut c: C<'_>, ptr: i32| {
        with_str(&mut c, ptr, |s, text| s.log_line(text));
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_not_due_before_interval() {
        let mut t = TimerState::default();
        t.start(1000, 100);
        assert!(!t.due(1099));
        assert!(t.due(1100));
        assert!(!t.due(1150));
        assert!(t.due(1200));
    }

    #[test]
    fn timer_skips_missed_periods() {
        let mut t = TimerState::default();
        t.start(0, 10);
        assert!(t.due(1005));
        assert_eq!(t.next_ms(), Some(1010));
    }

    #[test]
    fn timer_survives_wraparound() {
        let mut t = TimerState::default();
        t.start(u32::MAX - 5, 10);
        assert!(!t.due(u32::MAX));
        assert!(t.due(4));
    }

    #[test]
    fn cstr_clips_and_terminates() {
        let mem = b"hello\0world";
        assert_eq!(cstr(mem, 0, 256), "hello");
        assert_eq!(cstr(mem, 6, 256), "world");
        assert_eq!(cstr(mem, 6, 3), "wor");
        assert_eq!(cstr(mem, 100, 3), "");
        assert_eq!(cstr(mem, -1, 3), "hel");
    }
}
