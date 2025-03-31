// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::cross::{CrossGainParams, CrossFilterParams, CrossDistortParams, cross_gain, cross_filter, cross_distort},
    processors::gain_distortion::DistortionAlgorithm,
    random_processors::utils::{format_float, format_hz, format_float_percent, format_float_ms, get_filename},
    random_process::{start_event, complete_event},
    process::{ProcessorParams, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent},
    permute_files::PermuteUpdate,
    permute_error::PermuteError,
};

pub fn random_cross_gain(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::CrossGain, params);
    let mut rng = thread_rng();

    // Get a random file from the files list
    let sidechain_file = match select_sidechain_file(&params.permutation.file, &params.permutation.files) {
        Some(file) => file,
        None => {
            // If there's only one file, just return the original
            complete_event!(PermuteNodeName::CrossGain, params);
            return Ok(params.clone());
        }
    };

    let depth = rng.gen_range(0.2..0.9);
    let invert = rng.gen_bool(0.5);
    let window_size_ms = 100.0; // 100ms window size

    let cross_params = CrossGainParams {
        sidechain_file: sidechain_file.clone(),
        depth,
        invert,
        window_size_ms,
    };

    let mut new_params = cross_gain(params, &cross_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Sidechain File".to_string(),
                value: get_filename(&sidechain_file),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
            ProcessorAttribute {
                key: "Invert".to_string(),
                value: invert.to_string(),
            },
            ProcessorAttribute {
                key: "Window Size".to_string(),
                value: format_float_ms(window_size_ms),
            },
        ],
    );

    complete_event!(PermuteNodeName::CrossGain, new_params);
    Ok(new_params)
}

pub fn random_cross_filter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::CrossFilter, params);
    let mut rng = thread_rng();
    
    // Get a random file from the files list
    let sidechain_file = match select_sidechain_file(&params.permutation.file, &params.permutation.files) {
        Some(file) => file,
        None => {
            // If there's only one file, just return the original
            complete_event!(PermuteNodeName::CrossFilter, params);
            return Ok(params.clone());
        }
    };

    // Generate random filter parameters
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::LowPass, // make low pass most likely
        biquad::Type::BandPass,
    ];
    let filter_type = types[rng.gen_range(0..types.len())];

    // Base frequency between 200hz and 2000hz
    let base_freq = rng.gen_range(50.0..800.0);
    // Maximum frequency between base_freq and 10000hz
    let max_freq = rng.gen_range(base_freq..10000.0);
    // Q factor between 0.5 and 1.35 (similar to random_line_filter)
    let q = rng.gen_range(0.5..1.35);
    let window_size_ms = 100.0; // Fixed 10ms window for RMS calculation
    let invert = rng.gen_bool(0.5);

    let cross_params = CrossFilterParams {
        sidechain_file: sidechain_file.clone(),
        filter_type,
        base_freq,
        max_freq,
        q,
        window_size_ms,
        invert,
    };

    let mut new_params = cross_filter(params, &cross_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Sidechain File".to_string(),
                value: get_filename(&sidechain_file),
            },
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "Base Frequency".to_string(),
                value: format_hz(base_freq),
            },
            ProcessorAttribute {
                key: "Max Frequency".to_string(),
                value: format_hz(max_freq),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Window Size".to_string(),
                value: format_float_ms(window_size_ms),
            },
            ProcessorAttribute {
                key: "Invert".to_string(),
                value: invert.to_string(),
            },
        ],
    );

    complete_event!(PermuteNodeName::CrossFilter, new_params);
    Ok(new_params)
}

pub fn random_cross_distort(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::CrossDistort, params);
    let mut rng = rand::thread_rng();
    
    // Get a random file from the files list
    let sidechain_file = match select_sidechain_file(&params.permutation.file, &params.permutation.files) {
        Some(file) => file,
        None => {
            // If there's only one file, just return the original
            complete_event!(PermuteNodeName::CrossDistort, params);
            return Ok(params.clone());
        }
    };

    // Generate random distortion parameters
    let algorithms = [
        DistortionAlgorithm::Tanh,     // Bias towards the gentler algorithms
        DistortionAlgorithm::Tanh,
        DistortionAlgorithm::Atan,
        DistortionAlgorithm::Atan,
        DistortionAlgorithm::Cubic,
        DistortionAlgorithm::Cubic,
        DistortionAlgorithm::Saturate,
        DistortionAlgorithm::Saturate,
        DistortionAlgorithm::Power,     // Original algorithm used less frequently
    ];
    
    // Factor ranges depend on the algorithm
    let algorithm = algorithms[rng.gen_range(0..algorithms.len())];
    let (min_factor, increase) = match algorithm {
        DistortionAlgorithm::Power => {
            // Original power function - needs smaller values but wider range
            let min_factors = [0.4, 0.5, 0.6];
            let increases = [0.4, 0.5, 0.6];
            (
                min_factors[rng.gen_range(0..min_factors.len())],
                increases[rng.gen_range(0..increases.len())]
            )
        },
        DistortionAlgorithm::Tanh | DistortionAlgorithm::Atan => {
            // These work well with larger ranges
            let min_factors = [0.5, 1.0, 1.5];
            let increases = [2.0, 3.0, 4.0];
            (
                min_factors[rng.gen_range(0..min_factors.len())],
                increases[rng.gen_range(0..increases.len())]
            )
        },
        DistortionAlgorithm::Cubic => {
            // Cubic needs values around 1.0 for soft clipping but can handle wider range
            let min_factors = [0.3, 0.4, 0.5];
            let increases = [0.8, 1.0, 1.2];
            (
                min_factors[rng.gen_range(0..min_factors.len())],
                increases[rng.gen_range(0..increases.len())]
            )
        },
        DistortionAlgorithm::Saturate => {
            // Saturate works well with moderate to high ranges
            let min_factors = [0.5, 0.8, 1.0];
            let increases = [1.5, 2.0, 2.5];
            (
                min_factors[rng.gen_range(0..min_factors.len())],
                increases[rng.gen_range(0..increases.len())]
            )
        }
    };
    
    let cross_params = CrossDistortParams {
        sidechain_file,
        min_factor,
        max_factor: min_factor + increase,
        window_size_ms: 100.0, // Fixed 100ms window for RMS calculation
        algorithm,
        invert: rng.gen_bool(0.5),
    };

    let result = cross_distort(params, &cross_params);
    complete_event!(PermuteNodeName::CrossDistort, params);
    result
}

/// Select a random file from the available files list that is different from the current file
pub fn select_sidechain_file(current_file: &str, available_files: &[String]) -> Option<String> {
    if available_files.len() < 2 {
        return None;
    }
    
    let mut rng = rand::thread_rng();
    let filtered_files: Vec<&String> = available_files.iter()
        .filter(|f| *f != current_file)
        .collect();
        
    if filtered_files.is_empty() {
        None
    } else {
        Some(filtered_files[rng.gen_range(0..filtered_files.len())].clone())
    }
}