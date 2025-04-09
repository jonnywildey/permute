use rand::{rngs::ThreadRng, Rng};

pub trait DistributionRng {
    fn gen_distribution<T: Copy>(&mut self, distribution: Vec<(T, f64)>) -> T;
}

impl DistributionRng for ThreadRng {
    fn gen_distribution<T: Clone>(&mut self, distribution: Vec<(T, f64)>) -> T {
        let total_probability = distribution.iter().map(|(_, p)| p).sum::<f64>();
        let random_value = self.gen_range(0.0..total_probability);
        // create a cumulative distribution
        let cumulative_distribution = distribution.iter().scan(0.0, |acc, (_, p)| {
            *acc += p;
            Some(*acc)
        }).collect::<Vec<f64>>();
        // find the index of the cumulative distribution that is greater than the random value
        let index = cumulative_distribution.iter().position(|p| *p > random_value).unwrap_or(distribution.len() - 1);
        distribution[index].0.clone()
    }
}

pub fn format_float(value: f64) -> String {
    format!("{:.2}", value)
}

pub fn format_hz(value: f64) -> String {
    format!("{:.2} hz", value)
}

pub fn format_hz_usize(value: usize) -> String {
    format!("{} hz", value)
}

pub fn format_float_percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

pub fn format_float_ms(value: f64) -> String {
    format!("{:.2} ms", value)
}

pub fn format_samples_as_ms(samples: usize, sample_rate: usize) -> String {
    format!("{:.2} ms", (samples as f64 / sample_rate as f64) * 1000.0)
}

pub fn get_filename(path: &str) -> String {
    path.split('/').last().unwrap_or(path).to_string()
}

pub fn format_factor_to_pitch(factor: f64) -> String {
    let pitch = 12.0 * (factor / 2.0).log2();
    format!("{:.2} semitones", pitch)
}