#!/bin/bash
# Build web version of Fri3d Badge Emulator
# Usage: ./build_web.sh
#
# This only builds the WASM apps - no SDL2 or native dependencies needed!

set -e

echo "Building WASM apps (Zig)..."
zig build circles_zig test_ui_zig

echo "Building WASM apps (C)..."
zig build circles mandelbrot test_drawing test_ui

echo "Setting up www directory..."
mkdir -p www
cp src/ports/web/index.html www/
cp src/ports/web/platform.js www/
cp zig-out/bin/*.wasm www/

echo ""
echo "Build complete! To run locally:"
echo "  cd www && python3 -m http.server 8000"
echo "  Then open http://localhost:8000"
