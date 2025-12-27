#!/bin/bash
# Build web version of Fri3d Badge Emulator
# Usage: ./build_web.sh

set -e

echo "Building WASM apps..."
zig build

echo "Setting up www directory..."
mkdir -p www
cp src/ports/web/index.html www/
cp src/ports/web/platform.js www/
cp zig-out/bin/*.wasm www/

echo ""
echo "Build complete! To run locally:"
echo "  cd www && python3 -m http.server 8000"
echo "  Then open http://localhost:8000"
