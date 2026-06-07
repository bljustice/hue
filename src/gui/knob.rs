use egui::Ui;
use nice_plug::prelude::{Enum, EnumParam, Param};
use nice_plug::context::gui::ParamSetter;
use nice_plug_egui::widgets::ParamSlider;

pub fn param_column<P: Param>(ui: &mut Ui, label: &str, param: &P, setter: &ParamSetter) {
    ui.vertical(|ui| {
        ui.label(label);
        ui.add(ParamSlider::for_param(param, setter).with_width(70.0));
    });
}

pub fn enum_column<T: Enum + PartialEq + 'static>(
    ui: &mut Ui,
    label: &str,
    param: &EnumParam<T>,
    setter: &ParamSetter,
) {
    ui.vertical(|ui| {
        ui.label(label);
        let current = param.value();
        let mut selected = T::to_index(current);
        let prev = selected;

        egui::ComboBox::from_id_salt(label)
            .selected_text(T::variants()[selected])
            .show_ui(ui, |ui| {
                for (idx, name) in T::variants().iter().enumerate() {
                    ui.selectable_value(&mut selected, idx, *name);
                }
            });

        if selected != prev {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, T::from_index(selected));
            setter.end_set_parameter(param);
        }
    });
}
