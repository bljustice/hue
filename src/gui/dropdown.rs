use egui::Ui;
use nice_plug::context::gui::ParamSetter;
use nice_plug::prelude::{Enum, EnumParam};

pub fn enum_column<T: Enum + PartialEq + 'static>(
    ui: &mut Ui,
    label: &str,
    param: &EnumParam<T>,
    setter: &ParamSetter,
) {
    ui.vertical_centered(|ui| {
        ui.label(label);

        let current = param.value();
        let mut selected = T::to_index(current);
        let prev = selected;

        let combo_width = 140.0;
        ui.allocate_ui(egui::vec2(combo_width, ui.spacing().interact_size.y), |ui| {
            egui::ComboBox::from_id_salt(label)
                .width(combo_width)
                .selected_text(T::variants()[selected])
                .show_ui(ui, |ui| {
                    for (idx, name) in T::variants().iter().enumerate() {
                        ui.selectable_value(&mut selected, idx, *name);
                    }
                });
        });

        if selected != prev {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, T::from_index(selected));
            setter.end_set_parameter(param);
        }
    });
}
