use hound::{self};
use structopt::StructOpt;
mod process;
use process::{delay, half_gain, reverse, ProcessorParams};

/// Permute file
#[derive(StructOpt, Clone)]
struct PermuteArgs {
    /// The audio file to process
    #[structopt(long, short)]
    file: String,
    /// Output of processed file
    #[structopt(long, short)]
    output: String,
    /// Trail to add at beginning of file in seconds
    #[structopt(long = "inputTrail", short, default_value = "0")]
    input_trail: f64,
    /// Trail to add at beginning of file in seconds
    #[structopt(long = "outputTrail", short, default_value = "0")]
    output_trail: f64,
}

fn main() {
    let args = PermuteArgs::from_args();
    permute_file(args.clone());
}

fn permute_file(args: PermuteArgs) {
    println!("Reversing {} to {}", args.file, args.output);

    let mut reader = hound::WavReader::open(args.file).expect("Error opening file");
    let spec = reader.spec();

    let processor_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 64,
        sample_format: hound::SampleFormat::Float,
    };

    let normalise_factor: f64 = match spec.bits_per_sample {
        0..=24 => 1_f64 / (2_f64.powf(spec.bits_per_sample as f64) - 1_f64),
        _ => 1_f64,
    };

    let denormalise_factor = 1_f64 / normalise_factor;

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

    let processors: Vec<fn(ProcessorParams) -> ProcessorParams> =
        vec![half_gain, half_gain, reverse, delay, delay, reverse];

    let output_params = processors
        .iter()
        .fold(processor_params, |params, processor| processor(params));

    let mut pro_writer = hound::WavWriter::create(args.output, spec).expect("Error in output");

    for s in output_params.samples {
        pro_writer
            .write_sample((s * denormalise_factor) as i32)
            .expect("Error writing file");
    }
    pro_writer.finalize().expect("Error writing file");
}
