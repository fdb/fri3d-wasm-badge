# Fri3d WASM Badge — Agent Guidelines

## Default workflow: browser-first, hardware-last

**Firmware changes must be verified in the browser before being flashed.**
The firmware compiles to two targets from the same sources:

- **Hardware** (`firmware/`): PlatformIO + Arduino + wasm3 for the ESP32-S3.
- **Browser** (`firmware/web/`): Emscripten + wasm3 + an HTML canvas.

Both paths run identical `canvas.cpp`, `random.cpp`, `wasm_host.cpp`, and wasm3.
Only the entry point and host/IO layer differ. Any bug that reproduces in the
browser will reproduce on hardware, and vice versa — **use the browser as the
fast feedback loop, then confirm on hardware once**.

### Build + run the browser harness

```bash
# Rebuilds the Rust app, re-embeds the .wasm, emcc-compiles firmware to WASM.
firmware/web/build.sh

# Serve the harness.
cd firmware/web/dist && python3 -m http.server 8090
# Open http://localhost:8090/  (interactive) or /test.html (runs the suite).
```

The harness exposes `window.fri3d` (see [firmware/web/index.html](firmware/web/index.html)):

- `fri3d.tap(key, heldMs=50)` — one PRESS + RELEASE + SHORT/LONG_PRESS.
- `fri3d.render()` — run one render frame.
- `fri3d.readFb()` — copy the 128×64 mono framebuffer out as `Uint8Array`.
- `fri3d.rngGet()` / `fri3d.rngSeed(s)` — poke the host MT19937 directly.
- `fri3d.KEY` — `{UP, DOWN, LEFT, RIGHT, OK, BACK}` constants.

### Default loop for any feature / bugfix

1. **Reproduce or spec the behaviour as a test in [firmware/web/tests.js](firmware/web/tests.js).**
   For bugs, write a failing test first (red). For features, write the test
   that defines "done."
2. **Run the suite** by navigating Playwright to `test.html` and reading
   `window.testResults` — this is fully automatable.
3. **Fix until green.**
4. **Rebuild** via `firmware/web/build.sh` and **reload** the page.
5. **Only then** rebuild + flash hardware with `cd firmware && pio run -t upload`
   to confirm on the real badge. Hardware round-trips are expensive (10–30 s
   per flash plus possible USB-CDC wedging); browser round-trips are sub-second.

### Keeping the browser build up to date

The browser build **must always compile and pass tests** on every firmware
change. Anything that breaks the browser build is a regression. If a feature
genuinely can't be tested in the browser (e.g. hardware-only timing), split
the logic so the portable parts land in `canvas/random/wasm_host` and only a
thin shim lives in `main.cpp`. Never add hardware-only logic to files shared
with the web build.

## Canvas parity: keep C++ bit-exact with the Rust reference

The C++ `firmware/src/canvas.cpp` must produce byte-identical framebuffers
to `fri3d-runtime/src/canvas.rs` for every drawing primitive. This is
essential for apps like the keyboard that depend on pixel-perfect rendering.

**Workflow for adding or modifying a canvas primitive:**

1. Add a test case in [tools/canvas-parity-gen/src/main.rs](tools/canvas-parity-gen/src/main.rs)
   that calls the primitive on a fresh Rust Canvas and dumps the resulting
   framebuffer as a golden `.bin`.
2. Mirror the same case in [firmware/tools/canvas-parity/canvas_parity.cpp](firmware/tools/canvas-parity/canvas_parity.cpp).
3. Run:
   ```bash
   firmware/tools/canvas-parity/run.sh
   ```
   — regenerates goldens, rebuilds the C++ tester, diffs byte-for-byte.
   Any mismatch = a primitive that's drifted. Report shows first differing
   pixel coordinate.

Run this script after any `canvas.cpp` or `canvas.rs` change. CI will
eventually gate on it.

## Hardware gotchas (earned the hard way)

- **Don't call `Serial.flush()` anywhere near boot.** On the ESP32-S3 native
  USB-CDC, `flush()` blocks until a host reads — which means a standalone badge
  with no laptop attached hangs forever at the first `Serial.flush()`.
- **Always use `SET_LOOP_TASK_STACK_SIZE(32 * 1024)`** for wasm3.
  `CONFIG_ARDUINO_LOOP_STACK_SIZE` via `-D` silently loses to `sdkconfig.h`;
  the weak-function override doesn't.
- **Keep Rust WASM stack at 16 KB** via `.cargo/config.toml` `-C link-arg=-zstack-size=16384`.
  Rust's default 1 MB stack becomes a 17-page (1 MB) linear memory request
  that wasm3 struggles to satisfy in ESP32's heap layout.
- **USB-CDC re-enumeration wedges the host-side port** after repeated panic
  reboots. If `pio upload` fails with "Failed to connect" and the serial port
  is enumerating, hold `START` (GPIO0) + replug USB-C to force bootloader mode.

## Project structure

```
firmware/
  src/               # Shared C++ sources — compile for both hardware and web
    canvas.{h,cpp}   # 128×64 mono framebuffer (emulator-compatible semantics)
    random.{h,cpp}   # MT19937 port, bit-exact with fri3d-runtime/src/random.rs
    wasm_host.{h,cpp}# wasm3 integration — 24 host imports matching fri3d-wasm-api
    host_platform.h  # host_log / host_millis abstraction
    main.cpp         # Hardware main (TFT_eSPI + Fri3d buttons)
    embedded_app.h   # Auto-generated; run firmware/scripts/embed_app.sh to refresh
  lib/wasm3/         # Vendored wasm3 v0.5.0 (shared by both targets)
  web/               # Emscripten browser build
    main_web.cpp     # extern "C" shims called from JS
    index.html       # Interactive harness
    test.html, tests.js # Test suite
    build.sh         # emcc build
  scripts/embed_app.sh # Builds a Rust app in release mode, wasm-opt's it, generates embedded_app.h
  tools/hosttest/    # Native Unix wasm3 host for quick debugging
  platformio.ini     # Hardware build config
```

## Commit / ship conventions

Existing repo uses short imperative titles for commit messages ("Add X",
"Fix Y"). One-line subject plus optional paragraph for "why not what."
Use `/ship` to commit + push; main branch doesn't need PRs.
