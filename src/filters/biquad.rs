use crate::filters::coefficients::FilterCoefficients;

/// Represents a transposed direct form II biquad filter
/// https://en.wikipedia.org/wiki/Digital_biquad_filter
#[derive(Default)]
pub struct Biquad {
    s1: f32,
    s2: f32,

    pub coefficients: FilterCoefficients,
}

impl Biquad {
    pub fn process(&mut self, sample: f32) -> f32 {
        let out = self.coefficients.b0 * sample + self.s1;
        self.s1 = self.s2 + self.coefficients.b1 * sample - self.coefficients.a1 * out;
        self.s2 = self.coefficients.b2 * sample - self.coefficients.a2 * out;
        out
    }
}
