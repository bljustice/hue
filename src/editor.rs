use atomic_float::AtomicF32;
use nih_plug::context::gui::{GuiContext, ParamSetter};
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::style::Color;
use nih_plug_vizia::vizia::{prelude::*, views};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::{atomic::Ordering, Arc};

use crate::config;
use crate::gui::analyzer::{SpectrumAnalyzer, SpectrumBuffer};
use crate::params::{NoiseParams, NoiseType, WhiteNoiseDistribution};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_WIDTH: f32 = 400.0;
const PLUGIN_HEIGHT: f32 = 450.0;
const POINT_SCALE: f32 = 0.75;
const ICON_DOWN_OPEN: &str = "\u{e75c}";

#[derive(Lens)]
struct UiData {
    pub gui_context: Arc<dyn GuiContext>,
    params: Arc<NoiseParams>,
    noise_types: Vec<String>,
    white_noise_types: Vec<String>,
    debug: config::Debug,
    sample_rate: Arc<AtomicF32>,
    spectrum_buffer: SpectrumBuffer,
}

#[derive(Debug)]
pub enum ParamChangeEvent {
    NoiseEvent(String),
    WhiteNoiseDistributionEvent(String),
}

impl Model for UiData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        let setter = ParamSetter::new(self.gui_context.as_ref());
        event.map(|e, _| match e {
            ParamChangeEvent::NoiseEvent(s) => {
                if s == "white" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, NoiseType::White);
                    setter.end_set_parameter(&self.params.noise_type);
                } else if s == "pink" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, NoiseType::Pink);
                    setter.end_set_parameter(&self.params.noise_type);
                } else if s == "brown" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, NoiseType::Brown);
                    setter.end_set_parameter(&self.params.noise_type);
                } else if s == "violet" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, NoiseType::Violet);
                    setter.end_set_parameter(&self.params.noise_type);
                }
            }
            ParamChangeEvent::WhiteNoiseDistributionEvent(s) => {
                if s == "normal" {
                    setter.begin_set_parameter(&self.params.white_noise_distribution);
                    setter.set_parameter(
                        &self.params.white_noise_distribution,
                        WhiteNoiseDistribution::Normal,
                    );
                    setter.end_set_parameter(&self.params.white_noise_distribution);
                } else if s == "uniform" {
                    setter.begin_set_parameter(&self.params.white_noise_distribution);
                    setter.set_parameter(
                        &self.params.white_noise_distribution,
                        WhiteNoiseDistribution::Uniform,
                    );
                    setter.end_set_parameter(&self.params.white_noise_distribution);
                }
            }
        });
    }
}

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
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, context| {

        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);
        cx.add_theme(include_str!("gui/style.css"));

        UiData {
            gui_context: context.clone(),
            params: params.clone(),
            debug: debug.clone(),
            noise_types: vec![
                "white".to_string(),
                "pink".to_string(),
                "brown".to_string(),
                "violet".to_string(),
            ],
            white_noise_types: vec!["normal".to_string(), "uniform".to_string()],
            sample_rate: sample_rate.clone(),
            spectrum_buffer: spectrum_buffer.clone(),
        }
        .build(cx);
        ResizeHandle::new(cx);
        Binding::new(
            cx,
            UiData::params.map(|p| p.noise_type.to_string().to_lowercase()),
            move |cx, lens| {
                let noise_color = lens.get(cx);
                build_gui(cx).background_color(change_plugin_color(&noise_color));
            },
        );
    })
}

fn change_plugin_color(noise_color: &str) -> Color {
    let plugin_color = match noise_color {
        "white" => Color::from("#F9F6EE"),
        "pink" => Color::from("#FFC0CB"),
        "brown" => Color::from("#C19A6B"),
        "violet" => Color::from("#CF9FFF"),
        _ => Color::from("#F9F6EE"),
    };

    return plugin_color;
}

fn create_title_block(cx: &mut Context) -> Handle<VStack> {
    let version_str = format!("v{}", VERSION);
    VStack::new(cx, |cx| {
        Label::new(cx, "noisegen")
            .font_family(vec![FamilyOwned::Name(String::from(
                assets::NOTO_SANS_THIN,
            ))])
            .font_size(40.0 * POINT_SCALE);
        Label::new(cx, &version_str).font_size(15.0 * POINT_SCALE);
    })
    .class("title-container")
    .child_space(Stretch(1.0))
}

fn create_gain_block(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "Gain").left(Percentage(40.0));
        ParamSlider::new(cx, UiData::params, |params| &params.gain);
    })
    .class("gain-container")
}

fn create_spectrum_analyzer(cx: &mut Context) -> Handle<HStack> {
    HStack::new(cx, |cx| {
        ZStack::new(cx, |cx| {
            SpectrumAnalyzer::new(
                cx,
                UiData::spectrum_buffer.get(cx),
                UiData::sample_rate.get(cx),
            );
        });
    })
    .class("spectrum-analyzer-container")
}

