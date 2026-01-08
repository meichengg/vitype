#![allow(non_snake_case)]

use super::test_helpers::apply_vni_input;

mod vni_word_boundary_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniSpaceIsWordBoundary() {
        assert_eq!(apply_vni_input("a a1"), "a á");
        assert_eq!(apply_vni_input("vie6t5 na8m"), "việt năm");
    }

    #[test]
    fn testVniPunctuationIsWordBoundary() {
        assert_eq!(apply_vni_input("a1,a1"), "á,á");
        assert_eq!(apply_vni_input("a1.a1"), "á.á");
        assert_eq!(apply_vni_input("a1?a1"), "á?á");
        assert_eq!(apply_vni_input("a1!a1"), "á!á");
        assert_eq!(apply_vni_input("a1;a1"), "á;á");
        assert_eq!(apply_vni_input("a1:a1"), "á:á");
    }

    #[test]
    fn testVniNewlineIsWordBoundary() {
        assert_eq!(apply_vni_input("a1\na1"), "á\ná");
    }
}

mod vni_digits_not_boundaries_tests {
    use super::apply_vni_input;

    #[test]
    fn testVniDigitsNotBoundaries_FreeTransformAcrossDigit() {
        // If digits were boundaries, this would become "a06".
        // In VNI mode, 0 is not a boundary and is treated as a key (but with no tone to remove it is literal),
        // so 6 can still free-transform across it.
        assert_eq!(apply_vni_input("a06"), "â0");
    }

    #[test]
    fn testVniDigitsLiteralWhenNoApplicableTransform() {
        assert_eq!(apply_vni_input("2025"), "2025");
        assert_eq!(apply_vni_input("v2025"), "v2025");
    }
}


