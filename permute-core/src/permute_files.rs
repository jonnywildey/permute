use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

use crate::process::*;
use crate::random_process::*;

pub enum PermuteUpdate {
    UpdatePermuteNodeStarted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdatePermuteNodeCompleted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdateSetProcessors(Permutation, Vec<PermuteNodeName>),
    ProcessComplete,
}

#[derive(Debug, Clone)]
pub struct PermuteFilesParams {
    pub files: Vec<String>,
    pub output: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,

    pub update_sender: mpsc::Sender<PermuteUpdate>,
}

pub fn permute_files(params: PermuteFilesParams) -> JoinHandle<()> {
    thread::spawn(|| {
        let copied_params = params.clone();
        let files = params.files;
        for i in 0..files.len() {
            permute_file(copied_params.clone(), files[i].clone());
        }
        params
            .update_sender
            .send(PermuteUpdate::ProcessComplete)
            .unwrap();
    })
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
        high_sample_rate,
        normalise_at_end,
        update_sender,
        processor_count,
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
        let output_i = generate_file_name(file.clone(), output.clone(), i);
        let processors = generate_processor_sequence(GetProcessorNodeParams {
            depth: permutation_depth,
            normalise_at_end,
            high_sample_rate,
            processor_pool: processor_pool.clone(),
            processor_count,
        });

        let processor_fns = processors
            .iter()
            .map(|n| get_processor_function(*n))
            .collect();

        let permutation = Permutation {
            file: file.clone(),
            permutation_index: i,
            output: output_i.clone(),
            processor_pool: processor_pool.clone(),
            processors: processors.clone(),
            original_sample_rate: spec.sample_rate,
            node_index: 0,
        };
        let processor_params = ProcessorParams {
            samples: samples_64.clone(),
            spec: processor_spec,
            sample_length: sample_length,
            permutation: permutation.clone(),
            update_sender: update_sender.to_owned(),
        };
        let _ = update_sender.send(PermuteUpdate::UpdateSetProcessors(
            permutation.clone(),
            processors,
        ));

        let output_params = run_processors(RunProcessorsParams {
            processor_params: processor_params.clone(),
            processors: processor_fns,
        });
        let mut pro_writer = hound::WavWriter::create(output_i, spec).expect("Error in output");

        for mut s in output_params.samples {
            // overload protection
            if s >= 1.0 {
                s = 1.0;
            }
            if s <= -1.0 {
                s = -1.0;
            }
            let t = (s * denormalise_factor) as i32;
            pro_writer.write_sample(t).expect("Error writing file");
        }
        pro_writer.finalize().expect("Error writing file");
    }
}

fn generate_file_name(file: String, output: String, permutation_count: usize) -> String {
    let mut dir_path = Path::new(&output).canonicalize().unwrap();
    let file_path = Path::new(&file);
    let file_stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let extension = file_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let new_filename = [file_stem, &permutation_count.to_string(), ".", extension].concat();

    dir_path.push(new_filename);
    dir_path.into_os_string().into_string().unwrap()
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
        .fold(processor_params, |params, processor| {
            let new_params = processor(&params);
            ProcessorParams {
                permutation: Permutation {
                    file: new_params.permutation.file,
                    node_index: new_params.permutation.node_index + 1,
                    output: new_params.permutation.output,
                    permutation_index: new_params.permutation.permutation_index,
                    processor_pool: new_params.permutation.processor_pool,
                    processors: new_params.permutation.processors,
                    original_sample_rate: new_params.permutation.original_sample_rate,
                },
                sample_length: new_params.sample_length,
                samples: new_params.samples,
                spec: new_params.spec,
                update_sender: new_params.update_sender,
            }
        })
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
        PermuteNodeName::SampleRateConversionHigh => String::from("Sample rate conversion high"),
        PermuteNodeName::SampleRateConversionOriginal => String::from("Sample rate conversion low"),
    }
}
