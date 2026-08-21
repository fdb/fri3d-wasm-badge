#!/usr/bin/env bash
# Pack the apps, build the badge firmware, flash it, open the monitor.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"

for tool in cargo espflash; do
    command -v "$tool" >/dev/null || { echo "missing $tool (see hosts/badge/README.md)" >&2; exit 1; }
done

if ! ls /dev/cu.usbmodem* >/dev/null 2>&1; then
    echo "no /dev/cu.usbmodem* — plug in the badge over USB-C" >&2
    echo "(if it is plugged in and not enumerating: hold START + replug to enter the bootloader)" >&2
    exit 1
fi

echo "==> packing apps"
(cd "$root" && cargo run -q -p fri3d-pack)

echo "==> building firmware"
(cd "$here" && cargo build --release)

echo "==> flashing"
exec espflash flash --monitor --flash-size 16mb \
    "$here/target/xtensa-esp32s3-none-elf/release/fri3d-badge" "$@"
