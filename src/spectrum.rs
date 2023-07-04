use atomic_float::AtomicF32;
use nih_plug::{
    prelude::*,
    util::{window::multiply_with_window, StftHelper},
};
use realfft::{
    num_complex::{Complex, Complex32},
    num_traits::Zero,
    RealFftPlanner, RealToComplex,
};
use std::sync::{atomic::Ordering::Relaxed, Arc};
use triple_buffer::{Input, Output, TripleBuffer};

pub struct Spectrum {
    stft: StftHelper,
    input: Input<Vec<Complex<f32>>>,
    sample_rate: Arc<AtomicF32>,
    plan: Arc<dyn RealToComplex<f32>>,
    output_buffer: Vec<Complex32>,
    window: Vec<f32>,
}

impl Spectrum {
    pub fn new(
        sample_rate: f32,
        num_channels: usize,
        window_size: usize,
    ) -> (Self, Output<Vec<Complex<f32>>>) {
        let planner = RealFftPlanner::new().plan_fft_forward(window_size);
        let output_buffer = planner.make_output_vec();

        let (input, output) = TripleBuffer::new(&output_buffer).split();

        let this = Self {
            stft: StftHelper::new(num_channels, window_size, 0),
            input,
            sample_rate: Arc::new(AtomicF32::new(sample_rate)),
            plan: planner,
            output_buffer,
            window: util::window::hann(window_size)
                .into_iter()
                .map(|x| x / window_size as f32)
                .collect(),
        };
        (this, output)
    }

    pub fn set_sample_rate(&self, sample_rate: f32) {
        self.sample_rate.store(sample_rate, Relaxed);
    }

    pub fn process_buffer(&mut self, buffer: &Buffer) {
        self.stft.process_analyze_only(buffer, 2, |_, buffer| {
            multiply_with_window(buffer, &self.window);

            let fft_response = self.plan.process(buffer, &mut self.output_buffer);
            if fft_response.is_ok() {
                fft_response.unwrap();
            } else {
                self.output_buffer.fill(Complex32::zero());
            }
        });

        self.input.input_buffer().clone_from(&self.output_buffer);
        self.input.publish();
    }
}
