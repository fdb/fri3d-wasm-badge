#!/usr/bin/env bash
# Regenerate the Rust-side visual golden PNGs for every Fri3d app that has
# a config.yaml in tests/visual/apps/<app>/. Output lands in
# tests/visual/apps/<app>/golden/0.png and is what firmware/tools/
# app-runner/visual_parity.sh diffs the C++ renders against.
#
# Pass --dry-run to list what would be regenerated without writing files.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

DRY_RUN=0
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=1

cargo build --release --quiet -p visual-golden-gen

FRAMES=1
SEED=42

for app_dir in tests/visual/apps/*/; do
    app=$(basename "$app_dir")
    crate="fri3d-app-${app//_/-}"
    wasm="firmware/build/fri3d_app_${app}_opt.wasm"
    out="${app_dir}golden/0.png"

    if [[ ! -f "$wasm" ]]; then
        echo "[skip]  $app (no optimized wasm at $wasm; run firmware/scripts/embed_apps.sh first)"
        continue
    fi

    if [[ $DRY_RUN -eq 1 ]]; then
        echo "[plan]  $app -> $out"
    else
        target/release/visual-golden-gen "$wasm" "$out" --frames "$FRAMES" --seed "$SEED"
    fi
done
