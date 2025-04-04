use crate::{
    files::*, permute_error::PermuteError, 
    process::*, 
    random_process::*, 
    audio_cache::AUDIO_CACHE
};
use rand::thread_rng;
use sndfile::*;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use audio_info::AudioInfo;
use rayon::prelude::*;
use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;
use std::sync::Mutex;

pub enum PermuteUpdate {
    Error(String),
    UpdatePermuteNodeStarted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdatePermuteNodeCompleted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdateSetProcessors(Permutation, Vec<PermuteNodeName>),
    ProcessComplete(Option<Vec<Permutation>>),
    AudioInfoGenerated(String, AudioInfo),
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
    pub trim_all: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,
    pub output_file_as_wav: bool,
    pub update_sender: Arc<Sender<PermuteUpdate>>,
    pub create_subdirectories: bool,
    pub cancel_receiver: Arc<Receiver<()>>,
    pub constrain_length: bool,
}

pub fn permute_files(mut params: PermuteFilesParams) -> JoinHandle<()> {
    thread::Builder::new()
    .name("PermuteThread".to_string())
    .spawn(move || {
        let output = match params.create_subdirectories {
            true => {
                let o = get_output_run(params.output.clone());
                o.expect("error creating subdirectory")
            }
            false => params.output.clone(),
        };
        params.output = output;
        params.create_subdirectories = false;
        let output_permutations = Arc::new(Mutex::new(Vec::new()));
        
        // Process files in parallel using rayon
        params.files.par_iter().for_each(|file| {
            if params.cancel_receiver.try_recv().is_ok() {
                params.update_sender.send(PermuteUpdate::ProcessComplete(None))
                    .expect("Error sending message");
                return;
            }

            let result = permute_file(&params, file.clone());
            match result {
                Ok(permutations) => {
                    output_permutations.lock().unwrap().extend(permutations);
                }
                Err(err) => {
                    params.update_sender.send(PermuteUpdate::Error(err.to_string()))
                        .expect("Error sending message");
                }
            }
        });

        params
            .update_sender
            .send(PermuteUpdate::ProcessComplete(Some(output_permutations.lock().unwrap().clone())))
            .expect("Error sending message");
    })
    .expect("Error creating thread")
}


