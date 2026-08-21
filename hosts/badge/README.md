# Fri3d 2026 badge firmware (Rust, esp-hal)

The kernel (`fri3d-kernel`) plus the embedded app bundles (`fri3d-apps`)
on the ESP32-S3 of the Fri3d Camp 2026 badge. Bare metal: no ESP-IDF, no
RTOS. One loop: poll the CH32 expander for buttons, `kernel.step()`, blit
the 160×120 canvas 2× onto the full 320×240 LCD when it changed.

This crate is its own Cargo workspace. It targets
`xtensa-esp32s3-none-elf` with the `esp` toolchain and depends on the repo
crates by path. Hardware facts and the pinout source are in
[design_docs/003-badge-2026-hardware.md](../../design_docs/003-badge-2026-hardware.md).

## One-time setup

```bash
# Xtensa Rust toolchain + Xtensa GCC (installs the `esp` rustup channel).
cargo install espup
espup install

# Flasher / monitor (native USB, no driver needed on macOS).
cargo install espflash
```

`.cargo/config.toml` pins the linker to the GCC that `espup` installs
(`~/.rustup/toolchains/esp/xtensa-esp-elf/.../xtensa-esp32s3-elf-gcc`).
If your copy lives elsewhere, put `xtensa-esp32s3-elf-gcc` on `PATH` and
remove the `linker =` line.

## Build and flash

```bash
# From the repo root: rebuild and pack the apps (writes fri3d-apps/src/generated.rs).
cargo run -q -p fri3d-pack

# From hosts/badge: build + flash + open the serial monitor.
cd hosts/badge
cargo build --release
espflash flash --monitor --flash-size 16mb target/xtensa-esp32s3-none-elf/release/fri3d-badge
```

Or in one step: `hosts/badge/flash.sh`.

`cargo run --release` does the same through the `runner` in
`.cargo/config.toml`.

Flash settings: ESP32-S3 N16R8, 16 MB flash (`--flash-size 16mb`), octal
PSRAM (configured at runtime in `main.rs` via `PsramMode::OctalSpi`).
espflash writes its own bootloader and a single-app partition table; the
MicroPythonOS partition layout is not preserved. To go back to MPOS,
reflash its image.

Build an image without a badge attached:

```bash
espflash save-image --chip esp32s3 --flash-size 16mb --merge \
    target/xtensa-esp32s3-none-elf/release/fri3d-badge fri3d-badge.bin
```

## Recovery

If `espflash` cannot connect ("Failed to connect", port not enumerating,
or the board is stuck in a panic loop): hold **START** (GPIO0) while
plugging in USB-C to force the ROM bootloader, then flash again.

Logs go over the native USB-CDC/JTAG port (`esp-println` `jtag-serial`).
Nothing blocks on the host reading them, so a standalone badge boots fine.

## What the loop does

| Step | Detail |
| --- | --- |
| Expander | I²C1 (SDA 39 / SCL 42) @ 400 kHz, addr `0x50`. Boot: config `0x01` → 100 ms → `0x13` (LCD + LoRa out of reset, aux 3V3 on). Poll register `0x04` every 5 ms. Brightness register `0x12` follows the kernel's `system.brightness` setting. |
| Display | ST7789V on SPI2 (SCK 7 / MOSI 6 / MISO 8, DC 4, CS 5) @ 40 MHz, landscape via `Rotation::Deg270`, BGR, inverted. Chrome drawn once; canvas rows streamed 2 panel rows at a time through a 2 KB static buffer. Blit skipped when the framebuffer is byte-identical. |
| Memory | 96 KB internal heap + all of PSRAM via `esp-alloc`. The `Kernel` is boxed (PSRAM); wasmi linear memories land there too. |
| Stack | Bare metal: `main` runs on the full remaining DRAM stack, no 8 KB task limit. |

## Keys

| Badge | Kernel |
| --- | --- |
| joystick | Up / Down / Left / Right |
| A | Ok |
| X | Back |
| MENU | Menu (home) |
| B, Y, START | unused |
