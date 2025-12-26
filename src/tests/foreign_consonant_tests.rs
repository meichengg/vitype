// ForeignConsonantTests.swift
// vnkeyTests
//
// Created by Tran Dat on 24/12/25.

#![allow(non_snake_case)]

use super::test_helpers::apply_input;

// MARK: - Foreign Consonant Tests (z, w, j, f)
mod foreign_consonant_tests {
    use super::apply_input;

    // MARK: - Z as Consonant
    // z behaves like a normal consonant (n, m, k) when it appears BEFORE a vowel
    // When z comes AFTER a vowel, it removes tone (existing behavior)

    #[test]
    fn testZAsConsonantBasic() {
        // z + vowel = z as consonant
        assert_eq!(apply_input("za"), "za");
        assert_eq!(apply_input("ze"), "ze");
        assert_eq!(apply_input("zi"), "zi");
        assert_eq!(apply_input("zo"), "zo");
        assert_eq!(apply_input("zu"), "zu");
    }

    #[test]
    fn testZAsConsonantWithToneSac() {
        // z consonant + vowel + sắc tone
        assert_eq!(apply_input("zas"), "zá");
        assert_eq!(apply_input("zes"), "zé");
        assert_eq!(apply_input("zis"), "zí");
        assert_eq!(apply_input("zos"), "zó");
        assert_eq!(apply_input("zus"), "zú");
    }

    #[test]
    fn testZAsConsonantWithToneHuyen() {
        // z consonant + vowel + huyền tone
        assert_eq!(apply_input("zaf"), "zà");
        assert_eq!(apply_input("zef"), "zè");
        assert_eq!(apply_input("zif"), "zì");
        assert_eq!(apply_input("zof"), "zò");
        assert_eq!(apply_input("zuf"), "zù");
    }

    #[test]
    fn testZAsConsonantWithToneHoi() {
        // z consonant + vowel + hỏi tone
        assert_eq!(apply_input("zar"), "zả");
        assert_eq!(apply_input("zer"), "zẻ");
        assert_eq!(apply_input("zir"), "zỉ");
        assert_eq!(apply_input("zor"), "zỏ");
        assert_eq!(apply_input("zur"), "zủ");
    }

    #[test]
    fn testZAsConsonantWithToneNga() {
        // z consonant + vowel + ngã tone
        assert_eq!(apply_input("zax"), "zã");
        assert_eq!(apply_input("zex"), "zẽ");
        assert_eq!(apply_input("zix"), "zĩ");
        assert_eq!(apply_input("zox"), "zõ");
        assert_eq!(apply_input("zux"), "zũ");
    }

    #[test]
    fn testZAsConsonantWithToneNang() {
        // z consonant + vowel + nặng tone
        assert_eq!(apply_input("zaj"), "zạ");
        assert_eq!(apply_input("zej"), "zẹ");
        assert_eq!(apply_input("zij"), "zị");
        assert_eq!(apply_input("zoj"), "zọ");
        assert_eq!(apply_input("zuj"), "zụ");
    }

    #[test]
    fn testZAsConsonantWithCircumflexVowels() {
        // z consonant + circumflex vowels (â, ê, ô)
        assert_eq!(apply_input("zaas"), "zấ");
        assert_eq!(apply_input("zees"), "zế");
        assert_eq!(apply_input("zoos"), "zố");
        assert_eq!(apply_input("zeef"), "zề");
    }

    #[test]
    fn testZAsConsonantWithHornVowels() {
        // z consonant + horn/breve vowels (ă, ơ, ư)
        assert_eq!(apply_input("zaws"), "zắ");
        assert_eq!(apply_input("zows"), "zớ");
        assert_eq!(apply_input("zuws"), "zứ");
    }

    #[test]
    fn testZAsConsonantWithComplexVowelClusters() {
        // z consonant + complex vowel clusters
        assert_eq!(apply_input("zaos"), "záo"); // záo
        assert_eq!(apply_input("zuynhf"), "zuỳnh"); // zuỳnh
        assert_eq!(apply_input("zieeus"), "ziếu"); // ziếu
    }

