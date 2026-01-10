#![allow(non_snake_case)]

use super::apply_vni_input;

// MARK: - Foreign Consonant Tests (z, w, j, f)
mod foreign_consonant_tests {
    use super::apply_vni_input;

    // MARK: - Z as Consonant

    #[test]
    fn testZAsConsonantBasic() {
        assert_eq!(apply_vni_input("za"), "za");
        assert_eq!(apply_vni_input("ze"), "ze");
        assert_eq!(apply_vni_input("zi"), "zi");
        assert_eq!(apply_vni_input("zo"), "zo");
        assert_eq!(apply_vni_input("zu"), "zu");
    }

    #[test]
    fn testZAsConsonantWithToneSac() {
        assert_eq!(apply_vni_input("za1"), "zá");
        assert_eq!(apply_vni_input("ze1"), "zé");
        assert_eq!(apply_vni_input("zi1"), "zí");
        assert_eq!(apply_vni_input("zo1"), "zó");
        assert_eq!(apply_vni_input("zu1"), "zú");
    }

    #[test]
    fn testZAsConsonantWithToneHuyen() {
        assert_eq!(apply_vni_input("za2"), "zà");
        assert_eq!(apply_vni_input("ze2"), "zè");
        assert_eq!(apply_vni_input("zi2"), "zì");
        assert_eq!(apply_vni_input("zo2"), "zò");
        assert_eq!(apply_vni_input("zu2"), "zù");
    }

    #[test]
    fn testZAsConsonantWithToneHoi() {
        assert_eq!(apply_vni_input("za3"), "zả");
        assert_eq!(apply_vni_input("ze3"), "zẻ");
        assert_eq!(apply_vni_input("zi3"), "zỉ");
        assert_eq!(apply_vni_input("zo3"), "zỏ");
        assert_eq!(apply_vni_input("zu3"), "zủ");
    }

    #[test]
    fn testZAsConsonantWithToneNga() {
        assert_eq!(apply_vni_input("za4"), "zã");
        assert_eq!(apply_vni_input("ze4"), "zẽ");
        assert_eq!(apply_vni_input("zi4"), "zĩ");
        assert_eq!(apply_vni_input("zo4"), "zõ");
        assert_eq!(apply_vni_input("zu4"), "zũ");
    }

    #[test]
    fn testZAsConsonantWithToneNang() {
        assert_eq!(apply_vni_input("za5"), "zạ");
        assert_eq!(apply_vni_input("ze5"), "zẹ");
        assert_eq!(apply_vni_input("zi5"), "zị");
        assert_eq!(apply_vni_input("zo5"), "zọ");
        assert_eq!(apply_vni_input("zu5"), "zụ");
    }

    #[test]
    fn testZAsConsonantWithCircumflexVowels() {
        assert_eq!(apply_vni_input("za61"), "zấ");
        assert_eq!(apply_vni_input("ze61"), "zế");
        assert_eq!(apply_vni_input("zo61"), "zố");
        assert_eq!(apply_vni_input("ze62"), "zề");
    }

    #[test]
    fn testZAsConsonantWithHornVowels() {
        assert_eq!(apply_vni_input("za81"), "zắ");
        assert_eq!(apply_vni_input("zo71"), "zớ");
        assert_eq!(apply_vni_input("zu71"), "zứ");
    }

    #[test]
    fn testZAsConsonantWithComplexVowelClusters() {
        assert_eq!(apply_vni_input("zao1"), "záo");
        assert_eq!(apply_vni_input("zuynh2"), "zuỳnh");
        assert_eq!(apply_vni_input("zie6u1"), "ziếu");
    }

    #[test]
    fn testZAsConsonantUppercase() {
        assert_eq!(apply_vni_input("ZA"), "ZA");
        assert_eq!(apply_vni_input("ZA1"), "ZÁ");
        assert_eq!(apply_vni_input("Za1"), "Zá");
        assert_eq!(apply_vni_input("ZE62"), "ZỀ");
    }

