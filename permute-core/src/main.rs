mod permute_files;
mod process;
mod random_process;

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
}

fn main() {
    let args = PermuteArgs::from_args();
    println!(
        "Permuting {} to {}, {} mutations",
        args.file, args.output, args.permutations
    );

    let processor_pool: Vec<PermuteNodeName> = vec![
        // PermuteNodeName::Reverse,
        PermuteNodeName::MetallicDelay,
        PermuteNodeName::RhythmicDelay,
        // PermuteNodeName::HalfSpeed,
        // PermuteNodeName::DoubleSpeed,
        // PermuteNodeName::Wow,
        // PermuteNodeName::Flutter,
        // PermuteNodeName::Chorus,
    ];

    permute_files(PermuteFilesParams {
        files: vec![args.file],
        output: args.output,
        input_trail: args.input_trail,
        output_trail: args.output_trail,
        permutations: args.permutations,
        permutation_depth: args.permutation_depth,
        processor_pool: processor_pool,

        update_permute_node_progress,
        update_set_processors,
    });
}

fn update_permute_node_progress(
    permutation: Permutation,
    _: PermuteNodeName,
    event: PermuteNodeEvent,
) {
    match event {
        PermuteNodeEvent::NodeProcessStarted => {}
        PermuteNodeEvent::NodeProcessComplete => {
            let percentage_progress: f64 = ((permutation.node_index as f64 + 1.0)
                / permutation.processors.len() as f64)
                * 100.0;
            println!("{}%", percentage_progress.round())
        }
    }
}

fn update_set_processors(permutation: Permutation, processors: Vec<PermuteNodeName>) {
    let pretty_processors = processors
        .iter()
        .map(|p| get_processor_display_name(*p))
        .collect::<Vec<String>>();
    println!(
        "Permutating {} Processors {:#?}",
        permutation.output, pretty_processors
    );
}
