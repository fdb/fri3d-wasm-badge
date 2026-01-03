# Stage 007: IMGUI System

This stage specifies the immediate-mode UI library used by apps.

References: `docs/imgui.md`, `src/sdk/imgui.h`, `src/sdk/imgui.c`

## Core principles

- Immediate mode: widgets are created and queried each frame inside `render()`.
- No retained widget objects; app owns all state.
- Focus is the only persistent UI state managed by the library.
- WASM-friendly: no callbacks across the boundary; widgets return booleans.

## Frame lifecycle

- `ui_begin()`:
  - Clears the canvas.
  - Resets focus count and layout stack.
  - Resets menu state and deferred draw buffers.
- `ui_input(key, type)`:
  - Records last input (except releases).
  - On short/repeat: handles focus navigation (up/down) and sets `ok_pressed`/`back_pressed`.
- `ui_end()`:
  - Clamps focus to valid range; clears input flags for next frame.

## Layout model

- Stack-based layouts, max depth 8 (`UI_MAX_LAYOUT_DEPTH`).
- Default layout: vertical stack starting at (0,0) full screen width.
- Functions:
  - `ui_vstack(spacing)` and `ui_hstack(spacing)` push new layouts.
  - `ui_hstack_centered(spacing)` centers a horizontal group via deferred draws.
  - `ui_end_stack()` pops and advances parent cursor.
  - `ui_spacer(pixels)` increments cursor.
  - `ui_set_position(x, y)` sets absolute position for next widget only.

## Focus system

- Up/down changes focus index based on previous frame's focus count.
- Each focusable widget registers in render order (`UI_MAX_FOCUSABLE = 32`).
- Focus wraps around at bounds.
- A widget is activated when it is focused and `ok_pressed` is true.

## Widgets and behaviors

### Labels
- `ui_label(text, font, align)` draws text with alignment.
- Fonts: primary 12px, secondary 11px (as defined in `imgui.c`).

### Buttons
- `ui_button(text)` and `ui_button_at(x, y, text)`.
- Focused button is drawn inverted (white-on-black).
- For centered hstacks: button draw is deferred until `ui_end_stack()`.

### Progress
- `ui_progress(value, width)` draws outlined bar; width 0 uses layout width minus margins.

### Checkboxes
- Focused checkbox draws inverted row background.
- Toggled on activation.

### Icons
- `ui_icon(data, w, h)` renders XBM bits manually into the canvas.

### Separators
- `ui_separator()` draws a horizontal line.

## Menu system

- `ui_menu_begin(scroll*, visible, total)` initializes menu state.
- Each `ui_menu_item` or `ui_menu_item_value`:
  - Registers focus for all items (even if not visible).
  - Updates scroll to keep focused item visible.
  - Draws only items in visible range.
- `ui_menu_end()`:
  - Draws a dotted scrollbar track and solid thumb if needed.
  - Advances the parent layout cursor.

## Footer buttons

- Left/right footer buttons respond to left/right short/press input.
- Center footer draws an OK icon and text but does not consume OK in current implementation.

## Virtual keyboard

- Data structure: `ui_virtual_keyboard_t` holds buffer, selection, validator info.
- Layout: three rows of keys plus Enter and Backspace; origin `(1,29)`.
- Interaction:
  - Short press: move selection or type selected key.
  - Long press OK toggles case (uppercase); long press Back deletes.
  - Repeat: continuous navigation or backspace.
- Validator:
  - Optional callback validates on Enter.
  - Error message shown for 4 seconds (`UI_VK_VALIDATOR_TIMEOUT_MS`).

## Porting expectations

- Keep numeric constants consistent (menu item height, footer height, max focusable, etc.).
- Preserve focus behavior and menu scroll logic since tests assume it.
- Maintain keyboard layout and character mapping for the test UI.
