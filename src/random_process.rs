use crate::process::*;
use rand::prelude::*;

pub fn random_metallic_delay(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;

    delay_line(params, rng.gen_range(0_f64..0.8), rng.gen_range(10..sec_10))
}

pub fn random_rhythmic_delay(params: ProcessorParams) -> ProcessorParams {
    let mut rng = thread_rng();

    let sec_10 = (params.spec.sample_rate as f64 * 0.1) as usize;
    let sec = params.spec.sample_rate as usize;
    delay_line(
        params,
        rng.gen_range(0_f64..0.8),
        rng.gen_range(sec_10..sec),
    )
}
