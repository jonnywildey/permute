use biquad::*;
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, sync::mpsc};

use crate::permute_files::PermuteUpdate;

pub type ProcessorFn = fn(&ProcessorParams) -> ProcessorParams;

#[derive(Debug, Clone)]
pub struct ProcessorParams {
    pub spec: hound::WavSpec,
    pub samples: Vec<f64>,
    pub sample_length: usize,
    pub permutation: Permutation,

    pub update_sender: mpsc::Sender<PermuteUpdate>,
}

#[derive(Debug, Clone)]

pub struct Permutation {
    pub file: String,
    pub permutation_index: usize,
    pub output: String,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processors: Vec<PermuteNodeName>,
    pub original_sample_rate: u32,
    pub node_index: usize,
}

#[derive(Debug, Clone)]
pub enum PermuteNodeEvent {
    NodeProcessStarted,
    NodeProcessComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PermuteNodeName {
    Reverse,
    RhythmicDelay,
    MetallicDelay,
    HalfSpeed,
    DoubleSpeed,
    Wow,
    Flutter,
    Chorus,
    Phaser,
    Normalise,
    SampleRateConversionHigh,
    SampleRateConversionOriginal,
}

pub fn reverse(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: &ProcessorParams,
) -> ProcessorParams {
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessStarted,
    ));
    let mut new_samples = samples.clone();
    let channels = spec.channels as i32;

    for i in 0..*sample_length {
        let channel_offset: i32 = (channels * -1 + 1) + 2 * (i as i32 % channels);
        let sample_i: i32 = *sample_length as i32 - 1 - i as i32 + channel_offset;
        new_samples[i] = samples[sample_i as usize];
    }

    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    return ProcessorParams {
        samples: new_samples,
        spec: *spec,
        sample_length: *sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    };
}

pub struct DelayLineParams {
    pub feedback_factor: f64, // 0 - 1
    pub delay_sample_length: usize,
    pub dry_gain_factor: f64,
    pub wet_gain_factor: f64,
}

pub fn change_sample_rate(params: ProcessorParams, new_sample_rate: u32) -> ProcessorParams {
    if params.spec.sample_rate == new_sample_rate {
        return params;
    }
    let mut new_params = params.clone();
    let speed = params.spec.sample_rate as f64 / new_sample_rate as f64;

    if speed >= 2.0 {
        new_params = multi_channel_filter(
            &new_params,
            &FilterParams {
                filter_type: FilterType::LowPass,
                frequency: (new_sample_rate / 2) as f64,
                form: FilterForm::Form2,
                q: None,
            },
        );
    }

    let resampled = change_speed(new_params, speed);

    resampled
}

pub fn change_sample_rate_high(params: &ProcessorParams) -> ProcessorParams {
    let new_params = params.clone();
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.to_owned();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::SampleRateConversionHigh,
        PermuteNodeEvent::NodeProcessStarted,
    ));

    let new_sample_rate = match params.spec.sample_rate {
        0..=48000 => params.spec.sample_rate * 4,
        48001..=96000 => params.spec.sample_rate * 2,
        _ => params.spec.sample_rate,
    };

    let mut new_params = change_sample_rate(new_params, new_sample_rate);
    new_params.permutation.original_sample_rate = params.spec.sample_rate;
    new_params.spec.sample_rate = new_sample_rate;

    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        new_params.permutation.clone(),
        PermuteNodeName::SampleRateConversionHigh,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_params
}

pub fn change_sample_rate_original(params: &ProcessorParams) -> ProcessorParams {
    let new_params = params.clone();
    let update_sender = params.update_sender.to_owned();

    let permutation = params.permutation.to_owned();
    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::SampleRateConversionOriginal,
        PermuteNodeEvent::NodeProcessStarted,
    ));

    let new_params = change_sample_rate(new_params, permutation.original_sample_rate);

    let _ = update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        new_params.permutation.clone(),
        PermuteNodeName::SampleRateConversionOriginal,
        PermuteNodeEvent::NodeProcessComplete,
    ));
    new_params
}

pub fn delay_line(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: &ProcessorParams,
    DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor,
        wet_gain_factor,
    }: &DelayLineParams,
) -> ProcessorParams {
    // Ensure sample length matches channel count
    let sample_length = *sample_length;
    let delay_sample_length = delay_sample_length - (delay_sample_length % spec.channels as usize);
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
        spec: *spec,
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

    return new_processor_params;
}

pub fn gain(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: ProcessorParams,
    gain_factor: f64,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        new_samples[i] = samples[i] * gain_factor;
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
        update_sender,
        permutation,
    };
}

