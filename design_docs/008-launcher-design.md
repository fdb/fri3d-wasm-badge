# 008 — Launcher design

The launcher is an ordinary app (`apps/launcher`) with `system = true`.
The kernel keeps it resident and pauses/resumes it around other apps.

## Look: the home grid (design doc 017)

```
┌──────────────────────────────────────┐
│ Fri3d                            (((●│  green banner 24 px, Wi-Fi mark
│ APPS                             1/2 │  section label + page, muted
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ │
│ │ [2×] │ │ [2×] │ │ [2×] │ │ [2×] │ │  4×2 cells of 70×62, gap 8
│ │ Name │ │ Name │ │ Name │ │ Name │ │  white card, tan border;
│ └──────┘ └──────┘ └──────┘ └──────┘ │  the focused cell has a gold border
│ …second row…                         │
│ ■ Menu                        Open ■ │  footer 18 px
└──────────────────────────────────────┘
```

- The 16×16 icon is drawn at 2× and centred; the name sits on the cell's
  last text row.
- Left/Right step through apps and wrap; Up/Down move by a row. Holding
  a direction repeats (the kernel synthesises repeats).
- Eight apps per page; the page follows the selection.
- OK starts the app. Long OK opens the info page: icon card, name in the
  title face, author and id, description word-wrapped in a panel,
  `■ Back   Open ■`.
- Back does nothing on the home screen; Menu is handled by the kernel.

## Data flow

`on_start` / `on_resume` call `app_count()`. Each render fetches the
visible headers (eight) with `app_info(i, buf, 512)` — eight 512-byte
copies per frame, no cache, no allocation. The selected index survives
an app run, so returning lands on the app you came from.

## What it does not do

- No timers, no animation, no splash. Idle cost is zero.
- No per-app state in the launcher: the kernel's registry is the truth.
- No categories or folders yet. The pack tool orders apps (users first,
  system apps last); the launcher shows that order.

## Settings app

`apps/settings`, also `system = true`. A banner, then an `imgui` menu:
Wi-Fi, Brightness (10–100 %, Left/Right in steps of 10), Sound (toggle)
and About (kernel ABI version, app count). Values live in the `system`
namespace; hosts read `kernel.setting("system", "brightness")` after
each step and apply it (LCD backlight via the CH32 on the badge, dimmed
pixels on desktop/web).
