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

#[derive(Enum, PartialEq, Debug)]
pub enum WhiteNoiseDistribution {
    Uniform,
    Normal,
}

#[derive(Params)]
pub struct NoiseParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "noise type"]
    pub noise_type: EnumParam<NoiseType>,

    #[id = "white noise distribution"]
    pub white_noise_distribution: EnumParam<WhiteNoiseDistribution>,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "lpf fc"]
    pub lpf_fc: FloatParam,
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
            noise_type: EnumParam::new("Noise Type", NoiseType::White),
            white_noise_distribution: EnumParam::new(
                "White Noise Distribution",
                WhiteNoiseDistribution::Normal,
            ),
            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_smoother(SmoothingStyle::Linear(10.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            lpf_fc: FloatParam::new(
                "LP FC",
                5000.,
                FloatRange::Skewed {
                    min: 1000.,
                    max: 20_000.,
                    factor: FloatRange::skew_factor(-1.0),
                }
                )
                .with_smoother(SmoothingStyle::Logarithmic(100.0))
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
                .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
        }
    }
}
