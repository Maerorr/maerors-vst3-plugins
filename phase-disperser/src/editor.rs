use std::sync::Arc;

use nih_plug::plugin;
use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::PluginParams;

use self::param_knob::ParamKnob;

mod param_knob;

const TOTAL_HEIGHT: u32 = 350;
const TOTAL_WIDTH: u32 = 300;

const PANEL_HEIGHT: f32 = 200.0;
const PANEL_WIDTH: f32 = 280.0;
const SMALL_TEXT_SIZE: f32 = 15.0;

const BG_COLOR: Color = Color::rgb(239, 245, 247);
const PANEL_COLOR: Color = Color::rgb(229, 239, 242);
const PANEL_TEXT_COLOR: Color = Color::rgb(202, 222, 228);
const SLIDER_FILL_COLOR: Color = Color::rgb(141, 155, 160);

pub const COMFORTAA_LIGHT_TTF: &[u8] = include_bytes!("../res/Comfortaa-Light.ttf");
pub const COMFORTAA: &str = "Comfortaa";

const STYLE: &str = r#"
.param_knob {
    width: 100px;
    height: 100px;
}

label {
    child-space: 1s;
    font-size: 18;
    color: #C42626;
}

.header-label {
    color: #EAEEED;
}

knob {
    width: 50px;
    height: 50px;   
}

knob .track {
    background-color: #C42626;
}

.param-label {
    color: #EAEEED;
}

.tick {
    background-color: #C42626;
}

.main-gui {
    background-color: #1E1D1D;
}

"#;

#[derive(Lens)]
struct Data {
    plugin_data: Arc<PluginParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (TOTAL_WIDTH, TOTAL_HEIGHT))
}

pub(crate) fn create(
    plugin_data: Arc<PluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, 
        ViziaTheming::Custom, move |cx, _| {

            cx.add_fonts_mem(&[COMFORTAA_LIGHT_TTF]);
            cx.set_default_font(&[COMFORTAA]);

            cx.add_theme(STYLE);

            Data {
                plugin_data: plugin_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx| {

                Label::new(cx, "MAEROR'S\nPHASE DISPERSER")
                .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                .font_size(24.0)
                .height(Pixels(75.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .class("header-label");

                HStack::new(cx, |cx| {
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.amount, false)
                    .height(Pixels(30.0));
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.frequency, false)
                    .height(Pixels(30.0));
                }).col_between(Pixels(40.0));

                HStack::new(cx, |cx| {
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.spread, false)
                    .height(Pixels(30.0));
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.resonance, false)
                    .height(Pixels(30.0));
                }).col_between(Pixels(40.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .class("main-gui");
        })
}