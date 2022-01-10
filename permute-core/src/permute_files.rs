use std::path::Path;

use crate::process::*;
use crate::random_process::*;

#[derive(Debug, Clone)]
pub struct PermuteFilesParams {
    pub files: Vec<String>,
    pub output: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub update_permute_node_progress: fn(name: PermuteNodeName, event: PermuteNodeEvent),
}

pub fn permute_files(params: PermuteFilesParams) {
    let copied_params = params.clone();
    let files = params.files;
    for i in 0..files.len() {
        permute_file(copied_params.clone(), files[i].clone())
    }
}

fn permute_file(
    PermuteFilesParams {
        files: _,
        output,
        input_trail,
        output_trail,
        permutations,
        permutation_depth,
        processor_pool,
        update_permute_node_progress,
    }: PermuteFilesParams,
    file: String,
) {
    let mut reader = hound::WavReader::open(file).expect("Error opening file");
    let spec = reader.spec();

    let processor_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 64,
        sample_format: hound::SampleFormat::Float,
    };

    // Set all values to -1..1
    let denormalise_factor = match spec.bits_per_sample {
        0..=24 => 2_f64.powf((spec.bits_per_sample - 1) as f64) - 1_f64,
        _ => 1_f64,
    };
    let normalise_factor: f64 = match spec.bits_per_sample {
        0..=24 => 1_f64 / denormalise_factor,
        _ => 1_f64,
    };

    let samples_64 = reader
        .samples::<i32>()
        .map(|x| (x.unwrap()) as f64 * normalise_factor)
        .collect::<Vec<f64>>();
    let input_trail_buffer =
        vec![0_f64; (spec.sample_rate as f64 * input_trail * spec.channels as f64).ceil() as usize];
    let output_trail_buffer = vec![
        0_f64;
        (spec.sample_rate as f64 * output_trail * spec.channels as f64).ceil()
            as usize
    ];
    let samples_64 = [input_trail_buffer, samples_64, output_trail_buffer].concat();

    let sample_length = samples_64.len();

    let processor_params = ProcessorParams {
        samples: samples_64,
        spec: processor_spec,
        sample_length: sample_length,
        update_progress: update_permute_node_progress,
    };

    let output = output;
    for i in 1..=permutations {
        let output_i = generate_file_name(output.clone(), i);
        println!("Permutating {:?}", output_i);

        let processor_functions = processor_pool
            .iter()
            .map(|n| get_processor_function(*n))
            .collect();

        let processors = generate_processor_sequence(GetProcessorNodeParams {
            depth: permutation_depth,
            normalise_at_end: true,
            processor_functions: processor_functions,
        });

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

pub struct RunProcessorsParams {
    pub processors: Vec<ProcessorFn>,
    pub processor_params: ProcessorParams,
}

pub fn run_processors(
    RunProcessorsParams {
        processors,
        processor_params,
    }: RunProcessorsParams,
) -> ProcessorParams {
    processors
        .iter()
        .fold(processor_params, |params, processor| processor(params))
}

pub fn get_processor_function(name: PermuteNodeName) -> ProcessorFn {
    match name {
        PermuteNodeName::Reverse => reverse,
        PermuteNodeName::Chorus => random_chorus,
        PermuteNodeName::DoubleSpeed => double_speed,
        PermuteNodeName::Flutter => random_flutter,
        PermuteNodeName::HalfSpeed => half_speed,
        PermuteNodeName::MetallicDelay => random_metallic_delay,
        PermuteNodeName::RhythmicDelay => random_rhythmic_delay,
        PermuteNodeName::Wow => random_wow,
    }
}

pub fn get_processor_display_name(name: PermuteNodeName) -> String {
    match name {
        PermuteNodeName::Reverse => String::from("Reverse"),
        PermuteNodeName::Chorus => String::from("Chorus"),
        PermuteNodeName::DoubleSpeed => String::from("Double speed"),
        PermuteNodeName::Flutter => String::from("Flutter"),
        PermuteNodeName::HalfSpeed => String::from("Half speed"),
        PermuteNodeName::MetallicDelay => String::from("Metallic delay"),
        PermuteNodeName::RhythmicDelay => String::from("Rhythmic delay"),
        PermuteNodeName::Wow => String::from("Wow"),
    }
}
