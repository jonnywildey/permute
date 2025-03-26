use crate::{permute_error::PermuteError, permute_files::PermuteUpdate, process::*};
use rand::prelude::*;
use strum::IntoEnumIterator;
use crate::process::{
    CrossFilterParams, 
    cross_filter,
    PermuteNodeName,
    CrossGainParams,
    cross_gain,
    CrossDistortParams,
    cross_distort,
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
        PermuteNodeName::CrossDistort => random_cross_distort,
    }
}

// Random processors

pub fn random_metallic_delay(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::MetallicDelay, params);
    let mut rng = thread_rng();

    let sec_10 = (params.sample_rate as f64 * 0.1) as usize;
    let feedback_factor = rng.gen_range(0_f64..0.9);
    let delay_sample_length = rng.gen_range(10..sec_10);
    let wet_gain_factor = rng.gen_range(0.3..1_f64);

    let delay_params = DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor: 1_f64,
        wet_gain_factor,
    };

    let mut new_params = delay_line(&params.clone(), &delay_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
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
        ],
    );

    complete_event!(PermuteNodeName::MetallicDelay, new_params);
    Ok(new_params)
}

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

pub fn random_pitch(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::RandomPitch, params);
    let mut rng = thread_rng();

    let speeds: [f64; 10] =
        [-10.0, -8.0, -7.0, -5.0, -2.0, 2.0, 5.0, 7.0, 8.0, 10.0].map(|v| 2_f64.powf(v / 12.0));

    let speed = speeds[rng.gen_range(0..speeds.len())];

    let mut new_params = change_speed(params.clone(), speed);
    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Pitch".to_string(),
                value: format_factor_to_pitch(speed),
            },
        ],
    );
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

    let predelay_ms = rng.gen_range(0.0..90.0);
    let wet_mix = rng.gen_range(0.1_f64..0.4);
    let len_factor = len_factors[rng.gen_range(0..len_factors.len())];
    let decay_factor = decay_factors[rng.gen_range(0..decay_factors.len())];

    let mut new_params = reverb(
        params,
        ReverbParams {
            predelay_ms,
            wet_mix,
            len_factor,
            decay_factor,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Predelay".to_string(),
                value:  format_float_ms(predelay_ms),
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
        ],
    );

    complete_event!(PermuteNodeName::Reverb, new_params);
    Ok(new_params)
}

pub fn random_wow(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Wow, params);
    let mut rng = thread_rng();

    let speed_hz = rng.gen_range(0.2_f64..1.6_f64);
    let depth = rng.gen_range(0.3_f64..0.7_f64);

    let mut new_params = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz,
            depth,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Speed".to_string(),
                value: format_hz(speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
        ],
    );

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
    let speed_hz = factors[rng.gen_range(0..factors.len())];
    let depth = rng.gen_range(0.3_f64..0.99_f64);

    let mut new_params = tremolo(
        params.to_owned(),
        TremoloParams {
            speed_hz,
            depth,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Speed".to_string(),
                value: format_hz(speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
        ],
    );

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
        let min_speed_hz = hz.0;
        let max_speed_hz = hz.1;
    let depth = rng.gen_range(0.5_f64..0.99_f64);
    let frame_ms = 10;

    let mut new_params = tremolo_input_mod(
        params.to_owned(),
        TremoloInputModParams {
            min_speed_hz,
            max_speed_hz,
            depth,
            frame_ms,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Min Speed".to_string(),
                value: format_hz(min_speed_hz),
            },
            ProcessorAttribute {
                key: "Max Speed".to_string(),
                value: format_hz(max_speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
            ProcessorAttribute {
                key: "Frame".to_string(),
                value: format_samples_as_ms(frame_ms, params.sample_rate),
            },
        ],
    );

    complete_event!(PermuteNodeName::Lazer, new_params);
    Ok(new_params)
}

pub fn random_flutter(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Flutter, params);
    let mut rng = thread_rng();

    let depth = rng.gen_range(0.1_f64..0.27_f64).powf(2.0); // try and push values towards lower values
    let speed_hz = rng.gen_range(5_f64..20_f64);

    let mut new_params = vibrato(
        params.to_owned(),
        VibratoParams {
            speed_hz,
            depth,
        },
    )?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Speed".to_string(),
                value: format_hz(speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
        ],
    );

    complete_event!(PermuteNodeName::Flutter, new_params);
    Ok(new_params)
}

pub fn random_chorus(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Chorus, params);

    let mut rng = thread_rng();

    let millis_low = (params.sample_rate as f64 / 1000_f64 * 7_f64) as usize;
    let millis_high = (params.sample_rate as f64 / 1000_f64 * 20_f64) as usize;
    let feedback_factor = rng.gen_range(0_f64..0.6_f64);
    let delay_sample_length = rng.gen_range(millis_low..millis_high);
    let speed_hz = rng.gen_range(0.1_f64..2.0_f64);
    let depth = rng.gen_range(0.1_f64..0.3_f64);

    let mut new_params = chorus(
        params.to_owned(),
        ChorusParams {
            delay_params: DelayLineParams {
                feedback_factor,
                delay_sample_length,
                dry_gain_factor: 1_f64,
                wet_gain_factor: 1_f64,
            },
            vibrato_params: VibratoParams {
                speed_hz,
                depth,
            },
        },
    )?;


    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Feedback".to_string(),
                value: format_float_percent(feedback_factor),
            },
            ProcessorAttribute {
                key: "Delay".to_string(),
                value: format_samples_as_ms(delay_sample_length, params.sample_rate),
            },
            ProcessorAttribute {
                key: "Speed".to_string(),
                value: format_hz(speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
        ],
    );

    complete_event!(PermuteNodeName::Chorus, new_params);
    Ok(new_params)
}

