# 017 — Colour UI: 320×240, DB32, Pixelify

The badge LCD is 320×240 with 16-bit colour. The kernel draws on it at
1:1 with a 32-colour palette. This doc is the design system: the
framebuffer, the palette and its roles, the fonts, the icons, and the
pipeline that turns PNGs and BDFs into Rust tables.

The look comes from the Fri3d Foxhunt badge app: chunky pixel art, paper
background, white cards with a tan rest border, a gold border on the
focused item, a green title banner. Foxhunt used its own colours; the
badge uses the [DB32 palette](https://pixeljoint.com/forum/forum_posts.asp?TID=16247)
so every app, icon and font shares one set of 32 colours.

## Framebuffer

One byte per pixel, row-major, 76 800 bytes. The byte is an index into
`fri3d_kernel::palette::RGB`. Hosts convert at blit time: desktop and
web through a 32-entry RGB table, the badge through a 32-entry RGB565
table. No RGB ever crosses the app ABI.

`canvas_clear` fills with `PAPER`. `canvas_set_color(i)` takes a palette
index; out-of-range values become `INK`. There is no XOR mode: with 32
colours a highlight is a colour, not an inversion.

## Palette roles

| Role | Name | Index | Hex | Use |
| --- | --- | --- | --- | --- |
| paper | `PAPER` | 7 | `#eec39a` | screen background |
| card | `CARD` (= `WHITE`) | 21 | `#ffffff` | cells, panels, list focus row |
| ink | `INK` | 1 | `#222034` | text, frames |
| muted | `MUTED` (= `BROWN`) | 4 | `#8f563b` | secondary text, section labels |
| rest border | `REST_BORDER` (= `TAN`) | 6 | `#d9a066` | unfocused cell border, tracks, separators |
| focus | `FOCUS` (= `GOLD`) | 8 | `#fbf236` | focused border, progress fill |
| green | `GREEN` / `GREEN_DARK` | 10 / 12 | `#6abe30` / `#4b692f` | banner, primary button, "on" |
| terra | `TERRA` | 5 | `#df7126` | accent |
| red | `RED` | 27 | `#ac3232` | danger, hearts, fruit |
| blue | `BLUE` | 18 | `#639bff` | links, data |
| forest | `FOREST` | 14 | `#323c39` | dark screens (title, splash) |

All 32 names are in `fri3d_kernel::palette` and mirrored in
`fri3d_wasm_api::color`. `artwork/db32.gpl` is the same palette for
Aseprite.

## Layout tokens (badge px)

- Pad 8. Border 2 (draw `rframe` twice). Radius 2.
- Banner 24: `GREEN` box, 2-px `GREEN_DARK` bottom edge, title in the
  title face at baseline 18, right text in the body face, optional icon.
- Body row 14 (baseline 11). Menu row 28 (baseline 18). Footer 18.
- Home grid: 4×2 cells of 70×62, gap 8; icon at 2× (32 px), name below.
- Focus: `CARD` fill + 2-px `FOCUS` border. Buttons: `GREEN` fill with
  a `FOCUS` border when focused, `CARD` fill with an `INK` border at rest.

`fri3d_wasm_api::imgui` implements these (`ui_banner`, `ui_menu_item`,
`ui_button`, footers). Apps that draw by hand should reuse the numbers.

## Fonts

Pixelify Sans (OFL), snapped to 1-bit bitmaps in the Foxhunt repo's
font editor: `pixelify_r11.bdf` (ascent 8, descent 2) for body text and
`pixelify_b22.bdf` (ascent 16, descent 4) for titles. Both live in
`artwork/fonts/` with `CREDITS.md`.

`cargo run -p fri3d-fontgen` writes `fri3d-kernel/src/fonts.rs`: a
`BitmapFont` per file with 95 `Glyph`s (ASCII 32..=126), each its BDF
bounding box as MSB-first rows plus advance and offsets. `font.rs`
draws straight from the table; no decoder, no allocation.

Font ids: `PRIMARY`, `SECONDARY`, `KEYBOARD` → r11; `TITLE`,
`BIG_NUMBERS` → b22. Text `y` is the baseline.

## Icons

- App icons: `apps/<id>/icon.png`, 16×16. `fri3d-pack` snaps every
  opaque pixel to the nearest DB32 colour and stores 256 indices in the
  bundle header (255 = transparent). `AppInfo::draw_icon(x, y, scale)`.
- System icons: `artwork/icons/<name>.png`, up to 64×64, `[a-z0-9_]`
  names. `fri3d-pack` writes `fri3d-artwork/src/generated.rs` with one
  `pub static NAME: Image` per file. Apps depend on `fri3d-artwork` and
  call `canvas_draw_icon(x, y, scale, &art::WIFI)`.
- `canvas_draw_image(x, y, w, h, scale, ptr)` is the one import behind
  both. `scale` is clamped to 8; `w`/`h` to the screen.

Edit PNGs in Aseprite with the DB32 palette loaded; anything off-palette
is snapped, so a stray anti-aliased edge becomes a visible wrong colour.

## Why not more

- No alpha, no blending: a `draw_image` pixel is a palette index or a
  hole. Blending would need RGB in the kernel and a 2-byte framebuffer.
- No per-app palettes: 32 shared colours keep every screen in one family
  and keep the badge blit a table lookup.
- No font scaling: two faces cover banner and body. A third size is a
  third BDF, not a transform.
