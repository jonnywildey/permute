use std::f64::consts::PI;

// Basic implementation of sinwave.
// Assumes static frequency
pub fn lfo_sin(sample: usize, sample_rate: usize, lfo_rate: f64, phase: f64) -> f64 {
    ((sample as f64 / sample_rate as f64 * 2.0 * PI * lfo_rate) + (phase)).sin()
}

// Basic implementation of triangle wave.
// Assumes static frequency
pub fn lfo_tri(sample: usize, sample_rate: usize, lfo_rate: f64) -> f64 {
    // according to internet y = (A/P) * (P - abs(x % (2*P) - P) )
    // P = sample rate / 2
    // A = 2 (will need to subtract 1 to 0 center)
    // add p/2 to x to push phase 90 deg
    let cycle = sample_rate as f64 / lfo_rate as f64;
    let p = cycle / 2.0;
    return ((2.0 / p) * (p - ((sample as f64 + p / 2.0) % cycle - p).abs())) - 1.0;
}

// Basic implementation of exponential triangle wave.
// Assumes static frequency
#[allow(dead_code)]
pub fn lfo_tri_exp(sample: usize, sample_rate: usize, lfo_rate: f64, exp: f64) -> f64 {
    let cycle = sample_rate as f64 / lfo_rate as f64;
    let p = cycle / 2.0;
    return ((2.0 / p) * (p - ((sample as f64 + p / 2.0) % cycle - p).abs())).powf(exp) - 1.0;
}

#[derive(Debug, Clone)]
pub struct Oscillator {
    pub freq: f64,
    pub last_freq: f64,
    pub phase: f64,
    phase_inc: f64,
    sr: f64,
}

impl Oscillator {
    pub fn set_frequency(&mut self, new_freq: f64) {
        self.last_freq = self.freq;
        self.freq = new_freq;
        self.phase_inc = self.freq * 1.0_f64 / self.sr;
    }

    pub fn process(&mut self) -> f64 {
        // TODO - different wave types
        let t = -1.0_f64 + (2.0_f64 * self.phase);
        let out = 2.0_f64 * (t.abs() - 0.5_f64);
        self.phase += self.phase_inc;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
        return out;
    }
}

pub fn new_oscillator(sample_rate: f64) -> Oscillator {
    return Oscillator {
        sr: sample_rate,
        freq: 100_f64,
        last_freq: 100_f64,
        phase: 0_f64,
        phase_inc: 0_f64,
    };
}
