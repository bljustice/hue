use atomic_float::AtomicF32;
use std::{
    mem,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use crate::filters::biquad::Biquad;
use crate::gui;
use crate::params::NoiseParams;
use crate::spectrum::Spectrum;
use crate::config;
use crate::{
    envelope::follower::{EnvelopeFollower, EnvelopeMode},
    params::NoiseType,
};
use rand::{rngs::StdRng, SeedableRng};
use rand_distr::{Distribution, Uniform};

pub struct Noise {
    pub params: Arc<NoiseParams>,
    pub rng: StdRng,
    pub white: White,
    pub pink: Pink,
    pub brown: Brown,
    pub violet: Violet,
    pub debug: config::Debug,
    pub sample_rate: Arc<AtomicF32>,
    pub spectrum: Spectrum,
    pub spectrum_output_buffer: gui::analyzer::SpectrumBuffer,
    pub lpf: Biquad,
    pub hpf: Biquad,
    pub should_update_filter: Arc<AtomicBool>,
    pub envelope_follower: EnvelopeFollower,
}

impl Default for Noise {
    fn default() -> Self {
        let (spectrum, spectrum_out) = Spectrum::new(44.1e3, 2, 2048);
        let spectrum_output_buffer = Arc::new(Mutex::new(spectrum_out));
        let sample_rate = Arc::new(AtomicF32::new(44.1e3));

        let should_update_filter = Arc::new(AtomicBool::new(true));
        let env = EnvelopeFollower::new(&sample_rate.load(std::sync::atomic::Ordering::Relaxed));

        Self {
            params: Arc::new(NoiseParams::new(should_update_filter)),
            rng: StdRng::from_entropy(),
            white: White::new(),
            pink: Pink::new(),
            brown: Brown::new(0.99),
            violet: Violet::new(),
            debug: config::Debug::default(),
            sample_rate,
            spectrum,
            spectrum_output_buffer,
            lpf: Default::default(),
            hpf: Default::default(),
            should_update_filter: Arc::new(AtomicBool::new(false)),
            envelope_follower: env,
        }
    }
}

impl Noise {
    fn next(&mut self) -> f32 {
        let noise_sample = match self.params.noise_type.value() {
            NoiseType::White => self
                .white
                .next(&mut self.rng),
            NoiseType::Pink => self
                .pink
                .next(&mut self.rng),
            NoiseType::Brown => self
                .brown
                .next(&mut self.rng),
            NoiseType::Violet => self
                .violet
                .next(&mut self.rng),
        };
        noise_sample
    }

    fn filter_noise(&mut self, sample: f32) -> f32 {
        let lowpassed_noise = self.lpf.process(sample);
        self.hpf.process(lowpassed_noise)
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let noise_sample = self.next();
        let filtered_noise = self.filter_noise(noise_sample);
        let mix_level = self.params.mix.value();
        let gain = self.params.gain.value();

        let final_sample = match self.params.env_mode.value() {
            EnvelopeMode::Continuous => {
                ((filtered_noise * gain) * mix_level) + (sample * (1. - mix_level))
            }
            EnvelopeMode::Follow => {
                let envelope = self.envelope_follower.process(sample);
                let noise_w_envelope = envelope * filtered_noise;
                ((mix_level * noise_w_envelope) * gain) + (sample * (1. - mix_level))
            }
        };
        final_sample
    }
}

pub trait NoiseConfig {
    fn reset(&mut self);
    fn next(&mut self, rng: &mut StdRng) -> f32;
    fn white(&mut self, rng: &mut StdRng) -> f32 {
        let dist = Uniform::<f32>::new(-1.0, 1.0);
        dist.sample(rng).clamp(-1.0, 1.0)
    }
}

pub struct White;

impl White {
    pub fn new() -> Self {
        Self {}
    }
}

impl NoiseConfig for White {
    fn reset(&mut self) {}

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        return self.white(rng);
    }
}

#[derive(Debug, Clone)]
pub struct Pink {
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
}

impl Pink {
    pub fn new() -> Self {
        Pink {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
        }
    }
}

impl NoiseConfig for Pink {
    fn reset(&mut self) {
        let _ = mem::replace(self, Pink::new());
    }

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = self.white(rng);
        self.b0 = 0.99886 * self.b0 + white * 0.0555179;
        self.b1 = 0.99332 * self.b1 + white * 0.0750759;
        self.b2 = 0.96900 * self.b2 + white * 0.1538520;
        self.b3 = 0.86650 * self.b3 + white * 0.3104856;
        self.b4 = 0.55000 * self.b4 + white * 0.5329522;
        self.b5 = -0.7616 * self.b5 - white * 0.0168980;

        let out =
            (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362)
                * 0.11;

        self.b6 = white * 0.115926;
        out
    }
}

pub struct Brown {
    current_sample: f32,
    leak: f32,
}

impl Brown {
    fn new(leak: f32) -> Self {
        Self {
            current_sample: 0.0,
            leak,
        }
    }
}

impl NoiseConfig for Brown {
    fn reset(&mut self) {
        let _ = mem::replace(self, Brown::new(0.99));
    }

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = self.white(rng);
        self.current_sample = (self.leak * self.current_sample) + (1.0 - self.leak) * white;
        self.current_sample
    }
}

pub struct Violet {
    previous_sample: f32,
}

impl Violet {
    fn new() -> Self {
        Self {
            previous_sample: 0.0,
        }
    }
}

impl NoiseConfig for Violet {
    fn reset(&mut self) {
        let _ = mem::replace(self, Violet::new());
    }

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = self.white(rng) * 0.1;
        let violet = white - self.previous_sample;
        self.previous_sample = white;
        return violet;
    }
}
