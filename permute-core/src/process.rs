// External dependencies
use serde::{Deserialize, Serialize};
use sndfile::{Endian, MajorFormat, SubtypeFormat};
use strum::EnumIter;
use crossbeam_channel::Sender;

// Standard library
use std::sync::Arc;

// Internal modules
use crate::{
    permute_error::PermuteError,
    permute_files::PermuteUpdate,
};

pub type ProcessorPlanGenerator = fn(&mut ProcessorParams) -> ProcessorPlan;
pub type ProcessorPlan = (PermuteNodeName, Vec<ProcessorAttribute>, ProcessorClosure);
pub type ProcessorClosure = Box<dyn FnOnce(ProcessorParams) -> Result<ProcessorParams, PermuteError>>;

#[derive(Debug, Clone)]
pub struct ProcessorParams {
    pub samples: Vec<f64>,
    pub sample_length: usize,
    pub permutation: Permutation,

    pub channels: usize,
    pub sample_rate: usize,
    pub sub_format: SubtypeFormat,
    pub file_format: MajorFormat,
    pub endian: Endian,

    pub update_sender: Arc<Sender<PermuteUpdate>>,
}

impl Default for ProcessorParams {
    fn default() -> Self {
        ProcessorParams { 
            samples: vec![], 
            sample_length: 0, 
            permutation: 
            Permutation::default(), 
            channels: 0, 
            sample_rate: 0, 
            sub_format: SubtypeFormat::PCM_16, 
            file_format: MajorFormat::WAV,
            endian: Endian::Little,
            update_sender: Arc::new(crossbeam_channel::unbounded().0)
        }
    }
}


#[derive(Debug, Clone)]
pub struct Permutation {
    pub file: String,
    pub permutation_index: usize,
    pub output: String,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processors: Vec<PermutationProcessor>,
    pub original_sample_rate: usize,
    pub node_index: usize,
    pub files: Vec<String>,
}

impl Default for Permutation {
    fn default() -> Self {
        Permutation { 
            file: String::new(), 
            permutation_index: 0, 
            output: String::new(), 
            processor_pool: vec![], 
            processors: vec![], 
            original_sample_rate: 0, 
            node_index: 0, 
            files: vec![] 
        }
    }
}


#[derive(Debug, Clone)]
pub struct PermutationProcessor {
    pub name: PermuteNodeName,
    pub attributes: Vec<ProcessorAttribute>,
}       

#[derive(Debug, Clone)]
pub struct ProcessorAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum PermuteNodeEvent {
    NodeProcessStarted,
    NodeProcessComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum PermuteNodeName {
    GranularTimeStretch,
    Fuzz,
    Saturate,
    Reverse,
    Chorus,
    Phaser,
    DoubleSpeed,
    RandomPitch,
    Flutter,
    Flange,
    HalfSpeed,
    MetallicDelay,
    RhythmicDelay,
    Reverb,
    Wow,
    Tremolo,
    Lazer,
    Normalise,
    Trim,
    SampleRateConversionHigh,
    SampleRateConversionOriginal,
    Filter,
    OscillatingFilter,
    LineFilter,
    CrossGain,
    CrossFilter,
    CrossDistort,
    BlurStretch,
}

// Only processors we want to be visible to users
pub const ALL_PROCESSORS: [PermuteNodeName; 23] = [
    PermuteNodeName::GranularTimeStretch,
    PermuteNodeName::Fuzz,
    PermuteNodeName::Saturate,
    PermuteNodeName::Reverse,
    PermuteNodeName::Chorus,
    PermuteNodeName::Phaser,
    PermuteNodeName::DoubleSpeed,
    PermuteNodeName::RandomPitch,
    PermuteNodeName::Flutter,
    PermuteNodeName::Flange,
    PermuteNodeName::HalfSpeed,
    PermuteNodeName::MetallicDelay,
    PermuteNodeName::RhythmicDelay,
    PermuteNodeName::Reverb,
    PermuteNodeName::Wow,
    PermuteNodeName::Tremolo,
    PermuteNodeName::Lazer,
    // Do not expose these to users
    // PermuteNodeName::Normalise,
    // PermuteNodeName::Trim,
    // PermuteNodeName::SampleRateConversionHigh,
    // PermuteNodeName::SampleRateConversionOriginal,
    PermuteNodeName::Filter,
    PermuteNodeName::OscillatingFilter,
    PermuteNodeName::LineFilter,
    PermuteNodeName::CrossGain,
    PermuteNodeName::CrossFilter,
    // Cross Distort doesn't seem to do much different to cross gain
    // PermuteNodeName::CrossDistort,
    PermuteNodeName::BlurStretch,
];