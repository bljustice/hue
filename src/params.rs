use nih_plug::prelude::{
    formatters, util, Enum, EnumParam, FloatParam, FloatRange, Params, SmoothingStyle,
};
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

use crate::editor;

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
