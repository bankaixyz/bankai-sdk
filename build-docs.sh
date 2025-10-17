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

# Create a root index.html that redirects to the main SDK docs
cat > public/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Bankai SDK Documentation</title>
    <meta http-equiv="refresh" content="0; url=bankai_sdk/index.html">
    <link rel="canonical" href="bankai_sdk/index.html">
</head>
<body>
    <p>Redirecting to <a href="bankai_sdk/index.html">Bankai SDK Documentation</a>...</p>
</body>
</html>
EOF

echo "Documentation built successfully in public/ directory"
