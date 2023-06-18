use atomic_float::AtomicF32;
use std::sync::{Arc, Mutex};
use realfft::{RealFftPlanner, RealToComplex, num_complex::Complex32};

pub const SPECTRUM_WINDOW_SIZE: usize = 2048;
pub const DEFAULT_SAMPLE_RATE: f32 = 41000.0;

pub struct Spectrum {
    planner: Arc<dyn RealToComplex<f32>>,
    pub sample_rate: Arc<AtomicF32>,
    pub input_buffer: Vec<f32>,
    pub output_buffer: Arc<Mutex<Vec<Complex32>>>,
}

impl Spectrum {
    pub fn new() -> Self {
        let fft_planner = RealFftPlanner::new()
            .plan_fft_forward(SPECTRUM_WINDOW_SIZE);

        let output_vec = fft_planner.make_output_vec();

        Spectrum {
            planner: fft_planner,
            input_buffer: Vec::with_capacity(SPECTRUM_WINDOW_SIZE),
            output_buffer: Arc::new(Mutex::new(output_vec)),
            sample_rate: Arc::new(AtomicF32::new(DEFAULT_SAMPLE_RATE)),
        }
    }

    pub fn compute_fft(&mut self) {
        self.planner.process(&mut self.input_buffer, &mut self.output_buffer.lock().unwrap()).unwrap();
    }
}
