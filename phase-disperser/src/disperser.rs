use crate::filter::BiquadFilter;

pub struct Disperser {
    allpasses: Vec<BiquadFilter>,
    sample_rate: f32,
    amount: usize
}

impl Disperser {
    pub fn new() -> Self {
        Self {
            allpasses: vec![],
            sample_rate: 44100.0,
            amount: 200
        }
    }

    pub fn resize_buffers(&mut self, sample_rate: f32, frequency: f32, q: f32, amount: u32) {
        self.allpasses = Vec::with_capacity(amount as usize);
        self.sample_rate = sample_rate;
        self.amount = amount as usize;
        for _ in 0..200 {
            let mut bq = BiquadFilter::new();
            bq.set_sample_rate(sample_rate);
            bq.second_order_allpass_coefficients(sample_rate, frequency, q);
            self.allpasses.push(bq);
        }
    }

    pub fn set_params(&mut self, frequency: f32, q: f32, amount: u32) {
        for bq in self.allpasses.iter_mut() {
            bq.second_order_allpass_coefficients(self.sample_rate, frequency, q);
        }
        // if amount as usize != self.amount {
        //     // reset all filters above the amount index
        //     for i in (amount as usize)..self.amount {
        //         self.allpasses[i as usize].reset_filter();
        //     }
        // }
        self.amount = amount as usize;
    }

    pub fn process_left(&mut self, input: f32) -> f32 {
        let mut output = input;
        for i in 0..200 {
            output = self.allpasses[i].process_left(output);
        }
        output
    }

    pub fn process_right(&mut self, input: f32) -> f32 {
        let mut output = input;
        for i in 0..200 {
            output = self.allpasses[i].process_right(output);
        }
        output
    }
}