cargo build --release && RUST_BACKTRACE=full 
target/release/permute --file examples/vibe1.wav  --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1  --normalise --processorCount=2 --processor='Wow'
