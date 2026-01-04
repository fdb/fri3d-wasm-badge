# Stage 003: Runtime Core (App Manager, WASM Runner, Input, Random, Trace)

This stage describes the shared runtime components used by emulator and intended for firmware.

## App manager

References: `src/runtime/app_manager.h`, `src/runtime/app_manager.c`

Responsibilities:
- Maintain a registry of apps: `(id, name, path)`.
- Track launcher path and whether currently in launcher.
- Load/unload WASM modules via the wasm runner.
- Forward `render()` and `on_input()` calls to the active module.
- Handle app-initiated requests to switch apps (via callbacks from wasm runner).
- Use event-driven rendering by default (input-driven + request_render), with
  optional app-controlled timers for animations or game loops to conserve power.

Behavior summary:
- App IDs are 1-based in the registry. ID 0 maps to the launcher.
- `app_manager_show_launcher()` unloads current module and loads launcher WASM.
- `app_manager_launch_app_by_path()` loads a specific WASM file path.
- After each `render()` and `on_input()`, pending requests (exit/start app) are processed.
- Errors are stored in `last_error` (string buffer).

## WASM runner

References: `src/runtime/wasm_runner.h`, `src/runtime/wasm_runner.c`

Responsibilities:
- Initialize WAMR with a fixed heap (default 10 MiB).
- Load WASM module from file or memory, instantiate, create exec env.
- Register host imports under module name `env`.
- Look up optional exports: `on_input`, `get_scene`, `set_scene`, `get_scene_count`.
- Require export: `render` (fail load if missing).

Imports registered (names + signatures):
- Canvas:
  - `canvas_clear() -> void`
  - `canvas_width() -> i32`
  - `canvas_height() -> i32`
  - `canvas_set_color(i32) -> void`
  - `canvas_set_font(i32) -> void`
  - `canvas_draw_dot(i32, i32) -> void`
  - `canvas_draw_line(i32, i32, i32, i32) -> void`
  - `canvas_draw_frame(i32, i32, i32, i32) -> void`
  - `canvas_draw_box(i32, i32, i32, i32) -> void`
  - `canvas_draw_rframe(i32, i32, i32, i32, i32) -> void`
  - `canvas_draw_rbox(i32, i32, i32, i32, i32) -> void`
  - `canvas_draw_circle(i32, i32, i32) -> void`
  - `canvas_draw_disc(i32, i32, i32) -> void`
  - `canvas_draw_str(i32, i32, i32_ptr) -> void`
  - `canvas_string_width(i32_ptr) -> i32`
- Random:
  - `random_seed(i32) -> void`
  - `random_get() -> i32`
  - `random_range(i32) -> i32`
- Time:
  - `get_time_ms() -> i32`
- Timer:
  - `start_timer_ms(i32) -> void`
  - `stop_timer() -> void`
- Render:
  - `request_render() -> void`
- App control:
  - `exit_to_launcher() -> void`
  - `start_app(i32) -> void`

Runtime details:
- `wasm_runner_call_render()` clears the canvas before invoking `render()`.
- Input is delivered via `on_input(key, type)` if the export exists.
- `request_render()` asks the host to perform one extra render pass after the
  current frame to avoid one-frame-late UI updates.
- `start_timer_ms()` schedules periodic renders at the requested interval until
  `stop_timer()` is called or the app unloads.
- Exceptions during WASM calls are printed to stderr and cleared.

## Input manager

References: `src/runtime/input.h`, `src/runtime/input.c`, `docs/input_system.md`

Input keys:
- `up`, `down`, `left`, `right`, `ok`, `back`

Input types:
- `press`, `release`, `short_press`, `long_press`, `repeat`

Timing constants (milliseconds):
- `INPUT_LONG_PRESS_MS` = 300
- `INPUT_REPEAT_START_MS` = 300
- `INPUT_REPEAT_INTERVAL_MS` = 150
- Reset combo: left + back held for 500 (`INPUT_RESET_COMBO_MS`)

Behavior:
- Raw press/release events are queued immediately.
- Short/long is synthesized on release if long press not yet fired.
- Long press fires once after 300ms if key remains down.
- Repeat events fire every 150ms after long press is active.
- Reset combo callback triggers after left+back held for 500ms.

## Random

References: `src/runtime/random.h`, `src/runtime/random.c`

- Mersenne Twister MT19937-like implementation with 624 state size.
- `random_seed(seed)` resets state; `random_get()` returns 32-bit.
- `random_range(max)` returns [0, max).

## Trace instrumentation (tests)

References: `src/runtime/trace.h`, `src/runtime/trace.c`

- When compiled with `FRD_TRACE`, runtime records calls and results.
- Used by trace tests to compare runtime behavior across platforms.
- Porting note: not required for functionality, but needed for test parity.
