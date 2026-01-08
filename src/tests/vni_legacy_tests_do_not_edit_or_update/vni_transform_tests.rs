#![allow(non_snake_case)]

use super::test_helpers::apply_vni_input;

// MARK: - VNI Transform Keys (6, 7, 8, 9)

mod vni_consonant_d9_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniD9Basic() {
        assert_eq!(apply_vni_input("d9"), "đ");
        assert_eq!(apply_vni_input("D9"), "Đ");
    }

    #[test]
    fn testVniD9FreeTransformWithin4() {
        assert_eq!(apply_vni_input("di9"), "đi");
        assert_eq!(apply_vni_input("dai9"), "đai");
        assert_eq!(apply_vni_input("de9"), "đe");
        assert_eq!(apply_vni_input("do9"), "đo");
    }

    #[test]
    fn testVniD9Escape99() {
        assert_eq!(apply_vni_input("d99"), "d9");
        assert_eq!(apply_vni_input("D99"), "D9");
    }

    #[test]
    fn testVniD9RepeatedEscapeAppendsLiteral() {
        assert_eq!(apply_vni_input("d999"), "d99");
        assert_eq!(apply_vni_input("d9999"), "d999");
    }
}

mod vni_vowel_transform_basic_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniCircumflex6() {
        assert_eq!(apply_vni_input("a6"), "â");
        assert_eq!(apply_vni_input("e6"), "ê");
        assert_eq!(apply_vni_input("o6"), "ô");
    }

    #[test]
    fn testVniHorn7() {
        assert_eq!(apply_vni_input("o7"), "ơ");
        assert_eq!(apply_vni_input("u7"), "ư");
    }

    #[test]
    fn testVniBreve8() {
        assert_eq!(apply_vni_input("a8"), "ă");
    }

    #[test]
    fn testVniTransformUppercase() {
        assert_eq!(apply_vni_input("A6"), "Â");
        assert_eq!(apply_vni_input("E6"), "Ê");
        assert_eq!(apply_vni_input("O6"), "Ô");
        assert_eq!(apply_vni_input("O7"), "Ơ");
        assert_eq!(apply_vni_input("U7"), "Ư");
        assert_eq!(apply_vni_input("A8"), "Ă");
    }

    #[test]
    fn testVniStandaloneDigitsAreLiteral() {
        assert_eq!(apply_vni_input("6"), "6");
        assert_eq!(apply_vni_input("7"), "7");
        assert_eq!(apply_vni_input("8"), "8");
        assert_eq!(apply_vni_input("9"), "9");
    }

    #[test]
    fn testVniNonTransformableVowelsOutputLiteralKey() {
        assert_eq!(apply_vni_input("i6"), "i6");
        assert_eq!(apply_vni_input("y6"), "y6");
        assert_eq!(apply_vni_input("e7"), "e7");
        assert_eq!(apply_vni_input("a7"), "a7");
        assert_eq!(apply_vni_input("o8"), "o8");
        assert_eq!(apply_vni_input("u8"), "u8");
    }
}

mod vni_free_transform_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniFreeTransformThroughConsonant() {
        assert_eq!(apply_vni_input("that6"), "thât");
        assert_eq!(apply_vni_input("thet6"), "thêt");
        assert_eq!(apply_vni_input("thot7"), "thơt");
        assert_eq!(apply_vni_input("nam8"), "năm");
    }

    #[test]
    fn testVniFreeTransformStopsAtInterveningVowel() {
        // The current implementation allows skipping ONE trailing vowel (distance=1),
        // but stops if it hits a non-transformable vowel further back (distance > 1).
        assert_eq!(apply_vni_input("oi6"), "ôi"); // skip i (distance=1), transform o -> ô
        assert_eq!(apply_vni_input("uie7"), "uie7"); // stops at i (distance=2), so 7 is literal
        assert_eq!(apply_vni_input("oia7"), "oia7"); // stops at i (distance=2), so 7 is literal
    }
}

mod vni_escape_tests_for_transforms {
    use super::apply_vni_input;

    #[test]
    fn testVniEscape66() {
        assert_eq!(apply_vni_input("a66"), "a6");
        assert_eq!(apply_vni_input("e66"), "e6");
        assert_eq!(apply_vni_input("o66"), "o6");
    }

    #[test]
    fn testVniEscape77() {
        assert_eq!(apply_vni_input("o77"), "o7");
        assert_eq!(apply_vni_input("u77"), "u7");
    }

    #[test]
    fn testVniEscape88() {
        assert_eq!(apply_vni_input("a88"), "a8");
    }

    #[test]
    fn testVniRepeatedEscapeAppendsLiteral() {
        assert_eq!(apply_vni_input("a666"), "a66");
        assert_eq!(apply_vni_input("o777"), "o77");
        assert_eq!(apply_vni_input("a888"), "a88");
    }
}

mod vni_override_transform_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniOverrideBaseVowel() {
        // base -> circumflex -> horn
        assert_eq!(apply_vni_input("o67"), "ơ");
        // base -> horn -> circumflex
        assert_eq!(apply_vni_input("o76"), "ô");
        // base -> circumflex -> breve
        assert_eq!(apply_vni_input("a68"), "ă");
        // base -> breve -> circumflex
        assert_eq!(apply_vni_input("a86"), "â");
    }

    #[test]
    fn testVniOverrideTonedVowelPreservesTone() {
        // ă + sắc -> ắ, then 6 changes to â + sắc -> ấ
        assert_eq!(apply_vni_input("a816"), "ấ");
        // ơ + sắc -> ớ, then 6 changes to ô + sắc -> ố
        assert_eq!(apply_vni_input("o716"), "ố");
        // ô + sắc -> ố, then 7 changes to ơ + sắc -> ớ
        assert_eq!(apply_vni_input("o617"), "ớ");
    }

    #[test]
    fn testVniNoOverrideLiteralWhenNotSupported() {
        // 7 can't transform â/ê
        assert_eq!(apply_vni_input("a67"), "â7");
        assert_eq!(apply_vni_input("e67"), "ê7");
        // 8 can't transform o/u
        assert_eq!(apply_vni_input("o78"), "ơ8");
        assert_eq!(apply_vni_input("u78"), "ư8");
    }
}

mod vni_compound_7_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniCompoundUo7() {
        assert_eq!(apply_vni_input("uo7"), "ươ");
        assert_eq!(apply_vni_input("UO7"), "ƯƠ");
        assert_eq!(apply_vni_input("Uo7"), "Ươ");
    }

    #[test]
    fn testVniCompoundVariants() {
        assert_eq!(apply_vni_input("huu7"), "hưu");
        assert_eq!(apply_vni_input("hou7"), "hươ");
        assert_eq!(apply_vni_input("huou7"), "hươu");
        assert_eq!(apply_vni_input("mua7"), "mưa");
    }

    #[test]
    fn testVniCompoundEscape() {
        assert_eq!(apply_vni_input("uo77"), "uo7");
        assert_eq!(apply_vni_input("uoc77"), "uoc7");
    }

    #[test]
    fn testVniCompoundWithFinalConsonantBefore7() {
        assert_eq!(apply_vni_input("uoc7"), "ươc");
        assert_eq!(apply_vni_input("uoc71"), "ước");
    }

    #[test]
    fn testVniQuClusterNoCompound() {
        assert_eq!(apply_vni_input("quo7"), "quơ");
        assert_eq!(apply_vni_input("quoc7"), "quơc");
    }
}


