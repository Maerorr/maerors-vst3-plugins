use nih_plug::prelude::*;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

mod delay;
mod lfo;
mod editor;
mod filter;
mod delayingallpass;
mod flanger;

const MAX_BLOCK_SIZE: usize = 64;

struct FlangerPlugin {
    params: Arc<FlangerPluginParams>,
    sample_rate: f32,
    flanger: flanger::Flanger,
}

#[derive(Params)]
struct FlangerPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "depth"]
    depth: FloatParam,

    #[id = "rate"]
    rate: FloatParam,

    #[id = "feedback"]
    feedback: FloatParam,

    #[id = "wet"]
    wet: FloatParam,

    #[id = "dry"]
    dry: FloatParam,

    #[id = "stereo"]
    stereo: BoolParam,
}

impl Default for FlangerPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(FlangerPluginParams::default()),
            sample_rate: 44100.0,
            flanger: flanger::Flanger::new(44100.0),
        }
    }
}

impl Default for FlangerPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            depth: FloatParam::new("Depth", 0.1, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),

            rate: FloatParam::new("Rate", 0.5, FloatRange::Skewed { min: 0.02, max: 10.0, factor: 0.3 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: 0.0, max: 0.999 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            wet: FloatParam::new("Wet", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            dry: FloatParam::new("Dry", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            stereo: BoolParam::new("Stereo", false),
        }
    }
}

impl Plugin for FlangerPlugin {
    const NAME: &'static str = "Maeror's Flanger/Vibrato";
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
        self.sample_rate = _buffer_config.sample_rate as f32;
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.flanger.resize_buffers(self.sample_rate);
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
            // let gain = self.params.gain.smoothed.next();

            let depth = self.params.depth.smoothed.next();
            let rate = self.params.rate.smoothed.next();
            let feedback = self.params.feedback.smoothed.next();
            let wet = self.params.wet.smoothed.next();
            let dry = self.params.dry.smoothed.next();
            let stereo = self.params.stereo.value();

            self.flanger.set_params(depth, rate, feedback, wet, dry, stereo);

            for (num, sample) in channel_samples.into_iter().enumerate() {
                if num == 0 {
                    *sample = self.flanger.process_left(*sample);
                } else {
                    *sample = self.flanger.process_right(*sample);
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

impl ClapPlugin for FlangerPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for FlangerPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"maeror___flanger";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

//nih_export_clap!(MaerorChorus);
nih_export_vst3!(FlangerPlugin);
