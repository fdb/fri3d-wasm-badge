# 005 — Kernel architecture

Crate: `fri3d-kernel`. `#![no_std]` + `alloc`, `#![deny(unsafe_code)]`.
One crate, three hosts. The host owns the display, the buttons, the clock
and persistence; the kernel owns everything else.

## The host contract

```rust
let mut k = Kernel::new();
k.set_launcher(fri3d_apps::LAUNCHER)?;       // one resident app
for a in fri3d_apps::APPS { k.add_app(a)?; }  // registry order = app index
k.load_settings(&persisted_bytes);            // optional
k.boot(now_ms);
loop {
    k.push_raw_input(InputKey::Ok, pressed, now_ms);  // on every edge
    let r = k.step(now_ms);
    if r.frame { blit(&k.framebuffer()); }
    if k.take_settings_image(&mut img) { persist(&img); }
    while let Some(line) = k.take_log_line() { log(line); }
    sleep_until(r.next_wake_ms or next input);
}
```

Three calls per iteration. No callbacks into the host, no traits to
implement: that keeps the wasm32 (browser) host trivial and keeps the
kernel testable with a fake clock.

## State, allocated once

| Table | Size | Bound |
| --- | --- | --- |
| `Canvas` | 19 KB inline `[u8; 19200]` (160×120) | fixed |
| `InputManager` | 7 key states + `Deque<_, 32>` | `limits::INPUT_QUEUE` |
| `Registry` | `heapless::Vec<Bundle, 32>` of `&'static [u8]` | `limits::MAX_APPS` |
| `Settings` | 64 × 52-byte entries | `limits::SETTINGS_ENTRIES` |
| log ring | 8 × 96 bytes | `limits::LOG_LINES` |
| launcher instance | wasmi store + ≥1 wasm page | `limits::APP_MEMORY_MAX` |
| app instance | same, 0 or 1 of them | |

wasmi allocates when an instance is created (module translation, the
linear memory, the value stack). That happens at launch and at return-
to-launcher, never during a frame. Hosts should put the `Kernel` in a
`Box` — on the badge that means PSRAM — and keep stacks in internal RAM.

## The step

```text
step(now):
  input.update(now)                       -- long-press / repeat synthesis
  for ev in input:                        -- bounded by the queue
      Menu short press && app focused  -> exit app
      else focused.on_input(ev); needs_render = true; apply request
  reset combo (Left+Back 500 ms)       -> exit app
  needs_render |= focused.timer_due(now) | focused.render_requested
  if needs_render:
      up to 2×: clear canvas; focused.render(); apply request
  return { frame, next_wake_ms }
```

Rendering is event-driven. An idle launcher costs nothing; an app that
wants animation starts a timer (`start_timer_ms`) and the host can sleep
until `next_wake_ms`.

## Lifecycle

Two slots: launcher (resident) and app (0 or 1). Per slot the order is
`load → on_start → on_resume → … → on_pause → on_stop → drop`. The
launcher is paused while an app runs and resumed when it exits; it is
never stopped. All lifecycle exports are optional.

Ways an app exits, all ending in the same `stop_app`:
- it calls `exit_to_launcher()` or `start_app(i)` (the request is applied
  after the current call returns, so an app never sees its own teardown);
- the kernel's Menu short press;
- the reset combo;
- a trap: `unreachable`, memory out of bounds, **or out of fuel**.

If the launcher itself traps, the kernel reloads it. If the launcher
cannot load, the kernel draws its own error screen. There is no panic
path from app behaviour to the host.

## Protection

- **Fuel.** Every call into wasm gets `FUEL_PER_CALL` (40M) instructions.
  Measured use: Mandelbrot deep zoom 1.1M, launcher 2k, Snake 600.
- **Memory.** `StoreLimits`: one memory, 256 KB max, `memory.grow` past
  it traps.
- **Pointers.** Every host import that reads app memory bounds-checks
  the range and clips strings at 256 bytes. Bad pointers draw nothing;
  they never trap.
- **Settings.** An app may read and write its own namespace. System apps
  (launcher, settings) may also use `system`. Tested in
  `tests/lifecycle.rs::settings_policy_enforced`.

## Host imports (module `env`)

| Group | Functions |
| --- | --- |
| canvas | `canvas_clear/width/height/set_color/set_font`, `draw_dot/line/frame/box/rframe/rbox/circle/disc/str`, `string_width`, `draw_buffer(ptr,len)`, `draw_bitmap(x,y,w,h,ptr)` |
| random / time | `random_seed/get/range`, `get_time_ms` |
| timers | `start_timer_ms`, `stop_timer`, `request_render` |
| apps | `exit_to_launcher`, `start_app(i)`, `app_count`, `app_info(i,ptr,256)`, `kernel_version` |
| settings | `settings_get_u32(ns,key,default)`, `settings_set_u32(ns,key,value)` |
| debug | `log_str(ptr)` |

App exports: `render` (required), `on_input(key,kind)`, `on_start`,
`on_resume`, `on_pause`, `on_stop`, and the test hooks `get_scene`,
`set_scene`, `get_scene_count`.

## Testing

- Unit tests per module (input timing, bundle layout, settings image,
  timer wrap-around).
- `tests/lifecycle.rs`: hand-written WAT apps drive the real kernel: a
  recorder app, an infinite-loop app (killed by fuel), a trapping app, a
  launcher that requests an invalid index, a non-system app that tries to
  write `system`.
- `fri3d --headless --app snake --keys ok,down --screenshot out.png`: the
  desktop host as a deterministic screenshot tool; same input script,
  same bytes, on every machine.
