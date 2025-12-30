mod common;
mod telex;
mod vni;

use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use unicode_normalization::UnicodeNormalization;

// Re-export crate-visible types from common (keep public API surface minimal)
pub(crate) use common::{InputMethod, KeyTransformAction, OutputEncoding, WTransformKind};

// Use internal items from common
use common::{
    is_vowel, lower_char, BASE_VOWELS, NUCLEUS_ONLY_VOWELS, TONED_TO_BASE, VOWEL_TO_TONED,
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
    last_transform_key: Option<char>,
    last_w_transform_kind: WTransformKind,
    suppressed_transform_key: Option<char>,
    auto_fix_tone: bool,
    output_encoding: OutputEncoding,
    input_method: InputMethod,
}

impl VitypeEngine {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            raw_buffer: Vec::new(),
            history: VecDeque::new(),
            is_foreign_mode: false,
            last_transform_key: None,
            last_w_transform_kind: WTransformKind::None,
            suppressed_transform_key: None,
            auto_fix_tone: true,
            output_encoding: OutputEncoding::Unicode,
            input_method: InputMethod::Telex,
        }
    }

    fn process(&mut self, input: &str) -> Option<KeyTransformAction> {
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

    fn handle_invalid_syllable_if_needed(
        &mut self,
        previous_buffer_count: usize,
    ) -> Option<KeyTransformAction> {
        if !self.has_multiple_vowel_clusters(self.buffer.len()) {
            return None;
        }

        let needs_visible_rewrite = self.buffer != self.raw_buffer;

        self.is_foreign_mode = true;
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = None;

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

    fn reset_current_word(&mut self) {
        self.buffer.clear();
        self.raw_buffer.clear();
        self.last_transform_key = None;
        self.last_w_transform_kind = WTransformKind::None;
        self.suppressed_transform_key = None;
        self.is_foreign_mode = false;
    }

    fn reset(&mut self) {
        self.reset_current_word();
        self.history.clear();
    }

    fn commit_current_word_to_history_if_needed(&mut self) {
        if self.buffer.is_empty() {
            // If there's no visible output for the current word, there's nothing meaningful to restore later.
            // This preserves legacy behavior for edge-cases where raw keystrokes might exist but output doesn't.
            self.raw_buffer.clear();
            self.is_foreign_mode = false;
            return;
        }

        let buffer = std::mem::take(&mut self.buffer);
        let raw_buffer = std::mem::take(&mut self.raw_buffer);
        let is_foreign_mode = self.is_foreign_mode;
        self.history
            .push_back(HistorySegment::Word(WordSegment { buffer, raw_buffer, is_foreign_mode }));
        self.is_foreign_mode = false;

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
                self.last_transform_key = None;
                self.last_w_transform_kind = WTransformKind::None;
                self.suppressed_transform_key = None;
                true
            }
            Some(HistorySegment::Boundary(chars)) => {
                self.history.push_back(HistorySegment::Boundary(chars));
                false
            }
            None => false,
        }
    }

    fn delete_last_character(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            self.raw_buffer.pop();
            self.last_transform_key = None;
            self.last_w_transform_kind = WTransformKind::None;
            self.suppressed_transform_key = None;
            self.is_foreign_mode = self.has_multiple_vowel_clusters(self.buffer.len());
            return;
        }

        // If we're not currently composing a word, we may be right after boundary characters.
        match self.history.back_mut() {
            Some(HistorySegment::Boundary(chars)) => {
                chars.pop();
                if chars.is_empty() {
                    self.history.pop_back();
                }
                self.last_transform_key = None;
                self.last_w_transform_kind = WTransformKind::None;
                self.suppressed_transform_key = None;
                self.is_foreign_mode = false;

                // If we just deleted the last boundary, we're now at the end of the previous word.
                if self.buffer.is_empty() && matches!(self.history.back(), Some(HistorySegment::Word(_))) {
                    self.restore_last_word_from_history();
                }
            }
            Some(HistorySegment::Word(_)) => {
                // Cursor is at the end of a previously committed word (no trailing boundary).
                if self.restore_last_word_from_history() {
                    if !self.buffer.is_empty() {
                        self.buffer.pop();
                        self.raw_buffer.pop();
                        self.last_transform_key = None;
                        self.last_w_transform_kind = WTransformKind::None;
                        self.suppressed_transform_key = None;
                        self.is_foreign_mode = self.has_multiple_vowel_clusters(self.buffer.len());
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

fn convert_to_output_encoding(text: String, encoding: OutputEncoding) -> String {
    match encoding {
        OutputEncoding::Unicode => text,
        OutputEncoding::CompositeUnicode => text.nfd().collect(),
    }
}

// ==================== C FFI ====================

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
pub extern "C" fn vitype_engine_set_input_method(engine: *mut VitypeEngine, method: i32) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).input_method = match method {
            1 => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
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

#[cfg(test)]
mod tests;
