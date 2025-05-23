// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::delay_reverb::{DelayLineParams, ReverbParams, delay_line, reverb},
    random_processors::utils::{format_float_percent, format_samples_as_ms, format_float_ms},
    process::{ProcessorParams, ProcessorPlan, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent, ProcessorClosure},
    random_process::{start_event, complete_event},
    permute_files::PermuteUpdate,
};

pub fn random_metallic_delay(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let sec_10 = (params.sample_rate as f64 * 0.1) as usize;
    let feedback_factor = rng.gen_range(0_f64..0.9);
    let delay_sample_length = rng.gen_range(10..sec_10);
    let wet_gain_factor = rng.gen_range(0.3..1_f64);

    let attributes = vec![
        ProcessorAttribute {
            key: "Feedback".to_string(),
            value: format_float_percent(feedback_factor),
        },
        ProcessorAttribute {
            key: "Delay".to_string(),
            value: format_samples_as_ms(delay_sample_length, params.sample_rate),
        },
        ProcessorAttribute {
            key: "Wet".to_string(),
            value: format_float_percent(wet_gain_factor),
        },
    ];

    let delay_params = DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor: 1_f64,
        wet_gain_factor,
    };

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::MetallicDelay, &params);
        let new_params = delay_line(&params, &delay_params)?;
        complete_event!(PermuteNodeName::MetallicDelay, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::MetallicDelay, attributes, Box::new(processor))
}

pub fn random_rhythmic_delay(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let sec_10 = (params.sample_rate as f64 * 0.1) as usize;
    let sec = params.sample_rate as usize;
    let feedback_factor = rng.gen_range(0_f64..0.9);
    let delay_sample_length = rng.gen_range(sec_10..sec);

    let attributes = vec![
        ProcessorAttribute {
            key: "Feedback".to_string(),
            value: format_float_percent(feedback_factor),
        },
        ProcessorAttribute {
            key: "Delay".to_string(),
            value: format_samples_as_ms(delay_sample_length, params.sample_rate),
        },
    ];

    let delay_params = DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor: 1_f64,
        wet_gain_factor: 1_f64,
    };

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::RhythmicDelay, &params);
        let new_params = delay_line(&params, &delay_params)?;
        complete_event!(PermuteNodeName::RhythmicDelay, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::RhythmicDelay, attributes, Box::new(processor))
}

pub fn random_reverb(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let len_factors = [0.1, 0.3, 0.6, 1.0, 1.2, 1.4];
    let decay_factors = [0.2, 0.3, 0.325, 0.35, 0.4];

    let predelay_ms = rng.gen_range(0.0..90.0);
    let wet_mix = rng.gen_range(0.1_f64..0.4);
    let len_factor = len_factors[rng.gen_range(0..len_factors.len())];
    let decay_factor = decay_factors[rng.gen_range(0..decay_factors.len())];

    let attributes = vec![
        ProcessorAttribute {
            key: "Predelay".to_string(),
            value: format_float_ms(predelay_ms),
        },
        ProcessorAttribute {
            key: "Wet Mix".to_string(),
            value: format_float_percent(wet_mix),
        },
        ProcessorAttribute {
            key: "Length Factor".to_string(),
            value: len_factor.to_string(),
        },
        ProcessorAttribute {
            key: "Decay Factor".to_string(),
            value: decay_factor.to_string(),
        },
    ];

    let reverb_params = ReverbParams {
        predelay_ms,
        wet_mix,
        len_factor,
        decay_factor,
    };

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Reverb, &params);
        let new_params = reverb(&params, reverb_params)?;
        complete_event!(PermuteNodeName::Reverb, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Reverb, attributes, Box::new(processor))
}