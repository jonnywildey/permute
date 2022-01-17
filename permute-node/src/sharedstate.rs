use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use neon::prelude::*;
use permute::permute_files::*;
use permute::process::*;

#[derive(Debug, Clone)]
pub struct SharedState {
    // permute file params
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
    pub finished: bool,
    pub permutation_outputs: Vec<OutputProgress>,
}

impl SharedState {
    pub fn init(update_sender: mpsc::Sender<PermuteUpdate>) -> Self {
        Self {
            files: vec![],
            high_sample_rate: false,
            input_trail: 2.0,
            normalise_at_end: true,
            output: String::from(
                "/Users/jonnywildey/rustcode/permute/permute-core/renders/vibebeepui.wav",
            ),
            output_trail: 0.0,
            permutation_depth: 1,
            permutations: 3,
            processor_count: None,
            update_sender,
            processor_pool: vec![
                PermuteNodeName::Reverse,
                PermuteNodeName::MetallicDelay,
                PermuteNodeName::RhythmicDelay,
                PermuteNodeName::HalfSpeed,
                PermuteNodeName::DoubleSpeed,
                PermuteNodeName::Wow,
                PermuteNodeName::Flutter,
                PermuteNodeName::Chorus,
            ],
            finished: false,
            permutation_outputs: vec![],
        }
    }

    fn to_permute_params(&self) -> PermuteFilesParams {
        PermuteFilesParams {
            files: self.files.clone(),
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            normalise_at_end: self.normalise_at_end,
            output: self.output.clone(),
            output_trail: self.output_trail,
            permutation_depth: self.permutation_depth,
            permutations: self.permutations,
            processor_count: self.processor_count,
            processor_pool: self.processor_pool.clone(),
            update_sender: self.update_sender.to_owned(),
        }
    }

    pub fn add_file(&mut self, file: String) {
        let _ = &self.files.push(file);
    }

    pub fn add_output_progress(
        &mut self,
        permutation: Permutation,
        processors: Vec<PermuteNodeName>,
    ) {
        let output = permutation.output.clone();
        let _ = &self.permutation_outputs.push(OutputProgress {
            output,
            permutation: permutation.clone(),
            processors,
            progress: 0,
        });
    }

    pub fn update_output_progress(&mut self, permutation: Permutation) {
        println!("here");
        let percentage_progress: f64 =
            ((permutation.node_index as f64 + 1.0) / permutation.processors.len() as f64) * 100.0;

        let op = self
            .permutation_outputs
            .iter_mut()
            .find(|op| op.output == permutation.output);
        match op {
            Some(op) => {
                op.progress = percentage_progress as i32;
                op.permutation = permutation.clone();
            }
            None => {}
        }
    }

    pub fn set_finished(&mut self) {
        self.finished = true;
    }

    pub fn run_process(&mut self) -> JoinHandle<()> {
        self.permutation_outputs = vec![];
        let permute_params = Self::to_permute_params(&self);

        permute_files(permute_params)
    }
}

impl Finalize for SharedState {}

#[derive(Debug, Clone)]
pub struct OutputProgress {
    pub output: String,
    pub progress: i32,
    pub permutation: Permutation,
    pub processors: Vec<PermuteNodeName>,
}

impl Finalize for OutputProgress {}
