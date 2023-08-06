use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Debug)]
pub enum EnvelopeMode {
    Continuous,
    Follow,
}

/// creates an RMS envelope follower to follow the input audio's volume
/// so the noise being played doesn't play when no input audio comes through
pub struct EnvelopeFollower {
    sum_squared: f32,
    num_samples: usize,
}

impl EnvelopeFollower {
    pub fn new() -> Self {
        Self {
            sum_squared: 0.0,   
            num_samples: 0,
        }
    }

    pub fn process(&mut self, sample: f32, mode: EnvelopeMode) -> f32 {
        self.sum_squared = sample.powf(2.);
        self.num_samples += 1;

        let rms = match mode {
            EnvelopeMode::Continuous => 1.,
            EnvelopeMode::Follow => (self.sum_squared / (self.num_samples as f32)).sqrt(),

        };
        rms
    }
}


