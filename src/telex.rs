use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::common::{
    is_vowel, lower_char, KeyTransformAction, WTransformKind,
    TONED_TO_BASE, VOWEL_TO_TONED,
};
use crate::VitypeEngine;

// ==================== Telex Helper Functions ====================

fn is_tone_key(ch: char) -> bool {
    matches!(lower_char(ch), 's' | 'f' | 'r' | 'x' | 'j' | 'z')
}

pub(super) fn is_telex_word_boundary(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch.is_ascii_punctuation() || ch.is_ascii_digit()
}

// ==================== Telex Static Data ====================

static VOWEL_TRANSFORMS: Lazy<HashMap<char, Vec<(char, char)>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('a', vec![('a', 'â'), ('A', 'Â'), ('ă', 'â'), ('Ă', 'Â')]);
    map.insert('A', vec![('a', 'Â'), ('A', 'Â'), ('ă', 'Â'), ('Ă', 'Â')]);
    map.insert('e', vec![('e', 'ê'), ('E', 'Ê')]);
    map.insert('E', vec![('e', 'Ê'), ('E', 'Ê')]);
    map.insert('o', vec![('o', 'ô'), ('O', 'Ô'), ('ơ', 'ô'), ('Ơ', 'Ô')]);
    map.insert('O', vec![('o', 'Ô'), ('O', 'Ô'), ('ơ', 'Ô'), ('Ơ', 'Ô')]);
    map.insert(
        'w',
        vec![
            ('a', 'ă'), ('A', 'Ă'),
            ('ă', 'ă'), ('Ă', 'Ă'),
            ('o', 'ơ'), ('O', 'Ơ'),
            ('ô', 'ơ'), ('Ô', 'Ơ'),
            ('u', 'ư'), ('U', 'Ư'),
            ('ư', 'ư'), ('Ư', 'Ư'),
            ('á', 'ắ'), ('à', 'ằ'), ('ả', 'ẳ'), ('ã', 'ẵ'), ('ạ', 'ặ'),
            ('Á', 'Ắ'), ('À', 'Ằ'), ('Ả', 'Ẳ'), ('Ã', 'Ẵ'), ('Ạ', 'Ặ'),
            ('ó', 'ớ'), ('ò', 'ờ'), ('ỏ', 'ở'), ('õ', 'ỡ'), ('ọ', 'ợ'),
            ('Ó', 'Ớ'), ('Ò', 'Ờ'), ('Ỏ', 'Ở'), ('Õ', 'Ỡ'), ('Ọ', 'Ợ'),
            ('ố', 'ớ'), ('ồ', 'ờ'), ('ổ', 'ở'), ('ỗ', 'ỡ'), ('ộ', 'ợ'),
            ('Ố', 'Ớ'), ('Ồ', 'Ờ'), ('Ổ', 'Ở'), ('Ỗ', 'Ỡ'), ('Ộ', 'Ợ'),
            ('ú', 'ứ'), ('ù', 'ừ'), ('ủ', 'ử'), ('ũ', 'ữ'), ('ụ', 'ự'),
            ('Ú', 'Ứ'), ('Ù', 'Ừ'), ('Ủ', 'Ử'), ('Ũ', 'Ữ'), ('Ụ', 'Ự'),
        ],
    );
    map.insert(
        'W',
        vec![
            ('a', 'Ă'), ('A', 'Ă'),
            ('ă', 'Ă'), ('Ă', 'Ă'),
            ('o', 'Ơ'), ('O', 'Ơ'),
            ('ô', 'Ơ'), ('Ô', 'Ơ'),
            ('u', 'Ư'), ('U', 'Ư'),
            ('ư', 'Ư'), ('Ư', 'Ư'),
            ('á', 'Ắ'), ('à', 'Ằ'), ('ả', 'Ẳ'), ('ã', 'Ẵ'), ('ạ', 'Ặ'),
            ('Á', 'Ắ'), ('À', 'Ằ'), ('Ả', 'Ẳ'), ('Ã', 'Ẵ'), ('Ạ', 'Ặ'),
            ('ó', 'Ớ'), ('ò', 'Ờ'), ('ỏ', 'Ở'), ('õ', 'Ỡ'), ('ọ', 'Ợ'),
            ('Ó', 'Ớ'), ('Ò', 'Ờ'), ('Ỏ', 'Ở'), ('Õ', 'Ỡ'), ('Ọ', 'Ợ'),
            ('ố', 'Ớ'), ('ồ', 'Ờ'), ('ổ', 'Ở'), ('ỗ', 'Ỡ'), ('ộ', 'Ợ'),
            ('Ố', 'Ớ'), ('Ồ', 'Ờ'), ('Ổ', 'Ở'), ('Ỗ', 'Ỡ'), ('Ộ', 'Ợ'),
            ('ú', 'Ứ'), ('ù', 'Ừ'), ('ủ', 'Ử'), ('ũ', 'Ữ'), ('ụ', 'Ự'),
            ('Ú', 'Ứ'), ('Ù', 'Ừ'), ('Ủ', 'Ử'), ('Ũ', 'Ữ'), ('Ụ', 'Ự'),
        ],
    );
    map
});

