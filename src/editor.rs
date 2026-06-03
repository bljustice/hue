use atomic_float::AtomicF32;
use nice_plug::prelude::{Editor, Param};
use std::sync::Arc;
use vizia_plug::vizia::prelude::*;
use vizia_plug::widgets::param_base::ParamWidgetBase;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};

use crate::gui::analyzer::{SpectrumAnalyzer, SpectrumBuffer};
use crate::gui::debug::DebugContainer;
use crate::gui::knob::knob;
use crate::params::{NoiseParams, NoiseType};
use crate::config;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_WIDTH: f32 = 400.0;
const PLUGIN_HEIGHT: f32 = 550.0;
const POINT_SCALE: f32 = 0.75;
const NOTO_SANS: &str = "Noto Sans";

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (PLUGIN_WIDTH as u32, PLUGIN_HEIGHT as u32))
}   

pub(crate) fn create(
    params: Arc<NoiseParams>,
    editor_state: Arc<ViziaState>,
    debug: config::Debug,
    sample_rate: Arc<AtomicF32>,
    spectrum_buffer: SpectrumBuffer,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _context| {
        if let Err(err) = cx.add_stylesheet(include_str!("gui/style.css")) {
            eprintln!("Failed to load stylesheet: {err:?}");
        }

        let noise_norm = ParamWidgetBase::new(cx, &params.noise_type).modulated_signal(cx);
        let params_for_background = params.clone();
        let background = noise_norm.map(move |norm| {
            noise_type_color(params_for_background.noise_type.preview_plain(*norm))
        });

        let width = Signal::new(Pixels(PLUGIN_WIDTH));
        let height = Signal::new(Pixels(PLUGIN_HEIGHT));

        Resizable::new(
            cx,
            width,
            ResizeStackDirection::Right,
            move |_cx, w| width.set(Pixels(w)),
            |cx| {
                VStack::new(cx, |cx| {
                    create_title_block(cx);
                    create_spectrum_analyzer(cx, spectrum_buffer.clone(), sample_rate.clone());
                    Divider::new(cx).class("divider");
                    create_knob_row(cx, &params);
                    create_param_selectors(cx, &params);
                    if cfg!(debug_assertions) {
                        DebugContainer::new(cx, debug.clone());
                    }
                })
                .background_color(background)
                .gap(Pixels(0.0))
                .alignment(Alignment::TopCenter);
            }
        )
        .on_reset(move |_cx| width.set(Pixels(PLUGIN_WIDTH)))
        .on_reset(move |_cx| height.set(Pixels(PLUGIN_HEIGHT)));
    })
}

fn noise_type_color(noise_type: NoiseType) -> Color {
    match noise_type {
        NoiseType::White => Color::from("#F9F6EE"),
        NoiseType::Pink => Color::from("#FFC0CB"),
        NoiseType::Brown => Color::from("#C19A6B"),
        NoiseType::Violet => Color::from("#CF9FFF"),
    }
}

fn create_title_block(cx: &mut Context) -> Handle<VStack> {
    let version_str = format!("v{}", VERSION);
    VStack::new(cx, |cx| {
        Label::new(cx, "hue")
            .font_family(vec![FamilyOwned::Named(String::from(NOTO_SANS))])
            .font_weight(FontWeightKeyword::Light)
            .font_size(40.0 * POINT_SCALE);
        Label::new(cx, version_str).font_size(15.0 * POINT_SCALE);
    })
    .class("title-container")
}

fn create_knob_row(cx: &mut Context, params: &NoiseParams) {
    HStack::new(cx, |cx| {
        param_column(cx, "Gain", &params.gain);
        param_column(cx, "Mix", &params.mix);
        param_column(cx, "HPF", &params.hpf_fc);
        param_column(cx, "LPF", &params.lpf_fc);
    })
    .class("knob-container");
}

fn param_column<P: nice_plug::prelude::Param + 'static>(
    cx: &mut Context,
    label: &str,
    param: &P,
) {
    VStack::new(cx, |cx| {
        knob(cx, label);
    });
}

fn create_param_selectors(cx: &mut Context, params: &NoiseParams) {
    let build = |cx: &mut Context| {
        param_column(
            cx,
            "Noise Type",
            &params.noise_type,
        );
        param_column(
            cx,
            "Envelope Mode",
            &params.env_mode,
        );
    };

    if cfg!(debug_assertions) {
        HStack::new(cx, build).class("all-dropdowns-container");
    } else {
        HStack::new(cx, build)
            .class("all-dropdowns-container")
            .bottom(Percentage(25.0));
    }
}

fn create_spectrum_analyzer(
    cx: &mut Context,
    spectrum_buffer: SpectrumBuffer,
    sample_rate: Arc<AtomicF32>,
) -> Handle<HStack> {
    HStack::new(cx, |cx| {
        ZStack::new(cx, |cx| {
            SpectrumAnalyzer::new(cx, spectrum_buffer, sample_rate);
        });
    })
    .class("spectrum-analyzer-container")
}
