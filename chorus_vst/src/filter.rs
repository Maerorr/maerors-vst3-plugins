use std::f32::consts::PI;

use nih_plug::prelude::Enum;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowPass1,
    LowPass2,
    HighPass1,
    HighPass2,
    BandPass,
    Notch,
    FirstOrderAllPass,
    SecondOrderAllPass,
    LowShelf,
    HighShelf,
    Peak,
}

impl Enum for FilterType {
    fn variants() -> &'static [&'static str] {
        &[
            "First Order Low-Pass",
            "Second Order Low-Pass",
            "First Order High-Pass",
            "Second Order High-Pass",
            "Band-Pass",
            "Notch",
            "First Order All-Pass",
            "Second Order All-Pass",
            "LowShelf",
            "HighShelf",
            "Peak",
        ]
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&[
            "lp1",
            "lp2",
            "hp1",
            "hp2",
            "bp",
            "notch",
            "ap1",
            "ap2",
            "ls",
            "hs",
            "peak",
        ])
    }

    fn to_index(self) -> usize {
        match self {
            FilterType::LowPass1 => 0,
            FilterType::LowPass2 => 1,
            FilterType::HighPass1 => 2,
            FilterType::HighPass2 => 3,
            FilterType::BandPass => 4,
            FilterType::Notch => 5,
            FilterType::FirstOrderAllPass => 6,
            FilterType::SecondOrderAllPass => 7,
            FilterType::LowShelf => 8,
            FilterType::HighShelf => 9,
            FilterType::Peak => 10,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => FilterType::LowPass1,
            1 => FilterType::LowPass2,
            2 => FilterType::HighPass1,
            3 => FilterType::HighPass2,
            4 => FilterType::BandPass,
            5 => FilterType::Notch,
            6 => FilterType::FirstOrderAllPass,
            7 => FilterType::SecondOrderAllPass,
            8 => FilterType::LowShelf,
            9 => FilterType::HighShelf,
            10 => FilterType::Peak,
            _ => panic!("Invalid filter type index."),
        }
    }

}

#[derive(Clone, Copy)]
pub struct BiquadCoefficients {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    c0: f32,
    d0: f32,
}

