#!/bin/bash
set -e

cd "$(dirname "$0")"

# Build all WASM apps
TOOLCHAIN="$(pwd)/cmake/toolchain-wasm.cmake"
APPS_BUILD_DIR="build/apps"

echo "=== Building WASM apps ==="

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
echo "=== WASM app build complete! ==="
