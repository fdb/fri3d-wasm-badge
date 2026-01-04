#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building Rust emulator ==="
cargo build -p fri3d-emulator --release
