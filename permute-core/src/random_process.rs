use crate::{permute_error::PermuteError, permute_files::PermuteUpdate, process::*};
use rand::prelude::*;
use strum::IntoEnumIterator;
use crate::process::{
    CrossFilterParams, 
    cross_filter,
    PermuteNodeName,
    CrossGainParams,
    cross_gain,
};
use rand::Rng;

const MAX_LENGTH_INCREASING: usize = 3;

pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
    pub trim_at_end: bool,
    pub high_sample_rate: bool,
    pub depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processor_count: Option<i32>,
    pub constrain_length: bool,
}

macro_rules! start_event {
    ($name:expr, $params:expr) => {{
        $params
            .update_sender
            .send(PermuteUpdate::UpdatePermuteNodeStarted(
                $params.permutation.clone(),
                $name,
                PermuteNodeEvent::NodeProcessStarted,
            ))?;
    }};
}
pub(crate) use start_event;

macro_rules! complete_event {
    ($name:expr, $params:expr) => {{
        $params
            .update_sender
            .send(PermuteUpdate::UpdatePermuteNodeCompleted(
                $params.permutation.clone(),
                $name,
                PermuteNodeEvent::NodeProcessComplete,
            ))?;
    }};
}
pub(crate) use complete_event;

pub fn generate_processor_sequence(
    GetProcessorNodeParams {
        normalise_at_end,
        trim_at_end,
        high_sample_rate,
        depth,
        processor_pool,
        processor_count,
        constrain_length,
    }: GetProcessorNodeParams,
) -> Vec<PermuteNodeName> {
    let mut rng = thread_rng();
    let processor_count = processor_count.unwrap_or(rng.gen_range(2..5));
    let mut processors: Vec<PermuteNodeName> = vec![];

    for _ in 0..processor_count {
        let available_processors: Vec<PermuteNodeName> = if constrain_length && count_length_increasing(&processors) >= MAX_LENGTH_INCREASING {
            // If we've hit the length increase limit, filter out length-increasing processors
                processor_pool
                    .iter()
                    .filter(|p| !is_length_increasing(p))
                    .cloned()
                    .collect()
        } else {
            processor_pool.clone()
        };

        if !available_processors.is_empty() {
            processors.push(available_processors[rng.gen_range(0..available_processors.len())]);
        }
    }

    if depth > 1 {
        processors = [
            generate_processor_sequence(GetProcessorNodeParams {
                depth: depth - 1,
                normalise_at_end: false,
                trim_at_end: false,
                processor_pool,
                high_sample_rate: false,
                processor_count: Some(processor_count),
                constrain_length,
            }),
            processors,
        ]
        .concat();
    }

    if high_sample_rate {
        processors.insert(0, PermuteNodeName::SampleRateConversionHigh);
        processors.push(PermuteNodeName::SampleRateConversionOriginal);
    }
    if normalise_at_end {
        processors.push(PermuteNodeName::Normalise);
    }
    if trim_at_end {
        processors.push(PermuteNodeName::Trim);
    }

    processors
}

fn is_length_increasing(processor: &PermuteNodeName) -> bool {
    matches!(
        processor,
        PermuteNodeName::GranularTimeStretch | PermuteNodeName::HalfSpeed
    )
}

fn count_length_increasing(processors: &[PermuteNodeName]) -> usize {
    processors.iter().filter(|p| is_length_increasing(p)).count()
}

