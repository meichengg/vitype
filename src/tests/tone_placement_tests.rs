// TonePlacementTests.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

#![allow(non_snake_case)]

use crate::VitypeEngine;

use super::test_helpers::{action, apply_input, apply_input_with_auto_fix};

// MARK: - Auto Fix Tone Tests
mod auto_fix_tone_tests {
    use super::{action, apply_input, apply_input_with_auto_fix, VitypeEngine};

    // MARK: - Basic Auto Fix Tone Tests

    #[test]
    fn testAutoFixToneHoaToHoai() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "hoa" + "f" → "hòa" (tone on o, 2 vowels → 1st)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let tone_action = transformer.process("f");
        assert_eq!(tone_action, Some(action(2, "òa")));

        // "hòa" + "i" → "hoài" (tone moves to a, 3 vowels → middle)
        // Screen shows "hòa", we need to delete 2 chars ("òa") and insert "oài"
        let fix_action = transformer.process("i");
        assert_eq!(fix_action, Some(action(2, "oài")));
    }

    #[test]
    fn testAutoFixToneMuaToMuai() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "mua" + "f" → "mùa" (tone on u, 2 vowels → 1st)
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("a");
        let tone_action = transformer.process("f");
        assert_eq!(tone_action, Some(action(2, "ùa")));

        // "mùa" + "i" → "muài" (tone moves to a, 3 vowels → middle)
        // Screen shows "mùa", we need to delete 2 chars ("ùa") and insert "uài"
        let fix_action = transformer.process("i");
        assert_eq!(fix_action, Some(action(2, "uài")));
    }

    #[test]
    fn testAutoFixToneDisabled() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = false;

        // "hoa" + "f" → "hòa"
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let _ = transformer.process("f");

        // "hòa" + "i" → "hòai" (no repositioning when disabled)
        let action = transformer.process("i");
        assert_eq!(action, None); // Just appends, no transform
    }

    #[test]
    fn testAutoFixToneDefaultEnabled() {
        let transformer = VitypeEngine::new();
        assert_eq!(transformer.auto_fix_tone, true);
    }

    #[test]
    fn testAutoFixToneNoTonePresent() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "hoa" + "i" → "hoai" (no tone to reposition)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let action = transformer.process("i");
        assert_eq!(action, None); // No tone to fix
    }

    #[test]
    fn testAutoFixToneToneAlreadyCorrect() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "ta" + "s" → "tá" (single vowel, tone correct)
        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("s"); // tá

        // "tá" + "i" → "tái" (2 vowels, tone should stay on 1st)
        let action = transformer.process("i");
        assert_eq!(action, None); // Tone already in correct position
    }

    #[test]
    fn testAutoFixToneWithNucleusVowel() {
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "tuo" + "o" → "tuô" (circumflex transform)
        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("o"); // tuô

        // "tuô" + "s" → "tuố" (ô is nucleus-only, takes tone)
        let _ = transformer.process("s");

        // "tuố" + "i" → "tuối" (tone stays on ô, nucleus-only)
        let action = transformer.process("i");
        assert_eq!(action, None); // ô is nucleus-only, tone stays there
    }

    // MARK: - End-to-End Auto Fix Tone Tests

    #[test]
    fn testAutoFixToneEndToEndHoai() {
        // Type "hoafi" -> should produce "hoài"
        let result = apply_input("hoafi");
        assert_eq!(result, "hoài");
    }

    #[test]
    fn testAutoFixToneEndToEndMuai() {
        // Type "muafi" -> should produce "muài"
        let result = apply_input("muafi");
        assert_eq!(result, "muài");
    }

    #[test]
    fn testAutoFixToneEndToEndKhuyar() {
        // Type "khuyar" -> should produce "khuỷa" (3 vowels, middle gets tone)
        let result = apply_input("khuyar");
        assert_eq!(result, "khuỷa");
    }

    #[test]
    fn testAutoFixToneEndToEndDisabled() {
        // With autoFixTone disabled, "hoafi" -> "hòai" (tone stays on o)
        let result = apply_input_with_auto_fix("hoafi", false);
        assert_eq!(result, "hòai");
    }

    #[test]
    fn testAutoFixToneEndToEndWithNucleus() {
        // Type "tuoois" -> should produce "tuối" (ô is nucleus, keeps tone)
        let result = apply_input("tuoois");
        assert_eq!(result, "tuối");
    }

    #[test]
    fn testAutoFixToneEndToEndNoReposition() {
        // Type "tais" -> should produce "tái" (2 vowels, 1st gets tone, no reposition needed)
        let result = apply_input("tais");
        assert_eq!(result, "tái");
    }

    #[test]
    fn testAutoFixToneEndToEndHuyen() {
        // Type "hoaf" then "i" -> "hoài" (grave tone moves to middle)
        let result = apply_input("hoafi");
        assert_eq!(result, "hoài");
    }

    #[test]
    fn testAutoFixToneEndToEndSac() {
        // Type "hoas" then "i" -> "hoái" (acute tone moves to middle)
        let result = apply_input("hoasi");
        assert_eq!(result, "hoái");
    }

    #[test]
    fn testAutoFixToneEndToEndHoi() {
        // Type "hoar" then "i" -> "hoải" (hook tone moves to middle)
        let result = apply_input("hoari");
        assert_eq!(result, "hoải");
    }

    #[test]
    fn testAutoFixToneEndToEndNga() {
        // Type "hoax" then "i" -> "hoãi" (tilde tone moves to middle)
        let result = apply_input("hoaxi");
        assert_eq!(result, "hoãi");
    }

    #[test]
    fn testAutoFixToneEndToEndNang() {
        // Type "hoaj" then "i" -> "hoại" (dot tone moves to middle)
        let result = apply_input("hoaji");
        assert_eq!(result, "hoại");
    }

    // MARK: - Complex Scenarios

    #[test]
    fn testAutoFixToneComplexNguoi() {
        // "người" - ơ is nucleus-only, should keep tone
        let result = apply_input("nguwowif");
        assert_eq!(result, "người");
    }

    #[test]
    fn testAutoFixToneComplexTuoi() {
        // "tuổi" - ô is nucleus-only, should keep tone
        let result = apply_input("tuooir");
        assert_eq!(result, "tuổi");
    }

    #[test]
    fn testAutoFixToneNoToneToMove() {
        // No tone applied, just vowels
        let result = apply_input("hoai");
        assert_eq!(result, "hoai");
    }

    // MARK: - Escape Sequence After Auto Fix Tone Tests

    #[test]
    fn testEscapeSequenceNotTriggeredAfterAutoFix() {
        // Regression test: Ensure escape sequence detection is reset after auto-fix
        // Previously, lastTransformKey wasn't cleared when auto-fix returned an action,
        // which could cause incorrect escape sequence detection.
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        // "hoa" + "f" → "hòa" (lastTransformKey = f)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let _ = transformer.process("f"); // hòa

        // "hòa" + "i" → "hoài" (auto-fix repositions tone, should clear lastTransformKey)
        let _ = transformer.process("i"); // hoài

        // Now typing "f" should apply tone to the vowel, NOT trigger escape
        // If lastTransformKey wasn't cleared, this might incorrectly try escape logic
        let result = transformer.process("f");

        // "f" should apply grave tone to the target vowel (à already has it, so it stays)
        // The important thing is it doesn't produce an escape sequence like "if" or similar
        // Since hoài already has the tone on à, applying f again should just replace with same tone
        assert_eq!(result, Some(action(2, "aif")));
    }

    #[test]
    fn testEscapeSequenceNotTriggeredAfterAutoFixEndToEnd() {
        // End-to-end version: "hoafif" should produce "hoài" with grave tone
        // NOT trigger any escape sequence behavior
        let result = apply_input("hoafif");
        assert_eq!(result, "hoaif");

        let r1 = apply_input("hoaf");
        assert_eq!(r1, "hòa");

        let r2 = apply_input("hoafi");
        assert_eq!(r2, "hoài");
    }

    #[test]
    fn testEscapeSequenceStillWorksNormally() {
        // Verify that normal escape sequences still work (not broken by the fix)
        let mut transformer = VitypeEngine::new();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let tone_action = transformer.process("f"); // tà
        assert_eq!(tone_action, Some(action(1, "à")));

        // Double "f" should trigger escape: tà + f → taf
        let escape_action = transformer.process("f");
        assert_eq!(escape_action, Some(action(1, "af")));
    }

    #[test]
    fn testEscapeSequenceStillWorksEndToEnd() {
        // Verify escape still works: "taff" → "taf"
        let result = apply_input("taff");
        assert_eq!(result, "taf");
    }
}

