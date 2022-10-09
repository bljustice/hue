use nih_plug::prelude::{Params, FloatParam, FloatRange, SmoothingStyle, formatters, util};
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

use rand::{self, Rng};
use crate::editor;

pub struct Noise {
    pub params: Arc<NoiseParams>,
}

impl Noise {
    
    pub fn white(&self) -> f32 {
        // returns a float64 between 0 and 1
        return rand::thread_rng().gen();
    }

}

#[derive(Params)]
pub struct NoiseParams {

    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,

}

impl Default for Noise {
    fn default() -> Self {
        Self {
            params: Arc::new(NoiseParams::default()),
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
        }
    }
}

