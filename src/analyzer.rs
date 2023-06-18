use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;
use realfft::num_complex::Complex32;
use std::sync::{Arc, Mutex};
use nih_plug::prelude::FloatRange;
use nih_plug_vizia::vizia::{vg, context::Context};
use nih_plug_vizia::vizia::prelude::*;

fn filter_frequency_range() -> FloatRange {
    FloatRange::Skewed {
        min: 5.0, // This must never reach 0
        max: 20_000.0,
        factor: FloatRange::skew_factor(-2.5),
    }
}

pub struct SpectrumAnalyzer {
   spectrum: Arc<Mutex<Vec<Complex32>>>,
   sample_rate: Arc<AtomicF32>,
   frequency_range: FloatRange,
//    x_renormalize_display: Box<dyn Fn(f32) -> f32>,
}

impl SpectrumAnalyzer {
    pub fn new<SpectrumLens, SampleRateLens>(
        cx: &mut Context,
        spectrum: SpectrumLens,
        sample_rate: SampleRateLens,
        // x_renormalize_display: impl Fn(f32) -> f32 + Clone + 'static,
    ) -> Self 
    where 
        SpectrumLens: Lens<Target = Arc<Mutex<Vec<Complex32>>>>, 
        SampleRateLens: Lens<Target = Arc<AtomicF32>>,
    {
        SpectrumAnalyzer {
            spectrum: spectrum.get(cx),
            sample_rate: sample_rate.get(cx),
            frequency_range: filter_frequency_range(),
            // x_renormalize_display: Box::new(x_renormalize_display),
        }
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

        // This spectrum buffer is written to at the end of the process function when the editor is
        // open
        let mut spectrum = self.spectrum.lock().unwrap();
        // let spectrum = spectrum.read();
        
        let nyquist = self.sample_rate.load(Ordering::Relaxed) / 2.0;

        // This skips background and border drawing
        // NOTE: We could do the same thing like in Spectral Compressor and draw part of this
        //       spectrum analyzer as a single mesh but for whatever erason the aliasing/moire
        //       pattern here doesn't look nearly as bad.
        let line_width = cx.style.dpi_factor as f32 * 1.5;
        let paint = vg::Paint::color(cx.font_color().cloned().unwrap_or_default().into())
            .with_line_width(line_width);
        let mut path = vg::Path::new();
        for (bin_idx, magnitude) in spectrum.iter().enumerate() {
            // We'll match up the bin's x-coordinate with the filter frequency parameter
            let frequency = (bin_idx as f32 / spectrum.len() as f32) * nyquist;
            // NOTE: This takes the safe-mode switch into acocunt. When it is enabled, the range is
            //       zoomed in to match the X-Y pad.
            // let t = (self.x_renormalize_display)(self.frequency_range.normalize(frequency));
            // if t <= 0.0 || t >= 1.0 {
            //     continue;
            // }

            // Scale this so that 1.0/0 dBFS magnitude is at 80% of the height, the bars begin at
            // -80 dBFS, and that the scaling is linear
            let magnitude_db = nih_plug::util::gain_to_db(magnitude.im);
            let height = ((magnitude_db + 80.0) / 100.0).clamp(0.0, 1.0);

            path.move_to(
                // bounds.x + (bounds.w * t),
                bounds.x + (bounds.w),
                bounds.y + (bounds.h * (1.0 - height)),
            );
            // path.line_to(bounds.x + (bounds.w * t), bounds.y + bounds.h);
            path.line_to(bounds.x + bounds.w, bounds.y + bounds.h);
        }

        canvas.stroke_path(&mut path, paint);
    }
}
