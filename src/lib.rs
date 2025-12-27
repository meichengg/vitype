use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use unicode_normalization::UnicodeNormalization;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum OutputEncoding {
    #[default]
    Unicode = 0,
    CompositeUnicode = 1,
}

#[derive(Clone, Copy, PartialEq)]
enum WTransformKind {
    None,
    Standalone,
    CompoundUow,
    CompoundUoFinalConsonantW,
    CompoundUaw,
}

#[derive(Clone, Debug, PartialEq)]
struct KeyTransformAction {
    delete_count: usize,
    text: String,
}

pub struct VitypeEngine {
    buffer: Vec<char>,
    raw_buffer: Vec<char>,
    is_foreign_mode: bool,
    last_transform_key: Option<char>,
    last_w_transform_kind: WTransformKind,
    suppressed_transform_key: Option<char>,
    auto_fix_tone: bool,
    output_encoding: OutputEncoding,
}

impl VitypeEngine {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            raw_buffer: Vec::new(),
            is_foreign_mode: false,
            last_transform_key: None,
            last_w_transform_kind: WTransformKind::None,
            suppressed_transform_key: None,
            auto_fix_tone: true,
            output_encoding: OutputEncoding::Unicode,
        }
    }

    fn process(&mut self, input: &str) -> Option<KeyTransformAction> {
        let mut chars = input.chars();
        let ch = chars.next()?;
        if chars.next().is_some() {
            return None;
        }

        if is_word_boundary(ch) {
            self.reset();
            return None;
        }

        let previous_buffer_count = self.buffer.len();
        let ch_lower = lower_char(ch);
        if let Some(suppressed) = self.suppressed_transform_key {
            if suppressed != ch_lower {
                self.suppressed_transform_key = None;
            }
        }

        self.raw_buffer.push(ch);

        if self.is_foreign_mode {
            self.buffer.push(ch);
            return None;
        }

        if let Some(action) = self.try_escape_sequence(ch) {
            if !self.raw_buffer.is_empty() {
                self.raw_buffer.pop();
            }
            if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
                return Some(fallback);
            }
            return Some(action);
        }

        self.buffer.push(ch);

        if self.suppressed_transform_key == Some(ch_lower) {
            if self.auto_fix_tone && is_vowel(ch) {
                if let Some(action) = self.reposition_tone_if_needed(true, None) {
                    if let Some(fallback) =
                        self.handle_invalid_syllable_if_needed(previous_buffer_count)
                    {
                        return Some(fallback);
                    }
                    return Some(action);
                }
            }
            self.last_transform_key = None;
            self.last_w_transform_kind = WTransformKind::None;
            return self.handle_invalid_syllable_if_needed(previous_buffer_count);
        }

        if let Some(action) = self.try_consonant_transform(ch) {
            if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
                return Some(fallback);
            }
            return Some(action);
        }

        if let Some(action) = self.try_vowel_transform(ch) {
            if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
                return Some(fallback);
            }
            return Some(action);
        }

        if let Some(action) = self.try_tone_mark(ch) {
            if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
                return Some(fallback);
            }
            return Some(action);
        }

        if self.auto_fix_tone && is_vowel(ch) {
            if let Some(action) = self.reposition_tone_if_needed(true, None) {
                if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
                    return Some(fallback);
                }
                return Some(action);
            }
        }

        if self.auto_fix_tone && !is_vowel(ch) {
            if let Some(action) = self.reposition_tone_if_needed(true, None) {
                if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count)
                {
                    return Some(fallback);
                }
                return Some(action);
            }
        }

        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.handle_invalid_syllable_if_needed(previous_buffer_count)
    }

    fn try_escape_sequence(&mut self, ch: char) -> Option<KeyTransformAction> {
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
                            self.last_transform_key = None;
                            self.last_w_transform_kind = WTransformKind::None;
                            self.suppressed_transform_key = Some(ch_lower);
                            return Some(KeyTransformAction {
                                delete_count: 1,
                                text: replacement.to_string(),
                            });
                        }
                    }
                }
                WTransformKind::CompoundUow => {
                    let end_index = self.buffer.len();
                    if self.buffer.len() >= 2 {
                        let o_index = end_index - 1;
                        let u_index = end_index - 2;
                        let u_horn = self.buffer[u_index];
                        let o_horn = self.buffer[o_index];
                        if (u_horn == 'ư' || u_horn == 'Ư') && (o_horn == 'ơ' || o_horn == 'Ơ') {
                            let original_u = if u_horn.is_uppercase() { 'U' } else { 'u' };
                            let original_o = if o_horn.is_uppercase() { 'O' } else { 'o' };
                            self.buffer.drain(u_index..);
                            self.buffer.push(original_u);
                            self.buffer.push(original_o);
                            self.buffer.push(ch);
                            self.last_transform_key = None;
                            self.last_w_transform_kind = WTransformKind::None;
                            self.suppressed_transform_key = Some(ch_lower);
                            return Some(KeyTransformAction {
                                delete_count: 2,
                                text: format!("{}{}{}", original_u, original_o, ch),
                            });
                        }
                    }
                }
                WTransformKind::CompoundUoFinalConsonantW => {
                    if self.buffer.len() >= 3 {
                        let mut o_index = self.buffer.len();
                        while o_index > 0 {
                            o_index -= 1;
                            if is_vowel(self.buffer[o_index]) {
                                break;
                            }
                        }
                        if o_index < self.buffer.len() && is_vowel(self.buffer[o_index]) {
                            if o_index == 0 {
                                return None;
                            }
                            let u_index = o_index - 1;
                            let u_horn = self.buffer[u_index];
                            let o_horn = self.buffer[o_index];
                            if (u_horn == 'ư' || u_horn == 'Ư') && (o_horn == 'ơ' || o_horn == 'Ơ') {
                                let delete_count = self.buffer.len() - u_index;
                                let original_u = if u_horn.is_uppercase() { 'U' } else { 'u' };
                                let original_o = if o_horn.is_uppercase() { 'O' } else { 'o' };
                                self.buffer[u_index] = original_u;
                                self.buffer[o_index] = original_o;
                                self.buffer.push(ch);
                                self.last_transform_key = None;
                                self.last_w_transform_kind = WTransformKind::None;
                                self.suppressed_transform_key = Some(ch_lower);
                                let output_text = self.buffer_string_from(u_index);
                                return Some(KeyTransformAction {
                                    delete_count,
                                    text: output_text,
                                });
                            }
                        }
                    }
                }
                WTransformKind::CompoundUaw => {
                    let end_index = self.buffer.len();
                    if self.buffer.len() >= 2 {
                        let a_index = end_index - 1;
                        let u_index = end_index - 2;
                        let u_horn = self.buffer[u_index];
                        let a_char = self.buffer[a_index];
                        if (u_horn == 'ư' || u_horn == 'Ư') && (a_char == 'a' || a_char == 'A') {
                            let original_u = if u_horn.is_uppercase() { 'U' } else { 'u' };
                            self.buffer.drain(u_index..);
                            self.buffer.push(original_u);
                            self.buffer.push(a_char);
                            self.buffer.push(ch);
                            self.last_transform_key = None;
                            self.last_w_transform_kind = WTransformKind::None;
                            self.suppressed_transform_key = Some(ch_lower);
                            return Some(KeyTransformAction {
                                delete_count: 2,
                                text: format!("{}{}{}", original_u, a_char, ch),
                            });
                        }
                    }
                }
                WTransformKind::None => {
                    if let Some((index, original)) =
                        self.find_last_untransformable_vowel(ch_lower, self.buffer.len())
                    {
                        let delete_count = self.buffer.len() - index;
                        self.buffer[index] = original;
                        self.buffer.push(ch);
                        self.last_transform_key = None;
                        self.last_w_transform_kind = WTransformKind::None;
                        self.suppressed_transform_key = Some(ch_lower);
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
                    self.last_transform_key = None;
                    self.last_w_transform_kind = WTransformKind::None;
                    self.suppressed_transform_key = Some(ch_lower);
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
                    self.last_transform_key = None;
                    self.last_w_transform_kind = WTransformKind::None;
                    self.suppressed_transform_key = Some(ch_lower);
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
                self.last_transform_key = None;
                self.last_w_transform_kind = WTransformKind::None;
                self.suppressed_transform_key = Some(ch_lower);
                let output_text = self.buffer_string_from(index);
                return Some(KeyTransformAction {
                    delete_count,
                    text: output_text,
                });
            }
        }

        if is_tone_key(ch) {
            if let Some(toned_index) = self.find_last_toned_vowel_index() {
                if let Some((base_vowel, last_tone)) = TONED_TO_BASE.get(&self.buffer[toned_index])
                {
                    if lower_char(*last_tone) == ch_lower {
                        let delete_count = self.buffer.len() - toned_index;
                        self.buffer[toned_index] = *base_vowel;
                        self.buffer.push(ch);
                        self.last_transform_key = None;
                        self.last_w_transform_kind = WTransformKind::None;
                        self.suppressed_transform_key = Some(ch_lower);
                        let output_text = self.buffer_string_from(toned_index);
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

    fn try_consonant_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
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

    fn try_vowel_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
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
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = Some('w');

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

    fn try_tone_mark(&mut self, ch: char) -> Option<KeyTransformAction> {
        let ch_lower = lower_char(ch);
        if !is_tone_key(ch_lower) {
            return None;
        }

        if self.buffer.is_empty() {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let vowel_index = self.find_target_vowel_index(trigger_index)?;
        let vowel = self.buffer[vowel_index];
        let delete_count = trigger_index - vowel_index;

        if ch_lower == 'z' {
            let base_vowel = self.get_base_vowel(vowel);
            if base_vowel == vowel {
                return None;
            }
            self.buffer[vowel_index] = base_vowel;
            self.buffer.pop();
            self.last_transform_key = Some('z');
            self.last_w_transform_kind = WTransformKind::None;
            let output_text = self.buffer_string_from(vowel_index);
            return Some(KeyTransformAction {
                delete_count,
                text: output_text,
            });
        }

        let base_vowel = self.get_base_vowel(vowel);
        let tone_map = VOWEL_TO_TONED.get(&base_vowel)?;
        let toned_vowel = tone_map.get(&ch_lower)?;

        self.buffer[vowel_index] = *toned_vowel;
        self.buffer.pop();
        self.last_transform_key = Some(ch);
        self.last_w_transform_kind = WTransformKind::None;

        let output_text = self.buffer_string_from(vowel_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn reposition_tone_if_needed(
        &mut self,
        suppressed_last_char: bool,
        min_start_offset: Option<usize>,
    ) -> Option<KeyTransformAction> {
        let mut toned_index: Option<usize> = None;
        let mut tone_key: Option<char> = None;
        for (idx, ch) in self.buffer.iter().enumerate() {
            if let Some((_, tone)) = TONED_TO_BASE.get(ch) {
                toned_index = Some(idx);
                tone_key = Some(*tone);
            }
        }

        let current_toned_index = toned_index?;
        let tone = tone_key?;
        let target_index = self.find_target_vowel_index(self.buffer.len())?;
        if current_toned_index == target_index {
            return None;
        }

        let toned_vowel = self.buffer[current_toned_index];
        let (base_vowel, _) = TONED_TO_BASE.get(&toned_vowel)?;
        let target_vowel = self.buffer[target_index];
        let target_base = self.get_base_vowel(target_vowel);

        let tone_map = VOWEL_TO_TONED.get(&target_base)?;
        let new_toned_vowel = tone_map.get(&tone)?;

        let current_offset = current_toned_index;
        let target_offset = target_index;
        let mut start_offset = if current_offset < target_offset {
            current_offset
        } else {
            target_offset
        };
        if let Some(min_start) = min_start_offset {
            if min_start < start_offset {
                start_offset = min_start;
            }
        }

        self.buffer[current_offset] = *base_vowel;
        self.buffer[target_offset] = *new_toned_vowel;

        let delete_adjustment = if suppressed_last_char { 1 } else { 0 };
        let mut delete_count = self.buffer.len().saturating_sub(start_offset);
        if delete_adjustment > 0 {
            delete_count = delete_count.saturating_sub(delete_adjustment);
        }
        let output_text = self.buffer_string_from(start_offset);

        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn handle_invalid_syllable_if_needed(
        &mut self,
        previous_buffer_count: usize,
    ) -> Option<KeyTransformAction> {
        if !self.has_multiple_vowel_clusters(self.buffer.len()) {
            return None;
        }

        self.is_foreign_mode = true;
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = None;

        let output_text = self.raw_buffer.iter().collect::<String>();
        let delete_count = previous_buffer_count;
        self.buffer = self.raw_buffer.clone();

        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn has_multiple_vowel_clusters(&self, before: usize) -> bool {
        let vowel_indices = self.effective_vowel_indices(before);
        if vowel_indices.len() <= 1 {
            return false;
        }

        let mut previous = vowel_indices[0];
        for index in vowel_indices.iter().skip(1) {
            if *index != previous + 1 {
                return true;
            }
            previous = *index;
        }

        false
    }

    fn effective_vowel_indices(&self, before: usize) -> Vec<usize> {
        let mut all_indices = Vec::new();
        for idx in 0..before {
            if is_vowel(self.buffer[idx]) {
                all_indices.push(idx);
            }
        }

        let mut vowel_indices = Vec::new();
        for (i, &vowel_index) in all_indices.iter().enumerate() {
            let ch = self.buffer[vowel_index];
            let base_vowel = self.get_base_vowel(ch);
            let base_lower = lower_char(base_vowel);
            let mut skip = false;

            if base_lower == 'u' && vowel_index > 0 {
                let prev_char = self.buffer[vowel_index - 1];
                if prev_char == 'q' || prev_char == 'Q' {
                    skip = true;
                }
            }

            if base_lower == 'i' && vowel_index > 0 {
                let prev_char = self.buffer[vowel_index - 1];
                if prev_char == 'g' || prev_char == 'G' {
                    if i < all_indices.len() - 1 {
                        skip = true;
                    }
                }
            }

            if !skip {
                vowel_indices.push(vowel_index);
            }
        }

        vowel_indices
    }

    fn find_target_vowel_index(&self, before: usize) -> Option<usize> {
        let mut all_vowel_indices = Vec::new();
        for idx in 0..before {
            if is_vowel(self.buffer[idx]) {
                all_vowel_indices.push(idx);
            }
        }

        if all_vowel_indices.is_empty() {
            return None;
        }

        let mut vowel_indices = Vec::new();
        for (i, &vowel_index) in all_vowel_indices.iter().enumerate() {
            let ch = self.buffer[vowel_index];
            let mut skip = false;

            if (ch == 'u' || ch == 'U') && vowel_index > 0 {
                let prev_char = self.buffer[vowel_index - 1];
                if prev_char == 'q' || prev_char == 'Q' {
                    skip = true;
                }
            }

            let base_vowel = self.get_base_vowel(ch);
            let base_lower = lower_char(base_vowel);
            if base_lower == 'i' && vowel_index > 0 {
                let prev_char = self.buffer[vowel_index - 1];
                if prev_char == 'g' || prev_char == 'G' {
                    if i < all_vowel_indices.len() - 1 {
                        skip = true;
                    }
                }
            }

            if !skip {
                vowel_indices.push(vowel_index);
            }
        }

        if vowel_indices.len() == 1 {
            return Some(vowel_indices[0]);
        }

        for &vowel_index in vowel_indices.iter().rev() {
            let ch = self.buffer[vowel_index];
            if NUCLEUS_ONLY_VOWELS.contains(&ch) {
                return Some(vowel_index);
            }
        }

        if vowel_indices.len() == 2 {
            let has_final_consonant = vowel_indices[1] + 1 < before;

            if has_final_consonant {
                return Some(vowel_indices[1]);
            }

            return Some(vowel_indices[0]);
        }

        let middle_index = (vowel_indices.len() - 1) / 2;
        Some(vowel_indices[middle_index])
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

    fn find_last_toned_vowel_index(&self) -> Option<usize> {
        let mut index = self.buffer.len();
        while index > 0 {
            index -= 1;
            if TONED_TO_BASE.contains_key(&self.buffer[index]) {
                return Some(index);
            }
        }
        None
    }

    fn find_last_matching_vowel_index(
        &self,
        key: char,
        before: usize,
        max_distance: usize,
    ) -> Option<usize> {
        let key_lower = lower_char(key);
        let mut index = before;
        let mut distance = 0;
        if before == 0 {
            return None;
        }
        let adjacent_index = before - 1;

        while index > 0 && distance < max_distance {
            index -= 1;
            distance += 1;
            let ch = self.buffer[index];
            if is_vowel(ch) {
                let base_vowel = self.get_base_vowel(ch);
                let base_lower = lower_char(base_vowel);
                if base_lower != key_lower {
                    if index == adjacent_index
                        && (base_lower == 'i' || base_lower == 'y' || base_lower == 'u')
                    {
                        continue;
                    }
                    return None;
                }

                if base_lower == key_lower {
                    let ch_lower = lower_char(ch);
                    if ch_lower == key_lower || TONED_TO_BASE.contains_key(&ch) {
                        let actual_base =
                            TONED_TO_BASE.get(&ch).map(|(base, _)| *base).unwrap_or(ch);
                        let actual_base_lower = lower_char(actual_base);
                        if actual_base_lower == key_lower {
                            return Some(index);
                        }
                    }
                }

                return None;
            }
        }

        None
    }

    fn get_base_vowel(&self, ch: char) -> char {
        if BASE_VOWELS.contains(&ch) {
            return ch;
        }
        if let Some((base, _)) = TONED_TO_BASE.get(&ch) {
            return *base;
        }
        ch
    }

    fn buffer_string_from(&self, start: usize) -> String {
        self.buffer[start..].iter().collect()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.raw_buffer.clear();
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = None;
        self.is_foreign_mode = false;
    }

    fn delete_last_character(&mut self) {
        self.buffer.pop();
        self.raw_buffer.pop();
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = None;
        self.is_foreign_mode = self.has_multiple_vowel_clusters(self.buffer.len());
    }
}

fn lower_char(ch: char) -> char {
    ch.to_lowercase().next().unwrap_or(ch)
}

fn is_tone_key(ch: char) -> bool {
    TONE_KEYS.contains(&lower_char(ch))
}

fn is_word_boundary(ch: char) -> bool {
    WORD_BOUNDARY_CHARS.contains(&ch)
}

fn is_vowel(ch: char) -> bool {
    VOWELS.contains(&ch)
}

fn convert_to_output_encoding(text: String, encoding: OutputEncoding) -> String {
    match encoding {
        OutputEncoding::Unicode => text,
        OutputEncoding::CompositeUnicode => text.nfd().collect(),
    }
}

#[repr(C)]
pub struct VitypeTransformResult {
    pub has_action: bool,
    pub delete_count: i32,
    pub text: *mut c_char,
}

fn empty_result() -> VitypeTransformResult {
    VitypeTransformResult {
        has_action: false,
        delete_count: 0,
        text: ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_new() -> *mut VitypeEngine {
    Box::into_raw(Box::new(VitypeEngine::new()))
}

#[no_mangle]
pub extern "C" fn vitype_engine_free(engine: *mut VitypeEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_reset(engine: *mut VitypeEngine) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).reset();
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_delete_last_character(engine: *mut VitypeEngine) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).delete_last_character();
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_set_auto_fix_tone(engine: *mut VitypeEngine, enabled: bool) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).auto_fix_tone = enabled;
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_set_output_encoding(engine: *mut VitypeEngine, encoding: i32) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).output_encoding = match encoding {
            1 => OutputEncoding::CompositeUnicode,
            _ => OutputEncoding::Unicode,
        };
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_process(
    engine: *mut VitypeEngine,
    input_utf8: *const c_char,
) -> VitypeTransformResult {
    if engine.is_null() || input_utf8.is_null() {
        return empty_result();
    }

    let input = unsafe { CStr::from_ptr(input_utf8) };
    let input_str = match input.to_str() {
        Ok(value) => value,
        Err(_) => return empty_result(),
    };

    let (action, output_encoding) = unsafe {
        ((*engine).process(input_str), (*engine).output_encoding)
    };
    match action {
        Some(action) => {
            let output_text = convert_to_output_encoding(action.text, output_encoding);
            let c_text = CString::new(output_text).unwrap_or_else(|_| CString::new("").unwrap());
            VitypeTransformResult {
                has_action: true,
                delete_count: action.delete_count as i32,
                text: c_text.into_raw(),
            }
        }
        None => empty_result(),
    }
}

#[no_mangle]
pub extern "C" fn vitype_engine_free_string(text: *mut c_char) {
    if text.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(text));
    }
}

static TONE_KEYS: Lazy<HashSet<char>> = Lazy::new(|| {
    let chars = ['s', 'f', 'r', 'x', 'j', 'z'];
    chars.iter().cloned().collect()
});

static WORD_BOUNDARY_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    let chars = [
        ' ', '\n', '\r', '\t', ',', '.', ';', ':', '!', '?',
        '(', ')', '[', ']', '{', '}', '"', '\'', '/', '\\',
        '-', '_', '@', '#', '$', '%', '^', '&', '*', '=', '+',
        '<', '>', '`', '~', '|',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    chars.iter().cloned().collect()
});

static VOWELS: Lazy<HashSet<char>> = Lazy::new(|| {
    let chars = [
        'a', 'ă', 'â', 'e', 'ê', 'i', 'o', 'ô', 'ơ', 'u', 'ư', 'y',
        'A', 'Ă', 'Â', 'E', 'Ê', 'I', 'O', 'Ô', 'Ơ', 'U', 'Ư', 'Y',
        'á', 'à', 'ả', 'ã', 'ạ',
        'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ',
        'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ',
        'é', 'è', 'ẻ', 'ẽ', 'ẹ',
        'ế', 'ề', 'ể', 'ễ', 'ệ',
        'í', 'ì', 'ỉ', 'ĩ', 'ị',
        'ó', 'ò', 'ỏ', 'õ', 'ọ',
        'ố', 'ồ', 'ổ', 'ỗ', 'ộ',
        'ớ', 'ờ', 'ở', 'ỡ', 'ợ',
        'ú', 'ù', 'ủ', 'ũ', 'ụ',
        'ứ', 'ừ', 'ử', 'ữ', 'ự',
        'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ',
        'Á', 'À', 'Ả', 'Ã', 'Ạ',
        'Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ',
        'Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ',
        'É', 'È', 'Ẻ', 'Ẽ', 'Ẹ',
        'Ế', 'Ề', 'Ể', 'Ễ', 'Ệ',
        'Í', 'Ì', 'Ỉ', 'Ĩ', 'Ị',
        'Ó', 'Ò', 'Ỏ', 'Õ', 'Ọ',
        'Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ',
        'Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ',
        'Ú', 'Ù', 'Ủ', 'Ũ', 'Ụ',
        'Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự',
        'Ý', 'Ỳ', 'Ỷ', 'Ỹ', 'Ỵ',
    ];
    chars.iter().cloned().collect()
});

static VOWEL_TO_TONED: Lazy<HashMap<char, HashMap<char, char>>> = Lazy::new(|| {
    fn tone_map(entries: &[(char, char)]) -> HashMap<char, char> {
        entries.iter().cloned().collect()
    }

    let mut map = HashMap::new();
    map.insert('a', tone_map(&[('s', 'á'), ('f', 'à'), ('r', 'ả'), ('x', 'ã'), ('j', 'ạ')]));
    map.insert('ă', tone_map(&[('s', 'ắ'), ('f', 'ằ'), ('r', 'ẳ'), ('x', 'ẵ'), ('j', 'ặ')]));
    map.insert('â', tone_map(&[('s', 'ấ'), ('f', 'ầ'), ('r', 'ẩ'), ('x', 'ẫ'), ('j', 'ậ')]));
    map.insert('e', tone_map(&[('s', 'é'), ('f', 'è'), ('r', 'ẻ'), ('x', 'ẽ'), ('j', 'ẹ')]));
    map.insert('ê', tone_map(&[('s', 'ế'), ('f', 'ề'), ('r', 'ể'), ('x', 'ễ'), ('j', 'ệ')]));
    map.insert('i', tone_map(&[('s', 'í'), ('f', 'ì'), ('r', 'ỉ'), ('x', 'ĩ'), ('j', 'ị')]));
    map.insert('o', tone_map(&[('s', 'ó'), ('f', 'ò'), ('r', 'ỏ'), ('x', 'õ'), ('j', 'ọ')]));
    map.insert('ô', tone_map(&[('s', 'ố'), ('f', 'ồ'), ('r', 'ổ'), ('x', 'ỗ'), ('j', 'ộ')]));
    map.insert('ơ', tone_map(&[('s', 'ớ'), ('f', 'ờ'), ('r', 'ở'), ('x', 'ỡ'), ('j', 'ợ')]));
    map.insert('u', tone_map(&[('s', 'ú'), ('f', 'ù'), ('r', 'ủ'), ('x', 'ũ'), ('j', 'ụ')]));
    map.insert('ư', tone_map(&[('s', 'ứ'), ('f', 'ừ'), ('r', 'ử'), ('x', 'ữ'), ('j', 'ự')]));
    map.insert('y', tone_map(&[('s', 'ý'), ('f', 'ỳ'), ('r', 'ỷ'), ('x', 'ỹ'), ('j', 'ỵ')]));
    map.insert('A', tone_map(&[('s', 'Á'), ('f', 'À'), ('r', 'Ả'), ('x', 'Ã'), ('j', 'Ạ')]));
    map.insert('Ă', tone_map(&[('s', 'Ắ'), ('f', 'Ằ'), ('r', 'Ẳ'), ('x', 'Ẵ'), ('j', 'Ặ')]));
    map.insert('Â', tone_map(&[('s', 'Ấ'), ('f', 'Ầ'), ('r', 'Ẩ'), ('x', 'Ẫ'), ('j', 'Ậ')]));
    map.insert('E', tone_map(&[('s', 'É'), ('f', 'È'), ('r', 'Ẻ'), ('x', 'Ẽ'), ('j', 'Ẹ')]));
    map.insert('Ê', tone_map(&[('s', 'Ế'), ('f', 'Ề'), ('r', 'Ể'), ('x', 'Ễ'), ('j', 'Ệ')]));
    map.insert('I', tone_map(&[('s', 'Í'), ('f', 'Ì'), ('r', 'Ỉ'), ('x', 'Ĩ'), ('j', 'Ị')]));
    map.insert('O', tone_map(&[('s', 'Ó'), ('f', 'Ò'), ('r', 'Ỏ'), ('x', 'Õ'), ('j', 'Ọ')]));
    map.insert('Ô', tone_map(&[('s', 'Ố'), ('f', 'Ồ'), ('r', 'Ổ'), ('x', 'Ỗ'), ('j', 'Ộ')]));
    map.insert('Ơ', tone_map(&[('s', 'Ớ'), ('f', 'Ờ'), ('r', 'Ở'), ('x', 'Ỡ'), ('j', 'Ợ')]));
    map.insert('U', tone_map(&[('s', 'Ú'), ('f', 'Ù'), ('r', 'Ủ'), ('x', 'Ũ'), ('j', 'Ụ')]));
    map.insert('Ư', tone_map(&[('s', 'Ứ'), ('f', 'Ừ'), ('r', 'Ử'), ('x', 'Ữ'), ('j', 'Ự')]));
    map.insert('Y', tone_map(&[('s', 'Ý'), ('f', 'Ỳ'), ('r', 'Ỷ'), ('x', 'Ỹ'), ('j', 'Ỵ')]));
    map
});

static TONED_TO_BASE: Lazy<HashMap<char, (char, char)>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for (base, tones) in VOWEL_TO_TONED.iter() {
        for (tone, accented) in tones.iter() {
            map.insert(*accented, (*base, *tone));
        }
    }
    map
});

static VOWEL_TRANSFORMS: Lazy<HashMap<char, Vec<(char, char)>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('a', vec![('a', 'â'), ('A', 'Â')]);
    map.insert('A', vec![('a', 'Â'), ('A', 'Â')]);
    map.insert('e', vec![('e', 'ê'), ('E', 'Ê')]);
    map.insert('E', vec![('e', 'Ê'), ('E', 'Ê')]);
    map.insert('o', vec![('o', 'ô'), ('O', 'Ô')]);
    map.insert('O', vec![('o', 'Ô'), ('O', 'Ô')]);
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

static NUCLEUS_ONLY_VOWELS: Lazy<HashSet<char>> = Lazy::new(|| {
    let chars = [
        'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ',
        'Ă', 'Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ',
        'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ',
        'Â', 'Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ',
        'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ',
        'Ê', 'Ế', 'Ề', 'Ể', 'Ễ', 'Ệ',
        'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ',
        'Ô', 'Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ',
        'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ',
        'Ơ', 'Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ',
        'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự',
        'Ư', 'Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự',
    ];
    chars.iter().cloned().collect()
});

static BASE_VOWELS: Lazy<HashSet<char>> = Lazy::new(|| {
    let chars = [
        'a', 'ă', 'â', 'e', 'ê', 'i', 'o', 'ô', 'ơ', 'u', 'ư', 'y',
        'A', 'Ă', 'Â', 'E', 'Ê', 'I', 'O', 'Ô', 'Ơ', 'U', 'Ư', 'Y',
    ];
    chars.iter().cloned().collect()
});

#[cfg(test)]
mod tests;
