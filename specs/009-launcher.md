# Stage 009: Launcher

The launcher is a WASM app that lists installed apps and starts them by ID.

Reference: `src/apps/launcher/main.c`

## Behavior

- Uses IMGUI (`ui_begin`, `ui_menu_begin`, `ui_menu_item`, `ui_end`).
- Title: "Fri3d Apps" centered, separator line, menu below.
- App entries (name + ID) are statically defined in the launcher and must match the host registry order.
- Selecting an entry calls `start_app(id)`.

## Current app list (must match host registry)

1. Circles
2. Mandelbrot
3. Test Drawing
4. Test UI
5. Snake

## Porting expectations

- Keep IDs stable across host and launcher to avoid mismatches.
- Preserve menu layout and scroll behavior for consistent UX and tests.
