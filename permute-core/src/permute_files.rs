use std::path::Path;

use crate::process::*;
use crate::random_process::*;

pub type UpdatePermuteNodeProgress =
    fn(permutation: Permutation, name: PermuteNodeName, event: PermuteNodeEvent);
pub type UpdateSetProcessors = fn(permutation: Permutation, processors: Vec<PermuteNodeName>);

#[derive(Debug, Clone)]
pub struct PermuteFilesParams {
    pub files: Vec<String>,
    pub output: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,

    pub update_permute_node_progress: UpdatePermuteNodeProgress,
    pub update_set_processors: UpdateSetProcessors,
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
        update_set_processors,
    }: PermuteFilesParams,
    file: String,
) {
    let mut reader = hound::WavReader::open(file.clone()).expect("Error opening file");
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

    for i in 1..=permutations {
        let permutation = Permutation {
            file: file.clone(),
            permutation_index: i,
            output: output.clone(),
            processor_pool: processor_pool.clone(),
            node_index: -1, // easier to add 1
        };
        let processor_params = ProcessorParams {
            samples: samples_64.clone(),
            spec: processor_spec,
            sample_length: sample_length,
            permutation: permutation.clone(),
            update_progress: update_permute_node_progress,
        };

        let output_i = generate_file_name(output.clone(), i);

        let processors = generate_processor_sequence(GetProcessorNodeParams {
            depth: permutation_depth,
            normalise_at_end: true,
            processor_pool: processor_pool.clone(),
        });
        update_set_processors(permutation.clone(), processors.clone());

        let processors = processors
            .iter()
            .map(|n| get_processor_function(*n))
            .collect();

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
    processors.iter().fold(
        processor_params,
        |ProcessorParams {
             permutation:
                 Permutation {
                     file,
                     permutation_index,
                     output,
                     processor_pool,
                     node_index,
                 },
             sample_length,
             update_progress,
             samples,
             spec,
         },
         processor| {
            processor(ProcessorParams {
                permutation: Permutation {
                    file,
                    node_index: node_index + 1,
                    output,
                    permutation_index,
                    processor_pool,
                },
                sample_length,
                samples,
                spec,
                update_progress,
            })
        },
    )
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
        PermuteNodeName::Normalise => String::from("Normalise"),
    }
}
