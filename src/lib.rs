mod common;
mod telex;
mod vni;
mod ffi;

use std::collections::VecDeque;

pub use ffi::VitypeTransformResult;

// Re-export crate-visible types from common (keep public API surface minimal)
pub(crate) use common::{
    InputMethod, KeyTransformAction, OutputEncoding, TonePlacement, WTransformKind,
};

// Use internal items from common
use common::{
    is_vowel, lower_char, BASE_VOWELS, TONED_TO_BASE, VOWEL_TO_TONED,
};

// Use internal items from telex and vni
use telex::is_telex_word_boundary;
use vni::is_vni_word_boundary;

// ==================== VitypeEngine ====================

const HISTORY_WORD_LIMIT: usize = 3;

#[derive(Clone, Debug)]
struct WordSegment {
    buffer: Vec<char>,
    raw_buffer: Vec<char>,
    is_foreign_mode: bool,
    transforms_locked: bool,
}

#[derive(Clone, Debug)]
enum HistorySegment {
    Word(WordSegment),
    Boundary(Vec<char>),
}

pub struct VitypeEngine {
    buffer: Vec<char>,
    raw_buffer: Vec<char>,
    history: VecDeque<HistorySegment>,
    is_foreign_mode: bool,
    transforms_locked: bool,
    last_transform_key: Option<char>,
    last_w_transform_kind: WTransformKind,
    suppressed_transform_key: Option<char>,
    auto_fix_tone: bool,
    tone_placement: TonePlacement,
    output_encoding: OutputEncoding,
    input_method: InputMethod,
}

