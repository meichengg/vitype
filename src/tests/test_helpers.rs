// TestHelpers.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

use crate::{InputMethod, KeyTransformAction, TonePlacement, VitypeEngine};

pub(super) fn action(delete_count: usize, text: &str) -> KeyTransformAction {
    KeyTransformAction {
        delete_count,
        text: text.to_string(),
    }
}

pub(super) fn apply_input(input: &str) -> String {
    apply_input_with_auto_fix(input, true)
}

/// Simulates typing input through the KeyTransformer and returns the final output string.
/// - Parameters:
///   - input: The raw input string to process character by character
///   - autoFixTone: Whether to enable auto tone fixing (default: true)
/// - Returns: The transformed output string
pub(super) fn apply_input_with_auto_fix(input: &str, auto_fix_tone: bool) -> String {
    let mut engine = VitypeEngine::new();
    engine.auto_fix_tone = auto_fix_tone;
    let mut output: Vec<char> = Vec::new();

    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }

    output.into_iter().collect()
}

pub(super) fn apply_input_with_free_tone_placement(
    input: &str,
    free_tone_placement: bool,
) -> String {
    let mut engine = VitypeEngine::new();
    engine.free_tone_placement = free_tone_placement;
    let mut output: Vec<char> = Vec::new();

    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }

    output.into_iter().collect()
}

pub(super) fn apply_vni_input(input: &str) -> String {
    apply_vni_input_with_auto_fix(input, true)
}

pub(super) fn apply_vni_input_with_auto_fix(input: &str, auto_fix_tone: bool) -> String {
    let mut engine = VitypeEngine::new();
    engine.input_method = InputMethod::Vni;
    engine.auto_fix_tone = auto_fix_tone;

    let mut output: Vec<char> = Vec::new();
    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }
    output.into_iter().collect()
}

pub(super) fn apply_vni_input_with_free_tone_placement(
    input: &str,
    free_tone_placement: bool,
) -> String {
    let mut engine = VitypeEngine::new();
    engine.input_method = InputMethod::Vni;
    engine.free_tone_placement = free_tone_placement;
    let mut output: Vec<char> = Vec::new();

    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }

    output.into_iter().collect()
}

pub(super) fn apply_input_with_tone_placement(
    input: &str,
    tone_placement: TonePlacement,
) -> String {
    let mut engine = VitypeEngine::new();
    engine.tone_placement = tone_placement;
    let mut output: Vec<char> = Vec::new();

    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }

    output.into_iter().collect()
}

pub(super) fn apply_vni_input_with_tone_placement(
    input: &str,
    tone_placement: TonePlacement,
) -> String {
    let mut engine = VitypeEngine::new();
    engine.input_method = InputMethod::Vni;
    engine.tone_placement = tone_placement;
    let mut output: Vec<char> = Vec::new();

    for ch in input.chars() {
        let ch_str = ch.to_string();
        if let Some(action) = engine.process(&ch_str) {
            if action.delete_count > 0 && output.len() >= action.delete_count {
                for _ in 0..action.delete_count {
                    output.pop();
                }
            }
            output.extend(action.text.chars());
        } else {
            output.push(ch);
        }
    }

    output.into_iter().collect()
}