fn permute_file(
    params: &PermuteFilesParams,
    file: String,
) -> Result<Vec<Permutation>, PermuteError> {
    let snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(file.clone())?;
    let sample_rate = snd.get_samplerate();
    let channels = snd.get_channels();
    let sub_format = snd.get_subtype_format();
    let file_format = match params.output_file_as_wav {
        true => MajorFormat::WAV,
        false => snd.get_major_format(),
    };
    let endian = snd.get_endian();

    let samples_64 = AUDIO_CACHE.get_samples(&file)?;

    let input_trail_buffer =
        vec![0_f64; (sample_rate as f64 * params.input_trail * channels as f64).ceil() as usize];
    let output_trail_buffer =
        vec![0_f64; (sample_rate as f64 * params.output_trail * channels as f64).ceil() as usize];
    let samples_64 = [input_trail_buffer, samples_64.to_vec(), output_trail_buffer].concat();

    let sample_length = samples_64.len();

    let mut generated_processors: Vec<(Vec<ProcessorFn>, ProcessorParams)> = vec![];

    let output = match params.create_subdirectories {
        true => {
            let o = get_output_run(params.output.clone());
            o.expect("error creating subdirectory")
        }
        false => params.output.clone(),
    };

    for i in 1..=params.permutations {
        let output_i = generate_file_name(file.clone(), output.clone(), i, params.output_file_as_wav);

        let processors = generate_processor_sequence(GetProcessorNodeParams { 
            depth: params.permutation_depth,
            normalise_at_end: params.normalise_at_end,
            trim_at_end: params.trim_all,
            processor_pool: params.processor_pool.clone(),
            high_sample_rate: params.high_sample_rate,
            processor_count: params.processor_count,
            constrain_length: params.constrain_length,
            rng: thread_rng(),
            original_depth: params.permutation_depth,
        });

        let processor_fns = processors
            .iter()
            .map(|n| get_processor_function(*n))
            .collect();

        // Create a map of existing processors and their attributes from the previous permutation
        let existing_processors: HashMap<PermuteNodeName, Vec<ProcessorAttribute>> = HashMap::new();

        let permutation_processors: Vec<PermutationProcessor> = processors.iter().map(|n| PermutationProcessor {
            name: n.clone(),
            attributes: existing_processors.get(n).cloned().unwrap_or_default(),
        }).collect();

        let permutation = Permutation {
            file: file.clone(),
            permutation_index: i,
            output: output_i,
            processor_pool: params.processor_pool.clone(),
            processors: permutation_processors,
            original_sample_rate: sample_rate,
            node_index: 0,
            files: params.files.clone(),
        };
        let processor_params = ProcessorParams {
            samples: samples_64.clone(),
            sample_length,
            permutation: permutation.clone(),
            channels,
            sample_rate,
            file_format,
            sub_format,
            endian,
            update_sender: params.update_sender.clone(),
        };
        let _ = params.update_sender.send(PermuteUpdate::UpdateSetProcessors(
            permutation.clone(),
            processors,
        ));

        generated_processors.push((processor_fns, processor_params));
    }

    let mut output_permutations: Vec<Permutation> = vec![];
    for (processor_fns, processor_params) in generated_processors.iter() {
        let output_params = run_processors(RunProcessorsParams {
            processor_params: processor_params.clone(),
            processors: processor_fns.to_vec(),
        })?;
        output_permutations.push(output_params.permutation.clone());
        let mut snd = sndfile::OpenOptions::WriteOnly(WriteOptions::new(
            output_params.file_format,
            output_params.sub_format,
            output_params.endian,
            output_params.sample_rate,
            output_params.channels,
        ))
        .from_path(output_params.permutation.output.clone())?;

        snd.write_from_iter(output_params.samples.clone().into_iter())?;

        // Generate audio info for the output file
        let mut audio_info = AudioInfo::default();
        if let Ok(()) = audio_info.update_file(output_params.permutation.output.clone()) {
            params.update_sender.send(PermuteUpdate::AudioInfoGenerated(
                output_params.permutation.output.clone(),
                audio_info,
            ))?;
        }
    }
    Ok(output_permutations)
}

pub fn process_file(
    file: String,
    process: PermuteNodeName,
    update_sender: Sender<PermuteUpdate>,
) -> Result<(), PermuteError> {
    let snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(file.clone())?;
    let sample_rate = snd.get_samplerate();
    let channels = snd.get_channels();
    let endian = snd.get_endian();
    let file_format = snd.get_major_format();
    let sub_format = snd.get_subtype_format();

    // Use audio cache to get samples
    let samples = AUDIO_CACHE.get_samples(&file)?;
    let sample_length = samples.len();

    let process_fn = get_processor_function(process);

    let new_params = process_fn(&mut ProcessorParams {
        channels,
        endian,
        file_format,
        sub_format,
        sample_length,
        samples: samples.to_vec(),
        sample_rate,
        update_sender: Arc::new(update_sender.clone()),
        permutation: Permutation {
            file: file.clone(),
            node_index: 0,
            original_sample_rate: sample_rate,
            output: file.clone(),
            permutation_index: 0,
            processor_pool: vec![process],
            processors: vec![PermutationProcessor {
                name: process,
                attributes: vec![],
            }],
            files: vec![file.clone()],
        },
    })?;

    
    let mut snd = sndfile::OpenOptions::WriteOnly(WriteOptions::new(
        file_format,
        sub_format,
        endian,
        sample_rate,
        channels,
    ))
    .from_path(file)?;

   snd.write_from_iter(new_params.samples.clone().into_iter())?;

    update_sender
        .send(PermuteUpdate::ProcessComplete(Some(vec![new_params.permutation])))
        .expect("Error sending message");
    Ok(())
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
            let new_params = processor(&mut params?)?;

            Ok(ProcessorParams {
                permutation: Permutation {
                    node_index: new_params.permutation.node_index + 1,
                    processors: new_params.permutation.processors.clone(),
                    ..new_params.permutation
                },
                ..new_params
            })
        })
}
