use egui::style::WidgetVisuals;
use egui::{Color32, CornerRadius, Stroke, Ui};
use nice_plug::context::gui::ParamSetter;
use nice_plug::prelude::{Enum, EnumParam};

const COMBO_BG: Color32 = Color32::from_gray(230);
const COMBO_TEXT: Color32 = Color32::BLACK;
const COMBO_BORDER: Color32 = Color32::from_gray(190);
const POPUP_BG: Color32 = Color32::from_gray(248);

fn fixed_combo_visuals() -> WidgetVisuals {
    WidgetVisuals {
        weak_bg_fill: COMBO_BG,
        bg_fill: COMBO_BG,
        bg_stroke: Stroke::new(1.0, COMBO_BORDER),
        fg_stroke: Stroke::new(1.0, COMBO_TEXT),
        corner_radius: CornerRadius::same(2),
        expansion: 0.0,
    }
}

fn apply_combo_style(style: &mut egui::Style) {
    let fixed = fixed_combo_visuals();
    style.visuals.override_text_color = Some(COMBO_TEXT);
    style.visuals.window_fill = POPUP_BG;
    style.visuals.panel_fill = POPUP_BG;
    style.visuals.widgets.inactive = fixed;
    style.visuals.widgets.hovered = fixed;
    style.visuals.widgets.active = fixed;
    style.visuals.widgets.open = fixed;
}

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
            ui.scope(|ui| {
                apply_combo_style(ui.style_mut());

                egui::ComboBox::from_id_salt(label)
                    .width(combo_width)
                    .selected_text(T::variants()[selected])
                    .popup_style(apply_combo_style.into())
                    .show_ui(ui, |ui| {
                        for (idx, name) in T::variants().iter().enumerate() {
                            ui.selectable_value(&mut selected, idx, *name);
                        }
                    });
            });
        });

        if selected != prev {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, T::from_index(selected));
            setter.end_set_parameter(param);
        }
    });
}
