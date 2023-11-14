use disperser::Disperser;
use nih_plug::prelude::*;

use std::{sync::{Arc}, collections::VecDeque, env};

use nih_plug_vizia::ViziaState;

mod editor;
mod filter;
mod disperser;


pub struct EffectPlugin {
    params: Arc<PluginParams>,
    disperser: Disperser,
    sample_rate: f32,
}

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "frequency"]
    frequency: FloatParam,

    #[id = "spread"]
    spread: FloatParam,

    #[id = "resonance"]
    resonance: FloatParam,

    #[id = "amount"]
    amount: IntParam,
}

impl Default for EffectPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),
            disperser: Disperser::new(),
            sample_rate: 44100.0,
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            frequency: FloatParam::new("Frequency", 1000.0, FloatRange::Linear { min: 500.0, max: 12000.0 })
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz())
            .with_smoother(SmoothingStyle::Linear(1.0)),

            spread: FloatParam::new("Spread", 0.1, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage())
            .with_smoother(SmoothingStyle::Linear(1.0)),

            resonance: FloatParam::new("Resonance", 0.707, FloatRange::Linear { min: 0.707, max: 10.0 })
            .with_value_to_string(formatters::v2s_f32_rounded(2))
            .with_smoother(SmoothingStyle::Linear(1.0)),
            
            amount: IntParam::new("Amount", 100, IntRange::Linear { min: 1, max: 200 })
        }
    }
}

impl Plugin for EffectPlugin {
    const NAME: &'static str = "Maeror's Phase Disperser";
    const VENDOR: &'static str = "Hubert Łabuda";
    const URL: &'static str = "https://www.linkedin.com/in/hubert-%C5%82abuda/";
    const EMAIL: &'static str = "none";
    const VERSION: &'static str = "none";
    
    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.

        self.sample_rate = _buffer_config.sample_rate;
        self.disperser.resize_buffers(self.sample_rate, 500.0, 0.707, 200);

        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // In current configuration this function iterates as follows:
        // 1. outer loop iterates block-size times
        // 2. inner loop iterates channel-size times. 

        for (i, channel_samples) in buffer.iter_samples().enumerate() {
            // Smoothing is optionally built into the parameters themselves

            let frequency = self.params.frequency.smoothed.next();
            let spread = self.params.spread.smoothed.next();
            let q = self.params.resonance.smoothed.next();
            let amount = self.params.amount.value();

            self.disperser.set_params(frequency, q, spread, amount as u32);
            
            for (num, sample) in channel_samples.into_iter().enumerate() {
                // processing
                if num == 0 {
                    *sample = self.disperser.process_left(*sample);
                } else {
                    *sample = self.disperser.process_right(*sample);
                }
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }
}

impl ClapPlugin for EffectPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for EffectPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"maeror-dsprs-vst";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx];
}


nih_export_vst3!(EffectPlugin);
