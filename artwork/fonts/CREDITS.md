# Font credits

## Pixelify Sans → `pixelify_r11.bdf`, `pixelify_b22.bdf`
- **Designer:** Stefie Justprince
- **License:** SIL Open Font License 1.1 (OFL)
- **Source:** https://fonts.google.com/specimen/Pixelify+Sans
- **Upstream repo:** https://github.com/eifetx/Pixelify-Sans

Both files are 1-bit bitmaps derived from Pixelify Sans by snapping its
outlines to the font's native pixel grid (regular at 11 px/em, bold at
22 px/em). The OFL permits this modification and redistribution; the
bitmaps remain under the OFL. Keep this credit with any badge build.

`cargo run -p fri3d-fontgen` turns these BDFs into
`fri3d-kernel/src/fonts.rs`. Edit the BDF (the editor in the foxhunt
repo, `tools/bitmap_fonts`, opens them), then regenerate.
