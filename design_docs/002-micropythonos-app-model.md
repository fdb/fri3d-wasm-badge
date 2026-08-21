# 002 — MicroPythonOS app model

Source: github.com/microPythonOS/MicroPythonOS (`lib/mpos/app/*.py`,
`lib/mpos/content/app_manager.py`, `builtin/apps/com.micropythonos.launcher`).

## What MPOS does

**App layout.** One directory per app, reverse-DNS name:
`apps/com.micropythonos.helloworld/`. Contents: `MANIFEST.JSON`,
`icon_64x64.png`, one or more `.py` entry points. Packaged as `.mpk`
(zip). Builtin apps live in `builtin/apps/` and cannot be uninstalled.

**Manifest.**
```json
{"name": "HelloWorld", "publisher": "MicroPythonOS",
 "short_description": "Minimal app", "long_description": "...",
 "fullname": "com.micropythonos.helloworld", "version": "0.2.0",
 "category": "development",
 "activities": [{"entrypoint": "hello.py", "classname": "Hello",
   "intent_filters": [{"action": "main", "category": "launcher"}]}]}
```
Optional `services` with `boot_completed` intent filters.

**Lifecycle.** Android-shaped. `Activity` has `onCreate`, `onStart`,
`onResume`, `onPause`, `onStop`, `onDestroy`, `onBackPressed`. A global
screen stack: `setContentView` pauses+stops the top, pushes, starts +
resumes the new one. `finish()` pops: pause, stop, destroy, resume the
previous. The launcher is always the bottom of the stack. MENU toggles a
drawer; X is back.

**Launcher.** A flex grid of 64x64 icons with labels, rebuilt in
`onResume` when the app list changed. Tap → 250 ms splash → `start_app`.

**Settings.** A declarative list of `{title, key, ui, default_value,
changed_callback}`. Persists JSON under `data/prefs/<fullname>/`.

## What we keep

- One folder per app with a manifest and an icon next to the code.
- The lifecycle vocabulary: start / resume / pause / stop.
- The launcher is always resident at the bottom.
- A settings app that is itself a regular app.

## What we change, and why

| MPOS | Fri3d kernel | Why |
| --- | --- | --- |
| JSON manifest parsed at boot | `manifest.toml` at build time → fixed 256-byte binary header | No parser on the badge; zero-copy registry |
| Arbitrary-depth activity stack | Depth 2: launcher + one app | Bounded memory; one wasm instance alive besides the launcher |
| Intents, implicit resolution, chooser | `start_app(index)` only | No use case yet; keep the ABI small |
| Services at boot | None | Out of scope for now, by request |
| 64x64 colour PNG icons | 14x14 1-bit icons | 160x120 canvas; Flipper-style list |
| Python classes | `#[no_mangle] extern "C"` exports from a wasm module | No allocation, no GC, sandboxed |
| `onCreate` + `onStart` | `on_start` only | Instantiation *is* create; wasm data segments are the constructor |
| `onDestroy` | none — dropping the instance frees everything | A wasm instance owns its memory; there is nothing to clean up after `on_stop` |