    #[test]
    fn testZAsConsonantVsToneRemoval() {
        assert_eq!(apply_vni_input("a10"), "a");
        assert_eq!(apply_vni_input("za"), "za");
    }

    #[test]
    fn testZAsConsonantWithFinalConsonant() {
        assert_eq!(apply_vni_input("zam1"), "zám");
        assert_eq!(apply_vni_input("zang1"), "záng");
        assert_eq!(apply_vni_input("zanh2"), "zành");
    }

    // MARK: - J as Consonant

    #[test]
    fn testJAsConsonantBasic() {
        assert_eq!(apply_vni_input("ja"), "ja");
        assert_eq!(apply_vni_input("je"), "je");
        assert_eq!(apply_vni_input("ji"), "ji");
        assert_eq!(apply_vni_input("jo"), "jo");
        assert_eq!(apply_vni_input("ju"), "ju");
    }

    #[test]
    fn testJAsConsonantWithToneSac() {
        assert_eq!(apply_vni_input("ja1"), "já");
        assert_eq!(apply_vni_input("je1"), "jé");
        assert_eq!(apply_vni_input("ji1"), "jí");
        assert_eq!(apply_vni_input("jo1"), "jó");
        assert_eq!(apply_vni_input("ju1"), "jú");
    }

    #[test]
    fn testJAsConsonantWithToneHuyen() {
        assert_eq!(apply_vni_input("ja2"), "jà");
        assert_eq!(apply_vni_input("je2"), "jè");
        assert_eq!(apply_vni_input("ji2"), "jì");
        assert_eq!(apply_vni_input("jo2"), "jò");
        assert_eq!(apply_vni_input("ju2"), "jù");
    }

    #[test]
    fn testJAsConsonantWithToneHoi() {
        assert_eq!(apply_vni_input("ja3"), "jả");
        assert_eq!(apply_vni_input("je3"), "jẻ");
        assert_eq!(apply_vni_input("ji3"), "jỉ");
        assert_eq!(apply_vni_input("jo3"), "jỏ");
        assert_eq!(apply_vni_input("ju3"), "jủ");
    }

    #[test]
    fn testJAsConsonantWithToneNga() {
        assert_eq!(apply_vni_input("ja4"), "jã");
        assert_eq!(apply_vni_input("je4"), "jẽ");
        assert_eq!(apply_vni_input("ji4"), "jĩ");
        assert_eq!(apply_vni_input("jo4"), "jõ");
        assert_eq!(apply_vni_input("ju4"), "jũ");
    }

    #[test]
    fn testJAsConsonantWithToneNang() {
        assert_eq!(apply_vni_input("ja5"), "jạ");
        assert_eq!(apply_vni_input("je5"), "jẹ");
        assert_eq!(apply_vni_input("ji5"), "jị");
        assert_eq!(apply_vni_input("jo5"), "jọ");
        assert_eq!(apply_vni_input("ju5"), "jụ");
    }

    #[test]
    fn testJAsConsonantWithCircumflexVowels() {
        assert_eq!(apply_vni_input("ja61"), "jấ");
        assert_eq!(apply_vni_input("je61"), "jế");
        assert_eq!(apply_vni_input("jo61"), "jố");
        assert_eq!(apply_vni_input("je62"), "jề");
    }

    #[test]
    fn testJAsConsonantWithHornVowels() {
        assert_eq!(apply_vni_input("ja81"), "jắ");
        assert_eq!(apply_vni_input("jo71"), "jớ");
        assert_eq!(apply_vni_input("ju71"), "jứ");
    }

    #[test]
    fn testJAsConsonantUppercase() {
        assert_eq!(apply_vni_input("JA"), "JA");
        assert_eq!(apply_vni_input("JA1"), "JÁ");
        assert_eq!(apply_vni_input("Ja1"), "Já");
        assert_eq!(apply_vni_input("JE62"), "JỀ");
    }

    #[test]
    fn testJAsConsonantVsToneApplication() {
        assert_eq!(apply_vni_input("a5"), "ạ");
        assert_eq!(apply_vni_input("ja"), "ja");
    }

