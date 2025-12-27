#!/bin/bash
# Build web version of Fri3d Badge Emulator
# Usage: ./build_web.sh
#
# This only builds the WASM apps - no SDL2 or native dependencies needed!

set -e

echo "Building web version..."
zig build web

echo ""
echo "Build complete! Files are in www/"
echo ""
echo "To run locally:"
echo "  cd www && python3 -m http.server 8000"
echo "  Then open http://localhost:8000"
