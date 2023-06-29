use atomic_float::AtomicF32;
use nih_plug::{
    prelude::*,
    util::{window::multiply_with_window, StftHelper},
};
use realfft::{num_complex::Complex32, num_traits::Zero, RealFftPlanner, RealToComplex};
use std::sync::{atomic::Ordering::Relaxed, Arc};
use triple_buffer::{Input, Output, TripleBuffer};

pub struct Analyzer {
    stft: StftHelper,
    input: Input<Vec<f32>>,
    scratch: Vec<f32>,
    sample_rate: Arc<AtomicF32>,
    plan: Arc<dyn RealToComplex<f32>>,
    output_buffer: Vec<Complex32>,
    window: Vec<f32>,
}

impl Analyzer {
    pub fn new(
        sample_rate: f32,
        num_channels: usize,
        window_size: usize,
    ) -> (Self, Output<Vec<f32>>) {
        let planner = RealFftPlanner::new().plan_fft_forward(window_size);
        let output_buffer = planner.make_output_vec();
        let scratch = vec![0.; window_size / 2 + 1];

        let (input, output) = TripleBuffer::new(&scratch).split();

        let this = Self {
            stft: StftHelper::new(num_channels, window_size, 0),
            input,
            scratch,
            sample_rate: Arc::new(AtomicF32::new(sample_rate)),
            plan: planner,
            output_buffer: output_buffer,
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
            if self.plan.process(buffer, &mut self.output_buffer).is_err() {
                self.output_buffer.fill(Complex32::zero());
            }

            for (scratch, fft) in self
                .scratch
                .iter_mut()
                .zip(self.output_buffer.iter_mut().map(|c| c.norm()))
            {
                let decay = f32::ln(1e-3);
                let mix = f32::exp(decay * 1024. / self.sample_rate.load(Relaxed));
                *scratch = lerp(mix, fft, *scratch).max(fft);
            }
        });
        self.input.input_buffer().clone_from(&self.scratch);
        self.input.publish();
    }
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + (b - a) * t
}
