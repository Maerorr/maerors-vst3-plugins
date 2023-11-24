
pub struct MidSideMixer {
    mid: f32,
    l_side: f32,
    r_side: f32,

    mid_mix: f32,
    side_mix: f32,
}

impl MidSideMixer {
    pub fn new() -> Self {
        Self {
            mid: 0.0,
            l_side: 0.0,
            r_side: 0.0,

            mid_mix: 0.0,
            side_mix: 0.0,
        }
    }

    pub fn set_params(&mut self, mid_mix: f32, side_mix: f32) {
        self.mid_mix = mid_mix;
        self.side_mix = side_mix;
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        self.mid = (left + right) * 0.5;
        self.l_side = left - right;
        self.r_side = right - left;    

        (self.mid * self.mid_mix + self.l_side * self.side_mix, self.mid * self.mid_mix + self.r_side * self.side_mix)
    }
}