pub fn ceiling(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: ProcessorParams,
    ceiling: f64,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    let abs_max = samples.iter().fold(0_f64, |a, b| {
        let b = b.abs();
        if b >= a {
            b
        } else {
            a
        }
    });
    let ceiling_factor = ceiling / abs_max;

    for i in 0..sample_length {
        new_samples[i] = samples[i] * ceiling_factor;
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
        update_sender,
        permutation,
    };
}

pub struct SampleLine {
    pub samples: Vec<f64>,
    pub gain_factor: f64,
}

pub fn sum(sample_lines: Vec<SampleLine>) -> Vec<f64> {
    // get max len
    let samples_len = sample_lines.iter().fold(0, |a, b| {
        let b_len = b.samples.len();
        if b_len > a {
            b_len
        } else {
            a
        }
    });

    let mut new_samples = vec![0_f64; samples_len];
    let sample_lines_len = sample_lines.len();

    for i in 0..sample_lines_len {
        for j in 0..sample_lines[i].samples.len() {
            new_samples[j] += sample_lines[i].samples[j] * sample_lines[i].gain_factor;
        }
    }
    new_samples
}

pub fn change_speed(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: ProcessorParams,
    speed: f64,
) -> ProcessorParams {
    let channels = spec.channels as usize;
    let mut new_sample_length: usize = ((sample_length as f64) / speed).ceil() as usize;
    let sample_mod = new_sample_length % channels as usize;
    if sample_mod > 0 {
        new_sample_length = new_sample_length - sample_mod;
    }
    let mut new_samples = vec![0_f64; new_sample_length];

    new_samples[0] = samples[0];
    let mut ptr1: usize;
    let mut ptr2: usize;
    let mut new_ptr: usize;
    for c_offset in 0..channels {
        for i in (0..new_sample_length / channels).step_by(channels) {
            ptr1 = channels * (((i as f64 - 1_f64) * speed).floor() as usize) + c_offset;
            ptr2 = channels * ((i as f64 * speed).floor() as usize) + c_offset;
            new_ptr = (channels * i) + c_offset;

            new_samples[new_ptr] = samples[ptr1] + ((samples[ptr2] - samples[ptr1]) * speed);
        }
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: new_sample_length,
        update_sender,
        permutation,
    };
}

fn split_channels(samples: Vec<f64>, channels: u16) -> Vec<Vec<f64>> {
    let channels: usize = channels as usize;
    let mut by_channels: Vec<Vec<f64>> = vec![];
    for c in 0..channels {
        let mut channel_sample: Vec<f64> = vec![];
        for i in (c..samples.len()).step_by(channels) {
            channel_sample.push(samples[i]);
        }
        by_channels.push(channel_sample);
    }
    by_channels
}

fn interleave_channels(by_channels: Vec<Vec<f64>>) -> Vec<f64> {
    let channel_length = by_channels[0].len();
    let channels = by_channels.len();
    let total_sample_length = channel_length * channels;

    let mut samples: Vec<f64> = vec![0_f64; total_sample_length];

    for c in 0..channels {
        for i in 0..by_channels[c].len() {
            samples[(i * channels) + c] = by_channels[c][i];
        }
    }
    samples
}

pub struct VibratoParams {
    pub speed_hz: f64,
    pub depth: f64, // usable values seem to be around 0 - 0.2
}

pub fn vibrato(
    ProcessorParams {
        samples,
        sample_length: _,
        spec,
        update_sender,
        permutation,
    }: ProcessorParams,
    VibratoParams { speed_hz, depth }: VibratoParams,
) -> ProcessorParams {
    // let adjusted_depth = depth.powf(2_f64) * 512_f64; // ideally 1 should be a somewhat usable value
    let adjusted_depth = depth * spec.sample_rate as f64 * 2_f64.powf(-7.0);
    let channel_samples = split_channels(samples, spec.channels);
    let mut new_channel_samples: Vec<Vec<f64>> = vec![];

    for c in 0..channel_samples.len() {
        let cs = &channel_samples[c];
        let channel_length = cs.len();
        let mut ns = vec![0_f64; channel_length];

        let mut ptr1: usize;
        let mut ptr2: usize;
        let mut speed: f64;
        let mut cos_amplitude: f64;
        for i in 0..channel_length {
            cos_amplitude = (i as f64 / spec.sample_rate as f64 * 2.0 * PI * speed_hz).cos();
            speed = 1_f64 + cos_amplitude;

            let offset_f = (i as f64 - 1_f64) + (speed * adjusted_depth);
            let offset = offset_f.floor() as usize;
            let frac = offset_f - offset as f64;

            ptr1 = offset;
            ptr2 = ptr1 + 1;

            // Can't guarantee sped up samples will go slightly over original length
            if ptr2 >= channel_length {
                break;
            }

            ns[i] = cs[ptr1] + (cs[ptr2] - cs[ptr1]) * frac;
        }
        new_channel_samples.push(ns);
    }

    let interleave_samples = interleave_channels(new_channel_samples);
    let interleave_sample_length = interleave_samples.len();

    return ProcessorParams {
        samples: interleave_samples,
        spec: spec,
        sample_length: interleave_sample_length,
        update_sender,
        permutation,
    };
}

