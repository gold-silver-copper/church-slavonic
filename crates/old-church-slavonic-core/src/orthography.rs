//! Lossless display normalization and conservative lookup keys.

use crate::InflectionError;
use core::{fmt, ops::Deref};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

pub const MAX_INPUT_CHARS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Script {
    Cyrillic,
    Glagolitic,
    Latin,
    Mixed,
    Unknown,
}

/// A normalized, single-script Old Church Slavonic dictionary lemma.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lemma {
    text: String,
    script: Script,
}

impl Lemma {
    /// Parse, NFC-normalize, and validate one Cyrillic or Glagolitic lemma.
    pub fn parse(input: &str) -> Result<Self, InflectionError> {
        let text = canonical_display(input).map_err(|error| match error {
            InflectionError::InvalidInput { reason } => {
                InflectionError::invalid_lemma(input, reason)
            }
            other => other,
        })?;
        let mut has_base = false;
        for ch in text.chars() {
            if ch.is_alphabetic() {
                has_base = true;
            } else if is_combining_mark(ch) {
                if !has_base {
                    return Err(InflectionError::invalid_lemma(
                        input,
                        "a combining mark must follow a lemma letter",
                    ));
                }
            } else {
                return Err(InflectionError::invalid_lemma(
                    input,
                    format!("the lemma contains a non-letter character {ch:?}"),
                ));
            }
        }
        let script = detect_script(&text);
        match script {
            Script::Cyrillic | Script::Glagolitic => Ok(Self { text, script }),
            Script::Mixed => Err(InflectionError::invalid_lemma(
                input,
                "the lemma mixes Cyrillic, Glagolitic, Latin, or another script",
            )),
            Script::Latin => Err(InflectionError::invalid_lemma(
                input,
                "the lemma is Latin; expected Old Church Slavonic Cyrillic or Glagolitic",
            )),
            Script::Unknown => Err(InflectionError::invalid_lemma(
                input,
                "the lemma has no Cyrillic or Glagolitic letters",
            )),
        }
    }

    /// The normalized spelling.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The lemma's single validated script.
    pub fn script(&self) -> Script {
        self.script
    }
}

impl AsRef<str> for Lemma {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Lemma {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for Lemma {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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

    #[test]
    fn lemma_is_normalized_single_script_and_letters_only() {
        let lemma = Lemma::parse("И\u{306}").expect("decomposed Cyrillic lemma");
        assert_eq!(lemma.as_str(), "Й");
        assert_eq!(lemma.script(), Script::Cyrillic);
        assert_eq!(
            Lemma::parse("ⱄⰾⱁⰲⱁ").expect("Glagolitic").script(),
            Script::Glagolitic
        );
        for invalid in [
            "слоword",
            "слоα",
            "слово.",
            "<слово>",
            "\u{301}слово",
            "latin",
        ] {
            assert!(Lemma::parse(invalid).is_err(), "accepted {invalid:?}");
        }
    }
}