pub fn get_processor_function(name: PermuteNodeName) -> ProcessorFn {
    match name {
        PermuteNodeName::GranularTimeStretch => random_granular_time_stretch,
        PermuteNodeName::Fuzz => random_fuzz,
        PermuteNodeName::Saturate => random_saturate,
        PermuteNodeName::Reverse => reverse,
        PermuteNodeName::Chorus => random_chorus,
        PermuteNodeName::Phaser => random_phaser,
        PermuteNodeName::DoubleSpeed => double_speed,
        PermuteNodeName::RandomPitch => random_pitch,
        PermuteNodeName::Flutter => random_flutter,
        PermuteNodeName::Flange => random_zero_flange,
        PermuteNodeName::HalfSpeed => half_speed,
        PermuteNodeName::MetallicDelay => random_metallic_delay,
        PermuteNodeName::RhythmicDelay => random_rhythmic_delay,
        PermuteNodeName::Reverb => random_reverb,
        PermuteNodeName::Wow => random_wow,
        PermuteNodeName::Tremolo => random_tremolo,
        PermuteNodeName::Lazer => random_lazer,
        PermuteNodeName::Normalise => normalise,
        PermuteNodeName::Trim => trim,
        PermuteNodeName::SampleRateConversionHigh => change_sample_rate_high,
        PermuteNodeName::SampleRateConversionOriginal => change_sample_rate_original,
        PermuteNodeName::Filter => random_filter,
        PermuteNodeName::LineFilter => random_line_filter,
        PermuteNodeName::OscillatingFilter => random_oscillating_filter,
        PermuteNodeName::CrossGain => random_cross_gain,
        PermuteNodeName::CrossFilter => random_cross_filter,
    }
}

// Random processors

pub fn random_metallic_delay(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::MetallicDelay, params);
    let mut rng = thread_rng();

    let sec_10 = (params.sample_rate as f64 * 0.1) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(10..sec_10),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.3..1_f64),
    };

    let new_params = delay_line(&params.clone(), &delay_params)?;
    complete_event!(PermuteNodeName::MetallicDelay, new_params);
    Ok(new_params)
}

pub fn random_fuzz(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Fuzz, params);
    let mut rng = thread_rng();

    let factors = [0.6, 0.75, 0.8, 0.95, 1.25];
    let factor = factors[rng.gen_range(0..factors.len())];

    let new_params = distort(&params.clone(), factor)?;
    complete_event!(PermuteNodeName::Fuzz, new_params);
    Ok(new_params)
}

pub fn random_saturate(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Saturate, params);

    let new_params = saturate(&params.clone())?;
    complete_event!(PermuteNodeName::Saturate, new_params);
    Ok(new_params)
}

pub fn random_pitch(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::RandomPitch, params);
    let mut rng = thread_rng();

    let speeds: [f64; 10] =
        [-10.0, -8.0, -7.0, -5.0, -2.0, 2.0, 5.0, 7.0, 8.0, 10.0].map(|v| 2_f64.powf(v / 12.0));

    let speed = speeds[rng.gen_range(0..speeds.len())];

    let new_params = change_speed(params.clone(), speed);
    complete_event!(PermuteNodeName::RandomPitch, new_params);

    Ok(new_params)
}

pub fn random_granular_time_stretch(
    params: &ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::GranularTimeStretch, params);

    let mut rng = thread_rng();
    let grain = [
        200, 400, 600, 1000, 1600, 2000, 2200, 2400, 2600, 2800, 3000, 4000, 10000, 20000,
    ];
    // let grain = [2000];
    let stretch = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4];
    let blend = [
        20, 40, 80, 100, 140, 160, 180, 200, 220, 240, 300, 340, 400, 500, 1200, 2000, 4000,
    ];
    // let blend = [80];

    let new_params = time_stretch_cross(
        &params,
        TimeStretchParams {
            grain_samples: grain[rng.gen_range(0..grain.len())],
            stretch_factor: stretch[rng.gen_range(0..stretch.len())],
            blend_samples: blend[rng.gen_range(0..blend.len())],
        },
    )?;
    complete_event!(PermuteNodeName::GranularTimeStretch, new_params);
    Ok(new_params)
}

pub fn random_rhythmic_delay(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::RhythmicDelay, params);

    let mut rng = thread_rng();

    let sec_10 = (params.sample_rate as f64 * 0.1) as usize;
    let sec = params.sample_rate as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.9),
        delay_sample_length: rng.gen_range(sec_10..sec),
        dry_gain_factor: 1_f64,
        wet_gain_factor: 1_f64,
    };

    let new_params = delay_line(&params, &delay_params)?;
    complete_event!(PermuteNodeName::RhythmicDelay, new_params);

    Ok(new_params)
}

