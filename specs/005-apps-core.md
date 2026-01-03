# Stage 005: App Model and Built-in Apps (Pre-IMGUI)

This stage defines the WASM app API and documents the built-in apps that do not depend on IMGUI.

## App API (WASM)

References: `src/sdk/*.h`, `src/apps/*/main.c`

### Required exports

- `void render(void)`
  - Called every frame by the host.
  - The host clears the canvas before calling this.

### Optional exports

- `void on_input(input_key_t key, input_type_t type)`
  - Called for each input event.
- Scene control (used for tests):
  - `uint32_t get_scene_count(void)`
  - `uint32_t get_scene(void)`
  - `void set_scene(uint32_t scene)`

### Imports (module = `env`)

- Canvas API: see `src/sdk/canvas.h` and `specs/004-canvas.md`.
- Input enums: `input_key_t`, `input_type_t` in `src/sdk/input.h`.
- Time: `get_time_ms()` in `src/sdk/frd.h`.
- App control: `exit_to_launcher()`, `start_app(id)` in `src/sdk/app.h`.
- Random: `random_seed/get/range` in `src/sdk/random.h`.

### Conventions

- Apps usually map Back short press to exit via `exit_to_launcher()`.
- Some apps use Back long press to exit when short press is repurposed.
- Apps should be deterministic when used with visual tests (fixed RNG seeds or scenes).

## Built-in apps (pre-IMGUI)

These apps are used to validate canvas primitives and runtime behavior before IMGUI is implemented.

### Circles (`src/apps/circles/main.c`)

- Draws 10 random circles each frame using `random_range`.
- Seed fixed per frame (`g_seed`) for determinism.
- `OK` press generates a new seed via `random_get()`.
- Back short press exits to launcher.
- Scene API returns 1 scene.

### Mandelbrot (`src/apps/mandelbrot/main.c`)

- Renders a 128x64 Mandelbrot set (50 iterations).
- Arrow keys pan; `OK` zooms in; Back short press zooms out.
- Back long press exits to launcher.
- Scene API returns 1 scene.

### Test Drawing (`src/apps/test_drawing/main.c`)

- Multi-scene app showcasing drawing primitives:
  - Lines, random pixels, circles, filled circles (XOR), rectangles, text, mixed, checkerboard.
- Fixed RNG seed for determinism.
- Left/Up or Right/Down cycles scenes.
- Back short press exits.
- Scene API: `get_scene_count()` returns SCENE_COUNT.

### Snake (`src/apps/snake/main.c`)

- Grid-based snake game with 4x4 pixel cells, board size 30x14.
- Movement ticks every 250ms using `get_time_ms()`.
- Uses `random_range()` to place fruit on even grid positions.
- Back short/long press exits; `OK` restarts after game over.

## Porting expectations

- Preserve input semantics and timings for determinism and tests.
- Keep app rendering consistent with the canvas primitive behavior.
- Run visual + trace tests after this stage (see `specs/006-tests.md`) before implementing IMGUI apps.
- IMGUI-dependent apps are covered later (see `specs/008-apps-imgui.md`).
