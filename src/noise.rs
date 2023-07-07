use atomic_float::AtomicF32;
use std::{
    mem,
    sync::{Arc, Mutex},
};

use crate::{config, params::WhiteNoiseDistribution};
use crate::gui;
use crate::params::NoiseParams;
use crate::spectrum::Spectrum;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal, Uniform};

// fn get_norm_dist_white_noise(rng: &mut StdRng) -> f32 {
//     let normal_dist = Normal::<f32>::new(0.0, 1.0).unwrap();
//     let random_sample = normal_dist.sample(rng);
//     return random_sample.clamp(-1.0, 1.0);
// }

// fn get_uniform_dist_white_noise(rng: &mut StdRng) -> f32 {
//     let uniform_dist = Uniform::<f32>::new(-1.0, 1.0);
//     let random_sample = uniform_dist.sample(rng);
//     return random_sample.clamp(-1.0, 1.0);
// }

pub struct Noise {
    pub params: Arc<NoiseParams>,
    pub sample: f32,
    pub rng: StdRng,
    pub white: White,
    pub pink: Pink,
    pub brown: Brown,
    pub violet: Violet,
    pub debug: config::Debug,
    pub sample_rate: Arc<AtomicF32>,
    pub spectrum: Spectrum,
    pub spectrum_output_buffer: gui::analyzer::SpectrumBuffer,
}

impl Default for Noise {
    fn default() -> Self {
        let (spectrum, spectrum_out) = Spectrum::new(44.1e3, 2, 2048);
        let spectrum_output_buffer = Arc::new(Mutex::new(spectrum_out));
        let sample_rate = Arc::new(AtomicF32::new(44.1e3));

        Self {
            params: Arc::new(NoiseParams::default()),
            sample: 0.0,
            rng: StdRng::from_entropy(),
            white: White::new(),
            pink: Pink::new(),
            brown: Brown::new(0.99),
            violet: Violet::new(),
            debug: config::Debug::default(),
            sample_rate,
            spectrum,
            spectrum_output_buffer,
        }
    }
}

pub trait NoiseConfig {
    fn reset(&mut self);
    fn next(&mut self, white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32;
    fn white(&mut self, white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32 {
        let random_sample: f32 = match white_noise_type {
            WhiteNoiseDistribution::Normal => {
                let dist = Normal::<f32>::new(0.0, 1.0).unwrap();
                return dist.sample(rng).clamp(-1.0, 1.0);
            }
            WhiteNoiseDistribution::Uniform => {
                let dist = Uniform::<f32>::new(0.0, 1.0);
                return dist.sample(rng).clamp(-1.0, 1.0);
            }
            _ => 0.0,
        };
        return random_sample;
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

    fn next(&mut self, white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32 {
        return self.white(white_noise_type, rng) * 0.1;
    }
}

#[derive(Debug, Clone)]
pub struct Pink {
    rows: [i64; Pink::MAX_RANDOM_ROWS],
    running_sum: i64,
    index: i32,
}

impl Pink {
    const MAX_RANDOM_ROWS: usize = 30;
    const RANDOM_BITS: usize = 24;
    const RANDOM_SHIFT: usize = std::mem::size_of::<i64>() * 8 - Pink::RANDOM_BITS;
    const INDEX_MASK: i32 = (1 << (Pink::MAX_RANDOM_ROWS - 1)) - 1;
    const SCALAR: f32 =
        1.0 / ((Pink::MAX_RANDOM_ROWS + 1) * (1 << (Pink::RANDOM_BITS - 1)) as usize) as f32;

    pub fn new() -> Self {
        Pink {
            rows: [0; Pink::MAX_RANDOM_ROWS],
            running_sum: 0,
            index: 0,
        }
    }
}

impl NoiseConfig for Pink {
    fn reset(&mut self) {
        mem::replace(self, Pink::new());
    }

    fn next(&mut self, _white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32 {
        // ported from here: https://github.com/PortAudio/portaudio/blob/master/examples/paex_pink.c
        let mut new_random: i64;

        self.index = (self.index + 1) & Pink::INDEX_MASK;
        if self.index != 0 {
            let mut num_zeroes = 0;
            let mut n = self.index;

            while (n & 1) == 0 {
                n = n >> 1;
                num_zeroes += 1;
            }

            self.running_sum -= self.rows[num_zeroes];
            new_random = rng.gen::<i64>() >> Pink::RANDOM_SHIFT;
            self.running_sum += new_random;
            self.rows[num_zeroes] = new_random;
        }

        new_random = rng.gen::<i64>() >> Pink::RANDOM_SHIFT;
        return (self.running_sum + new_random) as f32 * Pink::SCALAR;
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
        mem::replace(self, Brown::new(0.99));
    }

    fn next(&mut self, white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32 {
        // let white = get_uniform_dist_white_noise(rng);
        let white = self.white(white_noise_type, rng);
        self.current_sample =
            ((self.leak * self.current_sample) + (1.0 - self.leak) * white).clamp(-1.0, 1.0);
        return self.current_sample;
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
        mem::replace(self, Violet::new());
    }

    fn next(&mut self, white_noise_type: &WhiteNoiseDistribution, rng: &mut StdRng) -> f32 {
        // let white = get_norm_dist_white_noise(rng) * 0.1;
        let white = self.white(white_noise_type, rng) * 0.1;
        let violet = white - self.previous_sample;
        self.previous_sample = white;
        return violet;
    }
}
