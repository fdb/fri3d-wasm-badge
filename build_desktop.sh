#!/bin/bash
set -e

echo "Building WASM apps..."
./build_apps.sh

echo "Building desktop emulator..."
cargo build --release -p fri3d-desktop

echo ""
echo "Build complete!"
echo "Run with: ./target/release/fri3d-emulator [app.wasm]"
