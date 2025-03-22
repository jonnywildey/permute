#!/bin/bash

set -e # Exit on any error

echo "Building Permute for ARM64..."

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Step 1: Build permute-core
echo "Building permute-core..."
cd permute-core
./build.sh
cd ..

# Step 2: Build permute-node with universal binary support
echo "Building permute-node..."
cd permute-node
./build.sh
cd ..

# Step 3: Install the new permute-node package in the app
echo "Installing permute-node in the app..."
cd permute-node
npm run update-deps
cd ..

# Step 4: Build and package the app
echo "Building and packaging the app..."
cd permute-app
npm run package
cd ..

echo "Build process completed!" 