pub fn random_phaser(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Phaser, params);

    let mut rng = thread_rng();
    let stages = PhaserStages::iter().choose(&mut rng).unwrap();
    let base_freq = rng.gen_range(300.0..700.0);
    let lfo_rate = rng.gen_range(0.2..2.0);
    let q = rng.gen_range(0.15..0.5);
    let lfo_depth = rng.gen_range(0.5..0.95);

    let phaser_params = PhaserParams {
        base_freq,
        lfo_rate,
        q,
        stages: stages.clone(),
        lfo_depth,
        stage_hz: 0.0,
        dry_mix: 1.0,
        wet_mix: 1.0,
    };

    let mut new_params = phaser(&params.to_owned(), &phaser_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Stages".to_string(),
                value: format!("{:?}", stages),
            },
            ProcessorAttribute {
                key: "Base Frequency".to_string(),
                value: format_hz(base_freq),
            },
            ProcessorAttribute {
                key: "LFO Rate".to_string(),
                value: format_hz(lfo_rate),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                    value: format_float(q),
            },
            ProcessorAttribute {
                key: "LFO Depth".to_string(),
                value: format_float_percent(lfo_depth),
            },
        ],
    );

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

    let mut flanged = ProcessorParams {
        samples: summed,
        ..delayed_vib
    };

    // Update processor attributes
    flanged.update_processor_attributes(
        flanged.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Speed".to_string(),
                value: format_hz(speed_hz),
            },
            ProcessorAttribute {
                key: "Depth".to_string(),
                value: format_float_percent(depth),
            },
            ProcessorAttribute {
                key: "Delay".to_string(),
                value: format_samples_as_ms(delay_sample_length as usize, params.sample_rate),
            },
            ProcessorAttribute {
                key: "Wet".to_string(),
                value: format_float_percent(wet),
            },
        ],
    );

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

    let filter_type = types[rng.gen_range(0..types.len())];
    let frequency = freqs[rng.gen_range(0..freqs.len())];
    let q = rng.gen_range(0.15_f64..1.2_f64);
    let form = FilterForm::Form2;

    let filter_params = FilterParams {
        filter_type,
        frequency,
        q: Some(q),
        form: form.clone(),
    };

    let mut new_params = filter(params, &filter_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "Frequency".to_string(),
                value: format_hz(frequency),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

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

    let filter_type = types[rng.gen_range(0..types.len())];
    let frequency = freqs[rng.gen_range(0..freqs.len())];
    let lfo_rate = lfo_rates[rng.gen_range(0..lfo_rates.len())];
    let lfo_factor = lfo_factors[rng.gen_range(0..lfo_factors.len())];
    let q = rng.gen_range(0.5_f64..1.3_f64);
    let form = FilterForm::Form2;

    let filter_params = OscillatingFilterParams {
        filter_type,
        frequency,
        q: Some(q),
        form: form.clone(),
        lfo_rate,
        lfo_factor,
    };

    let mut new_params = oscillating_filter(params, &filter_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "Frequency".to_string(),
                value: format_hz(frequency),
            },
            ProcessorAttribute {
                key: "LFO Rate".to_string(),
                value: format_hz(lfo_rate),
            },
            ProcessorAttribute {
                key: "LFO Factor".to_string(),
                value: format_float_percent(lfo_factor),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

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

    let filter_type = types[rng.gen_range(0..types.len())];
    let hz_from = freqs[rng.gen_range(0..freqs.len())];
    let hz_to = freqs[rng.gen_range(0..freqs.len())];
    let q = rng.gen_range(0.5_f64..1.35_f64);
    let form = FilterForm::Form2;

    let filter_params = LineFilterParams {
        filter_type,
        form: form.clone(),
        hz_from,
        hz_to,
        q: Some(q),
    };

    let mut new_params = multi_line_filter(&params.to_owned(), &filter_params)?;

    // Update processor attributes
    new_params.update_processor_attributes(
        new_params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "From".to_string(),
                value: format_hz(hz_from),
            },
            ProcessorAttribute {
                key: "To".to_string(),
                value: format_hz(hz_to),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                        value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

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

pub fn format_float(value: f64) -> String {
    format!("{:.2}", value)
}

pub fn format_hz(value: f64) -> String {
    format!("{:.2} hz", value)
}

pub fn format_float_percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

pub fn format_float_ms(value: f64) -> String {
    format!("{:.2} ms", value)
}

pub fn format_samples_as_ms(samples: usize, sample_rate: usize) -> String {
    format!("{:.2} ms", (samples as f64 / sample_rate as f64) * 1000.0)
}

pub fn get_filename(path: &str) -> String {
    path.split('/').last().unwrap_or(path).to_string()
}

pub fn format_factor_to_pitch(factor: f64) -> String {
    let pitch = 12.0 * (factor / 2.0).log2();
    format!("{:.2} semitones", pitch)
}

/// Select a random processor from the processor pool
pub fn select_random_processor(processor_pool: &[PermuteNodeName]) -> PermuteNodeName {
    processor_pool[rand::thread_rng().gen_range(0..processor_pool.len())]
}
