use std::sync::Arc;

use nih_plug::prelude::{util, Editor, Vst3Plugin};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};


use crate::ChorusParams;


#[derive(Lens)]
struct Data {
    chorus_data: Arc<ChorusParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 300))
}

pub(crate) fn create(
    chorus_data: Arc<ChorusParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, 
        ViziaTheming::Custom, move |cx, _| {
            assets::register_noto_sans_light(cx);
            assets::register_noto_sans_thin(cx);

            Data {
                chorus_data: chorus_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx| {
                Label::new(cx, "CHORUS")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(30.0));
                
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Depth").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "Rate").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "Delay").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "feedback").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "Wet").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "Dry").font_size(15.0)
                        .height(Pixels(30.0));
                    }).child_top(Pixels(6.0)).row_between(Pixels(3.0));
    
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, Data::chorus_data, |params| &params.depth)
                        .height(Pixels(30.0));
                    
                        ParamSlider::new(cx, Data::chorus_data, |params| &params.rate)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::chorus_data, |params| &params.delay_ms)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::chorus_data, |params| &params.feedback)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::chorus_data, |params| &params.wet)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::chorus_data, |params| &params.dry)
                        .height(Pixels(30.0));
                    }).row_between(Pixels(3.0));
                }).col_between(Pixels(30.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

        })
}