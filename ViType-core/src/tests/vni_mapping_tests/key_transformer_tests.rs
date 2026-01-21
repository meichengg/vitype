// KeyTransformerTests.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

#![allow(non_snake_case)]

use super::{action, apply_vni_input, create_vni_engine};

mod key_transformer_tests {
    use super::{action, apply_vni_input, create_vni_engine};
    use crate::HistorySegment;
    use crate::VitypeEngine;

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
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("d"), None);
        assert_eq!(transformer.process("9"), Some(action(1, "đ")));
    }

    #[test]
    fn testConsonantDDUppercase() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("D"), None);
        assert_eq!(transformer.process("9"), Some(action(1, "Đ")));
    }

    #[test]
    fn testConsonantDDMixedCase() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("D"), None);
        assert_eq!(transformer.process("9"), Some(action(1, "Đ")));
    }

    #[test]
    fn testEscapeToneBasic() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("1"), Some(action(1, "á")));
        assert_eq!(transformer.process("1"), Some(action(1, "a1")));
    }

    #[test]
    fn testConsonantDDAfterOtherChars() {
        assert_eq!(apply_vni_input("ad9"), "ad9");
    }

    // MARK: - Vowel Transform Tests (aa, ee, oo)

    #[test]
    fn testVowelAA() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("6"), Some(action(1, "â")));
    }

    #[test]
    fn testVowelEE() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("e"), None);
        assert_eq!(transformer.process("6"), Some(action(1, "ê")));
        assert_eq!(apply_vni_input("mem62"), "mềm");
    }

    #[test]
    fn testVowelOO() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("o"), None);
        assert_eq!(transformer.process("6"), Some(action(1, "ô")));

        assert_eq!(apply_vni_input("thoi6"), "thôi");
    }

    #[test]
    fn testFreeTransformStopsAtInterveningVowel() {
        assert_eq!(apply_vni_input("device"), "device");
    }

    #[test]
    fn testFreeTransformRepositionsTone() {
        assert_eq!(apply_vni_input("tuyet5"), "tuỵet");
        assert_eq!(apply_vni_input("tuyet56"), "tuyệt");
        assert_eq!(apply_vni_input("tuyet65"), "tuyệt");
    }

    #[test]
    fn testInvalidSyllableRevertsToRawText() {
        assert_eq!(apply_vni_input("thatae"), "thatae");
    }

    #[test]
    fn testInvalidSyllableMultipleClusters() {
        assert_eq!(apply_vni_input("hoao"), "hoao");
        assert_eq!(apply_vni_input("eaoe"), "eaoe");
        assert_eq!(apply_vni_input("oa o"), "oa o");
    }

    #[test]
    fn testNoOpRewriteActionIsNotEmittedForEnglishMultipleClusters() {
        let mut transformer = create_vni_engine();

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
        assert_eq!(apply_vni_input("device6"), "device6");
        assert_eq!(apply_vni_input("device a6"), "device â");
    }

    #[test]
    fn testBackspaceAcrossWordBoundaryCanEditPreviousTone() {
        let mut engine = create_vni_engine();
        let mut output: Vec<char> = Vec::new();

        for ch in "ta1 ".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "tá ");

        // Backspace deletes the boundary (space) and restores the last word into the active buffer.
        backspace(&mut engine, &mut output);
        assert_eq!(output.iter().collect::<String>(), "tá");

        // Now that we're back inside the previous word, tone removal should apply.
        apply_key(&mut engine, &mut output, '0');
        assert_eq!(output.iter().collect::<String>(), "ta");

        // And we can continue typing boundaries as usual.
        apply_key(&mut engine, &mut output, '1');
        assert_eq!(output.iter().collect::<String>(), "tá");
    }

    #[test]
    fn testBackspaceAfterToneThenForeignModeRewriteStaysCorrect() {
        let mut engine = create_vni_engine();
        let mut output: Vec<char> = Vec::new();

        for ch in "re1".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "ré");

        backspace(&mut engine, &mut output);
        assert_eq!(output.iter().collect::<String>(), "r");

        for ch in "epo".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "repo");
    }

    #[test]
    fn testWordHistoryIsLimitedToRecentWords() {
        let mut engine = create_vni_engine();
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
        let mut engine = create_vni_engine();
        let mut output: Vec<char> = Vec::new();

        // "chans" -> "chán", "ddi" -> "đi"
        for ch in "chan1 qua d9i".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua đi");

        // Backspace 3 times: delete 'i', 'đ', and the preceding space.
        for _ in 0..3 {
            backspace(&mut engine, &mut output);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua");

        // Apply sắc tone to "qua" -> "quá"
        apply_key(&mut engine, &mut output, '1');
        assert_eq!(output.iter().collect::<String>(), "chán quá");
    }

    #[test]
    fn testBackspaceThreeThenTransformPreviousWordInSentence() {
        let mut engine = create_vni_engine();
        let mut output: Vec<char> = Vec::new();

        for ch in "chan1 qua d9i".chars() {
            apply_key(&mut engine, &mut output, ch);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua đi");

        for _ in 0..3 {
            backspace(&mut engine, &mut output);
        }
        assert_eq!(output.iter().collect::<String>(), "chán qua");

        // Apply breve transform to 'a' via 'w': "qua" -> "quă"
        apply_key(&mut engine, &mut output, '8');
        assert_eq!(output.iter().collect::<String>(), "chán quă");
    }

    #[test]
    fn testVowelAAUppercase() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("A"), None);
        assert_eq!(transformer.process("6"), Some(action(1, "Â")));
    }

    // MARK: - Vowel Transform Tests (aw, ow, uw)

    #[test]
    fn testVowelAW() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("8"), Some(action(1, "ă")));
    }

    #[test]
    fn testVowelOW() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("o"), None);
        assert_eq!(transformer.process("7"), Some(action(1, "ơ")));
    }

    #[test]
    fn testVowelUW() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("u"), None);
        assert_eq!(transformer.process("7"), Some(action(1, "ư")));
    }

    #[test]
    fn testCircumflexOverridesBreveOrHorn() {
        // VNI keys can override each other on the same vowel:
        // - a8 (ă) then 6 (â)
        // - o7 (ơ) then 6 (ô)
        assert_eq!(apply_vni_input("ha86"), "hâ");
        assert_eq!(apply_vni_input("ha8y6"), "hây");
        assert_eq!(apply_vni_input("ha8y126"), "hầy");

        assert_eq!(apply_vni_input("ho76"), "hô");
        assert_eq!(apply_vni_input("ho7i16"), "hối");
        assert_eq!(apply_vni_input("ho726"), "hồ");
    }

    #[test]
    fn testVowelWAfterConsonantU() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("u"), None);
        assert_eq!(transformer.process("7"), Some(action(1, "ư")));
    }

    // MARK: - Tone Mark Tests

    #[test]
    fn testToneSac() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("1"), Some(action(1, "á")));
    }

    #[test]
    fn testToneHuyen() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("2"), Some(action(1, "à")));
    }

    #[test]
    fn testToneHoi() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("3"), Some(action(1, "ả")));
    }

    #[test]
    fn testToneNga() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("4"), Some(action(1, "ã")));
    }

    #[test]
    fn testToneNang() {
        let mut transformer = create_vni_engine();

        assert_eq!(transformer.process("t"), None);
        assert_eq!(transformer.process("a"), None);
        assert_eq!(transformer.process("5"), Some(action(1, "ạ")));
    }

    #[test]
    fn testToneRemovalZ() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("1"); // tá
        assert_eq!(transformer.process("0"), Some(action(1, "a")));
    }

    // MARK: - Tone Replacement Tests

    #[test]
    fn testToneReplacement() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("2"); // tà
        assert_eq!(transformer.process("1"), Some(action(1, "á")));
    }

    // MARK: - Complex Vowel + Tone Tests

    #[test]
    fn testToneOnTransformedVowel() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("c");
        let _ = transformer.process("a");
        let _ = transformer.process("6"); // câ
        assert_eq!(transformer.process("1"), Some(action(1, "ấ")));
    }

    #[test]
    fn testToneOnUW() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("7"); // tư
        assert_eq!(transformer.process("1"), Some(action(1, "ứ")));
    }

    #[test]
    fn testToneOnOW() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("m");
        let _ = transformer.process("o");
        let _ = transformer.process("7"); // mơ
        assert_eq!(transformer.process("2"), Some(action(1, "ờ")));
    }

    // MARK: - Escape Sequence Tests

    #[test]
    fn testEscapeDD() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("d");
        let _ = transformer.process("9"); // đ
        assert_eq!(transformer.process("9"), Some(action(1, "d9")));
    }

    #[test]
    fn testEscapeAA() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("a");
        let _ = transformer.process("6"); // â
        assert_eq!(transformer.process("6"), Some(action(1, "a6")));
    }

    #[test]
    fn testEscapeAW() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("a");
        let _ = transformer.process("8"); // ă
        assert_eq!(transformer.process("8"), Some(action(1, "a8")));
    }

    #[test]
    fn testEscapeTone() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("1"); // tá
        assert_eq!(transformer.process("1"), Some(action(1, "a1")));
    }

    #[test]
    fn testEscapeWithTrailingConsonants() {
        assert_eq!(apply_vni_input("e4"), "ẽ");
        assert_eq!(apply_vni_input("e4p6"), "ễp");
        assert_eq!(apply_vni_input("e44"), "e4");
        assert_eq!(apply_vni_input("e44p6"), "e4p6");
        assert_eq!(apply_vni_input("e44pe"), "e4pe");
    }

    // MARK: - Buffer Reset Tests

    #[test]
    fn testResetOnSpace() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("d");
        let _ = transformer.process(" ");
        assert_eq!(transformer.process("d"), None);
    }

    #[test]
    fn testResetOnNewline() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("d");
        let _ = transformer.process("\n");
        assert_eq!(transformer.process("d"), None);
    }

    #[test]
    fn testResetOnPunctuation() {
        let mut transformer = create_vni_engine();

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
        let mut transformer = create_vni_engine();

        let _ = transformer.process("v");
        let _ = transformer.process("i");
        let _ = transformer.process("e");
        let ee_action = transformer.process("6"); // viê
        assert_eq!(ee_action, Some(action(1, "ê")));

        let tone_action = transformer.process("5"); // việ
        assert_eq!(tone_action, Some(action(1, "ệ")));

        let _ = transformer.process("t"); // việt
    }

    #[test]
    fn testWordNam() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("n");
        let _ = transformer.process("a");
        let aw_action = transformer.process("8"); // nă
        assert_eq!(aw_action, Some(action(1, "ă")));

        let _ = transformer.process("m"); // năm
    }

    #[test]
    fn testWordDe() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("d");
        let dd_action = transformer.process("9"); // đ
        assert_eq!(dd_action, Some(action(1, "đ")));

        let _ = transformer.process("e");
        let ee_action = transformer.process("6"); // đê
        assert_eq!(ee_action, Some(action(1, "ê")));

        let tone_action = transformer.process("2"); // đề
        assert_eq!(tone_action, Some(action(1, "ề")));
    }

    #[test]
    fn testWordNguoi() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("n");
        let _ = transformer.process("g");
        let _ = transformer.process("u");
        let uw_action = transformer.process("7"); // ngư
        assert_eq!(uw_action, Some(action(1, "ư")));

        let _ = transformer.process("o");
        let ow_action = transformer.process("7"); // ngươ
        assert_eq!(ow_action, Some(action(2, "ươ")));

        let _ = transformer.process("i"); // ngươi
                                          // ơ is special vowel → tone goes on ơ (not i)
        let tone_action = transformer.process("2");
        // deleteCount=2: delete "ơi", text="ời": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ời")));
    }

    #[test]
    fn testWordNuoc() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("n");
        let _ = transformer.process("u");
        let uw_action = transformer.process("7"); // nư
        assert_eq!(uw_action, Some(action(1, "ư")));

        let _ = transformer.process("o");
        let ow_action = transformer.process("7"); // nươ
        assert_eq!(ow_action, Some(action(2, "ươ")));

        let tone_action = transformer.process("1"); // nướ
        assert_eq!(tone_action, Some(action(1, "ớ")));

        let _ = transformer.process("c"); // nước
    }

    // MARK: - Case Preservation Tests

    #[test]
    fn testUppercaseWord() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("V");
        let _ = transformer.process("I");
        let _ = transformer.process("E");
        let ee_action = transformer.process("6");
        assert_eq!(ee_action, Some(action(1, "Ê")));

        let tone_action = transformer.process("5");
        assert_eq!(tone_action, Some(action(1, "Ệ")));
    }

    #[test]
    fn testMixedCaseHa() {
        let mut transformer = create_vni_engine();

        let _ = transformer.process("H");
        let _ = transformer.process("a");
        let tone_action = transformer.process("2");
        assert_eq!(tone_action, Some(action(1, "à")));
    }

    // MARK: - Tone Placement Rules Tests

    #[test]
    fn testToneOnTwoVowelsFirst() {
        let mut transformer = create_vni_engine();

        // "mua" + f → "mùa" (2 vowels → tone on 1st vowel)
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("a");
        let tone_action = transformer.process("2");
        // deleteCount=2: delete "ua", text="ùa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ùa")));
    }

    #[test]
    fn testToneOnTwoVowelsHoa() {
        let mut transformer = create_vni_engine();

        // "hoa" + s → "hóa" (2 vowels → tone on 1st vowel)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let tone_action = transformer.process("1");
        // deleteCount=2: delete "oa", text="óa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "óa")));
    }

    #[test]
    fn testToneOnTwoVowelsWithFinalConsonant() {
        // two regular vowels + final consonant → tone on 2nd vowel
        assert_eq!(apply_vni_input("toen2"), "toèn");
        assert_eq!(apply_vni_input("tien2"), "tièn");
    }

    #[test]
    fn testToneOnSpecialVowelE() {
        let mut transformer = create_vni_engine();

        // "tiên" - iê cluster, ê is special → tone on ê
        let _ = transformer.process("t");
        let _ = transformer.process("i");
        let _ = transformer.process("e");
        let _ = transformer.process("6"); // tiê
        let _ = transformer.process("n"); // tiên
        let tone_action = transformer.process("1");
        // deleteCount=2: delete "ên", text="ến": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ến")));
    }

    #[test]
    fn testToneOnNucleusVowelO() {
        let mut transformer = create_vni_engine();

        // "muôn" - ô is nucleus-only vowel → tone on ô
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("6"); // muô
        let _ = transformer.process("n"); // muôn
        let tone_action = transformer.process("1");
        // deleteCount=2: delete "ôn", text="ốn": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ốn")));
    }

    #[test]
    fn testToneOnThreeVowelsMiddle() {
        let mut transformer = create_vni_engine();

        // "khuya" - 3 vowels (u, y, a), none are nucleus-only → tone on middle (y)
        let _ = transformer.process("k");
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("y");
        let _ = transformer.process("a");
        let tone_action = transformer.process("3");
        // deleteCount=2: delete "ya", text="ỷa": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ỷa")));
    }

    #[test]
    fn testToneOnNucleusVowelTuoi() {
        let mut transformer = create_vni_engine();

        // "tuôi" - ô is nucleus-only vowel → tone on ô (not middle by count)
        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("6"); // tuô
        let _ = transformer.process("i"); // tuôi
        let tone_action = transformer.process("3");
        // deleteCount=2: delete "ôi", text="ổi": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ổi")));
    }

    #[test]
    fn testToneOnSpecialVowelOHorn() {
        let mut transformer = create_vni_engine();

        // "thơ" + f → "thờ" (ơ is special vowel)
        let _ = transformer.process("t");
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("7"); // thơ
        let tone_action = transformer.process("2");
        assert_eq!(tone_action, Some(action(1, "ờ")));
    }

    #[test]
    fn testToneOnSpecialVowelECircumflex() {
        let mut transformer = create_vni_engine();

        // "đêm" + f → "đềm" (ê is nucleus-only vowel)
        let _ = transformer.process("d");
        let _ = transformer.process("9"); // đ
        let _ = transformer.process("e");
        let _ = transformer.process("6"); // đê
        let _ = transformer.process("m"); // đêm
        let tone_action = transformer.process("2");
        // deleteCount=2: delete "êm", text="ềm": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ềm")));
    }

    // MARK: - Nucleus-Only Vowel Priority Tests

    #[test]
    fn testToneOnNucleusVowelUHorn() {
        let mut transformer = create_vni_engine();

        // "hưu" + f → "hừu" (ư is nucleus-only, takes tone over u)
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("7"); // hư
        let _ = transformer.process("u"); // hưu
        let tone_action = transformer.process("2");
        // deleteCount=2: delete "ưu", text="ừu": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ừu")));
    }

    #[test]
    fn testToneOnNucleusVowelHuou() {
        let mut transformer = create_vni_engine();

        // "hươu" + s → "hướu" (ơ is nucleus-only, takes tone)
        let _ = transformer.process("h");
        let _ = transformer.process("u");
        let _ = transformer.process("7"); // hư
        let _ = transformer.process("o");
        let _ = transformer.process("7"); // hươ
        let _ = transformer.process("u"); // hươu
        let tone_action = transformer.process("1");
        // deleteCount=2: delete "ơu", text="ớu": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ớu")));
    }

    #[test]
    fn testToneOnNucleusVowelThat() {
        let mut transformer = create_vni_engine();

        // "thât" + j → "thật" (â is nucleus-only, takes tone)
        let _ = transformer.process("t");
        let _ = transformer.process("h");
        let _ = transformer.process("a");
        let _ = transformer.process("6"); // thâ
        let _ = transformer.process("t"); // thât
        let tone_action = transformer.process("5");
        // deleteCount=2: delete "ât", text="ật": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ật")));
    }

    #[test]
    fn testToneOnNucleusVowelABreve() {
        let mut transformer = create_vni_engine();

        // "năm" + s → "nắm" (ă is nucleus-only, takes tone)
        let _ = transformer.process("n");
        let _ = transformer.process("a");
        let _ = transformer.process("8"); // nă
        let _ = transformer.process("m"); // năm
        let tone_action = transformer.process("1");
        // deleteCount=2: delete "ăm", text="ắm": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ắm")));
    }

    #[test]
    fn testToneOnNucleusVowelQuyen() {
        let mut transformer = create_vni_engine();

        // "quyên" + f → "quyền" (ê is nucleus-only, takes tone over u and y)
        let _ = transformer.process("q");
        let _ = transformer.process("u");
        let _ = transformer.process("y");
        let _ = transformer.process("e");
        let _ = transformer.process("6"); // quyê
        let _ = transformer.process("n"); // quyên
        let tone_action = transformer.process("2");
        // deleteCount=2: delete "ên", text="ền": replaces from toned vowel to end
        assert_eq!(tone_action, Some(action(2, "ền")));
    }

    // MARK: - đâu Tests

    #[test]
    fn testWordDau() {
        // "đâu" from various input patterns
        assert_eq!(apply_vni_input("d9a6u"), "đâu"); // sequential: d9→đ, a6→â, u
        assert_eq!(apply_vni_input("dau96"), "đâu"); // free transform both d and a
        assert_eq!(apply_vni_input("d9au6"), "đâu"); // d9→đ, free transform a...a
    }

    #[test]
    fn testFreeTransformDStrokeMaxGap() {
        // "đuong" via d...9 with 4 characters between
        assert_eq!(apply_vni_input("duong9"), "đuong");
    }

    // MARK: - Free Transform with Tone Tests

    #[test]
    fn testFreeTransformWithTone() {
        // "toàn" from various input patterns
        assert_eq!(apply_vni_input("toan2"), "toàn");
        assert_eq!(apply_vni_input("toa2n"), "toàn");

        // "hoành" from various input patterns
        assert_eq!(apply_vni_input("hoan2h"), "hoành");
        assert_eq!(apply_vni_input("hoanh2"), "hoành");
        assert_eq!(apply_vni_input("ho2anh"), "hoành");
    }
}
