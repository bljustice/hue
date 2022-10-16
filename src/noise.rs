use nih_plug::prelude::{Params, FloatParam, FloatRange, SmoothingStyle, formatters, util, EnumParam, Enum};
use nih_plug_vizia::ViziaState;
use nih_plug::context::GuiContext;
use std::sync::Arc;

use rand::{self, Rng};
use crate::editor;

pub struct Noise {
    pub params: Arc<NoiseParams>,
    pub pink_b0: f32,
    pub pink_b1: f32,
    pub pink_b2: f32,
    pub pink_b3: f32,
    pub pink_b4: f32,
    pub pink_b5: f32,
    pub pink_b6: f32,
}

impl Noise {
    
    pub fn white(&self) -> f32 {
        // returns a float64 between 0 and 1
        return rand::thread_rng().gen();
    }


    pub fn pink(&mut self) -> f32 {

        // Pink noise via Paul Kellet's method

        let white = self.white();

        self.pink_b0 = 0.99886 * self.pink_b0 + white * 0.0555179;
        self.pink_b1 = 0.99332 * self.pink_b1 + white * 0.0750759;
        self.pink_b2 = 0.96900 * self.pink_b2 + white * 0.1538520;
        self.pink_b3 = 0.86650 * self.pink_b3 + white * 0.3104856;
        self.pink_b4 = 0.55000 * self.pink_b4 + white * 0.5329522;
        self.pink_b5 = -0.7616 * self.pink_b5 - white * 0.0168980;
        self.pink_b6 = white * 0.115926;
        let pink = self.pink_b0 + self.pink_b1 + self.pink_b2 + self.pink_b3 + self.pink_b4 + self.pink_b5 + self.pink_b6 + white * 0.5362;
        return pink;

    }

}
#[derive(Enum, PartialEq)]
pub enum NoiseType {
    #[id = "white"]
    White,
    #[id = "pink"]
    Pink,
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

impl Default for Noise {
    fn default() -> Self {
        Self {
            params: Arc::new(NoiseParams::default()),
            pink_b0: 0.0,
            pink_b1: 0.0,
            pink_b2: 0.0,
            pink_b3: 0.0,
            pink_b4: 0.0,
            pink_b5: 0.0,
            pink_b6: 0.0,
        }
    }
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
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            noise_type: EnumParam::new("white", NoiseType::White),
        }
    }
}

