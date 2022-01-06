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

pub type ProcessorFn = fn(ProcessorParams) -> ProcessorParams;
pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
}

pub fn get_processor_node(
    GetProcessorNodeParams { normalise_at_end }: GetProcessorNodeParams,
) -> impl Fn(ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    let processor_pool: Vec<ProcessorFn> =
        vec![reverse, random_metallic_delay, random_rhythmic_delay];

    let processor_count = rng.gen_range(2..8);

    let mut processors: Vec<ProcessorFn> = vec![];
    for _ in 0..processor_count {
        processors.push(processor_pool[rng.gen_range(0..processor_pool.len())])
    }
    if normalise_at_end {
        processors.push(normalise)
    }
    move |processor_params: ProcessorParams| {
        return processors
            .iter()
            .fold(processor_params, |params, processor| processor(params));
    }
}
