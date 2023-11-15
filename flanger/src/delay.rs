use std::collections::VecDeque;

const MAX_DELAY: usize = 1; // 3 seconds at 44100Hz

#[derive(Clone)]
pub struct Delay {
    x_buffer: Vec<f32>,
    y_buffer: Vec<f32>,
    buffer_size: usize,
    write_index: usize,
    sample_rate: f32,

    pub delay: f32, // Changed the delay to f32 for interpolated delay

    // function variables to avoid alocation
    read_index: f32,
    read_index_floor: usize,
    read_index_frac: f32,
    output: f32,
}

impl Delay {
    pub fn new(sample_rate: usize, delay: f32) -> Self {
        let buffer_size = MAX_DELAY * sample_rate;
        let x_buffer = vec![0.0; buffer_size];
        let y_buffer = vec![0.0; buffer_size];
        let write_index = 0;

        Self {
            x_buffer,
            y_buffer,
            buffer_size,
            write_index,
            sample_rate: sample_rate as f32,
            delay,
            read_index: 0.0,
            read_index_floor: 0,
            read_index_frac: 0.0,
            output: 0.0,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: usize) {
        let buffer_size = MAX_DELAY * sample_rate;
        self.x_buffer.resize(buffer_size, 0.0);
        self.y_buffer.resize(buffer_size, 0.0);
        self.buffer_size = buffer_size;
    }

    // y(n) = x(n - delay) + fb * y(n - delay)
    pub fn process_sample(&mut self, x: f32, delay_samples_f32: f32) -> f32 {
        self.x_buffer[self.write_index] = x;

        self.read_index = ((self.write_index + self.buffer_size) as f32 - delay_samples_f32) % self.buffer_size as f32;
        self.read_index_floor = self.read_index.floor() as usize;
        self.read_index_frac = self.read_index.fract();

        self.output = self.x_buffer[self.read_index_floor] * (1.0 - self.read_index_frac)
            + self.x_buffer[(self.read_index_floor + 1) % self.buffer_size] * self.read_index_frac;

        self.y_buffer[self.write_index] = self.output;

        self.write_index = (self.write_index + 1) % self.buffer_size;

        self.output
    }

    fn calculate_read_index(&self, delay: f32) -> f32 {
        let delay_samples = delay;
        let read_index = self.write_index as f32 - delay_samples;
        if read_index < 0.0 {
            read_index + self.buffer_size as f32
        } else if read_index > self.buffer_size as f32 {
            read_index - self.buffer_size as f32
        } else {
            read_index
        }
    }
}