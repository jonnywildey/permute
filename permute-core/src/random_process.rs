use crate::{permute_error::PermuteError, permute_files::PermuteUpdate, process::*};
use rand::prelude::*;
use strum::IntoEnumIterator;

pub struct GetProcessorNodeParams {
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processor_count: Option<i32>,
}

pub fn generate_processor_sequence(
    GetProcessorNodeParams {
        normalise_at_end,
        high_sample_rate,
        depth,
        processor_pool,
        processor_count,
    }: GetProcessorNodeParams,
) -> Vec<PermuteNodeName> {
    let mut rng = thread_rng();

    let processor_count = processor_count.unwrap_or(rng.gen_range(2..5));
    let mut processors: Vec<PermuteNodeName> = vec![];

    for _ in 0..processor_count {
        processors.push(processor_pool[rng.gen_range(0..processor_pool.len())])
    }
    if depth > 1 {
        processors = [
            generate_processor_sequence(GetProcessorNodeParams {
                depth: depth - 1,
                normalise_at_end: false,
                processor_pool,
                high_sample_rate: false,
                processor_count: Some(processor_count),
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

    processors
}

pub fn get_processor_function(name: PermuteNodeName) -> ProcessorFn {
    match name {
        PermuteNodeName::TimeStretch => random_time_stretch,
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
        PermuteNodeName::Wow => random_wow,
        PermuteNodeName::Normalise => normalise,
        PermuteNodeName::SampleRateConversionHigh => change_sample_rate_high,
        PermuteNodeName::SampleRateConversionOriginal => change_sample_rate_original,
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

pub fn random_time_stretch(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::TimeStretch, params);

    let mut rng = thread_rng();
    // let grain = [
    //     200, 400, 600, 1000, 1600, 2000, 2200, 2400, 2600, 2800, 3000, 4000, 10000, 20000,
    // ];
    let grain = [10000];
    let stretch = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4];
    // let blend = [10, 12, 16, 20, 8, 80, 160, 1200, 2000, 4000];
    let blend = [100];

    let new_params = time_stretch_cross(
        &params,
        TimeStretchParams {
            grain_samples: grain[rng.gen_range(0..grain.len())],
            stretch_factor: stretch[rng.gen_range(0..stretch.len())],
            blend_samples: blend[rng.gen_range(0..blend.len())],
        },
    )?;
    complete_event!(PermuteNodeName::TimeStretch, new_params);
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

    let millis_low = (params.sample_rate as f64 / 1000_f64 * 4_f64) as usize;
    let millis_high = (params.sample_rate as f64 / 1000_f64 * 20_f64) as usize;
    let delay_params = DelayLineParams {
        feedback_factor: rng.gen_range(0_f64..0.8_f64),
        delay_sample_length: rng.gen_range(millis_low..millis_high),
        dry_gain_factor: 1_f64,
        wet_gain_factor: rng.gen_range(0.7..1_f64),
    };

    let vibrato_params = VibratoParams {
        speed_hz: rng.gen_range(0.5_f64..5_f64),
        depth: rng.gen_range(0.2_f64..0.4_f64),
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

pub fn normalise(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Normalise, params);

    let new_params = ceiling(params.to_owned(), 1_f64);
    complete_event!(PermuteNodeName::Normalise, new_params);
    Ok(new_params)
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
