use std::time::Duration;
use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Debug)]
pub enum EnvelopeMode {
    Continuous,
    Follow,
}

pub struct EnvelopeFollower {
    envelope_value: f32,
    attack_coefficient: f32,
    release_coefficient: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: &f32) -> Self {
        Self {
            envelope_value: 0.,
            attack_coefficient: Self::calculate_coefficient(&sample_rate, Duration::from_millis(10)),
            release_coefficient: Self::calculate_coefficient(&sample_rate, Duration::from_millis(100)),
        }
    }

    fn calculate_coefficient(sample_rate: &f32, time: Duration) -> f32 {
        (-1.0 / (sample_rate * time.as_secs_f32())).exp()
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let sample_abs = sample.abs();
        let env_coefficient = match self.envelope_value < sample_abs {
            true => self.attack_coefficient,
            false => self.release_coefficient,
        };
        self.envelope_value = (self.envelope_value * env_coefficient) + sample_abs * (1.0 - env_coefficient);
        self.envelope_value
    }
}


