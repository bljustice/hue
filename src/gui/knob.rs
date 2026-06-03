use vizia_plug::vizia::{
    prelude::*,
    view::View,
    views::{ArcTrack, Knob, TickKnob},
};

pub struct KnobState {
    value: Signal<f32>,
}

pub enum KnobEvent {
    SetValue(f32),
}

impl Model for KnobState {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            KnobEvent::SetValue(value) => {
                self.value.set(*value);
            }
        });
    }
}

pub struct KnobContainer {}

pub fn knob(cx: &mut Context, label: &str) {
    let value = Signal::new(0.0);

    VStack::new(cx, |cx| {
        KnobState {value}.build(cx);
        Label::new(cx, label.to_string()).class("knob");
        Knob::new(cx, 0.5, value, false).on_change(|cx, val| cx.emit(KnobEvent::SetValue(val)));
    });
}