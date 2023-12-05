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

const MAX_BLOCK_SIZE: usize = 32;

struct ScratchBuffer {
    rate: [f32; MAX_BLOCK_SIZE],
    depth: [f32; MAX_BLOCK_SIZE],
    delay: [f32; MAX_BLOCK_SIZE],
    feedback: [f32; MAX_BLOCK_SIZE],
    mix: [f32; MAX_BLOCK_SIZE],
}

impl Default for ScratchBuffer {
    fn default() -> Self {
        Self {
            rate: [0.0; MAX_BLOCK_SIZE],
            depth: [0.0; MAX_BLOCK_SIZE],
            delay: [0.0; MAX_BLOCK_SIZE],
            feedback: [0.0; MAX_BLOCK_SIZE],
            mix: [0.0; MAX_BLOCK_SIZE],
        }
    }
}

pub struct ChorusPlugin {
    params: Arc<ChorusParams>,
    sample_rate: f32,
    chorus: chorus::Chorus,
    output_hpf: filter::BiquadFilter,
    scr_buf: ScratchBuffer,
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
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "mono"]
    pub mono: BoolParam,

    #[id = "credits"]
    pub credits: BoolParam,
}

impl Default for ChorusPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ChorusParams::default()),
            sample_rate: 44100.0,
            chorus: Chorus::new(44100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            output_hpf: filter::BiquadFilter::new(),
            scr_buf: ScratchBuffer::default(),
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
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            
            // RATE
            rate: FloatParam::new("Rate", 0.5, FloatRange::Skewed { min: 0.02, max: 10.0, factor: 0.3 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // DELAY
            delay_ms: FloatParam::new("Delay", 15.0, FloatRange::Linear { min: 0.1, max: 50.0 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // FEEDBACK
            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: 0.0, max: 0.999 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            // WET
            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(15.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // MONO
            mono: BoolParam::new("Mono", false),


            // CREDITS
            credits: BoolParam::new("Credits", false),
        }
    }
}

impl Plugin for ChorusPlugin {
    const NAME: &'static str = "Maeror's Chorus";
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

        for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            let block_len = block.samples();

            let rate = &mut self.scr_buf.rate;
            self.params.rate.smoothed.next_block(rate, block_len);

            let depth = &mut self.scr_buf.depth;
            self.params.depth.smoothed.next_block(depth, block_len);

            let delay = &mut self.scr_buf.delay;
            self.params.delay_ms.smoothed.next_block(delay, block_len);

            let feedback = &mut self.scr_buf.feedback;
            self.params.feedback.smoothed.next_block(feedback, block_len);

            let mix = &mut self.scr_buf.mix;
            self.params.mix.smoothed.next_block(mix, block_len);

            let mono = self.params.mono.value();

            for (channel_idx, block_channel) in block.into_iter().enumerate() {

                for (sample_idx, sample) in block_channel.into_iter().enumerate() {

                    self.chorus.set_params(
                        self.sample_rate, 
                        unsafe { *delay.get_unchecked(sample_idx)}, 
                        unsafe { *feedback.get_unchecked(sample_idx)},
                        unsafe { *depth.get_unchecked(sample_idx)},
                        unsafe { *rate.get_unchecked(sample_idx)},
                        unsafe { *mix.get_unchecked(sample_idx)},
                        mono,);
                    
                    if channel_idx == 0 {
                        *sample = self.chorus.process_left(*sample);
                        *sample = self.output_hpf.process_left(*sample);
                    } else {
                        *sample = self.chorus.process_right(*sample);
                        *sample = self.output_hpf.process_right(*sample);
                    }
                    self.chorus.update_modulators();
                }
            }
        }
        // for (i, channel_samples) in buffer.iter_samples().enumerate() {

        //     let depth = self.params.depth.smoothed.next();
        //     let rate = self.params.rate.smoothed.next();
        //     let delay_ms = self.params.delay_ms.smoothed.next();
        //     let feedback = self.params.feedback.smoothed.next();
        //     let wet = self.params.wet.smoothed.next();
        //     let dry = self.params.dry.smoothed.next();

        //     self.chorus.set_params(self.sample_rate, delay_ms, feedback, depth, rate, wet, dry);

        //     for (num, sample) in channel_samples.into_iter().enumerate() {
        //         if num == 0 {
        //             *sample = self.chorus.process_left(*sample);
        //             *sample = self.output_hpf.process_left(*sample);
        //         } else {
        //             *sample = self.chorus.process_right(*sample);
        //             *sample = self.output_hpf.process_right(*sample);
        //         }
        //     }
        // }

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
    const VST3_CLASS_ID: [u8; 16] = *b"maeror____Chorus";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Delay, Vst3SubCategory::Modulation, Vst3SubCategory::Fx];
}

//nih_export_clap!(Chorus);
nih_export_vst3!(ChorusPlugin);