#!/bin/bash

set -e # Exit on any error

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Create output directory if it doesn't exist
mkdir -p permute-library

# Function to check dynamic library dependencies
check_dylib_deps() {
    local dylib=$1
    echo "Checking dependencies for $dylib..."
    otool -L "$dylib"
}

# Copy ARM64 libsndfile to a temporary location
echo "Setting up ARM64 libsndfile..."
mkdir -p arm64_lib
cp ../libsndfile-binaries/libsndfile_universal.dylib arm64_lib/libsndfile.dylib

# Also set up x86_64 libsndfile in the same way
echo "Setting up x86_64 libsndfile..."
mkdir -p x86_64_lib
cp ../libsndfile-binaries/libsndfile_universal.dylib x86_64_lib/libsndfile.dylib

echo "Building for ARM64..."
RUSTFLAGS="-L $(pwd)/arm64_lib" cargo build --release --target aarch64-apple-darwin || {
    echo "Failed to build for ARM64"
    exit 1
}

echo "Building for x86_64..."
RUSTFLAGS="-L $(pwd)/x86_64_lib" cargo build --release --target x86_64-apple-darwin || {
    echo "Failed to build for x86_64"
    exit 1
}

# Check if both files exist
if [ ! -f "target/aarch64-apple-darwin/release/libpermute_node.dylib" ]; then
    echo "ARM64 build not found"
    exit 1
fi

if [ ! -f "target/x86_64-apple-darwin/release/libpermute_node.dylib" ]; then
    echo "x86_64 build not found"
    exit 1
fi

# Use install_name_tool to fix the library references from /usr/local/lib to @rpath
echo "Fixing dynamic library paths for ARM64 build..."
install_name_tool -change /usr/local/lib/libsndfile.1.dylib @rpath/libsndfile.1.dylib "target/aarch64-apple-darwin/release/libpermute_node.dylib"

echo "Fixing dynamic library paths for x86_64 build..."
install_name_tool -change /usr/local/lib/libsndfile.1.dylib @rpath/libsndfile.1.dylib "target/x86_64-apple-darwin/release/libpermute_node.dylib"

# Check dependencies for both builds
echo "Checking ARM64 build dependencies..."
check_dylib_deps "target/aarch64-apple-darwin/release/libpermute_node.dylib"

echo "Checking x86_64 build dependencies..."
check_dylib_deps "target/x86_64-apple-darwin/release/libpermute_node.dylib"

echo "Creating universal binary..."
lipo -create \
    "target/aarch64-apple-darwin/release/libpermute_node.dylib" \
    "target/x86_64-apple-darwin/release/libpermute_node.dylib" \
    -output "permute-library/index.node" || {
    echo "Failed to create universal binary"
    exit 1
}

# Fix the dynamic library paths in the final universal binary
echo "Fixing dynamic library paths in universal binary..."
install_name_tool -change /usr/local/lib/libsndfile.1.dylib @rpath/libsndfile.1.dylib "permute-library/index.node"
# Also add an rpath to look in the Frameworks directory relative to the executable
install_name_tool -add_rpath @executable_path/../Frameworks "permute-library/index.node"

# Check final binary
echo "Checking universal binary dependencies..."
check_dylib_deps "permute-library/index.node"

npm run build-ts

# Clean up temporary directories
rm -rf arm64_lib x86_64_lib

echo "Build completed successfully!"