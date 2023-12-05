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
    delay_samples_f32: f32,
    feedback: f32,
    depth: f32,
    sample_rate: f32,
    calc_depth: f32,
    mono: bool,
    wet: f32,
    dry: f32,

    // struct variables not to allocate in process()
    lx: f32,
    llfo1: f32,
    llfo2: f32,
    llfo3: f32,
    loffset1: f32,
    loffset2: f32,
    loffset3: f32,
    ldelayed_signal: f32,
    left_out: f32,

    rx: f32,
    rlfo1: f32,
    rlfo2: f32,
    rlfo3: f32,
    roffset1: f32,
    roffset2: f32,
    roffset3: f32,
    rdelayed_signal: f32,
    right_out: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32, delay_ms: f32, feedback: f32, depth: f32, rate: f32, wet: f32, dry: f32) -> Self {
        let mut left_delays: Vec<Delay> = Vec::with_capacity(5);
        let mut right_delays: Vec<Delay> = Vec::with_capacity(5);
        let mut left_lfos: Vec<lfo::LFO> = Vec::with_capacity(5);
        let mut right_lfos: Vec<lfo::LFO> = Vec::with_capacity(5);

        let delay_samples_f32: f32 = (delay_ms as f32 / 1000.0) * sample_rate as f32;

        for i in 0..5 {
            left_delays.push(Delay::new(sample_rate as usize, delay_samples_f32, 0.0));
            right_delays.push(Delay::new(sample_rate as usize, delay_samples_f32, 0.0));
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
            mono: false,
            delay_ms,
            delay_samples_f32,
            lx: 0.0,
            llfo1: 0.0,
            llfo2: 0.0,
            llfo3: 0.0,
            loffset1: 0.0,
            loffset2: 0.0,
            loffset3: 0.0,
            ldelayed_signal: 0.0,
            left_out: 0.0,
            rx: 0.0,
            rlfo1: 0.0,
            rlfo2: 0.0,
            rlfo3: 0.0,
            roffset1: 0.0,
            roffset2: 0.0,
            roffset3: 0.0,
            rdelayed_signal: 0.0,
            right_out: 0.0,
        }
    }

    pub fn set_params(&mut self, sample_rate: f32, delay: f32, feedback: f32, depth: f32, rate: f32, mix: f32, mono: bool) {
        // resize all buffers relying on sample rate
        self.sample_rate = sample_rate;
    
        for (lfol, lfor) in self.left_lfos.iter_mut().zip(self.right_lfos.iter_mut()) {
            lfol.sample_rate = sample_rate;
            lfor.sample_rate = sample_rate;
        }

        let delay_samples_f32: f32 = (delay as f32 / 1000.0) * self.sample_rate as f32;

        for d in self.left_delays.iter_mut() {
            d.delay = delay_samples_f32;
        }
        for d in self.right_delays.iter_mut() {
            d.delay = delay_samples_f32;
        }

        self.feedback = feedback;

        self.depth = depth;
        self.calc_depth = depth / 1000.0 * self.sample_rate;

        for (lfol, lfor) in self.left_lfos.iter_mut().zip(self.right_lfos.iter_mut()) {
            lfol.rate = rate;
            lfor.rate = rate;
        }

        self.wet = mix;
        self.dry = 1.0 - mix;
        self.mono = mono;
        self.delay_ms = delay;
        self.delay_samples_f32 = delay_samples_f32;
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
        self.lx = x + self.wet * self.feedback * self.left_feedback_buffer.get(0).unwrap();

        self.llfo1 = self.left_lfos[0].next_value();
        self.llfo2 = self.left_lfos[1].next_value();
        self.llfo3 = self.left_lfos[2].next_value();

        self.loffset1 = (self.llfo1 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);
        self.loffset2 = (self.llfo2 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);
        self.loffset3 = (self.llfo3 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);

        self.ldelayed_signal = 0.0;
        self.ldelayed_signal += self.left_delays[0].process_sample(self.lx, self.delay_samples_f32 + self.loffset1);
        self.ldelayed_signal += self.left_delays[1].process_sample(self.lx, self.delay_samples_f32 + self.loffset2);
        self.ldelayed_signal += self.left_delays[2].process_sample(self.lx, self.delay_samples_f32 + self.loffset3);

        //self.left_feedback_buffer.rotate_right(1);
        self.left_feedback_buffer[0] = self.ldelayed_signal / 3.0;

        self.left_out = 
        self.dry * x 
        + self.wet * 1.0/3.0 * self.ldelayed_signal;

        if self.wet + self.dry > 1.0 {
            self.left_out /= self.wet + self.dry;
        }

        self.left_out
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        self.rx = x + self.wet * self.feedback * self.right_feedback_buffer.get(0).unwrap();

        // mono, meaning mono modulation
        if self.mono {
            self.rlfo1 = self.left_lfos[0].next_value();
            self.rlfo2 = self.left_lfos[1].next_value();
            self.rlfo3 = self.left_lfos[2].next_value();
        } else {
            self.rlfo1 = self.right_lfos[0].next_value();
            self.rlfo2 = self.right_lfos[1].next_value();
            self.rlfo3 = self.right_lfos[2].next_value();
        }
        

        self.roffset1 = (self.rlfo1 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);
        self.roffset2 = (self.rlfo2 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);
        self.roffset3 = (self.rlfo3 * self.calc_depth / 2.0).clamp(-(self.delay_samples_f32) + 1.0 , self.delay_samples_f32 - 1.0);

        self.rdelayed_signal = 0.0;
        self.rdelayed_signal += self.right_delays[0].process_sample(self.rx, self.delay_samples_f32 + self.roffset1);
        self.rdelayed_signal += self.right_delays[1].process_sample(self.rx, self.delay_samples_f32 + self.roffset2);
        self.rdelayed_signal += self.right_delays[2].process_sample(self.rx, self.delay_samples_f32 + self.roffset3);

        //self.right_feedback_buffer.rotate_right(1);
        self.right_feedback_buffer[0] = self.rdelayed_signal / 3.0;

        self.right_out = self.dry * x 
        + self.wet * 1.0/3.0 * self.rdelayed_signal;

        if self.wet + self.dry > 1.0 {
            self.right_out /= self.wet + self.dry;
        }

        self.right_out
    }

    pub fn update_modulators(&mut self) {
        for lfo in self.left_lfos.iter_mut() {
            lfo.update_lfo();
        }
        for lfo in self.right_lfos.iter_mut() {
            lfo.update_lfo();
        }
    }
}