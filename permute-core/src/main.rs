mod permute_files;
mod process;
mod random_process;

use std::{sync::mpsc, thread};

use permute_files::*;
use process::*;
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
    /// Number of times to randomly process file
    #[structopt(long, short)]
    permutations: usize,
    /// How much the file is permuted. Numbers larger than 5 will take a long time to process
    #[structopt(long = "depth", short, default_value = "1")]
    permutation_depth: usize,
    /// Whether to normalise at end
    #[structopt(long)]
    normalise: bool,
    /// Whether to run fx at a high sample rate
    #[structopt(long = "highSampleRate")]
    high_sample_rate: bool,
    /// How many processes to pick from per depth. If not included a random value from 2-5 will be used
    #[structopt(long = "processor", default_value = "0")]
    processor_count: i32,
}

fn main() {
    let args = PermuteArgs::from_args();
    println!(
        "Permuting {} to {}, {} mutations",
        args.file, args.output, args.permutations
    );

    let processor_pool: Vec<PermuteNodeName> = vec![
        PermuteNodeName::Reverse,
        PermuteNodeName::MetallicDelay,
        PermuteNodeName::RhythmicDelay,
        PermuteNodeName::HalfSpeed,
        PermuteNodeName::DoubleSpeed,
        PermuteNodeName::Wow,
        PermuteNodeName::Flutter,
        PermuteNodeName::Chorus,
    ];

    let processor_count: Option<i32> = match args.processor_count {
        0 => None,
        _ => Some(args.processor_count),
    };

    let (tx, rx) = mpsc::channel::<PermuteUpdate>();

    let handle = thread::spawn(move || {
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
            PermuteUpdate::UpdatePermuteNodeStarted(_, _, _) => {}
            PermuteUpdate::UpdateSetProcessors(permutation, processors) => {
                let pretty_processors = processors
                    .iter()
                    .map(|p| get_processor_display_name(*p))
                    .collect::<Vec<String>>();
                println!(
                    "Permutating {} Processors {:#?}",
                    permutation.output, pretty_processors
                );
            }
        }
    }
}
