use std::{f32::consts::PI, ops::Range};

use rand::Rng;

pub struct LFO {
    pub rate: f32,
    phase: f32,
    pub sample_rate: f32
}

impl LFO {
    pub fn new(sample_rate: f32, rate: f32) -> Self {
        Self {
            sample_rate,
            rate,
            phase: 0.0,
        }
    }

    pub fn new_random_phase(sample_rate: f32, rate: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            sample_rate,
            rate,
            phase: rng.gen_range(0.0..(2.0 * PI))
        }
    }

    pub fn new_with_phase(sample_rate: f32, rate: f32, phase: f32) -> Self {
        Self {
            sample_rate,
            rate,
            phase
        }
    }

    /// returns next value of LFO. Values of <-1, 1>
    pub fn next_value(&mut self) -> f32 {
        self.phase.sin()
    }

    pub fn next_value_range(&mut self, range: Range<f32>) -> f32 {
        let value = self.next_value();
        let scaled = (value + 1.0) / 2.0;
        let scaled = scaled * (range.end - range.start) + range.start;
        scaled
    }

    pub fn update_lfo(&mut self) {
        self.phase += 2.0 * std::f32::consts::PI * self.rate / self.sample_rate;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
    }
}