# 001 — Performance lessons from TigerBeetle

Sources: ixuvo.com "TigerBeetle core system architecture & performance
engineering"; `tigerbeetle/docs/TIGER_STYLE.md`.

TigerBeetle is a financial database. The badge is a 240 MHz microcontroller
with a 320x240 screen driven from a 160x120 canvas. The shapes match: one loop, fixed buffers, fixed-size
messages, no heap after boot, a deterministic core that runs on a laptop.

## Lessons and how the kernel applies them

### Static allocation at startup
**Lesson.** Allocate every buffer once at init. Name every limit.
**Kernel.** `fri3d_kernel::limits` lists every cap: apps, fuel, memory,
input queue, settings entries, log lines. `Canvas` is a fixed
`[u8; 19200]`. `InputManager` is a `heapless::Deque`. `Settings` is a
fixed table. The only allocator user is wasmi, and it allocates at app
start, not per frame.

### Zero allocation in steady state
**Lesson.** No `malloc` in the hot path. No long-tail latency from the
allocator; no use-after-free class of bugs.
**Kernel.** Host imports read app strings straight out of wasm linear
memory (`&[u8]` slice) and hand them to the font renderer. No `String`,
no `Vec` per call.

### Put a limit on everything; bounded loops
**Lesson.** Every loop and queue has a fixed upper bound. No recursion.
**Kernel.** Every app call runs with a fuel budget
(`FUEL_PER_CALL`). Out of fuel = trap = the kernel kills the app and
returns to the launcher with an error line. Linear memory is capped by
a wasmi `StoreLimits`. The render loop does at most two passes per step.
The app stack depth is two (launcher + one app), not a list.

### Fixed-size records, zero-copy
**Lesson.** Flat structs sized to cache lines. No serialization layer.
**Kernel.** The `.fab` bundle has a fixed 256-byte header. The registry
holds `&'static [u8]` slices into flash. `app_info()` copies that header
to the app in one `memcpy`; the launcher reads name and icon at fixed
offsets. No parser, no TOML on the badge.

### Single-threaded event loop
**Lesson.** One thread mutates state. I/O is asynchronous around it.
**Kernel.** `Kernel::step(now_ms)` is the only entry point that runs
app code. Hosts may poll buttons from another core, but they hand
transitions over through `push_raw_input` on the kernel's thread.

### Batching
**Lesson.** Amortize per-item cost. Separate control plane from data
plane.
**Kernel.** `step` drains every pending input event, then renders once.
`canvas_draw_buffer` lets an app push a whole frame in one host call
instead of 19200 `canvas_draw_dot` crossings. The host blits only when
the framebuffer actually changed.

### Determinism and simulation
**Lesson.** Same inputs, same outputs. Replay bugs from a seed.
**Kernel.** The kernel takes the clock as a parameter. Tests construct a
kernel, push scripted input with fake timestamps, and hash the
framebuffer. The desktop, browser and badge hosts run the same bytes.

### Assertions
**Lesson.** Two assertions per function. Assert both the positive and
the negative space. Handle every error.
**Kernel.** Host imports validate every argument (key ids, font ids,
pointer ranges) and `debug_assert!` invariants. A trap in an app is an
error the kernel handles, never a panic that reboots the badge.

### Napkin math first
**Lesson.** Do the resource math before coding.
**Badge numbers.**
- Framebuffer: 19 KB mono → 320x240 RGB565 blit = 150 KB → at 40 MHz SPI
  ≈ 30 ms. That is the floor per frame; rendering only on change is what
  makes the launcher idle at ~0% CPU.
- wasmi interpretation: ~20–50x slower than native. A 160x120 Mandelbrot
  at 30 iterations ≈ 600k multiply-adds → 2.5M fuel measured →
  ~100–300 ms on ESP32-S3. Hence the fuel cap and the bulk-blit import.
- One Rust app: 1–2 wasm pages (64–128 KB) with a 16 KB stack. 8 MB
  PSRAM holds this many times over; internal SRAM (320 KB) is for the
  interpreter stack and DMA buffers only.

### Developer-experience rules worth copying
- Explicit integer widths: `u8`/`u16` for coordinates documents the
  160x120 limit.
- Push `if`s up, `for`s down: `step()` decides, leaves compute.
- Units last: `timeout_ms`, `fuel_per_call`.
