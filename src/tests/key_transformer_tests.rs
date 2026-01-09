// KeyTransformerTests.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

#![allow(non_snake_case)]

use crate::VitypeEngine;

use super::test_helpers::{action, apply_input};

mod key_transformer_tests {
    use super::{action, apply_input, VitypeEngine};
    use crate::HistorySegment;

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

    // MARK: - Consonant Tests (dd -> đ)

    #[test]
    fn testConsonantDD() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("d"), None);
        assert_eq!(transformer.process("d"), Some(action(1, "đ")));
    }

    #[test]
    fn testConsonantDDUppercase() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("D"), None);
        assert_eq!(transformer.process("D"), Some(action(1, "Đ")));
    }

    #[test]
    fn testConsonantDDMixedCase() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("D"), None);
        assert_eq!(transformer.process("d"), Some(action(1, "Đ")));
    }

    #[test]
    fn testConsonantDDAfterOtherChars() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("d"), None);
        assert_eq!(transformer.process("d"), Some(action(1, "đ")));
    }

    // MARK: - Vowel Transform Tests (aa, ee, oo)

    #[test]
    fn testVowelAA() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("a"), Some(action(1, "â")));
    }

    #[test]
    fn testVowelEE() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("e"), None);
        assert_eq!(transformer.process("e"), Some(action(1, "ê")));
        assert_eq!(apply_input("memef"), "mềm");
    }

    #[test]
    fn testVowelOO() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("o"), None);
        assert_eq!(transformer.process("o"), Some(action(1, "ô")));

        assert_eq!(apply_input("thoio"), "thôi");
    }

    #[test]
    fn testFreeTransformStopsAtInterveningVowel() {
        assert_eq!(apply_input("device"), "device");
    }

    #[test]
    fn testFreeTransformRepositionsTone() {
        assert_eq!(apply_input("tuyetj"), "tuỵet");
        assert_eq!(apply_input("tuyetje"), "tuyệt");
        assert_eq!(apply_input("tuyetej"), "tuyệt");
    }

    #[test]
    fn testInvalidSyllableRevertsToRawText() {
        assert_eq!(apply_input("thatae"), "thatae");
    }

    #[test]
    fn testInvalidSyllableMultipleClusters() {
        assert_eq!(apply_input("hoao"), "hoao");
        assert_eq!(apply_input("eaoe"), "eaoe");
        assert_eq!(apply_input("oa o"), "oa o");
    }

    #[test]
    fn testNoOpRewriteActionIsNotEmittedForEnglishMultipleClusters() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("p"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("y"), None);
        assert_eq!(transformer.process("m"), None);

        // "paym" + "e" creates multiple vowel clusters (a/y then e). The engine should switch to
        // foreign mode, but must not emit a rewrite action when the visible text is already raw.
        assert_eq!(transformer.process("e"), None);

        assert_eq!(transformer.process("n"), None);
        assert_eq!(transformer.process("t"), None);
    }

    #[test]
    fn testInvalidSyllableDisablesTransformsUntilBoundary() {
        assert_eq!(apply_input("devicee"), "devicee");
        assert_eq!(apply_input("device aa"), "device â");
    }

    #[test]
    fn testBackspaceAcrossWordBoundaryCanEditPreviousTone() {
        let mut engine = VitypeEngine::new();
        let mut output: Vec<char> = Vec::new();

        for ch in "tas ".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "tá ");

        // Backspace deletes the boundary (space) and restores the last word into the active buffer.
        backspace(&mut engine, &mut output);
        assert_eq!(output.iter().collect::<String>(), "tá");

        // Now that we're back inside the previous word, tone removal should apply.
        apply_key(&mut engine, &mut output, 'z');
        assert_eq!(output.iter().collect::<String>(), "ta");

        // And we can continue typing boundaries as usual.
        apply_key(&mut engine, &mut output, 's');
        assert_eq!(output.iter().collect::<String>(), "tá");
    }

    #[test]
    fn testWordHistoryIsLimitedToRecentWords() {
        let mut engine = VitypeEngine::new();
        let mut output: Vec<char> = Vec::new();

        // Commit 5 words, each followed by a boundary (space)
        for _ in 0..5 {
            for ch in "ta ".chars() {
                apply_key(&mut engine, &mut output, ch);
            }
        }

        let word_count = engine
            .history
            .iter()
            .filter(|seg| matches!(seg, HistorySegment::Word(_)))
            .count();
        assert_eq!(word_count, 3);
        assert_eq!(output.iter().collect::<String>(), "ta ta ta ta ta ");
    }

    #[test]
    fn testBackspaceThreeThenTonePreviousWordInSentence() {
        let mut engine = VitypeEngine::new();
        let mut output: Vec<char> = Vec::new();

        // "chans" -> "chán", "ddi" -> "đi"
        for ch in "chans qua ddi".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua đi");

        // Backspace 3 times: delete 'i', 'đ', and the preceding space.
        for _ in 0..3 {
            backspace(&mut engine, &mut output);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua");

        // Apply sắc tone to "qua" -> "quá"
        apply_key(&mut engine, &mut output, 's');
        assert_eq!(output.iter().collect::<String>(), "chán quá");
    }

    #[test]
    fn testBackspaceThreeThenTransformPreviousWordInSentence() {
        let mut engine = VitypeEngine::new();
        let mut output: Vec<char> = Vec::new();

        for ch in "chans qua ddi".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua đi");

        for _ in 0..3 {
            backspace(&mut engine, &mut output);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua");

        // Apply breve transform to 'a' via 'w': "qua" -> "quă"
        apply_key(&mut engine, &mut output, 'w');
        assert_eq!(output.iter().collect::<String>(), "chán quă");
    }

    #[test]
    fn testVowelAAUppercase() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("A"), None);
        assert_eq!(transformer.process("A"), Some(action(1, "Â")));
    }

    // MARK: - Vowel Transform Tests (aw, ow, uw)

    #[test]
    fn testVowelAW() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("w"), Some(action(1, "ă")));
    }

    #[test]
    fn testVowelOW() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("o"), None);
        assert_eq!(transformer.process("w"), Some(action(1, "ơ")));
    }

    #[test]
    fn testVowelUW() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("u"), None);
        assert_eq!(transformer.process("w"), Some(action(1, "ư")));
    }

    #[test]
    fn testCircumflexOverridesBreveOrHorn() {
        // After applying a w-transform (ă/ơ), the circumflex key (a/o) should be able to
        // override it on the same vowel without requiring an escape sequence.
        assert_eq!(apply_input("hawa"), "hâ");
        assert_eq!(apply_input("hawya"), "hây");
        assert_eq!(apply_input("hawysfa"), "hầy");

        assert_eq!(apply_input("howo"), "hô");
        assert_eq!(apply_input("howio"), "hôi");
        assert_eq!(apply_input("howsio"), "hối");
        assert_eq!(apply_input("howfo"), "hồ");
    }

    #[test]
    fn testVowelWAfterConsonantU() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("u"), None);
        assert_eq!(transformer.process("w"), Some(action(1, "ư")));
    }

    // MARK: - Tone Mark Tests

    #[test]
    fn testToneSac() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("s"), Some(action(1, "á")));
    }

    #[test]
    fn testToneHuyen() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("f"), Some(action(1, "à")));
    }

    #[test]
    fn testToneHoi() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("r"), Some(action(1, "ả")));
    }

    #[test]
    fn testToneNga() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("x"), Some(action(1, "ã")));
    }

    #[test]
    fn testToneNang() {
        let mut transformer = VitypeEngine::new();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("j"), Some(action(1, "ạ")));
    }

    #[test]
    fn testToneRemovalZ() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("s"); // tá
        assert_eq!(transformer.process("z"), Some(action(1, "a")));
    }

    // MARK: - Tone Replacement Tests

    #[test]
    fn testToneReplacement() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("f"); // tà
        assert_eq!(transformer.process("s"), Some(action(1, "á")));
    }

    // MARK: - Complex Vowel + Tone Tests

    #[test]
    fn testToneOnTransformedVowel() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("c");
        let _ = transformer.process("a");
        let _ = transformer.process("a"); // câ
        assert_eq!(transformer.process("s"), Some(action(1, "ấ")));
    }

    #[test]
    fn testToneOnUW() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("w"); // tư
        assert_eq!(transformer.process("s"), Some(action(1, "ứ")));
    }

    #[test]
    fn testToneOnOW() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("m");
        let _ = transformer.process("o");
        let _ = transformer.process("w"); // mơ
        assert_eq!(transformer.process("f"), Some(action(1, "ờ")));
    }

    // MARK: - Escape Sequence Tests

    #[test]
    fn testEscapeDD() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("d");
        let _ = transformer.process("d"); // đ
        assert_eq!(transformer.process("d"), Some(action(1, "dd")));
    }

    #[test]
    fn testEscapeAA() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("a");
        let _ = transformer.process("a"); // â
        assert_eq!(transformer.process("a"), Some(action(1, "aa")));
    }

    #[test]
    fn testEscapeAW() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("a");
        let _ = transformer.process("w"); // ă
        assert_eq!(transformer.process("w"), Some(action(1, "aw")));
    }

    #[test]
    fn testEscapeTone() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("s"); // tá
        assert_eq!(transformer.process("s"), Some(action(1, "as")));
    }

    #[test]
    fn testEscapeWithTrailingConsonants() {
        assert_eq!(apply_input("ex"), "ẽ");
        assert_eq!(apply_input("expe"), "ễp");
        assert_eq!(apply_input("exx"), "ex");
        assert_eq!(apply_input("exxpe"), "expe");
        assert_eq!(apply_input("exxpee"), "expee");
    }

    // MARK: - Buffer Reset Tests

    #[test]
    fn testResetOnSpace() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("d");
        let _ = transformer.process(" ");
        assert_eq!(transformer.process("d"), None);
    }

    #[test]
    fn testResetOnNewline() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("d");
        let _ = transformer.process("\n");
        assert_eq!(transformer.process("d"), None);
    }

    #[test]
    fn testResetOnPunctuation() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("d");
        let _ = transformer.process(",");
        assert_eq!(transformer.process("d"), None);

        transformer.reset();
        let _ = transformer.process("d");
        let _ = transformer.process(".");
        assert_eq!(transformer.process("d"), None);
    }

    // MARK: - Real Vietnamese Words Tests

    #[test]
    fn testWordViet() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("v");
        let _ = transformer.process("i");
        let _ = transformer.process("e");
        let ee_action = transformer.process("e"); // viê
        assert_eq!(ee_action, Some(action(1, "ê")));

        let tone_action = transformer.process("j"); // việ
        assert_eq!(tone_action, Some(action(1, "ệ")));

        let _ = transformer.process("t"); // việt
    }

    #[test]
    fn testWordNam() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("n");
        let _ = transformer.process("a");
        let aw_action = transformer.process("w"); // nă
        assert_eq!(aw_action, Some(action(1, "ă")));

        let _ = transformer.process("m"); // năm
    }

    #[test]
    fn testWordDe() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("d");
        let dd_action = transformer.process("d"); // đ
        assert_eq!(dd_action, Some(action(1, "đ")));

        let _ = transformer.process("e");
        let ee_action = transformer.process("e"); // đê
        assert_eq!(ee_action, Some(action(1, "ê")));

        let tone_action = transformer.process("f"); // đề
        assert_eq!(tone_action, Some(action(1, "ề")));
    }

    #[test]
    fn testWordNguoi() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("n");
        let _ = transformer.process("g");
        let _ = transformer.process("u");
        let uw_action = transformer.process("w"); // ngư
        assert_eq!(uw_action, Some(action(1, "ư")));

        let _ = transformer.process("o");
        let ow_action = transformer.process("w"); // ngươ
        assert_eq!(ow_action, Some(action(1, "ơ")));

        let _ = transformer.process("i"); // ngươi
                                          // ơ is special vowel → tone goes on ơ (not i)
        let tone_action = transformer.process("f");
        // deleteCount=2: delete "ơi", text="ời": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ời")));
    }

    #[test]
    fn testWordNuoc() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("n");
        let _ = transformer.process("u");
        let uw_action = transformer.process("w"); // nư
        assert_eq!(uw_action, Some(action(1, "ư")));

        let _ = transformer.process("o");
        let ow_action = transformer.process("w"); // nươ
        assert_eq!(ow_action, Some(action(1, "ơ")));

        let tone_action = transformer.process("s"); // nướ
        assert_eq!(tone_action, Some(action(1, "ớ")));

        let _ = transformer.process("c"); // nước
    }

    // MARK: - Case Preservation Tests

    #[test]
    fn testUppercaseWord() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("V");
        let _ = transformer.process("I");
        let _ = transformer.process("E");
        let ee_action = transformer.process("E");
        assert_eq!(ee_action, Some(action(1, "Ê")));

        let tone_action = transformer.process("J");
        assert_eq!(tone_action, Some(action(1, "Ệ")));
    }

    #[test]
    fn testMixedCaseHa() {
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("H");
        let _ = transformer.process("a");
        let tone_action = transformer.process("f");
        assert_eq!(tone_action, Some(action(1, "à")));
    }

    // MARK: - Tone Placement Rules Tests

    #[test]
    fn testToneOnTwoVowelsFirst() {
        let mut transformer = VitypeEngine::new();

        // "mua" + f → "mùa" (2 vowels → tone on 1st vowel)
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("a");
        let tone_action = transformer.process("f");
        // deleteCount=2: delete "ua", text="ùa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ùa")));
    }

    #[test]
    fn testToneOnTwoVowelsHoa() {
        let mut transformer = VitypeEngine::new();

        // "hoa" + s → "hóa" (2 vowels → tone on 1st vowel)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let tone_action = transformer.process("s");
        // deleteCount=2: delete "oa", text="óa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "óa")));
    }

    #[test]
    fn testToneOnTwoVowelsWithFinalConsonant() {
        // two regular vowels + final consonant → tone on 2nd vowel
        assert_eq!(apply_input("toenf"), "toèn");
        assert_eq!(apply_input("tienf"), "tièn");
    }

    #[test]
    fn testToneOnSpecialVowelE() {
        let mut transformer = VitypeEngine::new();

        // "tiên" - iê cluster, ê is special → tone on ê
        let _ = transformer.process("t");
        let _ = transformer.process("i");
        let _ = transformer.process("e");
        let _ = transformer.process("e"); // tiê
        let _ = transformer.process("n"); // tiên
        let tone_action = transformer.process("s");
        // deleteCount=2: delete "ên", text="ến": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ến")));
    }

    #[test]
    fn testToneOnNucleusVowelO() {
        let mut transformer = VitypeEngine::new();

        // "muôn" - ô is nucleus-only vowel → tone on ô
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("o"); // muô
        let _ = transformer.process("n"); // muôn
        let tone_action = transformer.process("s");
        // deleteCount=2: delete "ôn", text="ốn": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ốn")));
    }

    #[test]
    fn testToneOnThreeVowelsMiddle() {
        let mut transformer = VitypeEngine::new();

        // "khuya" - 3 vowels (u, y, a), none are nucleus-only → tone on middle (y)
        let _ = transformer.process("k");
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("y");
        let _ = transformer.process("a");
        let tone_action = transformer.process("r");
        // deleteCount=2: delete "ya", text="ỷa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ỷa")));
    }

    #[test]
    fn testToneOnNucleusVowelTuoi() {
        let mut transformer = VitypeEngine::new();

        // "tuôi" - ô is nucleus-only vowel → tone on ô (not middle by count)
        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("o"); // tuô
        let _ = transformer.process("i"); // tuôi
        let tone_action = transformer.process("r");
        // deleteCount=2: delete "ôi", text="ổi": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ổi")));
    }

    #[test]
    fn testToneOnSpecialVowelOHorn() {
        let mut transformer = VitypeEngine::new();

        // "thơ" + f → "thờ" (ơ is special vowel)
        let _ = transformer.process("t");
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("w"); // thơ
        let tone_action = transformer.process("f");
        assert_eq!(tone_action, Some(action(1, "ờ")));
    }

    #[test]
    fn testToneOnSpecialVowelECircumflex() {
        let mut transformer = VitypeEngine::new();

        // "đêm" + f → "đềm" (ê is nucleus-only vowel)
        let _ = transformer.process("d");
        let _ = transformer.process("d"); // đ
        let _ = transformer.process("e");
        let _ = transformer.process("e"); // đê
        let _ = transformer.process("m"); // đêm
        let tone_action = transformer.process("f");
        // deleteCount=2: delete "êm", text="ềm": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ềm")));
    }

    // MARK: - Nucleus-Only Vowel Priority Tests

    #[test]
    fn testToneOnNucleusVowelUHorn() {
        let mut transformer = VitypeEngine::new();

        // "hưu" + f → "hừu" (ư is nucleus-only, takes tone over u)
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("w"); // hư
        let _ = transformer.process("u"); // hưu
        let tone_action = transformer.process("f");
        // deleteCount=2: delete "ưu", text="ừu": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ừu")));
    }

    #[test]
    fn testToneOnNucleusVowelHuou() {
        let mut transformer = VitypeEngine::new();

        // "hươu" + s → "hướu" (ơ is nucleus-only, takes tone)
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("w"); // hư
        let _ = transformer.process("o");
        let _ = transformer.process("w"); // hươ
        let _ = transformer.process("u"); // hươu
        let tone_action = transformer.process("s");
        // deleteCount=2: delete "ơu", text="ớu": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ớu")));
    }

    #[test]
    fn testToneOnNucleusVowelThat() {
        let mut transformer = VitypeEngine::new();

        // "thât" + j → "thật" (â is nucleus-only, takes tone)
        let _ = transformer.process("t");
        let _ = transformer.process("h");
        let _ = transformer.process("a");
        let _ = transformer.process("a"); // thâ
        let _ = transformer.process("t"); // thât
        let tone_action = transformer.process("j");
        // deleteCount=2: delete "ât", text="ật": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ật")));
    }

    #[test]
    fn testToneOnNucleusVowelABreve() {
        let mut transformer = VitypeEngine::new();

        // "năm" + s → "nắm" (ă is nucleus-only, takes tone)
        let _ = transformer.process("n");
        let _ = transformer.process("a");
        let _ = transformer.process("w"); // nă
        let _ = transformer.process("m"); // năm
        let tone_action = transformer.process("s");
        // deleteCount=2: delete "ăm", text="ắm": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ắm")));
    }

    #[test]
    fn testToneOnNucleusVowelQuyen() {
        let mut transformer = VitypeEngine::new();

        // "quyên" + f → "quyền" (ê is nucleus-only, takes tone over u and y)
        let _ = transformer.process("q");
        let _ = transformer.process("u");
        let _ = transformer.process("y");
        let _ = transformer.process("e");
        let _ = transformer.process("e"); // quyê
        let _ = transformer.process("n"); // quyên
        let tone_action = transformer.process("f");
        // deleteCount=2: delete "ên", text="ền": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ền")));
    }

    // MARK: - đâu Tests

    #[test]
    fn testWordDau() {
        // "đâu" from various input patterns
        assert_eq!(apply_input("ddaau"), "đâu"); // sequential: dd→đ, aa→â, u
        assert_eq!(apply_input("dauda"), "đâu"); // free transform both d and a
        assert_eq!(apply_input("ddaua"), "đâu"); // dd→đ, free transform a...a
    }

    // MARK: - Free Transform with Tone Tests

    #[test]
    fn testFreeTransformWithTone() {
        // "toàn" from various input patterns
        assert_eq!(apply_input("toanf"), "toàn");
        assert_eq!(apply_input("tofan"), "toàn");

        // "hoành" from various input patterns
        assert_eq!(apply_input("hoanfh"), "hoành");
        assert_eq!(apply_input("hoanhf"), "hoành");
        assert_eq!(apply_input("hofanh"), "hoành");
    }
}
