use nih_plug::prelude::*;
use noise::NoiseConfig;
use std::sync::{atomic::Ordering, Arc};

mod gui;
mod config;
mod editor;
mod noise;
mod spectrum;

impl Plugin for noise::Noise {
    const NAME: &'static str = "noisegen";
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.debug.clone(),
            self.sample_rate.clone(),
            // self.analyzer_input.clone(),
            self.analyzer_output.clone(),
        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext,
    ) -> bool {
        let sr = _buffer_config.sample_rate;
        self.sample_rate.store(sr, Ordering::Relaxed);
        self.analyzer_in.set_sample_rate(sr);
        // self.analyzer_out.set_sample_rate(sr);

        true
    }

    fn reset(&mut self) {
        match self.params.noise_type.value() {
            noise::NoiseType::White => self.white.reset(),
            noise::NoiseType::Pink => self.pink.reset(),
            noise::NoiseType::Brown => self.brown.reset(),
            noise::NoiseType::Violet => self.violet.reset(),
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            let noise_sample = match self.params.noise_type.value() {
                noise::NoiseType::White => self.white.next(&mut self.rng),
                noise::NoiseType::Pink => self.pink.next(&mut self.rng),
                noise::NoiseType::Brown => self.brown.next(&mut self.rng),
                noise::NoiseType::Violet => self.violet.next(&mut self.rng),
            };

            let final_sample = noise_sample * gain;
            for sample in channel_samples {
                *sample = final_sample;
                // this is useful for debugging the noise algorithm difference equations
                if cfg!(debug_assertions) {
                    self.debug
                        .current_sample_val
                        .store(final_sample, Ordering::Relaxed);
                    self.debug
                        .sample_rate
                        .store(self.sample_rate.load(Ordering::Relaxed), Ordering::Relaxed);

                    if final_sample > self.debug.max_sample_val.load(Ordering::Relaxed) {
                        self.debug
                            .max_sample_val
                            .store(final_sample, Ordering::Relaxed);
                    } else if final_sample < self.debug.min_sample_val.load(Ordering::Relaxed) {
                        self.debug
                            .min_sample_val
                            .store(final_sample, Ordering::Relaxed);
                    }
                }
            }
        }
        if self.params.editor_state.is_open() {
            self.analyzer_in.process_buffer(buffer);
            // self.analyzer_out.process_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for noise::Noise {
    const CLAP_ID: &'static str = "";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A noise generator");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for noise::Noise {
    const VST3_CLASS_ID: [u8; 16] = *b"NoiseGenVIIIZIAA";
    const VST3_CATEGORIES: &'static str = "Fx|Noise";
}

nih_export_clap!(noise::Noise);
nih_export_vst3!(noise::Noise);
