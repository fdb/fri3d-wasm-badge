#!/bin/bash
# Build WASM apps for Fri3d Badge
# Usage: ./build_apps.sh
#
# This builds all Zig apps to WASM format.
# The same .wasm files work on web, desktop emulator, and ESP32.

set -e

echo "Building WASM apps..."
zig build

echo ""
echo "Build complete! WASM files are in zig-out/bin/"
echo ""
echo "Available apps:"
ls -1 zig-out/bin/*.wasm 2>/dev/null || echo "  (no apps found)"
