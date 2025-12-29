#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building all targets ==="
echo ""

# Build desktop emulator + apps
./build_desktop.sh

echo ""

# Build web target (optional, requires wasm-pack)
if command -v wasm-pack &> /dev/null; then
    ./build_web.sh
else
    echo "Skipping web build (wasm-pack not installed)"
    echo "Install with: cargo install wasm-pack"
fi

echo ""
echo "=== All builds complete! ==="
