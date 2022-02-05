use crate::permute_error::PermuteError;
use crate::process::*;
use crate::random_process::*;
use sndfile::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub enum PermuteUpdate {
    Error(String),
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
    thread::Builder::new()
        .name("PermuteThread".to_string())
        .spawn(|| {
            let copied_params = params.clone();
            let files = params.files;
            for i in 0..files.len() {
                permute_file(copied_params.clone(), files[i].clone())
                    .map_err(|err| {
                        params
                            .update_sender
                            .send(PermuteUpdate::Error(err.to_string()))
                            .expect("Error sending message");
                    })
                    .unwrap();
            }
            params
                .update_sender
                .send(PermuteUpdate::ProcessComplete)
                .expect("Error sending message");
        })
        .expect("Error creating thread")
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
) -> Result<(), PermuteError> {
    let mut snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(file.clone())?;
    let sample_rate = snd.get_samplerate();
    let channels = snd.get_channels();
    let file_format = snd.get_subtype_format();
    let samples_64: Vec<f64> = snd.read_all_to_vec()?;
    let endian = snd.get_endian();

    let input_trail_buffer =
        vec![0_f64; (sample_rate as f64 * input_trail * channels as f64).ceil() as usize];
    let output_trail_buffer =
        vec![0_f64; (sample_rate as f64 * output_trail * channels as f64).ceil() as usize];
    let samples_64 = [input_trail_buffer, samples_64, output_trail_buffer].concat();

    let sample_length = samples_64.len();

    let mut generated_processors: Vec<(Vec<ProcessorFn>, ProcessorParams)> = vec![];

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
            output: output_i,
            processor_pool: processor_pool.clone(),
            processors: processors.clone(),
            original_sample_rate: sample_rate,
            node_index: 0,
        };
        let processor_params = ProcessorParams {
            samples: samples_64.clone(),
            sample_length,
            permutation: permutation.clone(),
            channels,
            sample_rate,
            file_format,
            endian,
            update_sender: update_sender.to_owned(),
        };
        let _ = update_sender.send(PermuteUpdate::UpdateSetProcessors(
            permutation.clone(),
            processors,
        ));

        generated_processors.push((processor_fns, processor_params));
    }

    for (processor_fns, processor_params) in generated_processors.iter() {
        let output_params = run_processors(RunProcessorsParams {
            processor_params: processor_params.clone(),
            processors: processor_fns.to_vec(),
        })?;
        let mut snd = sndfile::OpenOptions::WriteOnly(WriteOptions::new(
            MajorFormat::WAV,
            output_params.file_format,
            output_params.endian,
            output_params.sample_rate,
            output_params.channels,
        ))
        .from_path(output_params.permutation.output.clone())?;

        snd.write_from_iter(output_params.samples.clone().into_iter())?;
    }
    Ok(())
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
) -> Result<ProcessorParams, PermuteError> {
    processors
        .iter()
        .fold(Ok(processor_params), |params, processor| {
            let new_params = processor(&params?)?;
            Ok(ProcessorParams {
                permutation: Permutation {
                    node_index: new_params.permutation.node_index + 1,
                    ..new_params.permutation
                },
                ..new_params
            })
        })
}
