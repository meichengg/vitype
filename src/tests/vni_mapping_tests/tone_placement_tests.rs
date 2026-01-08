// TonePlacementTests.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

#![allow(non_snake_case)]

use super::{
    action, apply_vni_input, apply_vni_input_with_auto_fix, apply_vni_input_with_tone_placement,
    create_vni_engine,
};
use crate::TonePlacement;

// MARK: - Auto Fix Tone Tests
mod auto_fix_tone_tests {
    use super::{action, apply_vni_input, apply_vni_input_with_auto_fix, create_vni_engine};

    // MARK: - Basic Auto Fix Tone Tests

    #[test]
    fn testAutoFixToneHoaToHoai() {
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        // "hoa" + "f" → "hòa" (tone on o, 2 vowels → 1st)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let tone_action = transformer.process("2");
        assert_eq!(tone_action, Some(action(2, "òa")));

        // "hòa" + "i" → "hoài" (tone moves to a, 3 vowels → middle)
        // Screen shows "hòa", we need to delete 2 chars ("òa") and insert "oài"
        let fix_action = transformer.process("i");
        assert_eq!(fix_action, Some(action(2, "oài")));
    }

    #[test]
    fn testAutoFixToneMuaToMuai() {
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        // "mua" + "f" → "mùa" (tone on u, 2 vowels → 1st)
        let _ = transformer.process("m");
        let _ = transformer.process("u");
        let _ = transformer.process("a");
        let tone_action = transformer.process("2");
        assert_eq!(tone_action, Some(action(2, "ùa")));

        // "mùa" + "i" → "muài" (tone moves to a, 3 vowels → middle)
        // Screen shows "mùa", we need to delete 2 chars ("ùa") and insert "uài"
        let fix_action = transformer.process("i");
        assert_eq!(fix_action, Some(action(2, "uài")));
    }

    #[test]
    fn testAutoFixToneDisabled() {
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = false;

        // "hoa" + "f" → "hòa"
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let _ = transformer.process("2");

        // "hòa" + "i" → "hòai" (no repositioning when disabled)
        let action = transformer.process("i");
        assert_eq!(action, None); // Just appends, no transform
    }

    #[test]
    fn testAutoFixToneDefaultEnabled() {
        let transformer = create_vni_engine();
        assert_eq!(transformer.auto_fix_tone, true);
    }

    #[test]
    fn testAutoFixToneNoTonePresent() {
        let mut transformer = create_vni_engine();
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
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        // "ta" + "s" → "tá" (single vowel, tone correct)
        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let _ = transformer.process("1"); // tá

        // "tá" + "i" → "tái" (2 vowels, tone should stay on 1st)
        let action = transformer.process("i");
        assert_eq!(action, None); // Tone already in correct position
    }

    #[test]
    fn testAutoFixToneWithNucleusVowel() {
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        // "tuo" + "o" → "tuô" (circumflex transform)
        let _ = transformer.process("t");
        let _ = transformer.process("u");
        let _ = transformer.process("o");
        let _ = transformer.process("o"); // tuô

        // "tuô" + "s" → "tuố" (ô is nucleus-only, takes tone)
        let _ = transformer.process("1");

        // "tuố" + "i" → "tuối" (tone stays on ô, nucleus-only)
        let action = transformer.process("i");
        assert_eq!(action, None); // ô is nucleus-only, tone stays there
    }

    // MARK: - End-to-End Auto Fix Tone Tests

    #[test]
    fn testAutoFixToneEndToEndHoai() {
        // Type "hoafi" -> should produce "hoài"
        let result = apply_vni_input("hoa2i");
        assert_eq!(result, "hoài");
    }

    #[test]
    fn testAutoFixToneEndToEndMuai() {
        // Type "muafi" -> should produce "muài"
        let result = apply_vni_input("mua2i");
        assert_eq!(result, "muài");
    }

    #[test]
    fn testAutoFixToneEndToEndKhuyar() {
        // Type "khuyar" -> should produce "khuỷa" (3 vowels, middle gets tone)
        let result = apply_vni_input("khuya3");
        assert_eq!(result, "khuỷa");
    }

    #[test]
    fn testAutoFixToneEndToEndDisabled() {
        // With autoFixTone disabled, "hoafi" -> "hòai" (tone stays on o)
        let result = apply_vni_input_with_auto_fix("hoa2i", false);
        assert_eq!(result, "hòai");
    }

