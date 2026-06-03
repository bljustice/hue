use atomic_float::AtomicF32;
use nice_plug::prelude::*;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::Duration;
use triple_buffer::Output;
use vizia_plug::vizia::{layout::BoundingBox, prelude::*, vg};
use realfft::num_complex::Complex;

pub type SpectrumBuffer = Arc<Mutex<Output<Vec<Complex<f32>>>>>;

fn filter_frequency_range() -> FloatRange {
    FloatRange::Skewed {
        min: 5.0,
        max: 20_000.0,
        factor: FloatRange::skew_factor(-2.5),
    }
}

// Spectrum UI object credits to SolarLiner
// It was reworked below to fit my use case
// https://github.com/SolarLiner/valib/blob/master/plugins/abrasive/src/editor/analyzer.rs
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
        let redraw_tick = SyncSignal::new(0u32);
        let tick = redraw_tick.clone();
        let timer = cx.add_timer(Duration::from_millis(33), None, move |_cx, action| {
            if matches!(action, TimerAction::Tick(_)) {
                tick.update(|count| *count += 1);
            }
        });
        cx.start_timer(timer);

        Self {
            spectrum,
            sample_rate,
            frequency_range: filter_frequency_range(),
        }
        .build(cx, |_cx| ())
        .bind(redraw_tick, |mut handle| handle.needs_redraw())
    }

    fn draw_analyzer(&self, cx: &mut DrawContext, canvas: &Canvas, bounds: BoundingBox) {
        let line_width = cx.scale_factor() * 1.5;
        let mut paint = vg::Paint::default();
        paint.set_color(cx.font_color());
        paint.set_style(vg::PaintStyle::Stroke);
        paint.set_stroke_width(line_width);
        paint.set_anti_alias(true);

        let mut path = vg::PathBuilder::new();

        let mut spectrum = self.spectrum.lock().unwrap();
        let amplitude_spectrum: Vec<f32> = spectrum.read().iter().map(|c| c.norm()).collect();

        let sr = self.sample_rate.load(Ordering::Relaxed);

        for (bin_index, amplitude) in amplitude_spectrum.iter().enumerate() {
            if bin_index == 0 {
                path.move_to((bounds.x - 100., bounds.y + bounds.h));
                continue;
            }

            let frequency = bin_index as f32 * sr / amplitude_spectrum.len() as f32;
            let x = self.frequency_range.normalize(frequency);

            // this changes the height of the visualized spectrum
            let h = (util::gain_to_db(*amplitude) + 100.) / 120.;

            path.line_to((bounds.x + bounds.w * x, bounds.y + bounds.h * (1. - h)));
        }

        let path = path.detach();
        canvas.draw_path(&path, &paint);
    }
}

impl View for SpectrumAnalyzer {
    fn element(&self) -> Option<&'static str> {
        Some("spectrum-analyzer")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        self.draw_analyzer(cx, canvas, bounds);
    }
}
