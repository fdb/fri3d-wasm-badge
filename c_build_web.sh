#!/bin/bash
set -e

cd "$(dirname "$0")"

# Check for Emscripten
if ! command -v emcmake &> /dev/null; then
    echo "Error: Emscripten not found."
    echo ""
    echo "Install via Homebrew:"
    echo "  brew install emscripten"
    echo ""
    echo "Or install the SDK:"
    echo "  git clone https://github.com/emscripten-core/emsdk.git"
    echo "  cd emsdk && ./emsdk install latest && ./emsdk activate latest"
    echo "  source ./emsdk_env.sh"
    exit 1
fi

BUILD_DIR="build"

# Build all WASM apps first (they get embedded in the web build)
echo "=== Building WASM apps ==="
TOOLCHAIN="$(pwd)/cmake/toolchain-wasm.cmake"
APPS_BUILD_DIR="$BUILD_DIR/apps"

for app_dir in src/apps/*/; do
    if [ -d "$app_dir" ]; then
        app_name=$(basename "$app_dir")
        echo "Building app: $app_name"
        cmake -B "$APPS_BUILD_DIR/$app_name" -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN" -S "$app_dir" > /dev/null
        cmake --build "$APPS_BUILD_DIR/$app_name" > /dev/null
    fi
done

# Build web target with Emscripten (using root CMakeLists.txt)
echo ""
echo "=== Building web emulator ==="

emcmake cmake -B "$BUILD_DIR/web" -S . -DBUILD_EMULATOR=OFF -DBUILD_WEB=ON
cmake --build "$BUILD_DIR/web"

echo ""
echo "=== Build complete! ==="
echo ""
echo "To run the web emulator:"
echo "  cd build/web/src/web && python3 -m http.server 8080"
echo ""
echo "Then open: http://localhost:8080/fri3d_web.html"
