---
name: flash
description: Build the latest Fri3d badge firmware (including any updated WASM apps) and flash it to the physical ESP32-S3 device connected over USB-C. Use when the user says "flash", "flash the badge", "deploy to hardware", or similar.
---

# Flash — Build & Upload to Badge

Rebuilds the embedded WASM apps (if the Rust sources changed), compiles the
PlatformIO firmware, and uploads it to the Fri3d 2024 badge connected via USB-C.
Reports size stats and verifies the upload succeeded.

## When to use

- User asks to flash, upload, deploy, or reflash the badge.
- After any change to files under `firmware/` or `fri3d-app-*/` that the user
  wants to see running on hardware.
- Only for projects that have `firmware/platformio.ini` — this skill is
  Fri3d-specific.

## Steps

### 1. Sanity-check the environment

Run these in one message (parallel where independent):

```bash
# Working directory check
test -f firmware/platformio.ini || echo "not in a Fri3d repo"

# USB device present?
ls /dev/cu.usbmodem* 2>&1

# Required tools available
command -v pio && command -v cargo && command -v wasm-opt
```

If `/dev/cu.usbmodem*` is empty, ask the user to plug in the USB-C cable.

If `pio` is missing, install via `uv tool install platformio`.

If `wasm-opt` is missing, `brew install binaryen`.

### 2. Rebuild embedded WASM apps

If any file under `fri3d-app-*/src/` or `fri3d-wasm-api/src/` changed since
the last embed, regenerate `firmware/src/embedded_apps.h`:

```bash
firmware/scripts/embed_apps.sh
```

This rebuilds every Rust app in release mode, wasm-opt shrinks them, and
writes the combined embed header. Safe to run every time — it's idempotent
and fast (~1 s if nothing changed).

### 3. Build the firmware

```bash
cd firmware && pio run
```

Read the RAM + Flash usage lines from the tail of the output. Example:

```
RAM:   [=         ]   9.5% (used 31060 bytes from 327680 bytes)
Flash: [=         ]   6.2% (used 404861 bytes from 6553600 bytes)
```

Report these to the user. If either is > 80%, flag it; the badge has plenty
of headroom today but compiled WASM apps scale with count.

### 4. Upload to the badge

Detect the port (usually `/dev/cu.usbmodem1101` on macOS) and flash:

```bash
pio run -t upload --upload-port /dev/cu.usbmodem1101
```

Watch for `Hash of data verified.` in the output — that confirms a correct
flash. The badge resets itself via RTS after upload.

### 5. Handle common failure modes

- **"Failed to connect to ESP32-S3: No serial data received"** — the previous
  firmware is in a panic-reboot loop and the USB-CDC is re-enumerating too
  fast for esptool to latch. Ask the user to *hold the START button while
  unplugging and replugging USB-C* (or tapping reset). This forces the ROM
  bootloader to stay in download mode. Then retry the upload.

- **"Device not configured" (ioctl error on DTR)** — same root cause as above.
  Same fix.

- **`/dev/cu.usbmodem1101` missing after reset** — the USB-CDC endpoint is
  wedged from too many crashes. A full physical unplug + replug of USB-C
  (without holding any button) resets the host-side USB state and is usually
  enough if the firmware is fine.

### 6. Optional: verify boot log

If the user asked to "flash and check" or similar, tail serial briefly after
the reset:

```bash
sleep 2 && uv run --with pyserial python3 -c "
import serial, time
s = serial.Serial('/dev/cu.usbmodem1101', 115200, timeout=0.3)
t_end = time.time() + 5
while time.time() < t_end:
    line = s.readline()
    if line: print(line.decode(errors='replace').rstrip())
"
```

Expect `[fri3d] boot` → `[wasm] ...` init lines → `[fri3d] wasm ready`. Silence
is OK — firmware is running fine, just nothing interesting to log. Panics
print a `Guru Meditation` dump.

## Notes

- **Never use `pio run -t uploadfs`** unless the user explicitly asks — that
  flashes the filesystem partition, not the code.
- **Don't force-flash** if something seems wrong. The badge is the user's
  physical hardware; a failed upload that needs the hold-START dance is
  their problem to disentangle, and cycling flash attempts just makes the
  USB-CDC wedge worse.
- **Do not alter `platformio.ini`** build flags in this skill — if the flash
  fails for build reasons, fix the underlying issue rather than working
  around it with flags.
- This skill does not commit or push. Pair with `/ship` if needed.
