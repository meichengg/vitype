#![allow(non_snake_case)]

use super::test_helpers::apply_vni_input;

// MARK: - Ensure Telex keys behave as literal characters in VNI mode

mod vni_telex_key_literal_tests {
    use super::apply_vni_input;

    #[test]
    fn testTelexToneKeysAreLiteralInVni() {
        // In VNI mode, tone keys are digits, so Telex tone letters should be literal.
        assert_eq!(apply_vni_input("tas"), "tas");
        assert_eq!(apply_vni_input("taf"), "taf");
        assert_eq!(apply_vni_input("tar"), "tar");
        assert_eq!(apply_vni_input("tax"), "tax");
        assert_eq!(apply_vni_input("taj"), "taj");
        assert_eq!(apply_vni_input("taz"), "taz");
    }

    #[test]
    fn testTelexWIsLiteralInVni() {
        // In Telex, w has standalone behavior; in VNI it should be a normal letter.
        assert_eq!(apply_vni_input("w"), "w");
        assert_eq!(apply_vni_input("wa"), "wa");
        assert_eq!(apply_vni_input("wifi"), "wifi");
        assert_eq!(apply_vni_input("wweb"), "wweb");
    }

    #[test]
    fn testTelexCompositeSequencesAreLiteralInVni() {
        // Telex sequences should not transform in VNI mode.
        assert_eq!(apply_vni_input("dd"), "dd");
        assert_eq!(apply_vni_input("aa"), "aa");
        assert_eq!(apply_vni_input("ee"), "ee");
        assert_eq!(apply_vni_input("oo"), "oo");
        assert_eq!(apply_vni_input("aw"), "aw");
        assert_eq!(apply_vni_input("ow"), "ow");
        assert_eq!(apply_vni_input("uw"), "uw");
    }
}


