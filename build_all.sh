#!/bin/bash
set -e

cd "$(dirname "$0")"

# Build host emulator
echo "=== Building host emulator ==="
cmake -B build
cmake --build build

# Build all apps
TOOLCHAIN="$(pwd)/cmake/toolchain-wasm.cmake"

for app_dir in src/apps/*/; do
    if [ -d "$app_dir" ]; then
        app_name=$(basename "$app_dir")
        echo ""
        echo "=== Building app: $app_name ==="
        cmake -B "build-apps/$app_name" -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN" -S "$app_dir"
        cmake --build "build-apps/$app_name"
    fi
done

echo ""
echo "=== Build complete! ==="
echo ""
echo "Run an app with:"
echo "  ./build/src/host/host_emulator build-apps/<app_name>/<app_name>"
echo ""
echo "Available apps:"
for app_dir in src/apps/*/; do
    if [ -d "$app_dir" ]; then
        app_name=$(basename "$app_dir")
        echo "  ./build/src/host/host_emulator build-apps/$app_name/$app_name"
    fi
done
