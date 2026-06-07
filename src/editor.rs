use atomic_float::AtomicF32;
use egui::{Color32, Margin, RichText, Separator};
use nice_plug::prelude::Editor;
use nice_plug_egui::{create_egui_editor, EguiSettings, EguiState};
use std::sync::Arc;

use crate::config;
use crate::gui::analyzer::{spectrum_analyzer, SpectrumBuffer};
use crate::gui::debug::debug_panel;
use crate::gui::knob::{enum_column, param_column};
use crate::params::{NoiseParams, NoiseType};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_WIDTH: f32 = 600.0;
const PLUGIN_HEIGHT: f32 = 550.0;

pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(PLUGIN_WIDTH as u32, PLUGIN_HEIGHT as u32)
}

pub(crate) fn create(
    params: Arc<NoiseParams>,
    editor_state: Arc<EguiState>,
    debug: config::Debug,
    sample_rate: Arc<AtomicF32>,
    spectrum_buffer: SpectrumBuffer,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state.clone(),
        (),
        EguiSettings::default(),
        |ctx, _queue, _state| {
            ctx.global_style_mut(|style| {
                style.visuals.window_fill = Color32::from_rgb(0xF9, 0xF6, 0xEE);
                style.visuals.panel_fill = Color32::from_rgb(0xF9, 0xF6, 0xEE);
            });
        },
        move |ui, setter, _queue, _state| {
            let bg = noise_type_color(params.noise_type.value());

            egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(Margin::same(12)).fill(bg))
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("hue")
                            .size(30.0)
                            .color(Color32::from_gray(40)));
                        ui.label(format!("v{VERSION}"));
                    });
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        spectrum_analyzer(ui, &spectrum_buffer, &sample_rate);
                    });
                    ui.add(Separator::default());
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            param_column(ui, "Gain", &params.gain, setter);
                            param_column(ui, "Mix", &params.mix, setter);
                            param_column(ui, "HPF", &params.hpf_fc, setter);
                            param_column(ui, "LPF", &params.lpf_fc, setter);
                        });
                    });
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            enum_column(ui, "Noise Type", &params.noise_type, setter);
                            ui.add_space(32.0);
                            enum_column(ui, "Envelope Mode", &params.env_mode, setter);
                        });
                    });

                    if !cfg!(debug_assertions) {
                        ui.add_space(ui.available_height() * 0.25);
                    } else {
                        ui.add_space(8.0);
                    }

                    if cfg!(debug_assertions) {
                        ui.add_space(8.0);
                        debug_panel(ui, &debug);
                    }
                });
        },
    )
}

fn noise_type_color(noise_type: NoiseType) -> Color32 {
    match noise_type {
        NoiseType::White => Color32::from_rgb(0xF9, 0xF6, 0xEE),
        NoiseType::Pink => Color32::from_rgb(0xFF, 0xC0, 0xCB),
        NoiseType::Brown => Color32::from_rgb(0xC1, 0x9A, 0x6B),
        NoiseType::Violet => Color32::from_rgb(0xCF, 0x9F, 0xFF),
    }
}
