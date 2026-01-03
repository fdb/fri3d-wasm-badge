# Rust Rewrite Plan (Summary)

Purpose: guide a staged Rust reimplementation while keeping the current C/C++ code intact for reference and parity checks.

## Goals

- Preserve behavior and output across platforms (emulator, web, firmware).
- Keep the same WASM app API and runtime semantics as the C runtime.
- Use a Rust workspace with multiple crates (e.g., `fri3d-runtime`, `fri3d-emulator`, `fri3d-platform-*`).
- Use Rust-native dependencies where appropriate (e.g., `wasmi` for WASM, `minifb` for desktop display).
- Replace u8g2 with a Rust-owned graphics pipeline (framebuffer + drawing primitives), but maintain identical pixel output.
- Keep C code in place for reference and fallback until Rust reaches feature parity.
- Avoid `unsafe` by default across runtime and apps; isolate any unavoidable `unsafe` in small, well-audited boundaries or alternative safe APIs.
- Avoid the `sdk` label in Rust; prefer `fri3d-runtime` (and related crates) for shared APIs.

## Staged plan

### Stage 001: Basics

- Document core invariants: 128x64 1bpp framebuffer, input model, WASM imports/exports, full-frame redraw, deterministic tests.
- Define platform-neutral architecture: platform layer (display/input/time) + shared runtime + apps.
- Rust mapping: define shared crates, traits, and types that mirror the runtime boundaries.

### Stage 002: Platforms

- Desktop emulator:
  - Rust windowing via `minifb` (or similar), 4x scale, framebuffer-to-RGBA conversion.
  - Input mapping identical to SDL emulator: arrows/OK/Back, no OS key repeats.
  - Screenshot support for tests (PNG encoding in Rust).
- Web emulator:
  - Plan to use wasm32 target and JS glue later; keep parity with runtime behavior.
- ESP32 firmware:
  - Plan to reuse runtime crate; keep hardware bindings separate.

### Stage 003: Runtime Core

- App manager:
  - Registry of apps, launcher handling, app switching, error storage.
- WASM runner:
  - Use `wasmi` to load/instantiate modules.
  - Register imports under module name `env` with the same signatures.
  - Require `render()`, optional `on_input`, `get_scene`, `set_scene`, `get_scene_count`.
  - Clear canvas before each `render()`.
- Input manager:
  - Match timing constants and queue behavior exactly.
  - Synthesize short/long/repeat; reset combo left+back held 500ms.
- Random:
  - Reproduce MT19937-compatible behavior with identical outputs.
- Trace instrumentation:
  - Provide optional trace hooks for parity tests (feature-flagged).

### Stage 004+: Graphics and Apps

- Implement a Rust framebuffer and drawing primitives that match C output:
  - Dot/line/frame/box/rframe/rbox/circle/disc/text, XOR rules.
  - Match font rendering; provide internal font assets or equivalent bitmaps.
- Port SDK bindings for apps and web runtime glue.
- Migrate apps incrementally with test parity.
  - Prefer a safe Rust app API; if WASM ABI constraints require `unsafe`, wrap them behind a safe, minimal SDK layer.

## Deliverables (early)

- Workspace root `Cargo.toml` with member crates.
- `fri3d-runtime` crate with core runtime modules.
- `fri3d-emulator` crate with display/input/time backends.
- Baseline tests or validation harness to compare outputs against C runtime.

## Constraints

- Keep WASM import module name `env` and function signatures unchanged.
- Preserve input timing constants and reset combo behavior.
- Keep launcher and app IDs consistent with existing registry.
- Prefer safe Rust APIs; if `unsafe` is required, confine it to a small FFI boundary with tests.
