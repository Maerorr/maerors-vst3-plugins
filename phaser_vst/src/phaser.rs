use std::collections::VecDeque;

use crate::{delayingallpass::DelayingAllPass, lfo, filter::BiquadFilter};

const PHASER_DELAYS: [f32; 12] = [
    16.0, 1600.0, 
    33.0, 3300.0, 
    48.0, 4800.0,
    98.0, 9800.0,
    160.0, 16000.0,
    260.0, 20480.0,
    ];

#[derive (Clone)]
pub struct Phaser {
    left_feedback_buffer: VecDeque<f32>,
    right_feedback_buffer: VecDeque<f32>,
    allpasses: Vec<BiquadFilter>,
    feedback: f32,
    rate: f32,
    sample_rate: f32,
    lfo: lfo::LFO,
    depth: f32,
    stages: usize,
    offset: f32,
    intensity: f32,
}

impl Phaser {
    pub fn new(sample_rate: f32) -> Self {
        let mut left_feedback_buffer: VecDeque<f32> = VecDeque::with_capacity(1000);
        let mut right_feedback_buffer: VecDeque<f32> = VecDeque::with_capacity(1000);
        for _ in 0..1000 {
            left_feedback_buffer.push_front(0.0);
            right_feedback_buffer.push_front(0.0);
        }

        let mut allpasses: Vec<BiquadFilter> = Vec::new();
        for i in 0..6 {
            let mut allpass = BiquadFilter::new();
            allpass.first_order_allpass_coefficients(sample_rate, PHASER_DELAYS[i*2]);
            allpasses.push(allpass);
        }

        let lfo = lfo::LFO::new(sample_rate, 0.2);

        Self {
            left_feedback_buffer,
            right_feedback_buffer,
            allpasses,
            feedback: 0.0,
            rate: 0.0,
            sample_rate: 44100.0,
            lfo: lfo,
            depth: 0.0,
            stages: 0,
            offset: 0.0,
            intensity: 0.0,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.left_feedback_buffer = VecDeque::with_capacity(sample_rate as usize);
        self.right_feedback_buffer = VecDeque::with_capacity(sample_rate as usize);
        for _ in 0..(sample_rate as usize) {
            self.left_feedback_buffer.push_front(0.0);
            self.right_feedback_buffer.push_front(0.0);
        }
        self.lfo.sample_rate = sample_rate;
    }

    pub fn set_params(&mut self, rate: f32, depth: f32, stages: usize, offset: f32, feedback: f32, intensity: f32) {
        self.rate = rate;
        self.lfo.rate = rate;
        self.depth = depth;
        self.offset = offset;
        self.stages = stages;
        self.intensity = intensity;
        self.feedback = feedback;
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let y: f32;
        let mut phased_signal = 
            x + self.feedback * self.left_feedback_buffer[0];
        

        for i in 0..(self.stages * 2) {
            self.allpasses[i].first_order_allpass_coefficients(
                self.sample_rate, 
                lerp(PHASER_DELAYS[2*i], 
                PHASER_DELAYS[2*i+1],
                (self.lfo.next_value() * self.depth + self.offset).clamp(-1.0, 1.0) / 2.0 + 0.5
            ));
            phased_signal = self.allpasses[i].process_left(phased_signal);
        }
        self.left_feedback_buffer.rotate_right(1);
        self.left_feedback_buffer[0] = phased_signal;

        let x_gain = 1.0 - self.intensity / 2.0;
        y = x_gain * x + self.intensity / 2.0 * phased_signal;
        y
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let y: f32;
        let mut phased_signal = 
            x + self.feedback * self.right_feedback_buffer[0];
        

        for i in 0..(self.stages * 2) {
            self.allpasses[i].first_order_allpass_coefficients(
                self.sample_rate, 
                lerp(PHASER_DELAYS[2*i], 
                PHASER_DELAYS[2*i+1],
                (self.lfo.next_value() * self.depth + self.offset).clamp(-1.0, 1.0) / 2.0 + 0.5
            ));
            phased_signal = self.allpasses[i].process_right(phased_signal);
        }

        self.right_feedback_buffer.rotate_right(1);
        self.right_feedback_buffer[0] = phased_signal;

        // do this once, in right channel since both channels share a common
        self.lfo.update_lfo();
        let x_gain = 1.0 - self.intensity / 2.0;
        y = x_gain * x + self.intensity / 2.0 * phased_signal;
        y
    }
    
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}