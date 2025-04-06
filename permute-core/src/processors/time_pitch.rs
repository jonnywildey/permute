use crate::{
    process::ProcessorParams,
    permute_error::PermuteError,
    permute_files::PermuteUpdate,
    process::{PermuteNodeEvent, PermuteNodeName},
    processors::{filter::{FilterType, FilterForm, FilterParams, multi_channel_filter}, 
    gain_distortion::{split_channels, interleave_channels}},
};
use std::f64::consts::PI;
use rand::{rngs::ThreadRng, Rng};
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

pub fn reverse(
    params: ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    let mut new_samples = params.samples.clone();
    let channels = params.channels as i32;

    for i in 0..params.sample_length {
        let channel_idx = i as i32 % channels;
        let sample_group = i as i32 / channels;
        let reversed_group = (params.sample_length as i32 / channels) - 1 - sample_group;
        let sample_i = (reversed_group * channels + channel_idx) as usize;
        new_samples[i] = params.samples[sample_i];
    }
    return Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
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
        .map(|(_, d)| {
            let a = d[0];
            let b = d[1];
            return (a, b);
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
                        let f = (pos as f64 / (half_blend) as f64);

                        new_samples.push(params.samples[j] * f);
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

#[derive(Debug, Clone, Copy)    ]
pub enum WindowType { 
    Hamming,
    Blackman,
}

pub struct StftTimeStretchParams {
    pub window_size: usize,
    pub hop_size: usize,
    pub stretch_factor: f64,
    pub rng: ThreadRng,
    pub window_type: WindowType,
}

fn hamming_window(window_size: usize) -> Vec<f64> {
    (0..window_size).map(|i| 0.54 - 0.46 * (2.0 * PI * i as f64 / 
        (window_size - 1) as f64).cos()).collect()
}

fn blackman_window(window_size: usize) -> Vec<f64> {
    (0..window_size).map(|i| 0.42 - 0.5 * (2.0 * PI * i as f64 / 
        (window_size - 1) as f64).cos() + 0.08 * (4.0 * PI * i as f64 / 
            (window_size - 1) as f64).cos()).collect()
}

pub fn stft_time_stretch(
    params: &ProcessorParams,
    StftTimeStretchParams {
        window_size,
        hop_size,
        stretch_factor,
        window_type,
        mut rng
    }: StftTimeStretchParams,
) -> Result<ProcessorParams, PermuteError> {
    // Split into channels
    let channel_samples = split_channels(params.samples.clone(), params.channels);
    let mut new_channel_samples: Vec<Result<Vec<f64>, PermuteError>> = vec![];

    // Create FFT planner and plan forward/backward FFTs
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    let ifft = planner.plan_fft_inverse(window_size);

    for channel in channel_samples {
        let samples = channel;

        // Create Blackman window
        let window = match window_type {   
            WindowType::Hamming => hamming_window(window_size),
            WindowType::Blackman => blackman_window(window_size),
        };


        // Calculate number of frames
        // let num_frames = (padded_len - window_size) / hop_size + 1;
        let num_frames =  ((samples.len() as f64 / hop_size as f64) * stretch_factor) as usize;
        // println!("num_frames: {:?}, frame length sec: {}, hop size: {}, stretch factor: {}", num_frames, window_size as f64 / params.sample_rate as f64, hop_size as f64 / params.sample_rate as f64, stretch_factor);
   
        // Buffer for FFT processing
        let mut fft_buffer = vec![Complex::new(0.0, 0.0); window_size];
        
        let mut all_frames: Vec<Vec<f64>> = vec![];
        // Process each frame
        let mut current_pos = 0;
        for _frame in 0..num_frames {
            let mut frame_buffer: Vec<f64> = vec![];
            // Calculate frame positions with proper stretching
            let analysis_pos = current_pos;
            
            // Extract and window the frame
            for i in 0..window_size {
                if analysis_pos + i < samples.len() {
                    fft_buffer[i] = Complex::new(
                        samples[analysis_pos + i] * window[i],
                        0.0
                    );
                } else {
                    fft_buffer[i] = Complex::new(0.0, 0.0);
                }
            }
            current_pos = current_pos + (hop_size as f64 / stretch_factor) as usize;
            
            // Forward FFT
            fft.process(&mut fft_buffer);
            
            // Phase vocoder processing
            for i in 0..fft_buffer.len() {                    
                let magnitude = fft_buffer[i].norm();
                let random_phase = rng.gen_range(0.0..2.0 * PI);
                let new_phase =  fft_buffer[i].arg() + random_phase;
                fft_buffer[i] = Complex::from_polar(magnitude, new_phase);
            }
                        
            // Inverse FFT
            ifft.process(&mut fft_buffer);
            for i in 0..window_size {
                frame_buffer.push(fft_buffer[i].re * window[i]);
            }
            all_frames.push(frame_buffer);
        }
        // Overlap-add with window
        let output_len = ((samples.len() as f64 * stretch_factor) as usize);
        let mut pos: usize = 0;
        let mut output_buffer: Vec<f64> = vec![0.0; output_len];
        for frame in all_frames {
            let current_pos = pos;
            for i in 0..window_size {
                if pos < output_len {
                    output_buffer[pos] += frame[i];
                    pos += 1;
                }
            }
            pos = current_pos + hop_size;
            // println!("pos: {}, sec {},", pos, pos as f64 / params.sample_rate as f64);
        }
        
        new_channel_samples.push(Ok(output_buffer));
    }

    // Interleave channels back together
    let new_channel_samples = new_channel_samples.into_iter().collect();

    let interleave_samples = interleave_channels(new_channel_samples).unwrap();
    let interleave_sample_length = interleave_samples.len();
    
    Ok(ProcessorParams {
        samples: interleave_samples,
        sample_length: interleave_sample_length,
        ..params.clone()
    })
}

