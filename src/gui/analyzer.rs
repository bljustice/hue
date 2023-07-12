use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::{cache::BoundingBox, prelude::*, vg};
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

pub struct SpectrumAnalyzer {
    spectrum: SpectrumBuffer,
    sample_rate: Arc<AtomicF32>,
    frequency_range: FloatRange,
}

impl SpectrumAnalyzer {
    pub fn new(
        cx: &mut Context,
        spectrum: SpectrumBuffer,
        sample_rate: Arc<AtomicF32>,
    ) -> Handle<Self> {
        Self {
            spectrum,
            sample_rate,
            frequency_range: filter_frequency_range(),
        }
        .build(cx, |_cx| ())
    }

    fn draw_analyzer(&self, cx: &mut DrawContext, canvas: &mut Canvas, bounds: BoundingBox) {
        let line_width = cx.style.dpi_factor as f32 * 1.5;
        let line_paint =
            vg::Paint::color(cx.font_color().cloned().unwrap_or(Color::white()).into())
                .with_line_width(line_width);

        let mut path = vg::Path::new();

        let mut spectrum = self.spectrum.lock().unwrap();
        let amplitude_spectrum: Vec<f32> = spectrum.read().iter().map(|c| c.norm()).collect();

        let sr = self.sample_rate.load(Ordering::Relaxed);

        for (bin_index, amplitude) in amplitude_spectrum.iter().enumerate() {
            if bin_index == 0 {
                path.move_to(bounds.x - 100., bounds.y + bounds.h);
                continue;
            }

            let frequency = bin_index as f32 * sr / amplitude_spectrum.len() as f32;
            let x = self.frequency_range.normalize(frequency);

            // this changes the height of the visualized spectrum
            let h = (util::gain_to_db(*amplitude) + 100.) / 120.;

            path.line_to(bounds.x + bounds.w * x, bounds.y + bounds.h * (1. - h));
        }

        canvas.stroke_path(&mut path, line_paint);
    }
}

impl View for SpectrumAnalyzer {
    fn element(&self) -> Option<&'static str> {
        Some("spectrum-analyzer")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        self.draw_analyzer(cx, canvas, bounds);
    }
}