    #[test]
    fn testJAsConsonantWithFinalConsonant() {
        assert_eq!(apply_vni_input("jam1"), "jám");
        assert_eq!(apply_vni_input("jang1"), "jáng");
        assert_eq!(apply_vni_input("janh2"), "jành");
    }

    // MARK: - F as Consonant

    #[test]
    fn testFAsConsonantBasic() {
        assert_eq!(apply_vni_input("fa"), "fa");
        assert_eq!(apply_vni_input("fe"), "fe");
        assert_eq!(apply_vni_input("fi"), "fi");
        assert_eq!(apply_vni_input("fo"), "fo");
        assert_eq!(apply_vni_input("fu"), "fu");
    }

    #[test]
    fn testFAsConsonantWithToneSac() {
        assert_eq!(apply_vni_input("fa1"), "fá");
        assert_eq!(apply_vni_input("fe1"), "fé");
        assert_eq!(apply_vni_input("fi1"), "fí");
        assert_eq!(apply_vni_input("fo1"), "fó");
        assert_eq!(apply_vni_input("fu1"), "fú");
    }

    #[test]
    fn testFAsConsonantWithToneHuyen() {
        assert_eq!(apply_vni_input("fa2"), "fà");
        assert_eq!(apply_vni_input("fe2"), "fè");
        assert_eq!(apply_vni_input("fi2"), "fì");
        assert_eq!(apply_vni_input("fo2"), "fò");
        assert_eq!(apply_vni_input("fu2"), "fù");
    }

    #[test]
    fn testFAsConsonantWithToneHoi() {
        assert_eq!(apply_vni_input("fa3"), "fả");
        assert_eq!(apply_vni_input("fe3"), "fẻ");
        assert_eq!(apply_vni_input("fi3"), "fỉ");
        assert_eq!(apply_vni_input("fo3"), "fỏ");
        assert_eq!(apply_vni_input("fu3"), "fủ");
    }

    #[test]
    fn testFAsConsonantWithToneNga() {
        assert_eq!(apply_vni_input("fa4"), "fã");
        assert_eq!(apply_vni_input("fe4"), "fẽ");
        assert_eq!(apply_vni_input("fi4"), "fĩ");
        assert_eq!(apply_vni_input("fo4"), "fõ");
        assert_eq!(apply_vni_input("fu4"), "fũ");
    }

    #[test]
    fn testFAsConsonantWithToneNang() {
        assert_eq!(apply_vni_input("fa5"), "fạ");
        assert_eq!(apply_vni_input("fe5"), "fẹ");
        assert_eq!(apply_vni_input("fi5"), "fị");
        assert_eq!(apply_vni_input("fo5"), "fọ");
        assert_eq!(apply_vni_input("fu5"), "fụ");
    }

    #[test]
    fn testFAsConsonantWithCircumflexVowels() {
        assert_eq!(apply_vni_input("fa61"), "fấ");
        assert_eq!(apply_vni_input("fe61"), "fế");
        assert_eq!(apply_vni_input("fo61"), "fố");
        assert_eq!(apply_vni_input("fe62"), "fề");
    }

    #[test]
    fn testFAsConsonantWithHornVowels() {
        assert_eq!(apply_vni_input("fa81"), "fắ");
        assert_eq!(apply_vni_input("fo71"), "fớ");
        assert_eq!(apply_vni_input("fu71"), "fứ");
    }

    #[test]
    fn testFAsConsonantUppercase() {
        assert_eq!(apply_vni_input("FA"), "FA");
        assert_eq!(apply_vni_input("FA1"), "FÁ");
        assert_eq!(apply_vni_input("Fa1"), "Fá");
        assert_eq!(apply_vni_input("FE62"), "FỀ");
    }

    #[test]
    fn testFAsConsonantVsToneApplication() {
        assert_eq!(apply_vni_input("a2"), "à");
        assert_eq!(apply_vni_input("fa"), "fa");
    }

    #[test]
    fn testFAsConsonantWithFinalConsonant() {
        assert_eq!(apply_vni_input("fam1"), "fám");
        assert_eq!(apply_vni_input("fang1"), "fáng");
        assert_eq!(apply_vni_input("fanh2"), "fành");
    }

