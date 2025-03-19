mod display_node;
mod files;
mod osc;
mod permute_error;
mod permute_files;
mod process;
mod random_process;

use std::{sync::mpsc, thread};

use display_node::*;
use permute_files::*;
use structopt::StructOpt;

use crate::process::PermuteNodeName;

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
    #[structopt(long = "outputAsWav")]
    output_file_as_wav: bool,
    /// Number of times to randomly process file
    #[structopt(long, short)]
    permutations: usize,
    /// How much the file is permuted. Numbers larger than 5 will take a long time to process
    #[structopt(long = "depth", short = "d", default_value = "1")]
    permutation_depth: usize,
    /// Whether to normalise at end
    #[structopt(long)]
    normalise: bool,
    /// Whether to trim at end
    #[structopt(long = "trimAll")]
    trim_all: bool,
    /// Store new permutations in a subdirectory. Avoids overwrites
    #[structopt(long = "createSubdirectories")]
    create_subdirectories: bool,
    /// Whether to run fx at a high sample rate
    #[structopt(long = "highSampleRate")]
    high_sample_rate: bool,
    /// How many processes to pick from per depth. If not included a random value from 2-5 will be used
    #[structopt(long = "processorCount", default_value = "0")]
    processor_count: i32,
    /// Run audio through a specific process
    #[structopt(long = "processor", default_value = "")]
    processor: String,
}

fn main() {
    let args = PermuteArgs::from_args();

    let processor_pool: Vec<PermuteNodeName> = match args.processor.as_str() {
        "" => vec![
            PermuteNodeName::GranularTimeStretch,
            PermuteNodeName::Reverse,
            PermuteNodeName::MetallicDelay,
            PermuteNodeName::RhythmicDelay,
            PermuteNodeName::HalfSpeed,
            PermuteNodeName::DoubleSpeed,
            PermuteNodeName::RandomPitch,
            PermuteNodeName::Wow,
            PermuteNodeName::Flutter,
            PermuteNodeName::Chorus,
            PermuteNodeName::Phaser,
            PermuteNodeName::Flange,
            PermuteNodeName::Filter,
            PermuteNodeName::Lazer,
            PermuteNodeName::LineFilter,
            PermuteNodeName::OscillatingFilter,
        ],
        str => vec![get_processor_from_display_name(str).expect("Processor not found")],
    };

    let processor_count: Option<i32> = match args.processor_count {
        0 => None,
        _ => Some(args.processor_count),
    };

    let (tx, rx) = mpsc::channel::<PermuteUpdate>();

    println!(
        "Permuting {} to {}, {} mutations",
        args.file, args.output, args.permutations
    );

    thread::spawn(move || {
        permute_files(PermuteFilesParams {
            files: vec![args.file],
            output: args.output,
            input_trail: args.input_trail,
            output_trail: args.output_trail,
            permutations: args.permutations,
            permutation_depth: args.permutation_depth,
            processor_pool: processor_pool,
            high_sample_rate: args.high_sample_rate,
            normalise_at_end: args.normalise,
            trim_all: args.trim_all,
            create_subdirectories: args.create_subdirectories,
            output_file_as_wav: args.output_file_as_wav,
            update_sender: tx,
            processor_count,
        });
    });

    while let Ok(message) = rx.recv() {
        match message {
            PermuteUpdate::UpdatePermuteNodeCompleted(permutation, _, _) => {
                let percentage_progress: f64 = ((permutation.node_index as f64 + 1.0)
                    / permutation.processors.len() as f64)
                    * 100.0;
                println!("{}%", percentage_progress.round());
            }
            PermuteUpdate::UpdatePermuteNodeStarted(permutation, _, _) => {
                if permutation.node_index == 0 {
                    println!("Permuting {}", permutation.output);
                }
            }
            PermuteUpdate::UpdateSetProcessors(permutation, processors) => {
                let pretty_processors = processors
                    .iter()
                    .map(|p| get_processor_display_name(*p))
                    .collect::<Vec<String>>();
                println!(
                    "File {} Processors {:#?}",
                    permutation.output, pretty_processors
                );
            }
            PermuteUpdate::Error(err) => {
                panic!("{}", err);
            }
            PermuteUpdate::ProcessComplete => {}
        }
    }
}
