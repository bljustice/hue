use nih_plug_vizia::vizia::{
    prelude::*,
    view::View,
    views::{ArcTrack, Knob, TickKnob},
};

pub struct KnobContainer {}

impl View for KnobContainer {
    fn element(&self) -> Option<&'static str> {
        Some("knob-vstack")
    }
}

impl KnobContainer {
    pub fn new<L, V, F>(
        cx: &mut Context,
        label: String,
        lens: L,
        value_lens: V,
        on_change_callback: F,
    ) -> Handle<Self>
    where
        L: Lens<Target = f32>,
        V: Lens<Target = String>,
        F: 'static + Fn(&mut EventContext, f32),
    {
        Self {}.build(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, &label);
                Knob::custom(cx, 0.5, lens, move |cx, lens| {
                    TickKnob::new(
                        cx,
                        Percentage(80.),
                        Pixels(2.),
                        Percentage(75.),
                        270.,
                        KnobMode::Continuous,
                    )
                    .value(lens.clone())
                    .class("tick");
                    ArcTrack::new(
                        cx,
                        false,
                        Percentage(100.),
                        Percentage(10.),
                        -135.,
                        135.,
                        KnobMode::Continuous,
                    )
                    .value(lens)
                    .class("track")
                })
                .on_changing(on_change_callback);
                Label::new(cx, value_lens);
            })
            .child_space(Stretch(1.0));
        })
    }
}
