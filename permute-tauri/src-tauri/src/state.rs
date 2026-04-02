use audio_info::{AudioFileError, AudioInfo};
use crossbeam_channel::Sender;
use permute::{
    audio_cache::AUDIO_CACHE,
    display_node::{get_processor_display_name, get_processor_from_display_name},
    permute_error::PermuteError,
    permute_files::{permute_files, process_file, PermuteFilesParams, PermuteUpdate},
    process::{Permutation, PermuteNodeName, ProcessorAttribute, ALL_PROCESSORS},
    rms_cache::clear_file_from_rms_cache,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
    sync::Arc,
    thread::{self, JoinHandle},
};

// ─── App state wrapper ───────────────────────────────────────────────────────

pub struct AppState {
    pub shared: Arc<std::sync::Mutex<SharedState>>,
}

// ─── DTO structs (serialise to match TypeScript IPermuteState) ───────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermuteStateDto {
    pub output: String,
    pub error: String,
    pub processing: bool,
    pub high_sample_rate: bool,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: u32,
    pub permutation_depth: u32,
    pub processor_count: u32,
    pub processor_pool: Vec<String>,
    pub all_processors: Vec<String>,
    pub normalise_at_end: bool,
    pub trim_all: bool,
    pub create_subdirectories: bool,
    pub viewed_welcome: bool,
    pub max_stretch: f64,
    pub files: Vec<PermutationInputDto>,
    pub permutation_outputs: Vec<PermutationOutputDto>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermutationInputDto {
    pub path: String,
    pub name: String,
    pub duration_sec: f64,
    pub image: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermutationOutputDto {
    pub path: String,
    pub name: String,
    pub progress: i32,
    pub image: String,
    pub duration_sec: f64,
    pub deleted: bool,
    pub processors: Vec<ProcessorDto>,
}

#[derive(Serialize, Clone)]
pub struct ProcessorDto {
    pub name: String,
    pub attributes: Vec<ProcessorAttributeDto>,
}

#[derive(Serialize, Clone)]
pub struct ProcessorAttributeDto {
    pub key: String,
    pub value: String,
}

// ─── SharedState ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SharedState {
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
    pub viewed_welcome: bool,
    pub max_stretch: f64,
    pub update_sender: Arc<Sender<PermuteUpdate>>,
    pub processing: bool,
    outputs: HashMap<(usize, usize), OutputProgress>,
    pub files: Vec<AudioInfo>,
    pub cancel_sender: crossbeam_channel::Sender<()>,
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
            processor_count: Some(
                std::thread::available_parallelism()
                    .map(|n| std::cmp::min(n.get(), 4) as i32)
                    .unwrap_or(1),
            ),
            update_sender: Arc::new(update_sender),
            processor_pool: ALL_PROCESSORS.to_vec(),
            all_processors: ALL_PROCESSORS.to_vec(),
            processing: false,
            outputs: HashMap::new(),
            files: vec![],
            cancel_sender,
            constrain_length: true,
            create_subdirectories: true,
            viewed_welcome: false,
            max_stretch: 17.0,
        }
    }

    pub fn to_state_dto(&self) -> PermuteStateDto {
        let files = self
            .files
            .iter()
            .map(|f| PermutationInputDto {
                path: f.path.clone(),
                name: f.name.clone(),
                duration_sec: f.duration_sec,
                image: f.image.clone(),
            })
            .collect();

        let processor_pool = self
            .processor_pool
            .iter()
            .map(|p| get_processor_display_name(*p).to_string())
            .collect();

        let all_processors = self
            .all_processors
            .iter()
            .map(|p| get_processor_display_name(*p).to_string())
            .collect();

        let permutation_outputs = self
            .get_ordered_outputs()
            .into_iter()
            .map(|o| {
                let processors = o
                    .permutation
                    .processors
                    .iter()
                    .map(|p| ProcessorDto {
                        name: get_processor_display_name(p.name).to_string(),
                        attributes: p
                            .attributes
                            .iter()
                            .map(|a| ProcessorAttributeDto {
                                key: a.key.clone(),
                                value: a.value.clone(),
                            })
                            .collect(),
                    })
                    .collect();
                PermutationOutputDto {
                    path: o.output.clone(),
                    name: o.audio_info.name.clone(),
                    progress: o.progress,
                    image: o.audio_info.image.clone(),
                    duration_sec: o.audio_info.duration_sec,
                    deleted: o.deleted,
                    processors,
                }
            })
            .collect();

        PermuteStateDto {
            output: self.output.clone(),
            error: self.error.clone(),
            processing: self.processing,
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            output_trail: self.output_trail,
            permutations: self.permutations as u32,
            permutation_depth: self.permutation_depth as u32,
            processor_count: self.processor_count.unwrap_or(0) as u32,
            processor_pool,
            all_processors,
            normalise_at_end: self.normalise_at_end,
            trim_all: self.trim_all,
            create_subdirectories: self.create_subdirectories,
            viewed_welcome: self.viewed_welcome,
            max_stretch: self.max_stretch,
            files,
            permutation_outputs,
        }
    }

    pub fn clear_error(&mut self) {
        self.error = String::default();
    }

    pub fn set_error(&mut self, error: String) {
        self.error = error;
    }

    fn to_permute_params(&mut self) -> PermuteFilesParams {
        let (cancel_sender, cancel_receiver) = crossbeam_channel::bounded(1);
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
            max_stretch: self.max_stretch,
        }
    }

    pub fn add_file(&mut self, file: String) -> Result<(), AudioFileError> {
        self.clear_error();
        if self.files.iter().any(|f| f.path == file) {
            return Ok(());
        }
        let mut audio_info = AudioInfo::default();
        audio_info.update_file(file.clone())?;
        let file_index = self.files.len();
        for i in 1..=self.permutations {
            self.outputs.insert(
                (file_index, i),
                OutputProgress {
                    output: String::new(),
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
                    source_file: file.clone(),
                },
            );
        }
        self.files.push(audio_info);
        Ok(())
    }

    pub fn remove_file(&mut self, file: String) {
        self.clear_error();
        self.files.retain(|f| f.path != file);
    }

    pub fn clear_all_files(&mut self) {
        self.clear_error();
        self.files.clear();
        self.outputs.clear();
    }

    pub fn add_processor(&mut self, name: String) {
        self.clear_error();
        if let Ok(processor) = get_processor_from_display_name(&name) {
            if self.processor_pool.iter().all(|p| *p != processor) {
                self.processor_pool.push(processor);
            }
        }
    }

    pub fn remove_processor(&mut self, name: String) {
        self.clear_error();
        if let Ok(processor) = get_processor_from_display_name(&name) {
            self.processor_pool.retain(|p| *p != processor);
        }
    }

    pub fn set_output(&mut self, output: String) {
        self.clear_error();
        self.output = output;
    }

    pub fn get_ordered_outputs(&self) -> Vec<OutputProgress> {
        let mut keys: Vec<_> = self.outputs.keys().collect();
        keys.sort_by_key(|k| *k);
        keys.iter()
            .filter_map(|key| {
                self.outputs.get(key).and_then(|o| {
                    if !o.deleted {
                        Some(o.clone())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn add_output_progress(
        &mut self,
        permutation: Permutation,
        processors: Vec<(PermuteNodeName, Vec<ProcessorAttribute>)>,
    ) {
        if let Some(file_index) = self.files.iter().position(|f| f.path == permutation.file) {
            let key = (file_index, permutation.permutation_index);
            self.outputs.insert(
                key,
                OutputProgress {
                    output: permutation.output.clone(),
                    permutation: permutation.clone(),
                    processors: processors.iter().map(|(p, _)| *p).collect(),
                    progress: 0,
                    audio_info: AudioInfo::default(),
                    deleted: false,
                    source_file: permutation.file.clone(),
                },
            );
        }
    }

    pub fn update_output_progress(&mut self, permutation: Permutation) {
        let percentage = ((permutation.node_index as f64 + 1.0)
            / permutation.processors.len() as f64)
            * 100.0;
        if let Some(file_index) = self.files.iter().position(|f| f.path == permutation.file) {
            let key = (file_index, permutation.permutation_index);
            if let Some(output) = self.outputs.get_mut(&key) {
                output.progress = percentage as i32;
                output.permutation = permutation.clone();
            }
        }
    }

    pub fn set_finished(&mut self) -> Result<(), AudioFileError> {
        self.processing = false;
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.processing = false;
        self.error = "Processing cancelled by user".to_string();
        let _ = self.cancel_sender.send(());
    }

    pub fn set_normalised(&mut self, normalised: bool) {
        self.normalise_at_end = normalised;
    }

    pub fn set_trim_all(&mut self, trim_all: bool) {
        self.trim_all = trim_all;
    }

    pub fn set_max_stretch(&mut self, max_stretch: f64) {
        self.max_stretch = max_stretch;
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
        self.clear_error();
        self.processing = true;
        let update_sender = Arc::try_unwrap(self.update_sender.clone())
            .unwrap_or_else(|arc| (*arc).clone());
        process_file(file.clone(), PermuteNodeName::Reverse, update_sender)?;
        self.processing = false;
        for output in self.outputs.values_mut() {
            if output.output == file {
                output.audio_info.update_file(file.clone())?;
                break;
            }
        }
        AUDIO_CACHE.clear_file(&file);
        clear_file_from_rms_cache(&file);
        Ok(())
    }

    pub fn trim_file(&mut self, file: String) -> Result<(), PermuteError> {
        self.clear_error();
        self.processing = true;
        let update_sender = Arc::try_unwrap(self.update_sender.clone())
            .unwrap_or_else(|arc| (*arc).clone());
        process_file(file.clone(), PermuteNodeName::Trim, update_sender)?;
        self.processing = false;
        for output in self.outputs.values_mut() {
            if output.output == file {
                output.audio_info.update_file(file.clone())?;
                break;
            }
        }
        AUDIO_CACHE.clear_file(&file);
        clear_file_from_rms_cache(&file);
        Ok(())
    }

    pub fn run_process(&mut self) -> JoinHandle<()> {
        self.clear_error();
        if self.processing {
            return thread::spawn(|| {});
        }
        self.processing = true;
        self.outputs.clear();
        let params = self.to_permute_params();
        let state_clone = Arc::new(std::sync::Mutex::new(self.clone()));
        thread::spawn(move || {
            let handle = permute_files(params);
            match handle.join() {
                Ok(_) => {
                    if let Ok(mut s) = state_clone.lock() {
                        let _ = s.set_finished();
                    }
                }
                Err(e) => {
                    if let Ok(mut s) = state_clone.lock() {
                        s.set_error(format!("Processing thread panicked: {:?}", e));
                        let _ = s.set_finished();
                    }
                }
            }
        })
    }

    pub fn delete_output_file(&mut self, file: String) -> Result<(), std::io::Error> {
        fs::remove_file(&file)?;
        for output in self.outputs.values_mut() {
            if output.output == file {
                output.deleted = true;
                break;
            }
        }
        Ok(())
    }

    pub fn delete_all_output_files(&mut self) -> Result<(), std::io::Error> {
        let mut last_output_path = String::new();
        for output in self.outputs.values_mut() {
            if let Err(e) = fs::remove_file(&output.output) {
                eprintln!("Error deleting file {}: {}", output.output, e);
            }
            last_output_path = output.output.clone();
            output.deleted = true;
        }
        if self.create_subdirectories && !last_output_path.is_empty() {
            if let Some(dir) = Path::new(&last_output_path).parent() {
                if dir.is_dir() && dir.read_dir().map(|mut r| r.next().is_none()).unwrap_or(false) {
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
        for output in self.outputs.values_mut() {
            if output.output == file {
                output.audio_info = audio_info.clone();
                output.permutation.output = file.clone();
                break;
            }
        }
    }

    pub fn set_viewed_welcome(&mut self, viewed: bool) {
        self.viewed_welcome = viewed;
    }

    pub fn write_to_json(&mut self, path: String) -> std::io::Result<()> {
        self.clear_error();
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
            viewed_welcome: self.viewed_welcome,
            max_stretch: self.max_stretch,
        };
        let json = serde_json::to_string(&data)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn read_from_json(&mut self, path: String) -> std::io::Result<()> {
        self.clear_error();
        let file = File::open(&path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not open file '{}': {}", path, e),
            )
        })?;
        let reader = BufReader::new(file);
        let data: SharedStateSerializable = serde_json::from_reader(reader).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Invalid scene file format: {}", e),
            )
        })?;
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
        self.viewed_welcome = data.viewed_welcome;
        self.max_stretch = data.max_stretch;
        Ok(())
    }
}

// ─── Serialisable snapshot for JSON persistence ───────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct SharedStateSerializable {
    #[serde(default)]
    pub files: Vec<AudioInfo>,
    #[serde(default)]
    pub output: String,
    #[serde(default = "default_input_trail")]
    pub input_trail: f64,
    #[serde(default = "default_output_trail")]
    pub output_trail: f64,
    #[serde(default = "default_permutations")]
    pub permutations: usize,
    #[serde(default = "default_permutation_depth")]
    pub permutation_depth: usize,
    #[serde(default = "default_processor_pool")]
    pub processor_pool: Vec<PermuteNodeName>,
    #[serde(default = "default_normalise_at_end")]
    pub normalise_at_end: bool,
    #[serde(default)]
    pub trim_all: bool,
    #[serde(default)]
    pub high_sample_rate: bool,
    #[serde(default = "default_processor_count")]
    pub processor_count: Option<i32>,
    #[serde(default = "default_true")]
    pub create_subdirectories: bool,
    #[serde(default)]
    pub viewed_welcome: bool,
    #[serde(default = "default_max_stretch")]
    pub max_stretch: f64,
}

fn default_input_trail() -> f64 { 0.0 }
fn default_output_trail() -> f64 { 2.0 }
fn default_permutations() -> usize { 3 }
fn default_permutation_depth() -> usize { 2 }
fn default_processor_pool() -> Vec<PermuteNodeName> { ALL_PROCESSORS.to_vec() }
fn default_normalise_at_end() -> bool { true }
fn default_processor_count() -> Option<i32> {
    Some(
        std::thread::available_parallelism()
            .map(|n| std::cmp::min(n.get(), 4) as i32)
            .unwrap_or(1),
    )
}
fn default_true() -> bool { true }
fn default_max_stretch() -> f64 { 17.0 }

// ─── OutputProgress ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OutputProgress {
    pub output: String,
    pub progress: i32,
    pub permutation: Permutation,
    pub processors: Vec<PermuteNodeName>,
    pub audio_info: AudioInfo,
    pub deleted: bool,
    pub source_file: String,
}
