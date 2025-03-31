use std::f64::consts::PI;
use strum::EnumIter;

use crate::{
permute_files::PermuteUpdate,
processors::gain_distortion::{split_channels, interleave_channels, sum, SampleLine},
process::{PermuteNodeEvent, PermuteNodeName, ProcessorParams},
permute_error::PermuteError,
processors::filter::FilterType,
processors::osc::{lfo_sin, lfo_tri, new_oscillator},
};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz};

use super::delay_reverb::delay_line;
use super::delay_reverb::DelayLineParams;

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

    update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation.clone(),
        PermuteNodeName::Tremolo,
        PermuteNodeEvent::NodeProcessComplete,
    ))?;
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
                // val = sumframes.iter().fold(0.0, |acc, v| acc + v) / frame_count_f;
                // max
                val = sumframes.iter().fold(0.0, |acc, v| {
                    if *v > acc {
                        return *v;
                    } else {
                        return acc;
                    }
                });
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

    update_sender.send(PermuteUpdate::UpdatePermuteNodeCompleted(
        permutation.clone(),
        PermuteNodeName::Lazer,
        PermuteNodeEvent::NodeProcessComplete,
    ))?;
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

#[derive(Clone, EnumIter, Debug)]
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

