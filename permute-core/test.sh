RUST_BACKTRACE=full cargo build --release
RUST_BACKTRACE=full target/release/permute --file=examples/beep24.wav --trimAll --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=4 --depth=1  --normalise --processorCount=1 --processor='Lazer'
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/beep241.wav
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/beep242.wav
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/beep243.wav
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/beep244.wav