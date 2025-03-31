

pub fn format_float(value: f64) -> String {
    format!("{:.2}", value)
}

pub fn format_hz(value: f64) -> String {
    format!("{:.2} hz", value)
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