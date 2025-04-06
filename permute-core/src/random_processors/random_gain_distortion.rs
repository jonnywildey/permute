// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::gain_distortion::{FuzzParams, fuzz, saturate, ceiling, trim_threshold},
    random_processors::utils::format_float,
    random_process::{start_event, complete_event},
    process::{ProcessorParams, ProcessorPlan, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent},
    permute_files::PermuteUpdate,
};

pub fn random_fuzz(_params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let gain = rng.gen_range(0.5_f64..3.0_f64);
    let output_gain = rng.gen_range(0.1_f64..1.0_f64);

    let attributes = vec![
        ProcessorAttribute {
            key: "Gain".to_string(),
            value: format_float(gain),
        },
        ProcessorAttribute {
            key: "Output Gain".to_string(),
            value: format_float(output_gain),
        },
    ];

    let fuzz_params = FuzzParams {
        gain,
        output_gain,
    };

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Fuzz, &params);
        let new_params = fuzz(params, fuzz_params)?;
        complete_event!(PermuteNodeName::Fuzz, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Fuzz, attributes, Box::new(processor))
}

pub fn random_saturate(_params: &mut ProcessorParams) -> ProcessorPlan {
    let attributes = vec![
        ProcessorAttribute {
            key: "Algorithm".to_string(),
            value: "Saturate".to_string(),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Saturate, &params);
        let new_params = saturate(&params)?;
        complete_event!(PermuteNodeName::Saturate, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Saturate, attributes, Box::new(processor))
}

pub fn normalise(_params: &mut ProcessorParams) -> ProcessorPlan  {
    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Normalise, &params);
        let new_params = ceiling(params, 1_f64);
        complete_event!(PermuteNodeName::Normalise, new_params);
        Ok(new_params)
    };
    let attributes = vec![
        ProcessorAttribute {
            key: "Ceiling".to_string(),
            value: format_float(1_f64),
        },
    ];
    (PermuteNodeName::Normalise, attributes, Box::new(processor))
}

pub fn auto_trim(params: &mut ProcessorParams) -> ProcessorPlan {
    let threshold = 0.1_f64;
    let attributes = vec![
        ProcessorAttribute {
            key: "Threshold".to_string(),
            value: format_float(threshold),
        },
    ];
    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Trim, &params);
        let new_params = trim_threshold(&params, threshold)?;
        complete_event!(PermuteNodeName::Trim, new_params);
        Ok(new_params)
    };
    (PermuteNodeName::Trim, attributes, Box::new(processor))
}