    // MARK: - W as Consonant (literal in VNI)

    #[test]
    fn testWAsConsonantAfterEscapeBasic() {
        assert_eq!(apply_vni_input("wa"), "wa");
        assert_eq!(apply_vni_input("we"), "we");
        assert_eq!(apply_vni_input("wi"), "wi");
        assert_eq!(apply_vni_input("wo"), "wo");
        assert_eq!(apply_vni_input("wu"), "wu");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneSac() {
        assert_eq!(apply_vni_input("wa1"), "wá");
        assert_eq!(apply_vni_input("we1"), "wé");
        assert_eq!(apply_vni_input("wi1"), "wí");
        assert_eq!(apply_vni_input("wo1"), "wó");
        assert_eq!(apply_vni_input("wu1"), "wú");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneHuyen() {
        assert_eq!(apply_vni_input("wa2"), "wà");
        assert_eq!(apply_vni_input("we2"), "wè");
        assert_eq!(apply_vni_input("wi2"), "wì");
        assert_eq!(apply_vni_input("wo2"), "wò");
        assert_eq!(apply_vni_input("wu2"), "wù");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneHoi() {
        assert_eq!(apply_vni_input("wa3"), "wả");
        assert_eq!(apply_vni_input("we3"), "wẻ");
        assert_eq!(apply_vni_input("wi3"), "wỉ");
        assert_eq!(apply_vni_input("wo3"), "wỏ");
        assert_eq!(apply_vni_input("wu3"), "wủ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneNga() {
        assert_eq!(apply_vni_input("wa4"), "wã");
        assert_eq!(apply_vni_input("we4"), "wẽ");
        assert_eq!(apply_vni_input("wi4"), "wĩ");
        assert_eq!(apply_vni_input("wo4"), "wõ");
        assert_eq!(apply_vni_input("wu4"), "wũ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneNang() {
        assert_eq!(apply_vni_input("wa5"), "wạ");
        assert_eq!(apply_vni_input("we5"), "wẹ");
        assert_eq!(apply_vni_input("wi5"), "wị");
        assert_eq!(apply_vni_input("wo5"), "wọ");
        assert_eq!(apply_vni_input("wu5"), "wụ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithCircumflexVowels() {
        assert_eq!(apply_vni_input("wa61"), "wấ");
        assert_eq!(apply_vni_input("we61"), "wế");
        assert_eq!(apply_vni_input("wo61"), "wố");
        assert_eq!(apply_vni_input("we62"), "wề");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithHornVowels() {
        assert_eq!(apply_vni_input("wa81"), "wắ");
        assert_eq!(apply_vni_input("wo71"), "wớ");
        assert_eq!(apply_vni_input("wu71"), "wứ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeUppercase() {
        assert_eq!(apply_vni_input("WA"), "WA");
        assert_eq!(apply_vni_input("WA1"), "WÁ");
        assert_eq!(apply_vni_input("Wa1"), "Wá");
        assert_eq!(apply_vni_input("WE62"), "WỀ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithFinalConsonant() {
        assert_eq!(apply_vni_input("wam1"), "wám");
        assert_eq!(apply_vni_input("wang1"), "wáng");
        assert_eq!(apply_vni_input("wanh2"), "wành");
    }

    #[test]
    fn testWAsConsonantAfterEscapeRealWords() {
        assert_eq!(apply_vni_input("web"), "web");
    }

    #[test]
    fn testWAsConsonantVsStandaloneW() {
        assert_eq!(apply_vni_input("u7a"), "ưa");
        assert_eq!(apply_vni_input("wa"), "wa");
    }

    #[test]
    fn testWAsConsonantThenWTransform() {
        assert_eq!(apply_vni_input("wu7"), "wư");
        assert_eq!(apply_vni_input("wa8"), "wă");
        assert_eq!(apply_vni_input("wo7"), "wơ");
    }

    // MARK: - Mixed Foreign Consonants

    #[test]
    fn testZWithWTransform() {
        assert_eq!(apply_vni_input("zu7a"), "zưa");
        assert_eq!(apply_vni_input("zu7"), "zư");
        assert_eq!(apply_vni_input("za8"), "ză");
        assert_eq!(apply_vni_input("zo7"), "zơ");
    }

    #[test]
    fn testJWithWTransform() {
        assert_eq!(apply_vni_input("ju7a"), "jưa");
        assert_eq!(apply_vni_input("ju7"), "jư");
        assert_eq!(apply_vni_input("ja8"), "jă");
        assert_eq!(apply_vni_input("jo7"), "jơ");
    }

    #[test]
    fn testFWithWTransform() {
        assert_eq!(apply_vni_input("fu7a"), "fưa");
        assert_eq!(apply_vni_input("fu7"), "fư");
        assert_eq!(apply_vni_input("fa8"), "fă");
        assert_eq!(apply_vni_input("fo7"), "fơ");
    }

    // MARK: - Tone Key Position Distinction

    #[test]
    fn testToneKeyPositionZ() {
        assert_eq!(apply_vni_input("za"), "za");
        assert_eq!(apply_vni_input("za1"), "zá");
        assert_eq!(apply_vni_input("a10"), "a");
        assert_eq!(apply_vni_input("a20"), "a");
    }

    #[test]
    fn testToneKeyPositionJ() {
        assert_eq!(apply_vni_input("ja"), "ja");
        assert_eq!(apply_vni_input("ja1"), "já");
        assert_eq!(apply_vni_input("a5"), "ạ");
        assert_eq!(apply_vni_input("ta5"), "tạ");
    }

    #[test]
    fn testToneKeyPositionF() {
        assert_eq!(apply_vni_input("fa"), "fa");
        assert_eq!(apply_vni_input("fa1"), "fá");
        assert_eq!(apply_vni_input("a2"), "à");
        assert_eq!(apply_vni_input("ta2"), "tà");
    }

    // MARK: - Multiple Tone Keys in Sequence

    #[test]
    fn testZFollowedByVowelThenToneReplacement() {
        assert_eq!(apply_vni_input("za12"), "zà");
        assert_eq!(apply_vni_input("za13"), "zả");
        assert_eq!(apply_vni_input("za14"), "zã");
        assert_eq!(apply_vni_input("za15"), "zạ");
    }

    #[test]
    fn testJFollowedByVowelThenToneReplacement() {
        assert_eq!(apply_vni_input("ja12"), "jà");
        assert_eq!(apply_vni_input("ja13"), "jả");
        assert_eq!(apply_vni_input("ja14"), "jã");
        assert_eq!(apply_vni_input("ja15"), "jạ");
    }

    #[test]
    fn testFFollowedByVowelThenToneReplacement() {
        assert_eq!(apply_vni_input("fa12"), "fà");
        assert_eq!(apply_vni_input("fa13"), "fả");
        assert_eq!(apply_vni_input("fa14"), "fã");
        assert_eq!(apply_vni_input("fa15"), "fạ");
    }
}

// MARK: - Repeated Escape Sequence Tests
mod repeated_escape_tests {
    use super::apply_vni_input;

    #[test]
    fn testRepeatedToneEscapeSac() {
        assert_eq!(apply_vni_input("chan1"), "chán");
        assert_eq!(apply_vni_input("chan11"), "chan1");
        assert_eq!(apply_vni_input("chan111"), "chan11");
        assert_eq!(apply_vni_input("chan1111"), "chan111");
        assert_eq!(apply_vni_input("chan11111"), "chan1111");
    }

    #[test]
    fn testRepeatedToneEscapeHuyen() {
        assert_eq!(apply_vni_input("ta2"), "tà");
        assert_eq!(apply_vni_input("ta22"), "ta2");
        assert_eq!(apply_vni_input("ta222"), "ta22");
        assert_eq!(apply_vni_input("ta2222"), "ta222");
        assert_eq!(apply_vni_input("ta22222"), "ta2222");
    }

    #[test]
    fn testRepeatedToneEscapeHoi() {
        assert_eq!(apply_vni_input("ta3"), "tả");
        assert_eq!(apply_vni_input("ta33"), "ta3");
        assert_eq!(apply_vni_input("ta333"), "ta33");
        assert_eq!(apply_vni_input("ta3333"), "ta333");
        assert_eq!(apply_vni_input("ta33333"), "ta3333");
    }

    #[test]
    fn testRepeatedToneEscapeNga() {
        assert_eq!(apply_vni_input("ta4"), "tã");
        assert_eq!(apply_vni_input("ta44"), "ta4");
        assert_eq!(apply_vni_input("ta444"), "ta44");
        assert_eq!(apply_vni_input("ta4444"), "ta444");
        assert_eq!(apply_vni_input("ta44444"), "ta4444");
    }

    #[test]
    fn testRepeatedToneEscapeNang() {
        assert_eq!(apply_vni_input("ta5"), "tạ");
        assert_eq!(apply_vni_input("ta55"), "ta5");
        assert_eq!(apply_vni_input("ta555"), "ta55");
        assert_eq!(apply_vni_input("ta5555"), "ta555");
        assert_eq!(apply_vni_input("ta55555"), "ta5555");
    }

    #[test]
    fn testRepeatedToneEscapeVowelE() {
        assert_eq!(apply_vni_input("te1"), "té");
        assert_eq!(apply_vni_input("te11"), "te1");
        assert_eq!(apply_vni_input("te111"), "te11");
        assert_eq!(apply_vni_input("te1111"), "te111");
    }

    #[test]
    fn testRepeatedToneEscapeVowelI() {
        assert_eq!(apply_vni_input("ti1"), "tí");
        assert_eq!(apply_vni_input("ti11"), "ti1");
        assert_eq!(apply_vni_input("ti111"), "ti11");
        assert_eq!(apply_vni_input("ti1111"), "ti111");
    }

    #[test]
    fn testRepeatedToneEscapeVowelO() {
        assert_eq!(apply_vni_input("to1"), "tó");
        assert_eq!(apply_vni_input("to11"), "to1");
        assert_eq!(apply_vni_input("to111"), "to11");
        assert_eq!(apply_vni_input("to1111"), "to111");
    }

    #[test]
    fn testRepeatedToneEscapeVowelU() {
        assert_eq!(apply_vni_input("tu1"), "tú");
        assert_eq!(apply_vni_input("tu11"), "tu1");
        assert_eq!(apply_vni_input("tu111"), "tu11");
        assert_eq!(apply_vni_input("tu1111"), "tu111");
    }

    #[test]
    fn testRepeatedToneEscapeVowelY() {
        assert_eq!(apply_vni_input("ty1"), "tý");
        assert_eq!(apply_vni_input("ty11"), "ty1");
        assert_eq!(apply_vni_input("ty111"), "ty11");
        assert_eq!(apply_vni_input("ty1111"), "ty111");
    }

    #[test]
    fn testRepeatedToneEscapeCircumflexA() {
        assert_eq!(apply_vni_input("a61"), "ấ");
        assert_eq!(apply_vni_input("a611"), "â1");
        assert_eq!(apply_vni_input("a6111"), "â11");
        assert_eq!(apply_vni_input("a61111"), "â111");
        assert_eq!(apply_vni_input("a611111"), "â1111");
    }

    #[test]
    fn testRepeatedToneEscapeCircumflexE() {
        assert_eq!(apply_vni_input("e61"), "ế");
        assert_eq!(apply_vni_input("e611"), "ê1");
        assert_eq!(apply_vni_input("e6111"), "ê11");
        assert_eq!(apply_vni_input("e61111"), "ê111");
    }

    #[test]
    fn testRepeatedToneEscapeCircumflexO() {
        assert_eq!(apply_vni_input("o61"), "ố");
        assert_eq!(apply_vni_input("o611"), "ô1");
        assert_eq!(apply_vni_input("o6111"), "ô11");
        assert_eq!(apply_vni_input("o61111"), "ô111");
    }

    #[test]
    fn testRepeatedToneEscapeBreveA() {
        assert_eq!(apply_vni_input("a81"), "ắ");
        assert_eq!(apply_vni_input("a811"), "ă1");
        assert_eq!(apply_vni_input("a8111"), "ă11");
        assert_eq!(apply_vni_input("a81111"), "ă111");
    }

    #[test]
    fn testRepeatedToneEscapeHornO() {
        assert_eq!(apply_vni_input("o71"), "ớ");
        assert_eq!(apply_vni_input("o711"), "ơ1");
        assert_eq!(apply_vni_input("o7111"), "ơ11");
        assert_eq!(apply_vni_input("o71111"), "ơ111");
    }

    #[test]
    fn testRepeatedToneEscapeHornU() {
        assert_eq!(apply_vni_input("u71"), "ứ");
        assert_eq!(apply_vni_input("u711"), "ư1");
        assert_eq!(apply_vni_input("u7111"), "ư11");
        assert_eq!(apply_vni_input("u71111"), "ư111");
    }

    #[test]
    fn testRepeatedVowelEscapeAA() {
        assert_eq!(apply_vni_input("a6"), "â");
        assert_eq!(apply_vni_input("a66"), "a6");
        assert_eq!(apply_vni_input("a666"), "a66");
        assert_eq!(apply_vni_input("a6666"), "a666");
        assert_eq!(apply_vni_input("a66666"), "a6666");
    }

    #[test]
    fn testRepeatedVowelEscapeEE() {
        assert_eq!(apply_vni_input("e6"), "ê");
        assert_eq!(apply_vni_input("e66"), "e6");
        assert_eq!(apply_vni_input("e666"), "e66");
        assert_eq!(apply_vni_input("e6666"), "e666");
    }

    #[test]
    fn testRepeatedVowelEscapeOO() {
        assert_eq!(apply_vni_input("o6"), "ô");
        assert_eq!(apply_vni_input("o66"), "o6");
        assert_eq!(apply_vni_input("o666"), "o66");
        assert_eq!(apply_vni_input("o6666"), "o666");
    }

    #[test]
    fn testRepeatedVowelEscapeAW() {
        assert_eq!(apply_vni_input("a8"), "ă");
        assert_eq!(apply_vni_input("a88"), "a8");
        assert_eq!(apply_vni_input("a888"), "a88");
        assert_eq!(apply_vni_input("a8888"), "a888");
        assert_eq!(apply_vni_input("a88888"), "a8888");
    }

    #[test]
    fn testRepeatedVowelEscapeOW() {
        assert_eq!(apply_vni_input("o7"), "ơ");
        assert_eq!(apply_vni_input("o77"), "o7");
        assert_eq!(apply_vni_input("o777"), "o77");
        assert_eq!(apply_vni_input("o7777"), "o777");
    }

    #[test]
    fn testRepeatedVowelEscapeUW() {
        assert_eq!(apply_vni_input("u7"), "ư");
        assert_eq!(apply_vni_input("u77"), "u7");
        assert_eq!(apply_vni_input("u777"), "u77");
        assert_eq!(apply_vni_input("u7777"), "u777");
    }

    #[test]
    fn testRepeatedConsonantEscapeDD() {
        assert_eq!(apply_vni_input("d9"), "đ");
        assert_eq!(apply_vni_input("d99"), "d9");
        assert_eq!(apply_vni_input("d999"), "d99");
        assert_eq!(apply_vni_input("d9999"), "d999");
        assert_eq!(apply_vni_input("d99999"), "d9999");
    }

    #[test]
    fn testConsonantEscapeDDMidWordLiteral() {
        assert_eq!(apply_vni_input("ad9"), "ad9");
        assert_eq!(apply_vni_input("ad9d"), "ad9d");
    }

    #[test]
    fn testRepeatedConsonantEscapeDDUppercase() {
        assert_eq!(apply_vni_input("D9"), "Đ");
        assert_eq!(apply_vni_input("D99"), "D9");
        assert_eq!(apply_vni_input("D999"), "D99");
        assert_eq!(apply_vni_input("D9999"), "D999");
    }

    #[test]
    fn testRepeatedStandaloneWEscape() {
        assert_eq!(apply_vni_input("u7"), "ư");
        assert_eq!(apply_vni_input("u77"), "u7");
        assert_eq!(apply_vni_input("u777"), "u77");
        assert_eq!(apply_vni_input("u7777"), "u777");
        assert_eq!(apply_vni_input("u77777"), "u7777");
    }

    #[test]
    fn testRepeatedStandaloneWEscapeAfterConsonant() {
        assert_eq!(apply_vni_input("tu7"), "tư");
        assert_eq!(apply_vni_input("tu77"), "tu7");
        assert_eq!(apply_vni_input("tu777"), "tu77");
        assert_eq!(apply_vni_input("tu7777"), "tu777");

        assert_eq!(apply_vni_input("device"), "device");
        assert_eq!(apply_vni_input("device1a6"), "device1a6");
    }

    #[test]
    fn testRepeatedEscapeInWordChan() {
        assert_eq!(apply_vni_input("chan11"), "chan1");
        assert_eq!(apply_vni_input("chan111"), "chan11");
        assert_eq!(apply_vni_input("chan1111"), "chan111");
    }

    #[test]
    fn testRepeatedEscapeInWordViet() {
        assert_eq!(apply_vni_input("vie65"), "việ");
        assert_eq!(apply_vni_input("vie655"), "viê5");
        assert_eq!(apply_vni_input("vie6555"), "viê55");
    }

    #[test]
    fn testRepeatedEscapeInWordDi() {
        assert_eq!(apply_vni_input("d9i"), "đi");
        assert_eq!(apply_vni_input("d99i"), "d9i");
        assert_eq!(apply_vni_input("d999i"), "d99i");
    }

    #[test]
    fn testMixedKeysAfterToneEscape() {
        assert_eq!(apply_vni_input("ta11t"), "ta1t");
        assert_eq!(apply_vni_input("ta11n"), "ta1n");
    }

    #[test]
    fn testMixedKeysAfterVowelEscape() {
        assert_eq!(apply_vni_input("a66t"), "a6t");
        assert_eq!(apply_vni_input("a66n"), "a6n");
    }

    #[test]
    fn testMixedKeysAfterConsonantEscape() {
        assert_eq!(apply_vni_input("d99a"), "d9a");
        assert_eq!(apply_vni_input("d99e"), "d9e");
    }

    #[test]
    fn testNoToneOrAccentAfterEscape() {
        // After escaping a transform key, all subsequent transform keys in the same word
        // should be treated as literal until a word boundary.
        assert_eq!(apply_vni_input("ta112"), "ta12"); // ta1 + 2 (literal), not tà1
        assert_eq!(apply_vni_input("a661"), "a61"); // a6 + 1 (literal), not ấ
        assert_eq!(apply_vni_input("d99a1"), "d9a1"); // d9 escaped, then 1 stays literal
    }

    #[test]
    fn testForeignModeLiteralTransformKeys() {
        assert_eq!(apply_vni_input("aba1"), "aba1");
        assert_eq!(apply_vni_input("aba6"), "aba6");
        assert_eq!(apply_vni_input("abad9"), "abad9");
    }

    #[test]
    fn testReTransformAfterWordBoundary() {
        assert_eq!(apply_vni_input("ta11 ta1"), "ta1 tá");
        assert_eq!(apply_vni_input("a66 a6"), "a6 â");
        assert_eq!(apply_vni_input("d99 d9"), "d9 đ");
    }

    #[test]
    fn testRepeatedEscapeCasePreservationTone() {
        assert_eq!(apply_vni_input("TA1"), "TÁ");
        assert_eq!(apply_vni_input("TA11"), "TA1");
        assert_eq!(apply_vni_input("TA111"), "TA11");
    }

    #[test]
    fn testRepeatedEscapeCasePreservationVowel() {
        assert_eq!(apply_vni_input("A6"), "Â");
        assert_eq!(apply_vni_input("A66"), "A6");
        assert_eq!(apply_vni_input("A666"), "A66");
    }
}
