use std::f32::consts::{TAU, FRAC_1_SQRT_2};

pub enum FilterType {
    Lowpass,
    Highpass,
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

impl From<(FilterType, f32, f32)> for FilterCoefficients {
    fn from(params: (FilterType, f32, f32)) -> Self {

        let (filter_type, fc, sample_rate) = params;
        let f = match filter_type {
            FilterType::Lowpass => FilterCoefficients::lowpass(fc, sample_rate),
            FilterType::Highpass => todo!(),
        };
        f
    }
}

impl FilterCoefficients {
    fn lowpass(fc: f32, sample_rate: f32) -> Self {
        let omega_c = TAU * (fc / sample_rate);
        let cos_omega_c = omega_c.cos();
        let alpha = omega_c.sin() / (2.0 * FRAC_1_SQRT_2);

        let a0 = 1.0 + alpha;
        let b0 = ((1.0 - cos_omega_c) / 2.0) / a0;
        let b1 = (1.0 - cos_omega_c) / a0;
        let b2 = ((1.0 - cos_omega_c) / 2.0) / a0;
        let a1 = (-2.0 * cos_omega_c) / a0;
        let a2 = (1.0 - alpha) / a0;

        Self {
            b0,
            b1,
            b2,
            a0,
            a1,
            a2,
        }
    }
}
