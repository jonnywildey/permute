// External dependencies
use rand::{rngs::ThreadRng, thread_rng, Rng};

// Internal modules
use crate::{
    process::{ProcessorFn, PermuteNodeName},
    random_processors::{
        random_cross::{random_cross_gain, random_cross_filter, random_cross_distort},
        random_delay_verb::{random_metallic_delay, random_rhythmic_delay, random_reverb},
        random_filter::{random_filter, random_line_filter, random_oscillating_filter},
        random_gain_distortion::{random_fuzz, random_saturate, normalise},
        random_modulation::{random_chorus, random_phaser, random_wow, random_tremolo, random_lazer, random_zero_flange, random_flutter},
        random_time_pitch::{random_granular_time_stretch, random_pitch, half_speed, double_speed, random_blur_stretch},
    },
    processors::time_pitch::{change_sample_rate_high, change_sample_rate_original, reverse},
    processors::gain_distortion::trim,
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


const MAX_LENGTH_INCREASING: usize = 3;

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
        let available_processors: Vec<PermuteNodeName> = if constrain_length && count_length_increasing(&processors) >= MAX_LENGTH_INCREASING {
            // If we've hit the length increase limit, filter out length-increasing processors
                processor_pool
                    .iter()
                    .filter(|p| !is_length_increasing(p))
                    .cloned()
                    .collect()
        } else {
            processor_pool.clone()
        };

        if !available_processors.is_empty() {
            processors.push(available_processors[rng.gen_range(0..available_processors.len())]);
        }
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

fn is_length_increasing(processor: &PermuteNodeName) -> bool {
    matches!(
        processor,
        PermuteNodeName::GranularTimeStretch | 
        PermuteNodeName::HalfSpeed | 
        PermuteNodeName::BlurStretch |
        PermuteNodeName::RandomPitch
    )
}

fn count_length_increasing(processors: &[PermuteNodeName]) -> usize {
    processors.iter().filter(|p| is_length_increasing(p)).count()
}

pub fn get_processor_function(name: PermuteNodeName) -> ProcessorFn {
    match name {
        PermuteNodeName::GranularTimeStretch => random_granular_time_stretch,
        PermuteNodeName::Fuzz => random_fuzz,
        PermuteNodeName::Saturate => random_saturate,
        PermuteNodeName::Reverse => reverse,
        PermuteNodeName::Chorus => random_chorus,
        PermuteNodeName::Phaser => random_phaser,
        PermuteNodeName::DoubleSpeed => double_speed,
        PermuteNodeName::RandomPitch => random_pitch,
        PermuteNodeName::Flutter => random_flutter,
        PermuteNodeName::Flange => random_zero_flange,
        PermuteNodeName::HalfSpeed => half_speed,
        PermuteNodeName::MetallicDelay => random_metallic_delay,
        PermuteNodeName::RhythmicDelay => random_rhythmic_delay,
        PermuteNodeName::Reverb => random_reverb,
        PermuteNodeName::Wow => random_wow,
        PermuteNodeName::Tremolo => random_tremolo,
        PermuteNodeName::Lazer => random_lazer,
        PermuteNodeName::Normalise => normalise,
        PermuteNodeName::Trim => trim,
        PermuteNodeName::SampleRateConversionHigh => change_sample_rate_high,
        PermuteNodeName::SampleRateConversionOriginal => change_sample_rate_original,
        PermuteNodeName::Filter => random_filter,
        PermuteNodeName::LineFilter => random_line_filter,
        PermuteNodeName::OscillatingFilter => random_oscillating_filter,
        PermuteNodeName::CrossGain => random_cross_gain,
        PermuteNodeName::CrossFilter => random_cross_filter,
        PermuteNodeName::CrossDistort => random_cross_distort,
        PermuteNodeName::BlurStretch => random_blur_stretch,
    }
}
