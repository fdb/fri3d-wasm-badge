#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building all targets ==="
echo ""

# Build emulator (native)
./build_emulator.sh

echo ""

# Build web target
./build_web.sh

echo ""
echo "=== All builds complete! ==="