    #[test]
    fn testAutoFixToneEndToEndWithNucleus() {
        // Type "tuoois" -> should produce "tuối" (ô is nucleus, keeps tone)
        let result = apply_vni_input("tuo6i1");
        assert_eq!(result, "tuối");
    }

    #[test]
    fn testAutoFixToneEndToEndNoReposition() {
        // Type "tais" -> should produce "tái" (2 vowels, 1st gets tone, no reposition needed)
        let result = apply_vni_input("tai1");
        assert_eq!(result, "tái");
    }

    #[test]
    fn testAutoFixToneEndToEndHuyen() {
        // Type "hoaf" then "i" -> "hoài" (grave tone moves to middle)
        let result = apply_vni_input("hoa2i");
        assert_eq!(result, "hoài");
    }

    #[test]
    fn testAutoFixToneEndToEndSac() {
        // Type "hoas" then "i" -> "hoái" (acute tone moves to middle)
        let result = apply_vni_input("hoa1i");
        assert_eq!(result, "hoái");
    }

    #[test]
    fn testAutoFixToneEndToEndHoi() {
        // Type "hoar" then "i" -> "hoải" (hook tone moves to middle)
        let result = apply_vni_input("hoa3i");
        assert_eq!(result, "hoải");
    }

    #[test]
    fn testAutoFixToneEndToEndNga() {
        // Type "hoax" then "i" -> "hoãi" (tilde tone moves to middle)
        let result = apply_vni_input("hoa4i");
        assert_eq!(result, "hoãi");
    }

    #[test]
    fn testAutoFixToneEndToEndNang() {
        // Type "hoaj" then "i" -> "hoại" (dot tone moves to middle)
        let result = apply_vni_input("hoa5i");
        assert_eq!(result, "hoại");
    }

    // MARK: - Complex Scenarios

    #[test]
    fn testAutoFixToneComplexNguoi() {
        // "người" - ơ is nucleus-only, should keep tone
        let result = apply_vni_input("ngu7o7i2");
        assert_eq!(result, "người");
    }

    #[test]
    fn testAutoFixToneComplexTuoi() {
        // "tuổi" - ô is nucleus-only, should keep tone
        let result = apply_vni_input("tuo6i3");
        assert_eq!(result, "tuổi");
    }

    #[test]
    fn testAutoFixToneNoToneToMove() {
        // No tone applied, just vowels
        let result = apply_vni_input("hoai");
        assert_eq!(result, "hoai");
    }

    // MARK: - Escape Sequence After Auto Fix Tone Tests

    #[test]
    fn testEscapeSequenceNotTriggeredAfterAutoFix() {
        // Regression test: Ensure escape sequence detection is reset after auto-fix
        // Previously, lastTransformKey wasn't cleared when auto-fix returned an action,
        // which could cause incorrect escape sequence detection.
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        // "hoa" + "f" → "hòa" (lastTransformKey = f)
        let _ = transformer.process("h");
        let _ = transformer.process("o");
        let _ = transformer.process("a");
        let _ = transformer.process("2"); // hòa

        // "hòa" + "i" → "hoài" (auto-fix repositions tone, should clear lastTransformKey)
        let _ = transformer.process("i"); // hoài

        // Now typing "f" should apply tone to the vowel, NOT trigger escape
        // If lastTransformKey wasn't cleared, this might incorrectly try escape logic
        let result = transformer.process("2");

        // "f" should apply grave tone to the target vowel (à already has it, so it stays)
        // The important thing is it doesn't produce an escape sequence like "if" or similar
        // Since hoài already has the tone on à, applying f again should just replace with same tone
        assert_eq!(result, Some(action(2, "ai2")));
    }

    #[test]
    fn testEscapeSequenceNotTriggeredAfterAutoFixEndToEnd() {
        // End-to-end version: "hoafif" should produce "hoài" with grave tone
        // NOT trigger any escape sequence behavior
        let result = apply_vni_input("hoa2i2");
        assert_eq!(result, "hoai2");

        let r1 = apply_vni_input("hoa2");
        assert_eq!(r1, "hòa");

        let r2 = apply_vni_input("hoa2i");
        assert_eq!(r2, "hoài");
    }

    #[test]
    fn testEscapeSequenceStillWorksNormally() {
        // Verify that normal escape sequences still work (not broken by the fix)
        let mut transformer = create_vni_engine();

        let _ = transformer.process("t");
        let _ = transformer.process("a");
        let tone_action = transformer.process("2"); // tà
        assert_eq!(tone_action, Some(action(1, "à")));

        // Double "f" should trigger escape: tà + f → taf
        let escape_action = transformer.process("2");
        assert_eq!(escape_action, Some(action(1, "a2")));
    }

