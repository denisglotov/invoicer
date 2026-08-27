#!/usr/bin/env bash
set -euo pipefail

echo "==> Running Rust unit tests..."
cargo test

echo "==> Compiling WebAssembly release binary..."
cargo build --target wasm32-unknown-unknown --release

echo "==> Generating WebAssembly bindings with wasm-bindgen..."
mkdir -p www/pkg
wasm-bindgen --target web --out-dir www/pkg target/wasm32-unknown-unknown/release/invoicer.wasm

echo "==> Build complete! Output in www/"
