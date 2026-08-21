#!/usr/bin/env bash
# Deploy the firmware to a badge on USB-C: pack the apps, build
# hosts/badge, flash it, and print the first seconds of the boot log.
#
#   scripts/deploy_to_badge.sh             # flash, show 8 s of boot log, exit
#   scripts/deploy_to_badge.sh --monitor   # flash and stay in the serial monitor
#   scripts/deploy_to_badge.sh --no-build  # flash the last build
#
# Exit codes: 1 = missing tool or badge, 2 = build failed, 3 = flash failed.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
badge="$root/hosts/badge"
elf="$badge/target/xtensa-esp32s3-none-elf/release/fri3d-badge"
log_seconds=12
monitor=0
build=1

for arg in "$@"; do
    case "$arg" in
        --monitor) monitor=1 ;;
        --no-build) build=0 ;;
        -h|--help) sed -n '2,9p' "$0"; exit 0 ;;
        *) echo "unknown argument: $arg" >&2; exit 1 ;;
    esac
done

for tool in cargo espflash; do
    command -v "$tool" >/dev/null || { echo "missing $tool (see hosts/badge/README.md)" >&2; exit 1; }
done
rustup toolchain list | grep -q '^esp' || { echo "missing esp toolchain: cargo install espup && espup install" >&2; exit 1; }

port="$(ls /dev/cu.usbmodem* 2>/dev/null | head -n 1 || true)"
if [[ -z "$port" ]]; then
    echo "no /dev/cu.usbmodem* — plug in the badge over USB-C" >&2
    echo "(plugged in but silent: hold START and replug to enter the bootloader)" >&2
    exit 1
fi

if (( build )); then
    echo "==> packing apps"
    (cd "$root" && cargo run -q -p fri3d-pack)
    echo "==> building firmware"
    (cd "$badge" && cargo build --release) || exit 2
fi
[[ -f "$elf" ]] || { echo "no firmware at $elf" >&2; exit 2; }

echo "==> flashing $port"
# OTA layout (partitions.csv): otadata is erased so a USB flash always
# boots ota_0; the vfs partition is left alone.
flash_args=(--port "$port" --flash-size 16mb
    --partition-table "$badge/partitions.csv" --erase-parts otadata)
if (( monitor )); then
    exec espflash flash --monitor "${flash_args[@]}" "$elf"
fi
espflash flash "${flash_args[@]}" "$elf" || exit 3

echo "==> boot log (${log_seconds}s)"
# espflash's reset after a flash can land in download mode on USB-JTAG;
# a plain monitor read shows either the kernel banner or that state.
# macOS has no `timeout`: run the monitor in the background and stop it.
boot_log="$(mktemp)"
espflash monitor --port "$port" --non-interactive >"$boot_log" 2>/dev/null &
mon=$!
sleep "$log_seconds"
kill "$mon" 2>/dev/null || true
wait "$mon" 2>/dev/null || true
grep -aE '\[fri3d\]|\[perf\]|panic|waiting for download' "$boot_log" || echo "(no kernel output in ${log_seconds}s — replug USB-C without holding a button)"
rm -f "$boot_log"
echo "==> done"
