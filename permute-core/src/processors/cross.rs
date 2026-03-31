use biquad::{Coefficients, DirectForm2Transposed, ToHertz, Biquad};
use crate::{
process::ProcessorParams,
permute_error::PermuteError,
processors::gain_distortion::{DistortionAlgorithm, apply_distortion, calculate_rms},
rms_cache::{get_cached_rms, cache_rms},
audio_cache::AUDIO_CACHE,
};

#[derive(Debug, Clone)]
pub struct CrossGainParams {
    pub sidechain_file: String,
    pub depth: f64,
    pub invert: bool,
    pub window_size_ms: f64,
}

#[derive(Debug, Clone)]
pub struct CrossFilterParams {
    pub sidechain_file: String,
    pub filter_type: biquad::Type<f64>,
    pub base_freq: f64,
    pub max_freq: f64,
    pub q: f64,
    pub window_size_ms: f64,
    pub invert: bool,
}

pub fn cross_gain(params: &ProcessorParams, gain_params: &CrossGainParams) -> Result<ProcessorParams, PermuteError> {
    // Get the RMS signal from the sidechain file
    let rms_signal = get_sidechain_rms_signal(
        &gain_params.sidechain_file,
        gain_params.window_size_ms,
        params.samples.len(),
        params.sample_rate,
    )?;

    // Apply gain modulation
    let mut new_samples = params.samples.clone();
    for (i, sample) in new_samples.iter_mut().enumerate() {
        let rms = if gain_params.invert {
            1.0 - rms_signal[i]
        } else {
            rms_signal[i]
        };
        *sample *= 1.0 - (gain_params.depth * rms);
    }

    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}

pub fn cross_filter(params: &ProcessorParams, filter_params: &CrossFilterParams) -> Result<ProcessorParams, PermuteError> {
    // Get the RMS signal from the sidechain file
    let rms_signal = get_sidechain_rms_signal(
        &filter_params.sidechain_file,
        filter_params.window_size_ms,
        params.samples.len(),
        params.sample_rate,
    )?;

    let mut new_samples = params.samples.clone();
    
    // Create initial coefficients
    let initial_coeffs = Coefficients::<f64>::from_params(
        filter_params.filter_type,
        (params.sample_rate as u32).hz(),
        filter_params.base_freq.hz(),
        filter_params.q,
    )?;
    let mut filter = DirectForm2Transposed::<f64>::new(initial_coeffs);
    
    // Process each sample
    for (i, sample) in new_samples.iter_mut().enumerate() {
        let rms = if filter_params.invert {
            1.0 - rms_signal[i]
        } else {
            rms_signal[i]
        };
        
        // Calculate the current frequency based on RMS
        let freq = filter_params.base_freq + (rms * (filter_params.max_freq - filter_params.base_freq));
        
        // Update filter coefficients
        let coeffs = Coefficients::<f64>::from_params(
            filter_params.filter_type,
            (params.sample_rate as u32).hz(),
            freq.hz(),
            filter_params.q,
        )?;
        filter.update_coefficients(coeffs);
        
        // Process the sample
        *sample = filter.run(*sample);
    }

    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}

pub fn get_sidechain_rms_signal(
    sidechain_file: &str,
    window_size_ms: f64,
    target_length: usize,
    target_sample_rate: usize,
) -> Result<Vec<f64>, PermuteError> {
    // Check if RMS values are in cache
    if let Some(cached_rms) = get_cached_rms(sidechain_file, window_size_ms, target_length, target_sample_rate) {
        return Ok(cached_rms);
    }
    
    // If not in cache, calculate RMS values
    let samples = AUDIO_CACHE.get_samples(sidechain_file)?;
    
    // Convert window size from ms to samples
    let window_size = ((window_size_ms / 1000.0) * target_sample_rate as f64) as usize;
    
    // Calculate RMS values
    let rms_values = calculate_rms(&samples, window_size);
    
    // Resample to match target length if necessary
    let final_rms = if rms_values.len() != target_length {
        let mut resampled = Vec::with_capacity(target_length);
        for i in 0..target_length {
            let idx = (i as f64 * (rms_values.len() as f64 - 1.0) / (target_length as f64 - 1.0)) as usize;
            resampled.push(rms_values[idx]);
        }
        resampled
    } else {
        rms_values
    };

    // Cache the calculated RMS values
    cache_rms(
        sidechain_file.to_string(),
        window_size_ms,
        target_length,
        target_sample_rate,
        final_rms.clone()
    );

    Ok(final_rms)
}

#[derive(Debug, Clone)]
pub struct CrossMixParams {
    pub sidechain_file: String,
    pub offset_samples: usize, // offset in samples, pre-aligned to channel count
    pub mix: f64,              // blend of sidechain (0.0 = dry, 1.0 = sidechain only)
}

pub fn cross_mix(params: &ProcessorParams, mix_params: &CrossMixParams) -> Result<ProcessorParams, PermuteError> {
    let sidechain = AUDIO_CACHE.get_samples(&mix_params.sidechain_file)?;

    let current_len = params.samples.len();
    let offset = mix_params.offset_samples;
    let output_len = current_len.max(offset + sidechain.len());

    let mut output = vec![0.0f64; output_len];

    for (i, &s) in params.samples.iter().enumerate() {
        output[i] += s * (1.0 - mix_params.mix);
    }
    for (i, &s) in sidechain.iter().enumerate() {
        output[offset + i] += s * mix_params.mix;
    }

    Ok(ProcessorParams {
        samples: output,
        ..params.clone()
    })
}

