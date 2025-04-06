use crate::{
    files::*, permute_error::PermuteError, 
    process::*, 
    random_process::*, 
    audio_cache::AUDIO_CACHE,
};
use rand::thread_rng;
use sndfile::*;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use audio_info::AudioInfo;
use rayon::prelude::*;
use crossbeam_channel::{Sender, Receiver};
use std::sync::Mutex;

pub enum PermuteUpdate {
    Error(String),
    UpdatePermuteNodeStarted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdatePermuteNodeCompleted(Permutation, PermuteNodeName, PermuteNodeEvent),
    UpdateSetProcessors(Permutation, Vec<(PermuteNodeName, Vec<ProcessorAttribute>)>),
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
    pub max_stretch: f64,
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


// permute_file is used to generate a list of processor plans and parameters, then run them for each output file
// It is called once for each file in the input list
fn permute_file(
    params: &PermuteFilesParams,
    file: String,
) -> Result<Vec<Permutation>, PermuteError> {
    // Open the file and get metadata
    let snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(file.clone())?;
    let sample_rate = snd.get_samplerate();
    let channels = snd.get_channels();
    let sub_format = snd.get_subtype_format();
    let file_format = match params.output_file_as_wav {
        true => MajorFormat::WAV,
        false => snd.get_major_format(),
    };
    let endian = snd.get_endian();
    // Get samples. Either from audio cache or from file
    let samples_64 = AUDIO_CACHE.get_samples(&file)?;

    // Create input and output buffers
    let input_trail_buffer =
        vec![0_f64; (sample_rate as f64 * params.input_trail * channels as f64).ceil() as usize];
    let output_trail_buffer =
        vec![0_f64; (sample_rate as f64 * params.output_trail * channels as f64).ceil() as usize];
    let samples_64 = [input_trail_buffer, samples_64.to_vec(), output_trail_buffer].concat();
    let sample_length = samples_64.len();

    // set output directory
    let output = match params.create_subdirectories {
        true => {
            let o = get_output_run(params.output.clone());
            o.expect("error creating subdirectory")
        }
        false => params.output.clone(),
    };
    
    // Generate ordered list of processor plans and parameters for each output file
    // Each file will have a different ordered list of processor plans
    let mut outputs_processor_plans: Vec<(String, ProcessorParams, Vec<ProcessorPlan>)> = vec![];
    for i in 1..=params.permutations {
        let output_i = generate_file_name(file.clone(), output.clone(), i, params.output_file_as_wav);

        // Generate a random ordered list of processors
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
        let mut processor_plans: Vec<ProcessorPlan> = vec![];
        let mut node_index = 0;
        let mut last_params: ProcessorParams = ProcessorParams::default();
        // set the processor names so we have an ok idea of overall progress. 
        // Length of this vec is used to determine overall progress
        last_params.permutation.processors = processors.iter()
        .map(|p| PermutationProcessor {
            name: p.clone(),
            attributes: vec![],
        })
        .collect::<Vec<PermutationProcessor>>();
        for processor in processors.iter() {
            let processor_plan_gen = get_processor_plan(*processor);

            let mut processor_params = ProcessorParams {
                samples: samples_64.clone(),
                sample_length,
                channels,
                sample_rate,
                file_format,
                sub_format,
                endian,
                update_sender: params.update_sender.clone(),
                permutation: Permutation {
                    file: file.clone(),
                    permutation_index: i,
                    output: output_i.clone(),
                    processor_pool: params.processor_pool.clone(),
                    processors: last_params.permutation.processors.clone(),
                    original_sample_rate: sample_rate,
                    node_index: node_index,
                    files: params.files.clone(),
                },
            };
            let processor_plan = processor_plan_gen(&mut processor_params);
            // set the attributes for the processor now that we know them
            processor_params.permutation.processors[node_index].attributes = processor_plan.1.clone();
      
            processor_plans.push(processor_plan);
            last_params = processor_params;
            node_index += 1;
        }  

        // It is quite easy to get a list of processors that will increase the length of the audio way too much
        let (processor_plans, last_params) = filter_long_processes(processor_plans, last_params, params.max_stretch);

        params.update_sender.send(PermuteUpdate::UpdateSetProcessors(
            last_params.permutation.clone(),
            processor_plans.iter().map(|p| (p.0, p.1.clone())).collect(),
        ))?;

        outputs_processor_plans.push((output_i, last_params.clone(), processor_plans));
    }

    // Run each outputs processors
    let mut output_permutations: Vec<Permutation> = vec![];
    let update_sender = params.update_sender.clone();
    for (_output_i, processor_params, processor_plans) in outputs_processor_plans {
        let output_params = run_processors(RunProcessorsParams {
            processor_params: processor_params.clone(),
            processor_plans,
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
            update_sender.send(PermuteUpdate::AudioInfoGenerated(
                output_params.permutation.output.clone(),
                audio_info,
            ))?;
        }
    }
    Ok(output_permutations)
}


// process_file is used to run a single processor on a file e.g. Reverse or Trim
#[allow(dead_code)]
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

    // create processor plan
    let mut params = ProcessorParams {
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
    };
    let process_plan_fn = get_processor_plan(process);
    let process_plan = process_plan_fn(&mut params);

    // run processor plan
    let output_params = run_processors(RunProcessorsParams {
        processor_params: params,
        processor_plans: vec![process_plan],
    })?;

    
    let mut snd = sndfile::OpenOptions::WriteOnly(WriteOptions::new(
        file_format,
        sub_format,
        endian,
        sample_rate,
        channels,
    ))
    .from_path(file)?;

   snd.write_from_iter(output_params.samples.clone().into_iter())?;

    update_sender
        .send(PermuteUpdate::ProcessComplete(Some(vec![output_params.permutation])))
        .expect("Error sending message");
    Ok(())
}

pub struct RunProcessorsParams {
    pub processor_plans: Vec<ProcessorPlan>,
    pub processor_params: ProcessorParams,
}

pub fn run_processors(params: RunProcessorsParams) -> Result<ProcessorParams, PermuteError> {
    let mut processor_params = params.processor_params.clone();
    processor_params.permutation.node_index = 0;
    for processor in params.processor_plans.into_iter() {
        let (_name, _attributes, closure) = processor;
        processor_params = closure(processor_params)?;
        processor_params.permutation.node_index += 1;
    }

    Ok(processor_params)
}


fn filter_long_processes(processors: Vec<ProcessorPlan>, mut last_params: ProcessorParams, max_stretch: f64) -> (Vec<ProcessorPlan>, ProcessorParams) {
    // First pass: calculate length factors and mark processors to keep
    let mut filtered_processors = Vec::new();
    let mut filtered_processor_info = Vec::new();
    let mut cumulative_stretch = 1.0; // Start at 1 since it's multiplicative

    for (i, (name, attributes, closure)) in processors.into_iter().enumerate() {
        let mut processor_stretch = 1.0;
        
        // Find stretch/length factor in attributes
        for attr in &attributes {
            match attr.key.as_str() {
                "Length Factor" | "Stretch Factor" => {
                    if let Ok(factor) = attr.value.parse::<f64>() {
                        processor_stretch = factor;
                        break;
                    }
                },
                _ => {}
            }
        }

        // Calculate new cumulative stretch
        let new_cumulative = cumulative_stretch * processor_stretch;
        
        // Only keep processor if it doesn't exceed max_stretch
        if new_cumulative <= max_stretch {
            cumulative_stretch = new_cumulative;
            filtered_processors.push((name, attributes.clone(), closure));
            filtered_processor_info.push(last_params.permutation.processors[i].clone());
        } else {
            println!("Filtering out processor {:?} as it would increase stretch to {}", name, new_cumulative);
        }
    }

    // Update last_params with filtered processors
    last_params.permutation.processors = filtered_processor_info;
    last_params.permutation.node_index = 0;

    (filtered_processors, last_params)
}