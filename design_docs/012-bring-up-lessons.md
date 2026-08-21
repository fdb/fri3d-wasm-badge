# 012 — Hardware bring-up lessons (2026-08-21)

Short, chronological, so the next bring-up does not repeat them.

1. **First flash "hangs at waiting for download".** Not a hang: the flash
   succeeded and espflash's post-flash reset re-entered the ROM
   bootloader, because on the S3's native USB the DTR/RTS pattern *is*
   the boot strap. Power-cycle without holding START.

2. **No boot log.** USB-JTAG output printed before the host attaches is
   dropped. Add a short delay before the first `println!` and a periodic
   heartbeat; read the port *before* resetting.

3. **Everything reports OK, screen black.** Display init, fills and the
   backlight register all returned success, yet the panel was dark after
   the first USB replug. Every later flash (chip reset with the CH32
   already powered) showed the image. In hindsight this was lesson 6
   already: the replug was a cold boot for the CH32. A "success" from an
   I²C write to a coprocessor says nothing about whether it acted.

4. **Blue tint.** `ColorOrder::Bgr` (copied from MPOS's driver flags)
   swaps R and B under mipidsi. Use `Rgb`.

5. **Canvas smaller than the panel.** 128×64 at 2× left a border; the
   user chose 160×120 at 2× to fill 320×240. Every hardcoded size in apps
   and the IMGUI had to move to constants. Do that on day one.

6. **Black screen on cold battery boot only.** The decisive symptom:
   USB-powered boots worked, battery boots did not. The CH32 that holds
   the LCD reset line boots ~1 s after the ESP32; MPOS waits for it. Our
   fire-and-forget config write was lost. Fix: write, read back, retry
   (`set_config_verified`).

7. **The power switch does not cut power while USB is attached**, so a
   cold boot cannot be captured over serial. Test cold boots on battery;
   read the log afterwards (the heartbeat shows uptime).

8. **espflash's partition parser** rejects numeric subtypes (`0x40`);
   use a named subtype (`undefined`) for custom data partitions.

9. **rust-lld cannot link esp-hal for Xtensa.** The `espup` GCC is the
   linker; nothing is compiled with it.

## Bring-up checklist for the next board

- Read the port before the first reset; print a heartbeat.
- Verify every coprocessor write by reading it back, with a retry loop.
- Fill the panel red, then green, before drawing anything real.
- Test a cold boot on battery before calling the display done.
- Put the screen size behind constants before writing the second app.

## Wi-Fi and IP bring-up (2026-08-21)

Both failures froze the badge until the panic-reboot handler existed;
neither can be reproduced on the desktop host.

- **esp-alloc is first-fit in registration order.** With the internal
  heap registered before PSRAM, the kernel and wasmi consumed it at boot
  and `esp-rtos` could not allocate the Wi-Fi task stack from internal
  RAM. Register PSRAM first; keep the internal region for capability-
  tagged allocations (radio stacks, DMA buffers).
- **Polling Embassy futures without an executor** needs
  `embassy-time-queue-utils/generic-queue-*`; the default timer queue
  asserts the waker belongs to an Embassy task.
- **Panics must reboot.** `esp-backtrace`'s `custom-halt` feature plus
  a 3 s delay and `software_reset()`.
