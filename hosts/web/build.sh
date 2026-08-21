#!/usr/bin/env bash
# Build the browser host: pack apps, compile the kernel to wasm32 via
# wasm-pack, assemble hosts/web/dist/.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"

cd "$root"
cargo run -q -p fri3d-pack

wasm-pack build "$here" --target web --out-dir dist/pkg --release

cp "$here/index.html" "$here/test.html" "$here/tests.js" "$here/dist/"
echo "built $here/dist (serve with: cd $here/dist && python3 -m http.server 8091)"
