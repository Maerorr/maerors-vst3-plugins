
use nih_plug::prelude::*;
use maeror_chorus::ChorusPlugin;

fn main() {
    nih_export_standalone::<ChorusPlugin>();
}