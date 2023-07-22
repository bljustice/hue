use nih_plug_vizia::vizia::prelude::*;

pub struct DebugContainer {}

impl View for DebugContainer {}

impl DebugContainer {
    pub fn new<L>(cx: &mut Context, lens: L, class_name: String) -> Handle<Self> 
    where L: Lens<Target = Vec<(String, f32)>>,
    {
        Self {

        }.build(cx, |cx| {
            VStack::new(cx, move |cx| {
                Binding::new(
                    cx,
                    lens,
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
                )
            })
            .class(&class_name)
            .background_color(Color::rgb(255, 255, 255))
            .color(Color::rgb(0x69, 0x69, 0x69));
        })
    }
}
