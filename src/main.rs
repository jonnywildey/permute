use std::path::Path;

use hound::{self};
use structopt::StructOpt;
mod process;
mod random_process;
use process::*;
use random_process::*;

/// Permute file
#[derive(StructOpt, Clone)]
struct PermuteArgs {
    /// The audio file to process
    #[structopt(long, short)]
    file: String,
    /// Output of processed file
    #[structopt(long, short = "o")]
    output: String,
    /// Trail to add at beginning of file in seconds
    #[structopt(long = "inputTrail", default_value = "0")]
    input_trail: f64,
    /// Trail to add at beginning of file in seconds
    #[structopt(long = "outputTrail", default_value = "0")]
    output_trail: f64,
    /// Number of times to randomly process file
    #[structopt(long, short)]
    permutations: usize,
    /// How much the file is permuted. Numbers larger than 5 will take a long time to process
    #[structopt(long = "depth", short, default_value = "1")]
    permutation_depth: usize,
}

fn main() {
    let args = PermuteArgs::from_args();
    permute_file(args.clone());
}

fn permute_file(args: PermuteArgs) {
    println!(
        "Permuting {} to {}, {} mutations",
        args.file, args.output, args.permutations
    );

    let mut reader = hound::WavReader::open(args.file).expect("Error opening file");
    let spec = reader.spec();

    let processor_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 64,
        sample_format: hound::SampleFormat::Float,
    };

    // Set all values to -1..1
    let normalise_factor: f64 = match spec.bits_per_sample {
        0..=24 => 1_f64 / (2_f64.powf((spec.bits_per_sample - 1) as f64) - 1_f64),
        _ => 1_f64,
    };

    let denormalise_factor = (1_f64 / normalise_factor) - 1_f64;

    let samples_64 = reader
        .samples::<i32>()
        .map(|x| (x.unwrap()) as f64 * normalise_factor)
        .collect::<Vec<f64>>();
    let input_trail_buffer =
        vec![0_f64; (spec.sample_rate as f64 * args.input_trail).ceil() as usize];
    let output_trail_buffer =
        vec![0_f64; (spec.sample_rate as f64 * args.output_trail).ceil() as usize];
    let samples_64 = [input_trail_buffer, samples_64, output_trail_buffer].concat();

    let sample_length = samples_64.len();

    let processor_params = ProcessorParams {
        samples: samples_64,
        spec: processor_spec,
        sample_length: sample_length,
    };

    let output = args.output;
    for i in 1..=args.permutations {
        let output_i = generate_file_name(output.clone(), i);
        println!("Permutating {:?}", output_i);

        // let processors = generate_processor_sequence(GetProcessorNodeParams {
        //     depth: args.permutation_depth,
        //     normalise_at_end: true,
        // });

        let processors: Vec<ProcessorFn> = vec![half_speed];

        let output_params = run_processors(RunProcessorsParams {
            processor_params: processor_params.clone(),
            processors: processors,
        });
        let mut pro_writer = hound::WavWriter::create(output_i, spec).expect("Error in output");

        for s in output_params.samples {
            let t = (s * denormalise_factor) as i32;
            pro_writer.write_sample(t).expect("Error writing file");
        }
        pro_writer.finalize().expect("Error writing file");
    }
}

fn generate_file_name(output: String, permutation_count: usize) -> std::path::PathBuf {
    let path = Path::new(&output);
    let file_stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
    let extension = path.extension().unwrap_or_default().to_str().unwrap_or("");
    let new_filename = [file_stem, &permutation_count.to_string(), ".", extension].concat();

    path.with_file_name(new_filename)
}
