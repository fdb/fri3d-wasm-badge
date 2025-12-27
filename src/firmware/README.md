# Fri3d Badge Firmware

ESP32-S3 firmware for the Fri3d Badge.

## Hardware

- **MCU:** ESP32-S3-WROOM-1
- **CPU:** 240 MHz
- **Flash:** 16 MB
- **PSRAM:** OPI PSRAM

### Display (SPI)

| Signal | GPIO |
|--------|------|
| MOSI   | 6    |
| MISO   | 8    |
| SCK    | 7    |
| CS     | 5    |
| DC     | 4    |
| RST    | 48   |

### Buttons (GPIO, active low with pull-up)

| Button | GPIO |
|--------|------|
| Up     | 9    |
| Down   | 10   |
| Left   | 11   |
| Right  | 12   |
| Ok     | 13   |
| Back   | 14   |

**Note:** Button GPIO assignments are placeholders. Update `input_gpio.cpp` when hardware design is finalized.

## Building

```bash
cd src/firmware
pio run
```

## Uploading

```bash
pio run --target upload
```

## Serial Monitor

```bash
pio device monitor
```

## TODO

- [ ] Integrate WAMR runtime
- [ ] Integrate shared runtime components (canvas, input manager, app manager)
- [ ] Implement SD card storage for WASM apps
- [ ] Implement SPIFFS fallback for app storage
- [ ] Test with actual hardware
