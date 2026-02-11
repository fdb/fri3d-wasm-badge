#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "=== Building Rust WASM apps ==="

target_dir="target/wasm32-unknown-unknown/debug"
output_root="build/apps"

rm -rf "$output_root"

for app_dir in fri3d-app-*/; do
    if [ ! -d "$app_dir" ]; then
        continue
    fi

    cargo_toml="$app_dir/Cargo.toml"
    if [ ! -f "$cargo_toml" ]; then
        continue
    fi

    app_name=$(basename "$app_dir")
    echo ""
    echo "=== Building app: $app_name ==="
    cargo build -p "$app_name" --target wasm32-unknown-unknown

    wasm_file="$target_dir/${app_name//-/_}.wasm"
    if [ ! -f "$wasm_file" ]; then
        echo "Error: expected wasm output at $wasm_file" >&2
        exit 1
    fi

    app_id=${app_name#fri3d-app-}
    app_id=${app_id//-/_}
    output_dir="$output_root/$app_id"
    mkdir -p "$output_dir"
    cp "$wasm_file" "$output_dir/$app_id.wasm"
    echo "Copied to $output_dir/$app_id.wasm"

done

echo ""
echo "=== Rust WASM app build complete! ==="
