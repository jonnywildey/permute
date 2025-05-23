// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    permute_files::PermuteUpdate, 
    process::{PermuteNodeEvent, PermuteNodeName, ProcessorAttribute, ProcessorPlan, ProcessorClosure, ProcessorParams}, 
    processors::time_pitch::{
        change_sample_rate, change_speed, reverse, stft_time_stretch, time_stretch_cross, StftTimeStretchParams, TimeStretchParams, WindowType
    }, 
    random_process::{complete_event, start_event}, random_processors::utils::{format_factor_to_pitch, DistributionRng}
};

use super::utils::{format_float_ms, format_hz_usize};

pub fn random_pitch(_params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();
    let speeds: [f64; 10] = [-10.0, -8.0, -7.0, -5.0, -2.0, 2.0, 5.0, 7.0, 8.0, 10.0].map(|v| 2_f64.powf(v / 12.0));
    let speed = speeds[rng.gen_range(0..speeds.len())];

    let attributes = vec![
        ProcessorAttribute {
            key: "Pitch".to_string(),
            value: format_factor_to_pitch(speed),
        },
        ProcessorAttribute {
            key: "Length Factor".to_string(),
            value: format!("{:.2}x", 1.0 / speed),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::RandomPitch, &params);
        let new_params: ProcessorParams = change_speed(params, speed);
        complete_event!(PermuteNodeName::RandomPitch, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::RandomPitch, attributes, Box::new(processor))
}

pub fn random_granular_time_stretch(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();
    let grain_distributions = vec![
        (5.0, 0.1),    // 5ms
        (10.0, 0.1),   // 10ms
        (15.0, 0.1),   // 15ms
        (25.0, 0.2),   // 25ms
        (40.0, 0.2),   // 40ms
        (50.0, 0.3),   // 50ms
        (55.0, 0.3),   // 55ms
        (60.0, 0.3),   // 60ms
        (65.0, 0.2),   // 65ms
        (70.0, 0.2),   // 70ms
        (75.0, 0.2),   // 75ms
        (100.0, 0.1),  // 100ms
        (250.0, 0.1),  // 250ms
        (500.0, 0.1),  // 500ms
    ];
    let mut grain_ms = rng.gen_distribution(grain_distributions);
    let grain_samples = ((grain_ms / 1000.0) * params.sample_rate as f64) as usize;
    
    let stretch_distributions = vec![
        (2, 1.0),
        (3, 0.1),
        (4, 0.1),
        (5, 0.1),
        (6, 0.1),
    ];
    let stretch_factor = rng.gen_distribution(stretch_distributions);
    
    let blend_distributions = vec![
        (0.5, 1.0),
        (1.0, 0.1),
        (2.0, 0.1),
        (3.5, 0.1),
        (4.0, 0.1),
        (4.5, 0.1),
        (5.0, 0.1),
        (5.5, 0.1),
        (8.5, 0.1),
        (10.0, 0.3),
        (12.5, 0.3),
        (20.5, 0.3),   
        (25.5, 0.3),   
        (30.0, 0.1),
        (35.0, 0.1),
        (50.0, 0.1),
        (73.0, 0.1),
        (80.0, 0.1),
        (90.0, 0.1),
        (100.0, 0.1),
    ];
    let blend_ms = rng.gen_distribution(blend_distributions);
    if blend_ms > grain_ms {
        grain_ms = blend_ms + grain_ms;
    }
    let blend_samples = ((blend_ms / 1000.0) * params.sample_rate as f64) as usize;

    let attributes = vec![
        ProcessorAttribute {
            key: "Grain".to_string(),
            value: format_float_ms(grain_ms),
        },
        ProcessorAttribute {
            key: "Stretch Factor".to_string(),
            value: format!("{}", stretch_factor),
        },
        ProcessorAttribute {
            key: "Blend".to_string(),
            value: format_float_ms(blend_ms),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::GranularTimeStretch, &params);
        let time_stretch_params = TimeStretchParams {
            grain_samples,
            stretch_factor,
            blend_samples,
        };
        let new_params = time_stretch_cross(&params, time_stretch_params)?;
        complete_event!(PermuteNodeName::GranularTimeStretch, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::GranularTimeStretch, attributes, Box::new(processor))
}

pub fn half_speed(_params: &mut ProcessorParams) -> ProcessorPlan {
    let attributes = vec![
        ProcessorAttribute {
            key: "Speed".to_string(),
            value: "0.5x".to_string(),
        },
        ProcessorAttribute {
            key: "Length Factor".to_string(),
            value: "2.0".to_string(),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::HalfSpeed, &params);
        let new_params = change_speed(params, 0.5_f64);
        complete_event!(PermuteNodeName::HalfSpeed, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::HalfSpeed, attributes, Box::new(processor))
}

pub fn double_speed(_params: &mut ProcessorParams) -> ProcessorPlan {
    let attributes = vec![
        ProcessorAttribute {
            key: "Speed".to_string(),
            value: "2.0x".to_string(),
        },
        ProcessorAttribute {
            key: "Length Factor".to_string(),
            value: "0.5".to_string(),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::DoubleSpeed, &params);
        let new_params = change_speed(params, 2_f64);
        complete_event!(PermuteNodeName::DoubleSpeed, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::DoubleSpeed, attributes, Box::new(processor))
}

pub fn random_blur_stretch(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = rand::thread_rng();
    // Randomize window size between 1024 and 4096 samples
    let window_distributions = vec![
        (4096, 0.1),
        (5000, 0.1),
        (6000, 0.2),
        (8192, 0.3),
        (10000, 0.2),
        (10240, 0.2),
        (12288, 0.2),
        (16384, 0.1),
        (32768, 0.1),
        (64000, 0.1),
        (128000, 0.1),
    ];
    let window_size = rng.gen_distribution(window_distributions);
    // Randomize hop size between 1/4 and 1/2 of window size
    let hop_options = [window_size/4, window_size/2, window_size/3, window_size/4];
    let hop_size = hop_options[rng.gen_range(0..hop_options.len())];
    // Randomize stretch factor between 0.5 and 2.0
    let stretch_distributions = vec![
        (0.25, 0.1),
        (0.5, 0.1),
        (0.75, 0.1),
        (1.5, 0.1),
        (2.0, 0.2),
        (3.0, 0.2),
        (4.0, 0.2),
        (6.0, 0.1),
        (8.0, 0.1),
        (16.0, 0.025),
    ];
    let stretch_factor = rng.gen_distribution(stretch_distributions);
    let window_type = match rng.gen_range(0..2) {
        0 => WindowType::Hamming,
        1 => WindowType::Blackman,
        _ => WindowType::Hamming,
    };

    let attributes = vec![
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
            value: format!("{}", stretch_factor),
        },
        ProcessorAttribute {
            key: "Window Type".to_string(),
            value: format!("{:?}", window_type),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::BlurStretch, &params);
        let result = stft_time_stretch(
            &params,
            StftTimeStretchParams {
                window_size,
                hop_size,
                stretch_factor,
                rng,
                window_type,
            },
        );
        complete_event!(PermuteNodeName::BlurStretch, &params);
        result
    };

    (PermuteNodeName::BlurStretch, attributes, Box::new(processor))
} 


pub fn reverse_with_plan(_params: &mut ProcessorParams) -> ProcessorPlan {
    let attributes = vec![];
    
    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Reverse, &params);
        let new_params = reverse(params)?;
        complete_event!(PermuteNodeName::Reverse, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Reverse, attributes, Box::new(processor))
}

pub fn change_sample_rate_high(params: &mut ProcessorParams) -> ProcessorPlan {
    let new_sample_rate = match params.sample_rate {
        0..=48000 => params.sample_rate * 4,
        48001..=96000 => params.sample_rate * 2,
        _ => params.sample_rate,
    };
    let attributes = vec![
        ProcessorAttribute {
            key: "Sample Rate".to_string(),
            value: format_hz_usize(new_sample_rate),
        },
        ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::SampleRateConversionHigh, &params);
        let new_params = change_sample_rate(params, new_sample_rate)?;
        complete_event!(PermuteNodeName::SampleRateConversionHigh, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::SampleRateConversionHigh, attributes, Box::new(processor))
}

pub fn change_sample_rate_original(
    params: &mut ProcessorParams,
) -> ProcessorPlan {
    let new_sample_rate = params.permutation.original_sample_rate;
    let attributes = vec![
        ProcessorAttribute {
            key: "Sample Rate".to_string(),
            value: format_hz_usize(new_sample_rate),
        },
        ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::SampleRateConversionHigh, &params);
        let new_params = change_sample_rate(params, new_sample_rate)?;
        complete_event!(PermuteNodeName::SampleRateConversionHigh, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::SampleRateConversionOriginal, attributes, Box::new(processor))
}