use std::collections::VecDeque;

const MAX_DELAY: usize = 3; // 3 seconds at 44100Hz

#[derive(Clone)]
pub struct Delay {
    x_buffer: Box<VecDeque<f32>>,
    y_buffer: Box<VecDeque<f32>>,
    
    pub delay: usize,
    pub feedback: f32,
}

impl Delay {
    pub fn new(sample_rate: usize, delay: usize, feedback: f32) -> Self {

        let mut xbuf: Box<VecDeque<f32>> = Box::new(VecDeque::with_capacity(MAX_DELAY * sample_rate));
        let mut ybuf: Box<VecDeque<f32>> = Box::new(VecDeque::with_capacity(MAX_DELAY * sample_rate));
        // fill with zeroes
        for _ in 0..MAX_DELAY * sample_rate {
            xbuf.push_front(0.0);
            ybuf.push_front(0.0);
        }

        let feedback = if feedback > 1.0 {
            1.0
        } else if feedback < 0.0 {
            0.0
        } else {
            feedback
        };

        Self {
            x_buffer: xbuf,
            y_buffer: ybuf,
            delay,
            feedback: feedback,
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: usize) {
        self.x_buffer = Box::new(VecDeque::with_capacity(MAX_DELAY * sample_rate));
        self.y_buffer = Box::new(VecDeque::with_capacity(MAX_DELAY * sample_rate));
        for _ in 0..MAX_DELAY * sample_rate {
            self.x_buffer.push_front(0.0);
            self.y_buffer.push_front(0.0);
        }
    }

    // y(n) = x(n - delay) + fb * y(n - delay)
    pub fn process_sample(&mut self, x: f32, delay: usize) -> f32 {
        self.x_buffer.rotate_right(1);
        self.x_buffer[0] = x;

        let y = 
        self.x_buffer.get(delay).unwrap()
        + self.feedback * self.y_buffer.get(delay).unwrap();

        self.y_buffer.rotate_right(1);
        self.y_buffer[0] = y;

        y
    }
}