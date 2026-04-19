#!/usr/bin/env bash
# Canvas parity test runner. Regenerates Rust golden framebuffers and diffs
# the firmware's C++ canvas against them byte-for-byte.
#
# Use this any time you modify firmware/src/canvas.{h,cpp} or the reference
# in fri3d-runtime/src/canvas.rs — if parity breaks, the two implementations
# have drifted and a WASM app will render differently depending on target.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

echo "[parity] regenerating Rust goldens..."
cargo run --quiet -p canvas-parity-gen

echo "[parity] building C++ test..."
c++ -std=c++17 -O2 \
    -I firmware/src \
    firmware/src/canvas.cpp \
    firmware/src/font.cpp \
    firmware/tools/canvas-parity/canvas_parity.cpp \
    -o firmware/tools/canvas-parity/canvas_parity

echo "[parity] running..."
exec firmware/tools/canvas-parity/canvas_parity
