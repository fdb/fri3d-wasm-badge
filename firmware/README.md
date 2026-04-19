# Fri3d WASM Badge — Hardware Firmware

First-light firmware for running this project on the real
[Fri3d Camp 2024 badge](https://github.com/Fri3dCamp/badge_2024_hw)
(ESP32-S3, 240×296 IPS LCD with GC9307 driver).

## What it does today

- Boots the LCD in landscape orientation (296×240) using [TFT_eSPI](https://github.com/Bodmer/TFT_eSPI).
- Keeps a **128×64 monochrome framebuffer** in RAM — identical semantics to
  [`fri3d-runtime/src/canvas.rs`](../fri3d-runtime/src/canvas.rs).
- Runs a native C++ port of [`fri3d-app-circles`](../fri3d-app-circles) as the
  render function: 10 animated circles.
- 2× nearest-neighbour upscales the framebuffer to 256×128 and pushes it to
  the centre of the LCD via a single SPI `pushImage` burst (80 MHz, PSRAM buffer).
- Reads the joystick + buttons via the Fri3d BSP's `Fri3d_Button` class.
  - **A**: reseed the circles (matches the WASM app's `KEY_OK` reshuffle).
  - **B**: toggle animation on/off.

## What it does _not_ do yet

- Does **not** load `.wasm` apps. The WASM apps in this repo currently run on
  the desktop/web emulators only. The next iteration plans to drop a WASM
  interpreter ([wasm3](https://github.com/wasm3/wasm3)) in and wire the host
  imports to the `Canvas` class here, so all the existing apps can run unchanged.
- No custom fonts yet (the emulator's bitmap font from
  [`fri3d-runtime/src/fonts.rs`](../fri3d-runtime/src/fonts.rs) isn't ported).
  The TFT_eSPI built-in font is used only for the chrome around the canvas.
- No LEDs, buzzer, IMU, SD card, battery readout, or Wi-Fi — those are on the
  backlog once the core render loop is proven on real hardware.

## Build & flash

Install [PlatformIO](https://platformio.org/install) (the `pio` CLI), then:

```bash
cd firmware
pio run                 # build
pio run -t upload       # flash over USB-C
pio device monitor      # serial logs (115200 baud)
```

The ESP32-S3 has **native USB**, so it should enumerate as a USB CDC device as
soon as you plug the badge in. If the first upload fails:

1. Hold the `START` (GPIO0) button.
2. Tap the reset button (if populated) — or unplug and re-plug USB-C with `START` held.
3. Release `START` once the upload starts.

Once the bootloader is reflashed once, subsequent uploads over USB CDC should
work without holding any buttons.

## Files

| Path | Purpose |
| --- | --- |
| [`platformio.ini`](platformio.ini) | Build config — board, partitions, TFT_eSPI pins, 80 MHz SPI. |
| [`src/main.cpp`](src/main.cpp) | Boot, main loop, blit pipeline, circles demo. |
| [`src/canvas.h`](src/canvas.h), [`src/canvas.cpp`](src/canvas.cpp) | C++ port of the emulator's `Canvas` drawing primitives. |
| [`src/Fri3dBadge_pins.h`](src/Fri3dBadge_pins.h) | GPIO map, vendored verbatim from [`Fri3dCamp/badge_2024_arduino`](https://github.com/Fri3dCamp/badge_2024_arduino). |
| [`src/Fri3dBadge_Button.h`](src/Fri3dBadge_Button.h), [`src/Fri3dBadge_Button.cpp`](src/Fri3dBadge_Button.cpp) | Debounced button/joystick helper, vendored from the same upstream. |

## Licensing notes

`Fri3dBadge_Button.{h,cpp}` descends from
[Jack Christensen's JC_Button](https://github.com/JChristensen/JC_Button)
(GPLv3). The vendored copies carry their original headers.
