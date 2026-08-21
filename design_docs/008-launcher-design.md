# 008 — Launcher design

The launcher is an ordinary app (`apps/launcher`) with `system = true`.
The kernel keeps it resident and pauses/resumes it around other apps.

## Look: Flipper Zero on 160×120

```
┌──────────────────────────────┐
│ Fri3d                    2/8 │  status bar, secondary font, 1 px rule
├──────────────────────────────┤
│ [icon] Circles              ▓│  6 rows × 16 px (rows = (120-13)/16)
│▐[icon] Dots                ▌░│  focused row: inverted rounded box
│ [icon] Mandelbrot           ░│  dotted scrollbar, solid thumb
└──────────────────────────────┘
```

- Icon 14×14 at x = 3, name in the bold primary font at x = 22.
- The focused row is `draw_rbox(0, y, 123, 16, r = 3)` in black, then
  icon and text in white. That is how Flipper's submenu shows selection
  and it stays legible when upscaled 2× on the colour LCD.
- Up/Down wrap. Holding Up/Down repeats (the kernel synthesises repeats).
- OK starts the app. Right (or long OK) opens an info page: name,
  version, author, id, description word-wrapped, `< Back   Open >`.
- Back does nothing on the home screen; Menu is handled by the kernel.

## Data flow

`on_start` / `on_resume` call `app_count()`. Each render fetches the
visible headers (six) with `app_info(i, buf, 256)` — three 256-byte
copies per frame, no cache, no allocation. The selected index survives
an app run, so returning lands on the app you came from.

## What it does not do

- No timers, no animation, no splash. Idle cost is zero.
- No per-app state in the launcher: the kernel's registry is the truth.
- No categories or folders yet. The pack tool orders apps (users first,
  system apps last); the launcher shows that order.

## Settings app

`apps/settings`, also `system = true`. Brightness (10–100 %, Left/Right
in steps of 10), Sound (toggle) and About (kernel ABI version, app
count). Values live in the `system` namespace; hosts read
`kernel.setting("system", "brightness")` after each step and apply it
(LCD backlight via the CH32 on the badge, amber tint on desktop/web).
