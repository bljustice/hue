use egui::{Color32, Ui};
use egui_knob::{Knob, KnobStyle};
use nice_plug::context::gui::ParamSetter;
use nice_plug::prelude::Param;

const KNOB_COLOR: Color32 = Color32::BLACK;

pub fn param_knob<P: Param>(ui: &mut Ui, label: &str, param: &P, setter: &ParamSetter) {
    let mut normalized = param.modulated_normalized_value();
    let default_normalized = param.preview_normalized(param.default_plain_value());

    ui.vertical_centered(|ui| {
        ui.label(label);
        let response = ui.add(
            Knob::new(&mut normalized, 0.0, 1.0, KnobStyle::Wiper)
                .with_size(44.0)
                .with_stroke_width(2.0)
                .with_background_arc(true)
                .with_show_filled_segments(true)
                .with_double_click_reset(default_normalized)
                .with_colors(KNOB_COLOR, KNOB_COLOR, KNOB_COLOR),
        );
        if response.drag_started() {
            setter.begin_set_parameter(param);
        }
        if response.changed() {
            let plain = param.preview_plain(normalized);
            if plain != param.modulated_plain_value() {
                setter.set_parameter(param, plain);
            }
        }
        if response.drag_stopped() {
            setter.end_set_parameter(param);
        }

        ui.label(param.normalized_value_to_string(normalized, true));
    });
}