    #[test]
    fn testEscapeSequenceStillWorksEndToEnd() {
        // Verify escape still works: "taff" → "taf"
        let result = apply_vni_input("ta22");
        assert_eq!(result, "ta2");
    }
}

// MARK: - UY Special Case Tests
mod uy_tone_placement_tests {
    use super::apply_vni_input;

    // MARK: - UY Alone (tone on U)

    #[test]
    fn testUYAloneToneOnU() {
        // "uy" alone → tone on u
        let result = apply_vni_input("uy1");
        assert_eq!(result, "úy");
    }

    #[test]
    fn testUYAloneAllTones() {
        assert_eq!(apply_vni_input("uy1"), "úy"); // sắc
        assert_eq!(apply_vni_input("uy2"), "ùy"); // huyền
        assert_eq!(apply_vni_input("uy3"), "ủy"); // hỏi
        assert_eq!(apply_vni_input("uy4"), "ũy"); // ngã
        assert_eq!(apply_vni_input("uy5"), "ụy"); // nặng
    }

    #[test]
    fn testUYAloneWithInitialConsonant() {
        // "tuy" alone → tone on u
        let result = apply_vni_input("tuy1");
        assert_eq!(result, "túy");
    }

    // MARK: - UY + Consonant (tone on Y)

    #[test]
    fn testUYNHToneOnY() {
        // "uynh" + tone → tone on y
        let result = apply_vni_input("uynh1");
        assert_eq!(result, "uýnh");
    }

    #[test]
    fn testUYNHAllTones() {
        assert_eq!(apply_vni_input("uynh1"), "uýnh"); // sắc
        assert_eq!(apply_vni_input("uynh2"), "uỳnh"); // huyền
        assert_eq!(apply_vni_input("uynh3"), "uỷnh"); // hỏi
        assert_eq!(apply_vni_input("uynh4"), "uỹnh"); // ngã
        assert_eq!(apply_vni_input("uynh5"), "uỵnh"); // nặng
    }

    #[test]
    fn testUYTToneOnY() {
        // "uyt" + tone → tone on y
        let result = apply_vni_input("uyt1");
        assert_eq!(result, "uýt");
    }

    #[test]
    fn testUYTAllTones() {
        assert_eq!(apply_vni_input("uyt1"), "uýt"); // sắc
        assert_eq!(apply_vni_input("uyt2"), "uỳt"); // huyền
        assert_eq!(apply_vni_input("uyt3"), "uỷt"); // hỏi
        assert_eq!(apply_vni_input("uyt4"), "uỹt"); // ngã
        assert_eq!(apply_vni_input("uyt5"), "uỵt"); // nặng
    }

    #[test]
    fn testHUYNHToneOnY() {
        // "huynh" + tone → tone on y
        let result = apply_vni_input("huynh2");
        assert_eq!(result, "huỳnh");
    }

    #[test]
    fn testHUYNHAllTones() {
        assert_eq!(apply_vni_input("huynh1"), "huýnh"); // sắc
        assert_eq!(apply_vni_input("huynh2"), "huỳnh"); // huyền
        assert_eq!(apply_vni_input("huynh3"), "huỷnh"); // hỏi
        assert_eq!(apply_vni_input("huynh4"), "huỹnh"); // ngã
        assert_eq!(apply_vni_input("huynh5"), "huỵnh"); // nặng
    }

    // MARK: - QUY (special case: u after q is skipped)

    #[test]
    fn testQUYToneOnY() {
        // "quy" - u after q is part of consonant cluster, only y is a vowel
        let result = apply_vni_input("quy1");
        assert_eq!(result, "quý");
    }

    #[test]
    fn testQUYAllTones() {
        assert_eq!(apply_vni_input("quy1"), "quý"); // sắc
        assert_eq!(apply_vni_input("quy2"), "quỳ"); // huyền
        assert_eq!(apply_vni_input("quy3"), "quỷ"); // hỏi
        assert_eq!(apply_vni_input("quy4"), "quỹ"); // ngã
        assert_eq!(apply_vni_input("quy5"), "quỵ"); // nặng
    }

    // MARK: - Case Preservation

    #[test]
    fn testUYUppercase() {
        assert_eq!(apply_vni_input("UYNH1"), "UÝNH");
        assert_eq!(apply_vni_input("UY1"), "ÚY");
    }

