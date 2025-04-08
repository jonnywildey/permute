// External dependencies
use rand::{rngs::ThreadRng, Rng};

// Internal modules
use crate::{
    process::{PermuteNodeName, ProcessorPlanGenerator}, random_processors::{
        random_cross::{random_cross_distort, random_cross_filter, random_cross_gain},
        random_delay_verb::{random_metallic_delay, random_reverb, random_rhythmic_delay},
        random_filter::{random_filter, random_line_filter, random_oscillating_filter},
        random_gain_distortion::{auto_trim, normalise, random_fuzz, random_saturate},
        random_modulation::{random_chorus, random_flutter, random_lazer, random_phaser, random_tremolo, random_wow, random_zero_flange},
        random_time_pitch::{change_sample_rate_high, change_sample_rate_original, double_speed, half_speed, random_blur_stretch, random_granular_time_stretch, random_pitch, reverse_with_plan},
    }
};

macro_rules! start_event {
    ($name:expr, $params:expr) => {{
        $params
            .update_sender
            .send(PermuteUpdate::UpdatePermuteNodeStarted(
                $params.permutation.clone(),
                $name,
                PermuteNodeEvent::NodeProcessStarted,
            ))?;
    }};
}
pub(crate) use start_event;

macro_rules! complete_event {
    ($name:expr, $params:expr) => {{
        $params
            .update_sender
            .send(PermuteUpdate::UpdatePermuteNodeCompleted(
                $params.permutation.clone(),
                $name,
                PermuteNodeEvent::NodeProcessComplete,
            ))?;
    }};
}
pub(crate) use complete_event;

pub struct GetProcessorNodeParams  {
    pub normalise_at_end: bool,
    pub trim_at_end: bool,
    pub high_sample_rate: bool,
    pub depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processor_count: Option<i32>,
    pub constrain_length: bool,
    pub rng: ThreadRng,
    pub original_depth: usize,
}

/// Select a random processor from the processor pool
pub fn select_random_processor(processor_pool: &[PermuteNodeName]) -> PermuteNodeName {
    processor_pool[rand::thread_rng().gen_range(0..processor_pool.len())]
}

pub fn generate_processor_sequence(
    params: GetProcessorNodeParams,
) -> Vec<PermuteNodeName> {
    let GetProcessorNodeParams {
        normalise_at_end,
        trim_at_end,
        high_sample_rate,
        depth,
        processor_pool,
        processor_count,
        constrain_length,
        mut rng,
        original_depth,
    } = params;
    let mut processors: Vec<PermuteNodeName> = vec![];
    if depth == 0 {
        if original_depth == depth {
            processors.push(select_random_processor(&processor_pool));
        }
        if high_sample_rate {
            processors.insert(0, PermuteNodeName::SampleRateConversionHigh);
            processors.push(PermuteNodeName::SampleRateConversionOriginal);
        }
        if normalise_at_end {
            processors.push(PermuteNodeName::Normalise);
        }
        if trim_at_end {
            processors.push(PermuteNodeName::Trim);
        }
        return processors;
    };

    let processor_count = processor_count.unwrap_or(rng.gen_range(2..5));

    for _ in 0..processor_count {
        processors.push(processor_pool[rng.gen_range(0..processor_pool.len())]);
    }

    processors = [
        processors,
        generate_processor_sequence(GetProcessorNodeParams {
            depth: depth - 1,
            normalise_at_end: normalise_at_end,
            trim_at_end: trim_at_end,
            processor_pool,
            high_sample_rate: high_sample_rate,
            processor_count: Some(processor_count),
            constrain_length,
            rng,
            original_depth: original_depth,
        }),
    ]
    .concat();

    processors
}

pub fn get_processor_plan(name: PermuteNodeName) -> ProcessorPlanGenerator {
    match name {
        // Time and pitch
        PermuteNodeName::GranularTimeStretch => random_granular_time_stretch,
        PermuteNodeName::DoubleSpeed => double_speed,
        PermuteNodeName::RandomPitch => random_pitch,
        PermuteNodeName::BlurStretch => random_blur_stretch,
        PermuteNodeName::HalfSpeed => half_speed,
        PermuteNodeName::Reverse => reverse_with_plan,
        // // Modulation
        PermuteNodeName::Chorus => random_chorus,
        PermuteNodeName::Phaser => random_phaser,
        PermuteNodeName::Flutter => random_flutter,
        PermuteNodeName::Flange => random_zero_flange,
        PermuteNodeName::Wow => random_wow,
        PermuteNodeName::Tremolo => random_tremolo,
        PermuteNodeName::Lazer => random_lazer,
        
        // // Delay and reverb
        PermuteNodeName::RhythmicDelay => random_rhythmic_delay,
        PermuteNodeName::Reverb => random_reverb,
        PermuteNodeName::MetallicDelay => random_metallic_delay,
        
        // // Gain and distortion
        PermuteNodeName::Fuzz => random_fuzz,
        PermuteNodeName::Saturate => random_saturate,
        // // Filters
        PermuteNodeName::Filter => random_filter,
        PermuteNodeName::LineFilter => random_line_filter,
        PermuteNodeName::OscillatingFilter => random_oscillating_filter,
        
        // // Cross/sidechain
        PermuteNodeName::CrossGain => random_cross_gain,
        PermuteNodeName::CrossFilter => random_cross_filter,
        PermuteNodeName::CrossDistort => random_cross_distort,
        // // Util
        PermuteNodeName::Normalise => normalise,
        PermuteNodeName::Trim => auto_trim,
        PermuteNodeName::SampleRateConversionHigh => change_sample_rate_high,
        PermuteNodeName::SampleRateConversionOriginal => change_sample_rate_original,
        _ => panic!("Processor not found {:?}", name),
    }
}
