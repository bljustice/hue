use atomic_float::AtomicF32;
use nih_plug::context::{GuiContext, ParamSetter};
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::style::Color;
use nih_plug_vizia::vizia::{prelude::*, views};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState};

use std::sync::{Arc, atomic::Ordering};

use crate::noise;

/// VIZIA uses points instead of pixels for text
const PLUGIN_WIDTH: f32 = 400.0;
const POINT_SCALE: f32 = 0.75;
const ICON_DOWN_OPEN: &str = "\u{e75c}";

const STYLE: &str = r#""#;

#[derive(Lens)]
struct UiData {
    pub gui_context: Arc<dyn GuiContext>,
    params: Arc<noise::NoiseParams>,
    noise_types: Vec<String>,
    current_sample_val: Arc<AtomicF32>,
}

#[derive(Debug)]
enum ParamChangeEvent {
    NoiseEvent(String),
}

impl Model for UiData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        let setter = ParamSetter::new(self.gui_context.as_ref());
        event.map(|e, _| match e {
            ParamChangeEvent::NoiseEvent(s) => {
                if s == "white" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, noise::NoiseType::White);
                    setter.end_set_parameter(&self.params.noise_type);
                } else if s == "pink" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, noise::NoiseType::Pink);
                    setter.end_set_parameter(&self.params.noise_type);
                } else if s == "brown" {
                    setter.begin_set_parameter(&self.params.noise_type);
                    setter.set_parameter(&self.params.noise_type, noise::NoiseType::Brown);
                    setter.end_set_parameter(&self.params.noise_type);
                }
            }
        });
    }
}


pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(PLUGIN_WIDTH as u32, 300)
}

pub(crate) fn create(
    params: Arc<noise::NoiseParams>,
    editor_state: Arc<ViziaState>,
    current_sample_val: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, move |cx, context| {
        cx.add_theme(STYLE);

        UiData {
            gui_context: context.clone(),
            params: params.clone(),
            current_sample_val: current_sample_val.clone(),
            noise_types: vec!["white".to_string(), "pink".to_string(), "brown".to_string()],
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
        )
    })
}

fn change_plugin_color(noise_color: &str) -> Color {
    let plugin_color = match noise_color {
        "white" => Color::from("#F9F6EE"),
        "pink" => Color::from("#FFC0CB"),
        "brown" => Color::from("#C19A6B"),
        _ => Color::from("#F9F6EE"),
    };

    return plugin_color;
}

fn build_gui(cx: &mut Context) -> Handle<VStack> {
    return VStack::new(cx, |cx| {
        Label::new(cx, "noisegen")
            .font(assets::NOTO_SANS_THIN)
            .font_size(40.0 * POINT_SCALE)
            .height(Pixels(50.0))
            .child_top(Stretch(1.0))
            .child_bottom(Pixels(0.0));
        Label::new(cx, "Gain").bottom(Pixels(-1.0));
        ParamSlider::new(cx, UiData::params, |params| &params.gain).bottom(Pixels(1.0));
        Dropdown::new(
            cx,
            move |cx| {
                HStack::new(cx, move |cx| {
                    Label::new(cx, UiData::params.map(|p| p.noise_type.to_string()));
                    Label::new(cx, ICON_DOWN_OPEN).class("arrow");
                })
                .class("title")
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
                                    .width(Stretch(1.0))
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
        );
        if cfg!(debug_assertions) {
            build_debug_window(cx);
        }
    })
    .row_between(Pixels(0.0))
    .child_left(Stretch(1.0))
    .child_right(Stretch(1.0));
}

fn build_debug_window(cx: &mut Context) -> Handle<VStack> {

    return VStack::new(
        cx,
        move |cx| {
            Dropdown::new(
                cx,
                move |cx| {
                    HStack::new(cx, move |cx| {
                        Label::new(cx, "Debug");
                        Label::new(cx, ICON_DOWN_OPEN).class("debug-dropdown-arrow");
                    })
                    .class("debug-dropdown-container")
                },
                |cx| {
                    Binding::new(
                        cx,
                        UiData::current_sample_val.map(|p| p.load(Ordering::Relaxed).to_string()),
                        move |cx, lens| {
                            VStack::new(
                                cx,
                                move | cx | {
                                    HStack::new(
                                        cx,
                                        move | cx | {
                                            let debug_sample_val = lens.get(cx);
                                            let fmt_string = format!("Current sample {}", debug_sample_val);
                                            Label::new(cx, &fmt_string)
                                                .child_right(Pixels(20.0));        
                                            Label::new(cx, "DB to Gain Val: ");
                                        }
                                    )
                                    .class("debug-text-container");
                                }
                            )
                            .height(Pixels(10.0))
                            .background_color(Color::rgb(255, 255, 255))
                            .class("debug-vstack-container");
                        }
                    )
                }
            )
            .width(Pixels(PLUGIN_WIDTH))
            .child_top(Pixels(100.0))
            .color(Color::rgb(0x69, 0x69, 0x69));
        }
    );
}
 
