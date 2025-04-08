// External dependencies
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;

// Internal modules
use crate::{
    process::{PermuteNodeName, ProcessorAttribute, ProcessorPlan, ProcessorParams}, 
    processors::{delay_reverb::{delay_line, DelayLineParams}, gain_distortion::{sum, SampleLine}, modulation::{
        chorus, phaser, tremolo, tremolo_input_mod, vibrato, ChorusParams, PhaserParams, PhaserStages, TremoloInputModParams, TremoloParams, VibratoParams
    }}, 
    random_processors::utils::{format_float, format_float_percent, format_hz, format_samples_as_ms},
    random_process::{start_event, complete_event},
    process::PermuteNodeEvent,
    permute_files::PermuteUpdate,
};

use super::utils::DistributionRng;

pub fn random_wow(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let speed_hz = rng.gen_range(0.2_f64..1.6_f64);
    let depth = rng.gen_range(0.3_f64..0.7_f64);

    let attributes = vec![
        ProcessorAttribute {
            key: "Speed".to_string(),
            value: format_hz(speed_hz),
        },
        ProcessorAttribute {
            key: "Depth".to_string(),
            value: format_float_percent(depth),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Wow, &params);
        let new_params = vibrato(
            params,
            VibratoParams {
                speed_hz,
                depth,
            },
        )?;
        complete_event!(PermuteNodeName::Wow, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Wow, attributes, Box::new(processor))
}

pub fn random_tremolo(params: &mut ProcessorParams) -> ProcessorPlan {
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

    let attributes = vec![
        ProcessorAttribute {
            key: "Speed".to_string(),
            value: format_hz(speed_hz),
        },
        ProcessorAttribute {
            key: "Depth".to_string(),
            value: format_float_percent(depth),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Tremolo, &params);
        let new_params = tremolo(
            params,
            TremoloParams {
                speed_hz,
                depth,
            },
        )?;
        complete_event!(PermuteNodeName::Tremolo, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Tremolo, attributes, Box::new(processor))
}

pub fn random_lazer(params: &mut ProcessorParams) -> ProcessorPlan {
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

    let attributes = vec![
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
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Lazer, &params);
        let new_params = tremolo_input_mod(
            params,
            TremoloInputModParams {
                min_speed_hz,
                max_speed_hz,
                depth,
                frame_ms,
            },
        )?;
        complete_event!(PermuteNodeName::Lazer, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Lazer, attributes, Box::new(processor))
}

pub fn random_flutter(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let depth = rng.gen_range(0.1_f64..0.27_f64).powf(2.0); // try and push values towards lower values
    let speed_hz = rng.gen_range(5_f64..20_f64);

    let attributes = vec![
        ProcessorAttribute {
            key: "Speed".to_string(),
            value: format_hz(speed_hz),
        },
        ProcessorAttribute {
            key: "Depth".to_string(),
            value: format_float_percent(depth),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Flutter, &params);
        let new_params = vibrato(
            params,
            VibratoParams {
                speed_hz,
                depth,
            },
        )?;
        complete_event!(PermuteNodeName::Flutter, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Flutter, attributes, Box::new(processor))
}

pub fn random_chorus(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let millis_low = (params.sample_rate as f64 / 1000_f64 * 7_f64) as usize;
    let millis_high = (params.sample_rate as f64 / 1000_f64 * 20_f64) as usize;
    let feedback_factor = rng.gen_range(0_f64..0.6_f64);
    let delay_sample_length = rng.gen_range(millis_low..millis_high);
    let speed_hz = rng.gen_range(0.1_f64..2.0_f64);
    let depth = rng.gen_range(0.1_f64..0.3_f64);
    let wet_gain_distributions = vec![
        (0.1, 0.1),
        (0.2, 0.1),
        (0.3, 0.2),
        (0.4, 0.2),
        (0.5, 0.3),
        (0.6, 0.3),
        (0.7, 0.2),
        (0.8, 0.2),
        (0.9, 0.1),
        (1.0, 0.1),
    ];
    let wet_gain = rng.gen_distribution(wet_gain_distributions);

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
            key: "Speed".to_string(),
            value: format_hz(speed_hz),
        },
        ProcessorAttribute {
            key: "Depth".to_string(),
            value: format_float_percent(depth),
        },
        ProcessorAttribute {
            key: "Wet".to_string(),
            value: format_float_percent(wet_gain),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Chorus, &params);
        let new_params = chorus(
            params,
            ChorusParams {
                delay_params: DelayLineParams {
                    feedback_factor,
                    delay_sample_length,
                    dry_gain_factor: 1_f64,
                    wet_gain_factor: wet_gain,
                },
                vibrato_params: VibratoParams {
                    speed_hz,
                    depth,
                },
            },
        )?;
        complete_event!(PermuteNodeName::Chorus, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Chorus, attributes, Box::new(processor))
}

pub fn random_phaser(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();
    let stages = PhaserStages::iter().choose(&mut rng).unwrap();
    let base_freq = rng.gen_range(300.0..700.0);
    let lfo_rate = rng.gen_range(0.2..2.0);
    let q = rng.gen_range(0.15..0.5);
    let lfo_depth = rng.gen_range(0.5..0.95);
    let wet_gain_distributions = vec![
        (0.1, 0.1),
        (0.2, 0.1),
        (0.3, 0.2),
        (0.4, 0.2),
        (0.5, 0.3),
        (0.6, 0.3),
        (0.7, 0.2),
        (0.8, 0.2),
        (0.9, 0.1),
        (1.0, 0.1),
    ];
    let wet_gain = rng.gen_distribution(wet_gain_distributions);
    let phaser_params = PhaserParams {
        base_freq,
        lfo_rate,
        q,
        stages: stages.clone(),
        lfo_depth,
        stage_hz: 0.0,
        dry_mix: 1.0,
        wet_mix: wet_gain,
    };

    let attributes = vec![
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
        ProcessorAttribute {
            key: "Wet".to_string(),
            value: format_float_percent(wet_gain),
        },
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Phaser, &params);
        let new_params = phaser(&params, &phaser_params)?;
        complete_event!(PermuteNodeName::Phaser, new_params);
        Ok(new_params)
    };

    (PermuteNodeName::Phaser, attributes, Box::new(processor))
}

pub fn random_zero_flange(params: &mut ProcessorParams) -> ProcessorPlan {
    let mut rng = thread_rng();

    let speed_hz = rng.gen_range(0.01_f64..1.1_f64);
    let depth = rng.gen_range(0.05_f64..0.2_f64);
    let delay_sample_length = params.sample_rate as f64 / 1000_f64 * rng.gen_range(1_f64..15_f64);
    let wet = rng.gen_range(-0.9_f64..-0.4_f64);

    let attributes = vec![
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
    ];

    let processor = move |params: ProcessorParams| {
        start_event!(PermuteNodeName::Flange, &params);
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

        let delayed = delay_line(&params, &delayed_params)?;
        let delayed_vib = vibrato(delayed, VibratoParams { speed_hz, depth })?;
        let half_delayed = delay_line(&params, &half_delayed_params)?;

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
    };

    (PermuteNodeName::Flange, attributes, Box::new(processor))
}