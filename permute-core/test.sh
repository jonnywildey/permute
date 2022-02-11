cargo build --release && RUST_BACKTRACE=full 
target/release/permute --file renders/YipWoo1.wav  --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1  --normalise --processorCount=1 --processor='Trim'
