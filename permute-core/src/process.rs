use biquad::*;
use serde::{Deserialize, Serialize};
use sndfile::{Endian, MajorFormat, SubtypeFormat};
use std::{
    f64::consts::{E, PI},
    sync::mpsc,
};
use strum::EnumIter;

use crate::osc::*;
use crate::{permute_error::PermuteError, permute_files::PermuteUpdate};

pub type ProcessorFn = fn(&ProcessorParams) -> Result<ProcessorParams, PermuteError>;

#[derive(Debug, Clone)]
pub struct ProcessorParams {
    pub samples: Vec<f64>,
    pub sample_length: usize,
    pub permutation: Permutation,

    pub channels: usize,
    pub sample_rate: usize,
    pub sub_format: SubtypeFormat,
    pub file_format: MajorFormat,
    pub endian: Endian,

    pub update_sender: mpsc::Sender<PermuteUpdate>,
}

#[derive(Debug, Clone)]
pub struct Permutation {
    pub file: String,
    pub permutation_index: usize,
    pub output: String,
    pub processor_pool: Vec<PermuteNodeName>,
    pub processors: Vec<PermuteNodeName>,
    pub original_sample_rate: usize,
    pub node_index: usize,
}

#[derive(Debug, Clone)]
pub enum PermuteNodeEvent {
    NodeProcessStarted,
    NodeProcessComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum PermuteNodeName {
    Reverse,
    RhythmicDelay,
    MetallicDelay,

    GranularTimeStretch,
    Reverb,

    Fuzz,
    Saturate,

    HalfSpeed,
    DoubleSpeed,
    RandomPitch,

    Filter,
    LineFilter,
    Tremolo,
    Lazer,
    OscillatingFilter,

    Wow,
    Flutter,
    Flange,
    Chorus,
    Phaser,
    Normalise,
    Trim,
    SampleRateConversionHigh,
    SampleRateConversionOriginal,
}

// Only processors we want to be visible to users
#[allow(dead_code)]
pub const ALL_PROCESSORS: [PermuteNodeName; 20] = [
    PermuteNodeName::Reverse,
    PermuteNodeName::GranularTimeStretch,
    PermuteNodeName::Reverb,
    PermuteNodeName::Saturate,
    PermuteNodeName::Fuzz,
    PermuteNodeName::MetallicDelay,
    PermuteNodeName::RhythmicDelay,
    PermuteNodeName::HalfSpeed,
    PermuteNodeName::DoubleSpeed,
    PermuteNodeName::RandomPitch,
    PermuteNodeName::Wow,
    PermuteNodeName::Flutter,
    PermuteNodeName::Chorus,
    PermuteNodeName::Flange,
    PermuteNodeName::Phaser,
    PermuteNodeName::Filter,
    PermuteNodeName::LineFilter,
    PermuteNodeName::Tremolo,
    PermuteNodeName::Lazer,
    PermuteNodeName::OscillatingFilter,
];

pub fn reverse(
    ProcessorParams {
        samples,
        sample_length,
        update_sender,
        permutation,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
    }: &ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessStarted,
    ))?;
    let mut new_samples = samples.clone();
    let channels = *channels as i32;

    for i in 0..*sample_length {
        let channel_offset: i32 = (channels * -1 + 1) + 2 * (i as i32 % channels);
        let sample_i: i32 = *sample_length as i32 - 1 - i as i32 + channel_offset;
        new_samples[i] = samples[sample_i as usize];
    }

    update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessComplete,
    ))?;
    return Ok(ProcessorParams {
        samples: new_samples,
        sample_length: *sample_length,
        channels: channels as usize,
        endian: *endian,
        file_format: *file_format,
        sub_format: *sub_format,
        sample_rate: *sample_rate,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    });
}

pub struct DelayLineParams {
    pub feedback_factor: f64, // 0 - 1
    pub delay_sample_length: usize,
    pub dry_gain_factor: f64,
    pub wet_gain_factor: f64,
}

