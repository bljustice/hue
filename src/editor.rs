use atomic_float::AtomicF32;
use nih_plug::context::gui::{GuiContext, ParamSetter};
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::style::Color;
use nih_plug_vizia::vizia::{prelude::*, views};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::{atomic::Ordering::Relaxed, Arc};

use crate::config;
use crate::gui::analyzer::{SpectrumAnalyzer, SpectrumBuffer};
use crate::gui::debug::DebugContainer;
use crate::params::{NoiseParams, NoiseType, WhiteNoiseDistribution};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_WIDTH: f32 = 400.0;
const PLUGIN_HEIGHT: f32 = 550.0;
const POINT_SCALE: f32 = 0.75;
const ICON_DOWN_OPEN: &str = "\u{25BC}";

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
    MixBeginSet,
    MixEndSet,
    MixSet(f32),
    GainBeginSet,
    GainEndSet,
    GainSet(f32),
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
            ParamChangeEvent::MixBeginSet => {
                setter.begin_set_parameter(&self.params.mix);
            }
            ParamChangeEvent::MixSet(f) => {
                setter.set_parameter(&self.params.mix, *f);
            }
            ParamChangeEvent::MixEndSet => {
                setter.end_set_parameter(&self.params.mix);
            }
            ParamChangeEvent::GainBeginSet => {
                setter.begin_set_parameter(&self.params.gain);
            }
            ParamChangeEvent::GainSet(f) => {
                setter.set_parameter(&self.params.gain, *f);
            }
            ParamChangeEvent::GainEndSet => {
                setter.end_set_parameter(&self.params.gain);
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
        Label::new(cx, "hue")
            .font_family(vec![FamilyOwned::Name(String::from(
                assets::NOTO_SANS_LIGHT,
            ))])
            .font_size(40.0 * POINT_SCALE);
        Label::new(cx, &version_str).font_size(15.0 * POINT_SCALE);
    })
    .class("title-container")
    .child_space(Stretch(1.0))
}

fn create_gain_block(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "Gain");        
        views::Knob::new(
            cx,
            0.5,
            UiData::params.map(|p| p.gain.value()),
            false
        )
        .on_changing(move |cx, val| {
            cx.emit(ParamChangeEvent::GainSet(val));
        })
        .on_press(move |cx| {
            cx.emit(ParamChangeEvent::GainBeginSet);
        })
        .on_mouse_up(move |cx, _button| {
            cx.emit(ParamChangeEvent::GainEndSet);
        });
        Label::new(cx, UiData::params.map(|p| p.gain.to_string()));
    })
    .class("gain-container")
}

fn create_mix_block(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "Mix");
        views::Knob::new(cx, 0.5, UiData::params.map(|p| p.mix.value()), false)
            .on_changing(move |cx, val| {
                cx.emit(ParamChangeEvent::MixSet(val));
            })
            .on_press(move |cx| {
                cx.emit(ParamChangeEvent::MixBeginSet);
            })
            .on_mouse_up(move |cx, _button| {
                cx.emit(ParamChangeEvent::MixEndSet);
            });
        Label::new(cx, UiData::params.map(|p| p.mix.to_string()));
    })
    .class("mix-container")
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
        Label::new(cx, "Distribution")
            .font_size(15.0 * POINT_SCALE)
            .class("dropdown-label");
        Dropdown::new(
            cx,
            move |cx| {
                VStack::new(cx, move |cx| {
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
                                    })
                                    .child_space(Stretch(1.0))
                                    .class("dropdown-label-value");
                            },
                        );
                    });
                });
            },
        )
        .width(Percentage(90.0))
        .class("white-noise-dropdown");
    })
    .child_space(Stretch(1.0))
    .class("white-noise-dropdown-container")
}

fn create_noise_selector(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Label::new(cx, "Noise Type")
            .font_size(15.0 * POINT_SCALE)
            .class("dropdown-label");
        Dropdown::new(
            cx,
            move |cx| {
                VStack::new(cx, move |cx| {
                    Label::new(cx, UiData::params.map(|p| p.noise_type.to_string()));
                    Label::new(cx, ICON_DOWN_OPEN).class("arrow");
                })
                .class("title")
                .child_space(Stretch(1.0))
            },
            move |cx| {
                List::new(cx, UiData::noise_types, move |cx, _idx, item| {
                    VStack::new(cx, move |cx| {
                        Binding::new(
                            cx,
                            UiData::params.map(|p| p.noise_type.to_string()),
                            move |cx, choice| {
                                let selected = *item.get(cx) == *choice.get(cx);
                                Label::new(cx, &item.get(cx))
                                    .background_color(if selected {
                                        Color::from("#c28919")
                                    } else {
                                        Color::transparent()
                                    })
                                    .on_press(move |cx| {
                                        cx.emit(ParamChangeEvent::NoiseEvent(item.get(cx)));
                                        cx.emit(views::PopupEvent::Close);
                                    })
                                    .child_space(Stretch(1.0))
                                    .class("dropdown-label-value");
                            },
                        );
                    });
                });
            },
        )
        .width(Percentage(90.0))
        .class("noise-dropdown");
    })
    .child_space(Stretch(1.0))
    .class("noise-dropdown-container")
}

fn create_noise_selector_row(cx: &mut Context) -> Handle<HStack> {
    if cfg!(debug_assertions) {
        return HStack::new(cx, move |cx| {
            create_noise_selector(cx);
            create_white_noise_selector(cx);
        })
        .class("all-dropdowns-container")
        .child_space(Stretch(1.0));
    } else {
        return HStack::new(cx, move |cx| {
            create_noise_selector(cx);
            create_white_noise_selector(cx);
        })
        .class("all-dropdowns-container")
        .child_space(Stretch(1.0))
        .bottom(Percentage(25.0));
    }
}

fn build_gui(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        create_title_block(cx);
        create_spectrum_analyzer(cx);
        HStack::new(cx, move |cx| {
            create_gain_block(cx);
            create_mix_block(cx);
        })
        .class("knob-container");
        create_noise_selector_row(cx);
        if cfg!(debug_assertions) {
            HStack::new(cx, move |cx| {
                DebugContainer::new(
                    cx,
                    UiData::debug.map(|p| {
                        return vec![
                            (
                                "Curent sample value".to_string(),
                                p.current_sample_val.load(Relaxed),
                            ),
                            (
                                "Min sample value seen".to_string(),
                                p.min_sample_val.load(Relaxed),
                            ),
                            (
                                "Max sample value seen".to_string(),
                                p.max_sample_val.load(Relaxed),
                            ),
                            (
                                "Current sampling rate".to_string(),
                                p.sample_rate.load(Relaxed),
                            ),
                            ("Output buffer len".to_string(), p.output_buffer.load(Relaxed)),
                            ("Mix level".to_string(), p.mix.load(Relaxed)),
                            ("Gain level".to_string(), p.gain.load(Relaxed)),
                        ];
                    }),
                    "debug-container".to_string()
                );
            })
            .class("debug-row");
        }
    })
    .row_between(Pixels(0.0))
    .child_left(Stretch(1.0))
    .child_right(Stretch(1.0))
}