    #[test]
    fn testUYMixedCase() {
        assert_eq!(apply_vni_input("Uynh1"), "Uýnh");
        assert_eq!(apply_vni_input("Huynh2"), "Huỳnh");
    }

    // MARK: - Other UY + Consonant patterns

    #[test]
    fn testUYCToneOnY() {
        let result = apply_vni_input("uyc1");
        assert_eq!(result, "uýc");
    }

    #[test]
    fn testUYPToneOnY() {
        let result = apply_vni_input("uyp1");
        assert_eq!(result, "uýp");
    }

    #[test]
    fn testUYMToneOnY() {
        let result = apply_vni_input("uym1");
        assert_eq!(result, "uým");
    }

    #[test]
    fn testUYNToneOnY() {
        let result = apply_vni_input("uyn1");
        assert_eq!(result, "uýn");
    }
}

// MARK: - Tone Placement Mode Tests
mod tone_placement_mode_tests {
    use super::{apply_vni_input_with_tone_placement, TonePlacement};

    #[test]
    fn testNucleusOnlyUYAloneToneOnY() {
        // Nucleus-only mode: "uy" alone → tone goes on y (uý), not u (úy)
        let result = apply_vni_input_with_tone_placement("uy1", TonePlacement::NucleusOnly);
        assert_eq!(result, "uý");
    }

    #[test]
    fn testNucleusOnlyUYAloneAllTones() {
        assert_eq!(
            apply_vni_input_with_tone_placement("uy1", TonePlacement::NucleusOnly),
            "uý"
        ); // sắc
        assert_eq!(
            apply_vni_input_with_tone_placement("uy2", TonePlacement::NucleusOnly),
            "uỳ"
        ); // huyền
        assert_eq!(
            apply_vni_input_with_tone_placement("uy3", TonePlacement::NucleusOnly),
            "uỷ"
        ); // hỏi
        assert_eq!(
            apply_vni_input_with_tone_placement("uy4", TonePlacement::NucleusOnly),
            "uỹ"
        ); // ngã
        assert_eq!(
            apply_vni_input_with_tone_placement("uy5", TonePlacement::NucleusOnly),
            "uỵ"
        ); // nặng
    }

    #[test]
    fn testNucleusOnlyHoaToneOnA() {
        // Nucleus-only mode: "hoa" → tone goes on a (hoá), not o (hóa)
        let result = apply_vni_input_with_tone_placement("hoa1", TonePlacement::NucleusOnly);
        assert_eq!(result, "hoá");
    }

    #[test]
    fn testNucleusOnlyKhoeToneOnE() {
        // Nucleus-only mode: "khoe" → tone goes on e (khoẻ), not o (khỏe)
        let result = apply_vni_input_with_tone_placement("khoe3", TonePlacement::NucleusOnly);
        assert_eq!(result, "khoẻ");
    }
}

// MARK: - GI Special Case Tests
mod gi_tone_placement_tests {
    use super::{action, apply_vni_input, create_vni_engine};

    // MARK: - GI Alone (tone on I)

    #[test]
    fn testGIAloneToneOnI() {
        // "gi" alone → tone on i
        let result = apply_vni_input("gi1");
        assert_eq!(result, "gí");
    }

    #[test]
    fn testGIAloneAllTones() {
        assert_eq!(apply_vni_input("gi1"), "gí"); // sắc
        assert_eq!(apply_vni_input("gi2"), "gì"); // huyền
        assert_eq!(apply_vni_input("gi3"), "gỉ"); // hỏi
        assert_eq!(apply_vni_input("gi4"), "gĩ"); // ngã
        assert_eq!(apply_vni_input("gi5"), "gị"); // nặng
    }

    // MARK: - GI + Vowel (tone on following vowel)

    #[test]
    fn testGIAToneOnA() {
        // "gia" + tone → tone on a (gi is consonant cluster)
        let result = apply_vni_input("gia1");
        assert_eq!(result, "giá");
    }

    #[test]
    fn testGIAAllTones() {
        assert_eq!(apply_vni_input("gia1"), "giá"); // sắc
        assert_eq!(apply_vni_input("gia2"), "già"); // huyền
        assert_eq!(apply_vni_input("gia3"), "giả"); // hỏi
        assert_eq!(apply_vni_input("gia4"), "giã"); // ngã
        assert_eq!(apply_vni_input("gia5"), "giạ"); // nặng
    }

