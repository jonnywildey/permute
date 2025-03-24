use audio_info::AudioFileError;
use audio_info::AudioInfo;
use neon::prelude::*;
use permute::display_node::*;
use permute::permute_error::PermuteError;
use permute::permute_files::*;
use permute::process::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::fs;
use crossbeam_channel::{Sender, Receiver};
use std::sync::Mutex;
use std::thread;
use std::collections::HashMap;

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
    pub trim_all: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,
    pub constrain_length: bool,
    pub create_subdirectories: bool,

    pub update_sender: Arc<Sender<PermuteUpdate>>,
    pub processing: bool,
    // Store outputs by (filename, permutation_index)
    outputs: HashMap<(String, usize), OutputProgress>,
    pub files: Vec<AudioInfo>,
    pub cancel_sender: Sender<()>,
}

impl SharedState {
    pub fn init(update_sender: Sender<PermuteUpdate>) -> Self {
        let (cancel_sender, _) = crossbeam_channel::bounded(1);
        Self {
            high_sample_rate: false,
            input_trail: 0.0,
            normalise_at_end: true,
            trim_all: false,
            error: String::default(),
            output: String::default(),
            output_trail: 2.0,
            permutation_depth: 2,
            permutations: 3,
            processor_count: Some(std::thread::available_parallelism()
                .map(|n| std::cmp::min(n.get(), 4) as i32)
                .unwrap_or(1)),
            update_sender: Arc::new(update_sender),
            processor_pool: ALL_PROCESSORS.to_vec(),
            all_processors: ALL_PROCESSORS.to_vec(),
            processing: false,
            outputs: HashMap::new(),
            files: vec![],
            cancel_sender,
            constrain_length: true,
            create_subdirectories: true,
        }
    }

    fn to_permute_params(&mut self) -> PermuteFilesParams {
        let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
        // Store the new sender for future cancellation
        self.cancel_sender = cancel_sender;
        
        PermuteFilesParams {
            files: self.files.iter().map(|ai| ai.path.clone()).collect(),
            constrain_length: self.constrain_length,
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            normalise_at_end: self.normalise_at_end,
            trim_all: self.trim_all,
            output: self.output.clone(),
            output_trail: self.output_trail,
            permutation_depth: self.permutation_depth,
            permutations: self.permutations,
            processor_count: self.processor_count,
            processor_pool: self.processor_pool.clone(),
            output_file_as_wav: true,
            update_sender: self.update_sender.clone(),
            create_subdirectories: self.create_subdirectories,
            cancel_receiver: Arc::new(cancel_receiver),
        }
    }

    pub fn add_file(&mut self, file: String) -> Result<(), AudioFileError> {
        if self.files.iter().any(|f| f.path == file) {
            return Ok(());
        }
        let mut audio_info = AudioInfo::default();
        audio_info.update_file(file.clone())?;
        
        // Pre-create output progress entries for all permutations
        for i in 1..=self.permutations {
            self.outputs.insert((file.clone(), i), OutputProgress {
                output: String::new(), // Will be set when actually processing
                progress: 0,
                permutation: Permutation {
                    file: file.clone(),
                    permutation_index: i,
                    output: String::new(),
                    processor_pool: vec![],
                    processors: vec![],
                    original_sample_rate: 0,
                    node_index: 0,
                    files: vec![],
                },
                processors: vec![],
                audio_info: AudioInfo::default(),
                deleted: false,
            });
        }

        self.files.push(audio_info);
        Ok(())
    }

