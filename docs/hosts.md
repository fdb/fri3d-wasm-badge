# Running the badge

One kernel, three hosts. Everything below assumes the apps are packed:

```bash
cargo run -q -p fri3d-pack          # apps/* → build/apps/*.fab → fri3d-apps
```

## Desktop

```bash
cargo run --release -p fri3d-host-desktop
```

| Key | Badge |
| --- | --- |
| Arrows / WASD | joystick |
| Z / Enter | A (OK) |
| X / Backspace | X (Back) |
| M / Esc | MENU (home) |
| F12 | screenshot → `screenshot_N.png` |

Settings persist in `~/.fri3d-badge/settings.bin`. Brightness tints the
amber background.

Headless, for scripts and CI:

```bash
fri3d --headless [--app NAME] [--scene N] [--keys k1,k2,…] [--frames N] \
      [--screenshot out.png] [--seed S] [--apps-dir DIR] [--list]
```

`--apps-dir` loads `.fab` files from a directory instead of the embedded
set (the one named `launcher.fab` becomes the launcher).

## Browser

```bash
hosts/web/build.sh                       # needs wasm-pack
cd hosts/web/dist && python3 -m http.server 8091
```

`/` is the interactive harness (same keys as desktop). `/test.html` runs
`tests.js` and sets `window.testResults`. `window.fri3d` exposes
`tap(key, heldMs)`, `render()`, `readFb()`, `startApp(i)`,
`exitToLauncher()`, `appCount()`, `appName(i)`, `rngSeed(s)`,
`rngGet()`, `KEY`. Settings persist in `localStorage`.

## Badge (Fri3d 2026)

One-time: `cargo install espup espflash && espup install`.

```bash
hosts/badge/flash.sh            # pack, build, flash (OTA layout), open monitor
```

Or by hand from `hosts/badge/`: `cargo build --release`, then
`espflash flash --flash-size 16mb --partition-table partitions.csv
--erase-parts otadata target/xtensa-esp32s3-none-elf/release/fri3d-badge`.

| Badge | Kernel key |
| --- | --- |
| joystick | Up / Down / Left / Right |
| A | OK |
| X | Back |
| MENU | Menu (home) |
| B, Y, START | unused |

Serial log over USB (no reset — resets re-enter the bootloader):

```bash
uv run --with pyserial python -c "
import serial,glob; s=serial.Serial(glob.glob('/dev/cu.usbmodem*')[0],115200,timeout=1)
[print(l.decode(errors='replace'),end='') for l in iter(s.readline,b'')]"
```

Expect `[fri3d] kernel up, N apps`, `ota slot Ok(Ota0) state Ok(Valid)`,
then `[fri3d] alive t=…` every 5 s.

Gotchas, all learned the hard way — see
[design_docs/012](../design_docs/012-bring-up-lessons.md):

- After a flash the badge may sit in `waiting for download`: replug
  USB-C without holding any button.
- The power switch does nothing while USB is attached.
- A cold boot on battery is the real test of display bring-up.
- `espflash --monitor` resets the chip; prefer the raw serial reader.
