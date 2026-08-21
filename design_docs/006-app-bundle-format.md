# 006 — App bundle (`.fab`) and `manifest.toml`

## Source side: one folder per app

```
apps/snake/
  Cargo.toml        crate fri3d-app-snake, cdylib, depends on fri3d-wasm-api
  manifest.toml     id, name, version, author, description, category, icon, system
  icon.png          16x16 PNG; opaque pixels snap to the DB32 palette, transparent = hole
  src/lib.rs
```

```toml
id = "snake"                 # [a-z0-9_]{1,23}; the settings namespace; must be unique
name = "Snake"               # ≤ 31 bytes, shown in the launcher
version = "0.1.0"            # ≤ 15 bytes
author = "Fri3d Camp"        # ≤ 31 bytes
description = "Classic snake on a 30x14 grid."   # ≤ 95 bytes, info screen
category = "Games"           # free text, used for ordering only
icon = "icon.png"
system = false               # true grants the `system` settings namespace
```

MicroPythonOS uses reverse-DNS `fullname`s and a JSON manifest with
activities and intent filters. We keep a flat id because there is one
entry point per app and no intent resolution; the id doubles as the
settings namespace, which is why it is restricted to `[a-z0-9_]`.

## Build side: `fri3d-pack`

`cargo run -p fri3d-pack` does, for every `apps/*/manifest.toml`:

1. `cargo build --release --target wasm32-unknown-unknown -p <crate>`.
2. `wasm-opt -Oz --strip-debug` if `wasm-opt` is on the PATH.
3. Decode `icon.png` into 256 palette indices (255 = transparent).
4. Write the 512-byte header + wasm to `build/apps/<id>.fab`.
5. Regenerate `fri3d-apps/src/generated.rs`:
   ```rust
   pub static LAUNCHER: &[u8] = include_bytes!("../../build/apps/launcher.fab");
   pub static APPS: &[&[u8]] = &[ /* launcher order */ ];
   ```

Launcher order: the `launcher` id first (it is not in `APPS`), then user
apps by name, then `system = true` apps. Hosts embed `fri3d-apps` and
pass the slices straight to the kernel; the desktop host can also load
`.fab` files from a directory (`--apps-dir`).

## Binary layout

Fixed 512-byte header, little-endian, NUL-padded strings. Reader and
writer share one definition in `fri3d_kernel::bundle` (`Bundle` parses,
`HeaderBuilder` writes; the pack tool depends on the kernel crate so the
layout cannot drift).

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | `FAB1` |
| 4 | 2 | format version = 2 |
| 6 | 2 | flags: bit 0 = system |
| 8 | 24 | id |
| 32 | 32 | name |
| 64 | 16 | version |
| 80 | 32 | author |
| 112 | 96 | description |
| 208 | 1 | icon width = 16 |
| 209 | 1 | icon height = 16 |
| 210 | 1 | payload kind: 0 = wasm (reserved for a native AOT payload) |
| 240 | 4 | payload length |
| 256 | 256 | icon: one DB32 index per pixel, row-major, 255 = transparent |
| 512 | n | payload |

`app_info(i, ptr, 512)` copies this header verbatim into the app's
memory; `fri3d_wasm_api::AppInfo` reads the fields back at the same
offsets. The launcher therefore needs no string parsing and no
allocation to show every installed app.

## Why 16×16 icons

The home grid (design doc 017) draws icons at 2× in 70×62 cells and lists
draw them at 1×. 16×16 is the size pixel artists expect, it scales to a
crisp 32×32, and 256 bytes per app is nothing next to the wasm payload.
