# 010 — Host contract and what three hosts taught us

The kernel is driven, never driving. A host is anything that can supply
a clock, button edges, a place to put 76 800 bytes, and (optionally)
a place to persist 3 332 bytes.

## The API

```rust
Kernel::new()                                   // allocates every table once
kernel.set_launcher(&'static [u8])              // one bundle
kernel.add_app(&'static [u8]) -> index          // registry order = app index
kernel.load_settings(&[u8])                     // optional, before boot
kernel.boot(now_ms)

kernel.push_raw_input(key, pressed, now_ms)     // idempotent per edge
let r = kernel.step(now_ms) -> StepResult { frame, next_wake_ms }
kernel.framebuffer() -> Ref<[u8]>               // 320*240 DB32 indices (palette::RGB)
kernel.take_settings_image(&mut [u8; IMAGE_LEN]) -> bool   // true when dirty
kernel.take_log_line() -> Option<String<96>>
kernel.setting("system", "brightness") -> Option<u32>
kernel.last_error() -> &str
kernel.stats                                    // frames, fuel, memory, traps
```

Test and tooling extras: `start_app(i)`, `exit_to_launcher()`,
`set_scene(n)`, `request_render()`, `random_seed(s)`, `random_get()`.

## Decisions

**The clock is a parameter.** `step(now_ms)` instead of an internal
timer. Tests run a year of uptime in a millisecond; the browser harness
advances a virtual clock synchronously inside `tap()`; wrap-around at
2³² ms is tested, not hoped for.

**No callbacks out of the kernel.** Logging, settings persistence and
brightness are all *pull*: the host asks after each step. A callback
trait would need `dyn` or generics through wasmi's store type and would
make the wasm32 host awkward. Polling 8 log lines costs nothing.

**Input as edges, not events.** `push_raw_input(key, pressed)` is
idempotent, so a host may call it every poll with the current level
(desktop, badge) or only on transitions (browser). The kernel
synthesises short/long/repeat identically everywhere.

**`StepResult::frame`, not a dirty flag on the canvas.** The kernel
knows when it rendered; the host decides whether to blit (the badge
additionally compares with the last blitted frame to skip identical
output — a `request_render` that changes nothing costs no SPI traffic).

**`next_wake_ms`** lets a host sleep. The badge does not yet; the field
is there so it can.

**`&'static [u8]` bundles.** Registry entries are slices into flash or
into `include_bytes!`. The desktop host leaks files it loads from disk;
that is honest for a process that runs them until exit.

## What each host taught us

### Desktop (minifb)
- Headless mode turned out more valuable than the window: scripted
  input + PNG + fuel/memory numbers make every design question answerable
  in under a second and reviewable in a screenshot.
- Apps that hardcode the screen size fail silently when the size changes.
  The SDK now exports `SCREEN_WIDTH/HEIGHT`; use them.

### Browser (wasm-bindgen)
- wasmi inside wasm works unchanged. Two gotchas: the root
  `.cargo/config.toml` sets a 16 KB wasm stack for *apps*; the host
  module needs 1 MB, so `hosts/web/.cargo/config.toml` overrides it. And
  wasmi's hash collections need entropy; use `prefer-btree-collections`.
- `ui_begin()` clears the canvas. Draw custom content after it.

### Badge (esp-hal)
- **The CH32 coprocessor boots later than the ESP32.** It holds the LCD
  reset line and the backlight. Unacknowledged writes at 300 ms leave the
  panel black on a cold start, while a USB flash (CH32 already warm)
  works. Write, read back, retry.
- **USB-Serial-JTAG resets re-enter the bootloader.** Toggling DTR/RTS
  from a script — or `espflash --monitor` — lands in `waiting for
  download`. A power cycle is the only clean cold boot, and the power
  switch does nothing while USB is attached.
- **Early `println!` is lost.** The JTAG FIFO drops output until a host
  attaches; the firmware waits 300 ms and prints a heartbeat so a late
  reader still sees state.
- **Colour order.** MPOS says BGR + byte swap; with mipidsi that is
  `ColorOrder::Rgb`. The symptom was a blue tint, not garbage.
- **Octal PSRAM** probes fine with `PsramMode::Auto`: 8.4 MB free.
- **Stack.** Bare metal has no task stack limit; the 32 KB wasm3 lesson
  from the Arduino firmware does not apply.
- **rust-lld cannot link Xtensa.** The GCC from `espup` stays as linker.
- **The main stack is what is left of internal DRAM.** `.bss` statics and
  the internal `heap_allocator!` come out of the same 512 KB; a 76 800-byte
  framebuffer copy as a static pushed the stack below what wasmi's
  translator needs to load the launcher (`write to the stack guard`
  panic, bootloop). Big buffers go to the PSRAM heap via `vec!` +
  `into_boxed_slice()` — never `Box::new([0; N])`, which builds the
  array on the stack first. The kernel's own framebuffer is boxed for the
  same reason.

## A fourth host would need

A clock, seven button levels, a 320×240 palette blit, and 3.3 KB of storage.
Nothing else. The web host is 200 lines; that is the budget.
