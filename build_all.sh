#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building all Rust targets ==="
echo ""

echo "=== Building Rust emulator ==="
cargo build -p fri3d-emulator

echo ""

./build_apps.sh

echo ""
echo "=== All Rust builds complete! ==="
