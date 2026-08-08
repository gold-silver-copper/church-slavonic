//! Lossless display normalization and conservative lookup keys.

use crate::InflectionError;
use unicode_normalization::UnicodeNormalization;

pub const MAX_INPUT_CHARS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Cyrillic,
    Glagolitic,
    Latin,
    Mixed,
    Unknown,
}

pub fn canonical_display(input: &str) -> Result<String, InflectionError> {
    validate(input)?;
    Ok(input.nfc().collect())
}

pub fn lookup_key(input: &str) -> Result<String, InflectionError> {
    let normalized = canonical_display(input)?;
    Ok(normalized.to_lowercase())
}

pub fn detect_script(input: &str) -> Script {
    let mut cyrillic = false;
    let mut glagolitic = false;
    let mut latin = false;
    for ch in input.chars().filter(|ch| ch.is_alphabetic()) {
        let cp = u32::from(ch);
        if (0x0400..=0x052f).contains(&cp)
            || (0x2de0..=0x2dff).contains(&cp)
            || (0xa640..=0xa69f).contains(&cp)
        {
            cyrillic = true;
        } else if (0x2c00..=0x2c5f).contains(&cp) {
            glagolitic = true;
        } else if ch.is_ascii_alphabetic() || (0x00c0..=0x024f).contains(&cp) {
            latin = true;
        }
    }
    match (cyrillic, glagolitic, latin) {
        (true, false, false) => Script::Cyrillic,
        (false, true, false) => Script::Glagolitic,
        (false, false, true) => Script::Latin,
        (false, false, false) => Script::Unknown,
        _ => Script::Mixed,
    }
}

fn validate(input: &str) -> Result<(), InflectionError> {
    if input.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "the lemma is empty".to_string(),
        });
    }
    if input.chars().count() > MAX_INPUT_CHARS {
        return Err(InflectionError::InvalidInput {
            reason: format!("the lemma exceeds {MAX_INPUT_CHARS} Unicode scalar values"),
        });
    }
    if input.chars().any(char::is_control) {
        return Err(InflectionError::InvalidInput {
            reason: "control characters are not allowed".to_string(),
        });
    }
    if input.chars().any(char::is_whitespace) {
        return Err(InflectionError::InvalidInput {
            reason: "the word-level API does not accept whitespace".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn scripts_are_distinguished_without_transliteration() {
        assert_eq!(detect_script("слово"), Script::Cyrillic);
        assert_eq!(detect_script("ⱄⰾⱁⰲⱁ"), Script::Glagolitic);
        assert_eq!(detect_script("slovo"), Script::Latin);
    }

    #[test]
    fn hostile_inputs_are_typed_errors() {
        assert!(lookup_key("").is_err());
        assert!(lookup_key("два слова").is_err());
        assert!(lookup_key("слово\0").is_err());
        assert!(lookup_key(&"x".repeat(MAX_INPUT_CHARS + 1)).is_err());
        assert_eq!(lookup_key(".").expect("punctuation is lossless"), ".");
        assert_eq!(
            lookup_key("LATIN").expect("non-OCS is not guessed"),
            "latin"
        );
    }
}