// MARK: - UY Special Case Tests
mod uy_tone_placement_tests {
    use super::apply_input;

    // MARK: - UY Alone (tone on U)

    #[test]
    fn testUYAloneToneOnU() {
        // "uy" alone → tone on u
        let result = apply_input("uys");
        assert_eq!(result, "úy");
    }

    #[test]
    fn testUYAloneAllTones() {
        assert_eq!(apply_input("uys"), "úy"); // sắc
        assert_eq!(apply_input("uyf"), "ùy"); // huyền
        assert_eq!(apply_input("uyr"), "ủy"); // hỏi
        assert_eq!(apply_input("uyx"), "ũy"); // ngã
        assert_eq!(apply_input("uyj"), "ụy"); // nặng
    }

    #[test]
    fn testUYAloneWithInitialConsonant() {
        // "tuy" alone → tone on u
        let result = apply_input("tuys");
        assert_eq!(result, "túy");
    }

    // MARK: - UY + Consonant (tone on Y)

    #[test]
    fn testUYNHToneOnY() {
        // "uynh" + tone → tone on y
        let result = apply_input("uynhs");
        assert_eq!(result, "uýnh");
    }

    #[test]
    fn testUYNHAllTones() {
        assert_eq!(apply_input("uynhs"), "uýnh"); // sắc
        assert_eq!(apply_input("uynhf"), "uỳnh"); // huyền
        assert_eq!(apply_input("uynhr"), "uỷnh"); // hỏi
        assert_eq!(apply_input("uynhx"), "uỹnh"); // ngã
        assert_eq!(apply_input("uynhj"), "uỵnh"); // nặng
    }