pub struct ChorusParams {
    pub delay_params: DelayLineParams,
    pub vibrato_params: VibratoParams,
}

pub fn chorus(
    params: ProcessorParams,
    ChorusParams {
        delay_params,
        vibrato_params,
    }: ChorusParams,
) -> ProcessorParams {
    let dry_samples = params.samples.clone();
    let update_sender = params.update_sender.to_owned();

    let delayed = delay_line(&params, &delay_params);
    let vibratod = vibrato(delayed, vibrato_params);

    let summed = sum(vec![
        SampleLine {
            samples: dry_samples,
            gain_factor: 1_f64,
        },
        SampleLine {
            samples: vibratod.samples,
            gain_factor: -1_f64,
        },
    ]);

    return ProcessorParams {
        sample_length: summed.len(),
        samples: summed,
        spec: vibratod.spec,
        update_sender,
        permutation: params.permutation,
    };
}

pub type FilterType<T> = Type<T>;

#[derive(Clone)]
pub enum FilterForm {
    Form1,
    Form2,
}

#[derive(Clone)]
pub struct FilterParams {
    pub frequency: f64,
    pub q: Option<f64>,
    pub filter_type: FilterType<f64>,
    pub form: FilterForm,
}

pub fn multi_channel_filter(
    params: &ProcessorParams,
    filter_params: &FilterParams,
) -> ProcessorParams {
    let copied_params = params.clone();
    let channel_samples = split_channels(params.samples.to_owned(), params.spec.channels);

    let split_params = channel_samples
        .iter()
        .map(|cs| {
            filter(
                &ProcessorParams {
                    permutation: copied_params.permutation.clone(),
                    sample_length: cs.len(),
                    samples: cs.to_vec(),
                    spec: copied_params.spec.clone(),
                    update_sender: copied_params.update_sender.to_owned(),
                },
                &filter_params.clone(),
            )
        })
        .collect::<Vec<ProcessorParams>>();

    let split_samples = split_params
        .iter()
        .map(|ss| ss.samples.to_vec())
        .collect::<Vec<Vec<f64>>>();

    let interleaved_samples = interleave_channels(split_samples);
    ProcessorParams {
        permutation: copied_params.permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        spec: copied_params.spec,
        update_sender: copied_params.update_sender,
    }
}

pub fn filter(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_sender,
        permutation,
    }: &ProcessorParams,
    FilterParams {
        filter_type,
        frequency,
        q,
        form,
    }: &FilterParams,
) -> ProcessorParams {
    // Cutoff and sampling frequencies
    let f0 = frequency.hz();
    let fs = spec.sample_rate.hz();
    let q = q.unwrap_or(Q_BUTTERWORTH_F64);

    // Create coefficients for the biquads
    let coeffs = Coefficients::<f64>::from_params(*filter_type, fs, f0, q).unwrap();

    let mut new_samples = vec![0_f64; *sample_length];
    match form {
        &FilterForm::Form1 => {
            let mut biquad1 = DirectForm1::<f64>::new(coeffs);

            for i in 0..*sample_length {
                new_samples[i] = biquad1.run(samples[i]);
            }
        }
        &FilterForm::Form2 => {
            let mut biquad2 = DirectForm2Transposed::<f64>::new(coeffs);

            for i in 0..*sample_length {
                new_samples[i] = biquad2.run(samples[i]);
            }
        }
    }

    return ProcessorParams {
        samples: new_samples,
        spec: *spec,
        sample_length: *sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    };
}

pub struct PhaserParams {
    pub stages: i32,
    pub base_freq: f64,
    pub lfo_depth: f64, // range lfo affects frequency. range 0-1
    pub stage_hz: f64,  // increases base freq by this amount
    pub lfo_rate: f64,
    pub q: f64,
    pub dry_mix: f64,
    pub wet_mix: f64,
}

