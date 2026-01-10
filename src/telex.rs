use crate::common::{lower_char, KeyTransformAction, WTransformKind};
use crate::diacritics::{
    apply_shape_preserving_tone, apply_telex_w_preserving_tone, escape_shape_preserving_tone,
    split_vowel_and_tone, VowelShape,
};
use crate::VitypeEngine;

// ==================== Telex Helper Functions ====================

fn is_tone_key(ch: char) -> bool {
    matches!(lower_char(ch), 's' | 'f' | 'r' | 'x' | 'j' | 'z')
}

pub(super) fn is_telex_word_boundary(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch.is_ascii_punctuation() || ch.is_ascii_digit()
}

fn telex_escape_char_for_key(ch: char, key_lower: char) -> Option<char> {
    match key_lower {
        'w' => escape_shape_preserving_tone(ch, VowelShape::Breve)
            .or_else(|| escape_shape_preserving_tone(ch, VowelShape::Horn)),
        'a' | 'e' | 'o' => {
            let (base, _) = split_vowel_and_tone(ch);
            let base_lower = lower_char(base);
            let expected = match key_lower {
                'a' => 'â',
                'e' => 'ê',
                'o' => 'ô',
                _ => return None,
            };
            if base_lower != expected {
                return None;
            }
            escape_shape_preserving_tone(ch, VowelShape::Circumflex)
        }
        _ => None,
    }
}

// ==================== Telex Methods on VitypeEngine ====================

impl VitypeEngine {
    pub(super) fn try_telex_escape_sequence(&mut self, ch: char) -> Option<KeyTransformAction> {
        let last_key = self.last_transform_key?;
        let ch_lower = lower_char(ch);
        let last_key_lower = lower_char(last_key);
        if ch_lower != last_key_lower {
            return None;
        }

        if ch_lower == 'w' {
            match self.last_w_transform_kind {
                WTransformKind::Standalone => {
                    if let Some(&last_char) = self.buffer.last() {
                        if last_char == 'ư' || last_char == 'Ư' {
                            let replacement = if last_char.is_uppercase() { 'W' } else { 'w' };
                            self.buffer.pop();
                            self.buffer.push(replacement);
                            self.clear_last_transform_and_suppress(ch_lower);
                            return Some(KeyTransformAction {
                                delete_count: 1,
                                text: replacement.to_string(),
                            });
                        }
                    }
                }
                WTransformKind::CompoundUow => {
                    if let Some(action) = self.try_escape_compound_horn_key(ch, ch_lower) {
                        return Some(action);
                    }
                }
                WTransformKind::CompoundUoiw => {
                    if let Some(action) = self.try_escape_compound_horn_key(ch, ch_lower) {
                        return Some(action);
                    }
                }
                WTransformKind::CompoundUoFinalConsonantW => {
                    if let Some(action) = self.try_escape_compound_horn_key(ch, ch_lower) {
                        return Some(action);
                    }
                }
                WTransformKind::CompoundUaw => {
                    if let Some(action) = self.try_escape_compound_horn_key(ch, ch_lower) {
                        return Some(action);
                    }
                }
                WTransformKind::None => {
                    if let Some((index, original)) =
                        self.find_last_untransformable_vowel(ch_lower, self.buffer.len())
                    {
                        let delete_count = self.buffer.len() - index;
                        self.buffer[index] = original;
                        self.buffer.push(ch);
                        self.clear_last_transform_and_suppress(ch_lower);
                        let output_text = self.buffer_string_from(index);
                        return Some(KeyTransformAction {
                            delete_count,
                            text: output_text,
                        });
                    }
                }
            }
        }

        if ch_lower == 'd' {
            if let Some(&last_char) = self.buffer.last() {
                if last_char == 'đ' || last_char == 'Đ' {
                    let is_upper = last_char == 'Đ';
                    let replacement = if is_upper {
                        if ch.is_uppercase() {
                            "DD".to_string()
                        } else {
                            "Dd".to_string()
                        }
                    } else if ch.is_uppercase() {
                        "dD".to_string()
                    } else {
                        "dd".to_string()
                    };
                    self.buffer.pop();
                    self.buffer.extend(replacement.chars());
                    self.clear_last_transform_and_suppress(ch_lower);
                    return Some(KeyTransformAction {
                        delete_count: 1,
                        text: replacement,
                    });
                }
            }
        }

        if let Some(&last_char) = self.buffer.last() {
            if let Some(original) = telex_escape_char_for_key(last_char, ch_lower) {
                self.buffer.pop();
                self.buffer.push(original);
                self.buffer.push(ch);
                self.clear_last_transform_and_suppress(ch_lower);
                return Some(KeyTransformAction {
                    delete_count: 1,
                    text: format!("{}{}", original, ch),
                });
            }
        }

        if ch_lower == 'a' || ch_lower == 'e' || ch_lower == 'o' {
            if let Some((index, original)) =
                self.find_last_untransformable_vowel(ch_lower, self.buffer.len())
            {
                let delete_count = self.buffer.len() - index;
                self.buffer[index] = original;
                self.buffer.push(ch);
                self.clear_last_transform_and_suppress(ch_lower);
                let output_text = self.buffer_string_from(index);
                return Some(KeyTransformAction {
                    delete_count,
                    text: output_text,
                });
            }
        }

        if is_tone_key(ch) {
            if let Some(action) = self.try_escape_repeated_tone_key(ch, ch_lower, ch_lower) {
                return Some(action);
            }
        }

        None
    }

