use std::sync::Arc;

use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::PluginParams;


use self::param_knob::ParamKnob;

mod param_knob;

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
    color: #9DEEDA;
}

.header-label {
    color: #EAEEED;
}

knob {
    width: 50px;
    height: 50px;   
}

knob .track {
    background-color: #54deb2;
}

.param-label {
    color: #EAEEED;
}

.tick {
    background-color: #54deb2;
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
    ViziaState::new(|| (350, 350))
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
                Label::new(cx, "MAEROR'S Mid/Side Mixer")
                .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                .font_size(24.0)
                .height(Pixels(75.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .class("header-label");
                
                HStack::new(cx, |cx| {
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.mid_mix, false);
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.side_mix, false);
                }).col_between(Pixels(15.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .class("main-gui");

        })
}