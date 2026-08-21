# 011 — Storage and OTA

## Partition layout

`hosts/badge/partitions.csv`, 16 MB flash:

| Name | Offset | Size | Purpose |
| --- | --- | --- | --- |
| otadata | 0x9000 | 8 KB | active slot + image state |
| nvs | 0xb000 | 20 KB | unused; kept for MPOS compatibility |
| ota_0 | 0x10000 | 3.5 MB | firmware A (USB flashes land here) |
| ota_1 | 0x390000 | 3.5 MB | firmware B |
| settings | 0x710000 | 64 KB | kernel settings image |
| vfs | 0x720000 | 8.9 MB | reserved for a file system |

The first four match the MicroPythonOS badge image byte for byte, so the
same flasher scripts address both firmwares and switching back to MPOS
is a reflash, not a repartition.

`flash.sh` passes `--erase-parts otadata`: a USB flash always boots
`ota_0`. `vfs` and `settings` are never erased by a flash.

## Settings persistence

The kernel owns a fixed 64-entry table and exposes it as a 3 332-byte
image (`settings::IMAGE_LEN`). The badge stores that image in the first
4 KB sector of `settings`: `FSET`, `u32` length, image. One erase + one
write per change; settings change a few times per day, so wear is not a
concern for years. No file system, no dependency, 40 lines.

The desktop host writes the same image to `~/.fri3d-badge/settings.bin`;
the browser host to `localStorage`. The kernel does not know which.

## OTA

In place:
- Two slots and `otadata`, read by `esp-bootloader-esp-idf`.
- `ota_confirm_boot()`: after the kernel is up, the running image is
  marked `Valid`. The ESP-IDF bootloader's rollback (boot the previous
  slot if the new image never confirms) therefore protects every update.

Missing:
- An updater: receive an image, write it to `next_partition()`, call
  `activate_next_partition()`, reboot. The API is there
  (`OtaUpdater` in `esp-bootloader-esp-idf`).
- A transport. First candidate: USB-serial with a tiny framed protocol,
  because it needs no Wi-Fi stack. Second: `esp-radio` Wi-Fi + HTTP from
  a build/app-store service.
- Signing. Ed25519 over the image, key in the firmware, before any
  over-the-air path is enabled.

## File system: the pure-Rust decision

`littlefs2` wraps the C littlefs — out, per the no-C rule. Pure-Rust
options:

| Crate | Shape | Fit |
| --- | --- | --- |
| `sequential-storage` | key→value map and queue on raw flash, wear-aware | settings, counters, small blobs |
| `ekv` | key→value database, transactions | app data |
| `tickv` | Tock OS key→value | similar |
| a pure-Rust littlefs | does not exist in a mature form | — |

For apps installed at runtime we do not need a file system: a `.fab` is
self-describing, so an "installed apps" region can be a simple
append-only log of bundles with a header index. The registry already
takes `&'static [u8]`; a flash-mapped slice works as well as
`include_bytes!`. That is the planned next step for `vfs`.