    pub(super) fn try_telex_consonant_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        let ch_lower = lower_char(ch);
        if ch_lower != 'd' {
            return None;
        }
        self.try_d_stroke('d')
    }

    pub(super) fn try_telex_vowel_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        let ch_lower = lower_char(ch);

        if ch_lower == 'w' {
            if self.buffer.is_empty() {
                return None;
            }

            if let Some(action) = self.try_compound_ua_escape('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uo_final_consonant_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uoi_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uu_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uou_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_ou_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uow_transform('w') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uaw_transform('w') {
                return Some(action);
            }

            if let Some((index, result)) =
                self.find_last_transformable_vowel(self.buffer.len() - 1)
            {
                self.buffer[index] = result;
                self.buffer.pop();
                self.last_transform_key = Some('w');
                self.last_w_transform_kind = WTransformKind::None;
                let delete_count = self.buffer.len() - index;
                let output_text = self.buffer_string_from(index);
                return Some(KeyTransformAction {
                    delete_count,
                    text: output_text,
                });
            }

            let replacement = if ch.is_uppercase() { 'Ư' } else { 'ư' };
            self.buffer.pop();
            self.buffer.push(replacement);
            self.last_transform_key = Some('w');
            self.last_w_transform_kind = WTransformKind::Standalone;
            return Some(KeyTransformAction {
                delete_count: 0,
                text: replacement.to_string(),
            });
        }

        if self.buffer.len() < 2 {
            return None;
        }

        if ch_lower == 'a' || ch_lower == 'e' || ch_lower == 'o' {
            if let Some(vowel_index) =
                self.find_last_matching_vowel_index(ch, self.buffer.len() - 1, 4)
            {
                let vowel = self.buffer[vowel_index];
                let result = apply_shape_preserving_tone(vowel, VowelShape::Circumflex)?;

                let vowel_offset = vowel_index;
                let trigger_index = self.buffer.len() - 1;
                let delete_count = trigger_index - vowel_index;

                self.buffer[vowel_index] = result;
                self.buffer.pop();
                self.last_transform_key = Some(ch);
                self.last_w_transform_kind = WTransformKind::None;

                if self.auto_fix_tone {
                    if let Some(action) =
                        self.reposition_tone_if_needed(false, Some(vowel_offset))
                    {
                        return Some(action);
                    }
                }

                let output_text = self.buffer_string_from(vowel_offset);
                return Some(KeyTransformAction {
                    delete_count,
                    text: output_text,
                });
            }
        }

        None
    }

    pub(super) fn try_telex_tone_mark(&mut self, ch: char) -> Option<KeyTransformAction> {
        let ch_lower = lower_char(ch);
        if !is_tone_key(ch_lower) {
            return None;
        }

        self.apply_tone_mark_internal(ch_lower, ch)
    }

    // ==================== Telex Helper Methods ====================

    fn find_last_untransformable_vowel(
        &self,
        key_lower: char,
        before: usize,
    ) -> Option<(usize, char)> {
        let mut index = before;
        while index > 0 {
            index -= 1;
            if let Some(original) = telex_escape_char_for_key(self.buffer[index], key_lower) {
                return Some((index, original));
            }
        }
        None
    }

    fn find_last_transformable_vowel(&self, before: usize) -> Option<(usize, char)> {
        let mut index = before;
        while index > 0 {
            index -= 1;
            let ch = self.buffer[index];
            if let Some(result) = apply_telex_w_preserving_tone(ch) {
                return Some((index, result));
            }
        }
        None
    }
}
