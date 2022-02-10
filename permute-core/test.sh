cargo build --release && RUST_BACKTRACE=full 
target/release/permute --file examples/snare.wav  --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1  --normalise --processorCount=1 --processor='Reverb'
target/release/permute --file examples/amen.wav  --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1  --normalise --processorCount=1 --processor='Reverb'
target/release/permute --file examples/guitarloop16mono.wav  --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1  --normalise --processorCount=1 --processor='Reverb'