fn create_white_noise_selector(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "White Noise Distribution").font_size(15.0 * POINT_SCALE);
        Dropdown::new(
            cx,
            move |cx| {
                HStack::new(cx, move |cx| {
                    Label::new(
                        cx,
                        UiData::params.map(|p| p.white_noise_distribution.to_string()),
                    );
                    Label::new(cx, ICON_DOWN_OPEN).class("arrow");
                })
                .class("title")
                .child_space(Stretch(1.0))
            },
            move |cx| {
                // List of options
                List::new(cx, UiData::white_noise_types, move |cx, _idx, item| {
                    VStack::new(cx, move |cx| {
                        Binding::new(
                            cx,
                            UiData::params.map(|p| p.white_noise_distribution.to_string()),
                            move |cx, choice| {
                                let selected = *item.get(cx) == *choice.get(cx);
                                Label::new(cx, &item.get(cx))
                                    .width(Percentage(100.0))
                                    .background_color(if selected {
                                        Color::from("#c28919")
                                    } else {
                                        Color::transparent()
                                    })
                                    .on_press(move |cx| {
                                        cx.emit(ParamChangeEvent::WhiteNoiseDistributionEvent(
                                            item.get(cx),
                                        ));
                                        cx.emit(views::PopupEvent::Close);
                                    });
                            },
                        );
                    });
                });
            },
        )
        .child_space(Stretch(1.0))
        .width(Percentage(100.0));
    })
    .child_space(Stretch(1.0))
    .class("white-noise-dropdown-container")
}

fn create_noise_selector(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "Noise Type").font_size(15.0 * POINT_SCALE);
        Dropdown::new(
            cx,
            move |cx| {
                HStack::new(cx, move |cx| {
                    Label::new(cx, UiData::params.map(|p| p.noise_type.to_string()));
                    Label::new(cx, ICON_DOWN_OPEN).class("arrow");
                })
                .class("title")
                .child_space(Stretch(1.0))
            },
            move |cx| {
                // List of options
                List::new(cx, UiData::noise_types, move |cx, _idx, item| {
                    VStack::new(cx, move |cx| {
                        Binding::new(
                            cx,
                            UiData::params.map(|p| p.noise_type.to_string()),
                            move |cx, choice| {
                                let selected = *item.get(cx) == *choice.get(cx);
                                Label::new(cx, &item.get(cx))
                                    .width(Percentage(100.0))
                                    .background_color(if selected {
                                        Color::from("#c28919")
                                    } else {
                                        Color::transparent()
                                    })
                                    .on_press(move |cx| {
                                        cx.emit(ParamChangeEvent::NoiseEvent(item.get(cx)));
                                        cx.emit(views::PopupEvent::Close);
                                    });
                            },
                        );
                    });
                });
            },
        )
        .child_space(Stretch(1.0))
        .width(Percentage(100.0));
    })
    .height(Percentage(10.0))
    .top(Percentage(5.0))
    .width(Percentage(100.0))
    .child_space(Stretch(1.0))
}

fn build_gui(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        create_title_block(cx);
        create_gain_block(cx);
        create_spectrum_analyzer(cx);
        HStack::new(cx, move |cx| {
            create_noise_selector(cx);
            create_white_noise_selector(cx);
        })
        .child_space(Stretch(1.0));
        if cfg!(debug_assertions) {
            build_debug_window(cx);
        }
    })
    .row_between(Pixels(0.0))
    .child_left(Stretch(1.0))
    .child_right(Stretch(1.0))
}

fn build_debug_window(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, move |cx| {
        Binding::new(
            cx,
            UiData::debug.map(|p| {
                return vec![
                    (
                        "Curent sample value",
                        p.current_sample_val.load(Ordering::Relaxed),
                    ),
                    (
                        "Min sample value seen",
                        p.min_sample_val.load(Ordering::Relaxed),
                    ),
                    (
                        "Max sample value seen",
                        p.max_sample_val.load(Ordering::Relaxed),
                    ),
                    (
                        "Current sampling rate",
                        p.sample_rate.load(Ordering::Relaxed),
                    ),
                    ("Output buffer len", p.output_buffer.load(Ordering::Relaxed)),
                ];
            }),
            move |cx, lens| {
                let debug_vals = lens.get(cx);
                for val_tuple in debug_vals {
                    HStack::new(cx, move |cx| {
                        let (sample_str, sample_val) = val_tuple;
                        let label_str = format!("{}: {}", &sample_str, &sample_val.to_string());
                        Label::new(cx, &label_str);
                    });
                }
            },
        );
        Binding::new(
            cx,
            UiData::params.map(|p| nih_plug::util::db_to_gain(p.gain.value())),
            move |cx, lens| {
                HStack::new(cx, move |cx| {
                    let gain_val = lens.get(cx);
                    let gain_str = format!("dB to gain val: {}", &gain_val.to_string());
                    Label::new(cx, &gain_str);
                });
            },
        );
    })
    .class("debug-container")
}
