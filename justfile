# List recipes
[private]
list:
    @just --list

# Serve the pure-WASM web app locally with live reload
web-serve: _trunk
    #!/usr/bin/env sh
    set -eu
    cd crates/beanifier-web
    trunk serve --open

# Build the pure-WASM web app into crates/beanifier-web/dist
web-build: _trunk
    #!/usr/bin/env sh
    set -eu
    cd crates/beanifier-web
    trunk build --release

# Ensure the wasm32 target and the trunk bundler are installed
[private]
_trunk:
    #!/usr/bin/env sh
    set -eu
    rustup target list --installed | grep -qx wasm32-unknown-unknown \
        || rustup target add wasm32-unknown-unknown
    command -v trunk >/dev/null 2>&1 || cargo install trunk

# Build the whole workspace
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Lint with clippy
lint:
    cargo clippy --workspace --all-targets

# Format all code
fmt:
    cargo fmt --all
