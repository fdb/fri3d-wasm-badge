# Stage 008: App Model and Built-in Apps (IMGUI)

This stage documents the built-in apps that depend on IMGUI.

## IMGUI dependency

These apps require the IMGUI system described in `specs/007-imgui.md` and should be implemented only after IMGUI is available.

## Built-in apps (IMGUI)

### Test UI (`src/apps/test_ui/main.c`)

- Multi-scene IMGUI demo: counter, menu, layout, progress, checkbox, footer, keyboard.
- Uses virtual keyboard with validation (min 3 chars, no spaces).
- Back long press exits; Back short/repeat cycles scenes.
- Scene API: `get_scene_count()` returns SCENE_COUNT.

## Porting expectations

- IMGUI behavior must match to keep this app's visuals and input flow consistent.
- Run visual + trace tests after IMGUI and these apps are implemented.
