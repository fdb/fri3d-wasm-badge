#!/usr/bin/env bash
# Compare the C++ app-runner's output to the Rust-generated visual goldens
# at tests/visual/apps/<app>/golden/<scene>.png. Any mismatch = a gap
# between the C++ and Rust implementations (most likely: missing font /
# draw_str support, or a canvas primitive regression).
#
# Requires: uv (for Pillow). Run from the repo root.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../../.."

# Ensure the runner binary is fresh.
firmware/tools/app-runner/build.sh >/dev/null

# Ensure release-optimized .wasm exists for each app.
for app in circles snake mandelbrot test_drawing test_ui wifi_pet launcher dots; do
    crate="fri3d-app-${app//_/-}"
    cargo build --release --quiet -p "$crate" --target wasm32-unknown-unknown 2>/dev/null || continue
    src="target/wasm32-unknown-unknown/release/fri3d_app_${app}.wasm"
    dst="firmware/build/fri3d_app_${app}_opt.wasm"
    mkdir -p firmware/build
    wasm-opt -Oz --strip-debug -o "$dst" "$src"
done

APPS=(circles snake mandelbrot test_drawing test_ui wifi_pet launcher dots)

pass=0
fail=0
missing=0

for app in "${APPS[@]}"; do
    wasm="firmware/build/fri3d_app_${app}_opt.wasm"
    golden="tests/visual/apps/${app}/golden/0.png"
    if [[ ! -f "$wasm" ]]; then
        echo "[skip]    $app (no wasm at $wasm)"
        continue
    fi
    if [[ ! -f "$golden" ]]; then
        echo "[skip]    $app (no golden at $golden)"
        missing=$((missing + 1))
        continue
    fi
    tmpbin="$(mktemp)"
    firmware/tools/app-runner/app_runner "$wasm" \
        --out "/tmp/${app}_cpp.png" \
        --out-bin "$tmpbin" \
        --frames 1 --seed 42 >/dev/null 2>&1

    diffs=$(uv run --with pillow --no-project python3 -c "
from PIL import Image
gold = Image.open('$golden').convert('L')
import sys
with open('$tmpbin', 'rb') as f:
    got = f.read()
d = 0
for y in range(gold.height):
    for x in range(gold.width):
        g = gold.getpixel((x, y))
        c = got[y*gold.width + x]
        if (g == 0 and c != 1) or (g == 255 and c != 0):
            d += 1
print(d)
")
    if [[ "$diffs" == "0" ]]; then
        echo "[pass]    $app"
        pass=$((pass + 1))
    else
        echo "[FAIL]    $app ($diffs px differ) — see /tmp/${app}_cpp.png vs $golden"
        fail=$((fail + 1))
    fi
    rm -f "$tmpbin"
done

echo
echo "== summary: $pass passed, $fail failed, $missing no-golden =="
exit $((fail > 0))
