use atomic_float::AtomicF32;
use std::sync::{atomic::Ordering::Relaxed, Arc};

#[derive(Clone)]
pub struct Debug {
    pub current_sample_val: Arc<AtomicF32>,
    pub max_sample_val: Arc<AtomicF32>,
    pub min_sample_val: Arc<AtomicF32>,
    pub sample_rate: Arc<AtomicF32>,
    pub output_buffer: Arc<AtomicF32>,
    pub mix: Arc<AtomicF32>,
    pub gain: Arc<AtomicF32>,
    pub envelope: Arc<AtomicF32>,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            current_sample_val: Arc::new(AtomicF32::new(0.0)),
            max_sample_val: Arc::new(AtomicF32::new(0.0)),
            min_sample_val: Arc::new(AtomicF32::new(0.0)),
            sample_rate: Arc::new(AtomicF32::new(0.0)),
            output_buffer: Arc::new(AtomicF32::new(0.0)),
            mix: Arc::new(AtomicF32::new(0.5)),
            gain: Arc::new(AtomicF32::new(0.0)),
            envelope: Arc::new(AtomicF32::new(0.)),
        }
    }
}

impl Debug {
    pub fn update(&mut self, sample_value: f32, sample_rate: f32, mix_level: f32, gain_level: f32, envelope: f32) {
        self.current_sample_val.store(sample_value, Relaxed);
        self.sample_rate.store(sample_rate, Relaxed);
        self.mix.store(mix_level, Relaxed);
        self.gain.store(gain_level, Relaxed);
        self.envelope.store(envelope, Relaxed);

        if sample_value > self.max_sample_val.load(Relaxed) {
            self.max_sample_val.store(sample_value, Relaxed);
        } else if sample_value < self.min_sample_val.load(Relaxed) {
            self.min_sample_val.store(sample_value, Relaxed);
        }
    }
}
