#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building all Rust targets ==="
echo ""

echo "=== Building Rust WASM apps ==="
./build_apps.sh

echo ""
echo "=== Building Rust emulator ==="
./build_emulator.sh

echo ""
echo "=== Building Rust web emulator ==="
./build_web.sh

echo ""
echo "=== All Rust builds complete! ==="