pub fn half_speed(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::HalfSpeed, params);
    let new_params = change_speed(params.to_owned(), 0.5_f64);
    complete_event!(PermuteNodeName::HalfSpeed, new_params);
    Ok(new_params)
}
pub fn double_speed(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::DoubleSpeed, params);
    let new_params = change_speed(params.to_owned(), 2_f64);
    complete_event!(PermuteNodeName::DoubleSpeed, new_params);
    Ok(new_params)
}

pub fn random_reverb(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Reverb, params);

    let mut rng = thread_rng();

    let len_factors = [0.1, 0.3, 0.6, 1.0, 1.2, 1.4];
    let decay_factors = [0.2, 0.3, 0.325, 0.35, 0.4];

    let reverb_params = ReverbParams {
        predelay_ms: rng.gen_range(0.0..90.0),
        wet_mix: rng.gen_range(0.1_f64..0.4),
        len_factor: len_factors[rng.gen_range(0..len_factors.len())],
        decay_factor: decay_factors[rng.gen_range(0..decay_factors.len())],
    };

    let new_params = reverb(&params, reverb_params)?;
    complete_event!(PermuteNodeName::Reverb, new_params);
    Ok(new_params)
}

pub fn random_wow(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Wow, params);
    let mut rng = thread_rng();

    let new_params = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz: rng.gen_range(0.2_f64..1.6_f64),
            depth: rng.gen_range(0.3_f64..0.7_f64),
        },
    )?;
    complete_event!(PermuteNodeName::Wow, new_params);
    Ok(new_params)
}

pub fn random_tremolo(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Tremolo, params);
    let mut rng = thread_rng();

    let factors = [
        rng.gen_range(0.2_f64..1_f64),
        rng.gen_range(0.5_f64..2_f64),
        rng.gen_range(0.8_f64..3_f64),
        rng.gen_range(1_f64..10_f64),
        rng.gen_range(8_f64..300_f64),
    ];

    let new_params = tremolo(
        params.to_owned(),
        TremoloParams {
            speed_hz: factors[rng.gen_range(0..factors.len())],
            depth: rng.gen_range(0.3_f64..0.99_f64),
        },
    )?;
    complete_event!(PermuteNodeName::Tremolo, new_params);
    Ok(new_params)
}

pub fn random_lazer(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Lazer, params);
    let mut rng = thread_rng();

    let hz_options = [
        (
            rng.gen_range(0.2_f64..10_f64),
            rng.gen_range(10_f64..50_f64),
        ),
        (
            rng.gen_range(10_f64..40_f64),
            rng.gen_range(40_f64..120_f64),
        ),
        (
            rng.gen_range(60_f64..100_f64),
            rng.gen_range(140_f64..300_f64),
        ),
        (
            rng.gen_range(10_f64..11_f64),
            rng.gen_range(500_f64..2000_f64),
        ),
        (
            rng.gen_range(200_f64..500_f64),
            rng.gen_range(1000_f64..5000_f64),
        ),
        (
            rng.gen_range(1_f64..100_f64),
            rng.gen_range(8000_f64..20000_f64),
        ),
    ];

    let hz = hz_options[rng.gen_range(0..hz_options.len())];

    let new_params = tremolo_input_mod(
        params.to_owned(),
        TremoloInputModParams {
            min_speed_hz: hz.0,
            max_speed_hz: hz.1,
            depth: rng.gen_range(0.5_f64..0.99_f64),
            frame_ms: 10,
        },
    )?;
    complete_event!(PermuteNodeName::Lazer, new_params);
    Ok(new_params)
}

pub fn random_flutter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Flutter, params);
    let mut rng = thread_rng();

    let depth = rng.gen_range(0.1_f64..0.27_f64).powf(2.0); // try and push values towards lower values
    let new_params = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz: rng.gen_range(5_f64..20_f64),
            depth,
        },
    )?;
    complete_event!(PermuteNodeName::Flutter, new_params);
    Ok(new_params)
}

