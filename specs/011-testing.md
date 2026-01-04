# Stage 011: Stability + Automated Testing

This stage expands testing coverage to keep the Rust rewrite stable and guide
automated coding agents.

## Goals

- Catch regressions in timing, input, and rendering behavior early.
- Provide deterministic, automated signals for CI and local runs.
- Validate energy-saving behavior (event-driven + timers) without masking bugs.

## Test categories

### Runtime unit tests (Rust)

- Input timing: short/long/repeat/reset behavior with deterministic time steps.
- Timer scheduling: start/stop behavior, due logic, catch-up behavior.
- Canvas primitives: buffer output for representative shapes and fonts.

### Trace integration tests

- Extend Rust runtime with a trace harness to mirror C trace tests.
- Capture host call sequences and render loop transitions.
- Add explicit tests for `request_render()` and timer-driven renders.

### Visual regression tests

- Continue using `tests/visual/run_visual_tests.py`.
- Ensure Rust builds are used for screenshots.
- Add new apps to CI build lists when needed.

### Web smoke tests

- Simple headless checks that load the web emulator, render the launcher,
  and switch apps once.
- Validate input and timer behavior in the browser host loop.

## Phase 1: Timer tests (implement now)

- Add deterministic unit tests for timer scheduling:
  - Not due before the interval elapses.
  - Due at exact interval boundaries.
  - Catch-up behavior when time jumps forward.
  - Stop disables future due events.
- Keep tests small and deterministic (no wall-clock time).

## Phase 2: Trace + request_render tests (next)

- Add a Rust trace harness that runs WASM apps for N frames.
- Record host calls and render invocations.
- Add trace specs that assert:
  - Timer-driven renders fire at expected frames.
  - `request_render()` triggers exactly one extra render.

## Phase 3: Web smoke tests

- Scripted browser test that loads `build/web/index.html`.
- Ensure the launcher renders and app switching works.
- Validate timer-driven updates (e.g., progress scene) without user input.
