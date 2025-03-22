#!/bin/bash

set -e # Exit on any error

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Setup libsndfile binaries
echo "Setting up libsndfile binaries..."
mkdir -p ../libsndfile-binaries/arm64
mkdir -p ../libsndfile-binaries/x86_64

# Copy the architecture-specific libraries
cp ../libsndfile-binaries/libsndfile_arm.dylib ../libsndfile-binaries/arm64/libsndfile.dylib
cp ../lib-sndfile-src/libsndfile.dylib ../libsndfile-binaries/x86_64/libsndfile.dylib

echo "Building for ARM64..."
cargo build --release --target aarch64-apple-darwin || {
    echo "Failed to build for ARM64"
    exit 1
}

echo "Building for x86_64..."
cargo build --release --target x86_64-apple-darwin || {
    echo "Failed to build for x86_64"
    exit 1
}

# Check if both files exist
if [ ! -f "target/aarch64-apple-darwin/release/libpermute.rlib" ]; then
    echo "ARM64 build not found"
    exit 1
fi

if [ ! -f "target/x86_64-apple-darwin/release/libpermute.rlib" ]; then
    echo "x86_64 build not found"
    exit 1
fi

echo "Build completed successfully!"