impl BiquadCoefficients {
    pub fn new(a0: f32, a1: f32, a2: f32, b0: f32, b1: f32, c0: f32, d0: f32) -> Self {
        Self {
            a0,
            a1,
            a2,
            b0,
            b1,
            c0,
            d0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BiquadFilter {
    coeffs: BiquadCoefficients,
    // x represents a sample from the input signal, y represents a sample from the output signal
    // where x1 is the previous sample, x2 is the sample before that, and so on.
    l_x1: f32,
    l_x2: f32,
    l_y1: f32,
    l_y2: f32,

    r_x1: f32,
    r_x2: f32,
    r_y1: f32,
    r_y2: f32,

    sample_rate: f32,
}

impl BiquadFilter {
    pub fn new() -> Self {
        let coeffs = BiquadCoefficients::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        Self {
            coeffs,
            l_x1: 0.0,
            l_x2: 0.0,
            l_y1: 0.0,
            l_y2: 0.0,
            r_x1: 0.0,
            r_x2: 0.0,
            r_y1: 0.0,
            r_y2: 0.0,
            sample_rate: 44100.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset_filter(&mut self) {
        self.l_x1 = 0.0;
        self.l_x2 = 0.0;
        self.l_y1 = 0.0;
        self.l_y2 = 0.0;
        self.r_x1 = 0.0;
        self.r_x2 = 0.0;
        self.r_y1 = 0.0;
        self.r_y2 = 0.0;
    }

    // one filtering step, taking current sample as input
    pub fn process_left(&mut self, x: f32) -> f32 {
        let y = 
            self.coeffs.a0 * x
            + self.coeffs.a1 * self.l_x1
            + self.coeffs.a2 * self.l_x2
            - self.coeffs.b0 * self.l_y1
            - self.coeffs.b1 * self.l_y2;
              
        self.l_x2 = self.l_x1;
        self.l_x1 = x;
        
        self.l_y2 = self.l_y1;
        self.l_y1 = y;

        let y = self.coeffs.c0 * y + self.coeffs.d0 * x;

        y
    }

    pub fn process_right(&mut self, x: f32) -> f32 {
        let y = 
            self.coeffs.a0 * x
            + self.coeffs.a1 * self.r_x1
            + self.coeffs.a2 * self.r_x2
            - self.coeffs.b0 * self.r_y1
            - self.coeffs.b1 * self.r_y2;
              
        self.r_x2 = self.r_x1;
        self.r_x1 = x;
        
        self.r_y2 = self.r_y1;
        self.r_y1 = y;

        let y = self.coeffs.c0 * y + self.coeffs.d0 * x;

        y
    }

    pub fn set_coefficients(&mut self, coeffs: BiquadCoefficients) {
        self.coeffs = coeffs;
    }

    pub fn get_coefficient(&self, i: usize) -> f32 {
        match i {
            0 => self.coeffs.a0,
            1 => self.coeffs.a1,
            2 => self.coeffs.a2,
            3 => self.coeffs.b0,
            4 => self.coeffs.b1,
            5 => self.coeffs.c0,
            6 => self.coeffs.d0,
            _ => panic!("Invalid coefficient index."),
        }
    }

    // for use of 1st order allpass filters in phaser
    pub fn get_s_value(&self) -> f32 {
        // sum all coeficients from a1 to b2
        self.coeffs.a1 + self.coeffs.a2 + self.coeffs.b0 + self.coeffs.b1
    }

    pub fn first_order_lpf_coefficients(&mut self, sample_rate: f32, cutoff: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let y = o.cos() / (1.0 + o.sin());
        let a0 = (1.0 - y) / 2.0;
        let a1 = (1.0 - y) / 2.0;
        let a2 = 0.0;
        let b0 = -y;
        let b1 = 0.0;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn first_order_hpf_coefficients(&mut self, sample_rate: f32, cutoff: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let y = o.cos() / (1.0 + o.sin());
        let a0 = (1.0 + y) / 2.0;
        let a1 = -((1.0 + y) / 2.0);
        let a2 = 0.0;
        let b0 = -y;
        let b1 = 0.0;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn second_order_lpf_coefficients(&mut self, sample_rate: f32, cutoff: f32, q: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let d = 1.0 / q;
        let b = 
        0.5 
        * ((1.0 - (d / 2.0) * o.sin()) 
        / (1.0 + (d / 2.0) * o.sin()));
        let y = (0.5 + b) * o.cos();
        let a0 = (0.5 + b - y) / 2.0;
        let a1 = 0.5 + b - y;
        let a2 = (0.5 + b - y) / 2.0;
        let b0 = -2.0 * y;
        let b1 = 2.0 * b;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
    }
    
    pub fn second_order_hpf_coefficients(&mut self, sample_rate: f32, cutoff: f32, q: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let d = 1.0 / q;
        let b = 
        0.5 
        * ((1.0 - (d / 2.0) * o.sin()) 
        / (1.0 + (d / 2.0) * o.sin()));
        let y = (0.5 + b) * o.cos();
        let a0 = (0.5 + b + y) / 2.0;
        let a1 = -(0.5 + b + y);
        let a2 = (0.5 + b + y) / 2.0;
        let b0 = -2.0 * y;
        let b1 = 2.0 * b;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn band_pass_coefficients(&mut self, sample_rate: f32, cutoff: f32, q: f32) {
        let k = (PI * cutoff / sample_rate).tan();
        let d = k * k * q + k + q;
        let a0 = k / d;
        let a1 = 0.0;
        let a2 = -k / d;
        let b0 = 2.0 * q * (k * k - 1.0) / d;
        let b1 = (k * k * q - k + q) / d;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
    }
    
    pub fn notch_coefficients(&mut self, sample_rate: f32, cutoff: f32, q: f32) {
        let k = (PI * cutoff / sample_rate).tan();
        let d = k * k * q + k + q;
        let a0 = (q * (k * k + 1.0)) / d;
        let a1 = (2.0 * q * (k * k - 1.0)) / d;
        let a2 = (q * (k * k + 1.0)) / d;
        let b0 = (2.0 * q * (k * k - 1.0)) / d;
        let b1 = (k * k * q - k + q) / d;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn first_order_allpass_coefficients(&mut self, sample_rate: f32, cutoff: f32) {
        let alpha = 
        ((PI * cutoff / sample_rate).tan() - 1.0) 
        / ((PI * cutoff / sample_rate).tan() + 1.0);
        let a0 = alpha;
        let a1 = 1.0;
        let a2 = 0.0;
        let b0 = alpha;
        let b1 = 0.0;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn second_order_allpass_coefficients(&mut self, sample_rate: f32, cutoff: f32, q: f32) {
        let bw = cutoff / q;
        let alpha = 
        ((PI * bw / sample_rate).tan() - 1.0) 
        / ((PI * bw / sample_rate).tan() + 1.0);
        let b = -(2.0 * PI * cutoff / sample_rate).cos();
        let a0 = -alpha;
        let a1 = b * (1.0 - alpha);
        let a2 = 1.0;
        let b0 = b * (1.0 - alpha);
        let b1 = -alpha;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0);
    }
    
    pub fn low_shelf_coefficients(&mut self, sample_rate: f32, cutoff: f32, gain: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let u = 10.0_f32.powf(gain / 20.0);
        let b = 4.0 / (1.0 + u);
        let d = b * (o / 2.0).tan();
        let y = (1.0 - d) / (1.0 + d);
    
        let a0 = (1.0 - y) / 2.0;
        let a1 = (1.0 - y) / 2.0;
        let a2 = 0.0;
        let b0 = -y;
        let b1 = 0.0;
        let c0 = u - 1.0;
        let d0 = 1.0;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, c0, d0);
    }
    
    pub fn high_shelf_coefficients(&mut self, sample_rate: f32, cutoff: f32, gain: f32) {
        let o = 2.0 * PI * cutoff / sample_rate;
        let u = 10.0_f32.powf(gain / 20.0);
        let b = (1.0 + u) / 4.0;
        let d = b * (o / 2.0).tan();
        let y = (1.0 - d) / (1.0 + d);
        let a0 = (1.0 + y) / 2.0;
        let a1 = -(1.0 + y) / 2.0;
        let a2 = 0.0;
        let b0 = -y; 
        let b1 = 0.0; 
        let c0 = u - 1.0;
        let d0 = 1.0;
        self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, c0, d0);
    }
    
    pub fn peak_coefficients(&mut self, sample_rate: f32, cutoff: f32,  q: f32, gain: f32) {
        let k = (PI * cutoff / sample_rate).tan();
        let v = 10.0_f32.powf(gain / 20.0);
        let d0 = 1.0 + (1.0 / q) * k + k*k;
        let e = 1.0 + (1.0 / (q * v)) * k + k*k;
        let alpha = 1.0 + (v/q) * k + k*k;
        let beta = 2.0 * (k*k - 1.0);
        let y = 1.0 - (v/q) * k + k*k;
        let d = 1.0 - (1.0 / q) * k + k*k;
        let p = 1.0 - (1.0/(q*v)) * k + k*k;
    
        if gain >= 0.0 {
            let a0 = alpha / d0;
            let a1 = beta / d0;
            let a2 = y / d0;
            let b0 = beta / d0;
            let b1 = d / d0;
            self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
        } else {
            let a0 = d0 / e;
            let a1 = beta / e;
            let a2 = d / e;
            let b0 = beta / e;
            let b1 = p / e;
            self.coeffs = BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
        }
    }

    pub fn coefficients(&mut self, filter_type: FilterType, cutoff: f32, q: f32, gain: f32) {
        match filter_type {
            FilterType::LowPass1 => {
                self.first_order_lpf_coefficients(self.sample_rate, cutoff);
            },
            FilterType::LowPass2 => {
                self.second_order_lpf_coefficients(self.sample_rate, cutoff, q);
            },
            FilterType::HighPass1 => {
                self.first_order_hpf_coefficients(self.sample_rate, cutoff);
            },
            FilterType::HighPass2 => {
                self.second_order_hpf_coefficients(self.sample_rate, cutoff, q);
            },
            FilterType::BandPass => {
                self.band_pass_coefficients(self.sample_rate, cutoff, q);
            },
            FilterType::Notch => {
                self.notch_coefficients(self.sample_rate, cutoff, q);
            },
            FilterType::FirstOrderAllPass => {
                self.first_order_allpass_coefficients(self.sample_rate, cutoff);
            },
            FilterType::SecondOrderAllPass => {
                self.second_order_allpass_coefficients(self.sample_rate, cutoff, q);
            },
            FilterType::LowShelf => {
                self.low_shelf_coefficients(self.sample_rate, cutoff, gain);
            },
            FilterType::HighShelf => {
                self.high_shelf_coefficients(self.sample_rate, cutoff, gain);
            }
            FilterType::Peak => {
                self.peak_coefficients(self.sample_rate, cutoff, q, gain);
            }
        }
    }
}