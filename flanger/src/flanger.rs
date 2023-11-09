use std::{collections::VecDeque, char::MAX, f32::consts::PI};

use crate::{lfo::{self, LFO}, delay::{Delay, self}};

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
    left_feedback_buffer: VecDeque<f32>,
    right_feedback_buffer: VecDeque<f32>,
    use_stereo_lfo: bool,
    wet: f32,
    dry: f32,
}

impl Flanger {
    pub fn new(sample_rate: f32) -> Self {
        let mut left_delay = Delay::new(sample_rate as usize, 0, 0.0);
        let mut right_delay = Delay::new(sample_rate as usize, 0, 0.0);
        left_delay.resize_buffers(sample_rate as usize);
        right_delay.resize_buffers(sample_rate as usize);

        let mut left_feedback_buffer: VecDeque<f32> = VecDeque::with_capacity(sample_rate as usize / 2);
        let mut right_feedback_buffer: VecDeque<f32> = VecDeque::with_capacity(sample_rate as usize / 2);

        for _ in 0..sample_rate as usize / 2 {
            left_feedback_buffer.push_back(0.0);
            right_feedback_buffer.push_back(0.0);
        }

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
            left_feedback_buffer: left_feedback_buffer,
            right_feedback_buffer: right_feedback_buffer,
            use_stereo_lfo: false,
            wet: 0.0,
            dry: 0.0,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.left_delay.resize_buffers(sample_rate as usize);
        self.right_delay.resize_buffers(sample_rate as usize);
        self.left_feedback_buffer = VecDeque::with_capacity(sample_rate  as usize / 2);
        self.right_feedback_buffer = VecDeque::with_capacity(sample_rate  as usize / 2);

        for _ in 0..(sample_rate as usize / 2) {
            self.left_feedback_buffer.push_back(0.0);
            self.right_feedback_buffer.push_back(0.0);
        }

        self.left_lfo.set_sample_rate(sample_rate as f32);
        self.right_lfo.set_sample_rate(sample_rate as f32);
    }

    pub fn set_params(&mut self, depth: f32, rate: f32, feedback: f32, wet: f32, dry: f32, stereo: bool) {
        self.rate = rate;
        self.left_lfo.rate = rate;
        self.right_lfo.rate = rate;

        self.feedback = feedback;
        self.left_delay.feedback = feedback;
        self.right_delay.feedback = feedback;

        self.depth = depth;
        self.calculated_depth = (depth * MAX_FLANGER_DELAY) * self.sample_rate;

        self.use_stereo_lfo = stereo;   
        self.wet = wet;
        self.dry = dry;     
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let delayed_signal = self.left_delay.process_sample(
            x, 
            (
                self.left_lfo.next_value_range(0.0..1.0) * self.calculated_depth 
            ) as usize);

        if self.wet + self.dry > 1.0 {
            return (self.dry * x + self.wet * delayed_signal) / (self.wet + self.dry)
        } else {
            return self.dry * x + self.wet * delayed_signal
        }
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let lfo_value = if self.use_stereo_lfo {
            self.right_lfo.update_lfo();
            self.left_lfo.update_lfo();
            self.right_lfo.next_value_range(0.0..1.0)
        } else {
            self.left_lfo.update_lfo();
            self.left_lfo.next_value_range(0.0..1.0)
        };
        let delayed_signal = self.right_delay.process_sample(
            x, 
            (
                lfo_value * self.calculated_depth 
            ) as usize);

        if self.wet + self.dry > 1.0 {
            return (self.dry * x + self.wet * delayed_signal) / (self.wet + self.dry)
        } else {
            return self.dry * x + self.wet * delayed_signal
        }
    }
}