#!/bin/bash
set -e

cd "$(dirname "$0")"

APPS="circles launcher mandelbrot test-drawing test-ui"

echo "=== Building WASM apps ==="

for app in $APPS; do
    echo "  Building $app..."
    cargo build --release \
        --target wasm32-unknown-unknown \
        -p $app 2>&1 | grep -v "^$" || true
done

# Create output directory and copy WASM files
mkdir -p build/apps
for app in $APPS; do
    # Convert app name (test-drawing -> test_drawing for Cargo output)
    cargo_name=$(echo $app | tr '-' '_')
    if [ -f "target/wasm32-unknown-unknown/release/${cargo_name}.wasm" ]; then
        cp "target/wasm32-unknown-unknown/release/${cargo_name}.wasm" "build/apps/${app}.wasm"
    fi
done

echo ""
echo "=== WASM apps build complete! ==="
echo ""
if [ -d "build/apps" ]; then
    ls -la build/apps/
fi