pub fn random_chorus(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Chorus, params);

    let mut rng = thread_rng();

    let millis_low = (params.sample_rate as f64 / 1000_f64 * 7_f64) as usize;
    let millis_high = (params.sample_rate as f64 / 1000_f64 * 20_f64) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.6_f64),
        delay_sample_length: rng.gen_range(millis_low..millis_high),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.7..1_f64),
    };

    let vibrato_params = VibratoParams {
        speed_hz: rng.gen_range(0.5_f64..5_f64),
        depth: rng.gen_range(0.03_f64..0.2_f64),
    };

    let new_params = chorus(
        params.to_owned(),
        ChorusParams {
            delay_params,
            vibrato_params,
        },
    )?;
    complete_event!(PermuteNodeName::Chorus, new_params);
    Ok(new_params)
}

pub fn random_phaser(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Phaser, params);

    let mut rng = thread_rng();
    let stages = PhaserStages::iter().choose(&mut rng).unwrap();

    let phaser_params = PhaserParams {
        base_freq: rng.gen_range(300.0..700.0),
        lfo_rate: rng.gen_range(0.2..2.0), // Maybe a separate one for faster?
        q: rng.gen_range(0.15..0.5),
        stages: stages,
        lfo_depth: rng.gen_range(0.5..0.95),
        stage_hz: 0.0,
        dry_mix: 1.0,
        wet_mix: 1.0,
    };

    let new_params = phaser(&params.to_owned(), &phaser_params)?;
    complete_event!(PermuteNodeName::Phaser, new_params);
    Ok(new_params)
}

pub fn random_zero_flange(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Flange, params);
    let mut rng = thread_rng();

    let speed_hz = rng.gen_range(0.01_f64..1.1_f64);
    let depth = rng.gen_range(0.05_f64..0.2_f64);
    let delay_sample_length = params.sample_rate as f64 / 1000_f64 * rng.gen_range(1_f64..15_f64);
    let wet = rng.gen_range(-0.9_f64..-0.4_f64);

    let delayed_params = DelayLineParams {
        feedback_factor: 0.0,
        delay_sample_length: delay_sample_length as usize,
        dry_gain_factor: 0.0,
        wet_gain_factor: 1.0,
    };
    let half_delayed_params = DelayLineParams {
        feedback_factor: 0.0,
        delay_sample_length: delay_sample_length as usize / 2,
        dry_gain_factor: 0.0,
        wet_gain_factor: 1.0,
    };

    let delayed = delay_line(params, &delayed_params)?;
    let delayed_vib = vibrato(delayed, VibratoParams { speed_hz, depth })?;
    let half_delayed = delay_line(params, &half_delayed_params)?;

    let summed = sum(vec![
        SampleLine {
            samples: delayed_vib.samples,
            gain_factor: 1_f64,
        },
        SampleLine {
            samples: half_delayed.samples,
            gain_factor: wet,
        },
    ]);

    let flanged = ProcessorParams {
        samples: summed,
        ..delayed_vib
    };

    complete_event!(PermuteNodeName::Flange, flanged);
    Ok(flanged)
}

pub fn random_filter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Filter, params);

    let mut rng = thread_rng();

    let freqs = [
        200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 800.0, 1000.0, 1200.0, 1600.0, 2000.0, 2400.0,
        3200.0, 4000.0, 4800.0, 6400.0,
    ];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::BandPass,
    ];

    let filter_params = FilterParams {
        filter_type: types[rng.gen_range(0..types.len())],
        form: FilterForm::Form2,
        frequency: freqs[rng.gen_range(0..freqs.len())],
        q: Some(rng.gen_range(0.15_f64..1.2_f64)),
    };

    let new_params = multi_channel_filter(&params.to_owned(), &filter_params)?;

    complete_event!(PermuteNodeName::Filter, new_params);
    Ok(new_params)
}

