use biquad::*;
use std::f64::consts::PI;

pub type ProcessorFn = fn(ProcessorParams) -> ProcessorParams;

#[derive(Clone)]
pub struct ProcessorParams {
    pub spec: hound::WavSpec,
    pub samples: Vec<f64>,
    pub sample_length: usize,
    pub permutation: Permutation,

    pub update_progress:
        fn(permutation: Permutation, name: PermuteNodeName, event: PermuteNodeEvent),
}

#[derive(Clone)]

pub struct Permutation {
    pub file: String,
    pub permutation_index: usize,
    pub output: String,
}

#[derive(Debug, Clone)]
pub enum PermuteNodeEvent {
    NodeProcessStarted,
    NodeProcessComplete,
}

#[derive(Debug, Clone, Copy)]
pub enum PermuteNodeName {
    Reverse,
    RhythmicDelay,
    MetallicDelay,
    HalfSpeed,
    DoubleSpeed,
    Wow,
    Flutter,
    Chorus,
    Normalise,
}

pub fn reverse(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_progress,
        permutation,
    }: ProcessorParams,
) -> ProcessorParams {
    update_progress(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessStarted,
    );
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        new_samples[i] = samples[sample_length - 1 - i]
    }

    update_progress(
        permutation.clone(),
        PermuteNodeName::Reverse,
        PermuteNodeEvent::NodeProcessComplete,
    );
    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
        update_progress,
        permutation,
    };
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
        spec,
        update_progress,
        permutation,
    }: ProcessorParams,
    DelayLineParams {
        feedback_factor,
        delay_sample_length,
        dry_gain_factor,
        wet_gain_factor,
    }: DelayLineParams,
) -> ProcessorParams {
    let mut new_samples = vec![0_f64; sample_length];

    for i in delay_sample_length..sample_length {
        let delay_i = i - delay_sample_length;
        new_samples[i] += samples[delay_i];
    }

    new_samples = sum(vec![
        SampleLine {
            gain_factor: dry_gain_factor,
            samples: samples,
        },
        SampleLine {
            gain_factor: wet_gain_factor,
            samples: new_samples,
        },
    ]);

    let new_feedback_factor = feedback_factor.powf(1.5); // try and make feedback a bit less non linear
    let new_processor_params = ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
        update_progress,
        permutation,
    };
    let new_delay_params = DelayLineParams {
        feedback_factor: new_feedback_factor,
        delay_sample_length: delay_sample_length,
        dry_gain_factor: 1_f64,
        wet_gain_factor: feedback_factor,
    };

    if feedback_factor > 0_f64 {
        return delay_line(new_processor_params, new_delay_params);
    }

    return new_processor_params;
}

pub fn gain(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_progress,
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
        update_progress,
        permutation,
    };
}

pub fn normalise(params: ProcessorParams) -> ProcessorParams {
    ceiling(params, 1_f64)
}

pub fn ceiling(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_progress,
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
        update_progress,
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
        update_progress,
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
        update_progress,
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
        sample_length,
        spec,
        update_progress,
        permutation,
    }: ProcessorParams,
    VibratoParams { speed_hz, depth }: VibratoParams,
) -> ProcessorParams {
    let adjusted_depth = depth.powf(2_f64) * 512_f64; // ideally 1 should be a somewhat usable value
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

            ptr1 = ((i as f64 - 1_f64) + (speed * adjusted_depth)).floor() as usize;
            ptr2 = ptr1 + 1;

            if ptr2 >= channel_length {
                break;
            }

            ns[i] = cs[ptr1] + ((cs[ptr2] - cs[ptr1]) * speed);
        }
        new_channel_samples.push(ns);
    }

    let interleave_samples = interleave_channels(new_channel_samples);
    let interleave_sample_length = interleave_samples.len();

    return ProcessorParams {
        samples: interleave_samples,
        spec: spec,
        sample_length: interleave_sample_length,
        update_progress,
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
    let update_progress = params.update_progress;

    let delayed = delay_line(params.clone(), delay_params);
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
        update_progress: update_progress,
        permutation: params.permutation,
    };
}

pub type FilterType<T> = Type<T>;

pub struct FilterParams {
    pub frequency: f64,
    pub q: Option<f64>,
    pub filter_type: FilterType<f64>,
}

pub fn filter(
    ProcessorParams {
        samples,
        sample_length,
        spec,
        update_progress,
        permutation,
    }: ProcessorParams,
    FilterParams {
        filter_type,
        frequency,
        q,
    }: FilterParams,
) -> ProcessorParams {
    // Cutoff and sampling frequencies
    let f0 = frequency.hz();
    let fs = spec.sample_rate.hz();
    let q = q.unwrap_or(Q_BUTTERWORTH_F64);

    // Create coefficients for the biquads
    let coeffs = Coefficients::<f64>::from_params(filter_type, fs, f0, q).unwrap();
    let mut biquad1 = DirectForm2Transposed::<f64>::new(coeffs);

    let mut new_samples = vec![0_f64; sample_length];

    for i in 0..sample_length {
        new_samples[i] = biquad1.run(samples[i]);
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
        update_progress,
        permutation,
    };
}
