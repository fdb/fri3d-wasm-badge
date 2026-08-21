# Fri3d WASM Badge — Agent Guidelines

Everything is Rust. One kernel crate (`fri3d-kernel`, `no_std`), three
hosts (desktop, web, badge), apps as wasm bundles. Read
[design_docs/](design_docs/README.md) before changing the kernel.

## Default workflow: desktop-first, browser second, hardware last

All three hosts run the same kernel bytes, so any bug that reproduces in
the desktop host reproduces on the badge. Use the cheapest loop that
shows the problem:

1. **Kernel tests** (`cargo test -p fri3d-kernel`) — milliseconds. Lifecycle,
   input timing, fuel, settings policy. Add a WAT-based test in
   `fri3d-kernel/tests/lifecycle.rs` for kernel behaviour.
2. **Headless desktop** — sub-second. Deterministic input scripts and
   PNG screenshots:
   ```bash
   cargo run -q -p fri3d-pack                      # after any app change
   cargo run -q --release -p fri3d-host-desktop -- \
       --headless --app snake --keys ok,down --frames 2 --screenshot out.png
   ```
   Prints fuel per render and app memory. Look at the PNG.
3. **Browser harness** — `hosts/web/build.sh`, serve `hosts/web/dist`,
   open `/test.html` and read `window.testResults` (Playwright-friendly).
   `window.fri3d` exposes `tap`, `render`, `readFb`, `startApp`.
4. **Badge** — `hosts/badge/flash.sh`. 20–30 s per round-trip, and only
   after the desktop and browser hosts agree.

## Hard rules

- **The kernel stays `no_std` and `#![deny(unsafe_code)]`.** No `std`
  types in `fri3d-kernel/src`. Hosts provide the allocator.
- **No allocation in the frame path.** Host imports borrow wasm memory
  as slices. New tables get a constant in `fri3d_kernel::limits`.
- **Every app call is fuel-capped.** Do not add an import that loops on
  app-controlled input without a bound.
- **Bundle layout is shared.** Change `fri3d_kernel::bundle` and
  `fri3d_wasm_api::AppInfo` together; bump `FORMAT_VERSION` if offsets
  move.
- **Canvas changes need a screenshot.** After touching
  `fri3d-kernel/src/canvas.rs` or `font.rs`, run the headless desktop
  host on `test_drawing` (`--keys right` cycles scenes) and look.
- **Apps are `no_std`, no `alloc`.** State in `static AppCell<T>`.

## Adding an app

1. `apps/<id>/` with `Cargo.toml` (crate `fri3d-app-<id>`, cdylib),
   `manifest.toml`, a 14×14 `icon.png`, `src/lib.rs`.
2. Add `"apps/<id>"` to the workspace `members` in `Cargo.toml`.
3. `cargo run -p fri3d-pack` regenerates `fri3d-apps/src/generated.rs`.
4. Screenshot it headlessly; then browser; then badge.

## Hardware gotchas

- LCD reset and backlight go through the CH32 expander on I²C1
  (SDA 39 / SCL 42, addr 0x50), not GPIOs. See design_docs/003.
- **The CH32 boots in parallel with the ESP32 and ignores I²C for up to
  ~1 s on a cold start.** Every boot-time write to it (LCD reset, aux
  power, brightness) must be retried until it reads back. Fire-and-forget
  writes work after a USB flash (CH32 already warm) and fail on battery.
- Buttons (except START on GPIO 0) are read from expander register 0x04.
- 8 MB octal PSRAM: the kernel and wasmi memories live there; keep the
  interpreter stack in internal SRAM.
- Bootloader mode when a flash fails: hold START (GPIO 0) and replug USB-C.

## Commit / ship conventions

Short imperative titles ("Add X", "Fix Y"). One-line subject plus an
optional paragraph for "why not what". Use `/ship` to commit + push;
main branch doesn't need PRs.
