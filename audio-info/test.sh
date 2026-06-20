#!/bin/bash

set -e # Exit on any error
RUST_BACKTRACE=full cargo build --release

RUST_BACKTRACE=full target/release/audio-info --file=/Users/jonnywildey/downloads/songa.wav --output=songa.svg
