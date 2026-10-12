#!/bin/sh
# Release build of the browser game. Output lands in web/pkg/.
set -eu
cd "$(dirname "$0")/.."
rustup target add wasm32-unknown-unknown
wasm-pack build --target web --release --out-dir web/pkg
rm -f web/pkg/.gitignore web/pkg/README.md
