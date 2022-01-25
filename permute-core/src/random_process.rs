use crate::{permute_files::PermuteUpdate, process::*};
use rand::prelude::*;
use strum::IntoEnumIterator;

pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processor_count: Option<i32>,
}

pub fn generate_processor_sequence(
    GetProcessorNodeParams {
        normalise_at_end,
        high_sample_rate,
        depth,
        processor_pool,
        processor_count,
    }: GetProcessorNodeParams,
) -> Vec<PermuteNodeName> {
    let mut rng = thread_rng();

    let processor_count = processor_count.unwrap_or(rng.gen_range(2..5));
    let mut processors: Vec<PermuteNodeName> = vec![];

    for _ in 0..processor_count {
        processors.push(processor_pool[rng.gen_range(0..processor_pool.len())])
    }
    if depth > 1 {
        processors = [
            generate_processor_sequence(GetProcessorNodeParams {
                depth: depth - 1,
                normalise_at_end: false,
                processor_pool,
                high_sample_rate: false,
                processor_count: Some(processor_count),
            }),
            processors,
        ]
        .concat();
    }
    if high_sample_rate {
        processors.insert(0, PermuteNodeName::SampleRateConversionHigh);
        processors.push(PermuteNodeName::SampleRateConversionOriginal);
    }
    if normalise_at_end {
        processors.push(PermuteNodeName::Normalise);
    }

    processors
}

pub fn get_processor_function(name: PermuteNodeName) -> ProcessorFn {
    match name {
        PermuteNodeName::Reverse => reverse,
        PermuteNodeName::Chorus => random_chorus,
        PermuteNodeName::Phaser => random_phaser,
        PermuteNodeName::DoubleSpeed => double_speed,
        PermuteNodeName::Flutter => random_flutter,
        PermuteNodeName::HalfSpeed => half_speed,
        PermuteNodeName::MetallicDelay => random_metallic_delay,
        PermuteNodeName::RhythmicDelay => random_rhythmic_delay,
        PermuteNodeName::Wow => random_wow,
        PermuteNodeName::Normalise => normalise,
        PermuteNodeName::SampleRateConversionHigh => change_sample_rate_high,
        PermuteNodeName::SampleRateConversionOriginal => change_sample_rate_original,
    }
}

// Random processors

pub fn random_metallic_delay(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::MetallicDelay,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(10..sec_10),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.3..1_f64),
    };

    let new_params = delay_line(&params.clone(), &delay_params);
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        params.permutation.clone(),
        PermuteNodeName::MetallicDelay,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_params
}

pub fn random_rhythmic_delay(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::RhythmicDelay,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let sec = params.spec.sample_rate as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(sec_10..sec),
        dry_gain_factor: 1_f64,
        wet_gain_factor: 1_f64,
    };

    let new_params = delay_line(&params, &delay_params);
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::RhythmicDelay,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_params
}

pub fn half_speed(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::HalfSpeed,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let new_samples = change_speed(params.to_owned(), 0.5_f64);
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::HalfSpeed,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}
pub fn double_speed(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::DoubleSpeed,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let new_samples = change_speed(params.to_owned(), 2_f64);
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::DoubleSpeed,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}

pub fn random_wow(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::Wow,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut rng = thread_rng();

    let new_samples = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz: rng.gen_range(0.2_f64..1.6_f64),
            depth: rng.gen_range(0.3_f64..0.7_f64),
        },
    );
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::Wow,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}
pub fn random_flutter(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::Flutter,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut rng = thread_rng();

    let new_samples = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz: rng.gen_range(3_f64..20_f64),
            depth: rng.gen_range(0.05_f64..0.5_f64),
        },
    );
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::Flutter,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}

pub fn random_chorus(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::Chorus,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut rng = thread_rng();

    let millis_low = (params.spec.sample_rate as f64 / 1000_f64 * 4_f64) as usize;
    let millis_high = (params.spec.sample_rate as f64 / 1000_f64 * 20_f64) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.8_f64),
        delay_sample_length: rng.gen_range(millis_low..millis_high),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.7..1_f64),
    };

    let vibrato_params = VibratoParams {
        speed_hz: rng.gen_range(0.5_f64..5_f64),
        depth: rng.gen_range(0.2_f64..0.4_f64),
    };

    let new_samples = chorus(
        params.to_owned(),
        ChorusParams {
            delay_params,
            vibrato_params,
        },
    );
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::Chorus,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}

pub fn random_phaser(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        params.permutation.clone(),
        PermuteNodeName::Phaser,
        PermuteNodeEvent::NodeProcessStarted,
    ));

    let mut rng = thread_rng();

    let stages = PhaserStages::iter().choose(&mut rng).unwrap();

    let phaser_params = PhaserParams {
        base_freq: rng.gen_range(300.0..700.0),
        lfo_rate: rng.gen_range(0.2..2.0), // Maybe a separate one for faster?
        q: rng.gen_range(0.15..0.5),
        stages: stages,
        lfo_depth: rng.gen_range(0.5..0.95),
        stage_hz: 0.0,
        dry_mix: 1.0,
        wet_mix: 1.0,
    };

    let new_samples = phaser(&params.to_owned(), &phaser_params);

    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::Phaser,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}

pub fn normalise(params: &ProcessorParams) -> ProcessorParams {
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.clone();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::Normalise,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let new_samples = ceiling(params.to_owned(), 1_f64);
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation,
        PermuteNodeName::Normalise,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_samples
}
