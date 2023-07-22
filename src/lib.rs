use nih_plug::prelude::*;
use noise::NoiseConfig;
use params::NoiseType;
use std::sync::{atomic::Ordering, Arc};

mod config;
mod editor;
mod gui;
mod noise;
mod params;
mod spectrum;

impl Plugin for noise::Noise {
    const NAME: &'static str = "hue";
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    type BackgroundTask = ();
    type SysExMessage = ();

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.debug.clone(),
            self.sample_rate.clone(),
            self.spectrum_output_buffer.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = _buffer_config.sample_rate;
        self.sample_rate.store(sr, Ordering::Relaxed);
        self.spectrum.set_sample_rate(sr);

        true
    }

    fn reset(&mut self) {
        match self.params.noise_type.value() {
            NoiseType::White => self.white.reset(),
            NoiseType::Pink => self.pink.reset(),
            NoiseType::Brown => self.brown.reset(),
            NoiseType::Violet => self.violet.reset(),
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            let mix_level = self.params.mix.smoothed.next();

            let noise_sample = match self.params.noise_type.value() {
                NoiseType::White => self
                    .white
                    .next(&self.params.white_noise_distribution.value(), &mut self.rng),
                NoiseType::Pink => self
                    .pink
                    .next(&self.params.white_noise_distribution.value(), &mut self.rng),
                NoiseType::Brown => self
                    .brown
                    .next(&self.params.white_noise_distribution.value(), &mut self.rng),
                NoiseType::Violet => self
                    .violet
                    .next(&self.params.white_noise_distribution.value(), &mut self.rng),
            };

            let final_sample = noise_sample * gain * mix_level;
            for sample in channel_samples {
                *sample = final_sample + (*sample * (1.0 - mix_level));

                self.debug.update(
                    *sample,
                    self.sample_rate.load(Ordering::Relaxed),
                    mix_level,
                    gain,
                );
            }
        }
        if self.params.editor_state.is_open() {
            self.spectrum.process_buffer(buffer);
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
    const VST3_CLASS_ID: [u8; 16] = *b"huenoise........";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Tools, Vst3SubCategory::Fx];
}

nih_export_clap!(noise::Noise);
nih_export_vst3!(noise::Noise);
