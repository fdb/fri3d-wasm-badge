# 003 — Fri3d 2026 badge hardware

Sources: github.com/Fri3dCamp/badge_2026_hw (schematics),
MicroPythonOS `lib/mpos/board/fri3d_2026.py` and
`drivers/fri3d/expander.py` (the de-facto pinout reference),
github.com/Fri3dCamp/badge_firmware_MicroPythonOS (image layout).

## Facts the firmware depends on

| Item | Detail |
| --- | --- |
| MCU | ESP32-S3-WROOM-1-N16R8: 16 MB flash, 8 MB octal PSRAM, 2× Xtensa LX7 @ 240 MHz, native USB |
| Display | 2" IPS 240×320, **ST7789V**, SPI2: MOSI 6, MISO 8, SCK 7, DC 4, CS 5, 40 MHz. Landscape = rotation 270 → 320×240. RGB565, colour inversion ON. MPOS sets BGR plus an RGB565 byte swap; with mipidsi the working combination is `ColorOrder::Rgb` (verified on hardware: Bgr gave a blue tint). Reset + backlight via the CH32 coprocessor, **not** a GPIO |
| Coprocessor | CH32X035 on I²C1 (SDA 39, SCL 42), addr 0x50 |
| Buttons | All through the CH32 except START (GPIO 0, active low) |
| Touch | CST816S, I²C0 (SDA 9, SCL 18), addr 0x15, INT GPIO 13 |
| IMU | LSM6DSO, I²C0, addr 0x6A |
| LEDs | 5× WS2812 on GPIO 12 |
| Buzzer | GPIO 38 |
| microSD | Shared SPI2, CS 14 |
| Power | 2000 mAh LiPo, TP4056 over USB-C |

## CH32 expander register map

| Reg | Name | Format |
| --- | --- | --- |
| 0x04 | inputs | `u16` LE. Bit 11 usb_plugged, 10 joy_right, 9 joy_left, 8 joy_down, 7 joy_up, 6 menu, 5 B, 4 A, 3 Y, 2 X, 1 charger_standby, 0 charger_charging. 1 = pressed |
| 0x08 | analog | 5× `u16` LE: ain0, battery, usb, joy_y, joy_x |
| 0x12 | lcd_brightness | `u16` LE, 0–100 |
| 0x14 | debug_led | `u16` LE, 0–100 |
| 0x16 | config | `u8`, bits: 4 lora_reset, 3 remap, 2 reboot, 1 lcd_reset, 0 aux_power. MPOS writes `0x01`, waits 100 ms, then `0x13` to reset LCD + LoRa and power aux 3V3 |

Note: the MPOS driver reads bits 11..0 as a tuple where tuple index *i*
is bit *11 − i*. The table above already resolves that.

## Key mapping (kernel `InputKey`)

| Badge | Kernel |
| --- | --- |
| joystick up/down/left/right | Up / Down / Left / Right |
| A | Ok |
| X | Back (MPOS uses X as ESC; B is "next") |
| MENU | Menu (kernel home key) |
| B, Y, START | unused for now |

## Lessons

- **Reset pins are not GPIOs.** The LCD reset is a CH32 config bit. A
  display driver that wants a reset pin gets a dummy; the firmware
  toggles the CH32 bit before `init`.
- **The CH32 is not ready when the ESP32 is.** On a cold power-on it
  ignores I²C for up to ~1 s (MPOS waits a minimum uptime of 1000 ms).
  A fire-and-forget config write leaves the LCD in reset: the firmware
  runs, SPI writes "succeed", the panel stays black. Verified on
  hardware 2026-08-21. Write, read back, retry — `set_config_verified`
  in `hosts/badge/src/main.rs`.
- **Brightness is an I²C write**, so a settings change must cross from
  the kernel (settings table) to the host (I²C). The kernel exposes the
  value; the host applies it after each `step`.
- **PSRAM is octal** (`qio_opi`); esp-hal needs the `psram` feature and
  the octal mode config.
- The MPOS image puts MicroPythonOS in two 3.5 MB OTA slots plus retro-go
  partitions. A WASM firmware can use the same 16 MB partition table or
  its own; both are flashed with esptool.
