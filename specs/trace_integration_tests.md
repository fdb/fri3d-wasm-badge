# Trace-Based Integration Tests Plan

## Goal
Add deterministic, trace-based integration tests for WASM apps. Tests should:
- Feed scripted input events to an app.
- Capture a trace of host API calls (canvas/random/app lifecycle).
- Compare the trace against expectations (subset or exact).
- Run in CI alongside visual tests.

## Scope
- **Runtime:** emulator (native) harness first; web can reuse later.
- **Apps:** start with `circles`, `test_drawing`, `test_ui`, `mandelbrot`.
- **APIs traced:** `canvas_*`, `random_*`, `exit_to_launcher`, `start_app`.

## Design Overview
We introduce a trace layer that wraps host API calls and records them with timestamps, arguments, and optional frame index. The test harness uses the existing `wasm_runner` + `input_manager` to:
1) Load an app.
2) Apply a deterministic input script (press/release/short/long/repeat events).
3) Run for N frames.
4) Serialize the trace to JSON.
5) Compare with an expected spec.

## Proposed File Layout
```
/tests/trace/
  run_trace_tests.py          # driver (similar to visual tests)
  specs/
    circles/basic.json
    mandelbrot/zoom_out_long.json
  expected/
    circles/basic.json         # golden trace (exact)
/fri3d-trace-harness/
  src/main.rs                 # native CLI for traces (Rust)
```

## Trace Format (JSON)
### Basic schema
```json
{
  "app": "circles",
  "seed": 42,
  "frames": 2,
  "events": [
    {"frame": 0, "fn": "canvas_set_color", "args": [1]},
    {"frame": 0, "fn": "canvas_draw_circle", "args": [10, 12, 5]},
    {"frame": 0, "fn": "canvas_draw_circle", "args": [30, 20, 7]},
    {"frame": 0, "fn": "random_range", "args": [128], "ret": 10}
  ]
}
```

### Expected spec modes
- **Exact:** `expected_mode = "exact"` — trace must match exactly (order + args).
- **Contains:** `expected_mode = "contains"` — each listed entry must appear somewhere in trace.

Example spec:
```json
{
  "app": "circles",
  "expected_mode": "contains",
  "required": [
    {"fn": "canvas_draw_circle", "args": [10, 12, 5]},
    {"fn": "canvas_draw_circle", "args": [30, 20, 7]}
  ]
}
```

## Implementation Plan

### Phase 1 — Trace infrastructure
1. **Add trace hooks** in `wasm_runner`:
   - Create `trace.h/.c` with a simple recorder:
     - `trace_begin(frame)`
     - `trace_call(fn_name, args, argc)`
     - `trace_result(retval)` (optional)
     - `trace_flush(path)`
   - Provide a compile-time flag `FRD_TRACE` to enable/disable.

2. **Wrap native symbols**:
   - In `wasm_runner.c`, for each native function (canvas/random/app), call `trace_call` before invoking the real host function.
   - Keep overhead low; when tracing disabled, macros compile to no-ops.

3. **Add frame indexing**:
   - In the harness, call `trace_begin(frame)` once per frame.

### Phase 2 — Harness
1. **Create `fri3d-trace-harness/src/main.rs`**:
   - CLI: `trace_harness --app build/apps/circles/circles.wasm --frames 2 --input script.json --out trace.json`.
   - Load app with `wasm_runner`.
   - Apply scripted input events (press/release/short/long/repeat).
   - Run N frames (`wasm_runner_call_render`).

2. **Input script format** (JSON):
```json
[
  {"time_ms": 0, "key": "ok", "type": "press"},
  {"time_ms": 120, "key": "ok", "type": "release"},
  {"time_ms": 400, "key": "back", "type": "long_press"}
]
```

3. **Time handling**:
   - Use a simulated clock in the harness to drive `input_manager_update` with deterministic timing.

### Phase 3 — Comparator and runner
1. **`tests/trace/run_trace_tests.py`**:
   - Read specs under `tests/trace/specs/`.
   - Run harness to produce trace.
   - Compare trace vs expected spec:
     - `exact`: full ordered equality
     - `contains`: each required entry appears in order (or any order, configurable)

2. **Output**:
   - Print diffs for missing or mismatched calls.

### Phase 4 — Examples
- **circles/basic.json** (contains mode): ensure at least 10 `canvas_draw_circle` calls on frame 0.
- **mandelbrot/zoom_out_long.json**: simulate long-press back, expect `exit_to_launcher` call.
- **test_ui/menu_scroll.json**: hold down for repeat and verify multiple `ui`-related calls if traced or just verify key input path (if input trace is added).

## Example Specs

### circles/basic.json
```json
{
  "app": "circles",
  "frames": 1,
  "expected_mode": "contains",
  "required": [
    {"fn": "canvas_set_color", "args": [1]},
    {"fn": "canvas_draw_circle"}
  ]
}
```

### mandelbrot/exit_long.json
```json
{
  "app": "mandelbrot",
  "frames": 1,
  "input": [
    {"time_ms": 0, "key": "back", "type": "press"},
    {"time_ms": 350, "key": "back", "type": "release"}
  ],
  "expected_mode": "contains",
  "required": [
    {"fn": "exit_to_launcher"}
  ]
}
```

## Open Questions
- Should we trace input events too (to debug event sequencing)?
- Do we want per-app random seeds to make traces deterministic?
- Should traces include canvas state (font/color) at each draw call?

## Notes
- Keep trace harness separate from visual tests to avoid display dependencies.
- Use compile-time flags to avoid trace overhead in release builds.
- This plan assumes the native emulator path first; web harness can be added later.
