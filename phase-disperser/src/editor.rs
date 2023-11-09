use std::sync::Arc;

use nih_plug::plugin;
use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::PluginParams;

const TOTAL_HEIGHT: u32 = 300;
const TOTAL_WIDTH: u32 = 450;

const PANEL_HEIGHT: f32 = 200.0;
const PANEL_WIDTH: f32 = 280.0;
const SMALL_TEXT_SIZE: f32 = 15.0;

const BG_COLOR: Color = Color::rgb(239, 245, 247);
const PANEL_COLOR: Color = Color::rgb(229, 239, 242);
const PANEL_TEXT_COLOR: Color = Color::rgb(202, 222, 228);
const SLIDER_FILL_COLOR: Color = Color::rgb(141, 155, 160);

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
            assets::register_noto_sans_light(cx);
            assets::register_noto_sans_thin(cx);

            Data {
                plugin_data: plugin_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx| {
                
                Label::new(cx, "Maeror's Phase Disperser")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(30.0))
                .top(Pixels(10.0))
                .bottom(Pixels(10.0));

                HStack::new(cx, |cx| {
                    // DELAY TIMING STACK
                    VStack::new(cx, |cx| {                       
                    
                        HStack::new(cx, |cx| {
                            VStack::new(cx, |cx| {
                                Label::new(cx, "Amount")
                                .font_size(SMALL_TEXT_SIZE).height(Pixels(30.0))
                                .child_space(Stretch(1.0));

                                Label::new(cx, "Frequency")
                                .child_space(Stretch(1.0))
                                .font_size(SMALL_TEXT_SIZE).height(Pixels(30.0));

                                Label::new(cx, "Spread")
                                .child_space(Stretch(1.0))
                                .font_size(SMALL_TEXT_SIZE).height(Pixels(30.0));

                                Label::new(cx, "Resonance")
                                .child_space(Stretch(1.0))
                                .font_size(SMALL_TEXT_SIZE).height(Pixels(30.0));
                            })
                            .row_between(Pixels(5.0))
                            .child_left(Pixels(10.0));
                            
                            VStack::new(cx, |cx| {
                                ParamSlider::new(cx, Data::plugin_data, |params| &params.amount)
                                .height(Pixels(30.0));

                                ParamSlider::new(cx, Data::plugin_data, |params| &params.frequency)
                                .height(Pixels(30.0));

                                ParamSlider::new(cx, Data::plugin_data, |params| &params.spread)
                                .height(Pixels(30.0));

                                ParamSlider::new(cx, Data::plugin_data, |params| &params.resonance)
                                .height(Pixels(30.0));
                            })
                            .row_between(Pixels(5.0))
                            .child_right(Pixels(10.0));
                        });
                    })
                    .row_between(Pixels(3.0))
                    .width(Pixels(PANEL_WIDTH))
                    .height(Pixels(PANEL_HEIGHT))
                    .background_color(PANEL_COLOR);


                }).col_between(Pixels(10.0));
                
            })
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .background_color(BG_COLOR);

        })
}