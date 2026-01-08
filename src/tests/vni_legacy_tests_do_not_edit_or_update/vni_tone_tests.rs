#![allow(non_snake_case)]

use super::test_helpers::{apply_vni_input, apply_vni_input_with_auto_fix};

// MARK: - VNI Tone Keys (0-5)

mod vni_tone_basic_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniToneAAll() {
        assert_eq!(apply_vni_input("a1"), "á");
        assert_eq!(apply_vni_input("a2"), "à");
        assert_eq!(apply_vni_input("a3"), "ả");
        assert_eq!(apply_vni_input("a4"), "ã");
        assert_eq!(apply_vni_input("a5"), "ạ");
    }

    #[test]
    fn testVniToneEAll() {
        assert_eq!(apply_vni_input("e1"), "é");
        assert_eq!(apply_vni_input("e2"), "è");
        assert_eq!(apply_vni_input("e3"), "ẻ");
        assert_eq!(apply_vni_input("e4"), "ẽ");
        assert_eq!(apply_vni_input("e5"), "ẹ");
    }

    #[test]
    fn testVniToneIAll() {
        assert_eq!(apply_vni_input("i1"), "í");
        assert_eq!(apply_vni_input("i2"), "ì");
        assert_eq!(apply_vni_input("i3"), "ỉ");
        assert_eq!(apply_vni_input("i4"), "ĩ");
        assert_eq!(apply_vni_input("i5"), "ị");
    }

    #[test]
    fn testVniToneOAll() {
        assert_eq!(apply_vni_input("o1"), "ó");
        assert_eq!(apply_vni_input("o2"), "ò");
        assert_eq!(apply_vni_input("o3"), "ỏ");
        assert_eq!(apply_vni_input("o4"), "õ");
        assert_eq!(apply_vni_input("o5"), "ọ");
    }

    #[test]
    fn testVniToneUAll() {
        assert_eq!(apply_vni_input("u1"), "ú");
        assert_eq!(apply_vni_input("u2"), "ù");
        assert_eq!(apply_vni_input("u3"), "ủ");
        assert_eq!(apply_vni_input("u4"), "ũ");
        assert_eq!(apply_vni_input("u5"), "ụ");
    }

    #[test]
    fn testVniToneYAll() {
        assert_eq!(apply_vni_input("y1"), "ý");
        assert_eq!(apply_vni_input("y2"), "ỳ");
        assert_eq!(apply_vni_input("y3"), "ỷ");
        assert_eq!(apply_vni_input("y4"), "ỹ");
        assert_eq!(apply_vni_input("y5"), "ỵ");
    }
}

mod vni_tone_case_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniToneUppercaseBaseVowels() {
        assert_eq!(apply_vni_input("A1"), "Á");
        assert_eq!(apply_vni_input("A2"), "À");
        assert_eq!(apply_vni_input("E3"), "Ẻ");
        assert_eq!(apply_vni_input("I4"), "Ĩ");
        assert_eq!(apply_vni_input("O5"), "Ọ");
        assert_eq!(apply_vni_input("U1"), "Ú");
        assert_eq!(apply_vni_input("Y2"), "Ỳ");
    }

    #[test]
    fn testVniToneMixedCasePreservation() {
        assert_eq!(apply_vni_input("a1B"), "áB");
        assert_eq!(apply_vni_input("A1b"), "Áb");
    }
}

mod vni_tone_replacement_and_removal_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniToneReplacementOnA() {
        assert_eq!(apply_vni_input("a12"), "à"); // á -> à
        assert_eq!(apply_vni_input("a13"), "ả"); // á -> ả
        assert_eq!(apply_vni_input("a14"), "ã"); // á -> ã
        assert_eq!(apply_vni_input("a15"), "ạ"); // á -> ạ
    }

    #[test]
    fn testVniToneReplacementOnTransformedVowel() {
        // â + sắc, then replace to huyền
        assert_eq!(apply_vni_input("a612"), "ầ");
        // ơ + hỏi, then replace to nặng
        assert_eq!(apply_vni_input("o735"), "ợ");
    }

    #[test]
    fn testVniToneRemoval0() {
        assert_eq!(apply_vni_input("a10"), "a");
        assert_eq!(apply_vni_input("e10"), "e");
        assert_eq!(apply_vni_input("o10"), "o");
        assert_eq!(apply_vni_input("u10"), "u");
        assert_eq!(apply_vni_input("y10"), "y");
    }

    #[test]
    fn testVniToneRemovalPreservesVowelTransform() {
        assert_eq!(apply_vni_input("a810"), "ă");
        assert_eq!(apply_vni_input("a610"), "â");
        assert_eq!(apply_vni_input("o710"), "ơ");
        assert_eq!(apply_vni_input("u710"), "ư");
    }
}

mod vni_tone_escape_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniToneEscape11() {
        assert_eq!(apply_vni_input("ta11"), "ta1");
        assert_eq!(apply_vni_input("chan11"), "chan1");
    }

    #[test]
    fn testVniToneEscape22() {
        assert_eq!(apply_vni_input("ta22"), "ta2");
        assert_eq!(apply_vni_input("hoang22"), "hoang2");
    }

    #[test]
    fn testVniToneEscape33() {
        assert_eq!(apply_vni_input("ta33"), "ta3");
        assert_eq!(apply_vni_input("chan33"), "chan3");
    }

    #[test]
    fn testVniToneEscape44() {
        assert_eq!(apply_vni_input("ta44"), "ta4");
        assert_eq!(apply_vni_input("chan44"), "chan4");
    }

    #[test]
    fn testVniToneEscape55() {
        assert_eq!(apply_vni_input("ta55"), "ta5");
        assert_eq!(apply_vni_input("chan55"), "chan5");
    }

    #[test]
    fn testVniRepeatedToneEscapeAppendsLiteral() {
        // tone -> escape -> literal repeats
        assert_eq!(apply_vni_input("chan1"), "chán");
        assert_eq!(apply_vni_input("chan11"), "chan1");
        assert_eq!(apply_vni_input("chan111"), "chan11");
        assert_eq!(apply_vni_input("chan1111"), "chan111");
    }
}

mod vni_auto_fix_tone_tests {
    use super::{apply_vni_input, apply_vni_input_with_auto_fix};

    #[test]
    fn testVniAutoFixToneDisabledHoa2i() {
        // hoa + 2 -> hòa, then i appended => hòai (no reposition)
        assert_eq!(apply_vni_input_with_auto_fix("hoa2i", false), "hòai");
    }

    #[test]
    fn testVniAutoFixToneDisabledHoa2n() {
        // hoa + 2 -> hòa, then n appended => hòan (no reposition)
        assert_eq!(apply_vni_input_with_auto_fix("hoa2n", false), "hòan");
    }

    #[test]
    fn testVniAutoFixToneEnabledHoa2i() {
        assert_eq!(apply_vni_input("hoa2i"), "hoài");
    }

    #[test]
    fn testVniAutoFixToneEnabledHoa2n() {
        assert_eq!(apply_vni_input("hoa2n"), "hoàn");
    }
}


