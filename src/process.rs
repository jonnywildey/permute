use biquad::*;
use hound::WavSpec;
use std::f64::consts::PI;

#[derive(Clone)]
pub struct ProcessorParams {
    pub spec: hound::WavSpec,
    pub samples: Vec<f64>,
    pub sample_length: usize,
}

pub fn reverse(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        new_samples[i] = samples[sample_length - 1 - i]
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
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
    }: ProcessorParams,
    ceiling: f64,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    let abs_max = samples.iter().fold(0_f64, |a, b| {
        let b = b.abs();
        if a >= b {
            a
        } else {
            b
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

    for i in 0..samples_len {
        for j in 0..sample_lines_len {
            new_samples[i] += sample_lines[j].samples[i] * sample_lines[j].gain_factor;
        }
    }
    new_samples
}

pub fn change_speed(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
    speed: f64,
) -> ProcessorParams {
    let mut new_sample_length: usize = ((sample_length as f64) / speed).ceil() as usize;
    let sample_mod = new_sample_length % spec.channels as usize;
    if sample_mod > 0 {
        new_sample_length = new_sample_length - sample_mod;
    }
    let mut new_samples = vec![0_f64; new_sample_length];

    println!(
        "{}, {}, {}",
        sample_length,
        new_sample_length,
        sample_length as f64 / speed
    );

    new_samples[0] = samples[0];
    let mut ptr1: usize;
    let mut ptr2: usize;
    for i in 1..new_sample_length {
        ptr1 = ((i as f64 - 1_f64) * speed).floor() as usize;
        ptr2 = (i as f64 * speed).floor() as usize;

        new_samples[i] = samples[ptr1] + ((samples[ptr2] - samples[ptr1]) * speed);
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: new_sample_length,
    };
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
    }: ProcessorParams,
    VibratoParams { speed_hz, depth }: VibratoParams,
) -> ProcessorParams {
    let adjusted_depth = depth * 0.001; // ideally 1 should be a somewhat usable value
    let mut new_samples = vec![0_f64; sample_length];

    new_samples[0] = samples[0];
    let mut ptr1: usize;
    let mut ptr2: usize;
    let mut speed: f64;
    let mut cos_amplitude: f64;
    for i in 1..sample_length {
        cos_amplitude = (i as f64 / spec.sample_rate as f64 * 2.0 * PI * speed_hz).cos();
        speed = 1_f64 + cos_amplitude * adjusted_depth;

        ptr1 = ((i as f64 - 1_f64) * speed).floor() as usize;
        ptr2 = (i as f64 * speed).floor() as usize;

        if ptr2 >= sample_length {
            break;
        }

        new_samples[i] = samples[ptr1] + ((samples[ptr2] - samples[ptr1]) * speed);
    }

    return ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
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

    let delayed = delay_line(params, delay_params);
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
        sample_length: vibratod.sample_length,
        samples: summed,
        spec: vibratod.spec,
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
    };
}

pub struct PhaserParams {
    pub frequency_hz: f64,
    pub q: Option<f64>,
    pub phases: i32,
}

// pub fn phaser(
//     ProcessorParams {
//         samples,
//         sample_length,
//         spec,
//     }: ProcessorParams,
//     PhaserParams {
//         frequency_hz,
//         phases,
//         q,
//     }: PhaserParams,
// ) -> ProcessorParams {
//     let dry_samples = samples.clone();
//     // Cutoff and sampling frequencies
//     let f0 = frequency_hz.hz();
//     let fs = spec.sample_rate.hz();
//     let q = q.unwrap_or(Q_BUTTERWORTH_F64);

//     let coeffs = Coefficients::<f64>::from_params(FilterType::AllPass, fs, f0, q).unwrap();
//     let mut filters: Vec<DirectForm2Transposed<f64>> = vec![0..phases]
//         .iter()
//         .map(|_| DirectForm2Transposed::<f64>::new(coeffs))
//         .collect();

//     let mut filtered_samples = samples.clone();

//     for i in 0..sample_length {
//         let cos_amplitude = (i as f64 / spec.sample_rate as f64 * 2.0 * PI * 100_f64).cos();
//         let depth = 1_f64 + cos_amplitude * 0.1;

//         let coeffs = Coefficients::<f64>::from_params(
//             FilterType::AllPass,
//             fs,
//             (frequency_hz * depth).hz(),
//             q,
//         )
//         .unwrap();

//         for j in 0..filters.len() {
//             filters[j].update_coefficients(coeffs);
//             filtered_samples[i] = filters[j].run(filtered_samples[i]);
//         }
//     }

//     let summed = sum(vec![
//         SampleLine {
//             samples: dry_samples,
//             gain_factor: 1_f64,
//         },
//         SampleLine {
//             samples: filtered_samples,
//             gain_factor: 1.5_f64,
//         },
//     ]);

//     return ProcessorParams {
//         sample_length: sample_length,
//         samples: summed,
//         spec: spec,
//     };
// }