pub fn change_sample_rate(
    params: ProcessorParams,
    new_sample_rate: usize,
) -> Result<ProcessorParams, PermuteError> {
    if params.sample_rate == new_sample_rate {
        return Ok(params);
    }
    let mut new_params = params.clone();
    let speed = params.sample_rate as f64 / new_sample_rate as f64;

    if speed >= 2.0 {
        new_params = multi_channel_filter(
            &new_params,
            &FilterParams {
                filter_type: FilterType::LowPass,
                frequency: (new_sample_rate / 2) as f64,
                form: FilterForm::Form2,
                q: None,
            },
        )?;
    }

    let resampled = change_speed(new_params, speed);

    Ok(resampled)
}

pub fn change_sample_rate_high(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    let new_params = params.clone();
    let update_sender = params.update_sender.to_owned();
    let permutation = params.permutation.to_owned();
    update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::SampleRateConversionHigh,
        PermuteNodeEvent::NodeProcessStarted,
    ))?;

    let new_sample_rate = match params.sample_rate {
        0..=48000 => params.sample_rate * 4,
        48001..=96000 => params.sample_rate * 2,
        _ => params.sample_rate,
    };

    let mut new_params = change_sample_rate(new_params, new_sample_rate)?;
    new_params.permutation.original_sample_rate = params.sample_rate;
    new_params.sample_rate = new_sample_rate;

    update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        new_params.permutation.clone(),
        PermuteNodeName::SampleRateConversionHigh,
        PermuteNodeEvent::NodeProcessComplete,
    ))?;
    Ok(new_params)
}

pub fn change_sample_rate_original(
    params: &ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    let new_params = params.clone();
    let update_sender = params.update_sender.to_owned();

    let permutation = params.permutation.to_owned();
    update_sender.send(PermuteUpdate::UpdatePermuteNodeStarted(
        permutation.clone(),
        PermuteNodeName::SampleRateConversionOriginal,
        PermuteNodeEvent::NodeProcessStarted,
    ))?;

    let new_params = change_sample_rate(new_params, permutation.original_sample_rate)?;

    update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        new_params.permutation.clone(),
        PermuteNodeName::SampleRateConversionOriginal,
        PermuteNodeEvent::NodeProcessComplete,
    ))?;
    Ok(new_params)
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

// pub fn gain(
//     ProcessorParams {
//         samples,
//         sample_length,
//         channels,
//         endian,
//         file_format,
//         sample_rate,
//         update_sender,
//         permutation,
//     }: ProcessorParams,
//     gain_factor: f64,
// ) -> ProcessorParams {
//     let mut new_samples = samples.clone();

//     for i in 0..sample_length {
//         new_samples[i] = samples[i] * gain_factor;
//     }

//     return ProcessorParams {
//         samples: new_samples,
//         channels,
//         endian,
//         file_format,
//         sample_rate,
//         sample_length: sample_length,
//         update_sender,
//         permutation,
//     };
// }

pub fn ceiling(
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
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        sample_length: sample_length,
        update_sender,
        permutation,
    };
}

#[derive(Debug, Clone)]
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
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        update_sender,
        permutation,
        ..
    }: ProcessorParams,
    speed: f64,
) -> ProcessorParams {
    let channel_samples = split_channels(samples, channels);
    let mut new_channel_samples: Vec<Result<Vec<f64>, PermuteError>> = vec![];

    for c in 0..channel_samples.len() {
        let cs = &channel_samples[c];
        let new_sample_length: usize = ((cs.len() as f64) / speed).ceil() as usize;
        let mut ns: Vec<f64> = vec![0_f64; new_sample_length];

        let mut v1: f64;
        let mut v2: f64;
        let len = new_sample_length - 1;
        for i in 0..len {
            let offset_f = (i as f64 - 1_f64) * speed;
            let offset = offset_f.floor() as usize;
            let frac = offset_f - offset as f64;

            v1 = cs[offset];
            v2 = cs[offset + 1];

            ns[i] = v1 + (v2 - v1) * frac;
        }
        new_channel_samples.push(Ok(ns));
    }

    let new_channel_samples = new_channel_samples.into_iter().collect();

    let interleave_samples = interleave_channels(new_channel_samples).unwrap();
    let interleave_sample_length = interleave_samples.len();

    return ProcessorParams {
        samples: interleave_samples,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        sample_length: interleave_sample_length,
        update_sender,
        permutation,
    };
}