static VOWEL_UNTRANSFORMS: Lazy<HashMap<char, (char, char)>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('â', ('a', 'a')); map.insert('Â', ('a', 'A'));
    map.insert('ê', ('e', 'e')); map.insert('Ê', ('e', 'E'));
    map.insert('ô', ('o', 'o')); map.insert('Ô', ('o', 'O'));
    map.insert('ă', ('w', 'a')); map.insert('Ă', ('w', 'A'));
    map.insert('ơ', ('w', 'o')); map.insert('Ơ', ('w', 'O'));
    map.insert('ư', ('w', 'u')); map.insert('Ư', ('w', 'U'));
    map.insert('ắ', ('w', 'á')); map.insert('ằ', ('w', 'à')); map.insert('ẳ', ('w', 'ả')); map.insert('ẵ', ('w', 'ã')); map.insert('ặ', ('w', 'ạ'));
    map.insert('Ắ', ('w', 'Á')); map.insert('Ằ', ('w', 'À')); map.insert('Ẳ', ('w', 'Ả')); map.insert('Ẵ', ('w', 'Ã')); map.insert('Ặ', ('w', 'Ạ'));
    map.insert('ớ', ('w', 'ó')); map.insert('ờ', ('w', 'ò')); map.insert('ở', ('w', 'ỏ')); map.insert('ỡ', ('w', 'õ')); map.insert('ợ', ('w', 'ọ'));
    map.insert('Ớ', ('w', 'Ó')); map.insert('Ờ', ('w', 'Ò')); map.insert('Ở', ('w', 'Ỏ')); map.insert('Ỡ', ('w', 'Õ')); map.insert('Ợ', ('w', 'Ọ'));
    map.insert('ứ', ('w', 'ú')); map.insert('ừ', ('w', 'ù')); map.insert('ử', ('w', 'ủ')); map.insert('ữ', ('w', 'ũ')); map.insert('ự', ('w', 'ụ'));
    map.insert('Ứ', ('w', 'Ú')); map.insert('Ừ', ('w', 'Ù')); map.insert('Ử', ('w', 'Ủ')); map.insert('Ữ', ('w', 'Ũ')); map.insert('Ự', ('w', 'Ụ'));
    map.insert('ấ', ('a', 'á')); map.insert('ầ', ('a', 'à')); map.insert('ẩ', ('a', 'ả')); map.insert('ẫ', ('a', 'ã')); map.insert('ậ', ('a', 'ạ'));
    map.insert('Ấ', ('a', 'Á')); map.insert('Ầ', ('a', 'À')); map.insert('Ẩ', ('a', 'Ả')); map.insert('Ẫ', ('a', 'Ã')); map.insert('Ậ', ('a', 'Ạ'));
    map.insert('ế', ('e', 'é')); map.insert('ề', ('e', 'è')); map.insert('ể', ('e', 'ẻ')); map.insert('ễ', ('e', 'ẽ')); map.insert('ệ', ('e', 'ẹ'));
    map.insert('Ế', ('e', 'É')); map.insert('Ề', ('e', 'È')); map.insert('Ể', ('e', 'Ẻ')); map.insert('Ễ', ('e', 'Ẽ')); map.insert('Ệ', ('e', 'Ẹ'));
    map.insert('ố', ('o', 'ó')); map.insert('ồ', ('o', 'ò')); map.insert('ổ', ('o', 'ỏ')); map.insert('ỗ', ('o', 'õ')); map.insert('ộ', ('o', 'ọ'));
    map.insert('Ố', ('o', 'Ó')); map.insert('Ồ', ('o', 'Ò')); map.insert('Ổ', ('o', 'Ỏ')); map.insert('Ỗ', ('o', 'Õ')); map.insert('Ộ', ('o', 'Ọ'));
    map
});

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
            if let Some((key, original)) = VOWEL_UNTRANSFORMS.get(&last_char) {
                if lower_char(*key) == ch_lower {
                    self.buffer.pop();
                    self.buffer.push(*original);
                    self.buffer.push(ch);
                    self.clear_last_transform_and_suppress(ch_lower);
                    return Some(KeyTransformAction {
                        delete_count: 1,
                        text: format!("{}{}", original, ch),
                    });
                }
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

        if self.buffer.len() < 2 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let first_d_index =
            self.find_last_matching_d_index(trigger_index, 4)?;

        let first_d = self.buffer[first_d_index];
        let result = if first_d.is_uppercase() { 'Đ' } else { 'đ' };
        let delete_count = trigger_index - first_d_index;

        self.buffer[first_d_index] = result;
        self.buffer.pop();
        self.last_transform_key = Some('d');
        self.last_w_transform_kind = WTransformKind::None;

        let output_text = self.buffer_string_from(first_d_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    pub(super) fn try_telex_vowel_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        let ch_lower = lower_char(ch);

        if ch_lower == 'w' {
            if self.buffer.is_empty() {
                return None;
            }

            if let Some(action) = self.try_compound_uaw_escape() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uo_final_consonant_w_transform() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uuw_transform() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uou_w_transform() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_ouw_transform() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uow_transform() {
                return Some(action);
            }

            if let Some(action) = self.try_compound_uaw_transform() {
                return Some(action);
            }

            if let Some((index, transform)) =
                self.find_last_transformable_vowel(ch, self.buffer.len() - 1)
            {
                let result = transform.1;
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
            if let Some(transforms) = VOWEL_TRANSFORMS.get(&ch) {
                if let Some(vowel_index) = self.find_last_matching_vowel_index(ch, self.buffer.len() - 1, 4) {
                    let vowel = self.buffer[vowel_index];
                    let vowel_base = self.get_base_vowel(vowel);
                    if let Some(transform) = transforms
                        .iter()
                        .find(|(from, _)| *from == vowel_base || *from == vowel)
                    {
                        let mut result = transform.1;
                        if let Some((_, tone)) = TONED_TO_BASE.get(&vowel) {
                            if let Some(tone_map) = VOWEL_TO_TONED.get(&result) {
                                if let Some(toned_result) = tone_map.get(tone) {
                                    result = *toned_result;
                                }
                            }
                        }

                        let vowel_offset = vowel_index;
                        let trigger_index = self.buffer.len() - 1;
                        let delete_count = trigger_index - vowel_index;

                        self.buffer[vowel_index] = result;
                        self.buffer.pop();
                        self.last_transform_key = Some(ch);
                        self.last_w_transform_kind = WTransformKind::None;

                        if self.auto_fix_tone {
                            if let Some(action) = self.reposition_tone_if_needed(false, Some(vowel_offset)) {
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

    // ==================== Telex Compound Transforms ====================

    fn try_compound_uo_final_consonant_w_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 4 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        if trigger_index == 0 {
            return None;
        }
        let mut scan_index = trigger_index - 1;

        if is_vowel(self.buffer[scan_index]) {
            return None;
        }

        while scan_index > 0 && !is_vowel(self.buffer[scan_index]) {
            scan_index -= 1;
        }

        if !is_vowel(self.buffer[scan_index]) {
            return None;
        }

        let o_index = scan_index;
        let o = self.buffer[o_index];
        if o != 'o' && o != 'O' {
            return None;
        }
        if TONED_TO_BASE.contains_key(&o) {
            return None;
        }
        if o_index == 0 {
            return None;
        }

        let u_index = o_index - 1;
        let u = self.buffer[u_index];
        if u != 'u' && u != 'U' {
            return None;
        }
        if TONED_TO_BASE.contains_key(&u) {
            return None;
        }

        if u_index > 0 {
            let prev_char = self.buffer[u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let u_horn = if u.is_uppercase() { 'Ư' } else { 'ư' };
        let o_horn = if o.is_uppercase() { 'Ơ' } else { 'ơ' };

        self.buffer[u_index] = u_horn;
        self.buffer[o_index] = o_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::CompoundUoFinalConsonantW;

        let delete_count = self.buffer.len() - u_index;
        let output_text = self.buffer_string_from(u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_uow_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 3 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let o_index = trigger_index - 1;
        let u_index = o_index - 1;

        let u = self.buffer[u_index];
        let o = self.buffer[o_index];

        if (u != 'u' && u != 'U') || (o != 'o' && o != 'O') {
            return None;
        }

        if TONED_TO_BASE.contains_key(&u) || TONED_TO_BASE.contains_key(&o) {
            return None;
        }

        if u_index > 0 {
            let prev_char = self.buffer[u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let u_horn = if u.is_uppercase() { 'Ư' } else { 'ư' };
        let o_horn = if o.is_uppercase() { 'Ơ' } else { 'ơ' };

        self.buffer[u_index] = u_horn;
        self.buffer[o_index] = o_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::CompoundUow;

        let delete_count = self.buffer.len() - u_index;
        let output_text = self.buffer_string_from(u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_uaw_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 3 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let a_index = trigger_index - 1;
        let u_index = a_index - 1;

        let u = self.buffer[u_index];
        let a = self.buffer[a_index];

        if (u != 'u' && u != 'U') || (a != 'a' && a != 'A') {
            return None;
        }

        if TONED_TO_BASE.contains_key(&u) || TONED_TO_BASE.contains_key(&a) {
            return None;
        }

        if u_index > 0 {
            let prev_char = self.buffer[u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let u_horn = if u.is_uppercase() { 'Ư' } else { 'ư' };
        self.buffer[u_index] = u_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::CompoundUaw;

        let delete_count = self.buffer.len() - u_index;
        let output_text = self.buffer_string_from(u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_uaw_escape(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 3 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let a_index = trigger_index - 1;
        let u_index = a_index - 1;

        let u_horn = self.buffer[u_index];
        let a = self.buffer[a_index];

        if (u_horn != 'ư' && u_horn != 'Ư') || (a != 'a' && a != 'A') {
            return None;
        }

        let delete_count = trigger_index - u_index;
        let original_u = if u_horn.is_uppercase() { 'U' } else { 'u' };

        self.buffer[u_index] = original_u;
        self.clear_last_transform_and_suppress('w');

        let output_text = self.buffer_string_from(u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_uuw_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 3 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let last_u_index = trigger_index - 1;
        let first_u_index = last_u_index - 1;

        let first_u = self.buffer[first_u_index];
        let last_u = self.buffer[last_u_index];

        if (first_u != 'u' && first_u != 'U') || (last_u != 'u' && last_u != 'U') {
            return None;
        }

        if TONED_TO_BASE.contains_key(&first_u) || TONED_TO_BASE.contains_key(&last_u) {
            return None;
        }

        if first_u_index > 0 {
            let prev_char = self.buffer[first_u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let u_horn = if first_u.is_uppercase() { 'Ư' } else { 'ư' };
        self.buffer[first_u_index] = u_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::None;

        let delete_count = self.buffer.len() - first_u_index;
        let output_text = self.buffer_string_from(first_u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_ouw_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 3 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let u_index = trigger_index - 1;
        let o_index = u_index - 1;

        let o = self.buffer[o_index];
        let u = self.buffer[u_index];

        if (o != 'o' && o != 'O') || (u != 'u' && u != 'U') {
            return None;
        }

        if TONED_TO_BASE.contains_key(&o) || TONED_TO_BASE.contains_key(&u) {
            return None;
        }

        let u_horn = if u.is_uppercase() { 'Ư' } else { 'ư' };
        let o_horn = if o.is_uppercase() { 'Ơ' } else { 'ơ' };

        self.buffer[o_index] = u_horn;
        self.buffer[u_index] = o_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::CompoundUow;

        let delete_count = self.buffer.len() - o_index;
        let output_text = self.buffer_string_from(o_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_compound_uou_w_transform(&mut self) -> Option<KeyTransformAction> {
        if self.buffer.len() < 4 {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let last_u_index = trigger_index - 1;
        let o_index = last_u_index - 1;
        let first_u_index = o_index - 1;

        let first_u = self.buffer[first_u_index];
        let o = self.buffer[o_index];
        let last_u = self.buffer[last_u_index];

        if (first_u != 'u' && first_u != 'U')
            || (o != 'o' && o != 'O')
            || (last_u != 'u' && last_u != 'U')
        {
            return None;
        }

        if TONED_TO_BASE.contains_key(&first_u)
            || TONED_TO_BASE.contains_key(&o)
            || TONED_TO_BASE.contains_key(&last_u)
        {
            return None;
        }

        if first_u_index > 0 {
            let prev_char = self.buffer[first_u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let u_horn = if first_u.is_uppercase() { 'Ư' } else { 'ư' };
        let o_horn = if o.is_uppercase() { 'Ơ' } else { 'ơ' };

        self.buffer[first_u_index] = u_horn;
        self.buffer[o_index] = o_horn;
        self.buffer.pop();
        self.last_transform_key = Some('w');
        self.last_w_transform_kind = WTransformKind::CompoundUow;

        let delete_count = self.buffer.len() - first_u_index;
        let output_text = self.buffer_string_from(first_u_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    // ==================== Telex Helper Methods ====================

    fn find_last_matching_d_index(&self, end_index: usize, max_distance: usize) -> Option<usize> {
        let mut index = end_index;
        let mut distance = 0;
        while index > 0 && distance < max_distance {
            index -= 1;
            distance += 1;
            let ch = self.buffer[index];
            if lower_char(ch) == 'd' {
                return Some(index);
            }
        }
        None
    }

    fn find_last_untransformable_vowel(
        &self,
        key_lower: char,
        before: usize,
    ) -> Option<(usize, char)> {
        let mut index = before;
        while index > 0 {
            index -= 1;
            if let Some((key, original)) = VOWEL_UNTRANSFORMS.get(&self.buffer[index]) {
                if lower_char(*key) == key_lower {
                    return Some((index, *original));
                }
            }
        }
        None
    }

    fn find_last_transformable_vowel(
        &self,
        key: char,
        before: usize,
    ) -> Option<(usize, (char, char))> {
        let transforms = VOWEL_TRANSFORMS.get(&key)?;
        let mut index = before;
        while index > 0 {
            index -= 1;
            let ch = self.buffer[index];
            if let Some(transform) = transforms.iter().find(|(from, _)| *from == ch) {
                return Some((index, *transform));
            }
        }
        None
    }
}
