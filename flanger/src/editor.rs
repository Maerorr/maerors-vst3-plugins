use std::sync::Arc;

use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};


use crate::FlangerPluginParams;

mod param_knob;
use param_knob::ParamKnob;

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
    color: #BF9FF3;
}

.header-label {
    color: #EAEEED;
}

knob {
    width: 50px;
    height: 50px;   
}

knob .track {
    background-color: #9359F3;
}

.param-label {
    color: #EAEEED;
}

.tick {
    background-color: #9359F3;
}

.main-gui {
    background-color: #1E1D1D;
}

"#;

#[derive(Lens)]
struct Data {
    phaser_data: Arc<FlangerPluginParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (350, 350))
}

pub(crate) fn create(
    phaser_data: Arc<FlangerPluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, 
        ViziaTheming::Custom, move |cx, _| {

            cx.add_fonts_mem(&[COMFORTAA_LIGHT_TTF]);
            cx.set_default_font(&[COMFORTAA]);

            cx.add_theme(STYLE);

            Data {
                phaser_data: phaser_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx: &mut Context| {
                Label::new(cx, "MAEROR'S\nFLANGER/VIBRATO")
                .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                .font_size(24.0)
                .height(Pixels(75.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .class("header-label");
                
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, Data::phaser_data, |params| &params.depth, false)
                        .height(Pixels(30.0));
                    
                        ParamKnob::new(cx, Data::phaser_data, |params| &params.rate, false)
                        .height(Pixels(30.0));

                        ParamKnob::new(cx, Data::phaser_data, |params| &params.feedback, false)
                        .height(Pixels(30.0));
                        
                    }).col_between(Pixels(15.0));

                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, Data::phaser_data, |params| &params.wet, false)
                        .height(Pixels(30.0));

                        ParamKnob::new(cx, Data::phaser_data, |params| &params.dry, false)
                        .height(Pixels(30.0));

                        ParamButton::new(cx, Data::phaser_data, |params| &params.stereo)
                        .height(Pixels(30.0))
                        .space(Stretch(1.0))
                        .bottom(Percentage(51.0));
                        
                    }).col_between(Pixels(15.0));

                }).col_between(Pixels(30.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .class("main-gui");

        })
}