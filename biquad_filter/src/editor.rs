use std::sync::Arc;

use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};


use crate::FilterPluginParams;


#[derive(Lens)]
struct Data {
    filter_data: Arc<FilterPluginParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 200))
}

pub(crate) fn create(
    filter_data: Arc<FilterPluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, 
        ViziaTheming::Custom, move |cx, _| {
            assets::register_noto_sans_light(cx);
            assets::register_noto_sans_thin(cx);

            Data {
                filter_data: filter_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx| {
                Label::new(cx, "BIQUAD FILTER")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(30.0));
                
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "filter type").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "cutoff").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "resonance").font_size(15.0)
                        .height(Pixels(30.0));
    
                        Label::new(cx, "gain").font_size(15.0)
                        .height(Pixels(30.0));
    
                    }).child_top(Pixels(6.0));
    
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, Data::filter_data, |params| &params.filter_type)
                        .height(Pixels(30.0));
                    
                        ParamSlider::new(cx, Data::filter_data, |params| &params.cutoff)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::filter_data, |params| &params.resonance)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::filter_data, |params| &params.gain)
                        .height(Pixels(30.0));
                    });
                }).col_between(Pixels(30.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

        })
}