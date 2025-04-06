
use std::f64::consts::PI;

use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Q_BUTTERWORTH_F64};

use crate::{
    permute_error::PermuteError, 
    process::ProcessorParams,
    processors::filter::FilterType,
    processors::osc::lfo_sin,
};

use super::gain_distortion::{sum, SampleLine};

pub struct ReverbParams {
    pub predelay_ms: f64,
    pub wet_mix: f64,
    pub len_factor: f64,
    pub decay_factor: f64, // be careful
}

// Reverb processor. 80s sounding reverb
pub fn reverb(
    params: &ProcessorParams,
    ReverbParams {
        predelay_ms,
        wet_mix,
        len_factor,
        decay_factor,
    }: ReverbParams,
) -> Result<ProcessorParams, PermuteError> {
    let comb_delays_ms: Vec<(f64, f64, f64, f64)> = vec![
        (
            (11.73 * len_factor) + predelay_ms,
            decay_factor - 0.1313,
            0.0,
            5.0 * len_factor,
        ),
        (
            (19.31 * len_factor) + predelay_ms,
            decay_factor - 0.2743,
            1.0,
            10.0 * len_factor,
        ),
        (
            (7.97 * len_factor) + predelay_ms,
            decay_factor - 0.31,
            PI * 0.5,
            2.2 * len_factor,
        ),
    ];

    let comb_delays_gain = 1_f64 / comb_delays_ms.len() as f64;

    let comb_filters = comb_delays_ms
        .clone()
        .into_iter()
        .map(|(ms, df, phase, lfo_rate)| {
            let comb_samples = modulated_comb_filter(
                params.samples.clone(),
                params.sample_rate,
                ms,
                lfo_rate,
                phase,
                df,
            );
            SampleLine {
                gain_factor: comb_delays_gain,
                samples: comb_samples,
            }
        })
        .collect();

    let summed = sum(comb_filters);

    let all_pass_params_1 = vec![
        (
            decay_factor,
            89.27 * len_factor,
            7.89 * len_factor,
            1.2,
            5000.0 * len_factor,
        ),
        (
            decay_factor * 0.5,
            58.5 * len_factor,
            12.3 * len_factor,
            0.5,
            2500.0 * len_factor,
        ),
    ];
    let multi_pass_1 = verb_block(all_pass_params_1, summed.clone(), params)?;

    let all_pass_params_2 = vec![
        (
            decay_factor,
            109.27 * len_factor,
            4.3 * len_factor,
            0.7,
            4000.0 * len_factor,
        ),
        (
            decay_factor * 0.5,
            135.5 * len_factor,
            16.5 * len_factor,
            3.2,
            1750.0 * len_factor,
        ),
    ];
    let multi_pass_2 = verb_block(all_pass_params_2, summed.clone(), params)?;

    let all_pass_params_3 = vec![
        (
            decay_factor,
            59.27 * len_factor,
            1.5 * len_factor,
            1.8,
            3000.0 * len_factor,
        ),
        (
            decay_factor * 0.5,
            155.5 * len_factor,
            9.5 * len_factor,
            3.7,
            1400.0 * len_factor,
        ),
    ];
    let multi_pass_3 = verb_block(all_pass_params_3, summed, params)?;

    let summed = sum(vec![
        SampleLine {
            gain_factor: 0.5,
            samples: multi_pass_1,
        },
        SampleLine {
            gain_factor: 0.35,
            samples: multi_pass_2,
        },
        SampleLine {
            gain_factor: 0.5,
            samples: multi_pass_3,
        },
    ]);

    let mixed = sum(vec![
        SampleLine {
            samples: params.samples.clone(),
            gain_factor: 1.0,
        },
        SampleLine {
            gain_factor: wet_mix,
            samples: summed,
        },
    ]);
    Ok(ProcessorParams {
        samples: mixed,
        ..params.clone()
    })
}