pub fn phaser(
    params: &ProcessorParams,
    PhaserParams {
        base_freq,
        dry_mix,
        lfo_depth,
        lfo_rate,
        q,
        stage_hz,
        stages,
        wet_mix,
    }: &PhaserParams,
) -> ProcessorParams {
    let channel_samples = split_channels(params.samples.to_owned(), params.spec.channels);

    let split_params = channel_samples
        .iter()
        .map(|cs| {
            let filters: Vec<(f64, DirectForm1<f64>)> = (0..*stages)
                .map(|i| {
                    let base_freq = base_freq + (i as f64 * stage_hz);
                    let coeffs = Coefficients::<f64>::from_params(
                        FilterType::AllPass,
                        (params.spec.sample_rate).hz(),
                        base_freq.hz(),
                        *q,
                    )
                    .unwrap();
                    let filter = DirectForm1::<f64>::new(coeffs);
                    return (base_freq, filter);
                })
                .collect();

            let mut new_samples = cs.clone();
            let mut lfo_amplitude: f64;
            let sample_rate = params.spec.sample_rate;
            for (base_freq, mut filter) in filters.iter() {
                for i in 0..cs.len() {
                    lfo_amplitude = lfo_tri(i, sample_rate, *lfo_rate);
                    let offset = base_freq * lfo_depth * lfo_amplitude;
                    let mut freq = base_freq + offset;
                    if freq <= 0.0 {
                        freq = 0.0001;
                    }
                    let new_coeffs = Coefficients::<f64>::from_params(
                        FilterType::AllPass,
                        params.spec.sample_rate.hz(),
                        freq.hz(),
                        *q,
                    )
                    .unwrap();
                    filter.update_coefficients(new_coeffs);
                    new_samples[i] = filter.run(new_samples[i]);
                }
            }

            // let coeffs = Coefficients::<f64>::from_params(
            //     FilterType::AllPass,
            //     (params.spec.sample_rate).hz(),
            //     (phaser_params.base_freq).hz(),
            //     phaser_params.q,
            // )
            // .unwrap();

            // let mut filter = DirectForm1::<f64>::new(coeffs);
            // let mut new_samples = cs.clone();
            // let mut cos_amplitude: f64;

            // for i in 0..cs.len() {
            //     cos_amplitude =
            //         (i as f64 / params.spec.sample_rate as f64 * 2.0 * PI * phaser_params.lfo_rate)
            //             .cos();
            //     let mut freq = phaser_params.base_freq
            //         + (phaser_params.base_freq * phaser_params.lfo_depth * cos_amplitude as f64);
            //     if freq <= 0.0 {
            //         freq = 0.0001;
            //     }

            //     let new_coeffs = Coefficients::<f64>::from_params(
            //         FilterType::AllPass,
            //         params.spec.sample_rate.hz(),
            //         freq.hz(),
            //         phaser_params.q,
            //     )
            //     .unwrap();
            //     filter.update_coefficients(new_coeffs);
            //     new_samples[i] = filter.run(cs[i]);
            // }
            // for i in 0..cs.len() {
            //     cos_amplitude =
            //         (i as f64 / params.spec.sample_rate as f64 * 2.0 * PI * phaser_params.lfo_rate)
            //             .cos();
            //     let mut freq = phaser_params.base_freq
            //         + phaser_params.stage_hz
            //         + (phaser_params.base_freq * phaser_params.lfo_depth * cos_amplitude as f64);
            //     if freq <= 0.0 {
            //         freq = 0.0001;
            //     }

            //     let new_coeffs = Coefficients::<f64>::from_params(
            //         FilterType::AllPass,
            //         params.spec.sample_rate.hz(),
            //         freq.hz(),
            //         phaser_params.q,
            //     )
            //     .unwrap();
            //     filter.update_coefficients(new_coeffs);
            //     new_samples[i] = filter.run(cs[i]);
            // }

            new_samples
        })
        .collect::<Vec<Vec<f64>>>();

    let interleaved_samples = interleave_channels(split_params);

    let summed = sum(vec![
        SampleLine {
            samples: params.samples.to_owned(),
            gain_factor: *dry_mix,
        },
        SampleLine {
            samples: interleaved_samples,
            gain_factor: *wet_mix,
        },
    ]);

    return ProcessorParams {
        sample_length: summed.len(),
        samples: summed,
        spec: params.spec,
        update_sender: params.update_sender.to_owned(),
        permutation: params.permutation.to_owned(),
    };
}

pub fn lfo_sin(sample: usize, sample_rate: u32, lfo_rate: f64) -> f64 {
    (sample as f64 / sample_rate as f64 * 2.0 * PI * lfo_rate).sin()
}

pub fn lfo_cos(sample: usize, sample_rate: u32, lfo_rate: f64) -> f64 {
    (sample as f64 / sample_rate as f64 * 2.0 * PI * lfo_rate).cos()
}

pub fn lfo_tri(sample: usize, sample_rate: u32, lfo_rate: f64) -> f64 {
    let cycle: u32 = sample_rate / lfo_rate as u32; // rounding error here. no guarantee lfo_rate matches
    let pos: f64 = (sample as u32 % cycle) as f64;
    let half_cycle: f64 = (cycle as f64) / 2.0;
    if pos <= half_cycle {
        return pos / half_cycle;
    } else {
        return 1.0 - ((pos - half_cycle) / half_cycle);
    }
}
