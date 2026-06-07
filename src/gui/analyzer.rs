use atomic_float::AtomicF32;
use egui::{Color32, Pos2, Ui, Vec2};
use nice_plug::prelude::{util, FloatRange};
use realfft::num_complex::Complex;
use std::sync::{atomic::Ordering, Arc, Mutex};
use triple_buffer::Output;

pub type SpectrumBuffer = Arc<Mutex<Output<Vec<Complex<f32>>>>>;

fn filter_frequency_range() -> FloatRange {
    FloatRange::Skewed {
        min: 5.0,
        max: 20_000.0,
        factor: FloatRange::skew_factor(-2.5),
    }
}

// Spectrum UI credits to SolarLiner
// https://github.com/SolarLiner/valib/blob/master/plugins/abrasive/src/editor/analyzer.rs
pub fn spectrum_analyzer(
    ui: &mut Ui,
    spectrum: &SpectrumBuffer,
    sample_rate: &Arc<AtomicF32>,
) {
    let width = ui.available_rect_before_wrap().width();
    let height = 60.0_f32.max(width * 0.12);
    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

    if rect.width() == 0.0 || rect.height() == 0.0 {
        return;
    }

    let frequency_range = filter_frequency_range();
    let mut spectrum = spectrum.lock().unwrap();
    let amplitude_spectrum: Vec<f32> = spectrum.read().iter().map(|c| c.norm()).collect();
    let sr = sample_rate.load(Ordering::Relaxed);
    let fft_size = 2.0 * amplitude_spectrum.len().saturating_sub(1) as f32;

    let stroke = egui::Stroke::new(1.5, ui.visuals().text_color());
    let mut points = Vec::with_capacity(amplitude_spectrum.len());

    // Anchor the path at the bottom-left, like the original vizia version.
    points.push(Pos2::new(rect.min.x, rect.max.y));

    for (bin_index, amplitude) in amplitude_spectrum.iter().enumerate() {
        if bin_index == 0 {
            continue;
        }

        let frequency = bin_index as f32 * sr / fft_size;
        let x = frequency_range.normalize(frequency).clamp(0.0, 1.0);
        let h = ((util::gain_to_db(*amplitude) + 100.) / 120.).clamp(0.0, 1.0);

        points.push(Pos2::new(
            rect.min.x + rect.width() * x,
            rect.min.y + rect.height() * (1. - h),
        ));
    }

    if points.len() >= 2 {
        ui.painter().add(egui::Shape::line(points, stroke));
    }

    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, Color32::from_gray(180)),
        egui::StrokeKind::Inside,
    );
}