    #[test]
    fn testZAsConsonantUppercase() {
        assert_eq!(apply_input("ZA"), "ZA");
        assert_eq!(apply_input("ZAS"), "ZÁ");
        assert_eq!(apply_input("Zas"), "Zá");
        assert_eq!(apply_input("ZEEF"), "ZỀ");
    }

    #[test]
    fn testZAsConsonantVsToneRemoval() {
        // z AFTER vowel removes tone (existing behavior)
        assert_eq!(apply_input("asz"), "a");
        // z BEFORE vowel is consonant
        assert_eq!(apply_input("za"), "za"); // z is consonant
    }

    #[test]
    fn testZAsConsonantWithFinalConsonant() {
        // z consonant + vowel + final consonant
        assert_eq!(apply_input("zams"), "zám");
        assert_eq!(apply_input("zangs"), "záng");
        assert_eq!(apply_input("zanhf"), "zành");
    }

    // MARK: - J as Consonant
    // j behaves like a normal consonant when it appears BEFORE a vowel
    // When j comes AFTER a vowel, it applies nặng tone (existing behavior)

    #[test]
    fn testJAsConsonantBasic() {
        // j + vowel = j as consonant
        assert_eq!(apply_input("ja"), "ja");
        assert_eq!(apply_input("je"), "je");
        assert_eq!(apply_input("ji"), "ji");
        assert_eq!(apply_input("jo"), "jo");
        assert_eq!(apply_input("ju"), "ju");
    }

    #[test]
    fn testJAsConsonantWithToneSac() {
        assert_eq!(apply_input("jas"), "já");
        assert_eq!(apply_input("jes"), "jé");
        assert_eq!(apply_input("jis"), "jí");
        assert_eq!(apply_input("jos"), "jó");
        assert_eq!(apply_input("jus"), "jú");
    }

    #[test]
    fn testJAsConsonantWithToneHuyen() {
        assert_eq!(apply_input("jaf"), "jà");
        assert_eq!(apply_input("jef"), "jè");
        assert_eq!(apply_input("jif"), "jì");
        assert_eq!(apply_input("jof"), "jò");
        assert_eq!(apply_input("juf"), "jù");
    }

    #[test]
    fn testJAsConsonantWithToneHoi() {
        assert_eq!(apply_input("jar"), "jả");
        assert_eq!(apply_input("jer"), "jẻ");
        assert_eq!(apply_input("jir"), "jỉ");
        assert_eq!(apply_input("jor"), "jỏ");
        assert_eq!(apply_input("jur"), "jủ");
    }

    #[test]
    fn testJAsConsonantWithToneNga() {
        assert_eq!(apply_input("jax"), "jã");
        assert_eq!(apply_input("jex"), "jẽ");
        assert_eq!(apply_input("jix"), "jĩ");
        assert_eq!(apply_input("jox"), "jõ");
        assert_eq!(apply_input("jux"), "jũ");
    }

    #[test]
    fn testJAsConsonantWithToneNang() {
        // j as consonant, then j as tone key
        assert_eq!(apply_input("jaj"), "jạ");
        assert_eq!(apply_input("jej"), "jẹ");
        assert_eq!(apply_input("jij"), "jị");
        assert_eq!(apply_input("joj"), "jọ");
        assert_eq!(apply_input("juj"), "jụ");
    }

    #[test]
    fn testJAsConsonantWithCircumflexVowels() {
        assert_eq!(apply_input("jaas"), "jấ");
        assert_eq!(apply_input("jees"), "jế");
        assert_eq!(apply_input("joos"), "jố");
        assert_eq!(apply_input("jeef"), "jề");
    }

    #[test]
    fn testJAsConsonantWithHornVowels() {
        assert_eq!(apply_input("jaws"), "jắ");
        assert_eq!(apply_input("jows"), "jớ");
        assert_eq!(apply_input("juws"), "jứ");
    }

    #[test]
    fn testJAsConsonantUppercase() {
        assert_eq!(apply_input("JA"), "JA");
        assert_eq!(apply_input("JAS"), "JÁ");
        assert_eq!(apply_input("Jas"), "Já");
        assert_eq!(apply_input("JEEF"), "JỀ");
    }

