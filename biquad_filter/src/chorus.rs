use std::collections::VecDeque;

use rand::distributions::uniform::SampleRange;

use crate::{delay::Delay, lfo};

pub struct Chorus {
    left_delays: Vec<Delay>,
    right_delays: Vec<Delay>,
    left_lfos: Vec<lfo::LFO>,
    right_lfos: Vec<lfo::LFO>,
    left_feedback_buffer: Box<VecDeque<f32>>,
    right_feedback_buffer: Box<VecDeque<f32>>,
    delay_ms: f32,
    delay_samples: usize,
    feedback: f32,
    depth: f32,
    sample_rate: f32,
    calc_depth: f32,
    wet: f32,
    dry: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32, delay_ms: f32, feedback: f32, depth: f32, rate: f32, wet: f32, dry: f32) -> Self {
        let mut left_delays: Vec<Delay> = Vec::with_capacity(5);
        let mut right_delays: Vec<Delay> = Vec::with_capacity(5);
        let mut left_lfos: Vec<lfo::LFO> = Vec::with_capacity(5);
        let mut right_lfos: Vec<lfo::LFO> = Vec::with_capacity(5);

        let delay_samples: usize = ((delay_ms as f32 / 1000.0) * sample_rate).round() as usize;

        for i in 0..5 {
            left_delays.push(Delay::new(sample_rate as usize, delay_samples, 0.0));
            right_delays.push(Delay::new(sample_rate as usize, delay_samples, 0.0));
            left_lfos.push(lfo::LFO::new_random_phase(sample_rate, rate));
            right_lfos.push(lfo::LFO::new_random_phase(sample_rate, rate));
        }

        let mut left_feedback_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        let mut right_feedback_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            left_feedback_buffer.push_front(0.0);
            right_feedback_buffer.push_front(0.0);
        }

        Self {
            left_delays,
            right_delays,
            left_lfos,
            right_lfos,
            left_feedback_buffer,
            right_feedback_buffer,
            sample_rate,
            feedback: feedback,
            depth: depth,
            calc_depth: 0.0,
            wet: wet,
            dry: dry,
            delay_ms,
            delay_samples: delay_samples,
        }
    }

    pub fn set_params(&mut self, sample_rate: f32, delay: f32, feedback: f32, depth: f32, rate: f32, wet: f32, dry: f32) {
        // resize all buffers relying on sample rate
        self.sample_rate = sample_rate;
    
        for (lfol, lfor) in self.left_lfos.iter_mut().zip(self.right_lfos.iter_mut()) {
            lfol.sample_rate = sample_rate;
            lfor.sample_rate = sample_rate;
        }

        let delay_samples: usize = ((delay as f32 / 1000.0) * self.sample_rate).round() as usize;

        for d in self.left_delays.iter_mut() {
            d.delay = delay_samples;
        }
        for d in self.right_delays.iter_mut() {
            d.delay = delay_samples;
        }

        self.feedback = feedback;

        self.depth = depth;
        self.calc_depth = depth / 1000.0 * self.sample_rate;
        // if self.calc_depth > self.delay_samples as f32 {
        //     self.calc_depth = self.delay_samples as f32;
        // }

        for (lfol, lfor) in self.left_lfos.iter_mut().zip(self.right_lfos.iter_mut()) {
            lfol.rate = rate;
            lfor.rate = rate;
        }

        self.wet = wet;
        self.dry = dry;
        self.delay_ms = delay;
        self.delay_samples = delay_samples;
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        for (dl, dr) in self.left_delays.iter_mut().zip(self.right_delays.iter_mut()) {
            dl.resize_buffers(sample_rate as usize);
            dr.resize_buffers(sample_rate as usize);
        }

        self.left_feedback_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_feedback_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            self.left_feedback_buffer.push_front(0.0);
            self.right_feedback_buffer.push_front(0.0);
        }
    }



    pub fn process_left(&mut self, x: f32) -> f32 {
        let xx = x + self.wet * self.feedback * self.left_feedback_buffer.get(self.delay_samples).unwrap();

        let offset1 = ((self.left_lfos[0].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);
        let offset2 = ((self.left_lfos[1].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);
        let offset3 = ((self.left_lfos[2].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);

        self.left_lfos[0].update_lfo();
        self.left_lfos[1].update_lfo();
        self.left_lfos[2].update_lfo();

        let mut delayed_signal = 0.0;
        delayed_signal += self.left_delays[0].process_sample(xx, (self.delay_samples as i32 + offset1) as usize);
        delayed_signal += self.left_delays[1].process_sample(xx, (self.delay_samples as i32 + offset2) as usize);
        delayed_signal += self.left_delays[2].process_sample(xx, (self.delay_samples as i32 + offset3) as usize);

        self.left_feedback_buffer.rotate_right(1);
        self.left_feedback_buffer[0] = delayed_signal / 3.0;

        let mut left_out = 
        self.dry * x 
        + self.wet * 1.0/3.0 * delayed_signal;

        if self.wet + self.dry > 1.0 {
            left_out /= self.wet + self.dry;
        }

        left_out
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let xx = x + self.wet * self.feedback * self.right_feedback_buffer.get(self.delay_samples).unwrap();

        let offset1 = ((self.right_lfos[0].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);
        let offset2 = ((self.right_lfos[1].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);
        let offset3 = ((self.right_lfos[2].next_value() * self.calc_depth / 2.0).round() as i32).clamp(-(self.delay_samples as i32) + 1 , self.delay_samples as i32 - 1);

        self.right_lfos[0].update_lfo();
        self.right_lfos[1].update_lfo();
        self.right_lfos[2].update_lfo();

        let mut delayed_signal = 0.0;
        delayed_signal += self.right_delays[0].process_sample(xx, (self.delay_samples as i32 + offset1) as usize);
        delayed_signal += self.right_delays[1].process_sample(xx, (self.delay_samples as i32 + offset2) as usize);
        delayed_signal += self.right_delays[2].process_sample(xx, (self.delay_samples as i32 + offset3) as usize);

        self.right_feedback_buffer.rotate_right(1);
        self.right_feedback_buffer[0] = delayed_signal / 3.0;

        let mut right_out = self.dry * x 
        + self.wet * 1.0/3.0 * delayed_signal;

        if self.wet + self.dry > 1.0 {
            right_out /= self.wet + self.dry;
        }

        right_out
    }
}