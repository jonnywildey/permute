use audio_info::AudioFileError;
use audio_info::AudioInfo;
use neon::prelude::*;
use permute::display_node::*;
use permute::permute_files::*;
use permute::process::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::sync::mpsc;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub struct SharedState {
    // permute file params
    pub output: String,
    pub error: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub all_processors: Vec<PermuteNodeName>,
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,

    pub update_sender: mpsc::Sender<PermuteUpdate>,
    pub processing: bool,
    pub permutation_outputs: Vec<OutputProgress>,
    pub files: Vec<AudioInfo>,
}

impl SharedState {
    pub fn init(update_sender: mpsc::Sender<PermuteUpdate>) -> Self {
        Self {
            // files: vec![],
            high_sample_rate: false,
            input_trail: 0.0,
            normalise_at_end: true,
            error: String::default(),
            output: String::default(),
            output_trail: 2.0,
            permutation_depth: 2,
            permutations: 3,
            processor_count: None,
            update_sender,
            processor_pool: ALL_PROCESSORS.to_vec(),
            all_processors: ALL_PROCESSORS.to_vec(),
            processing: false,
            permutation_outputs: vec![],
            files: vec![],
        }
    }

    fn to_permute_params(&self) -> PermuteFilesParams {
        PermuteFilesParams {
            files: self.files.iter().map(|ai| ai.path.clone()).collect(),
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            normalise_at_end: self.normalise_at_end,
            output: self.output.clone(),
            output_trail: self.output_trail,
            permutation_depth: self.permutation_depth,
            permutations: self.permutations,
            processor_count: match self.permutation_depth {
                0 => Some(1),
                _ => None,
            },
            processor_pool: self.processor_pool.clone(),
            update_sender: self.update_sender.to_owned(),
        }
    }

    pub fn add_file(&mut self, file: String) -> Result<(), AudioFileError> {
        if self.files.iter().any(|f| f.path == file) {
            return Ok(());
        }
        let mut audio_info = AudioInfo::default();
        audio_info.update_file(file)?;
        self.files.push(audio_info);
        Ok(())
    }

    pub fn remove_file(&mut self, file: String) {
        // self.files.retain(|f| *f != file);
        self.files.retain(|f| f.path != file);
    }

    pub fn add_processor(&mut self, name: String) {
        let processor = get_processor_from_display_name(&name).unwrap();
        if self.processor_pool.iter().all(|p| *p != processor) {
            let _ = &self.processor_pool.push(processor);
        }
    }

    pub fn remove_processor(&mut self, name: String) {
        let processor = get_processor_from_display_name(&name).unwrap();
        self.processor_pool.retain(|p| *p != processor);
    }

    pub fn set_output(&mut self, output: String) {
        self.output = output;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = error;
    }

    pub fn add_output_progress(
        &mut self,
        permutation: Permutation,
        processors: Vec<PermuteNodeName>,
    ) {
        let path = permutation.output.clone();
        let _ = &self.permutation_outputs.push(OutputProgress {
            output: path,
            permutation: permutation.clone(),
            processors,
            progress: 0,
            audio_info: AudioInfo::default(),
        });
    }

    pub fn update_output_progress(&mut self, permutation: Permutation) {
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

    pub fn set_finished(&mut self) -> Result<(), AudioFileError> {
        self.processing = false;

        for permutation_output in self.permutation_outputs.iter_mut() {
            permutation_output
                .audio_info
                .update_file(permutation_output.output.clone())?;
        }
        Ok(())
    }

    pub fn set_normalised(&mut self, normalised: bool) {
        self.normalise_at_end = normalised;
    }

    pub fn set_input_trail(&mut self, trail: f64) {
        self.input_trail = trail;
    }

    pub fn set_output_trail(&mut self, trail: f64) {
        self.output_trail = trail;
    }

    pub fn set_permutations(&mut self, permutations: usize) {
        self.permutations = permutations;
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.permutation_depth = depth;
    }

    pub fn run_process(&mut self) -> JoinHandle<()> {
        self.permutation_outputs = vec![];
        self.processing = true;
        self.error = String::default();
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
    pub audio_info: AudioInfo,
}

impl Finalize for OutputProgress {}

impl SharedState {
    pub fn write_to_json(&self, path: String) -> std::io::Result<()> {
        let data = SharedStateSerializable {
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
        };
        let json = serde_json::to_string(&data)?;
        let mut file = File::create(path)?;
        file.write(json.as_bytes())?;
        Ok(())
    }

    pub fn read_from_json(&mut self, path: String) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: SharedStateSerializable = serde_json::from_reader(reader)?;

        self.files = data.files;
        self.high_sample_rate = data.high_sample_rate;
        self.input_trail = data.input_trail;
        self.normalise_at_end = data.normalise_at_end;
        self.output = data.output;
        self.output_trail = data.output_trail;
        self.permutation_depth = data.permutation_depth;
        self.permutations = data.permutations;
        self.processor_count = data.processor_count;
        self.processor_pool = data.processor_pool;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SharedStateSerializable {
    pub files: Vec<AudioInfo>,
    pub output: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,
}