impl VitypeEngine {
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::new(),
            raw_buffer: Vec::new(),
            history: VecDeque::new(),
            is_foreign_mode: false,
            transforms_locked: false,
            last_transform_key: None,
            last_w_transform_kind: WTransformKind::None,
            suppressed_transform_key: None,
            auto_fix_tone: true,
            tone_placement: TonePlacement::Orthographic,
            output_encoding: OutputEncoding::Unicode,
            input_method: InputMethod::Telex,
        }
    }

    fn return_action_or_fallback(
        &mut self,
        action: KeyTransformAction,
        previous_buffer_count: usize,
    ) -> Option<KeyTransformAction> {
        if let Some(fallback) = self.handle_invalid_syllable_if_needed(previous_buffer_count) {
            return Some(fallback);
        }
        Some(action)
    }

    fn clear_last_transform_state(&mut self) {
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
    }

    fn clear_transform_state(&mut self) {
        self.clear_last_transform_state();
        self.suppressed_transform_key = None;
    }

    fn clear_last_transform_and_suppress(&mut self, suppressed_key: char) {
        self.clear_last_transform_state();
        self.suppressed_transform_key = Some(suppressed_key);
        self.transforms_locked = true;
    }

    pub(crate) fn set_auto_fix_tone(&mut self, enabled: bool) {
        self.auto_fix_tone = enabled;
    }

    pub(crate) fn set_input_method(&mut self, method: InputMethod) {
        self.input_method = method;
    }

    pub(crate) fn set_output_encoding(&mut self, encoding: OutputEncoding) {
        self.output_encoding = encoding;
    }

    pub(crate) fn set_tone_placement(&mut self, placement: TonePlacement) {
        self.tone_placement = placement;
    }

    pub(crate) fn output_encoding(&self) -> OutputEncoding {
        self.output_encoding
    }

    pub(crate) fn process(&mut self, input: &str) -> Option<KeyTransformAction> {
        let mut chars = input.chars();
        let ch = chars.next()?;
        if chars.next().is_some() {
            return None;
        }

        if is_word_boundary(ch, self.input_method) {
            self.commit_current_word_to_history_if_needed();
            self.push_boundary_to_history(ch);
            self.reset_current_word();
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

        if self.is_foreign_mode || self.transforms_locked {
            self.buffer.push(ch);
            return None;
        }

        if let Some(action) = self.try_escape_sequence(ch) {
            self.raw_buffer.pop();
            return self.return_action_or_fallback(action, previous_buffer_count);
        }

        self.buffer.push(ch);

        if self.suppressed_transform_key == Some(ch_lower) {
            if self.auto_fix_tone && is_vowel(ch) {
                if let Some(action) = self.reposition_tone_if_needed(true, None) {
                    return self.return_action_or_fallback(action, previous_buffer_count);
                }
            }
            self.clear_last_transform_state();
            return self.handle_invalid_syllable_if_needed(previous_buffer_count);
        }

        if let Some(action) = self.try_consonant_transform(ch) {
            return self.return_action_or_fallback(action, previous_buffer_count);
        }

        if let Some(action) = self.try_vowel_transform(ch) {
            return self.return_action_or_fallback(action, previous_buffer_count);
        }

        if let Some(action) = self.try_tone_mark(ch) {
            return self.return_action_or_fallback(action, previous_buffer_count);
        }

        if let Some(action) = self.try_auto_fix_uhorn_o_before_consonant(ch) {
            return self.return_action_or_fallback(action, previous_buffer_count);
        }

        if self.auto_fix_tone {
            if let Some(action) = self.reposition_tone_if_needed(true, None) {
                return self.return_action_or_fallback(action, previous_buffer_count);
            }
        }

        self.clear_last_transform_state();
        self.handle_invalid_syllable_if_needed(previous_buffer_count)
    }

    // ==================== Dispatch Methods ====================

    fn try_escape_sequence(&mut self, ch: char) -> Option<KeyTransformAction> {
        match self.input_method {
            InputMethod::Telex => self.try_telex_escape_sequence(ch),
            InputMethod::Vni => self.try_vni_escape_sequence(ch),
        }
    }

    fn try_consonant_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        match self.input_method {
            InputMethod::Telex => self.try_telex_consonant_transform(ch),
            InputMethod::Vni => self.try_vni_consonant_transform(ch),
        }
    }

    fn try_vowel_transform(&mut self, ch: char) -> Option<KeyTransformAction> {
        match self.input_method {
            InputMethod::Telex => self.try_telex_vowel_transform(ch),
            InputMethod::Vni => self.try_vni_vowel_transform(ch),
        }
    }

    fn try_tone_mark(&mut self, ch: char) -> Option<KeyTransformAction> {
        match self.input_method {
            InputMethod::Telex => self.try_telex_tone_mark(ch),
            InputMethod::Vni => self.try_vni_tone_mark(ch),
        }
    }

    // ==================== Common Engine Methods ====================

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

    fn try_auto_fix_uhorn_o_before_consonant(
        &mut self,
        ch: char,
    ) -> Option<KeyTransformAction> {
        if is_vowel(ch) {
            return None;
        }

        if self.buffer.len() < 3 {
            return None;
        }

        let last_index = self.buffer.len() - 1;
        if is_vowel(self.buffer[last_index]) {
            return None;
        }

        let mut scan_index = last_index;
        while scan_index > 0 && !is_vowel(self.buffer[scan_index]) {
            scan_index -= 1;
        }

        if !is_vowel(self.buffer[scan_index]) {
            return None;
        }

        let o_index = scan_index;
        let o_char = self.buffer[o_index];
        let o_base = self.get_base_vowel(o_char);
        if lower_char(o_base) != 'o' {
            return None;
        }
        if o_index == 0 {
            return None;
        }

        let u_index = o_index - 1;
        let u_char = self.buffer[u_index];
        let u_base = self.get_base_vowel(u_char);
        if lower_char(u_base) != 'ư' {
            return None;
        }

        if u_index > 0 {
            let prev_char = self.buffer[u_index - 1];
            if prev_char == 'q' || prev_char == 'Q' {
                return None;
            }
        }

        let horn_base = if o_base.is_uppercase() { 'Ơ' } else { 'ơ' };
        let new_o = if let Some((_, tone_key)) = TONED_TO_BASE.get(&o_char) {
            let tone_map = VOWEL_TO_TONED.get(&horn_base)?;
            *tone_map.get(tone_key)?
        } else {
            horn_base
        };

        if new_o == o_char {
            return None;
        }

        self.buffer[o_index] = new_o;
        self.clear_last_transform_state();

        if self.auto_fix_tone {
            if let Some(action) = self.reposition_tone_if_needed(true, Some(o_index)) {
                return Some(action);
            }
        }

        let delete_count = self.buffer.len().saturating_sub(o_index + 1);
        let output_text = self.buffer_string_from(o_index);
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

        let needs_visible_rewrite = self.buffer != self.raw_buffer;

        self.is_foreign_mode = true;
        self.clear_transform_state();

        self.buffer = self.raw_buffer.clone();

        if !needs_visible_rewrite {
            return None;
        }

        let output_text = self.raw_buffer.iter().collect::<String>();
        Some(KeyTransformAction {
            delete_count: previous_buffer_count,
            text: output_text,
        })
    }

    fn has_multiple_vowel_clusters(&self, before: usize) -> bool {
        let mut previous_index: Option<usize> = None;
        let mut has_gap = false;

        self.for_each_effective_vowel_index(before, |index| {
            if let Some(previous) = previous_index {
                if index != previous + 1 {
                    has_gap = true;
                    return false;
                }
            }
            previous_index = Some(index);
            true
        });

        has_gap
    }

    fn is_u_vowel_after_q(&self, vowel_index: usize) -> bool {
        if vowel_index == 0 {
            return false;
        }
        let ch = self.buffer[vowel_index];
        let base_vowel = self.get_base_vowel(ch);
        if lower_char(base_vowel) != 'u' {
            return false;
        }
        let prev_char = self.buffer[vowel_index - 1];
        prev_char == 'q' || prev_char == 'Q'
    }

    fn is_gi_i_candidate(&self, vowel_index: usize) -> bool {
        if vowel_index == 0 {
            return false;
        }
        let ch = self.buffer[vowel_index];
        let base_vowel = self.get_base_vowel(ch);
        if lower_char(base_vowel) != 'i' {
            return false;
        }
        let prev_char = self.buffer[vowel_index - 1];
        prev_char == 'g' || prev_char == 'G'
    }

    fn is_nucleus_only_vowel(&self, ch: char) -> bool {
        matches!(
            self.get_base_vowel(ch),
            'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' | 'Ă' | 'Â' | 'Ê' | 'Ô' | 'Ơ' | 'Ư'
        )
    }

    fn for_each_effective_vowel_index<F>(&self, before: usize, mut f: F)
    where
        F: FnMut(usize) -> bool,
    {
        let limit = before.min(self.buffer.len());
        let mut pending_gi_i: Option<usize> = None;

        for index in 0..limit {
            if !is_vowel(self.buffer[index]) {
                continue;
            }

            if pending_gi_i.is_some() {
                pending_gi_i = None;
            }

            if self.is_u_vowel_after_q(index) {
                continue;
            }

            if self.is_gi_i_candidate(index) {
                pending_gi_i = Some(index);
                continue;
            }

            if !f(index) {
                return;
            }
        }

        if let Some(index) = pending_gi_i {
            let _ = f(index);
        }
    }

    fn find_target_vowel_index(&self, before: usize) -> Option<usize> {
        let mut vowel_count = 0usize;
        let mut first_index: usize = 0;
        let mut second_index: usize = 0;
        let mut first_base_lower: char = '\0';
        let mut second_base_lower: char = '\0';
        let mut last_nucleus_only_index: Option<usize> = None;

        self.for_each_effective_vowel_index(before, |index| {
            vowel_count += 1;
            if vowel_count == 1 {
                first_index = index;
                first_base_lower = lower_char(self.get_base_vowel(self.buffer[index]));
            } else if vowel_count == 2 {
                second_index = index;
                second_base_lower = lower_char(self.get_base_vowel(self.buffer[index]));
            }

            if self.is_nucleus_only_vowel(self.buffer[index]) {
                last_nucleus_only_index = Some(index);
            }
            true
        });

        if vowel_count == 0 {
            return None;
        }

        if vowel_count == 1 {
            return Some(first_index);
        }

        if let Some(index) = last_nucleus_only_index {
            return Some(index);
        }

        if vowel_count == 2 {
            if self.tone_placement == TonePlacement::NucleusOnly {
                // Nucleus-only overrides for the vowel clusters where the orthographic rules
                // may place tone on a glide-like vowel ("oa", "oe", "uy").
                if (first_base_lower == 'u' && second_base_lower == 'y')
                    || (first_base_lower == 'o'
                        && (second_base_lower == 'a' || second_base_lower == 'e'))
                {
                    return Some(second_index);
                }
            }

            let has_final_consonant = second_index + 1 < before;

            if has_final_consonant {
                return Some(second_index);
            }

            return Some(first_index);
        }

        let target_position = (vowel_count - 1) / 2;
        let mut position = 0usize;
        let mut target_index: Option<usize> = None;
        self.for_each_effective_vowel_index(before, |index| {
            if position == target_position {
                target_index = Some(index);
                return false;
            }
            position += 1;
            true
        });
        target_index
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

    fn apply_tone_mark_internal(
        &mut self,
        tone_key: char,
        store_last_key: char,
    ) -> Option<KeyTransformAction> {
        if self.buffer.is_empty() {
            return None;
        }

        let trigger_index = self.buffer.len() - 1;
        let vowel_index = self.find_target_vowel_index(trigger_index)?;
        let vowel = self.buffer[vowel_index];

        let mut start_index = vowel_index;
        if let Some(earliest) = self.clear_other_tones(vowel_index, trigger_index) {
            if earliest < start_index {
                start_index = earliest;
            }
        }

        if tone_key == 'z' {
            let base_vowel = self.get_base_vowel(vowel);
            let mut changed = false;
            if base_vowel != vowel {
                self.buffer[vowel_index] = base_vowel;
                changed = true;
            }
            if start_index != vowel_index {
                changed = true;
            }
            if !changed {
                return None;
            }

            self.buffer.pop();
            self.last_transform_key = Some(store_last_key);
            self.last_w_transform_kind = WTransformKind::None;
            let delete_count = trigger_index - start_index;
            let output_text = self.buffer_string_from(start_index);
            return Some(KeyTransformAction {
                delete_count,
                text: output_text,
            });
        }

        let base_vowel = self.get_base_vowel(vowel);
        let tone_map = VOWEL_TO_TONED.get(&base_vowel)?;
        let toned_vowel = tone_map.get(&tone_key)?;

        self.buffer[vowel_index] = *toned_vowel;
        self.buffer.pop();
        self.last_transform_key = Some(store_last_key);
        self.last_w_transform_kind = WTransformKind::None;

        let delete_count = trigger_index - start_index;
        let output_text = self.buffer_string_from(start_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn try_escape_compound_horn_key(
        &mut self,
        key_to_push: char,
        suppressed_key: char,
    ) -> Option<KeyTransformAction> {
        match self.last_w_transform_kind {
            WTransformKind::CompoundUow => {
                let end_index = self.buffer.len();
                if self.buffer.len() < 2 {
                    return None;
                }

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
                    self.buffer.push(key_to_push);
                    self.clear_last_transform_and_suppress(suppressed_key);
                    return Some(KeyTransformAction {
                        delete_count: 2,
                        text: format!("{}{}{}", original_u, original_o, key_to_push),
                    });
                }
            }
            WTransformKind::CompoundUoFinalConsonantW => {
                if self.buffer.len() < 3 {
                    return None;
                }

                let mut o_index = self.buffer.len();
                while o_index > 0 {
                    o_index -= 1;
                    if is_vowel(self.buffer[o_index]) {
                        break;
                    }
                }

                if o_index >= self.buffer.len() || !is_vowel(self.buffer[o_index]) {
                    return None;
                }
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
                    self.buffer.push(key_to_push);
                    self.clear_last_transform_and_suppress(suppressed_key);
                    let output_text = self.buffer_string_from(u_index);
                    return Some(KeyTransformAction {
                        delete_count,
                        text: output_text,
                    });
                }
            }
            WTransformKind::CompoundUaw => {
                let end_index = self.buffer.len();
                if self.buffer.len() < 2 {
                    return None;
                }

                let a_index = end_index - 1;
                let u_index = end_index - 2;
                let u_horn = self.buffer[u_index];
                let a_char = self.buffer[a_index];
                if (u_horn == 'ư' || u_horn == 'Ư') && (a_char == 'a' || a_char == 'A') {
                    let original_u = if u_horn.is_uppercase() { 'U' } else { 'u' };
                    self.buffer.drain(u_index..);
                    self.buffer.push(original_u);
                    self.buffer.push(a_char);
                    self.buffer.push(key_to_push);
                    self.clear_last_transform_and_suppress(suppressed_key);
                    return Some(KeyTransformAction {
                        delete_count: 2,
                        text: format!("{}{}{}", original_u, a_char, key_to_push),
                    });
                }
            }
            _ => {}
        }

        None
    }

    fn try_escape_repeated_tone_key(
        &mut self,
        key_to_push: char,
        internal_tone_key: char,
        suppressed_key: char,
    ) -> Option<KeyTransformAction> {
        let toned_index = self.find_last_toned_vowel_index()?;
        let (base_vowel, last_tone_key) = TONED_TO_BASE.get(&self.buffer[toned_index])?;
        if lower_char(*last_tone_key) != internal_tone_key {
            return None;
        }

        let delete_count = self.buffer.len() - toned_index;
        self.buffer[toned_index] = *base_vowel;
        self.buffer.push(key_to_push);
        self.clear_last_transform_and_suppress(suppressed_key);
        let output_text = self.buffer_string_from(toned_index);
        Some(KeyTransformAction {
            delete_count,
            text: output_text,
        })
    }

    fn clear_other_tones(&mut self, except_index: usize, before: usize) -> Option<usize> {
        let mut earliest: Option<usize> = None;
        let limit = before.min(self.buffer.len());
        for idx in 0..limit {
            if idx == except_index {
                continue;
            }
            let ch = self.buffer[idx];
            if let Some((base, _)) = TONED_TO_BASE.get(&ch) {
                if self.buffer[idx] != *base {
                    self.buffer[idx] = *base;
                    if earliest.map_or(true, |current| idx < current) {
                        earliest = Some(idx);
                    }
                }
            }
        }
        earliest
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

    fn reset_current_word(&mut self) {
        self.buffer.clear();
        self.raw_buffer.clear();
        self.clear_transform_state();
        self.is_foreign_mode = false;
        self.transforms_locked = false;
    }

    pub(crate) fn reset(&mut self) {
        self.reset_current_word();
        self.history.clear();
    }

    fn commit_current_word_to_history_if_needed(&mut self) {
        if self.buffer.is_empty() {
            // If there's no visible output for the current word, there's nothing meaningful to restore later.
            // This preserves legacy behavior for edge-cases where raw keystrokes might exist but output doesn't.
            self.raw_buffer.clear();
            self.is_foreign_mode = false;
            self.transforms_locked = false;
            return;
        }

        let buffer = std::mem::take(&mut self.buffer);
        let raw_buffer = std::mem::take(&mut self.raw_buffer);
        let is_foreign_mode = self.is_foreign_mode;
        let transforms_locked = self.transforms_locked;
        self.history.push_back(HistorySegment::Word(WordSegment {
            buffer,
            raw_buffer,
            is_foreign_mode,
            transforms_locked,
        }));
        self.is_foreign_mode = false;
        self.transforms_locked = false;

        self.trim_history_to_word_limit();
    }

    fn push_boundary_to_history(&mut self, ch: char) {
        match self.history.back_mut() {
            Some(HistorySegment::Boundary(chars)) => chars.push(ch),
            _ => self.history.push_back(HistorySegment::Boundary(vec![ch])),
        }
        self.trim_history_to_word_limit();
    }

    fn trim_history_to_word_limit(&mut self) {
        let mut word_count = self
            .history
            .iter()
            .filter(|seg| matches!(seg, HistorySegment::Word(_)))
            .count();

        while word_count > HISTORY_WORD_LIMIT {
            match self.history.pop_front() {
                Some(HistorySegment::Word(_)) => word_count -= 1,
                Some(HistorySegment::Boundary(_)) => {}
                None => break,
            }
        }

        // Avoid keeping dangling leading separators that belong to dropped words.
        while matches!(self.history.front(), Some(HistorySegment::Boundary(_))) {
            self.history.pop_front();
        }
    }

    fn restore_last_word_from_history(&mut self) -> bool {
        match self.history.pop_back() {
            Some(HistorySegment::Word(word)) => {
                self.buffer = word.buffer;
                self.raw_buffer = word.raw_buffer;
                self.is_foreign_mode = word.is_foreign_mode;
                self.transforms_locked = word.transforms_locked;
                self.clear_transform_state();
                true
            }
            Some(HistorySegment::Boundary(chars)) => {
                self.history.push_back(HistorySegment::Boundary(chars));
                false
            }
            None => false,
        }
    }

    pub(crate) fn delete_last_character(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            self.raw_buffer.pop();
            self.clear_transform_state();
            self.is_foreign_mode = self.has_multiple_vowel_clusters(self.buffer.len());
            if self.buffer.is_empty() {
                self.transforms_locked = false;
            }
            return;
        }

        // If we're not currently composing a word, we may be right after boundary characters.
        match self.history.back_mut() {
            Some(HistorySegment::Boundary(chars)) => {
                chars.pop();
                if chars.is_empty() {
                    self.history.pop_back();
                }
                self.clear_transform_state();
                self.is_foreign_mode = false;
                self.transforms_locked = false;

                // If we just deleted the last boundary, we're now at the end of the previous word.
                if self.buffer.is_empty()
                    && matches!(self.history.back(), Some(HistorySegment::Word(_)))
                {
                    self.restore_last_word_from_history();
                }
            }
            Some(HistorySegment::Word(_)) => {
                // Cursor is at the end of a previously committed word (no trailing boundary).
                if self.restore_last_word_from_history() {
                    if !self.buffer.is_empty() {
                        self.buffer.pop();
                        self.raw_buffer.pop();
                        self.clear_transform_state();
                        self.is_foreign_mode = self.has_multiple_vowel_clusters(self.buffer.len());
                        if self.buffer.is_empty() {
                            self.transforms_locked = false;
                        }
                    }
                }
            }
            None => {}
        }
    }
}

// ==================== Helper Functions ====================

fn is_word_boundary(ch: char, input_method: InputMethod) -> bool {
    match input_method {
        InputMethod::Telex => is_telex_word_boundary(ch),
        InputMethod::Vni => is_vni_word_boundary(ch),
    }
}

#[cfg(test)]
mod tests;
