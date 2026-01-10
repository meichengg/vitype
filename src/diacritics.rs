use crate::common::{lower_char, TONED_TO_BASE, VOWEL_TO_TONED};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VowelShape {
    Circumflex,
    Horn,
    Breve,
}

pub(crate) fn split_vowel_and_tone(ch: char) -> (char, Option<char>) {
    if let Some((base, tone)) = TONED_TO_BASE.get(&ch) {
        return (*base, Some(*tone));
    }
    (ch, None)
}

pub(crate) fn apply_tone(base: char, tone: Option<char>) -> Option<char> {
    match tone {
        Some(tone_key) => VOWEL_TO_TONED.get(&base)?.get(&tone_key).copied(),
        None => Some(base),
    }
}

pub(crate) fn apply_shape(base: char, kind: VowelShape) -> Option<char> {
    let base_lower = lower_char(base);
    let is_upper = base.is_uppercase();
    let shaped = match kind {
        VowelShape::Circumflex => match base_lower {
            'a' | 'ă' => if is_upper { 'Â' } else { 'â' },
            'e' => if is_upper { 'Ê' } else { 'ê' },
            'o' | 'ơ' => if is_upper { 'Ô' } else { 'ô' },
            _ => return None,
        },
        VowelShape::Horn => match base_lower {
            'o' | 'ô' => if is_upper { 'Ơ' } else { 'ơ' },
            'u' => if is_upper { 'Ư' } else { 'ư' },
            _ => return None,
        },
        VowelShape::Breve => match base_lower {
            'a' | 'â' => if is_upper { 'Ă' } else { 'ă' },
            _ => return None,
        },
    };
    Some(shaped)
}

pub(crate) fn apply_shape_preserving_tone(ch: char, kind: VowelShape) -> Option<char> {
    let (base, tone) = split_vowel_and_tone(ch);
    let shaped_base = apply_shape(base, kind)?;
    apply_tone(shaped_base, tone)
}

pub(crate) fn escape_shape_preserving_tone(ch: char, kind: VowelShape) -> Option<char> {
    let (base, tone) = split_vowel_and_tone(ch);
    let base_lower = lower_char(base);
    let is_upper = base.is_uppercase();
    let unshaped = match kind {
        VowelShape::Circumflex => match base_lower {
            'â' => if is_upper { 'A' } else { 'a' },
            'ê' => if is_upper { 'E' } else { 'e' },
            'ô' => if is_upper { 'O' } else { 'o' },
            _ => return None,
        },
        VowelShape::Horn => match base_lower {
            'ơ' => if is_upper { 'O' } else { 'o' },
            'ư' => if is_upper { 'U' } else { 'u' },
            _ => return None,
        },
        VowelShape::Breve => match base_lower {
            'ă' => if is_upper { 'A' } else { 'a' },
            _ => return None,
        },
    };
    apply_tone(unshaped, tone)
}

pub(crate) fn apply_telex_w_preserving_tone(ch: char) -> Option<char> {
    let (base, tone) = split_vowel_and_tone(ch);
    let base_lower = lower_char(base);
    let is_upper = base.is_uppercase();

    let shaped_base = match base_lower {
        'a' => if is_upper { 'Ă' } else { 'ă' },
        'ă' => {
            if tone.is_some() {
                return None;
            }
            if is_upper { 'Ă' } else { 'ă' }
        }
        'o' | 'ô' => if is_upper { 'Ơ' } else { 'ơ' },
        'u' => if is_upper { 'Ư' } else { 'ư' },
        'ư' => {
            if tone.is_some() {
                return None;
            }
            if is_upper { 'Ư' } else { 'ư' }
        }
        _ => return None,
    };

    apply_tone(shaped_base, tone)
}
