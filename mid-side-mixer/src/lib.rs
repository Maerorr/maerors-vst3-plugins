use nih_plug::prelude::*;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use mid_side_mixer::MidSideMixer;

mod mid_side_mixer;
mod editor;

struct EffectPlugin {
    params: Arc<PluginParams>,

    midside_mixer: MidSideMixer,
}

#[derive(Params)]
struct PluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "mid-mix"]
    mid_mix: FloatParam,

    #[id = "side-mix"]
    side_mix: FloatParam,
}

impl Default for EffectPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),

            midside_mixer: MidSideMixer::new(),
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            mid_mix: FloatParam::new("Mid Mix", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            side_mix: FloatParam::new("Side Mix", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(1.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for EffectPlugin {
    const NAME: &'static str = "Maeror's Mid-Side Mixer";
    const VENDOR: &'static str = "Maeror";
    const URL: &'static str = "";
    const EMAIL: &'static str = "none";
    const VERSION: &'static str = "0.0.1";

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

        for mut channel_samples in buffer.iter_samples() {
            let mid_mix = self.params.mid_mix.smoothed.next();
            let side_mix = self.params.side_mix.smoothed.next();
            self.midside_mixer.set_params(mid_mix, side_mix);

            unsafe {
                let l = channel_samples.get_unchecked_mut(0).clone();
                let r = channel_samples.get_unchecked_mut(1).clone();

                let (l_out, r_out) = self.midside_mixer.process(l, r);

                *channel_samples.get_unchecked_mut(0) = l_out;
                *channel_samples.get_unchecked_mut(1) = r_out;
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
    const VST3_CLASS_ID: [u8; 16] = *b"maeror-midsidemx";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx];
}


nih_export_vst3!(EffectPlugin);
