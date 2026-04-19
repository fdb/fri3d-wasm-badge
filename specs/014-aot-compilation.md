# AOT-Compiled WASM Apps via Build Service

**Status:** draft — investigation spike, not yet committed to landing on `main`.
**Target:** execute on a branch (recommend `git worktree add` in a separate dir)
so it can be shelved cleanly if the tradeoffs don't work out.

## Motivation

wasm3's interpreter is costing us real latency on compute-heavy apps.
Measured on ESP32-S3 with the current Q16.16 + cardioid-check Mandelbrot:

| View | Per-frame cost |
| --- | --- |
| Default (most pixels hit cardioid) | ~225 ms |
| Deep zoom, boundary-heavy | ~800 ms |

The bottleneck at deep zoom is pure interpretation overhead: every pixel
runs ~30 iterations of a simple fixed-point loop, and wasm3 charges
dispatch overhead per bytecode op. There is no algorithmic lever left —
the app already skips ~40% of pixels via cardioid/bulb checks.

A just-in-time / ahead-of-time compile would turn each WASM op into native
Xtensa instructions, eliminating the interpreter dispatch entirely.
Empirically, wasm3 → WAMR-AOT on ESP32 class hardware is 10-30× faster.
For Mandelbrot, that gets us from 800 ms to roughly 30-80 ms per frame
even at deep zoom.

## Proposal

Add a **build service** that compiles `.wasm` → `.aot` (WAMR AOT binary)
for the badge's exact target (`xtensa-esp32s3`), signs the result, and
serves it over HTTP. The badge either embeds a handful of AOTs at build
time or downloads them on demand.

```
┌──────────────┐     .wasm     ┌──────────────────┐     .aot     ┌────────┐
│ Rust app     │──────────────▶│ Build service    │─────────────▶│ Badge  │
│ fri3d-app-*  │               │ wamrc / wasm-opt │              │ (WAMR) │
└──────────────┘               └──────────────────┘              └────────┘
                                  (Cloudflare                      (runtime
                                   Worker or box)                   replaces
                                                                    wasm3)
```

## Why a service (not local wamrc)

- `wamrc` is a cross-compiler that wraps LLVM. It's a ~100 MB install
  and compiles per target triple — putting it on every developer's
  machine is friction.
- AOT output is target-specific. `esp32s3` AOTs won't run on a future
  `esp32p4`. A central service can fan out to multiple targets.
- Signing for AOT trust lives naturally on the server (key stays off
  user machines).
- `embed_apps.sh` stays simple: upload `.wasm`, get back `.aot`, embed.
  No new local toolchain.

## Non-goals (for this spike)

- **Not** a real app store with user accounts / uploads / moderation.
  This service is an *internal build tool* — one trusted build boxy
  running on the LAN or a Cloudflare Worker.
- **Not** replacing the WASM SDK. Apps still write Rust-to-wasm32 via
  `fri3d-wasm-api`. This only changes what's embedded in firmware.
- **Not** a browser-side AOT. The web harness keeps wasm3 (or even
  direct JIT via V8) — AOT targets the badge only.

## Decisions that need answering before commit

Questions the spike should *answer*, not assume:

1. **WAMR footprint.** wasm3 is ~50 KB of flash + ~2.7 KB RAM. WAMR-AOT
   runtime is reportedly 100-300 KB flash + higher RAM. Does it still
   fit comfortably in our 6.5 MB flash / 320 KB SRAM? (Yes on paper,
   but measure.)
2. **Real speedup on our workload.** Synthetic WAMR benchmarks cite
   "10-30×" against pure interpreters — but wasm3 has a very tight
   dispatch loop. Measure Mandelbrot at default view + deep zoom and
   compare to the current 225 ms / 800 ms baseline.
3. **AOT binary size.** Typically 2-5× the .wasm. If our current
   ~3 KB app grows to 15 KB AOT, all 7 apps combined stay under 150 KB
   — fine. If it grows 10×, reconsider.
4. **Boot cost.** Does `m3_ParseModule` equivalent in WAMR-AOT add
   launcher-to-app latency beyond the current ~1 ms? Measure.
5. **Dev loop friction.** Is round-tripping every .wasm change through
   a build service tolerable for local iteration? Mitigation options:
   cache by content hash, run service as a local daemon, or keep
   wasm3 as a fallback in firmware with a build-time flag.
6. **Debugging.** wasm3 traps print module-level info. AOT traps
   print machine-level addresses. Acceptable? Needs a symbol map.

## Architecture detail

### Build service

Single HTTP endpoint:

```
POST /compile
Content-Type: application/wasm
Body: <wasm bytes>
Response: <aot bytes>  (or 4xx with error JSON)
```

