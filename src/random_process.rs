use crate::process::*;
use rand::prelude::*;

pub fn random_metallic_delay(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(10..sec_10),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.3..1_f64),
    };

    delay_line(params, delay_params)
}

pub fn random_rhythmic_delay(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let sec = params.spec.sample_rate as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(sec_10..sec),
        dry_gain_factor: 1_f64,
        wet_gain_factor: 1_f64,
    };

    delay_line(params, delay_params)
}

pub fn half_speed(params: ProcessorParams) -> ProcessorParams {
    change_speed(params, 0.5_f64)
}
pub fn double_speed(params: ProcessorParams) -> ProcessorParams {
    change_speed(params, 2_f64)
}

pub fn random_wow(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    vibrato(
        params,
        VibratoParams {
            speed_hz: rng.gen_range(0.1_f64..1_f64),
            depth: rng.gen_range(0.1_f64..0.5_f64),
        },
    )
}
pub fn random_flutter(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    vibrato(
        params,
        VibratoParams {
            speed_hz: rng.gen_range(1_f64..10_f64),
            depth: rng.gen_range(0.1_f64..0.3_f64),
        },
    )
}

pub fn random_chorus(params: ProcessorParams) -> ProcessorParams {
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
        speed_hz: rng.gen_range(0.1_f64..5_f64),
        depth: rng.gen_range(0.1_f64..0.7_f64),
    };

    chorus(
        params,
        ChorusParams {
            delay_params,
            vibrato_params,
        },
    )
}

pub type ProcessorFn = fn(ProcessorParams) -> ProcessorParams;
pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
    pub depth: usize,
}

pub fn generate_processor_sequence(
    GetProcessorNodeParams {
        normalise_at_end,
        depth,
    }: GetProcessorNodeParams,
) -> Vec<ProcessorFn> {
    let mut rng = thread_rng();

    let processor_pool: Vec<ProcessorFn> = vec![
        // reverse,
        // random_metallic_delay,
        // random_rhythmic_delay,
        // reverse,
        // random_metallic_delay,
        // random_rhythmic_delay,
        half_speed,
        double_speed,
        // random_wow,
        // random_flutter,
        // random_chorus,
    ];

    let processor_count = rng.gen_range(2..8);
    let mut processors: Vec<ProcessorFn> = vec![];

    for _ in 0..processor_count {
        processors.push(processor_pool[rng.gen_range(0..processor_pool.len())])
    }
    if depth > 0 {
        processors = [
            generate_processor_sequence(GetProcessorNodeParams {
                depth: depth - 1,
                normalise_at_end: false,
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

pub struct RunProcessorsParams {
    pub processors: Vec<ProcessorFn>,
    pub processor_params: ProcessorParams,
}

pub fn run_processors(
    RunProcessorsParams {
        processors,
        processor_params,
    }: RunProcessorsParams,
) -> ProcessorParams {
    processors
        .iter()
        .fold(processor_params, |params, processor| processor(params))
}