    #[test]
    fn testJAsConsonantVsToneApplication() {
        // j AFTER vowel applies nặng tone (existing behavior)
        assert_eq!(apply_input("aj"), "ạ");
        // j BEFORE vowel is consonant
        assert_eq!(apply_input("ja"), "ja");
    }

    #[test]
    fn testJAsConsonantWithFinalConsonant() {
        assert_eq!(apply_input("jams"), "jám");
        assert_eq!(apply_input("jangs"), "jáng");
        assert_eq!(apply_input("janhf"), "jành");
    }

    // MARK: - F as Consonant
    // f behaves like a normal consonant when it appears BEFORE a vowel
    // When f comes AFTER a vowel, it applies huyền tone (existing behavior)

    #[test]
    fn testFAsConsonantBasic() {
        // f + vowel = f as consonant
        assert_eq!(apply_input("fa"), "fa");
        assert_eq!(apply_input("fe"), "fe");
        assert_eq!(apply_input("fi"), "fi");
        assert_eq!(apply_input("fo"), "fo");
        assert_eq!(apply_input("fu"), "fu");
    }

    #[test]
    fn testFAsConsonantWithToneSac() {
        assert_eq!(apply_input("fas"), "fá");
        assert_eq!(apply_input("fes"), "fé");
        assert_eq!(apply_input("fis"), "fí");
        assert_eq!(apply_input("fos"), "fó");
        assert_eq!(apply_input("fus"), "fú");
    }

    #[test]
    fn testFAsConsonantWithToneHuyen() {
        // f as consonant, then f as tone key
        assert_eq!(apply_input("faf"), "fà");
        assert_eq!(apply_input("fef"), "fè");
        assert_eq!(apply_input("fif"), "fì");
        assert_eq!(apply_input("fof"), "fò");
        assert_eq!(apply_input("fuf"), "fù");
    }

    #[test]
    fn testFAsConsonantWithToneHoi() {
        assert_eq!(apply_input("far"), "fả");
        assert_eq!(apply_input("fer"), "fẻ");
        assert_eq!(apply_input("fir"), "fỉ");
        assert_eq!(apply_input("for"), "fỏ");
        assert_eq!(apply_input("fur"), "fủ");
    }

    #[test]
    fn testFAsConsonantWithToneNga() {
        assert_eq!(apply_input("fax"), "fã");
        assert_eq!(apply_input("fex"), "fẽ");
        assert_eq!(apply_input("fix"), "fĩ");
        assert_eq!(apply_input("fox"), "fõ");
        assert_eq!(apply_input("fux"), "fũ");
    }

    #[test]
    fn testFAsConsonantWithToneNang() {
        assert_eq!(apply_input("faj"), "fạ");
        assert_eq!(apply_input("fej"), "fẹ");
        assert_eq!(apply_input("fij"), "fị");
        assert_eq!(apply_input("foj"), "fọ");
        assert_eq!(apply_input("fuj"), "fụ");
    }

    #[test]
    fn testFAsConsonantWithCircumflexVowels() {
        assert_eq!(apply_input("faas"), "fấ");
        assert_eq!(apply_input("fees"), "fế");
        assert_eq!(apply_input("foos"), "fố");
        assert_eq!(apply_input("feef"), "fề");
    }

    #[test]
    fn testFAsConsonantWithHornVowels() {
        assert_eq!(apply_input("faws"), "fắ");
        assert_eq!(apply_input("fows"), "fớ");
        assert_eq!(apply_input("fuws"), "fứ");
    }

    #[test]
    fn testFAsConsonantUppercase() {
        assert_eq!(apply_input("FA"), "FA");
        assert_eq!(apply_input("FAS"), "FÁ");
        assert_eq!(apply_input("Fas"), "Fá");
        assert_eq!(apply_input("FEEF"), "FỀ");
    }

    #[test]
    fn testFAsConsonantVsToneApplication() {
        // f AFTER vowel applies huyền tone (existing behavior)
        assert_eq!(apply_input("af"), "à");
        // f BEFORE vowel is consonant
        assert_eq!(apply_input("fa"), "fa");
    }

    #[test]
    fn testFAsConsonantWithFinalConsonant() {
        assert_eq!(apply_input("fams"), "fám");
        assert_eq!(apply_input("fangs"), "fáng");
        assert_eq!(apply_input("fanhf"), "fành");
    }

