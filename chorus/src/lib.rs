use chorus::Chorus;
use nih_plug::prelude::*;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

mod delay;
mod lfo;
mod editor;
mod chorus;
mod filter;

struct ChorusPlugin {
    params: Arc<ChorusParams>,
    sample_rate: f32,
    chorus: chorus::Chorus,
    output_hpf: filter::BiquadFilter,
}

#[derive(Params)]
struct ChorusParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    // parameters for chorus
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "delay_ms"]
    pub delay_ms: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "wet"]
    pub wet: FloatParam,
    #[id = "dry"]
    pub dry: FloatParam,
}

impl Default for ChorusPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ChorusParams::default()),
            sample_rate: 44100.0,
            chorus: Chorus::new(44100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            output_hpf: filter::BiquadFilter::new(),
        }
    }
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            // implement depth, rate, delay_ms, feedback, wet parameters
            // DEPTH
            depth: FloatParam::new("Depth", 5.0, FloatRange::Linear { min: 0.0, max: 25.0 })
            .with_unit("ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            
            // RATE
            rate: FloatParam::new("Rate", 0.5, FloatRange::Skewed { min: 0.02, max: 10.0, factor: 0.3 })
            .with_unit("Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // DELAY
            delay_ms: FloatParam::new("Delay", 15.0, FloatRange::Linear { min: 0.1, max: 50.0 })
            .with_unit("ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // FEEDBACK
            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: 0.0, max: 0.999 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            // WET
            wet: FloatParam::new("Wet", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // DRY
            dry: FloatParam::new("Dry", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for ChorusPlugin {
    const NAME: &'static str = "tsk_chorus";
    const VENDOR: &'static str = "236587 & 236598";
    const URL: &'static str = "none";
    const EMAIL: &'static str = "none";
    const VERSION: &'static str = "test";

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
        self.sample_rate = 2.0 * _buffer_config.sample_rate as f32;

        self.chorus.resize_buffers(self.sample_rate);
        self.output_hpf.set_sample_rate(_buffer_config.sample_rate as f32);
        self.output_hpf.coefficients(filter::FilterType::HighPass2, 25.0, 0.707, 1.0);
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
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

            let depth = self.params.depth.smoothed.next();
            let rate = self.params.rate.smoothed.next();
            let delay_ms = self.params.delay_ms.smoothed.next();
            let feedback = self.params.feedback.smoothed.next();
            let wet = self.params.wet.smoothed.next();
            let dry = self.params.dry.smoothed.next();

            self.chorus.set_params(self.sample_rate, delay_ms, feedback, depth, rate, wet, dry);

            for (num, sample) in channel_samples.into_iter().enumerate() {
                if num == 0 {
                    *sample = self.chorus.process_left(*sample);
                    *sample = self.output_hpf.process_left(*sample);
                } else {
                    *sample = self.chorus.process_right(*sample);
                    *sample = self.output_hpf.process_right(*sample);
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

impl ClapPlugin for ChorusPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for ChorusPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"tsk__ChorusRvdH.";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Delay, Vst3SubCategory::Modulation, Vst3SubCategory::Fx];
}

//nih_export_clap!(Chorus);
nih_export_vst3!(ChorusPlugin);
