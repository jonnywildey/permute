use std::f64::consts::PI;
use std::f64::consts::E;
use crate::permute_files::PermuteUpdate;
use crate::process::{PermuteNodeEvent, PermuteNodeName, ProcessorParams};
use crate::permute_error::PermuteError;

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

pub fn split_channels(samples: Vec<f64>, channels: usize) -> Vec<Vec<f64>> {
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

pub fn interleave_channels(
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

pub fn distort(params: &ProcessorParams, factor: f64) -> Result<ProcessorParams, PermuteError> {
    let new_samples = params
        .samples
        .iter()
        .map(|f| apply_distortion(*f, factor, DistortionAlgorithm::Power))
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
        .map(|f| apply_distortion(*f, 1.0, DistortionAlgorithm::Saturate))
        .collect();
    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}

pub fn trim(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    let threshold = 0.001;
    params
        .update_sender
        .send(PermuteUpdate::UpdatePermuteNodeStarted(
            params.permutation.clone(),
            PermuteNodeName::Trim,
            PermuteNodeEvent::NodeProcessStarted,
        ))?;

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

    params
        .update_sender
        .send(PermuteUpdate::UpdatePermuteNodeCompleted(
            params.permutation.clone(),
            PermuteNodeName::Trim,
            PermuteNodeEvent::NodeProcessComplete,
        ))?;
    Ok(ProcessorParams {
        samples: new_samples,
        sample_length: len,
        ..params.clone()
    })
}

// Calculate RMS values for a given window size
pub fn calculate_rms(samples: &[f64], window_size: usize) -> Vec<f64> {
    if window_size == 0 {
        return vec![0.0; samples.len()];
    }

    let mut rms_values = Vec::with_capacity(samples.len());
    let half_window = window_size / 2;

    // First pass: calculate all RMS values
    for i in 0..samples.len() {
        let start = if i < half_window { 0 } else { i - half_window };
        let end = if i + half_window >= samples.len() {
            samples.len()
        } else {
            i + half_window
        };

        let sum_squares: f64 = samples[start..end]
            .iter()
            .map(|&x| x * x)
            .sum();
        let rms = (sum_squares / (end - start) as f64).sqrt();
        rms_values.push(rms);
    }

    // Find the maximum RMS value for normalization
    let max_rms = rms_values.iter().fold(0.0_f64, |max, &val| max.max(val));
    
    // Normalize all values if max_rms is greater than 0
    if max_rms > 0.0 {
        for rms in rms_values.iter_mut() {
            *rms /= max_rms;
        }
    }

    rms_values
}

#[derive(Debug, Clone, Copy)]
pub enum DistortionAlgorithm {
    Power,      // Original algorithm
    Tanh,       // Hyperbolic tangent
    Atan,       // Arctangent
    Cubic,      // Soft clipping with cubic function
    Saturate,   // 1 - e^-|x|
}

pub fn apply_distortion(sample: f64, factor: f64, algorithm: DistortionAlgorithm) -> f64 {
    let s = sample.signum();
    let abs = sample.abs();
    
    match algorithm {
        DistortionAlgorithm::Power => {
            abs.powf(factor) * s
        },
        DistortionAlgorithm::Tanh => {
            (abs * factor).tanh() * s
        },
        DistortionAlgorithm::Atan => {
            (abs * factor).atan() * (PI/2.0).recip() * s
        },
        DistortionAlgorithm::Cubic => {
            let x = abs * factor;
            if x > 1.0 {
                s
            } else {
                (1.5 * x - 0.5 * x.powi(3)) * s
            }
        },
        DistortionAlgorithm::Saturate => {
            s * (1.0 - E.powf(-1.0 * abs * factor))
        },
    }
}

#[derive(Clone)]
pub struct FuzzParams {
    pub gain: f64,
    pub output_gain: f64,
}

pub fn fuzz(params: ProcessorParams, FuzzParams { gain, output_gain }: FuzzParams) -> Result<ProcessorParams, PermuteError> {
    let new_samples = params
        .samples
        .iter()
        .map(|f| {
            let distorted = f * gain;
            let clipped = if distorted > 1.0 {
                1.0
            } else if distorted < -1.0 {
                -1.0
            } else {
                distorted
            };
            clipped * output_gain
        })
        .collect();
    Ok(ProcessorParams {
        samples: new_samples,
        ..params
    })
}
