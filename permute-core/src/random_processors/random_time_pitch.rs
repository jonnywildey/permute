// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::time_pitch::{
        TimeStretchParams, time_stretch_cross, change_speed, 
        stft_time_stretch, StftTimeStretchParams, WindowType
    },
    random_processors::utils::{format_samples_as_ms, format_factor_to_pitch},
    process::{ProcessorParams, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent},
    random_process::{start_event, complete_event},
    permute_files::PermuteUpdate,
    permute_error::PermuteError,
};

pub fn random_pitch(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::RandomPitch, params);
    let mut rng = thread_rng();

    let speeds: [f64; 10] =
        [-10.0, -8.0, -7.0, -5.0, -2.0, 2.0, 5.0, 7.0, 8.0, 10.0].map(|v| 2_f64.powf(v / 12.0));

    let speed = speeds[rng.gen_range(0..speeds.len())];

    params.update_processor_attributes(
        params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Pitch".to_string(),
                value: format_factor_to_pitch(speed),
            },
        ],
    );

    let new_params = change_speed(params.clone(), speed);
    complete_event!(PermuteNodeName::RandomPitch, new_params);

    Ok(new_params)
}

pub fn random_granular_time_stretch(
    params: &mut ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::GranularTimeStretch, params);

    let mut rng = thread_rng();
    let grain = [
        200, 400, 600, 1000, 1600, 2000, 2200, 2400, 2600, 2800, 3000, 4000, 10000, 20000,
    ];
    let stretch = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4];
    let blend = [
        20, 40, 80, 100, 140, 160, 180, 200, 220, 240, 300, 340, 400, 500, 1200, 2000, 4000,
    ];

    let grain_samples = grain[rng.gen_range(0..grain.len())];
    let stretch_factor = stretch[rng.gen_range(0..stretch.len())];
    let blend_samples = blend[rng.gen_range(0..blend.len())];

    let time_stretch_params = TimeStretchParams {
        grain_samples,
        stretch_factor,
        blend_samples,
    };

    let mut new_params = time_stretch_cross(&params, time_stretch_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Grain".to_string(),
                value: format_samples_as_ms(grain_samples, params.sample_rate),
            },
            ProcessorAttribute {
                key: "Stretch Factor".to_string(),
                value: format!("{}X", stretch_factor.to_string()),
            },
            ProcessorAttribute {
                key: "Blend".to_string(),
                value: format_samples_as_ms(blend_samples, params.sample_rate),
            },
        ],
    );

    complete_event!(PermuteNodeName::GranularTimeStretch, new_params);
    Ok(new_params)
}

pub fn half_speed(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::HalfSpeed, params);
    let new_params = change_speed(params.to_owned(), 0.5_f64);
    complete_event!(PermuteNodeName::HalfSpeed, new_params);
    Ok(new_params)
}
pub fn double_speed(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::DoubleSpeed, params);
    let new_params = change_speed(params.to_owned(), 2_f64);
    complete_event!(PermuteNodeName::DoubleSpeed, new_params);
    Ok(new_params)
}

pub fn random_blur_stretch(
    params: &mut ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::BlurStretch, params);

    let mut rng = rand::thread_rng();
    // Randomize window size between 1024 and 4096 samples
    let window_options = [4096, 5000, 6000, 8192, 10000, 10240, 12288, 16384, 32768, 64000, 128000];
    let window_size = window_options[rng.gen_range(0..window_options.len())];
    // Randomize hop size between 1/4 and 1/2 of window size
    let hop_options = [window_size/4, window_size/2, window_size/3, window_size/4];
    let hop_size = hop_options[rng.gen_range(0..hop_options.len())];
    // Randomize stretch factor between 0.5 and 2.0
    let stretch_options = [0.25, 0.5, 0.75, 1.5, 2.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 6.0, 8.0, 16.0];
    let stretch_factor = stretch_options[rng.gen_range(0..stretch_options.len())];
    let window_type = match rng.gen_range(0..2) {
        0 => WindowType::Hamming,
        1 => WindowType::Blackman,
        _ => WindowType::Hamming,
    };
    params.update_processor_attributes(
        params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Window Size".to_string(),
                value: window_size.to_string(),
            },
            ProcessorAttribute {
                key: "Hop Size".to_string(),
                value: hop_size.to_string(),
            },
            ProcessorAttribute {
                key: "Stretch Factor".to_string(),
                value: stretch_factor.to_string(),
            },
            ProcessorAttribute {
                key: "Window Type".to_string(),
                value: format!("{:?}", window_type),
            },
        ],
    );
    let result = stft_time_stretch(
        params,
        StftTimeStretchParams {
            window_size,
            hop_size,
            stretch_factor,
            rng,
            window_type,
        },
    );

    
    complete_event!(PermuteNodeName::BlurStretch, params);
    result
} 