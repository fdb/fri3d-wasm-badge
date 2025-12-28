#!/bin/bash
# Build desktop emulator for Fri3d Badge
# Usage: ./build_emulator.sh
#
# Requires: SDL2 (brew install sdl2 / apt install libsdl2-dev)
# WAMR is built from source by Zig - no CMake needed!

set -e

echo "Building desktop emulator..."
zig build emulator -Doptimize=ReleaseFast

echo ""
echo "Build complete! Emulator is at ./zig-out/bin/fri3d_emulator"
echo ""
echo "To run:"
echo "  ./zig-out/bin/fri3d_emulator                           # Run default app"
echo "  ./zig-out/bin/fri3d_emulator zig-out/bin/circles_zig.wasm  # Run specific app"