Implementation sketch:

- Host: small box on Frederik's network (or a Cloudflare Worker shell
  that shells out to wamrc in a sandboxed build). For the spike, a
  local Python service (`uvicorn` + `subprocess`) is fine.
- Input validation: check WASM magic + bail on modules larger than 1 MB.
- Build step: `wamrc --target=xtensa --target-abi=xtensa-esp32s3 -o /tmp/out.aot /tmp/in.wasm`
- Cache by SHA-256 of input. Don't re-run wamrc on identical input.

### Firmware

Replace wasm3 with WAMR. WAMR's public API is similar:

| wasm3 | WAMR |
| --- | --- |
| `m3_NewEnvironment()` | `wasm_runtime_init()` |
| `m3_ParseModule()` | `wasm_runtime_load()` |
| `m3_LoadModule()` | `wasm_runtime_instantiate()` |
| `m3_LinkRawFunction()` | `wasm_runtime_register_natives()` |
| `m3_FindFunction()` | `wasm_runtime_lookup_function()` |
| `m3_CallV()` | `wasm_runtime_call_wasm_a()` |

All 25 host imports in `firmware/src/wasm_host.cpp` need a WAMR-style
registration (function signatures use a slightly different DSL).

### Embed pipeline

`firmware/scripts/embed_apps.sh` gets an extra step:

```
cargo build --release → fri3d_app_*.wasm
wasm-opt -Oz           → fri3d_app_*_opt.wasm
curl -XPOST service    → fri3d_app_*.aot
xxd -i                 → firmware/src/embedded_apps.h
```

If `FRI3D_AOT_SERVICE` env var is unset, fall back to embedding `.wasm`
directly (and firmware stays on wasm3). This gives us a dev loop that
doesn't need the service.

## Execution plan (suggested order for the worktree)

Each step is a commit on the spike branch. Abort at any point if the
answer to one of the "decisions" questions above rules out the approach.

1. **Measure baseline.** Record current Mandelbrot render time on HW
   at default view + deep-zoom preset. Save numbers in this spec.
2. **Spike WAMR integration.** Vendor or lib-link WAMR into `firmware/lib/`.
   Build firmware with WAMR replacing wasm3. Run launcher + circles +
   mandelbrot. Measure flash + RAM footprint deltas. Measure Mandelbrot
   render time (still loading `.wasm`, not AOT).
3. **Add AOT loading.** Hook WAMR's `wasm_runtime_load_aot()` path.
   Manually build one `.aot` for circles via `wamrc` on macOS
   (install via homebrew or build from source). Embed it. Measure
   render time delta vs the WAMR-interpreter version.
4. **If steps 2+3 show the expected speedup:** build the minimal
   service. Python + FastAPI + `subprocess.run(["wamrc", ...])` in
   a Docker image. One endpoint, SHA-256 cache.
5. **Integrate the service** into `firmware/scripts/embed_apps.sh`.
6. **Decide.** If the whole stack is < 400 KB firmware growth, < 50 ms
   launcher cold-start regression, and Mandelbrot fits in < 100 ms
   per frame: merge to `main`. Otherwise: land the *service* as a
   nice-to-have and keep wasm3 on the badge.

## Abort conditions

Don't merge if:

- WAMR + AOT doesn't give at least a 5× speedup over wasm3-interpreter
  on Mandelbrot (measured, not cited).
- Firmware size grows > 500 KB (our partition is 6.5 MB but we care
  about OTA cost).
- RAM headroom drops below 50 KB free after init.
- AOT binaries are > 10× the `.wasm` size on average.
- Dev round-trip for "edit mandelbrot → see it on hardware" grows by
  more than 10 seconds over the current embed_apps.sh + flash flow.

## What we're NOT committing to yet

- Over-the-air AOT downloads. The first version embeds AOTs at firmware
  build time, just like today's `.wasm`.
- Signing. Spike trusts the build service; add Ed25519 signing
  *after* the perf story checks out.
- Multi-target. Only `xtensa-esp32s3` for now; the service takes a
  single target string and fails on anything else.

## References

- WAMR: https://github.com/bytecodealliance/wasm-micro-runtime
- wamrc docs: https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/wamr-compiler/README.md
- Espressif ESP32-S3 target triple: `xtensa-esp32s3`,
  ABI `xtensa-esp32s3`, typically compiled with `-mcpu=esp32s3` upstream.
- Current wasm3 integration: [firmware/src/wasm_host.cpp](../firmware/src/wasm_host.cpp)
- Current Mandelbrot measurements: render times 225-800 ms depending on
  view, see git log at commit `4ef30be`.
