use crate::{
    process::ProcessorParams,
    permute_error::PermuteError,
    permute_files::PermuteUpdate,
    process::{PermuteNodeEvent, PermuteNodeName},
    processors::{filter::{FilterType, FilterForm, FilterParams, multi_channel_filter}, 
    gain_distortion::{split_channels, interleave_channels}},
};

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
        let channel_idx = i as i32 % channels;
        let sample_group = i as i32 / channels;
        let reversed_group = (*sample_length as i32 / channels) - 1 - sample_group;
        let sample_i = (reversed_group * channels + channel_idx) as usize;
        new_samples[i] = samples[sample_i];
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
            v2 = if offset + 1 < cs.len() {
                cs[offset + 1]
            } else {
                cs[offset]
            };

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