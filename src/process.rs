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

pub fn delay(params: ProcessorParams) -> ProcessorParams {
    delay_line(params, 0.75, 4000)
}

pub fn delay_line(
    ProcessorParams {
        samples,
        sample_length,
        spec,
    }: ProcessorParams,
    feedback_factor: f64, // 0 - 1
    delay_sample_length: usize,
) -> ProcessorParams {
    let mut new_samples = samples.clone();

    for i in 0..sample_length {
        let delay_i = i - delay_sample_length;
        if i >= delay_sample_length {
            new_samples[i] += samples[delay_i] * feedback_factor
        }
    }

    let new_params = ProcessorParams {
        samples: new_samples,
        spec: spec,
        sample_length: sample_length,
    };

    let new_feedback_factor = feedback_factor * feedback_factor;

    if new_feedback_factor > 0_f64 {
        return delay_line(new_params, new_feedback_factor, delay_sample_length);
    }
    return new_params;
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
