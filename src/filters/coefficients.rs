use std::f32::consts::{TAU, FRAC_1_SQRT_2};

pub enum FilterType {
    Lowpass,
}

pub struct FilterCoefficients {
    // feedforward coefficients
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    // feedback coefficients
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Default for FilterCoefficients {
    fn default() -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a0: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

impl FilterCoefficients {
    fn lowpass(&mut self, fc: f32, sample_rate: f32) {
        let omega_c = TAU * (fc / sample_rate);
        let cos_omega_c = omega_c.cos();
        let alpha = omega_c.sin() / (2.0 * FRAC_1_SQRT_2);

        self.a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_omega_c) / 2.0) / self.a0;
        self.b1 = (1.0 - cos_omega_c) / self.a0;
        self.b2 = ((1.0 - cos_omega_c) / 2.0) / self.a0;
        self.a1 = (-2.0 * cos_omega_c) / self.a0;
        self.a2 = (1.0 - alpha) / self.a0;
    }

    pub fn update(&mut self, fc: f32, sample_rate: f32, filter_type: FilterType) {
        match filter_type {
            FilterType::Lowpass => self.lowpass(fc, sample_rate),
        };

    }
}
