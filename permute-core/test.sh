#!/bin/bash

set -e # Exit on any error
RUST_BACKTRACE=full cargo build --release
# RUST_BACKTRACE=full target/release/permute --file=examples/beep24.wav --trimAll --createSubdirectories --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=4 --depth=1  --normalise --processorCount=1 --processor='Lazer'
# RUST_BACKTRACE=full target/release/permute --file=examples/beep24.wav --trimAll --createSubdirectories --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=3 --depth=4  --normalise
RUST_BACKTRACE=full target/release/permute --file=examples/beep24.wav --files=examples/vibe.wav --trimAll --output ./renders/ --inputTrail=0 --outputTrail=2  --permutations=1 --depth=1  --normalise --processorCount=1 --processor='Cross Filter'
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/beep241.wav
afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/vibe1.wav
# afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/snare1.wav
# afplay /Users/jonnywildey/rustcode/permute/permute-core/renders/GoodClick1.aif