pub fn random_oscillating_filter(
    params: &ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::OscillatingFilter, params);

    let mut rng = thread_rng();

    let freqs = [
        200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 800.0, 1000.0, 1200.0, 1600.0, 2000.0, 2400.0,
        3200.0, 4000.0, 4800.0, 6400.0,
    ];
    let lfo_rates = [
        0.2, 0.3, 0.4, 0.5, 0.75, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 15.0,
        25.0, 45.0, 80.0,
    ];
    let lfo_factors = [0.5, 0.6, 0.7, 0.8, 0.9, 0.95];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::BandPass,
    ];

    let filter_params = OscillatingFilterParams {
        filter_type: types[rng.gen_range(0..types.len())],
        form: FilterForm::Form2,
        frequency: freqs[rng.gen_range(0..freqs.len())],
        lfo_rate: lfo_rates[rng.gen_range(0..lfo_rates.len())],
        lfo_factor: lfo_factors[rng.gen_range(0..lfo_factors.len())],
        q: Some(rng.gen_range(0.5_f64..1.3_f64)),
    };

    let new_params = multi_oscillating_filter(&params.to_owned(), &filter_params)?;

    complete_event!(PermuteNodeName::OscillatingFilter, new_params);
    Ok(new_params)
}

pub fn random_line_filter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::LineFilter, params);

    let mut rng = thread_rng();

    let freqs = [
        100.0, 150.0, 160.0, 175.0, 200.0, 220.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0,
        600.0, 650.0, 800.0, 850.0, 1000.0, 1050.0, 1200.0, 1250.0, 1600.0, 2000.0, 2400.0, 3200.0,
        3500.0, 4000.0, 4800.0, 5200.0, 6400.0, 8000.0, 8500.0, 10000.0, 12000.0, 13000.0, 14000.0,
        15000.0,
    ];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::LowPass, // make low pass most likely
        biquad::Type::BandPass,
    ];

    let filter_params = LineFilterParams {
        filter_type: types[rng.gen_range(0..types.len())],
        form: FilterForm::Form2,
        hz_from: freqs[rng.gen_range(0..freqs.len())],
        hz_to: freqs[rng.gen_range(0..freqs.len())],
        q: Some(rng.gen_range(0.5_f64..1.35_f64)),
    };

    let new_params = multi_line_filter(&params.to_owned(), &filter_params)?;

    complete_event!(PermuteNodeName::LineFilter, new_params);
    Ok(new_params)
}

pub fn normalise(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Normalise, params);

    let new_params = ceiling(params.to_owned(), 1_f64);
    complete_event!(PermuteNodeName::Normalise, new_params);
    Ok(new_params)
}

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

    let cross_params = CrossGainParams {
        sidechain_file,
        depth: rng.gen_range(0.2..0.9), // Fixed depth for now, could be randomized
        invert: rng.gen_bool(0.5),
        window_size_ms: 100.0, // 10ms window size
    };

    let new_params = cross_gain(params, &cross_params)?;
    complete_event!(PermuteNodeName::CrossGain, new_params);
    Ok(new_params)
}

pub fn random_cross_filter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::CrossFilter, params);
    let mut rng = rand::thread_rng();
    
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

    // Base frequency between 200Hz and 2000Hz
    let base_freq = rng.gen_range(50.0..800.0);
    // Maximum frequency between base_freq and 10000Hz
    let max_freq = rng.gen_range(base_freq..10000.0);
    // Q factor between 0.5 and 1.35 (similar to random_line_filter)
    let q = rng.gen_range(0.5..1.35);
    
    let cross_params = CrossFilterParams {
        sidechain_file,
        filter_type,
        base_freq,
        max_freq,
        q,
        window_size_ms: 100.0, // Fixed 10ms window for RMS calculation
        invert: rng.gen_bool(0.5),
    };

    let result = cross_filter(params, &cross_params);
    complete_event!(PermuteNodeName::CrossFilter, params);
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
