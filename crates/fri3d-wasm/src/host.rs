//! Host functions for WASM apps

use fri3d_runtime::{Canvas, Color, Font, Random};
use wasmi::{Caller, Linker};

/// Host state accessible from WASM
pub struct HostState {
    pub canvas: Canvas,
    pub random: Random,
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl HostState {
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            random: Random::new(42),
        }
    }
}

/// Register all host functions with the linker
pub fn register_host_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), wasmi::errors::LinkerError> {
    // Canvas functions
    linker.func_wrap("env", "canvas_clear", |mut caller: Caller<'_, HostState>| {
        caller.data_mut().canvas.clear();
    })?;

    linker.func_wrap(
        "env",
        "canvas_width",
        |caller: Caller<'_, HostState>| -> u32 { caller.data().canvas.width() },
    )?;

    linker.func_wrap(
        "env",
        "canvas_height",
        |caller: Caller<'_, HostState>| -> u32 { caller.data().canvas.height() },
    )?;

    linker.func_wrap(
        "env",
        "canvas_set_color",
        |mut caller: Caller<'_, HostState>, color: u32| {
            if let Ok(c) = Color::try_from(color as u8) {
                caller.data_mut().canvas.set_color(c);
            }
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_set_font",
        |mut caller: Caller<'_, HostState>, font: u32| {
            if let Ok(f) = Font::try_from(font as u8) {
                caller.data_mut().canvas.set_font(f);
            }
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_dot",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32| {
            caller.data_mut().canvas.draw_pixel(x, y);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_line",
        |mut caller: Caller<'_, HostState>, x0: i32, y0: i32, x1: i32, y1: i32| {
            caller.data_mut().canvas.draw_line(x0, y0, x1, y1);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_frame",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, w: u32, h: u32| {
            caller.data_mut().canvas.draw_rect(x, y, w, h);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_box",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, w: u32, h: u32| {
            caller.data_mut().canvas.fill_rect(x, y, w, h);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_rframe",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, w: u32, h: u32, r: u32| {
            caller.data_mut().canvas.draw_round_rect(x, y, w, h, r);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_rbox",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, w: u32, h: u32, r: u32| {
            caller.data_mut().canvas.fill_round_rect(x, y, w, h, r);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_circle",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, r: u32| {
            caller.data_mut().canvas.draw_circle(x, y, r);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_disc",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, r: u32| {
            caller.data_mut().canvas.fill_circle(x, y, r);
        },
    )?;

    linker.func_wrap(
        "env",
        "canvas_draw_str",
        |mut caller: Caller<'_, HostState>, x: i32, y: i32, ptr: u32, len: u32| {
            // Get memory from the caller and copy string data
            let text_owned: Option<String> = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .and_then(|memory| {
                    let data = memory.data(&caller);
                    let start = ptr as usize;
                    let end = start + len as usize;
                    if end <= data.len() {
                        core::str::from_utf8(&data[start..end])
                            .ok()
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                });

            // Now draw the string (borrow released)
            if let Some(text) = text_owned {
                caller.data_mut().canvas.draw_str(x, y, &text);
            }
        },
    )?;

    // Random functions
    linker.func_wrap(
        "env",
        "random_seed",
        |mut caller: Caller<'_, HostState>, seed: u32| {
            caller.data_mut().random.seed(seed);
        },
    )?;

    linker.func_wrap(
        "env",
        "random_get",
        |mut caller: Caller<'_, HostState>| -> u32 { caller.data_mut().random.next() },
    )?;

    linker.func_wrap(
        "env",
        "random_range",
        |mut caller: Caller<'_, HostState>, max: u32| -> u32 {
            caller.data_mut().random.range(max)
        },
    )?;

    Ok(())
}
