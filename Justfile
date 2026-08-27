default: build

run: build serve

# Run unit and integration tests
test:
    cargo test

# Lint code with clippy and formatting checks
lint:
    cargo clippy --all-targets -- -D warnings
    cargo clippy --target wasm32-unknown-unknown -- -D warnings
    cargo fmt --check

# Format code with rustfmt
fmt:
    cargo fmt

# Build Rust WebAssembly release package
build: test lint
    cargo build --target wasm32-unknown-unknown --release
    mkdir -p www/pkg
    wasm-bindgen --target web --out-dir www/pkg target/wasm32-unknown-unknown/release/invoicer.wasm

# Start a local development web server
serve port="8080": build
    python3 -m http.server {{port}} --directory www

# Run cargo check on native and wasm32 targets
check:
    cargo check
    cargo check --target wasm32-unknown-unknown

# Clean all build artifacts
clean:
    cargo clean
    rm -rf www/pkg
