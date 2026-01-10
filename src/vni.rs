use crate::common::{KeyTransformAction, WTransformKind};
use crate::diacritics::{apply_shape_preserving_tone, escape_shape_preserving_tone, VowelShape};
use crate::VitypeEngine;

// ==================== VNI Helper Functions ====================

fn is_vni_tone_key(ch: char) -> bool {
    matches!(ch, '0'..='5')
}

fn is_vni_vowel_transform_key(ch: char) -> bool {
    matches!(ch, '6' | '7' | '8')
}

pub(super) fn is_vni_word_boundary(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch.is_ascii_punctuation()
}

fn vni_escape_char_for_key(ch: char, key: char) -> Option<char> {
    let kind = match key {
        '6' => VowelShape::Circumflex,
        '7' => VowelShape::Horn,
        '8' => VowelShape::Breve,
        _ => return None,
    };
    escape_shape_preserving_tone(ch, kind)
}

fn vni_apply_shape_for_key(ch: char, key: char) -> Option<char> {
    let kind = match key {
        '6' => VowelShape::Circumflex,
        '7' => VowelShape::Horn,
        '8' => VowelShape::Breve,
        _ => return None,
    };
    apply_shape_preserving_tone(ch, kind)
}

fn vni_tone_key_to_internal(ch: char) -> Option<char> {
    Some(match ch {
        '1' => 's', // sắc
        '2' => 'f', // huyền
        '3' => 'r', // hỏi
        '4' => 'x', // ngã
        '5' => 'j', // nặng
        '0' => 'z', // remove tone
        _ => return None,
    })
}

// ==================== VNI Methods on VitypeEngine ====================

impl VitypeEngine {
    pub(super) fn try_vni_escape_sequence(&mut self, ch: char) -> Option<KeyTransformAction> {
        let last_key = self.last_transform_key?;
        if ch != last_key {
            return None;
        }

        // Handle '7' escape for compound transforms (similar to 'w' in Telex)
        if ch == '7' {
            if let Some(action) = self.try_escape_compound_horn_key(ch, ch) {
                return Some(action);
            }
        }

        // Handle '9' escape (đ → d9)
        if ch == '9' {
            if let Some(&last_char) = self.buffer.last() {
                if last_char == 'đ' || last_char == 'Đ' {
                    let d_char = if last_char == 'Đ' { 'D' } else { 'd' };
                    self.buffer.pop();
                    self.buffer.push(d_char);
                    self.buffer.push('9');
                    self.clear_last_transform_and_suppress('9');
                    return Some(KeyTransformAction {
                        delete_count: 1,
                        text: format!("{}9", d_char),
                    });
                }
            }
        }

        // Handle '6', '7', '8' escapes (vowel transforms)
        if ch == '6' || ch == '7' || ch == '8' {
            // Check if last char in buffer is a transformed vowel
            if let Some(&last_char) = self.buffer.last() {
                if let Some(original) = vni_escape_char_for_key(last_char, ch) {
                    self.buffer.pop();
                    self.buffer.push(original);
                    self.buffer.push(ch);
                    self.clear_last_transform_and_suppress(ch);
                    return Some(KeyTransformAction {
                        delete_count: 1,
                        text: format!("{}{}", original, ch),
                    });
                }
            }

            // Check for non-adjacent transformed vowel (free transform escape)
            if let Some((index, original)) =
                self.find_last_vni_untransformable_vowel(ch, self.buffer.len())
            {
                let delete_count = self.buffer.len() - index;
                self.buffer[index] = original;
                self.buffer.push(ch);
                self.clear_last_transform_and_suppress(ch);
                let output_text = self.buffer_string_from(index);
                return Some(KeyTransformAction {
                    delete_count,
                    text: output_text,
                });
            }
        }

        // Handle tone escapes ('1'-'5', '0')
        if is_vni_tone_key(ch) {
            if let Some(internal_tone_key) = vni_tone_key_to_internal(ch) {
                if let Some(action) = self.try_escape_repeated_tone_key(ch, internal_tone_key, ch) {
                    return Some(action);
                }
            }
        }

        None
    }

    pub(super) fn try_vni_consonant_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        if ch != '9' {
            return None;
        }
        self.try_d_stroke('9')
    }

    pub(super) fn try_vni_vowel_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        if !is_vni_vowel_transform_key(ch) {
            return None;
        }

        if self.buffer.is_empty() {
            return None;
        }

        // Handle compound transforms for '7' key (similar to 'w' in Telex)
        if ch == '7' {
            if let Some(action) = self.try_compound_ua_escape('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uo_final_consonant_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uoi_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uu_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uou_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_ou_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uow_transform('7') {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uaw_transform('7') {
                return Some(action);
            }
        }

        // Find transformable vowel and apply transform
        let trigger_index = self.buffer.len() - 1;

        // Search backward for a transformable vowel (free transform, up to 4 chars)
        let vowel_index = self.find_last_vni_transformable_vowel(ch, trigger_index, 4)?;
        let vowel = self.buffer[vowel_index];

        let result = vni_apply_shape_for_key(vowel, ch)?;

        let delete_count = trigger_index - vowel_index;
        self.buffer[vowel_index] = result;
        self.buffer.pop(); // Remove the transform key
        self.last_transform_key = Some(ch);
        self.last_w_transform_kind = WTransformKind::None;

        if self.auto_fix_tone {
            if let Some(action) = self.reposition_tone_if_needed(false, Some(vowel_index)) {
                return Some(action);
            }
        }

        let output_text = self.buffer_string_from(vowel_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    pub(super) fn try_vni_tone_mark(&mut self, ch: char) -> Option<KeyTransformAction> {
        if !is_vni_tone_key(ch) {
            return None;
        }

        // Map VNI number key to internal (Telex) tone key
        let internal_tone_key = vni_tone_key_to_internal(ch)?;

        self.apply_tone_mark_internal(internal_tone_key, ch)
    }

    // ==================== VNI Helper Methods ====================

    fn find_last_vni_transformable_vowel(
        &self,
        key: char,
        before: usize,
        max_distance: usize,
    ) -> Option<usize> {
        let allow_adjacent_skip = |_base_lower: char| true;
        self.find_last_vowel_index_with_predicate(
            before,
            max_distance,
            allow_adjacent_skip,
            |ch, _| vni_apply_shape_for_key(ch, key).is_some(),
        )
    }

    fn find_last_vni_untransformable_vowel(
        &self,
        key: char,
        before: usize,
    ) -> Option<(usize, char)> {
        let mut index = before;
        while index > 0 {
            index -= 1;
            if let Some(original) = vni_escape_char_for_key(self.buffer[index], key) {
                return Some((index, original));
            }
        }
        None
    }
}
