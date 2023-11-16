use std::{collections::VecDeque, char::MAX, f32::consts::PI};

use crate::{lfo::{self, LFO}, delay::{Delay, self}, filter::BiquadFilter};

const MAX_FLANGER_DELAY: f32 = 0.015; // 15ms

pub struct Flanger {
    sample_rate: f32,
    depth: f32,
    calculated_depth: f32,
    rate: f32,
    feedback: f32,
    left_lfo: lfo::LFO,
    right_lfo: lfo::LFO,
    left_delay: Delay,
    right_delay: Delay,
    left_feedback_buffer: f32,
    right_feedback_buffer: f32,
    use_stereo_lfo: bool,
    wet: f32,
    dry: f32,

    out_hpf: BiquadFilter,
}

impl Flanger {
    pub fn new(sample_rate: f32) -> Self {
        let mut left_delay = Delay::new(sample_rate as usize, 0.0);
        let mut right_delay = Delay::new(sample_rate as usize, 0.0);
        left_delay.resize_buffers(sample_rate as usize);
        right_delay.resize_buffers(sample_rate as usize);

        Self {
            sample_rate,
            depth: 0.0,
            calculated_depth: 0.0,
            rate: 0.0,
            feedback: 0.0,
            left_lfo: LFO::new_with_phase(sample_rate, 0.2, 0.0),
            right_lfo: LFO::new_with_phase(sample_rate, 0.2, PI / 2.0),
            left_delay,
            right_delay,
            left_feedback_buffer: 0.0,
            right_feedback_buffer: 0.0,
            use_stereo_lfo: false,
            wet: 0.0,
            dry: 0.0,
            out_hpf: BiquadFilter::new(),
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.left_delay.resize_buffers(sample_rate as usize);
        self.right_delay.resize_buffers(sample_rate as usize);
        self.left_feedback_buffer = 0.0;
        self.right_feedback_buffer = 0.0;

        self.left_lfo.set_sample_rate(sample_rate as f32);
        self.right_lfo.set_sample_rate(sample_rate as f32);

        self.out_hpf.set_sample_rate(sample_rate);
        self.out_hpf.second_order_hpf_coefficients(sample_rate, 30.0, 0.750);
    }

    pub fn set_params(&mut self, depth: f32, rate: f32, feedback: f32, wet: f32, dry: f32, stereo: bool) {
        self.rate = rate;
        self.left_lfo.rate = rate;
        self.right_lfo.rate = rate;

        self.feedback = feedback;

        self.depth = depth;
        self.calculated_depth = (depth * MAX_FLANGER_DELAY) * self.sample_rate;

        self.use_stereo_lfo = stereo;   
        self.wet = wet;
        self.dry = dry;     
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let xx = x + self.left_feedback_buffer * self.feedback;
        let delayed_signal = self.left_delay.process_sample(
            xx, 
            self.left_lfo.next_value_range(0.0..1.0) * self.calculated_depth );

        self.left_feedback_buffer = delayed_signal;

        if self.wet + self.dry > 1.0 {
            return self.out_hpf.process_left((self.dry * x + self.wet * delayed_signal) / (self.wet + self.dry))
        } else {
            return self.out_hpf.process_left(self.dry * x + self.wet * delayed_signal)
        }
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let lfo_value = if self.use_stereo_lfo {
            self.right_lfo.update_lfo();
            self.left_lfo.update_lfo();
            self.right_lfo.next_value_range(0.05..1.0)
        } else {
            self.left_lfo.update_lfo();
            self.left_lfo.next_value_range(0.05..1.0)
        };

        let xx = x + self.right_feedback_buffer * self.feedback;

        let delayed_signal = self.right_delay.process_sample(
            xx, 
            lfo_value * self.calculated_depth );

        self.right_feedback_buffer = delayed_signal;

        if self.wet + self.dry > 1.0 {
            return self.out_hpf.process_right((self.dry * x + self.wet * delayed_signal) / (self.wet + self.dry))
        } else {
            return self.out_hpf.process_right(self.dry * x + self.wet * delayed_signal)
        }
    }
}