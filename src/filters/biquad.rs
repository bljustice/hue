use crate::filters::coefficients::FilterCoefficients;

/// Represents a transposed direct form II biquad filter
/// https://en.wikipedia.org/wiki/Digital_biquad_filter
pub struct Biquad {
    // feedforward coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    // feedback coefficients
    a0: f32,
    a1: f32,
    a2: f32,
    s1: f32,
    s2: f32,
}

impl Biquad {

    pub fn new() -> Self {
        Self {
            b0: 0.,
            b1: 0.,
            b2: 0.,
            a0: 0.,
            a1: 0.,
            a2: 0.,
            s1: 0.,
            s2: 0.,
        }
    }

    pub fn process(&mut self, sample: f32, coefficients: FilterCoefficients) -> f32 {
        let out = coefficients.b0 * sample + self.s1;
        self.s1 = self.s2 + coefficients.b1 * sample - coefficients.a1 * out;
        self.s2 = coefficients.b2 * sample - coefficients.a2 * out;
        out
    }
}