// Verb block for reverb
fn verb_block(
    all_pass_params: Vec<(f64, f64, f64, f64, f64)>,
    summed: Vec<f64>,
    params: &ProcessorParams,
) -> Result<Vec<f64>, PermuteError> {
    let mut all_passed = vec![];
    let all_pass_gain = 1_f64 / all_pass_params.len() as f64;
    all_pass_params
        .into_iter()
        .fold(summed, |acc, (df, ds, comb_ms, lfo_rate, freq)| {
            let low_pass_coeffs = Coefficients::<f64>::from_params(
                FilterType::LowPass,
                (params.sample_rate as u32).hz(),
                freq.hz(),
                Q_BUTTERWORTH_F64,
            )
            .unwrap();
            let ap_1 = all_pass(&acc, ds, df, params.sample_rate);
            let ap_2 = all_pass(&ap_1, ds, df, params.sample_rate);
            let combed =
                modulated_comb_filter(ap_2, params.sample_rate, comb_ms, lfo_rate, 0.0, 0.5);
            let mut filt = DirectForm1::<f64>::new(low_pass_coeffs);
            let lowpassed: Vec<f64> = combed.into_iter().map(|x| filt.run(x)).collect();
            all_passed.push(lowpassed.clone());
            lowpassed
        });
    let all_pass_lines: Vec<SampleLine> = all_passed
        .into_iter()
        .map(|ap| SampleLine {
            gain_factor: all_pass_gain,
            samples: ap,
        })
        .collect();
    let multi_pass = sum(all_pass_lines);
    Ok(multi_pass)
}

// Modulated comb filter for reverb
fn modulated_comb_filter(
    samples: Vec<f64>,
    sample_rate: usize,
    delay_ms: f64,
    lfo_rate: f64,
    phase: f64,
    decay_factor: f64,
) -> Vec<f64> {
    let sample_len = samples.len();
    let delay_s = ((sample_rate as f64) / (1000.0 / delay_ms)) as usize;
    let mut comb_samples = vec![0_f64; sample_len];
    for i in 0..sample_len - delay_s {
        let amplitude = lfo_sin(i, sample_rate, lfo_rate, phase);
        let speed = 1_f64 + amplitude * 10.0;

        let offset_f = (i as f64 - 1_f64) + speed;
        let offset = offset_f.floor() as usize;
        let frac = offset_f - offset as f64;

        let ptr1 = offset;
        let ptr2 = ptr1 + 1;

        if ptr1 >= samples.len() - 1 {
            break;
        }

        let v = samples[ptr1] + (samples[ptr2] - samples[ptr1]) * frac;

        comb_samples[i + delay_s] = v + (v * decay_factor);
    }
    comb_samples
}

// All pass filter for reverb
fn all_pass(samples: &Vec<f64>, delay_ms: f64, decay_factor: f64, sample_rate: usize) -> Vec<f64> {
    let delay_samples = ((sample_rate as f64) / (1000.0 / delay_ms)) as usize;
    let mut all_passed = vec![0_f64; samples.len()];
    for i in delay_samples..samples.len() {
        all_passed[i] += samples[i - delay_samples];
        all_passed[i] += -decay_factor * all_passed[i - delay_samples];
        all_passed[i] += decay_factor * all_passed[i];
    }
    all_passed
}

pub struct DelayLineParams {
    pub feedback_factor: f64, // 0 - 1
    pub delay_sample_length: usize,
    pub dry_gain_factor: f64,
    pub wet_gain_factor: f64,
}

pub fn delay_line(
    ProcessorParams {
        samples,
        sample_length,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        update_sender,
        permutation,
    }: &ProcessorParams,
    DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor,
        wet_gain_factor,
    }: &DelayLineParams,
) -> Result<ProcessorParams, PermuteError> {
    // Ensure sample length matches channel count
    let sample_length = *sample_length;
    let delay_sample_length = delay_sample_length - (delay_sample_length % *channels);
    let mut new_samples = vec![0_f64; sample_length];

    for i in delay_sample_length..sample_length {
        let delay_i = i - delay_sample_length;
        new_samples[i] += samples[delay_i];
    }

    new_samples = sum(vec![
        SampleLine {
            gain_factor: *dry_gain_factor,
            samples: samples.to_owned(),
        },
        SampleLine {
            gain_factor: *wet_gain_factor,
            samples: new_samples,
        },
    ]);

    let new_feedback_factor = feedback_factor.powf(1.5); // try and make feedback a bit less non linear
    let new_processor_params = ProcessorParams {
        samples: new_samples,
        channels: *channels,
        endian: *endian,
        file_format: *file_format,
        sub_format: *sub_format,
        sample_rate: *sample_rate,
        sample_length: sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    };
    let new_delay_params = DelayLineParams {
        feedback_factor: new_feedback_factor,
        delay_sample_length: delay_sample_length,
        dry_gain_factor: 1_f64,
        wet_gain_factor: *feedback_factor,
    };

    if *feedback_factor > 0_f64 {
        return delay_line(&new_processor_params, &new_delay_params);
    }

    return Ok(new_processor_params);
}