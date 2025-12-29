//! WASM app loading and execution

use crate::host::{register_host_functions, HostState};
use fri3d_runtime::{Canvas, InputKey, InputType};
use wasmi::{Engine, Instance, Linker, Module, Store, TypedFunc};

/// Error type for WASM operations
#[derive(Debug)]
pub enum WasmError {
    /// Failed to create module from bytes
    ModuleCreation(String),
    /// Failed to instantiate module
    Instantiation(String),
    /// Failed to find required export
    MissingExport(String),
    /// Runtime error during execution
    Runtime(String),
}

impl core::fmt::Display for WasmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WasmError::ModuleCreation(msg) => write!(f, "Module creation failed: {}", msg),
            WasmError::Instantiation(msg) => write!(f, "Instantiation failed: {}", msg),
            WasmError::MissingExport(msg) => write!(f, "Missing export: {}", msg),
            WasmError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for WasmError {}

/// A loaded WASM application
pub struct WasmApp {
    store: Store<HostState>,
    #[allow(dead_code)]
    instance: Instance,
    render_func: TypedFunc<(), ()>,
    on_input_func: Option<TypedFunc<(u32, u32), ()>>,
    get_scene_count_func: Option<TypedFunc<(), u32>>,
    set_scene_func: Option<TypedFunc<u32, ()>>,
}

impl WasmApp {
    /// Load a WASM app from bytes
    pub fn load(engine: &Engine, wasm_bytes: &[u8]) -> Result<Self, WasmError> {
        // Create module
        let module = Module::new(engine, wasm_bytes)
            .map_err(|e| WasmError::ModuleCreation(e.to_string()))?;

        // Create store with host state
        let mut store = Store::new(engine, HostState::new());

        // Create linker and register host functions
        let mut linker = <Linker<HostState>>::new(engine);
        register_host_functions(&mut linker)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        // Instantiate module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?
            .start(&mut store)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        // Get required exports
        let render_func = instance
            .get_typed_func::<(), ()>(&store, "render")
            .map_err(|_| WasmError::MissingExport("render".to_string()))?;

        // Get optional exports
        let on_input_func = instance
            .get_typed_func::<(u32, u32), ()>(&store, "on_input")
            .ok();

        let get_scene_count_func = instance
            .get_typed_func::<(), u32>(&store, "get_scene_count")
            .ok();

        let set_scene_func = instance
            .get_typed_func::<u32, ()>(&store, "set_scene")
            .ok();

        Ok(Self {
            store,
            instance,
            render_func,
            on_input_func,
            get_scene_count_func,
            set_scene_func,
        })
    }

    /// Call the app's render function
    pub fn render(&mut self) -> Result<(), WasmError> {
        self.render_func
            .call(&mut self.store, ())
            .map_err(|e| WasmError::Runtime(e.to_string()))
    }

    /// Call the app's on_input function (if it exists)
    pub fn on_input(&mut self, key: InputKey, event_type: InputType) -> Result<(), WasmError> {
        if let Some(func) = &self.on_input_func {
            func.call(&mut self.store, (key as u32, event_type as u32))
                .map_err(|e| WasmError::Runtime(e.to_string()))?;
        }
        Ok(())
    }

    /// Get the number of scenes (for testing)
    pub fn get_scene_count(&mut self) -> Result<u32, WasmError> {
        if let Some(func) = &self.get_scene_count_func {
            func.call(&mut self.store, ())
                .map_err(|e| WasmError::Runtime(e.to_string()))
        } else {
            Ok(1)
        }
    }

    /// Set the current scene (for testing)
    pub fn set_scene(&mut self, scene: u32) -> Result<(), WasmError> {
        if let Some(func) = &self.set_scene_func {
            func.call(&mut self.store, scene)
                .map_err(|e| WasmError::Runtime(e.to_string()))?;
        }
        Ok(())
    }

    /// Get a reference to the canvas
    pub fn canvas(&self) -> &Canvas {
        &self.store.data().canvas
    }

    /// Clear the canvas
    pub fn clear_canvas(&mut self) {
        self.store.data_mut().canvas.clear();
    }
}
