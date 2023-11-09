use filter::FilterType;
use nih_plug::prelude::*;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

mod delay;
mod lfo;
mod editor;
mod filter;
mod delayingallpass;
mod phaser;

const MAX_BLOCK_SIZE: usize = 64;

struct PhaserPlugin {
    params: Arc<PhaserPluginParams>,
    phaser: phaser::Phaser,
    output_hpf: filter::BiquadFilter,
    sample_rate: f32,
}

#[derive(Params)]
struct PhaserPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "depth"]
    depth: FloatParam,

    #[id = "rate"]
    rate: FloatParam,

    #[id = "feedback"]
    feedback: FloatParam,

    #[id = "stages"]
    stages: IntParam,

    #[id = "offset"]
    offset: FloatParam,

    #[id = "intensity"]
    intensity: FloatParam,
}

impl Default for PhaserPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(PhaserPluginParams::default()),
            phaser: phaser::Phaser::new(44100.0),
            sample_rate: 44100.0,
            output_hpf: filter::BiquadFilter::new(),
        }
    }
}

pub fn v2s_i32() -> Arc<dyn Fn(i32) -> String + Send + Sync> {
    Arc::new(move |value| {
        format!("{}", value)
    })
}

impl Default for PhaserPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            depth: FloatParam::new("Depth", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            rate: FloatParam::new("Rate", 0.5, FloatRange::Skewed { min: 0.02, max: 10.0, factor: 0.3 })
            .with_unit("Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: 0.0, max: 0.9 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            stages: IntParam::new("Stages", 3, IntRange::Linear { min: 1, max: 3 })
            .with_value_to_string(v2s_i32()),

            offset: FloatParam::new("Offset", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            intensity: FloatParam::new("Intensity", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for PhaserPlugin {
    const NAME: &'static str = "tsk phaser";
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
        self.sample_rate = _buffer_config.sample_rate as f32;
        self.output_hpf.set_sample_rate(self.sample_rate);
        self.output_hpf.second_order_hpf_coefficients(self.sample_rate, 25.0, 0.8);
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.phaser.resize_buffers(self.sample_rate);
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
            let stages = self.params.stages.value();
            let offset = self.params.offset.smoothed.next();
            let intensity = self.params.intensity.smoothed.next();

            self.phaser.set_params(rate, depth, stages as usize, offset, feedback, intensity);

            for (num, sample) in channel_samples.into_iter().enumerate() {
                if num == 0 {
                    *sample = self.phaser.process_left(*sample);
                    *sample = self.output_hpf.process_left(*sample);
                } else {
                    *sample = self.phaser.process_right(*sample);
                    *sample = self.output_hpf.process_left(*sample);
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

impl ClapPlugin for PhaserPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for PhaserPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"tsk__PhaserRvdH.";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

//nih_export_clap!(MaerorChorus);
nih_export_vst3!(PhaserPlugin);
