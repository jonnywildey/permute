use std::fs::File;
use std::path::Path;
use structopt::StructOpt;
use wav::{self, BitDepth};

/// Reverse file
#[derive(StructOpt, Clone)]
struct ReverseArgs {
    /// The audio file to look for
    #[structopt(long, short)]
    file: String,
    /// where to store the reversed file
    #[structopt(long, short)]
    output: String,
}

fn main() {
    let args = ReverseArgs::from_args();
    reverse_file(args.clone());
}

fn reverse_file(args: ReverseArgs) {
    println!("Reversing {} to {}", args.file, args.output);
    let mut path = File::open(Path::new(&args.file)).expect("Error opening file");
    let (header, data) = wav::read(&mut path).expect("Error reading file");
    let new_data = reverse(&data);

    let mut output_file = File::create(Path::new(&args.output)).expect("Error creating file");
    wav::write(header, &new_data, &mut output_file).expect("Error writing to file");
}

fn reverse(data: &BitDepth) -> BitDepth {
    return match data {
        BitDepth::TwentyFour(d) => BitDepth::TwentyFour(rev(d)),
        BitDepth::Sixteen(d) => BitDepth::Sixteen(rev(d)),
        BitDepth::ThirtyTwoFloat(d) => BitDepth::ThirtyTwoFloat(rev(d)),
        BitDepth::Eight(d) => BitDepth::Eight(rev(d)),
        BitDepth::Empty => BitDepth::Empty,
    };
}

fn rev<T: std::marker::Copy>(rev: &Vec<T>) -> Vec<T> {
    let mut new_rev = rev.clone();

    let length = rev.len();
    for i in 1..length {
        new_rev[i] = rev[length - 1 - i]
    }
    new_rev.to_vec()
}
