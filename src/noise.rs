use atomic_float::AtomicF32;
use nih_plug::prelude::{
    formatters, util, Enum, EnumParam, FloatParam, FloatRange, Params, SmoothingStyle,
};
use nih_plug_vizia::ViziaState;
use std::{
    mem,
    sync::{Arc, Mutex},
};

use crate::config;
use crate::editor;
use crate::spectrum::Analyzer;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal, Uniform};

fn get_norm_dist_white_noise(rng: &mut StdRng) -> f32 {
    let normal_dist = Normal::new(0.0, 1.0).unwrap();
    let random_sample = normal_dist.sample(rng) as f32;
    return random_sample.clamp(-1.0, 1.0);
}

fn get_uniform_dist_white_noise(rng: &mut StdRng) -> f32 {
    let uniform_dist = Uniform::new(-1.0, 1.0);
    let random_sample = uniform_dist.sample(rng) as f32;
    return random_sample.clamp(-1.0, 1.0);
}

pub struct Noise {
    pub params: Arc<NoiseParams>,
    pub rng: StdRng,
    pub white: White,
    pub pink: Pink,
    pub brown: Brown,
    pub violet: Violet,
    pub debug: config::Debug,
    pub sample_rate: Arc<AtomicF32>,
    pub analyzer_in: Analyzer,
    pub analyzer_output: editor::SpectrumUI,
}

impl Default for Noise {
    fn default() -> Self {
        let (analyzer_in, spectrum_out) = Analyzer::new(44.1e3, 2, 2048);
        let analyzer_output = Arc::new(Mutex::new(spectrum_out));
        let sample_rate = Arc::new(AtomicF32::new(44.1e3));

        Self {
            params: Arc::new(NoiseParams::default()),
            rng: StdRng::from_entropy(),
            white: White::new(),
            pink: Pink::new(),
            brown: Brown::new(0.99),
            violet: Violet::new(),
            debug: config::Debug::default(),
            sample_rate: sample_rate,
            analyzer_in: analyzer_in,
            analyzer_output: analyzer_output,
        }
    }
}

pub trait NoiseConfig {
    fn reset(&mut self);
    fn next(&mut self, rng: &mut StdRng) -> f32;
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
        let white_noise_sample = get_norm_dist_white_noise(rng) * 0.1;
        return white_noise_sample;
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

    fn next(&mut self, rng: &mut StdRng) -> f32 {
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

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = get_uniform_dist_white_noise(rng);
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

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = get_norm_dist_white_noise(rng) * 0.1;
        let violet = white - self.previous_sample;
        self.previous_sample = white;
        return violet;
    }
}

#[derive(Enum, PartialEq, Debug)]
pub enum NoiseType {
    #[id = "white"]
    White,
    #[id = "pink"]
    Pink,
    #[id = "brown"]
    Brown,
    #[id = "violet"]
    Violet,
}

#[derive(Params)]
pub struct NoiseParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "noise type"]
    pub noise_type: EnumParam<NoiseType>,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            noise_type: EnumParam::new("Noise Type", NoiseType::White),
        }
    }
}
