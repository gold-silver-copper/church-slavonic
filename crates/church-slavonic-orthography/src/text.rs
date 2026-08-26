//! Recension-agnostic text primitives: single-word shape validation, NFC
//! canonicalization, and script classification. Both the Glagolitic and the
//! Synodal modules build on these; nothing here knows about either recension's
//! letter repertoire beyond Unicode block identity.

use core::fmt;
use unicode_normalization::UnicodeNormalization;

/// Upper bound on the Unicode scalar count of one word-level input.
pub const MAX_INPUT_CHARS: usize = 4_096;

/// The script of a validated word, classified by Unicode block membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Script {
    Cyrillic,
    Glagolitic,
    Latin,
    Mixed,
    Unknown,
}

/// A word failed the recension-agnostic single-word shape rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidWord {
    pub reason: String,
}

impl fmt::Display for InvalidWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid input: {}", self.reason)
    }
}

impl std::error::Error for InvalidWord {}

fn invalid(reason: impl Into<String>) -> InvalidWord {
    InvalidWord {
        reason: reason.into(),
    }
}

/// Validates the recension-agnostic word shape: nonempty, bounded, and free of
/// control characters and whitespace.
pub fn validate_single_word(input: &str) -> Result<(), InvalidWord> {
    if input.is_empty() {
        return Err(invalid("the lemma is empty"));
    }
    if input.chars().count() > MAX_INPUT_CHARS {
        return Err(invalid(format!(
            "the lemma exceeds {MAX_INPUT_CHARS} Unicode scalar values"
        )));
    }
    if input.chars().any(char::is_control) {
        return Err(invalid("control characters are not allowed"));
    }
    if input.chars().any(char::is_whitespace) {
        return Err(invalid("the word-level API does not accept whitespace"));
    }
    Ok(())
}

/// Validates the single-word shape and returns the NFC-canonical spelling.
pub fn canonical_display(input: &str) -> Result<String, InvalidWord> {
    validate_single_word(input)?;
    Ok(input.nfc().collect())
}

/// The conservative case-folded lookup projection of a validated word: NFC
/// plus lowercase, never stripping combining marks.
pub fn lookup_key(input: &str) -> Result<String, InvalidWord> {
    let normalized = canonical_display(input)?;
    Ok(normalized.to_lowercase())
}

/// Classifies the alphabetic content of a word without transliterating it.
pub fn detect_script(input: &str) -> Script {
    let mut cyrillic = false;
    let mut glagolitic = false;
    let mut latin = false;
    let mut other = false;
    for ch in input.chars().filter(|ch| ch.is_alphabetic()) {
        let cp = u32::from(ch);
        if (0x0400..=0x052f).contains(&cp)
            || (0x2de0..=0x2dff).contains(&cp)
            || (0xa640..=0xa69f).contains(&cp)
        {
            cyrillic = true;
        } else if (0x2c00..=0x2c5f).contains(&cp) || (0x1e000..=0x1e02f).contains(&cp) {
            glagolitic = true;
        } else if ch.is_ascii_alphabetic() || (0x00c0..=0x024f).contains(&cp) {
            latin = true;
        } else {
            other = true;
        }
    }
    match (cyrillic, glagolitic, latin, other) {
        (true, false, false, false) => Script::Cyrillic,
        (false, true, false, false) => Script::Glagolitic,
        (false, false, true, false) => Script::Latin,
        (false, false, false, _) => Script::Unknown,
        _ => Script::Mixed,
    }
}

/// Unicode private-use scalar values are never valid Church Slavonic text.
pub fn is_private_use(character: char) -> bool {
    matches!(
        character as u32,
        0xe000..=0xf8ff | 0xf0000..=0xffffd | 0x100000..=0x10fffd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_are_distinguished_without_transliteration() {
        assert_eq!(detect_script("слово"), Script::Cyrillic);
        assert_eq!(detect_script("ⱄⰾⱁⰲⱁ"), Script::Glagolitic);
        assert_eq!(detect_script("slovo"), Script::Latin);
        assert_eq!(detect_script("слword"), Script::Mixed);
        assert_eq!(detect_script("."), Script::Unknown);
    }

    #[test]
    fn lookup_is_nfc_and_lowercase_without_stripping_marks() {
        let decomposed = "А\u{301}ЗЪ";
        assert_eq!(lookup_key(decomposed).expect("valid OCS"), "а\u{301}зъ");
        assert_eq!(lookup_key("цар҄ь").expect("valid OCS"), "цар҄ь");
        assert_eq!(lookup_key("И\u{306}").expect("decomposed breve"), "й");
        assert_eq!(
            lookup_key("а\u{315}\u{301}").expect("valid combining marks"),
            lookup_key("а\u{301}\u{315}").expect("canonical mark order")
        );
        assert_eq!(lookup_key(".").expect("punctuation is lossless"), ".");
        assert_eq!(
            lookup_key("LATIN").expect("non-OCS is not guessed"),
            "latin"
        );
    }

    #[test]
    fn hostile_word_shapes_are_typed_errors() {
        assert!(canonical_display("").is_err());
        assert!(canonical_display("два слова").is_err());
        assert!(canonical_display("слово\0").is_err());
        assert!(canonical_display(&"x".repeat(MAX_INPUT_CHARS + 1)).is_err());
        assert_eq!(
            canonical_display("И\u{306}").expect("decomposed breve"),
            "Й"
        );
    }
}
