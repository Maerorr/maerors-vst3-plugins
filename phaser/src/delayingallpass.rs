use std::collections::VecDeque;

use crate::delay::Delay;

#[derive(Clone)]
pub struct DelayingAllPass {
    delay_samples: usize,
    delay_ms: f32,
    gain: f32,
    sample_rate: f32,
    left_y_buffer: Box<VecDeque<f32>>,
    right_y_buffer: Box<VecDeque<f32>>,
    left_x_buffer: Box<VecDeque<f32>>,
    right_x_buffer: Box<VecDeque<f32>>,
}

impl DelayingAllPass {
    pub fn new(sample_rate: f32, delay_ms: f32, gain: f32) -> Self {
        let delay_samples: usize = ((delay_ms / 1000.0) * sample_rate).round() as usize;

        let mut left_x_buffer: Box<VecDeque<f32>> 
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        let mut right_x_buffer: Box<VecDeque<f32>>
            = Box::new(VecDeque::with_capacity(sample_rate as usize));

        let mut left_y_buffer: Box<VecDeque<f32>>
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        let mut right_y_buffer: Box<VecDeque<f32>>
            = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            left_x_buffer.push_front(0.0);
            right_x_buffer.push_front(0.0);
            left_y_buffer.push_front(0.0);
            right_y_buffer.push_front(0.0);
        }

        Self {
            sample_rate,
            delay_samples,
            delay_ms,
            left_y_buffer: left_y_buffer,
            right_y_buffer: right_y_buffer,
            left_x_buffer: left_x_buffer,
            right_x_buffer: right_x_buffer,
            gain,
        }
    }

    pub fn new_samples(&mut self, sample_rate: f32, delay_samples: f32, gain: f32) {
        self.sample_rate = sample_rate;
        self.delay_samples = delay_samples as usize;
        self.gain = gain;
        self.left_y_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_y_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.left_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            self.left_y_buffer.push_front(0.0);
            self.right_y_buffer.push_front(0.0);
            self.left_x_buffer.push_front(0.0);
            self.right_x_buffer.push_front(0.0);
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.left_y_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_y_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.left_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        self.right_x_buffer = Box::new(VecDeque::with_capacity(sample_rate as usize));
        for _ in 0..(sample_rate as usize) {
            self.left_y_buffer.push_front(0.0);
            self.right_y_buffer.push_front(0.0);
            self.left_x_buffer.push_front(0.0);
            self.right_x_buffer.push_front(0.0);
        }
    }

    pub fn set_params(&mut self, delay_ms: f32, gain: f32) {
        self.delay_samples = ((delay_ms as f32 / 1000.0) * self.sample_rate).round() as usize;
        self.delay_ms = delay_ms;
        self.gain = gain;
    }

    pub fn process_left(&mut self, x: f32) -> f32 {
        let y = -self.gain * x
        + self.left_x_buffer[self.delay_samples]
        + self.gain * self.left_y_buffer[self.delay_samples];

        self.left_x_buffer.rotate_right(1);
        self.left_x_buffer[0] = x;
        self.left_y_buffer.rotate_right(1);
        self.left_y_buffer[0] = y;
        y
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let y = -self.gain * x
        + self.right_x_buffer[self.delay_samples]
        + self.gain * self.right_y_buffer[self.delay_samples];
       
        self.right_x_buffer.rotate_right(1);
        self.right_x_buffer[0] = x;
        self.right_y_buffer.rotate_right(1);
        self.right_y_buffer[0] = y;
        y
    }
}