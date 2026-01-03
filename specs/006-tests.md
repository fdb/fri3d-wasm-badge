# Stage 006: Visual + Trace Tests

This stage documents the regression tests that validate rendering and runtime behavior.
Run these tests after implementing the pre-IMGUI apps (Stage 005) and before IMGUI.

## Visual regression tests

References: `tests/visual/`, `tests/visual/run_visual_tests.py`, `.github/workflows/visual-tests.yml`

Purpose:
- Render deterministic frames from WASM apps and compare against golden images.
- Detect rendering changes (canvas primitives, fonts, layout, IMGUI behavior).

How it works (high level):
- The emulator is run in headless mode with `--screenshot` for each test spec.
- Some apps use scene APIs (`get_scene_count`, `set_scene`) to render multiple frames.
- Output images are compared against golden PNGs.

Update flow:
- Run: `uv run tests/visual/run_visual_tests.py`
- Update golden images: `uv run tests/visual/run_visual_tests.py --update-golden`

CI requirement:
- If a new app is added with visual tests, add its build to `.github/workflows/visual-tests.yml` so the WASM binary is built before tests.
- Visual tests should run in GitHub Actions CI.

Porting expectations:
- Pixel parity is the goal; use the same graphics/canvas implementation across platforms.
- Fonts, XOR drawing, and layout must match to keep goldens stable.

## Trace integration tests

References: `tests/trace/`, `tests/trace/run_trace_tests.py`, `build_trace_tests.sh`, `.github/workflows/trace-tests.yml`

Purpose:
- Validate runtime behavior by recording and comparing host call traces.
- Ensures WASM imports (canvas/random/time/app control/input) are invoked consistently.

How it works (high level):
- The runtime is compiled with `FRD_TRACE` enabled.
- Each app/spec run records a sequence of host calls and results.
- Traces are compared against expected JSON snapshots.

Update flow:
- Run: `./build_trace_tests.sh`
- Update a specific golden trace:
  - `cp tests/trace/output/<app>/<spec>.json tests/trace/expected/<app>/<spec>.json`

CI requirement:
- If a new app is added with trace specs, add its build to `.github/workflows/trace-tests.yml`.
- Trace tests should run in GitHub Actions CI.

Porting expectations:
- Trace output should match in call order and argument values.
- Differences indicate behavioral changes in the runtime or input/timing logic.
