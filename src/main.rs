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

    println!("{}", header.bits_per_sample);
    if header.bits_per_sample != 24 {
        panic!("Only 24 bit files supported")
    }

    let new_data = reverse(&data);

    let mut output_file = File::create(Path::new(&args.output)).expect("Error creating file");
    wav::write(header, &new_data, &mut output_file).expect("Error writing to file");
}

fn reverse(data: &BitDepth) -> BitDepth {
    let rev = data.as_twenty_four().unwrap();
    let mut new_rev = rev.clone();

    let length = rev.len();
    for i in 1..length {
        new_rev[i] = rev[length - 1 - i]
    }

    BitDepth::TwentyFour(new_rev.to_vec())
}
