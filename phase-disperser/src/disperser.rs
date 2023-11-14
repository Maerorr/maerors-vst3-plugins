use crate::filter::BiquadFilter;

pub struct Disperser {
    allpasses: Vec<BiquadFilter>,
    output_hpf: BiquadFilter,
    sample_rate: f32,
    amount: usize
}

impl Disperser {
    pub fn new() -> Self {
        Self {
            allpasses: vec![],
            output_hpf: BiquadFilter::new(),
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
        self.output_hpf.set_sample_rate(sample_rate);
        self.output_hpf.second_order_hpf_coefficients(sample_rate, 30.0, 0.707)
    }

    pub fn set_params(&mut self, frequency: f32, q: f32, spread: f32, amount: u32) {
        //for (i, bq) in self.allpasses.iter_mut().enumerate() {
        for i in 0..(self.amount as usize) {
            //bq.second_order_allpass_coefficients(self.sample_rate, frequency, q);
            // based on spread parameter, spread the frequency values around the frequency value
            let mut freq = frequency;
            if spread > 0.0 {
                let spread_amount = spread;
                let spread_range = frequency * spread_amount;
                let spread_offset = spread_range * 0.5;
                let mut spread_min = frequency - spread_offset;
                if spread_min < 300.0 {
                    spread_min = 300.0;
                }
                let mut spread_max = frequency + spread_offset;
                if spread_max > 15000.0 {
                    spread_max = 15000.0;
                }
                freq = spread_min + (spread_max - spread_min) * (((i as i32 - self.amount as i32 / 2) as f32) / (self.amount as f32 / 2.0));
                if freq > 15000.0 {
                    freq = 15000.0;
                }
                if freq < 400.0 {
                    freq = 400.0;
                }
                self.allpasses[i].second_order_allpass_coefficients(self.sample_rate, freq, q);
            } else {
                self.allpasses[i].second_order_allpass_coefficients(self.sample_rate, freq, q);
            }
        }
        if amount as usize != self.amount {
            // reset all filters above the amount index
            for i in (amount as usize)..self.amount {
                self.allpasses[i as usize].reset_filter();
            }
        }
        self.amount = amount as usize;
    }

    pub fn process_left(&mut self, input: f32) -> f32 {
        let mut output = input;
        for i in 0..(self.amount) {
            output = self.allpasses[i].process_left(output);
        }
        output = self.output_hpf.process_left(output);
        output
    }

    pub fn process_right(&mut self, input: f32) -> f32 {
        let mut output = input;
        for i in 0..(self.amount) {
            output = self.allpasses[i].process_right(output);
        }
        output = self.output_hpf.process_right(output);
        output
    }
}