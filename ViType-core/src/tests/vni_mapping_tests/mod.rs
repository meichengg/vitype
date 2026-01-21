#![allow(non_snake_case)]

mod foreign_consonant_tests;
mod key_transformer_tests;
mod tone_placement_tests;
mod w_transform_tests;

pub(super) use super::test_helpers::{
    action, apply_vni_input, apply_vni_input_with_auto_fix, apply_vni_input_with_tone_placement,
};
use crate::{InputMethod, VitypeEngine};

fn create_vni_engine() -> VitypeEngine {
    let mut engine = VitypeEngine::new();
    engine.input_method = InputMethod::Vni;
    engine
}
