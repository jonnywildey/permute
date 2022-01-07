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

pub fn half_gain(params: ProcessorParams) -> ProcessorParams {
    gain(params, 0.5)
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

pub fn half_speed(params: ProcessorParams) -> ProcessorParams {
    change_speed(params, 0.5_f64)
}
pub fn double_speed(params: ProcessorParams) -> ProcessorParams {
    change_speed(params, 2_f64)
}

pub fn change_speed(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
    speed: f64,
) -> ProcessorParams {
    let new_sample_length: usize = ((sample_length as f64) / speed).ceil() as usize;
    let mut new_samples = vec![0_f64; new_sample_length];

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
        // speed = 1_f64 + (i as f64 / spec.sample_rate as f64 * speed_hz).sin() * 0.001 * depth;
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
