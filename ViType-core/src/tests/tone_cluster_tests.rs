#![allow(non_snake_case)]

use super::test_helpers::{
    apply_input, apply_input_with_free_tone_placement, apply_vni_input,
    apply_vni_input_with_free_tone_placement,
};

#[test]
fn testToneBlockedForInvalidClustersTelex() {
    assert_eq!(apply_input("aes"), "aes");
    assert_eq!(apply_input("aois"), "aois");
}

#[test]
fn testToneAllowedForValidClustersTelex() {
    assert_eq!(apply_input("euf"), "èu");
    assert_eq!(apply_input("uois"), "uói");
}

#[test]
fn testFreeTonePlacementAllowsInvalidClustersTelex() {
    assert_eq!(apply_input_with_free_tone_placement("aes", true), "áe");
    assert_eq!(apply_input_with_free_tone_placement("muafi", true), "muài");
}

#[test]
fn testToneBlockedForInvalidClustersVni() {
    assert_eq!(apply_vni_input("ae1"), "ae1");
    assert_eq!(apply_vni_input("aoi1"), "aoi1");
}

#[test]
fn testToneAllowedForValidClustersVni() {
    assert_eq!(apply_vni_input("eu2"), "èu");
    assert_eq!(apply_vni_input("uoi1"), "uói");
}

#[test]
fn testFreeTonePlacementAllowsInvalidClustersVni() {
    assert_eq!(apply_vni_input_with_free_tone_placement("ae1", true), "áe");
    assert_eq!(
        apply_vni_input_with_free_tone_placement("mua2i", true),
        "muài"
    );
}
