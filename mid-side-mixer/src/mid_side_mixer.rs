use std::f32::consts::PI;


pub struct MidSideMixer {
    mid: f32,
    l_side: f32,
    r_side: f32,

    left: f32,
    right: f32,

    gain_left: f32,
    gain_right: f32,

    mid_mix: f32,
    side_mix: f32,
    left_right_mix: f32,

    is_mid_side: bool,
}

impl MidSideMixer {
    pub fn new() -> Self {
        Self {
            mid: 0.0,
            l_side: 0.0,
            r_side: 0.0,

            left: 0.0,
            right: 0.0,

            gain_left: 0.0,
            gain_right: 0.0,

            mid_mix: 0.0,
            side_mix: 0.0,

            left_right_mix: 0.0,

            is_mid_side: true,
        }
    }

    pub fn set_params(&mut self, mid_mix: f32, side_mix: f32, left_right_mix: f32, is_mid_side: bool) {
        self.mid_mix = mid_mix;
        self.side_mix = side_mix;
        self.left_right_mix = left_right_mix;
        self.is_mid_side = is_mid_side;
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.is_mid_side {
            self.mid = (left + right) * 0.5;
            self.l_side = (left - right) * 0.5;
            self.r_side = (right - left) * 0.5;   
    
            return (self.mid * self.mid_mix + self.l_side * self.side_mix, self.mid * self.mid_mix + self.r_side * self.side_mix)
        } else {
            let x = self.left_right_mix / 2.0 + 0.5;
            let x = if x < 0.5 {
                (x * PI).sin() / 2.0
            } else {
                (x * PI + PI/2.0).cos() / 2.0 + 1.0
            };
            self.gain_left = x;
            self.gain_right = 1.0 - x;
            self.left = right * (-x + 0.5).clamp(0.0, 0.5) + if x < 0.5 { left * (x + 0.5)} else { left * (-(2.0 * x - 2.0)) };
            self.right = left * (x - 0.5).clamp(0.0, 0.5) + if x < 0.5 { right * (2.0 * x)} else { right * (-x + 1.5) };
            return (self.left, self.right)
        }
    }
}