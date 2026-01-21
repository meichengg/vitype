#![allow(non_snake_case)]

use super::test_helpers::{apply_vni_input, apply_vni_input_with_tone_placement};

// MARK: - VNI Tone Placement (same algorithm as Telex, different keys)

mod vni_uy_special_case_tests {
    use super::apply_vni_input;

    #[test]
    fn testUYAloneToneOnU() {
        assert_eq!(apply_vni_input("uy1"), "úy");
        assert_eq!(apply_vni_input("uy2"), "ùy");
        assert_eq!(apply_vni_input("uy3"), "ủy");
        assert_eq!(apply_vni_input("uy4"), "ũy");
        assert_eq!(apply_vni_input("uy5"), "ụy");
    }

    #[test]
    fn testUYNHToneOnY() {
        assert_eq!(apply_vni_input("uynh1"), "uýnh");
        assert_eq!(apply_vni_input("uynh2"), "uỳnh");
        assert_eq!(apply_vni_input("uynh3"), "uỷnh");
        assert_eq!(apply_vni_input("uynh4"), "uỹnh");
        assert_eq!(apply_vni_input("uynh5"), "uỵnh");
    }

    #[test]
    fn testHUYNHToneOnY() {
        assert_eq!(apply_vni_input("huynh2"), "huỳnh");
    }

    #[test]
    fn testQUYToneOnY() {
        assert_eq!(apply_vni_input("quy1"), "quý");
        assert_eq!(apply_vni_input("quy2"), "quỳ");
        assert_eq!(apply_vni_input("quy3"), "quỷ");
        assert_eq!(apply_vni_input("quy4"), "quỹ");
        assert_eq!(apply_vni_input("quy5"), "quỵ");
    }
}

mod vni_gi_special_case_tests {
    use super::apply_vni_input;

    #[test]
    fn testGIAloneToneOnI() {
        assert_eq!(apply_vni_input("gi1"), "gí");
        assert_eq!(apply_vni_input("gi2"), "gì");
        assert_eq!(apply_vni_input("gi3"), "gỉ");
        assert_eq!(apply_vni_input("gi4"), "gĩ");
        assert_eq!(apply_vni_input("gi5"), "gị");
    }

    #[test]
    fn testGIAVowelsSkipI() {
        assert_eq!(apply_vni_input("gia1"), "giá");
        assert_eq!(apply_vni_input("gia2"), "già");
        assert_eq!(apply_vni_input("gio1"), "gió");
        assert_eq!(apply_vni_input("gie1"), "gié");
    }

    #[test]
    fn testGIANGToneOnA() {
        assert_eq!(apply_vni_input("giang2"), "giàng");
        assert_eq!(apply_vni_input("giang1"), "giáng");
    }

    #[test]
    fn testNGINotAffected() {
        assert_eq!(apply_vni_input("ngi1"), "ngí");
    }
}

mod vni_qu_cluster_tests {
    use super::apply_vni_input;

    #[test]
    fn testQuaToneSkipsUAfterQ() {
        assert_eq!(apply_vni_input("qua1"), "quá");
        assert_eq!(apply_vni_input("qua2"), "quà");
    }

    #[test]
    fn testQuyen() {
        assert_eq!(apply_vni_input("quye6n2"), "quyền");
        assert_eq!(apply_vni_input("quye6n1"), "quyến");
    }
}

mod vni_nucleus_only_tests {
    use super::apply_vni_input;

    #[test]
    fn testTieng() {
        assert_eq!(apply_vni_input("tie6ng1"), "tiếng");
    }

    #[test]
    fn testTuoiNucleusOnlyO() {
        assert_eq!(apply_vni_input("tuo6i1"), "tuối");
        assert_eq!(apply_vni_input("tuo6i3"), "tuổi");
    }
}

mod vni_tone_placement_mode_tests {
    use crate::TonePlacement;

    use super::apply_vni_input_with_tone_placement;

    #[test]
    fn testNucleusOnlyUYAloneToneOnY() {
        assert_eq!(
            apply_vni_input_with_tone_placement("uy1", TonePlacement::NucleusOnly),
            "uý"
        );
    }

    #[test]
    fn testNucleusOnlyHoaToneOnA() {
        assert_eq!(
            apply_vni_input_with_tone_placement("hoa1", TonePlacement::NucleusOnly),
            "hoá"
        );
    }

    #[test]
    fn testNucleusOnlyKhoeToneOnE() {
        assert_eq!(
            apply_vni_input_with_tone_placement("khoe3", TonePlacement::NucleusOnly),
            "khoẻ"
        );
    }
}
