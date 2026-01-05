#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building Rust WASM apps ==="
./build_apps.sh

echo ""
echo "=== Building Rust web host ==="
cargo build -p fri3d-web --target wasm32-unknown-unknown --release

OUTPUT_DIR="build/web"
APP_OUTPUT_DIR="$OUTPUT_DIR/apps"

mkdir -p "$APP_OUTPUT_DIR"

HOST_WASM="target/wasm32-unknown-unknown/release/fri3d_web.wasm"
if [ ! -f "$HOST_WASM" ]; then
    echo "Error: expected wasm output at $HOST_WASM" >&2
    exit 1
fi

cp "$HOST_WASM" "$OUTPUT_DIR/fri3d_web.wasm"
cp "fri3d-web/shell.html" "$OUTPUT_DIR/index.html"

cp -R build/apps/* "$APP_OUTPUT_DIR/"

echo ""
echo "=== Build complete! ==="
echo ""
echo "To run the web emulator:"
echo "  cd build/web && python3 -m http.server 8080"
echo ""
echo "Then open: http://localhost:8080/"
