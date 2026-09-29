//! Explicit stress placement and editorial marking. Vowel indices refer to
//! written vowel letters in the supplied representation, not reconstructed
//! historical syllables. No instruction is inferred from a lemma's spelling.
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StressInstruction {
    #[default]
    Unspecified,
    StemVowel(usize),
    SuffixVowel(usize),
    WordVowel(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccentMark {
    Acute,
    Grave,
    Kamora,
}
impl AccentMark {
    fn character(self) -> char {
        match self {
            Self::Acute => '\u{301}',
            Self::Grave => '\u{300}',
            Self::Kamora => '\u{311}',
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AccentPolicy {
    /// Preserve input marks without predicting or supplying a written accent.
    #[default]
    PreserveInput,
    MarkPredicted {
        internal: AccentMark,
        final_vowel: AccentMark,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccentTrace {
    /// A predicted position is separate from marks already present in input.
    pub predicted_vowel: Option<usize>,
    pub supplied_mark: Option<AccentMark>,
    pub accented: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccentError {
    VowelOutOfRange,
    ConflictingInputMarks,
    OverrideWithoutPrediction,
}

pub fn position(
    stem: &str,
    suffix: &str,
    instruction: &StressInstruction,
) -> Result<Option<usize>, AccentError> {
    let count = |s: &str| {
        s.chars()
            .filter(|c| crate::orthography::is_vowel_letter(*c))
            .count()
    };
    let nstem = count(stem);
    let nsuffix = count(suffix);
    let index = match instruction {
        StressInstruction::Unspecified => return Ok(None),
        StressInstruction::StemVowel(i) if *i < nstem => *i,
        StressInstruction::SuffixVowel(i) if *i < nsuffix => nstem + i,
        StressInstruction::WordVowel(i) if *i < nstem + nsuffix => *i,
        _ => return Err(AccentError::VowelOutOfRange),
    };
    if stem
        .nfd()
        .chain(suffix.nfd())
        .any(|c| matches!(c, '\u{300}' | '\u{301}' | '\u{302}' | '\u{311}'))
    {
        return Err(AccentError::ConflictingInputMarks);
    }
    Ok(Some(index))
}

pub fn render(
    word: &str,
    position: Option<usize>,
    policy: &AccentPolicy,
    override_mark: Option<AccentMark>,
) -> Result<AccentTrace, AccentError> {
    let mut trace = AccentTrace {
        predicted_vowel: position,
        supplied_mark: None,
        accented: word.into(),
    };
    let Some(position) = position else {
        if override_mark.is_some() {
            return Err(AccentError::OverrideWithoutPrediction);
        }
        return Ok(trace);
    };
    let (offset, vowel) = word
        .char_indices()
        .filter(|(_, c)| crate::orthography::is_vowel_letter(*c))
        .nth(position)
        .ok_or(AccentError::VowelOutOfRange)?;
    let mut after = offset + vowel.len_utf8();
    // Preserve the existing combining cluster (for example a breathing mark).
    // The supplied stress mark follows it rather than reordering source marks.
    for ch in word[after..]
        .chars()
        .take_while(|c| unicode_normalization::char::canonical_combining_class(*c) != 0)
    {
        after += ch.len_utf8();
    }
    let mark = override_mark.or(match policy {
        AccentPolicy::PreserveInput => None,
        AccentPolicy::MarkPredicted {
            internal,
            final_vowel,
        } => Some(if after == word.len() {
            *final_vowel
        } else {
            *internal
        }),
    });
    if let Some(mark) = mark {
        trace.accented.insert(after, mark.character());
        trace.supplied_mark = Some(mark);
    }
    Ok(trace)
}
