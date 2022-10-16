use nih_plug::context::GuiContext;
use nih_plug::prelude::{Editor, Param};
use nih_plug_vizia::vizia::{prelude::*, views};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState};

use std::sync::Arc;

use crate::noise;

use self::ui_data_derived_lenses::gui_context;

/// VIZIA uses points instead of pixels for text
const POINT_SCALE: f32 = 0.75;
const ICON_DOWN_OPEN: &str = "\u{e75c}";

const STYLE: &str = r#""#;

#[derive(Lens)]
struct UiData {
    pub gui_context: Arc<dyn GuiContext>,
    params: Arc<noise::NoiseParams>,
    noise_types: Vec<String>,
}

#[derive(Debug)]
enum ParamChangeEvent {
    NoiseEvent(usize),
}

impl Model for UiData {

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            ParamChangeEvent::NoiseEvent(n) => {
                unsafe {
                    self.gui_context.raw_begin_set_parameter(self.params.noise_type.as_ptr());
                }
            }
        });
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(400, 300)
}

pub(crate) fn create(
    params: Arc<noise::NoiseParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, move |cx, context| {

        cx.add_theme(STYLE);

        UiData {
            gui_context: context.clone(),
            params: params.clone(),
            noise_types: vec!["white".to_string(), "pink".to_string()],
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "noisegen")
                .font(assets::NOTO_SANS_THIN)
                .font_size(40.0 * POINT_SCALE)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));
            Label::new(cx, "Gain").bottom(Pixels(-1.0));
            ParamSlider::new(cx, UiData::params, |params| &params.gain)
                .bottom(Pixels(1.0));
            Dropdown::new(
                cx,
                move |cx| {
                    HStack::new(
                        cx, 
                        move |cx| {
                            Label::new(cx, UiData::params.map(|p| p.noise_type.to_string()));
                            Label::new(cx, ICON_DOWN_OPEN).class("arrow");
                        }
                    )
                    .class("title")
                },
                move |cx| {
                    // List of options
                    List::new(cx, UiData::noise_types, move |cx, idx, item| {
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
                                            cx.emit(ParamChangeEvent::NoiseEvent(idx));
                                            cx.emit(views::PopupEvent::Close);
                                        });
                                    }
                                );
                            }
                        );
                    });
                }
            );
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
                
