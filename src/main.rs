use hound::{self};
use structopt::StructOpt;

/// Reverse file
#[derive(StructOpt, Clone)]
struct ReverseArgs {
    /// The audio file to look for
    #[structopt(long, short)]
    file: String,
    /// Where to store the reversed file
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
    let args = ReverseArgs::from_args();
    reverse_file(args.clone());
}

fn reverse_file(args: ReverseArgs) {
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

struct ProcessorParams {
    spec: hound::WavSpec,
    samples: Vec<f64>,
    sample_length: usize,
}

fn reverse(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        new_samples[i] = samples[sample_length - 1 - i]
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
    };
}

fn delay(params: ProcessorParams) -> ProcessorParams {
    delay_line(params, 0.75, 4000)
}

fn delay_line(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
    feedback_factor: f64, // 0 - 1
    delay_sample_length: usize,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        let delay_i = i - delay_sample_length;
        if i >= delay_sample_length {
            new_samples[i] += samples[delay_i] * feedback_factor
        }
    }

    let new_params = ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
    };

    let new_feedback_factor = feedback_factor * feedback_factor;

    if new_feedback_factor > 0_f64 {
        return delay_line(new_params, new_feedback_factor, delay_sample_length);
    }
    return new_params;
}

fn half_gain(params: ProcessorParams) -> ProcessorParams {
    gain(params, 0.5)
}

fn gain(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
    gain_factor: f64,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        new_samples[i] = samples[i] * gain_factor;
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
    };
}
