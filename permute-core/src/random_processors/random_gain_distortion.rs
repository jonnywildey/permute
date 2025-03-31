// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::gain_distortion::{FuzzParams, fuzz, saturate, ceiling},
    random_processors::utils::format_float,
    random_process::{start_event, complete_event},
    process::{ProcessorParams, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent},
    permute_files::PermuteUpdate,
    permute_error::PermuteError,
};

pub fn random_fuzz(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Fuzz, params);
    let mut rng = thread_rng();

    let gain = rng.gen_range(0.5_f64..3.0_f64);
    let output_gain = rng.gen_range(0.1_f64..1.0_f64);

    let mut new_params = fuzz(
        params.to_owned(),
        FuzzParams {
            gain,
            output_gain,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Gain".to_string(),
                value: format_float(gain),
            },
            ProcessorAttribute {
                key: "Output Gain".to_string(),
                value: format_float(output_gain),
            },
        ],
    );

    complete_event!(PermuteNodeName::Fuzz, new_params);
    Ok(new_params)
}

pub fn random_saturate(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Saturate, params);

    let new_params = saturate(&params.clone())?;
    complete_event!(PermuteNodeName::Saturate, new_params);
    Ok(new_params)
}

pub fn normalise(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Normalise, params);

    let new_params = ceiling(params.to_owned(), 1_f64);
    complete_event!(PermuteNodeName::Normalise, new_params);
    Ok(new_params)
}