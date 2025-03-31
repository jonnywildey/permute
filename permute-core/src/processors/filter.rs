pub type FilterType<T> = Type<T>;
use biquad::{Biquad, Coefficients, DirectForm1, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F64};
use crate::{
process::ProcessorParams,
permute_error::PermuteError,
processors::gain_distortion::{split_channels, interleave_channels},
processors::osc::lfo_tri,
};

#[derive(Clone, Debug)]
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