#!/bin/bash

set -e # Exit on any error

# Check for -dev flag
DEV_MODE=false
if [ "$1" == "-dev" ]; then
    DEV_MODE=true
    echo "Running in development mode..."
else
    echo "Building Permute for ARM64..."
fi

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Step 1: Build permute-core (skip in dev mode)
if [ "$DEV_MODE" = false ]; then
    echo "Building permute-core..."
    cd permute-core
    arch -arm64 ./build.sh
    cd ..
fi

# Step 2: Build permute-node with universal binary support
echo "Building permute-node..."
cd permute-node
arch -arm64 ./build.sh
cd ..

# Step 3: Install the new permute-node package in the app
echo "Installing permute-node in the app..."
cd permute-node
arch -arm64 npm run update-deps
cd ..

# Step 4: Build and package the app (skip in dev mode)
if [ "$DEV_MODE" = false ]; then
    echo "Building and packaging the app..."
    cd permute-app
    arch -arm64 npm run package
    cd ..
    echo "Build process completed!"
else
    echo "Starting development server..."
    cd permute-app
    arch -arm64 npm start
fi 