#[derive(Debug, Clone)]
pub struct CrossGrainParams {
    pub sidechain_file: String,
    pub grain_samples: usize, // grain size in samples, multiple of channels
    pub blend_samples: usize, // crossfade length in samples, multiple of channels
}

/// Find the next zero crossing at or after `nominal`, stepping by `channels`,
/// searching up to `max_search` samples ahead. Falls back to the channel-aligned
/// nominal position if none is found.
fn find_zero_crossing(samples: &[f64], nominal: usize, max_search: usize, channels: usize) -> usize {
    let aligned = (nominal / channels) * channels;
    let limit = (aligned + max_search).min(samples.len().saturating_sub(channels));
    let mut pos = aligned;
    while pos + channels < limit {
        if (samples[pos] >= 0.0) != (samples[pos + channels] >= 0.0) {
            return pos;
        }
        pos += channels;
    }
    aligned
}

fn apply_grain_fade(samples: &[f64], blend: usize, fade_in: bool, fade_out: bool) -> Vec<f64> {
    let len = samples.len();
    let blend = blend.min(len / 2);
    let mut result = samples.to_vec();
    if fade_in && blend > 0 {
        for i in 0..blend {
            result[i] *= i as f64 / blend as f64;
        }
    }
    if fade_out && blend > 0 {
        for i in 0..blend {
            result[len - 1 - i] *= i as f64 / blend as f64;
        }
    }
    result
}

pub fn cross_grain(params: &ProcessorParams, grain_params: &CrossGrainParams) -> Result<ProcessorParams, PermuteError> {
    let sidechain = AUDIO_CACHE.get_samples(&grain_params.sidechain_file)?;
    let channels = params.channels.max(1);
    let grain_samples = grain_params.grain_samples.max(channels);
    let blend_samples = grain_params.blend_samples;
    let current_len = params.samples.len();
    let sidechain_len = sidechain.len();

    if sidechain_len == 0 || current_len == 0 {
        return Ok(params.clone());
    }

    // Build zero-crossing-snapped grain boundaries for the current audio
    let max_snap = grain_samples / 2;
    let mut boundaries: Vec<usize> = vec![0];
    let mut pos = grain_samples;
    while pos < current_len {
        let snapped = find_zero_crossing(&params.samples, pos, max_snap, channels);
        boundaries.push(snapped);
        pos = snapped + grain_samples;
    }
    let num_grains = boundaries.len();

    let mut output: Vec<f64> = Vec::with_capacity(current_len * 2);
    let mut sc_pos = 0usize;

    for (i, &start) in boundaries.iter().enumerate() {
        let end = if i + 1 < num_grains { boundaries[i + 1] } else { current_len };
        let grain_len = end.saturating_sub(start);
        if grain_len == 0 {
            continue;
        }

        let is_first = i == 0;
        let is_last = i == num_grains - 1;

        // Current audio grain: no fade-in on the very first, fade-out on all
        let current_grain = &params.samples[start..start + grain_len];
        output.extend(apply_grain_fade(current_grain, blend_samples, !is_first, true));

        // Sidechain grain cycled from sc_pos, wrapping around if needed
        let mut sc_grain = Vec::with_capacity(grain_len);
        for j in 0..grain_len {
            sc_grain.push(sidechain[(sc_pos + j) % sidechain_len]);
        }
        // Advance sc_pos, keeping it channel-aligned
        sc_pos = ((sc_pos + grain_len) % sidechain_len) / channels * channels;

        // Sidechain grain: fade-in on all, no fade-out on the very last
        output.extend(apply_grain_fade(&sc_grain, blend_samples, true, !is_last));
    }

    Ok(ProcessorParams {
        sample_length: output.len(),
        samples: output,
        ..params.clone()
    })
}

#[derive(Debug, Clone)]
pub struct CrossDistortParams {
    pub sidechain_file: String,
    pub min_factor: f64,
    pub max_factor: f64,
    pub window_size_ms: f64,
    pub algorithm: DistortionAlgorithm,
    pub invert: bool,
}

pub fn cross_distort(params: &ProcessorParams, distort_params: &CrossDistortParams) -> Result<ProcessorParams, PermuteError> {
    // Get the RMS signal from the sidechain file
    let rms_signal = get_sidechain_rms_signal(
        &distort_params.sidechain_file,
        distort_params.window_size_ms,
        params.samples.len(),
        params.sample_rate,
    )?;

    let mut new_samples = params.samples.clone();
    
    // Process each sample
    for (i, sample) in new_samples.iter_mut().enumerate() {
        let rms = if distort_params.invert {
            1.0 - rms_signal[i]
        } else {
            rms_signal[i]
        };
        
        // Calculate the current distortion factor based on RMS
        let factor = distort_params.min_factor + (rms * (distort_params.max_factor - distort_params.min_factor));
        
        // Apply the selected distortion algorithm
        *sample = apply_distortion(*sample, factor, distort_params.algorithm);
    }

    Ok(ProcessorParams {
        samples: new_samples,
        ..params.clone()
    })
}