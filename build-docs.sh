#!/bin/bash
set -e

# Install Rust if not already installed
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install dependencies and build docs
cargo doc --no-deps --workspace --document-private-items

# Copy docs to output directory for Cloudflare Pages
mkdir -p public
cp -r target/doc/* public/

echo "Documentation built successfully in public/ directory"
