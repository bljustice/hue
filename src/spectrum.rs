use std::sync::Arc;
use nih_plug::prelude::Buffer;
use realfft::{RealFftPlanner, RealToComplex, num_complex::Complex32};

pub const SPECTRUM_WINDOW_SIZE: usize = 2048;

pub struct Spectrum {
    planner: Arc<dyn RealToComplex<f32>>,
    spectrum_output: Vec<Complex32>,
}

impl Spectrum {
    pub fn new() -> Self {
        let fft_planner = RealFftPlanner::new()
            .plan_fft_forward(SPECTRUM_WINDOW_SIZE);

        Spectrum {
            planner: fft_planner,
            spectrum_output: fft_planner.make_output_vec(),
        }
    }

    pub fn compute_fft(&mut self, buffer: &Buffer) {
        self.planner.process(buffer, &mut self.spectrum_output).unwrap();
        return self.spectrum_output;
    }
}
