use nih_plug::prelude::{
    formatters, util, Enum, EnumParam, FloatParam, FloatRange, Params, SmoothingStyle,
};
use nih_plug_vizia::ViziaState;
use std::{mem, sync::Arc};

use crate::editor;
use rand::{rngs::{StdRng}, thread_rng, Rng, SeedableRng};

pub struct Noise {
    pub params: Arc<NoiseParams>,
    pub rng: StdRng,
    pub white: White,
    pub pink: Pink,
    pub brown: Brown,
}

impl Default for Noise {
    fn default() -> Self {
        Noise {
            params: Arc::new(NoiseParams::default()),
            rng: StdRng::from_rng(thread_rng()).unwrap(),
            white: White::new(),
            pink: Pink::new(),
            brown: Brown::new(0.1)
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
        White {}
    }
}

impl NoiseConfig for White {
    fn reset(&mut self) {}

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        return rng.gen_range(-1.0..1.0) / 8.0;
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
        Brown {
            current_sample: 0.0,
            leak: leak,
        }
    }
}

impl NoiseConfig for Brown {
    fn reset(&mut self) {
        mem::replace(self, Brown::new(0.1,));
    }

    fn next(&mut self, rng: &mut StdRng) -> f32 {
        let white = rng.gen_range(-1.0..1.0);
        self.current_sample = (1.0 - self.leak) * self.current_sample + white;
        return self.current_sample;
    }
    
}

#[derive(Enum, PartialEq)]
pub enum NoiseType {
    #[id = "white"]
    White,
    #[id = "pink"]
    Pink,
    #[id = "brown"]
    Brown,
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
