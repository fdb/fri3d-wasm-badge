# Fri3d WASM Badge — Agent Guidelines

See [CLAUDE.md](CLAUDE.md) for the canonical workflow. Short version:

- **Firmware host lives in `firmware/src/`** — one C++ implementation, three
  targets (ESP32-S3, browser, native CLI).
- **Always verify in the browser or the native runner before flashing hardware.**
  `firmware/web/build.sh` + `firmware/tools/app-runner/build.sh` are sub-second
  feedback loops; `pio run -t upload` is 15+ seconds and can wedge USB.
- **Canvas primitives must stay byte-identical with the Rust reference.**
  Run `firmware/tools/canvas-parity/run.sh` after any `canvas.{h,cpp}` or
  `font.{h,cpp}` change.
- **TDD for bugfixes**: reproduce as a test in `firmware/web/tests.js` first,
  then fix.

## Common commands

```bash
# Rust apps (wasm32-unknown-unknown)
./build_apps.sh

# Browser emulator
firmware/web/build.sh
cd firmware/web/dist && python3 -m http.server 8090

# Native app runner (dumps PNGs)
firmware/tools/app-runner/build.sh
firmware/tools/app-runner/app_runner <wasm> --out out.png

# Full visual parity sweep (C++ host vs Rust goldens)
firmware/tools/app-runner/visual_parity.sh

# Canvas primitive parity (fine-grained)
firmware/tools/canvas-parity/run.sh

# Hardware firmware
cd firmware && pio run && pio run -t upload
```

## Commit style

Short imperative titles ("Add X", "Port Y", "Fix Z"), one-line subject + optional
paragraph explaining the *why*. Ship with `/ship` — main branch doesn't need PRs.
