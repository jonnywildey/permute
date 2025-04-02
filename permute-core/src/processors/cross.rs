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