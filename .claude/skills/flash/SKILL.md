---
name: flash
description: Build the latest Fri3d badge firmware (Rust, esp-hal) including any updated WASM apps and flash it to the ESP32-S3 badge over USB-C. Use when the user says "flash", "flash the badge", "deploy to hardware", or similar.
---

# Flash — Build & Upload to Badge

Packs the apps, builds `hosts/badge` with the Espressif Rust toolchain,
and flashes it with `espflash`. Reports image size and the boot log.

## Steps

### 1. Sanity-check the environment

```bash
test -f hosts/badge/Cargo.toml || echo "not in the Fri3d repo root"
ls /dev/cu.usbmodem* 2>&1
command -v espflash && rustup toolchain list | grep esp
```

- No `/dev/cu.usbmodem*`: ask the user to plug in the badge.
- No `espflash`: `cargo install espflash`.
- No `esp` toolchain: `cargo install espup && espup install`.

### 2. Verify on desktop first

Any app or kernel change must pass the cheap loops before a flash:

```bash
cargo test -q -p fri3d-kernel
cargo run -q -p fri3d-pack
cargo run -q --release -p fri3d-host-desktop -- --headless --screenshot /tmp/launcher.png
```

Look at the PNG. Only flash when it is right.

### 3. Build + flash

```bash
hosts/badge/flash.sh
```

It packs, builds (`cd hosts/badge && cargo build --release`), and runs
`espflash flash --monitor --flash-size 16mb`. Watch for the monitor's
`[fri3d] boot` line, then `kernel up: N apps`. Press Ctrl-C to leave the
monitor.

Report the image size from espflash's output (the app is ~1.1 MB today,
~7 % of the 16 MB flash).

### 4. Failure modes

- **"Failed to connect"** / device not enumerating: hold START (GPIO 0)
  while replugging USB-C to force the ROM bootloader, then retry.
- **Panic on boot about PSRAM**: octal PSRAM init failed; see
  `hosts/badge/README.md` (try `PsramMode::Auto`).
- After flashing, espflash's reset can land in download mode
  (`boot:0x1 ... waiting for download`): unplug and replug USB-C without
  holding any button. Do not toggle DTR/RTS from a script — on USB-JTAG
  that re-enters the bootloader.
- Verified on hardware 2026-08-21: `Rotation::Deg270`, `ColorOrder::Rgb`,
  inversion on, CH32 input bit map as in design_docs/003.

## Notes

- The previous C++/PlatformIO firmware under `firmware/` is a reference
  only. Do not `pio run` for the 2026 badge.
- Flashing replaces the MicroPythonOS partition layout; restoring MPOS
  means reflashing its image.
