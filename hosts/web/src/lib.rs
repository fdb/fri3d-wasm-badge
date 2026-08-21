//! Browser host. JS owns the clock, the canvas and key events; this crate
//! owns nothing but a boxed kernel.

use fri3d_kernel::settings::IMAGE_LEN;
use fri3d_kernel::types::InputKey;
use fri3d_kernel::{Kernel, StepResult};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebKernel {
    kernel: Box<Kernel>,
    last: StepResult,
}

#[wasm_bindgen]
impl WebKernel {
    #[wasm_bindgen(constructor)]
    pub fn new(now_ms: u32) -> Result<WebKernel, JsValue> {
        let mut kernel = Box::new(Kernel::new());
        kernel
            .set_launcher(fri3d_apps::LAUNCHER)
            .map_err(|e| JsValue::from_str(&format!("launcher: {e:?}")))?;
        for app in fri3d_apps::APPS {
            kernel
                .add_app(app)
                .map_err(|e| JsValue::from_str(&format!("app: {e:?}")))?;
        }
        kernel.boot(now_ms);
        Ok(Self { kernel, last: StepResult::default() })
    }

    pub fn push_key(&mut self, key: u32, pressed: bool, now_ms: u32) {
        if let Some(key) = InputKey::from_u32(key) {
            self.kernel.push_raw_input(key, pressed, now_ms);
        }
    }

    /// Returns true when the framebuffer changed.
    pub fn step(&mut self, now_ms: u32) -> bool {
        self.last = self.kernel.step(now_ms);
        self.last.frame
    }

    pub fn framebuffer(&self) -> Vec<u8> {
        self.kernel.framebuffer().to_vec()
    }

    pub fn app_count(&self) -> u32 {
        self.kernel.app_count() as u32
    }

    pub fn app_name(&self, index: u32) -> String {
        self.kernel.app_name(index as usize).unwrap_or("").to_string()
    }

    pub fn start_app(&mut self, index: u32) -> bool {
        self.kernel.start_app(index as usize)
    }

    pub fn exit_to_launcher(&mut self) {
        self.kernel.exit_to_launcher();
    }

    pub fn rng_seed(&mut self, seed: u32) {
        self.kernel.random_seed(seed);
    }

    pub fn rng_get(&mut self) -> u32 {
        self.kernel.random_get()
    }

    pub fn last_error(&self) -> String {
        self.kernel.last_error().to_string()
    }

    pub fn take_log(&mut self) -> Option<String> {
        self.kernel.take_log_line().map(|l| l.to_string())
    }

    pub fn settings_image(&mut self) -> Option<Vec<u8>> {
        let mut img = [0u8; IMAGE_LEN];
        self.kernel.take_settings_image(&mut img).then(|| img.to_vec())
    }

    pub fn load_settings(&mut self, bytes: &[u8]) {
        self.kernel.load_settings(bytes);
    }

    /// -1 when unset.
    pub fn setting(&self, ns: &str, key: &str) -> i64 {
        self.kernel.setting(ns, key).map(i64::from).unwrap_or(-1)
    }

    pub fn set_scene(&mut self, scene: u32) {
        self.kernel.set_scene(scene);
    }

    pub fn request_render(&mut self) {
        self.kernel.request_render();
    }

    /// -1 when no timer is pending.
    pub fn next_wake_ms(&self) -> i64 {
        self.last.next_wake_ms.map(i64::from).unwrap_or(-1)
    }
}
