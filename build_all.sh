#!/bin/bash
set -e

cd "$(dirname "$0")"

# Build output directory
BUILD_DIR="build"

# Build emulator
echo "=== Building emulator ==="
cmake -B "$BUILD_DIR/emulator" -S .
cmake --build "$BUILD_DIR/emulator"

# Build all WASM apps
TOOLCHAIN="$(pwd)/cmake/toolchain-wasm.cmake"
APPS_BUILD_DIR="$BUILD_DIR/apps"

for app_dir in src/apps/*/; do
    if [ -d "$app_dir" ]; then
        app_name=$(basename "$app_dir")
        echo ""
        echo "=== Building app: $app_name ==="
        cmake -B "$APPS_BUILD_DIR/$app_name" -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN" -S "$app_dir"
        cmake --build "$APPS_BUILD_DIR/$app_name"
    fi
done

echo ""
echo "=== Build complete! ==="
echo ""
echo "Run the emulator:"
echo "  ./build/emulator/src/emulator/fri3d_emulator"
echo ""
echo "Run a specific app:"
echo "  ./build/emulator/src/emulator/fri3d_emulator build/apps/<app_name>/<app_name>.wasm"
echo ""
echo "Available apps:"
for app_dir in src/apps/*/; do
    if [ -d "$app_dir" ]; then
        app_name=$(basename "$app_dir")
        echo "  ./build/emulator/src/emulator/fri3d_emulator build/apps/$app_name/$app_name.wasm"
    fi
done