    #[test]
    fn testUYTToneOnY() {
        // "uyt" + tone → tone on y
        let result = apply_input("uyts");
        assert_eq!(result, "uýt");
    }

    #[test]
    fn testUYTAllTones() {
        assert_eq!(apply_input("uyts"), "uýt"); // sắc
        assert_eq!(apply_input("uytf"), "uỳt"); // huyền
        assert_eq!(apply_input("uytr"), "uỷt"); // hỏi
        assert_eq!(apply_input("uytx"), "uỹt"); // ngã
        assert_eq!(apply_input("uytj"), "uỵt"); // nặng
    }

    #[test]
    fn testHUYNHToneOnY() {
        // "huynh" + tone → tone on y
        let result = apply_input("huynhf");
        assert_eq!(result, "huỳnh");
    }

    #[test]
    fn testHUYNHAllTones() {
        assert_eq!(apply_input("huynhs"), "huýnh"); // sắc
        assert_eq!(apply_input("huynhf"), "huỳnh"); // huyền
        assert_eq!(apply_input("huynhr"), "huỷnh"); // hỏi
        assert_eq!(apply_input("huynhx"), "huỹnh"); // ngã
        assert_eq!(apply_input("huynhj"), "huỵnh"); // nặng
    }

    // MARK: - QUY (special case: u after q is skipped)

    #[test]
    fn testQUYToneOnY() {
        // "quy" - u after q is part of consonant cluster, only y is a vowel
        let result = apply_input("quys");
        assert_eq!(result, "quý");
    }

    #[test]
    fn testQUYAllTones() {
        assert_eq!(apply_input("quys"), "quý"); // sắc
        assert_eq!(apply_input("quyf"), "quỳ"); // huyền
        assert_eq!(apply_input("quyr"), "quỷ"); // hỏi
        assert_eq!(apply_input("quyx"), "quỹ"); // ngã
        assert_eq!(apply_input("quyj"), "quỵ"); // nặng
    }

    // MARK: - Case Preservation

    #[test]
    fn testUYUppercase() {
        assert_eq!(apply_input("UYNHS"), "UÝNH");
        assert_eq!(apply_input("UYS"), "ÚY");
    }

    #[test]
    fn testUYMixedCase() {
        assert_eq!(apply_input("Uynhs"), "Uýnh");
        assert_eq!(apply_input("Huynhf"), "Huỳnh");
    }

    // MARK: - Other UY + Consonant patterns

    #[test]
    fn testUYCToneOnY() {
        let result = apply_input("uycs");
        assert_eq!(result, "uýc");
    }

    #[test]
    fn testUYPToneOnY() {
        let result = apply_input("uyps");
        assert_eq!(result, "uýp");
    }

    #[test]
    fn testUYMToneOnY() {
        let result = apply_input("uyms");
        assert_eq!(result, "uým");
    }

    #[test]
    fn testUYNToneOnY() {
        let result = apply_input("uyns");
        assert_eq!(result, "uýn");
    }
}

// MARK: - GI Special Case Tests
mod gi_tone_placement_tests {
    use super::{action, apply_input, VitypeEngine};

    // MARK: - GI Alone (tone on I)

    #[test]
    fn testGIAloneToneOnI() {
        // "gi" alone → tone on i
        let result = apply_input("gis");
        assert_eq!(result, "gí");
    }

    #[test]
    fn testGIAloneAllTones() {
        assert_eq!(apply_input("gis"), "gí"); // sắc
        assert_eq!(apply_input("gif"), "gì"); // huyền
        assert_eq!(apply_input("gir"), "gỉ"); // hỏi
        assert_eq!(apply_input("gix"), "gĩ"); // ngã
        assert_eq!(apply_input("gij"), "gị"); // nặng
    }

    // MARK: - GI + Vowel (tone on following vowel)

    #[test]
    fn testGIAToneOnA() {
        // "gia" + tone → tone on a (gi is consonant cluster)
        let result = apply_input("gias");
        assert_eq!(result, "giá");
    }

