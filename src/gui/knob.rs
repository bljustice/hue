use nih_plug_vizia::vizia::{prelude::*, view::View, views};

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
                views::Knob::new(cx, 0.5, lens, false).on_changing(on_change_callback);
                Label::new(cx, value_lens);
            })
            .child_space(Stretch(1.0));
        })
    }
}