fn split_channels(samples: Vec<f64>, channels: usize) -> Vec<Vec<f64>> {
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

fn interleave_channels(
    by_channels: Result<Vec<Vec<f64>>, PermuteError>,
) -> Result<Vec<f64>, PermuteError> {
    let by_channels = by_channels?;
    let channel_length = by_channels[0].len();
    let channels = by_channels.len();
    let total_sample_length = channel_length * channels;

    let mut samples: Vec<f64> = vec![0_f64; total_sample_length];

    for c in 0..channels {
        let len = by_channels[c].len();
        for i in 0..len {
            samples[(i * channels) + c] = by_channels[c][i];
        }
    }
    Ok(samples)
}

pub struct VibratoParams {
    pub speed_hz: f64,
    pub depth: f64, // usable values seem to be around 0 - 0.2
}

pub fn vibrato(
    ProcessorParams {
        samples,
        sample_length: _,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        update_sender,
        permutation,
    }: ProcessorParams,
    VibratoParams { speed_hz, depth }: VibratoParams,
) -> Result<ProcessorParams, PermuteError> {
    // let adjusted_depth = depth.powf(2_f64) * 512_f64; // ideally 1 should be a somewhat usable value
    let adjusted_depth = depth * sample_rate as f64 * 2_f64.powf(-7.0);
    let channel_samples = split_channels(samples, channels);
    let mut new_channel_samples: Vec<Result<Vec<f64>, PermuteError>> = vec![];

    for c in 0..channel_samples.len() {
        let cs = &channel_samples[c];
        let channel_length = cs.len();
        let mut ns = vec![0_f64; channel_length];

        let mut ptr1: usize;
        let mut ptr2: usize;
        let mut speed: f64;
        let mut cos_amplitude: f64;
        for i in 0..channel_length {
            cos_amplitude = (i as f64 / sample_rate as f64 * 2.0 * PI * speed_hz).cos();
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
        new_channel_samples.push(Ok(ns));
    }
    let new_channel_samples = new_channel_samples.into_iter().collect();

    let interleave_samples = interleave_channels(new_channel_samples)?;
    let interleave_sample_length = interleave_samples.len();

    return Ok(ProcessorParams {
        samples: interleave_samples,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        sample_length: interleave_sample_length,
        update_sender,
        permutation,
    });
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
) -> Result<ProcessorParams, PermuteError> {
    let dry_samples = params.samples.clone();
    let update_sender = params.update_sender.to_owned();

    let delayed = delay_line(&params, &delay_params)?;
    let vibratod = vibrato(delayed, vibrato_params)?;

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

    return Ok(ProcessorParams {
        sample_length: summed.len(),
        samples: summed,
        update_sender,
        channels: vibratod.channels,
        endian: vibratod.endian,
        file_format: vibratod.file_format,
        sub_format: vibratod.sub_format,
        sample_rate: vibratod.sample_rate,
        permutation: params.permutation,
    });
}

#[derive(Clone)]
pub struct TremoloParams {
    pub speed_hz: f64,
    pub depth: f64,
}

pub fn tremolo(
    ProcessorParams {
        samples,
        sample_length: _,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        update_sender,
        permutation,
    }: ProcessorParams,
    TremoloParams { speed_hz, depth }: TremoloParams,
) -> Result<ProcessorParams, PermuteError> {
    let channel_samples = split_channels(samples, channels);
    let mut new_channel_samples: Vec<Result<Vec<f64>, PermuteError>> = vec![];

    for c in 0..channel_samples.len() {
        let cs = &channel_samples[c];
        let mut ns: Vec<f64> = vec![0_f64; cs.len()];
        for i in 0..cs.len() {
            let amplitude = lfo_sin(i, sample_rate, speed_hz, 0.0);
            ns[i] = cs[i] - (cs[i] * amplitude * depth)
        }
        new_channel_samples.push(Ok(ns));
    }
    let interleaved_samples = interleave_channels(new_channel_samples.into_iter().collect())?;

    Ok(ProcessorParams {
        permutation: permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        channels: channels,
        endian: endian,
        file_format: file_format,
        sub_format: sub_format,
        sample_rate: sample_rate,
        update_sender: update_sender,
    })
}

#[derive(Clone)]
pub struct TremoloInputModParams {
    pub min_speed_hz: f64,
    pub max_speed_hz: f64,
    pub depth: f64,
    pub frame_ms: usize,
}

// Use amplitude of the signal to modulate speed of tremolo
pub fn tremolo_input_mod(
    ProcessorParams {
        samples,
        sample_length: _,
        channels,
        endian,
        file_format,
        sub_format,
        sample_rate,
        update_sender,
        permutation,
    }: ProcessorParams,
    TremoloInputModParams {
        min_speed_hz,
        max_speed_hz,
        depth,
        frame_ms,
    }: TremoloInputModParams,
) -> Result<ProcessorParams, PermuteError> {
    let channel_samples = split_channels(samples, channels);
    let mut new_channel_samples: Vec<Result<Vec<f64>, PermuteError>> = vec![];
    let frame_count = ((sample_rate as f64) * (frame_ms as f64) * 0.001) as usize;
    let frame_count_f = frame_count as f64;
    let speed_diff = max_speed_hz - min_speed_hz;
    let ramp_count = (frame_count);
    let ramp_count_64 = ramp_count as f64;

    for c in 0..channel_samples.len() {
        let cs = &channel_samples[c];
        let mut ns: Vec<f64> = vec![0_f64; cs.len()];

        let mut sumframes = vec![0_f64; frame_count];
        let mut val: f64;
        let mut speed: f64 = min_speed_hz;
        let mut old_speed: f64 = speed;
        let mut new_speed: f64 = speed;
        let mut ramp = ramp_count;
        let mut osc = new_oscillator(sample_rate as f64);
        osc.set_frequency(speed);

        for i in 0..cs.len() {
            sumframes[i % frame_count] = cs[i].abs();
            if i % frame_count == 0 {
                old_speed = new_speed;
                // average
                val = sumframes.iter().fold(0.0, |acc, v| acc + v) / frame_count_f;
                new_speed = min_speed_hz + (val * speed_diff);
                ramp = 0;
            }
            if ramp == ramp_count {
                speed = new_speed;
            } else {
                let diff = new_speed - old_speed;
                speed = old_speed + (diff * (ramp as f64 / ramp_count_64));
                ramp += 1;
            }

            osc.set_frequency(speed);
            let amp = osc.process();
            ns[i] = cs[i] - (cs[i] * amp * depth);
        }
        new_channel_samples.push(Ok(ns));
    }
    let interleaved_samples = interleave_channels(new_channel_samples.into_iter().collect())?;

    Ok(ProcessorParams {
        permutation: permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        channels: channels,
        endian: endian,
        file_format: file_format,
        sub_format: sub_format,
        sample_rate: sample_rate,
        update_sender: update_sender,
    })
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
) -> Result<ProcessorParams, PermuteError> {
    let copied_params = params.clone();
    let channel_samples = split_channels(params.samples.to_owned(), params.channels);

    let split_samples = channel_samples
        .iter()
        .map(|cs| {
            Ok(filter(
                &ProcessorParams {
                    permutation: copied_params.permutation.clone(),
                    sample_length: cs.len(),
                    samples: cs.to_vec(),
                    channels: params.channels,
                    endian: params.endian,
                    file_format: params.file_format,
                    sub_format: params.sub_format,
                    sample_rate: params.sample_rate,
                    update_sender: copied_params.update_sender.to_owned(),
                },
                &filter_params.clone(),
            )?
            .samples)
        })
        .collect::<Vec<Result<Vec<f64>, PermuteError>>>();

    let interleaved_samples = interleave_channels(split_samples.into_iter().collect())?;
    Ok(ProcessorParams {
        permutation: copied_params.permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        channels: copied_params.channels,
        endian: copied_params.endian,
        file_format: copied_params.file_format,
        sub_format: copied_params.sub_format,
        sample_rate: copied_params.sample_rate,
        update_sender: copied_params.update_sender,
    })
}

pub fn filter(
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
    FilterParams {
        filter_type,
        frequency,
        q,
        form,
    }: &FilterParams,
) -> Result<ProcessorParams, PermuteError> {
    // Cutoff and sampling frequencies
    let f0 = frequency.hz();
    let fs = (*sample_rate as u32).hz();
    let q = q.unwrap_or(Q_BUTTERWORTH_F64);

    // Create coefficients for the biquads
    let coeffs = Coefficients::<f64>::from_params(*filter_type, fs, f0, q)?;

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

    return Ok(ProcessorParams {
        samples: new_samples,
        channels: *channels,
        endian: *endian,
        file_format: *file_format,
        sub_format: *sub_format,
        sample_rate: *sample_rate,
        sample_length: *sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    });
}

pub fn oscillating_filter(
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
    OscillatingFilterParams {
        filter_type,
        frequency,
        q,
        form,
        lfo_rate,
        lfo_factor,
    }: &OscillatingFilterParams,
) -> Result<ProcessorParams, PermuteError> {
    // Cutoff and sampling frequencies
    let f0 = frequency.hz();
    let fs = (*sample_rate as u32).hz();
    let q = q.unwrap_or(Q_BUTTERWORTH_F64);

    // Create coefficients for the biquads
    let coeffs = Coefficients::<f64>::from_params(*filter_type, fs, f0, q)?;

    let mut new_samples = vec![0_f64; *sample_length];
    match form {
        &FilterForm::Form1 => {
            let mut biquad1 = DirectForm1::<f64>::new(coeffs);

            for i in 0..*sample_length {
                let lfo_gain = lfo_tri(i, *sample_rate, *lfo_rate);
                let mut new_frequency = frequency + (frequency * lfo_gain * lfo_factor);
                if new_frequency <= 0.0 {
                    new_frequency = 0.01
                }
                let new_coeffs =
                    Coefficients::<f64>::from_params(*filter_type, fs, new_frequency.hz(), q)?;
                biquad1.update_coefficients(new_coeffs);
                new_samples[i] = biquad1.run(samples[i]);
            }
        }
        &FilterForm::Form2 => {
            let mut biquad2 = DirectForm2Transposed::<f64>::new(coeffs);

            for i in 0..*sample_length {
                let lfo_gain = lfo_tri(i, *sample_rate, *lfo_rate);
                let mut new_frequency = frequency + (frequency * lfo_gain * lfo_factor);
                if new_frequency <= 0.0 {
                    new_frequency = 0.01
                }
                let new_coeffs =
                    Coefficients::<f64>::from_params(*filter_type, fs, new_frequency.hz(), q)?;
                biquad2.update_coefficients(new_coeffs);
                new_samples[i] = biquad2.run(samples[i]);
            }
        }
    }

    return Ok(ProcessorParams {
        samples: new_samples,
        channels: *channels,
        endian: *endian,
        file_format: *file_format,
        sub_format: *sub_format,
        sample_rate: *sample_rate,
        sample_length: *sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    });
}

#[derive(Clone)]
pub struct LineFilterParams {
    pub q: Option<f64>,
    pub filter_type: FilterType<f64>,
    pub form: FilterForm,
    pub hz_from: f64,
    pub hz_to: f64,
}

pub fn line_filter(
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
    LineFilterParams {
        filter_type,
        q,
        form,
        hz_from,
        hz_to,
    }: &LineFilterParams,
) -> Result<ProcessorParams, PermuteError> {
    // Cutoff and sampling frequencies
    let f0 = hz_from.hz();
    let fs = (*sample_rate as u32).hz();
    let q = q.unwrap_or(Q_BUTTERWORTH_F64);

    // Create coefficients for the biquads
    let coeffs = Coefficients::<f64>::from_params(*filter_type, fs, f0, q)?;

    let mut new_samples = vec![0_f64; *sample_length];
    match form {
        &FilterForm::Form1 => {
            let mut biquad1 = DirectForm1::<f64>::new(coeffs);

            for i in 0..*sample_length {
                let progress = i as f64 / *sample_length as f64;
                let new_frequency = hz_from + ((hz_to - hz_from) * progress);
                let new_coeffs =
                    Coefficients::<f64>::from_params(*filter_type, fs, new_frequency.hz(), q)?;
                biquad1.update_coefficients(new_coeffs);
                new_samples[i] = biquad1.run(samples[i]);
            }
        }
        &FilterForm::Form2 => {
            let mut biquad2 = DirectForm2Transposed::<f64>::new(coeffs);

            for i in 0..*sample_length {
                let progress = i as f64 / *sample_length as f64;
                let new_frequency = hz_from + ((hz_to - hz_from) * progress);
                let new_coeffs =
                    Coefficients::<f64>::from_params(*filter_type, fs, new_frequency.hz(), q)?;
                biquad2.update_coefficients(new_coeffs);
                new_samples[i] = biquad2.run(samples[i]);
            }
        }
    }

    return Ok(ProcessorParams {
        samples: new_samples,
        channels: *channels,
        endian: *endian,
        file_format: *file_format,
        sub_format: *sub_format,
        sample_rate: *sample_rate,
        sample_length: *sample_length,
        update_sender: update_sender.to_owned(),
        permutation: permutation.to_owned(),
    });
}

#[derive(Clone)]
pub struct OscillatingFilterParams {
    pub frequency: f64,
    pub q: Option<f64>,
    pub filter_type: FilterType<f64>,
    pub form: FilterForm,
    pub lfo_rate: f64,
    pub lfo_factor: f64,
}

pub fn multi_oscillating_filter(
    params: &ProcessorParams,
    filter_params: &OscillatingFilterParams,
) -> Result<ProcessorParams, PermuteError> {
    let copied_params = params.clone();
    let channel_samples = split_channels(params.samples.to_owned(), params.channels);

    let split_samples = channel_samples
        .iter()
        .map(|cs| {
            Ok(oscillating_filter(
                &ProcessorParams {
                    permutation: copied_params.permutation.clone(),
                    sample_length: cs.len(),
                    samples: cs.to_vec(),
                    channels: params.channels,
                    endian: params.endian,
                    file_format: params.file_format,
                    sub_format: params.sub_format,
                    sample_rate: params.sample_rate,
                    update_sender: copied_params.update_sender.to_owned(),
                },
                &filter_params.clone(),
            )?
            .samples)
        })
        .collect::<Vec<Result<Vec<f64>, PermuteError>>>();

    let interleaved_samples = interleave_channels(split_samples.into_iter().collect())?;
    Ok(ProcessorParams {
        permutation: copied_params.permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        channels: copied_params.channels,
        endian: copied_params.endian,
        file_format: copied_params.file_format,
        sub_format: copied_params.sub_format,
        sample_rate: copied_params.sample_rate,
        update_sender: copied_params.update_sender,
    })
}

pub fn multi_line_filter(
    params: &ProcessorParams,
    filter_params: &LineFilterParams,
) -> Result<ProcessorParams, PermuteError> {
    let copied_params = params.clone();
    let channel_samples = split_channels(params.samples.to_owned(), params.channels);

    let split_samples = channel_samples
        .iter()
        .map(|cs| {
            Ok(line_filter(
                &ProcessorParams {
                    permutation: copied_params.permutation.clone(),
                    sample_length: cs.len(),
                    samples: cs.to_vec(),
                    channels: params.channels,
                    endian: params.endian,
                    file_format: params.file_format,
                    sub_format: params.sub_format,
                    sample_rate: params.sample_rate,
                    update_sender: copied_params.update_sender.to_owned(),
                },
                &filter_params.clone(),
            )?
            .samples)
        })
        .collect::<Vec<Result<Vec<f64>, PermuteError>>>();

    let interleaved_samples = interleave_channels(split_samples.into_iter().collect())?;
    Ok(ProcessorParams {
        permutation: copied_params.permutation,
        sample_length: interleaved_samples.len(),
        samples: interleaved_samples,
        channels: copied_params.channels,
        endian: copied_params.endian,
        file_format: copied_params.file_format,
        sub_format: copied_params.sub_format,
        sample_rate: copied_params.sample_rate,
        update_sender: copied_params.update_sender,
    })
}

#[derive(Clone, EnumIter)]
pub enum PhaserStages {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Six = 6,
    Eight = 8,
    Twelve = 12,
    Sixteen = 16,
    Twenty = 20,
}

pub struct PhaserParams {
    pub stages: PhaserStages,
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
    phaser_params: &PhaserParams,
) -> Result<ProcessorParams, PermuteError> {
    let channel_samples = split_channels(params.samples.to_owned(), params.channels);

    let split_params = channel_samples
        .iter()
        .map(|cs| phase_stage(params, phaser_params, cs))
        .collect::<Vec<Result<Vec<f64>, PermuteError>>>()
        .into_iter()
        .collect();

    let interleaved_samples = interleave_channels(split_params)?;

    let summed = sum(vec![
        SampleLine {
            samples: params.samples.to_owned(),
            gain_factor: phaser_params.dry_mix,
        },
        SampleLine {
            samples: interleaved_samples,
            gain_factor: phaser_params.wet_mix,
        },
    ]);

    return Ok(ProcessorParams {
        sample_length: summed.len(),
        samples: summed,
        channels: params.channels,
        endian: params.endian,
        file_format: params.file_format,
        sub_format: params.sub_format,
        sample_rate: params.sample_rate,
        update_sender: params.update_sender.to_owned(),
        permutation: params.permutation.to_owned(),
    });
}

fn phase_stage(
    params: &ProcessorParams,
    phaser_params: &PhaserParams,
    samples: &Vec<f64>,
) -> Result<Vec<f64>, PermuteError> {
    let stages = phaser_params.stages.clone();
    let stage_hz = phaser_params.stage_hz;
    let base_freq = phaser_params.base_freq;
    let q = phaser_params.q;
    let lfo_rate = phaser_params.lfo_rate;
    let lfo_depth = phaser_params.lfo_depth;
    let filters: Result<Vec<(f64, DirectForm1<f64>)>, PermuteError> = (0..stages as i32)
        .map(|i| {
            let base_freq = base_freq + (i as f64 * stage_hz);
            let coeffs = Coefficients::<f64>::from_params(
                FilterType::AllPass,
                (params.sample_rate as u32).hz(),
                base_freq.hz(),
                q,
            )?;
            let filter = DirectForm1::<f64>::new(coeffs);
            return Ok((base_freq, filter));
        })
        .collect::<Vec<Result<(f64, DirectForm1<f64>), PermuteError>>>()
        .into_iter()
        .collect();
    let mut new_samples = samples.clone();
    let mut lfo_amplitude: f64;
    let sample_rate = params.sample_rate;
    for (base_freq, mut filter) in filters?.iter() {
        for i in 0..samples.len() {
            lfo_amplitude = lfo_tri(i, sample_rate, lfo_rate);
            let offset = base_freq * lfo_depth * lfo_amplitude;
            let mut freq = base_freq + offset;
            if freq <= 0.0 {
                freq = 0.0001;
            }
            let new_coeffs = Coefficients::<f64>::from_params(
                FilterType::AllPass,
                (params.sample_rate as u32).hz(),
                freq.hz(),
                q,
            )?;

            filter.update_coefficients(new_coeffs);
            new_samples[i] = filter.run(new_samples[i]);
        }
    }

    Ok(new_samples)
}

pub fn distort(params: &ProcessorParams, factor: f64) -> Result<ProcessorParams, PermuteError> {
    let new_samples = params
        .samples
        .iter()
        .map(|f| {
            let s = f.signum();
            f.abs().powf(factor) * s
        })
        .collect();
    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}

pub fn saturate(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    let new_samples = params
        .samples
        .iter()
        .map(|f| {
            let s = f.signum();
            s * (1.0 - E.powf(-1.0 * f.abs()))
        })
        .collect();
    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}

pub fn trim(params: &ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    let threshold = 0.001;

    let mut start: usize = 0;
    let mut end: usize = params.sample_length;

    for i in 0..params.sample_length {
        if params.samples[i].abs() > threshold {
            start = i - (i % params.channels);
            break;
        }
    }

    for i in (start..params.sample_length).rev() {
        if params.samples[i].abs() > threshold {
            end = i - (i % params.channels);
            break;
        }
    }

    let new_samples = params.samples[start..end].to_vec();
    let len = new_samples.len();

    Ok(ProcessorParams {
        samples: new_samples,
        sample_length: len,
        ..params.clone()
    })
}

pub struct TimeStretchParams {
    pub grain_samples: usize,
    pub blend_samples: usize, // exclusive in grain
    pub stretch_factor: usize,
}

pub fn time_stretch_cross(
    params: &ProcessorParams,
    TimeStretchParams {
        grain_samples,
        blend_samples,
        stretch_factor,
    }: TimeStretchParams,
) -> Result<ProcessorParams, PermuteError> {
    let mut new_samples: Vec<f64> = vec![];

    let blend_samples = match blend_samples {
        d if d > grain_samples => grain_samples,
        _ => blend_samples,
    };

    let count = 1;
    let mut counter = 0;

    let mut chunks: Vec<usize> = vec![0];
    for i in (params.channels..params.sample_length).step_by(params.channels) {
        let a = params.samples[i - params.channels];
        let b = params.samples[i];
        if (a > 0.0 && b < 0.0) || (a < 0.0 && b > 0.0) {
            if i > chunks.last().unwrap() + grain_samples {
                counter += 1;
            }
        }
        if counter == count {
            chunks.push(i);
            counter = 0;
        }
    }
    chunks.push(params.sample_length - 1);

    let half_blend = blend_samples / 2;

    let chunk_tuples: Vec<(usize, usize)> = chunks
        .windows(2)
        .enumerate()
        .map(|(i, d)| {
            let a = d[0];
            let b = d[1];
            return (a, b);
            // if i == 0 {
            //     return (a, b);
            // } else if i + 2 == chunks.len() {
            //     return (a, b);
            // } else {
            //     return (a, b);
            // }
        })
        .collect();

    for (i, (start, end)) in chunk_tuples.iter().enumerate() {
        for s in 0..stretch_factor {
            for j in *start..*end {
                let pos = j - start;
                if pos < half_blend {
                    if i == 0 && s == 0 {
                        new_samples.push(params.samples[j]);
                    } else {
                        let len = new_samples.len() - 1;
                        let f = (pos as f64 / (half_blend) as f64);

                        new_samples.push(params.samples[j] * f);
                        // new_samples[len - pos] = params.samples[j];
                        // new_samples[len - half_blend - pos] =
                        //     (params.samples[j] * f) + new_samples[len - half_blend - pos];
                    }
                } else if end - j < half_blend {
                    let f1 = ((end - j) as f64 / (half_blend) as f64);
                    new_samples.push(params.samples[j] * f1);
                } else {
                    new_samples.push(params.samples[j]);
                }
            }
        }
    }

    Ok(ProcessorParams {
        sample_length: new_samples.len(),
        samples: new_samples,
        ..params.clone()
    })
}

pub struct ReverbParams {
    pub predelay_ms: f64,
    pub wet_mix: f64,
    pub len_factor: f64,
    pub decay_factor: f64, // be careful
}

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
