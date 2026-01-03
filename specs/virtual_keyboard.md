# Virtual Keyboard

## Scope

Implement a virtual keyboard in an IMGUI-based UI.

Below is a reference of how Flipper Zero’s virtual keyboard (text input view) works, followed by a plan to implement similar behavior in an IMGUI context. Note that Flipper Zero uses a retained-mode GUI, while IMGUI is immediate-mode, so state management differs.

## Source of Truth (Flipper Zero)

- `tmp_flipper/text_input.c`
- `tmp_flipper/text_input.h`

### Data model & API

- `TextInput` is a GUI view module with a model storing header, text buffer pointer/size, min length, selection (row/col), validator callback/message, and a “clear_default_text” flag.
- The module does not own the text buffer; it edits a caller-provided `char*` with a size limit.

### Layout

- Keyboard is 3 rows with fixed pixel positions (origin at x=1, y=29) and a `FontKeyboard` glyph set.
- Keys (row order):
  - Row 1: `q w e r t y u i o p 0 1 2 3`
  - Row 2: `a s d f g h j k l [Backspace] 4 5 6`
  - Row 3: `z x c v b n m _ [Enter/Save] 7 8 9`
- Backspace and Save are rendered as icons (`I_KeyBackspace*`, `I_KeySave*`), not glyphs.

### Rendering

- Header line at top, then a rounded input frame.
- If the text overflows the frame, it trims from the left and shows an ellipsis.
- Caret behavior:
  - If `clear_default_text` is false, draw a `||` caret at the end of the text.
  - If `clear_default_text` is true, draw an inverted box behind the text, signaling “overwrite on first key.”
- Selected key is highlighted with a filled box; glyph color is inverted.

### Navigation & input

- D‑pad (Up/Down/Left/Right) moves selection with wrap on row ends.
- Because row lengths differ (14/13/12), moving between rows nudges the column to keep a sensible alignment.
- OK inserts the selected key; long‑OK acts as a temporary “shift” modifier for that keypress.
- Back key (long or repeat) triggers backspace.

### Character rules

- Auto‑uppercase when text is empty or `clear_default_text` is true.
- Long‑OK toggles that case rule for the current keypress.
- `_` becomes a space when uppercase mode is applied.

### Enter / validation

- Selecting Enter runs an optional validator callback.
  - On failure, a warning overlay appears for ~4 seconds; any subsequent non‑press input clears it sooner.
- If validation passes and text length ≥ `minimum_length`, the result callback fires.

## IMGUI Implementation Plan

### State (owned by your app)

Maintain a state struct persisted between frames:

- `char* buffer`, `size_t capacity`, `size_t min_len`
- `uint8_t row`, `uint8_t col`
- `bool clear_default_text`
- `bool validator_visible`, `float validator_deadline_ms`, `string validator_msg`
- Optional: `bool shift_modifier` (for long‑OK or separate key)

### Key model

Define a static array of key descriptors (char or special type + rect/position). You can:

- Mirror Flipper’s pixel positions, or
- Use a grid layout and compute positions from cell size.

### Input handling (per frame)

- Translate physical buttons to logical events (short/long/repeat).
- Apply the same row/column navigation rules or use nearest‑key by X position when changing rows.
- On OK:
  - Resolve the selected key to a character or action.
  - Apply case rules (auto‑uppercase, long‑OK shift inversion, `_`→space when uppercasing).
  - Append if there is capacity; else ignore.
- On Back:
  - Remove the last byte; if `clear_default_text` is set, clear the buffer and unset the flag.
- On Enter:
  - Run validator; if fail, show overlay and set a timeout.
  - If pass and length ≥ min, fire your “save” callback.

### Rendering (IMGUI)

- Use `FontKeyboard` for key glyphs.
- Draw header, input frame, clipped text, and caret/overwrite highlight.
- Draw keys with a selected highlight.
- Render special keys as icons or labeled boxes (“OK”, “DEL”).
- If `validator_visible` and time < deadline, draw a modal overlay.

### Notes for IMGUI vs retained-mode

- Flipper stores state in a view model and reacts to input callbacks; in IMGUI, you must persist the state externally and update it each frame.
- Time‑based validator dismissal should be driven by your GUI tick time (`now_ms`).
- Keep the data model pure (no rendering dependencies) so you can unit‑test navigation and input handling separately.
- Adapt IMGUI API to allow easy integration of the virtual keyboard into other apps. Then make an example scene in the test_ui app that uses this virtual keyboard to demonstrate its usage.
