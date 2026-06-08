use crate::config;
use egui::Ui;
use std::sync::atomic::Ordering::Relaxed;

pub fn debug_panel(ui: &mut Ui, debug: &config::Debug) {
    ui.group(|ui| {
        ui.label(format!(
            "Current sample value: {}",
            debug.current_sample_val.load(Relaxed)
        ));
        ui.label(format!(
            "Min sample value seen: {}",
            debug.min_sample_val.load(Relaxed)
        ));
        ui.label(format!(
            "Max sample value seen: {}",
            debug.max_sample_val.load(Relaxed)
        ));
        ui.label(format!(
            "Current sampling rate: {}",
            debug.sample_rate.load(Relaxed)
        ));
        ui.label(format!(
            "Output buffer len: {}",
            debug.output_buffer.load(Relaxed)
        ));
        ui.label(format!("Mix level: {}", debug.mix.load(Relaxed)));
        ui.label(format!("Gain level: {}", debug.gain.load(Relaxed)));
        ui.label(format!("Envelope: {}", debug.envelope.load(Relaxed)));
    });
}
