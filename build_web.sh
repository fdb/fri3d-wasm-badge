#!/bin/bash
set -e

cd "$(dirname "$0")"

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack not found."
    echo ""
    echo "Install via cargo:"
    echo "  cargo install wasm-pack"
    echo ""
    echo "Or download from: https://rustwasm.github.io/wasm-pack/installer/"
    exit 1
fi

echo "=== Building WASM apps ==="
./build_apps.sh

echo ""
echo "=== Building web runtime ==="
cd ports/web
wasm-pack build --target web --release

# Copy to www directory
cp -r pkg www/
mkdir -p www/apps
cp ../../build/apps/*.wasm www/apps/ 2>/dev/null || true

cd ../..

echo ""
echo "=== Web build complete! ==="
echo ""
echo "To run the web emulator:"
echo "  cd ports/web/www && python3 -m http.server 8080"
echo ""
echo "Then open: http://localhost:8080/"
