// VNI input method tests

use crate::{InputMethod, KeyTransformAction, VitypeEngine};

fn action(delete_count: usize, text: &str) -> KeyTransformAction {
    KeyTransformAction {
        delete_count,
        text: text.to_string(),
    }
}

fn create_vni_engine() -> VitypeEngine {
    let mut engine = VitypeEngine::new();
    engine.input_method = InputMethod::Vni;
    engine
}

fn apply_vni_input(input: &str) -> String {
    apply_vni_input_with_auto_fix(input, true)
}

fn apply_vni_input_with_auto_fix(input: &str, auto_fix_tone: bool) -> String {
    let mut engine = create_vni_engine();
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

fn apply_key(engine: &mut VitypeEngine, output: &mut Vec<char>, ch: char) {
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

fn backspace(engine: &mut VitypeEngine, output: &mut Vec<char>) {
    engine.delete_last_character();
    output.pop();
}

// ==================== Basic Consonant Tests ====================

#[test]
fn test_vni_d9_to_d_stroke() {
    assert_eq!(apply_vni_input("d9"), "đ");
}

#[test]
fn test_vni_d9_uppercase() {
    assert_eq!(apply_vni_input("D9"), "Đ");
}

#[test]
fn test_vni_d9_free_transform() {
    assert_eq!(apply_vni_input("di9"), "đi");
}

#[test]
fn test_vni_d9_free_transform_with_vowels() {
    assert_eq!(apply_vni_input("dai9"), "đai");
}

// ==================== Basic Vowel Transform Tests ====================

#[test]
fn test_vni_a6_circumflex() {
    assert_eq!(apply_vni_input("a6"), "â");
}

#[test]
fn test_vni_e6_circumflex() {
    assert_eq!(apply_vni_input("e6"), "ê");
}

#[test]
fn test_vni_o6_circumflex() {
    assert_eq!(apply_vni_input("o6"), "ô");
}

#[test]
fn test_vni_a6_uppercase() {
    assert_eq!(apply_vni_input("A6"), "Â");
}

#[test]
fn test_vni_o7_horn() {
    assert_eq!(apply_vni_input("o7"), "ơ");
}

#[test]
fn test_vni_u7_horn() {
    assert_eq!(apply_vni_input("u7"), "ư");
}

#[test]
fn test_vni_o7_uppercase() {
    assert_eq!(apply_vni_input("O7"), "Ơ");
}

#[test]
fn test_vni_a8_breve() {
    assert_eq!(apply_vni_input("a8"), "ă");
}

#[test]
fn test_vni_a8_uppercase() {
    assert_eq!(apply_vni_input("A8"), "Ă");
}

// ==================== Free Vowel Transform Tests ====================

#[test]
fn test_vni_free_transform_circumflex() {
    assert_eq!(apply_vni_input("that6"), "thât");
}

#[test]
fn test_vni_free_transform_horn() {
    assert_eq!(apply_vni_input("thot7"), "thơt");
}

#[test]
fn test_vni_free_transform_breve() {
    assert_eq!(apply_vni_input("nam8"), "năm");
}

// ==================== Transform Override Tests ====================

#[test]
fn test_vni_o67_override_circumflex_to_horn() {
    assert_eq!(apply_vni_input("o67"), "ơ");
}

#[test]
fn test_vni_o76_override_horn_to_circumflex() {
    assert_eq!(apply_vni_input("o76"), "ô");
}

#[test]
fn test_vni_a68_override_circumflex_to_breve() {
    assert_eq!(apply_vni_input("a68"), "ă");
}

#[test]
fn test_vni_a86_override_breve_to_circumflex() {
    assert_eq!(apply_vni_input("a86"), "â");
}

#[test]
fn test_vni_a67_no_override_literal() {
    // '7' can't transform 'â', outputs literal
    assert_eq!(apply_vni_input("a67"), "â7");
}

#[test]
fn test_vni_e67_no_override_literal() {
    // '7' can't transform 'ê', outputs literal
    assert_eq!(apply_vni_input("e67"), "ê7");
}

// ==================== Tone Tests ====================

#[test]
fn test_vni_a1_sac_tone() {
    assert_eq!(apply_vni_input("a1"), "á");
}

#[test]
fn test_vni_a2_huyen_tone() {
    assert_eq!(apply_vni_input("a2"), "à");
}

#[test]
fn test_vni_a3_hoi_tone() {
    assert_eq!(apply_vni_input("a3"), "ả");
}

#[test]
fn test_vni_a4_nga_tone() {
    assert_eq!(apply_vni_input("a4"), "ã");
}

#[test]
fn test_vni_a5_nang_tone() {
    assert_eq!(apply_vni_input("a5"), "ạ");
}

#[test]
fn test_vni_tone_on_transformed_vowel() {
    assert_eq!(apply_vni_input("a61"), "ấ"); // â + sắc
    assert_eq!(apply_vni_input("a82"), "ằ"); // ă + huyền
    assert_eq!(apply_vni_input("o73"), "ở"); // ơ + hỏi
}

#[test]
fn test_vni_a0_remove_tone() {
    assert_eq!(apply_vni_input("a10"), "a");
    assert_eq!(apply_vni_input("a20"), "a");
}

#[test]
fn test_vni_tone_replacement() {
    // First tone, then replace with different tone
    assert_eq!(apply_vni_input("a12"), "à"); // á → à
    assert_eq!(apply_vni_input("a13"), "ả"); // á → ả
}

// ==================== Escape Tests ====================

#[test]
fn test_vni_d99_escape() {
    assert_eq!(apply_vni_input("d99"), "d9");
}

#[test]
fn test_vni_a66_escape() {
    assert_eq!(apply_vni_input("a66"), "a6");
}

#[test]
fn test_vni_o77_escape() {
    assert_eq!(apply_vni_input("o77"), "o7");
}

#[test]
fn test_vni_a88_escape() {
    assert_eq!(apply_vni_input("a88"), "a8");
}

#[test]
fn test_vni_ta11_escape() {
    assert_eq!(apply_vni_input("ta11"), "ta1");
}

#[test]
fn test_vni_ta22_escape() {
    assert_eq!(apply_vni_input("ta22"), "ta2");
}

#[test]
fn test_vni_repeated_escape() {
    assert_eq!(apply_vni_input("a666"), "a66");
    assert_eq!(apply_vni_input("d999"), "d99");
    assert_eq!(apply_vni_input("ta111"), "ta11");
}

// ==================== Compound Transform Tests ====================

#[test]
fn test_vni_uo7_compound() {
    assert_eq!(apply_vni_input("uo7"), "ươ");
}

#[test]
fn test_vni_uo7_uppercase() {
    assert_eq!(apply_vni_input("UO7"), "ƯƠ");
    assert_eq!(apply_vni_input("Uo7"), "Ươ");
}

#[test]
fn test_vni_uu7_compound() {
    assert_eq!(apply_vni_input("huu7"), "hưu");
}

#[test]
fn test_vni_ua7_compound() {
    assert_eq!(apply_vni_input("mua7"), "mưa");
}

#[test]
fn test_vni_ou7_compound() {
    assert_eq!(apply_vni_input("hou7"), "hươ");
}

#[test]
fn test_vni_uou7_compound() {
    assert_eq!(apply_vni_input("huou7"), "hươu");
}

#[test]
fn test_vni_uo7_escape() {
    assert_eq!(apply_vni_input("uo77"), "uo7");
}

#[test]
fn test_vni_qu_cluster_no_compound() {
    // qu cluster: u is part of consonant, so quo7 → quơ (not qươ)
    assert_eq!(apply_vni_input("quo7"), "quơ");
}

// ==================== UO + Final Consonant + 7 Tests ====================

#[test]
fn test_vni_uoc7_compound() {
    assert_eq!(apply_vni_input("uoc7"), "ươc");
}

#[test]
fn test_vni_uoc71_with_tone() {
    assert_eq!(apply_vni_input("uoc71"), "ước");
}

#[test]
fn test_vni_muon_tone_override_after_7() {
    // "mu7o5n" → "mượn" (auto-fix ưo + consonant → ươ + consonant, keep nặng tone)
    assert_eq!(apply_vni_input("mu7o5n"), "mượn");
    // "mu75on" → "mượn" (tone on ư first, then auto-fix on final consonant)
    assert_eq!(apply_vni_input("mu75on"), "mượn");
    // "mu7on5" → "mượn" (tone after final consonant)
    assert_eq!(apply_vni_input("mu7on5"), "mượn");
}

// ==================== Full Word Tests ====================

#[test]
fn test_vni_viet() {
    assert_eq!(apply_vni_input("vie6t5"), "việt");
}

#[test]
fn test_vni_nam_with_breve() {
    assert_eq!(apply_vni_input("na8m"), "năm");
}

#[test]
fn test_vni_dep() {
    assert_eq!(apply_vni_input("d9e5p"), "đẹp");
}

#[test]
fn test_vni_nguoi() {
    assert_eq!(apply_vni_input("ngu7o7i2"), "người");
}

#[test]
fn test_vni_nuoc() {
    assert_eq!(apply_vni_input("nu7o71c"), "nước");
}

#[test]
fn test_vni_duoc() {
    assert_eq!(apply_vni_input("d9u7o75c"), "được");
}

#[test]
fn test_vni_tieng() {
    assert_eq!(apply_vni_input("tie6ng1"), "tiếng");
}

#[test]
fn test_vni_quoc() {
    assert_eq!(apply_vni_input("quo6c1"), "quốc");
}

#[test]
fn test_vni_hoc() {
    assert_eq!(apply_vni_input("ho5c"), "học");
}

#[test]
fn test_vni_that() {
    assert_eq!(apply_vni_input("that65"), "thật");
}

#[test]
fn test_vni_chan() {
    assert_eq!(apply_vni_input("chan1"), "chán");
}

#[test]
fn test_vni_qua() {
    assert_eq!(apply_vni_input("qua1"), "quá");
}

#[test]
fn test_vni_huynh() {
    assert_eq!(apply_vni_input("huynh2"), "huỳnh");
}

#[test]
fn test_vni_gi_alone() {
    assert_eq!(apply_vni_input("gi2"), "gì");
}

#[test]
fn test_vni_gia() {
    assert_eq!(apply_vni_input("gia1"), "giá");
}

#[test]
fn test_vni_giu() {
    assert_eq!(apply_vni_input("giu74"), "giữ");
}

// ==================== Word Boundary Tests ====================

#[test]
fn test_vni_numbers_not_word_boundaries() {
    // In VNI, numbers should trigger transforms, not reset buffer
    let mut engine = create_vni_engine();
    
    // Type "a1" - should produce "á"
    let result1 = engine.process("a");
    assert!(result1.is_none());
    
    let result2 = engine.process("1");
    assert!(result2.is_some());
    // delete_count is 1 because we replace the vowel "a" with "á"
    // (the '1' is consumed as a tone key, not appended)
    assert_eq!(result2.unwrap(), action(1, "á"));
}

#[test]
fn test_vni_space_is_word_boundary() {
    // Space should reset buffer
    assert_eq!(apply_vni_input("a a1"), "a á");
}

#[test]
fn test_vni_backspace_across_word_boundary_can_edit_previous_tone() {
    let mut engine = create_vni_engine();
    let mut output: Vec<char> = Vec::new();

    for ch in "a1 ".chars() {
        apply_key(&mut engine, &mut output, ch);
    }
    assert_eq!(output.iter().collect::<String>(), "á ");

    // Backspace deletes the boundary (space) and restores the last word into the active buffer.
    backspace(&mut engine, &mut output);
    assert_eq!(output.iter().collect::<String>(), "á");

    // Now that we're back inside the previous word, tone removal should apply (VNI '0').
    apply_key(&mut engine, &mut output, '0');
    assert_eq!(output.iter().collect::<String>(), "a");
}

#[test]
fn test_vni_backspace_three_then_tone_previous_word_in_sentence() {
    let mut engine = create_vni_engine();
    let mut output: Vec<char> = Vec::new();

    // "chan1" -> "chán", "di9" -> "đi"
    for ch in "chan1 qua di9".chars() {
        apply_key(&mut engine, &mut output, ch);
    }
    assert_eq!(output.iter().collect::<String>(), "chán qua đi");

    // Backspace 3 times: delete 'i', 'đ', and the preceding space.
    for _ in 0..3 {
        backspace(&mut engine, &mut output);
    }
    assert_eq!(output.iter().collect::<String>(), "chán qua");

    // Apply sắc tone to "qua" -> "quá" (VNI '1')
    apply_key(&mut engine, &mut output, '1');
    assert_eq!(output.iter().collect::<String>(), "chán quá");
}

#[test]
fn test_vni_backspace_three_then_transform_previous_word_in_sentence() {
    let mut engine = create_vni_engine();
    let mut output: Vec<char> = Vec::new();

    for ch in "chan1 qua di9".chars() {
        apply_key(&mut engine, &mut output, ch);
    }
    assert_eq!(output.iter().collect::<String>(), "chán qua đi");

    for _ in 0..3 {
        backspace(&mut engine, &mut output);
    }
    assert_eq!(output.iter().collect::<String>(), "chán qua");

    // Apply breve transform to 'a' via '8': "qua" -> "quă"
    apply_key(&mut engine, &mut output, '8');
    assert_eq!(output.iter().collect::<String>(), "chán quă");
}

// ==================== Tone Placement Tests ====================

#[test]
fn test_vni_tone_placement_single_vowel() {
    assert_eq!(apply_vni_input("ta1"), "tá");
}

#[test]
fn test_vni_tone_placement_two_vowels_open() {
    assert_eq!(apply_vni_input("hoa1"), "hóa");
}

#[test]
fn test_vni_tone_placement_two_vowels_closed() {
    assert_eq!(apply_vni_input("toan2"), "toàn");
}

#[test]
fn test_vni_tone_placement_nucleus_only() {
    assert_eq!(apply_vni_input("tie6n1"), "tiến");
    assert_eq!(apply_vni_input("muo6n1"), "muốn");
}

#[test]
fn test_vni_tone_placement_three_vowels() {
    assert_eq!(apply_vni_input("khuya3"), "khuỷa");
}

// ==================== Auto Fix Tone Tests ====================

#[test]
fn test_vni_auto_fix_tone_add_vowel() {
    // hoa + 2 → hòa, then + i → hoài (tone moves to middle)
    assert_eq!(apply_vni_input("hoa2i"), "hoài");
}

#[test]
fn test_vni_auto_fix_tone_add_consonant() {
    // hoa + 2 → hòa, then + n → hoàn (tone moves to second vowel)
    assert_eq!(apply_vni_input("hoa2n"), "hoàn");
}

// ==================== Transform + Tone Combined Tests ====================

#[test]
fn test_vni_transform_then_tone() {
    // First apply transform, then tone
    assert_eq!(apply_vni_input("a61"), "ấ"); // â + sắc
    assert_eq!(apply_vni_input("e62"), "ề"); // ê + huyền
    assert_eq!(apply_vni_input("o63"), "ổ"); // ô + hỏi
    assert_eq!(apply_vni_input("o74"), "ỡ"); // ơ + ngã
    assert_eq!(apply_vni_input("u75"), "ự"); // ư + nặng
    assert_eq!(apply_vni_input("a81"), "ắ"); // ă + sắc
}

#[test]
fn test_vni_tone_then_transform() {
    // First apply tone, then transform
    assert_eq!(apply_vni_input("a16"), "ấ"); // á + circumflex → ấ
    assert_eq!(apply_vni_input("a18"), "ắ"); // á + breve → ắ
    assert_eq!(apply_vni_input("o17"), "ớ"); // ó + horn → ớ
}

// ==================== Edge Cases ====================

#[test]
fn test_vni_standalone_7_no_transform() {
    // Unlike Telex 'w', VNI '7' alone should not produce 'ư'
    // It should just output '7' as a word boundary doesn't occur
    let mut engine = create_vni_engine();
    let result = engine.process("7");
    // '7' alone with no vowel to transform should return None
    assert!(result.is_none());
}

#[test]
fn test_vni_transform_on_toned_vowel() {
    // Apply transform on already toned vowel
    assert_eq!(apply_vni_input("o16"), "ố"); // ó + circumflex → ố
    assert_eq!(apply_vni_input("o17"), "ớ"); // ó + horn → ớ
}

#[test]
fn test_vni_di9_free_transform() {
    // "đi" using free transform (d...9)
    assert_eq!(apply_vni_input("di9"), "đi");
}

#[test]
fn test_vni_multiple_words() {
    assert_eq!(apply_vni_input("vie6t5 na8m"), "việt năm");
}
