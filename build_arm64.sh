#!/bin/bash

set -e

cd "$(dirname "$0")"

# Install frontend dependencies if needed
if [ ! -d "permute-tauri/node_modules" ]; then
    echo "Installing frontend dependencies..."
    cd permute-tauri
    npm install
    cd ..
fi

echo "Building Permute (universal macOS binary)..."
cd permute-tauri
npm run build:universal

echo ""
echo "Build complete. Output:"
echo "  permute-tauri/src-tauri/target/universal-apple-darwin/release/bundle/"
