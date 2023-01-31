use atomic_float::AtomicF32;
use std::sync::Arc;

#[derive(Clone)]
pub struct Debug {
    pub current_sample_val: Arc<AtomicF32>,
    pub max_sample_val: Arc<AtomicF32>,
    pub min_sample_val: Arc<AtomicF32>,    
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            current_sample_val: Arc::new(AtomicF32::new(0.0)),
            max_sample_val: Arc::new(AtomicF32::new(0.0)),
            min_sample_val: Arc::new(AtomicF32::new(0.0)),
        }
    }
}
