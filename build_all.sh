#!/bin/bash
# Build everything for Fri3d Badge
# Usage: ./build_all.sh
#
# This builds:
# - WASM apps (zig-out/bin/*.wasm)
# - Desktop emulator (zig-out/bin/fri3d_emulator)
# - Web version (www/)

set -e

cd "$(dirname "$0")"

echo "=== Building WASM apps ==="
zig build

echo ""
echo "=== Building desktop emulator ==="
zig build emulator -Doptimize=ReleaseFast

echo ""
echo "=== Building web version ==="
zig build web

echo ""
echo "=== Build complete! ==="
echo ""
echo "WASM apps:        zig-out/bin/*.wasm"
echo "Desktop emulator: ./zig-out/bin/fri3d_emulator"
echo "Web version:      www/"
echo ""
echo "Run the emulator:"
echo "  ./zig-out/bin/fri3d_emulator"
echo ""
echo "Run web version:"
echo "  cd www && python3 -m http.server 8000"
echo "  Then open http://localhost:8000"