    pub fn remove_file(&mut self, file: String) {
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

    pub fn get_ordered_outputs(&self) -> Vec<OutputProgress> {
        let mut ordered = Vec::new();
        // For each input file
        for file in &self.files {
            // For each permutation number
            for i in 1..=self.permutations {
                if let Some(output) = self.outputs.get(&(file.path.clone(), i)) {
                    ordered.push(output.clone());
                }
            }
        }
        ordered
    }

    pub fn add_output_progress(
        &mut self,
        permutation: Permutation,
        processors: Vec<PermuteNodeName>,
    ) {
        let key = (permutation.file.clone(), permutation.permutation_index);
        self.outputs.insert(key, OutputProgress {
            output: permutation.output.clone(),
            permutation: permutation.clone(),
            processors,
            progress: 0,
            audio_info: AudioInfo::default(),
            deleted: false,
        });
    }

    pub fn update_output_progress(&mut self, permutation: Permutation) {
        let percentage_progress: f64 =
            ((permutation.node_index as f64 + 1.0) / permutation.processors.len() as f64) * 100.0;

        let key = (permutation.file.clone(), permutation.permutation_index);
        if let Some(output) = self.outputs.get_mut(&key) {
            output.progress = percentage_progress as i32;
            output.permutation = permutation.clone();
        }
    }

    pub fn set_finished(&mut self) -> Result<(), AudioFileError> {
        self.processing = false;
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.processing = false;
        self.error = "Processing cancelled by user".to_string();
        self.processing = false;
        let _ = self.cancel_sender.send(());
    }

    pub fn set_normalised(&mut self, normalised: bool) {
        self.normalise_at_end = normalised;
    }

    pub fn set_trim_all(&mut self, trim_all: bool) {
        self.trim_all = trim_all;
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

    pub fn reverse_file(&mut self, file: String) -> Result<(), PermuteError> {
        self.processing = true;
        let search_file = file.clone();
        let update_sender = Arc::try_unwrap(self.update_sender.clone())
            .unwrap_or_else(|arc| (*arc).clone());
        process_file(file.clone(), PermuteNodeName::Reverse, update_sender)?;

        self.processing = false;
        if let Some(output) = self.outputs.get_mut(&(file, 1)) {
            let mut ai = output.audio_info.clone();
            ai.update_file(search_file).unwrap();
        }
        Ok(())
    }

    pub fn trim_file(&mut self, file: String) -> Result<(), PermuteError> {
        self.processing = true;
        let search_file = file.clone();
        let update_sender = Arc::try_unwrap(self.update_sender.clone())
            .unwrap_or_else(|arc| (*arc).clone());
        process_file(file.clone(), PermuteNodeName::Trim, update_sender)?;
        
        self.processing = false;
        if let Some(output) = self.outputs.get_mut(&(file, 1)) {
            let mut ai = output.audio_info.clone();
            ai.update_file(search_file).unwrap();
        }
        Ok(())
    }

    pub fn run_process(&mut self) -> JoinHandle<()> {
        if self.processing {
            return thread::spawn(|| {});
        }
        self.processing = true;
        let params = self.to_permute_params();
        
        // Spawn a thread for the actual processing
        let state = Arc::new(Mutex::new(self.clone()));
        thread::spawn(move || {
            // Get the handle from permute_files and wait for it
            let handle = permute_files(params);
            // Wait for the processing to complete
            match handle.join() {
                Ok(_) => {
                    if let Ok(mut state) = state.lock() {
                        let _ = state.set_finished();
                    }
                }
                Err(e) => {
                    if let Ok(mut state) = state.lock() {
                        state.set_error(format!("Processing thread panicked: {:?}", e));
                        let _ = state.set_finished();
                    }
                }
            }
        })
    }

    pub fn delete_output_file(&mut self, file: String) -> Result<(), std::io::Error> {
        fs::remove_file(&file)?;
        if let Some(output) = self.outputs.get_mut(&(file.clone(), 1)) {
            output.deleted = true;
        }
        Ok(())
    }

    pub fn delete_all_output_files(&mut self) -> Result<(), std::io::Error> {
        let mut to_delete = Vec::new();
        let mut last_output_path = String::new();
        
        for ((file, _), output) in self.outputs.iter() {
            if let Err(e) = fs::remove_file(&output.output) {
                println!("Error deleting file {}: {}", output.output, e);
            }
            last_output_path = output.output.clone();
            to_delete.push(file.clone());
        }
        
        for file in to_delete {
            for i in 1..=self.permutations {
                self.outputs.get_mut(&(file.clone(), i)).map(|o| o.deleted = true);
            }
        }

        if self.create_subdirectories && !last_output_path.is_empty() {
            if let Some(dir) = Path::new(&last_output_path).parent() {
                if dir.is_dir() && dir.read_dir().unwrap().next().is_none() {
                    fs::remove_dir(dir)?;
                }
            }
        }
        Ok(())
    }

    pub fn set_create_subdirectories(&mut self, create: bool) {
        self.create_subdirectories = create;
    }

    pub fn select_all_processors(&mut self) {
        self.processor_pool = self.all_processors.clone();
    }

    pub fn deselect_all_processors(&mut self) {
        self.processor_pool.clear();
    }

    pub fn update_output_audioinfo(&mut self, file: String, audio_info: AudioInfo) {
        // Find the output entry that matches this output file path
        for ((_, _), output) in self.outputs.iter_mut() {
            if output.output == file {
                output.audio_info = audio_info.clone();
                output.permutation.output = file.clone();
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.outputs.clear();
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
    pub deleted: bool,
}

impl Finalize for OutputProgress {}

impl SharedState {
    pub fn write_to_json(&self, path: String) -> std::io::Result<()> {
        let data = SharedStateSerializable {
            files: self.files.clone(),
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            normalise_at_end: self.normalise_at_end,
            trim_all: self.trim_all,
            output: self.output.clone(),
            output_trail: self.output_trail,
            permutation_depth: self.permutation_depth,
            permutations: self.permutations,
            processor_count: self.processor_count,
            processor_pool: self.processor_pool.clone(),
            create_subdirectories: self.create_subdirectories,
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
        self.trim_all = data.trim_all;
        self.output = data.output;
        self.output_trail = data.output_trail;
        self.permutation_depth = data.permutation_depth;
        self.permutations = data.permutations;
        self.processor_count = data.processor_count;
        self.processor_pool = data.processor_pool;
        self.create_subdirectories = data.create_subdirectories;
        
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
    pub trim_all: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,
    pub create_subdirectories: bool,
}