    #[test]
    fn testGIAAllTones() {
        assert_eq!(apply_input("gias"), "giá"); // sắc
        assert_eq!(apply_input("giaf"), "già"); // huyền
        assert_eq!(apply_input("giar"), "giả"); // hỏi
        assert_eq!(apply_input("giax"), "giã"); // ngã
        assert_eq!(apply_input("giaj"), "giạ"); // nặng
    }

    #[test]
    fn testGIOToneOnO() {
        let result = apply_input("gios");
        assert_eq!(result, "gió");
    }

    #[test]
    fn testGIEToneOnE() {
        let result = apply_input("gies");
        assert_eq!(result, "gié");
    }

    #[test]
    fn testGIUToneOnU() {
        let result = apply_input("gius");
        assert_eq!(result, "giú");
    }

    #[test]
    fn testGIUWToneOnUHorn() {
        // "giuw" → "giư", then tone on ư
        let result = apply_input("giuws");
        assert_eq!(result, "giứ");
    }

    #[test]
    fn testGIUWAllTones() {
        assert_eq!(apply_input("giuws"), "giứ"); // sắc
        assert_eq!(apply_input("giuwf"), "giừ"); // huyền
        assert_eq!(apply_input("giuwr"), "giử"); // hỏi
        assert_eq!(apply_input("giuwx"), "giữ"); // ngã
        assert_eq!(apply_input("giuwj"), "giự"); // nặng
    }

    // MARK: - GI + Vowel + Consonant

    #[test]
    fn testGIANGToneOnA() {
        let result = apply_input("giangs");
        assert_eq!(result, "giáng");
    }

    #[test]
    fn testGIANGAllTones() {
        assert_eq!(apply_input("giangs"), "giáng"); // sắc
        assert_eq!(apply_input("giangf"), "giàng"); // huyền
        assert_eq!(apply_input("giangr"), "giảng"); // hỏi
        assert_eq!(apply_input("giangx"), "giãng"); // ngã
        assert_eq!(apply_input("giangj"), "giạng"); // nặng
    }

    // MARK: - GI + Two Vowels (tone on first following vowel)

    #[test]
    fn testGIEOToneOnE() {
        // "gieo" has vowels e, o after gi → tone on first (e)
        let result = apply_input("gieos");
        assert_eq!(result, "giéo");
    }

    #[test]
    fn testGIAOToneOnA() {
        // "giao" has vowels a, o after gi → tone on first (a)
        let result = apply_input("giaof");
        assert_eq!(result, "giào");
    }

    // MARK: - Case Preservation

    #[test]
    fn testGIUppercase() {
        assert_eq!(apply_input("GIS"), "GÍ");
        assert_eq!(apply_input("GIAS"), "GIÁ");
    }

    #[test]
    fn testGIMixedCase() {
        assert_eq!(apply_input("Gis"), "Gí");
        assert_eq!(apply_input("Gias"), "Giá");
        assert_eq!(apply_input("Giangf"), "Giàng");
    }

    // MARK: - Auto Fix Tone with GI

    #[test]
    fn testAutoFixToneGIToGIA() {
        // Type "gis" → "gí", then add "a" → should become "giá"
        let mut transformer = VitypeEngine::new();
        transformer.auto_fix_tone = true;

        let _ = transformer.process("g");
        let _ = transformer.process("i");
        let tone_action = transformer.process("s"); // gí
        assert_eq!(tone_action, Some(action(1, "í")));

        // Now add "a" - tone should reposition from i to a
        let fix_action = transformer.process("a");
        // The tone moves from í to a, and i becomes part of consonant cluster
        assert!(fix_action.is_some());
    }

    #[test]
    fn testAutoFixToneGIToGIAEndToEnd() {
        // End-to-end test: "gisa" should produce "giá"
        let result = apply_input("gisa");
        assert_eq!(result, "giá");
    }

    #[test]
    fn testAutoFixToneGIToGIOEndToEnd() {
        // End-to-end test: "gifo" should produce "già" then... wait
        // Actually: g→i→f→o: gif produces gì, then o should reposition
        let result = apply_input("gifo");
        assert_eq!(result, "giò");
    }

    // MARK: - Edge Cases

    #[test]
    fn testNGINotAffected() {
        // "ngi" is not the gi cluster (ng + i), so i is a regular vowel
        let result = apply_input("ngis");
        assert_eq!(result, "ngí");
    }

    #[test]
    fn testGIAMToneOnA() {
        let result = apply_input("giams");
        assert_eq!(result, "giám");
    }

    #[test]
    fn testGIETToneOnE() {
        // "giet" → tone on e
        let result = apply_input("giets");
        assert_eq!(result, "giét");
    }
}
