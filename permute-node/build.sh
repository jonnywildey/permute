# cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create target/aarch64-apple-darwin/release/libpermute_node.dylib target/x86_64-apple-darwin/release/libpermute_node.dylib -output libpermute_node.dylib
mv libpermute_node.dylib permute-library/index.node