    // MARK: - W as Consonant (after ww escape)
    // w -> ư (standalone)
    // ww -> w (escape, w is suppressed as transform key)
    // wwa -> wa (w as consonant + a)

    #[test]
    fn testWAsConsonantAfterEscapeBasic() {
        // ww -> w, then vowel
        assert_eq!(apply_input("wwa"), "wa");
        assert_eq!(apply_input("wwe"), "we");
        assert_eq!(apply_input("wwi"), "wi");
        assert_eq!(apply_input("wwo"), "wo");
        assert_eq!(apply_input("wwu"), "wu");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneSac() {
        assert_eq!(apply_input("wwas"), "wá");
        assert_eq!(apply_input("wwes"), "wé");
        assert_eq!(apply_input("wwis"), "wí");
        assert_eq!(apply_input("wwos"), "wó");
        assert_eq!(apply_input("wwus"), "wú");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneHuyen() {
        assert_eq!(apply_input("wwaf"), "wà");
        assert_eq!(apply_input("wwef"), "wè");
        assert_eq!(apply_input("wwif"), "wì");
        assert_eq!(apply_input("wwof"), "wò");
        assert_eq!(apply_input("wwuf"), "wù");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneHoi() {
        assert_eq!(apply_input("wwar"), "wả");
        assert_eq!(apply_input("wwer"), "wẻ");
        assert_eq!(apply_input("wwir"), "wỉ");
        assert_eq!(apply_input("wwor"), "wỏ");
        assert_eq!(apply_input("wwur"), "wủ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneNga() {
        assert_eq!(apply_input("wwax"), "wã");
        assert_eq!(apply_input("wwex"), "wẽ");
        assert_eq!(apply_input("wwix"), "wĩ");
        assert_eq!(apply_input("wwox"), "wõ");
        assert_eq!(apply_input("wwux"), "wũ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithToneNang() {
        assert_eq!(apply_input("wwaj"), "wạ");
        assert_eq!(apply_input("wwej"), "wẹ");
        assert_eq!(apply_input("wwij"), "wị");
        assert_eq!(apply_input("wwoj"), "wọ");
        assert_eq!(apply_input("wwuj"), "wụ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithCircumflexVowels() {
        assert_eq!(apply_input("wwaas"), "wấ");
        assert_eq!(apply_input("wwees"), "wế");
        assert_eq!(apply_input("wwoos"), "wố");
        assert_eq!(apply_input("wweef"), "wề");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithHornVowels() {
        // After ww->w escape, w acts as consonant
        // Then aw -> ă, ow -> ơ, uw -> ư still work
        assert_eq!(apply_input("wwaws"), "wắ");
        assert_eq!(apply_input("wwows"), "wớ");
        assert_eq!(apply_input("wwuws"), "wứ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeUppercase() {
        assert_eq!(apply_input("WWA"), "WA");
        assert_eq!(apply_input("WWAS"), "WÁ");
        assert_eq!(apply_input("Wwas"), "Wá");
        assert_eq!(apply_input("WWEEF"), "WỀ");
    }

    #[test]
    fn testWAsConsonantAfterEscapeWithFinalConsonant() {
        assert_eq!(apply_input("wwams"), "wám");
        assert_eq!(apply_input("wwangs"), "wáng");
        assert_eq!(apply_input("wwanhf"), "wành");
    }

    #[test]
    fn testWAsConsonantAfterEscapeRealWords() {
        // Real-world use case: typing English/foreign words
        // assert_eq!(apply_input("wwifi"), "wifi");
        assert_eq!(apply_input("wweb"), "web");
        // assert_eq!(apply_input("wwindow"), "window");
    }

    #[test]
    fn testWAsConsonantVsStandaloneW() {
        // w without escape becomes ư
        assert_eq!(apply_input("wa"), "ưa"); // w -> ư, then a
        // ww escapes to w, then a is just vowel
        assert_eq!(apply_input("wwa"), "wa"); // ww -> w, then a
    }

    #[test]
    fn testWAsConsonantThenWTransform() {
        // After ww->w escape, can still use w to transform vowels
        assert_eq!(apply_input("wwuw"), "wư"); // ww->w, then uw->ư
        assert_eq!(apply_input("wwaw"), "wă"); // ww->w, then aw->ă
        assert_eq!(apply_input("wwow"), "wơ"); // ww->w, then ow->ơ
    }

    // MARK: - Mixed Foreign Consonants

    #[test]
    fn testZWithWTransform() {
        // z consonant + w transform on vowel
        assert_eq!(apply_input("zwa"), "zưa"); // z + ư + a
        assert_eq!(apply_input("zuw"), "zư"); // z + ư
        assert_eq!(apply_input("zaw"), "ză"); // z + ă
        assert_eq!(apply_input("zow"), "zơ"); // z + ơ
    }

    #[test]
    fn testJWithWTransform() {
        assert_eq!(apply_input("jwa"), "jưa");
        assert_eq!(apply_input("juw"), "jư");
        assert_eq!(apply_input("jaw"), "jă");
        assert_eq!(apply_input("jow"), "jơ");
    }

    #[test]
    fn testFWithWTransform() {
        assert_eq!(apply_input("fwa"), "fưa");
        assert_eq!(apply_input("fuw"), "fư");
        assert_eq!(apply_input("faw"), "fă");
        assert_eq!(apply_input("fow"), "fơ");
    }

    // MARK: - Tone Key Position Distinction
    // These tests verify that z/j/f act as consonants BEFORE vowels
    // but as tone keys AFTER vowels

    #[test]
    fn testToneKeyPositionZ() {
        // z before vowel = consonant
        assert_eq!(apply_input("za"), "za");
        assert_eq!(apply_input("zas"), "zá");

        // z after toned vowel = removes tone
        assert_eq!(apply_input("asz"), "a");
        assert_eq!(apply_input("àz"), "a"); // direct input of à, then z removes tone
    }

    #[test]
    fn testToneKeyPositionJ() {
        // j before vowel = consonant
        assert_eq!(apply_input("ja"), "ja");
        assert_eq!(apply_input("jas"), "já");

        // j after vowel = applies nặng tone
        assert_eq!(apply_input("aj"), "ạ");
        assert_eq!(apply_input("taj"), "tạ");
    }

    #[test]
    fn testToneKeyPositionF() {
        // f before vowel = consonant
        assert_eq!(apply_input("fa"), "fa");
        assert_eq!(apply_input("fas"), "fá");

        // f after vowel = applies huyền tone
        assert_eq!(apply_input("af"), "à");
        assert_eq!(apply_input("taf"), "tà");
    }

    // MARK: - Multiple Tone Keys in Sequence
    // When foreign consonant is followed by vowel, then tone key

    #[test]
    fn testZFollowedByVowelThenToneReplacement() {
        // z + a + s = zá, then f replaces tone
        assert_eq!(apply_input("zasf"), "zà");
        assert_eq!(apply_input("zasr"), "zả");
        assert_eq!(apply_input("zasx"), "zã");
        assert_eq!(apply_input("zasj"), "zạ");
    }

    #[test]
    fn testJFollowedByVowelThenToneReplacement() {
        assert_eq!(apply_input("jasf"), "jà");
        assert_eq!(apply_input("jasr"), "jả");
        assert_eq!(apply_input("jasx"), "jã");
        assert_eq!(apply_input("jasj"), "jạ");
    }

    #[test]
    fn testFFollowedByVowelThenToneReplacement() {
        assert_eq!(apply_input("fasf"), "fà");
        assert_eq!(apply_input("fasr"), "fả");
        assert_eq!(apply_input("fasx"), "fã");
        assert_eq!(apply_input("fasj"), "fạ");
    }
}

// MARK: - Repeated Escape Sequence Tests
mod repeated_escape_tests {
    use super::apply_input;

    // MARK: - Repeated Tone Escape Tests (s, f, r, x, j)
    // After escaping a tone (e.g., áss → as), subsequent same keys should just append
    // Bug: Currently cycles between toned and escaped state

    #[test]
    fn testRepeatedToneEscapeSac() {
        // chans → chán, chanss → chans, chansss → chanss, etc.
        assert_eq!(apply_input("chans"), "chán");
        assert_eq!(apply_input("chanss"), "chans");
        assert_eq!(apply_input("chansss"), "chanss");
        assert_eq!(apply_input("chanssss"), "chansss");
        assert_eq!(apply_input("chansssss"), "chanssss");
    }

    #[test]
    fn testRepeatedToneEscapeHuyen() {
        assert_eq!(apply_input("taf"), "tà");
        assert_eq!(apply_input("taff"), "taf");
        assert_eq!(apply_input("tafff"), "taff");
        assert_eq!(apply_input("taffff"), "tafff");
        assert_eq!(apply_input("tafffff"), "taffff");
    }

    #[test]
    fn testRepeatedToneEscapeHoi() {
        assert_eq!(apply_input("tar"), "tả");
        assert_eq!(apply_input("tarr"), "tar");
        assert_eq!(apply_input("tarrr"), "tarr");
        assert_eq!(apply_input("tarrrr"), "tarrr");
        assert_eq!(apply_input("tarrrrr"), "tarrrr");
    }

    #[test]
    fn testRepeatedToneEscapeNga() {
        assert_eq!(apply_input("tax"), "tã");
        assert_eq!(apply_input("taxx"), "tax");
        assert_eq!(apply_input("taxxx"), "taxx");
        assert_eq!(apply_input("taxxxx"), "taxxx");
        assert_eq!(apply_input("taxxxxx"), "taxxxx");
    }

    #[test]
    fn testRepeatedToneEscapeNang() {
        assert_eq!(apply_input("taj"), "tạ");
        assert_eq!(apply_input("tajj"), "taj");
        assert_eq!(apply_input("tajjj"), "tajj");
        assert_eq!(apply_input("tajjjj"), "tajjj");
        assert_eq!(apply_input("tajjjjj"), "tajjjj");
    }

    // MARK: - Repeated Tone Escape with Different Vowels

    #[test]
    fn testRepeatedToneEscapeVowelE() {
        assert_eq!(apply_input("tes"), "té");
        assert_eq!(apply_input("tess"), "tes");
        assert_eq!(apply_input("tesss"), "tess");
        assert_eq!(apply_input("tessss"), "tesss");
    }

    #[test]
    fn testRepeatedToneEscapeVowelI() {
        assert_eq!(apply_input("tis"), "tí");
        assert_eq!(apply_input("tiss"), "tis");
        assert_eq!(apply_input("tisss"), "tiss");
        assert_eq!(apply_input("tissss"), "tisss");
    }

    #[test]
    fn testRepeatedToneEscapeVowelO() {
        assert_eq!(apply_input("tos"), "tó");
        assert_eq!(apply_input("toss"), "tos");
        assert_eq!(apply_input("tosss"), "toss");
        assert_eq!(apply_input("tossss"), "tosss");
    }

    #[test]
    fn testRepeatedToneEscapeVowelU() {
        assert_eq!(apply_input("tus"), "tú");
        assert_eq!(apply_input("tuss"), "tus");
        assert_eq!(apply_input("tusss"), "tuss");
        assert_eq!(apply_input("tussss"), "tusss");
    }

    #[test]
    fn testRepeatedToneEscapeVowelY() {
        assert_eq!(apply_input("tys"), "tý");
        assert_eq!(apply_input("tyss"), "tys");
        assert_eq!(apply_input("tysss"), "tyss");
        assert_eq!(apply_input("tyssss"), "tysss");
    }

    // MARK: - Repeated Tone Escape with Transformed Vowels (â, ê, ô)

    #[test]
    fn testRepeatedToneEscapeCircumflexA() {
        // âs → ấ, âss → âs, âsss → âss, etc.
        assert_eq!(apply_input("aas"), "ấ");
        assert_eq!(apply_input("aass"), "âs");
        assert_eq!(apply_input("aasss"), "âss");
        assert_eq!(apply_input("aassss"), "âsss");
        assert_eq!(apply_input("aasssss"), "âssss");
    }

    #[test]
    fn testRepeatedToneEscapeCircumflexE() {
        assert_eq!(apply_input("ees"), "ế");
        assert_eq!(apply_input("eess"), "ês");
        assert_eq!(apply_input("eesss"), "êss");
        assert_eq!(apply_input("eessss"), "êsss");
    }

    #[test]
    fn testRepeatedToneEscapeCircumflexO() {
        assert_eq!(apply_input("oos"), "ố");
        assert_eq!(apply_input("ooss"), "ôs");
        assert_eq!(apply_input("oosss"), "ôss");
        assert_eq!(apply_input("oossss"), "ôsss");
    }

    // MARK: - Repeated Tone Escape with Breve/Horn Vowels (ă, ơ, ư)

    #[test]
    fn testRepeatedToneEscapeBreveA() {
        // ăs → ắ, ăss → ăs, ăsss → ăss, etc.
        assert_eq!(apply_input("aws"), "ắ");
        assert_eq!(apply_input("awss"), "ăs");
        assert_eq!(apply_input("awsss"), "ăss");
        assert_eq!(apply_input("awssss"), "ăsss");
    }

    #[test]
    fn testRepeatedToneEscapeHornO() {
        assert_eq!(apply_input("ows"), "ớ");
        assert_eq!(apply_input("owss"), "ơs");
        assert_eq!(apply_input("owsss"), "ơss");
        assert_eq!(apply_input("owssss"), "ơsss");
    }

    #[test]
    fn testRepeatedToneEscapeHornU() {
        assert_eq!(apply_input("uws"), "ứ");
        assert_eq!(apply_input("uwss"), "ưs");
        assert_eq!(apply_input("uwsss"), "ưss");
        assert_eq!(apply_input("uwssss"), "ưsss");
    }

    // MARK: - Repeated Vowel Transform Escape Tests (aa, ee, oo)

    #[test]
    fn testRepeatedVowelEscapeAA() {
        // aa → â, aaa → aa, aaaa → aaa, etc.
        assert_eq!(apply_input("aa"), "â");
        assert_eq!(apply_input("aaa"), "aa");
        assert_eq!(apply_input("aaaa"), "aaa");
        assert_eq!(apply_input("aaaaa"), "aaaa");
        assert_eq!(apply_input("aaaaaa"), "aaaaa");
    }

    #[test]
    fn testRepeatedVowelEscapeEE() {
        assert_eq!(apply_input("ee"), "ê");
        assert_eq!(apply_input("eee"), "ee");
        assert_eq!(apply_input("eeee"), "eee");
        assert_eq!(apply_input("eeeee"), "eeee");
    }

    #[test]
    fn testRepeatedVowelEscapeOO() {
        assert_eq!(apply_input("oo"), "ô");
        assert_eq!(apply_input("ooo"), "oo");
        assert_eq!(apply_input("oooo"), "ooo");
        assert_eq!(apply_input("ooooo"), "oooo");
    }

    // MARK: - Repeated W Transform Escape Tests (aw, ow, uw)

    #[test]
    fn testRepeatedVowelEscapeAW() {
        // aw → ă, aww → aw, awww → aww, etc.
        assert_eq!(apply_input("aw"), "ă");
        assert_eq!(apply_input("aww"), "aw");
        assert_eq!(apply_input("awww"), "aww");
        assert_eq!(apply_input("awwww"), "awww");
        assert_eq!(apply_input("awwwww"), "awwww");
    }

    #[test]
    fn testRepeatedVowelEscapeOW() {
        assert_eq!(apply_input("ow"), "ơ");
        assert_eq!(apply_input("oww"), "ow");
        assert_eq!(apply_input("owww"), "oww");
        assert_eq!(apply_input("owwww"), "owww");
    }

    #[test]
    fn testRepeatedVowelEscapeUW() {
        assert_eq!(apply_input("uw"), "ư");
        assert_eq!(apply_input("uww"), "uw");
        assert_eq!(apply_input("uwww"), "uww");
        assert_eq!(apply_input("uwwww"), "uwww");
    }

    // MARK: - Repeated Consonant Escape Tests (dd)

    #[test]
    fn testRepeatedConsonantEscapeDD() {
        // dd → đ, ddd → dd, dddd → ddd, etc.
        assert_eq!(apply_input("dd"), "đ");
        assert_eq!(apply_input("ddd"), "dd");
        assert_eq!(apply_input("dddd"), "ddd");
        assert_eq!(apply_input("ddddd"), "dddd");
        assert_eq!(apply_input("dddddd"), "ddddd");
    }

    #[test]
    fn testRepeatedConsonantEscapeDDUppercase() {
        assert_eq!(apply_input("DD"), "Đ");
        assert_eq!(apply_input("DDD"), "DD");
        assert_eq!(apply_input("DDDD"), "DDD");
        assert_eq!(apply_input("DDDDD"), "DDDD");
    }

    // MARK: - Repeated Standalone W Escape Tests

    #[test]
    fn testRepeatedStandaloneWEscape() {
        // w → ư, ww → w, www → ww, etc.
        assert_eq!(apply_input("w"), "ư");
        assert_eq!(apply_input("ww"), "w");
        assert_eq!(apply_input("www"), "ww");
        assert_eq!(apply_input("wwww"), "www");
        assert_eq!(apply_input("wwwww"), "wwww");
    }

    #[test]
    fn testRepeatedStandaloneWEscapeAfterConsonant() {
        // tw → tư, tww → tw, twww → tww, etc.
        assert_eq!(apply_input("tw"), "tư");
        assert_eq!(apply_input("tww"), "tw");
        assert_eq!(apply_input("twww"), "tww");
        assert_eq!(apply_input("twwww"), "twww");
    }

    // MARK: - Repeated Escape in Real Word Context

    #[test]
    fn testRepeatedEscapeInWordChan() {
        // Testing in realistic typing scenario
        assert_eq!(apply_input("chanss"), "chans");
        assert_eq!(apply_input("chansss"), "chanss");
        assert_eq!(apply_input("chanssss"), "chansss");
    }

    #[test]
    fn testRepeatedEscapeInWordViet() {
        // vieejt → việt, vieejtj → việtj? Let's test tone escape
        assert_eq!(apply_input("vieej"), "việ");
        assert_eq!(apply_input("vieejj"), "viêj");
        assert_eq!(apply_input("vieejjj"), "viêjj");
    }

    #[test]
    fn testRepeatedEscapeInWordDi() {
        // Testing đ escape in word context
        assert_eq!(apply_input("ddi"), "đi");
        assert_eq!(apply_input("dddi"), "ddi");
        assert_eq!(apply_input("ddddi"), "dddi");
    }

    // MARK: - Mixed Repeated Escape (Different Keys After Escape)

    #[test]
    fn testMixedKeysAfterToneEscape() {
        // After escaping tone, typing a different key should work normally
        assert_eq!(apply_input("tasst"), "tast"); // After escape, t just appends
        assert_eq!(apply_input("tassn"), "tasn"); // After escape, n just appends
    }

    #[test]
    fn testMixedKeysAfterVowelEscape() {
        // After escaping vowel transform, typing different key works normally
        assert_eq!(apply_input("aaat"), "aat"); // After escape, t just appends
        assert_eq!(apply_input("aaan"), "aan"); // After escape, n just appends
    }

    #[test]
    fn testMixedKeysAfterConsonantEscape() {
        // After escaping dd, typing different key works normally
        assert_eq!(apply_input("ddda"), "dda"); // After escape, a just appends
        assert_eq!(apply_input("ddde"), "dde"); // After escape, e just appends
    }

    // MARK: - Edge Cases: Re-transformation After Word Boundary

    #[test]
    fn testReTransformAfterWordBoundary() {
        // After word boundary (space), transforms should work again
        assert_eq!(apply_input("tass tas"), "tas tá"); // New word gets transformed
        assert_eq!(apply_input("aaa aa"), "aa â"); // New word gets transformed
        assert_eq!(apply_input("ddd dd"), "dd đ"); // New word gets transformed
    }

    // MARK: - Case Preservation in Repeated Escapes

    #[test]
    fn testRepeatedEscapeCasePreservationTone() {
        assert_eq!(apply_input("TAS"), "TÁ");
        assert_eq!(apply_input("TASS"), "TAS");
        assert_eq!(apply_input("TASSS"), "TASS");
    }

    #[test]
    fn testRepeatedEscapeCasePreservationVowel() {
        assert_eq!(apply_input("AA"), "Â");
        assert_eq!(apply_input("AAA"), "AA");
        assert_eq!(apply_input("AAAA"), "AAA");
    }
}
