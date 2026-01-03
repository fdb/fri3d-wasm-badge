# Stage 004: Canvas and Display Layer

This stage defines the drawing API exposed to apps and how the host renders pixels.

## Display interface

Reference: `src/runtime/display.h`

- `display_api_t` provides platform-specific hooks:
  - `init(display, headless)`
  - `get_u8g2(display)`
  - `flush(display)`
  - `should_quit(display)`
- Screen dimensions: `FRI3D_SCREEN_WIDTH = 128`, `FRI3D_SCREEN_HEIGHT = 64`.

## Canvas API (host side)

References: `src/runtime/canvas.h`, `src/runtime/canvas.c`

- Canvas wraps a `u8g2_t` framebuffer and tracks current color.
- Screen size is fixed; `canvas_width()` and `canvas_height()` return constants.

### Colors

Enum `Color` (matches SDK values):
- `ColorWhite = 0`
- `ColorBlack = 1`
- `ColorXOR = 2`

`ColorXOR` toggles pixels; `canvas_draw_circle()` and `canvas_draw_disc()` have XOR-safe custom algorithms to match u8g2 behavior for XOR draws.

### Fonts

Enum `Font` (matches SDK values):
- `FontPrimary` -> `u8g2_font_helvB08_tr`
- `FontSecondary` -> `u8g2_font_haxrcorp4089_tr`
- `FontKeyboard` -> `u8g2_font_profont11_mr`
- `FontBigNumbers` -> `u8g2_font_profont22_tn`

`canvas_set_font()` sets `u8g2_SetFontMode(..., 1)` and selects the font.

### Drawing primitives

- Pixel: `canvas_draw_dot(x, y)`
- Line: `canvas_draw_line(x1, y1, x2, y2)`
- Rectangles:
  - `canvas_draw_frame(x, y, w, h)`
  - `canvas_draw_box(x, y, w, h)`
  - `canvas_draw_rframe(x, y, w, h, radius)`
  - `canvas_draw_rbox(x, y, w, h, radius)`
- Circles:
  - `canvas_draw_circle(x, y, r)`
  - `canvas_draw_disc(x, y, r)`
- Text:
  - `canvas_draw_str(x, y, text)`
  - `canvas_string_width(text)`

Notes:
- `canvas_draw_str` uses UTF-8 width from u8g2 (`u8g2_DrawUTF8` / `u8g2_GetUTF8Width`).
- Many apps assume y passed to `canvas_draw_str` is the baseline (u8g2 behavior).

## Canvas API (WASM SDK)

Reference: `src/sdk/canvas.h`

- The WASM SDK mirrors the host API exactly, with imports declared using `WASM_IMPORT` from module `env`.
- Ensure the same enum numeric values for colors, fonts, and alignment.

## Porting expectations

- Maintain visual parity with u8g2 output for fonts and primitives.
- Preserve XOR behavior for circles/discs; the reference algorithm is in `src/runtime/canvas.c`.
- Preserve text baseline behavior and string width calculations for IMGUI layout.
