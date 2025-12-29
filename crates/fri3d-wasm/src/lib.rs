//! WASM execution layer for Fri3d Badge using wasmi

mod app;
mod host;

pub use app::{WasmApp, WasmError};
pub use host::HostState;