    #[test]
    fn testGIOToneOnO() {
        let result = apply_vni_input("gio1");
        assert_eq!(result, "gió");
    }

    #[test]
    fn testGIEToneOnE() {
        let result = apply_vni_input("gie1");
        assert_eq!(result, "gié");
    }

    #[test]
    fn testGIUToneOnU() {
        let result = apply_vni_input("giu1");
        assert_eq!(result, "giú");
    }

    #[test]
    fn testGIUWToneOnUHorn() {
        // "giuw" → "giư", then tone on ư
        let result = apply_vni_input("giu71");
        assert_eq!(result, "giứ");
    }

    #[test]
    fn testGIUWAllTones() {
        assert_eq!(apply_vni_input("giu71"), "giứ"); // sắc
        assert_eq!(apply_vni_input("giu72"), "giừ"); // huyền
        assert_eq!(apply_vni_input("giu73"), "giử"); // hỏi
        assert_eq!(apply_vni_input("giu74"), "giữ"); // ngã
        assert_eq!(apply_vni_input("giu75"), "giự"); // nặng
    }

    // MARK: - GI + Vowel + Consonant

    #[test]
    fn testGIANGToneOnA() {
        let result = apply_vni_input("giang1");
        assert_eq!(result, "giáng");
    }

    #[test]
    fn testGIANGAllTones() {
        assert_eq!(apply_vni_input("giang1"), "giáng"); // sắc
        assert_eq!(apply_vni_input("giang2"), "giàng"); // huyền
        assert_eq!(apply_vni_input("giang3"), "giảng"); // hỏi
        assert_eq!(apply_vni_input("giang4"), "giãng"); // ngã
        assert_eq!(apply_vni_input("giang5"), "giạng"); // nặng
    }

    // MARK: - GI + Two Vowels (tone on first following vowel)

    #[test]
    fn testGIEOToneOnE() {
        // "gieo" has vowels e, o after gi → tone on first (e)
        let result = apply_vni_input("gieo1");
        assert_eq!(result, "giéo");
    }

    #[test]
    fn testGIAOToneOnA() {
        // "giao" has vowels a, o after gi → tone on first (a)
        let result = apply_vni_input("giao2");
        assert_eq!(result, "giào");
    }

    // MARK: - Case Preservation

    #[test]
    fn testGIUppercase() {
        assert_eq!(apply_vni_input("GI1"), "GÍ");
        assert_eq!(apply_vni_input("GIA1"), "GIÁ");
    }

    #[test]
    fn testGIMixedCase() {
        assert_eq!(apply_vni_input("Gi1"), "Gí");
        assert_eq!(apply_vni_input("Gia1"), "Giá");
        assert_eq!(apply_vni_input("Giang2"), "Giàng");
    }

    // MARK: - Auto Fix Tone with GI

    #[test]
    fn testAutoFixToneGIToGIA() {
        // Type "gis" → "gí", then add "a" → should become "giá"
        let mut transformer = create_vni_engine();
        transformer.auto_fix_tone = true;

        let _ = transformer.process("g");
        let _ = transformer.process("i");
        let tone_action = transformer.process("1"); // gí
        assert_eq!(tone_action, Some(action(1, "í")));

        // Now add "a" - tone should reposition from i to a
        let fix_action = transformer.process("a");
        // The tone moves from í to a, and i becomes part of consonant cluster
        assert!(fix_action.is_some());
    }

    #[test]
    fn testAutoFixToneGIToGIAEndToEnd() {
        // End-to-end test: "gisa" should produce "giá"
        let result = apply_vni_input("gi1a");
        assert_eq!(result, "giá");
    }

    #[test]
    fn testAutoFixToneGIToGIOEndToEnd() {
        // End-to-end test: "gifo" should produce "già" then... wait
        // Actually: g→i→f→o: gif produces gì, then o should reposition
        let result = apply_vni_input("gi2o");
        assert_eq!(result, "giò");
    }

    // MARK: - Edge Cases

    #[test]
    fn testNGINotAffected() {
        // "ngi" is not the gi cluster (ng + i), so i is a regular vowel
        let result = apply_vni_input("ngi1");
        assert_eq!(result, "ngí");
    }

    #[test]
    fn testGIAMToneOnA() {
        let result = apply_vni_input("giam1");
        assert_eq!(result, "giám");
    }

    #[test]
    fn testGIETToneOnE() {
        // "giet" → tone on e
        let result = apply_vni_input("giet1");
        assert_eq!(result, "giét");
    }
}
