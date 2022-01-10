use crate::process::*;
use rand::prelude::*;

pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
    pub depth: usize,
    pub processor_functions: Vec<ProcessorFn>,
}

pub fn generate_processor_sequence(
    GetProcessorNodeParams {
        normalise_at_end,
        depth,
        processor_functions,
    }: GetProcessorNodeParams,
) -> Vec<ProcessorFn> {
    let mut rng = thread_rng();

    let processor_count = rng.gen_range(2..5);
    let mut processors: Vec<ProcessorFn> = vec![];

    for _ in 0..processor_count {
        processors.push(processor_functions[rng.gen_range(0..processor_functions.len())])
    }
    if depth > 0 {
        processors = [
            generate_processor_sequence(GetProcessorNodeParams {
                depth: depth - 1,
                normalise_at_end: false,
                processor_functions: processor_functions,
            }),
            processors,
        ]
        .concat();
    }
    if normalise_at_end {
        processors.push(normalise);
    }
    processors
}

// Random processors

pub fn random_metallic_delay(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::MetallicDelay,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(10..sec_10),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.3..1_f64),
    };

    let new_params = delay_line(params, delay_params);
    update_progress(
        PermuteNodeName::MetallicDelay,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_params
}

pub fn random_rhythmic_delay(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::RhythmicDelay,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let sec = params.spec.sample_rate as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(sec_10..sec),
        dry_gain_factor: 1_f64,
        wet_gain_factor: 1_f64,
    };

    let new_params = delay_line(params, delay_params);
    update_progress(
        PermuteNodeName::RhythmicDelay,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_params
}

pub fn half_speed(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::HalfSpeed,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let new_samples = change_speed(params, 0.5_f64);
    update_progress(
        PermuteNodeName::HalfSpeed,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_samples
}
pub fn double_speed(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::DoubleSpeed,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let new_samples = change_speed(params, 2_f64);
    update_progress(
        PermuteNodeName::DoubleSpeed,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_samples
}

pub fn random_wow(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(PermuteNodeName::Wow, PermuteNodeEvent::NodeProcessStarted);
    let mut rng = thread_rng();

    let new_samples = vibrato(
        params,
        VibratoParams {
            speed_hz: rng.gen_range(0.2_f64..1.6_f64),
            depth: rng.gen_range(0.3_f64..0.7_f64),
        },
    );
    update_progress(PermuteNodeName::Wow, PermuteNodeEvent::NodeProcessComplete);
    new_samples
}
pub fn random_flutter(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::Flutter,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let mut rng = thread_rng();

    let new_samples = vibrato(
        params,
        VibratoParams {
            speed_hz: rng.gen_range(3_f64..20_f64),
            depth: rng.gen_range(0.05_f64..0.5_f64),
        },
    );
    update_progress(
        PermuteNodeName::Flutter,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_samples
}

pub fn random_chorus(params: ProcessorParams) -> ProcessorParams {
    let update_progress = params.update_progress;
    update_progress(
        PermuteNodeName::Chorus,
        PermuteNodeEvent::NodeProcessStarted,
    );
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
        params,
        ChorusParams {
            delay_params,
            vibrato_params,
        },
    );
    update_progress(
        PermuteNodeName::Chorus,
        PermuteNodeEvent::NodeProcessComplete,
    );
    new_samples
}
