use atomic_float::AtomicF32;
use egui::{Color32, Id, Mesh, Pos2, Shape, Ui, Vec2};
use nice_plug::prelude::{util, FloatRange};
use realfft::num_complex::Complex;
use std::sync::{atomic::Ordering, Arc, Mutex};
use triple_buffer::Output;

pub type SpectrumBuffer = Arc<Mutex<Output<Vec<Complex<f32>>>>>;

// Medium-speed EQ-style analyzer ballistics.
const ATTACK_TIME: f32 = 0.035;
const RELEASE_TIME: f32 = 0.250;

fn filter_frequency_range() -> FloatRange {
    FloatRange::Skewed {
        min: 5.0,
        max: 20_000.0,
        factor: FloatRange::skew_factor(-2.5),
    }
}

fn smooth_spectrum(display: &mut [f32], target: &[f32], dt: f32) {
    let attack_coeff = (-dt / ATTACK_TIME).exp();
    let release_coeff = (-dt / RELEASE_TIME).exp();

    for (display, &target) in display.iter_mut().zip(target) {
        let coeff = if target > *display {
            attack_coeff
        } else {
            release_coeff
        };
        *display = *display * coeff + target * (1.0 - coeff);
    }
}

fn fill_under_curve(painter: &egui::Painter, points: &[Pos2], fill: Color32) {
    if points.len() < 2 {
        return;
    }

    let bottom_y = points[0].y;
    let mut mesh = Mesh::default();

    for window in points.windows(2) {
        let p0 = window[0];
        let p1 = window[1];
        let b0 = Pos2::new(p0.x, bottom_y);
        let b1 = Pos2::new(p1.x, bottom_y);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(p0, fill);
        mesh.colored_vertex(p1, fill);
        mesh.colored_vertex(b1, fill);
        mesh.colored_vertex(b0, fill);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }

    painter.add(Shape::mesh(mesh));
}

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

    let dt = ui.input(|i| i.stable_dt);
    let smoothed: Vec<f32> = ui.ctx().data_mut(|data| {
        let smoothed = data.get_temp_mut_or_insert_with(Id::new("spectrum_smoothed"), || {
            amplitude_spectrum.clone()
        });
        if smoothed.len() != amplitude_spectrum.len() {
            *smoothed = amplitude_spectrum.clone();
        }
        smooth_spectrum(smoothed, &amplitude_spectrum, dt);
        smoothed.clone()
    });

    let line_color = ui.visuals().text_color();
    let fill_color = line_color.gamma_multiply(0.28);
    let stroke = egui::Stroke::new(1.5, line_color);

    let bottom_left = Pos2::new(rect.min.x, rect.max.y);
    let bottom_right = Pos2::new(rect.max.x, rect.max.y);
    let mut curve_points = Vec::with_capacity(smoothed.len());

    for (bin_index, amplitude) in smoothed.iter().enumerate() {
        if bin_index == 0 {
            continue;
        }

        let frequency = bin_index as f32 * sr / fft_size;
        let x = frequency_range.normalize(frequency).clamp(0.0, 1.0);
        let h = ((util::gain_to_db(*amplitude) + 100.) / 120.).clamp(0.0, 1.0);

        curve_points.push(Pos2::new(
            rect.min.x + rect.width() * x,
            rect.min.y + rect.height() * (1. - h),
        ));
    }

    if curve_points.is_empty() {
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, Color32::from_gray(180)),
            egui::StrokeKind::Inside,
        );
        return;
    }

    let mut fill_points = Vec::with_capacity(curve_points.len() + 2);
    fill_points.push(bottom_left);
    fill_points.extend_from_slice(&curve_points);
    fill_points.push(bottom_right);
    fill_under_curve(ui.painter(), &fill_points, fill_color);

    let mut line_points = Vec::with_capacity(curve_points.len() + 1);
    line_points.push(bottom_left);
    line_points.extend(curve_points);
    ui.painter().add(Shape::line(line_points, stroke));

    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, Color32::from_gray(180)),
        egui::StrokeKind::Inside,
    );
}
