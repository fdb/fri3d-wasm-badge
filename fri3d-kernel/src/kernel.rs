//! The kernel proper: boot, lifecycle, and the per-iteration `step`.
//!
//! Two slots, never more: the launcher (resident for the whole uptime)
//! and at most one foreground app. Lifecycle per slot:
//!
//! ```text
//!   load ─▶ on_start ─▶ on_resume ─▶ [focused] ─▶ on_pause ─▶ on_stop ─▶ drop
//!                          ▲                          │
//!                          └──── (launcher only) ─────┘
//! ```
//!
//! Starting an app pauses the launcher. Leaving the app (its own request,
//! the Menu key, the reset combo, or a trap) stops and drops it and
//! resumes the launcher. The launcher never stops; if it traps, the
//! kernel reloads it.

use crate::bundle::Bundle;
use crate::host::{make_engine, AppInstance, AppRequest, CallError, LoadError, Shared, SharedRef};
use crate::input::{InputEvent, InputManager};
use crate::registry::RegistryError;
use crate::settings::{IMAGE_LEN, SYSTEM_NS};
use crate::net::NetRequest;
use crate::wifi::{ScanEntry, WifiRequest, WifiStatus, IMAGE_LEN as WIFI_IMAGE_LEN};
use crate::types::{Color, Font, InputKey, InputType};
use alloc::rc::Rc;
use core::cell::RefCell;
use heapless::String;
use wasmi::Engine;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Focus {
    Launcher,
    App,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StepResult {
    /// The framebuffer changed; the host should blit.
    pub frame: bool,
    /// Wall-clock (ms) of the next timer, if any. Hosts may sleep until then
    /// or until input arrives.
    pub next_wake_ms: Option<u32>,
}

pub struct Kernel {
    engine: Engine,
    shared: SharedRef,
    input: InputManager,
    launcher_bundle: Option<Bundle<'static>>,
    launcher: Option<AppInstance>,
    app: Option<AppInstance>,
    app_index: Option<usize>,
    focus: Focus,
    needs_render: bool,
    last_error: String<96>,
    /// Debug counters for the host's perf overlay / log.
    pub stats: Stats,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Stats {
    pub frames: u32,
    pub last_render_fuel: u64,
    pub app_memory_bytes: usize,
    pub app_traps: u32,
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            engine: make_engine(),
            shared: Rc::new(RefCell::new(Shared::new())),
            input: InputManager::new(),
            launcher_bundle: None,
            launcher: None,
            app: None,
            app_index: None,
            focus: Focus::Launcher,
            needs_render: true,
            last_error: String::new(),
            stats: Stats::default(),
        }
    }

    // -- boot ------------------------------------------------------------

    /// Register the launcher bundle. Must precede `boot`.
    pub fn set_launcher(&mut self, bytes: &'static [u8]) -> Result<(), RegistryError> {
        let bundle = Bundle::parse(bytes).map_err(RegistryError::Bundle)?;
        self.launcher_bundle = Some(bundle);
        Ok(())
    }

    /// Register an app. Order of registration = index apps see.
    pub fn add_app(&mut self, bytes: &'static [u8]) -> Result<usize, RegistryError> {
        self.shared.borrow_mut().registry.add(bytes)
    }

    pub fn load_settings(&mut self, image: &[u8]) {
        self.shared.borrow_mut().settings.load_image(image);
    }

    /// Start the launcher. Call once after registering bundles.
    pub fn boot(&mut self, now_ms: u32) {
        {
            let mut shared = self.shared.borrow_mut();
            shared.now_ms = now_ms;
            let on = shared.settings.get_or(SYSTEM_NS, "wifi", 1) != 0;
            shared.wifi.set_enabled(on);
        }
        self.reload_launcher();
        self.needs_render = true;
    }

    fn reload_launcher(&mut self) {
        self.launcher = None;
        let Some(bundle) = self.launcher_bundle else {
            self.set_error("no launcher registered");
            return;
        };
        match AppInstance::load(&self.engine, &self.shared, bundle) {
            Ok(mut inst) => {
                if let Err(e) = inst.on_start().and_then(|_| inst.on_resume()) {
                    self.set_call_error("launcher", e);
                }
                self.launcher = Some(inst);
            }
            Err(e) => self.set_load_error("launcher", e),
        }
    }

    // -- host-facing -----------------------------------------------------

    /// A physical key changed state.
    pub fn push_raw_input(&mut self, key: InputKey, pressed: bool, now_ms: u32) {
        self.input.push_raw(key, pressed, now_ms);
    }

    pub fn framebuffer(&self) -> core::cell::Ref<'_, [u8]> {
        core::cell::Ref::map(self.shared.borrow(), |s| s.canvas.buffer())
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn current_app_index(&self) -> Option<usize> {
        self.app_index
    }

    pub fn last_error(&self) -> &str {
        &self.last_error
    }

    pub fn app_count(&self) -> usize {
        self.shared.borrow().registry.len()
    }

    pub fn app_name(&self, index: usize) -> Option<&'static str> {
        self.shared.borrow().registry.get(index).map(|b| b.name())
    }

    pub fn setting(&self, ns: &str, key: &str) -> Option<u32> {
        self.shared.borrow().settings.get(ns, key)
    }

    pub fn set_setting(&mut self, ns: &str, key: &str, value: u32) -> bool {
        self.shared.borrow_mut().settings.set(ns, key, value)
    }

    /// If settings changed since the last call, write the persistent image
    /// into `out` and return true.
    pub fn take_settings_image(&mut self, out: &mut [u8; IMAGE_LEN]) -> bool {
        let mut shared = self.shared.borrow_mut();
        if !shared.settings.take_dirty() {
            return false;
        }
        shared.settings.write_image(out);
        true
    }

    // -- wifi (host driver side) ----------------------------------------

    pub fn load_wifi(&mut self, image: &[u8]) {
        self.shared.borrow_mut().wifi.load_image(image);
    }

    /// If saved networks changed since the last call, write the
    /// persistent image into `out` and return true.
    pub fn take_wifi_image(&mut self, out: &mut [u8; WIFI_IMAGE_LEN]) -> bool {
        let mut shared = self.shared.borrow_mut();
        if !shared.wifi.take_dirty() {
            return false;
        }
        shared.wifi.write_image(out);
        true
    }

    /// Next radio primitive the host must execute, if any.
    pub fn take_wifi_request(&mut self) -> Option<WifiRequest> {
        self.shared.borrow_mut().wifi.take_request()
    }

    pub fn wifi_scan_done(&mut self, entries: &[ScanEntry]) {
        self.shared.borrow_mut().wifi.scan_done(entries);
    }

    pub fn wifi_connect_done(&mut self, ok: bool) {
        self.shared.borrow_mut().wifi.connect_done(ok);
    }

    pub fn wifi_link_lost(&mut self) {
        let mut shared = self.shared.borrow_mut();
        let now = shared.now_ms;
        shared.wifi.link_lost(now);
    }

    pub fn wifi_status(&self) -> WifiStatus {
        self.shared.borrow().wifi.status()
    }

    pub fn wifi_current_ssid(&self) -> crate::wifi::Ssid {
        let shared = self.shared.borrow();
        let mut s = crate::wifi::Ssid::new();
        let _ = s.push_str(shared.wifi.current_ssid());
        s
    }

    /// Direct model access for hosts and tests (the `Sim` driver).
    pub fn wifi_mut(&mut self) -> core::cell::RefMut<'_, crate::wifi::Wifi> {
        core::cell::RefMut::map(self.shared.borrow_mut(), |s| &mut s.wifi)
    }

    // -- net (host driver side) -----------------------------------------

    pub fn take_net_request(&mut self) -> Option<NetRequest> {
        self.shared.borrow_mut().net.take_request()
    }

    pub fn net_progress(&mut self, bytes: u32) {
        self.shared.borrow_mut().net.progress(bytes);
    }

    pub fn net_done(&mut self, ok: bool) {
        let mut shared = self.shared.borrow_mut();
        let now = shared.now_ms;
        shared.net.done(ok, now);
    }

    pub fn net_mut(&mut self) -> core::cell::RefMut<'_, crate::net::Net> {
        core::cell::RefMut::map(self.shared.borrow_mut(), |s| &mut s.net)
    }

    /// Drain one buffered log line from apps.
    pub fn take_log_line(&mut self) -> Option<String<{ crate::limits::LOG_LINE_LEN }>> {
        self.shared.borrow_mut().log.pop_front()
    }

    /// Host RNG access, for tests and the browser harness.
    pub fn random_seed(&mut self, seed: u32) {
        self.shared.borrow_mut().random.seed(seed);
    }

    pub fn random_get(&mut self) -> u32 {
        self.shared.borrow_mut().random.get()
    }

    /// Start an app by registry index from the host (e.g. a CLI flag).
    pub fn start_app(&mut self, index: usize) -> bool {
        self.apply_request(AppRequest::StartApp(index as u32))
    }

    pub fn exit_to_launcher(&mut self) {
        self.apply_request(AppRequest::ExitToLauncher);
    }

    // Scene hooks for visual-golden tests.
    pub fn scene_count(&mut self) -> u32 {
        self.focused().map(|i| i.scene_count()).unwrap_or(1)
    }
    pub fn set_scene(&mut self, scene: u32) {
        if let Some(i) = self.focused() {
            i.set_scene(scene);
        }
        self.needs_render = true;
    }
    pub fn scene(&mut self) -> u32 {
        self.focused().map(|i| i.scene()).unwrap_or(0)
    }

    /// Force a render on the next step.
    pub fn request_render(&mut self) {
        self.needs_render = true;
    }

    // -- the loop --------------------------------------------------------

    pub fn step(&mut self, now_ms: u32) -> StepResult {
        {
            let mut shared = self.shared.borrow_mut();
            shared.now_ms = now_ms;
            shared.wifi.tick(now_ms);
            self.needs_render |= shared.wifi.take_changed() | shared.net.take_changed();
        }
        self.input.update(now_ms);

        while let Some(ev) = self.input.next_event() {
            self.dispatch_input(ev);
        }
        if self.input.take_reset_combo() {
            self.apply_request(AppRequest::ExitToLauncher);
        }

        let wake = self
            .focused()
            .map(|inst| inst.timer_due(now_ms) | inst.take_render_request())
            .unwrap_or(false);
        self.needs_render |= wake;

        let mut rendered = false;
        if self.needs_render {
            self.needs_render = false;
            // One extra pass when the frame itself asked for another (an
            // app switch or `request_render` inside render) so UI changes
            // never show up a frame late. Bounded at two.
            for _ in 0..2 {
                self.render_focused();
                rendered = true;
                let switched = self.take_request();
                let again = self.focused().map(|i| i.take_render_request()).unwrap_or(false);
                if !(switched || again) {
                    break;
                }
            }
        }

        StepResult {
            frame: rendered,
            next_wake_ms: self.focused().and_then(|i| i.timer_next_ms()),
        }
    }

    fn dispatch_input(&mut self, ev: InputEvent) {
        // Home key: the kernel owns it. Apps still receive the press so a
        // game can pause, but the short press always goes home.
        if ev.key == InputKey::Menu && ev.kind == InputType::ShortPress && self.focus == Focus::App {
            self.apply_request(AppRequest::ExitToLauncher);
            return;
        }
        let result = match self.focused() {
            Some(inst) => inst.on_input(ev.key as u32, ev.kind as u32),
            None => Ok(()),
        };
        self.needs_render = true;
        self.handle_call_result(result);
        self.take_request();
    }

    fn render_focused(&mut self) {
        self.shared.borrow_mut().canvas.clear();
        let result = match self.focused() {
            Some(inst) => inst.render(),
            None => Ok(()),
        };
        if let Some(inst) = self.focused() {
            self.stats.last_render_fuel = inst.last_call_fuel();
        }
        self.stats.frames = self.stats.frames.wrapping_add(1);
        match result {
            Ok(()) if self.focused().is_some() => {}
            Ok(()) => self.render_kernel_error(),
            Err(e) => {
                self.handle_call_result(Err(e));
                // The failing frame is garbage; draw the replacement now.
                self.shared.borrow_mut().canvas.clear();
                match self.focused() {
                    Some(inst) => {
                        let _ = inst.render();
                    }
                    None => self.render_kernel_error(),
                }
            }
        }
    }

    /// The kernel's own screen, shown only when the launcher itself is gone.
    fn render_kernel_error(&mut self) {
        let mut shared = self.shared.borrow_mut();
        let cv = &mut shared.canvas;
        cv.clear();
        cv.set_color(Color::Black);
        cv.set_font(Font::Primary);
        cv.draw_str(2, 12, "Kernel: launcher down");
        cv.set_font(Font::Secondary);
        let msg: String<96> = self.last_error.clone();
        cv.draw_str(2, 26, &msg);
    }

    fn focused(&mut self) -> Option<&mut AppInstance> {
        match self.focus {
            Focus::App => self.app.as_mut(),
            Focus::Launcher => self.launcher.as_mut(),
        }
    }

    /// Apply a pending app request. Returns true if focus changed.
    fn take_request(&mut self) -> bool {
        let request = core::mem::replace(&mut self.shared.borrow_mut().request, AppRequest::None);
        match request {
            AppRequest::None => false,
            other => self.apply_request(other),
        }
    }

    fn apply_request(&mut self, request: AppRequest) -> bool {
        match request {
            AppRequest::None => false,
            AppRequest::ExitToLauncher => {
                if self.focus == Focus::App {
                    self.stop_app();
                    true
                } else {
                    false
                }
            }
            AppRequest::StartApp(index) => self.launch(index as usize),
        }
    }

    fn launch(&mut self, index: usize) -> bool {
        let bundle = self.shared.borrow().registry.get(index);
        let Some(bundle) = bundle else {
            self.set_error("start_app: no such app");
            return false;
        };
        if self.focus == Focus::App {
            self.stop_app();
        }
        if let Some(l) = self.launcher.as_mut() {
            let r = l.on_pause();
            self.handle_launcher_result(r);
        }
        match AppInstance::load(&self.engine, &self.shared, bundle) {
            Ok(mut inst) => {
                let started = inst.on_start().and_then(|_| inst.on_resume());
                self.stats.app_memory_bytes = inst.memory_bytes();
                self.app = Some(inst);
                self.app_index = Some(index);
                self.focus = Focus::App;
                self.needs_render = true;
                if let Err(e) = started {
                    self.set_call_error(bundle.id(), e);
                    self.stop_app();
                    return false;
                }
                true
            }
            Err(e) => {
                self.set_load_error(bundle.id(), e);
                if let Some(l) = self.launcher.as_mut() {
                    let r = l.on_resume();
                    self.handle_launcher_result(r);
                }
                self.needs_render = true;
                false
            }
        }
    }

    fn stop_app(&mut self) {
        // The app's network operation dies with it.
        self.shared.borrow_mut().net.cancel();
        if let Some(mut app) = self.app.take() {
            // Best effort: a trapping on_pause/on_stop must not block exit.
            let _ = app.on_pause();
            let _ = app.on_stop();
        }
        self.app_index = None;
        self.focus = Focus::Launcher;
        self.stats.app_memory_bytes = 0;
        if let Some(l) = self.launcher.as_mut() {
            let r = l.on_resume();
            self.handle_launcher_result(r);
        }
        self.needs_render = true;
    }

    fn handle_call_result(&mut self, result: Result<(), CallError>) {
        let Err(e) = result else { return };
        match self.focus {
            Focus::App => {
                let id = self.app_index.and_then(|i| self.shared.borrow().registry.get(i)).map(|b| b.id()).unwrap_or("app");
                self.set_call_error(id, e);
                self.stats.app_traps = self.stats.app_traps.wrapping_add(1);
                self.stop_app();
            }
            Focus::Launcher => self.handle_launcher_result(Err(e)),
        }
    }

    fn handle_launcher_result(&mut self, result: Result<(), CallError>) {
        let Err(e) = result else { return };
        self.set_call_error("launcher", e);
        self.reload_launcher();
    }

    fn set_error(&mut self, msg: &str) {
        self.last_error.clear();
        let _ = self.last_error.push_str(&msg[..msg.len().min(96)]);
    }

    fn set_call_error(&mut self, who: &str, e: CallError) {
        let mut s: String<160> = String::new();
        let _ = match e {
            CallError::OutOfFuel => core::fmt::write(&mut s, format_args!("{who}: out of fuel")),
            CallError::Trap(t) => core::fmt::write(&mut s, format_args!("{who}: {t}")),
        };
        let mut end = s.len().min(96);
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        self.set_error(&s[..end]);
    }

    fn set_load_error(&mut self, who: &str, e: LoadError) {
        let mut s: String<160> = String::new();
        let _ = match e {
            LoadError::Compile(m) => core::fmt::write(&mut s, format_args!("{who}: compile: {m}")),
            LoadError::Instantiate(m) => core::fmt::write(&mut s, format_args!("{who}: link: {m}")),
            LoadError::MissingRender => core::fmt::write(&mut s, format_args!("{who}: no render()")),
            LoadError::MissingMemory => core::fmt::write(&mut s, format_args!("{who}: no memory")),
        };
        let mut end = s.len().min(96);
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        self.set_error(&s[..end]);
    }
}
