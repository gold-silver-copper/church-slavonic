use unicode_normalization::{UnicodeNormalization, char::canonical_combining_class};

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum OrthographyProfile {
    #[default]
    Expanded,
    ExpandedAccentless,
    SynodalLiturgical,
}

impl OrthographyProfile {
    pub const ALL: [Self; 3] = [
        Self::Expanded,
        Self::ExpandedAccentless,
        Self::SynodalLiturgical,
    ];
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct SynodalWord {
    source: String,
    canonical: String,
}

impl SynodalWord {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let source = value.into();
        validate_word(&source)?;
        let canonical = source.nfc().collect();
        Ok(Self { source, canonical })
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    #[must_use]
    pub fn lookup_key(&self) -> String {
        normalize_lookup(&self.canonical)
    }
}

impl TryFrom<String> for SynodalWord {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(value)
    }
}

impl From<SynodalWord> for String {
    fn from(value: SynodalWord) -> Self {
        value.canonical
    }
}

impl AsRef<str> for SynodalWord {
    fn as_ref(&self) -> &str {
        self.canonical()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct RenderedText(String);

impl RenderedText {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(Error::EmptyInput);
        }
        for (byte_index, character) in value.char_indices() {
            if is_private_use(character) || character.is_control() {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "controls and private-use characters are forbidden".into(),
                });
            }
            if character.is_alphabetic() && !is_cyrillic(character) {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "rendered Church Slavonic text cannot contain another script".into(),
                });
            }
        }
        Ok(Self(value.nfc().collect()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RenderedText {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(value)
    }
}

impl From<RenderedText> for String {
    fn from(value: RenderedText) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Loss {
    pub kind: String,
    pub original: String,
    pub replacement: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NormalizationReport {
    pub original: String,
    pub normalized: String,
    pub losses: Vec<Loss>,
}

/// Explicit caller-supplied positional-letter decision. These choices are
/// never inferred from spelling alone because Alypy §2 lists lexical and
/// grammatical exceptions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum InitialPresentation {
    Preserve,
    WideE,
    BroadOn,
    IotatedYa,
    DigraphUk,
}

impl InitialPresentation {
    pub const ALL: [Self; 5] = [
        Self::Preserve,
        Self::WideE,
        Self::BroadOn,
        Self::IotatedYa,
        Self::DigraphUk,
    ];
}

/// Applies one reviewed positional-letter decision and reports the change.
/// This is deliberately explicit: lexical semantics decide exceptions such as
/// the two spellings of `ꙗзыкъ`/`ѧзыкъ`, not a blind string rewrite.
pub fn apply_initial_presentation(
    word: &SynodalWord,
    presentation: InitialPresentation,
) -> Result<NormalizationReport> {
    if presentation == InitialPresentation::Preserve {
        return Ok(NormalizationReport {
            original: word.canonical().into(),
            normalized: word.canonical().into(),
            losses: Vec::new(),
        });
    }
    let mut characters = word.canonical().chars();
    let first = characters.next().ok_or(Error::EmptyInput)?;
    let replacement = match (presentation, first) {
        (InitialPresentation::WideE, 'е') => "є",
        (InitialPresentation::BroadOn, 'о') => "ѻ",
        (InitialPresentation::IotatedYa, 'ѧ') => "ꙗ",
        (InitialPresentation::DigraphUk, 'ꙋ' | 'у') => "ᲂу",
        _ => {
            return Err(Error::InvalidOrthography {
                reason: format!("{presentation:?} is incompatible with initial letter {first:?}"),
            });
        }
    };
    let mut normalized = String::from(replacement);
    normalized.extend(characters);
    let normalized = SynodalWord::parse(normalized)?.canonical().to_owned();
    Ok(NormalizationReport {
        original: word.canonical().into(),
        normalized,
        losses: vec![Loss {
            kind: "explicit-positional-presentation".into(),
            original: first.to_string(),
            replacement: replacement.into(),
        }],
    })
}

#[must_use]
pub fn normalize_lookup(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).nfc().collect()
}

/// Produces the explicit accent-insensitive lookup projection. Historical
/// letters remain distinct; only presentation accents and breathing are removed.
#[must_use]
pub fn normalize_lookup_accentless(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .nfd()
        .filter(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
            )
        })
        .nfc()
        .collect()
}

fn validate_word(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyInput);
    }

    let mut cluster_has_base = false;
    let mut saw_accent = false;
    let mut previous_ccc = 0;
    for (byte_index, character) in value.char_indices() {
        if is_private_use(character) || character.is_control() {
            return Err(Error::InvalidUnicode {
                byte_index,
                character,
                reason: "controls and private-use characters are forbidden".into(),
            });
        }

        let ccc = canonical_combining_class(character);
        if ccc == 0 && character != '\u{034f}' {
            if !is_cyrillic(character) {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "a Synodal word admits only standard Cyrillic letters and marks".into(),
                });
            }
            cluster_has_base = true;
            saw_accent = false;
            previous_ccc = 0;
            continue;
        }

        if !cluster_has_base {
            return Err(Error::InvalidOrthography {
                reason: "a combining mark cannot precede its base letter".into(),
            });
        }
        if !is_permitted_mark(character) {
            return Err(Error::InvalidUnicode {
                byte_index,
                character,
                reason: "combining mark is outside the Church Slavonic repertoire".into(),
            });
        }
        if character == '\u{0486}' && saw_accent {
            return Err(Error::InvalidOrthography {
                reason: "Church Slavonic breathing U+0486 must precede the accent".into(),
            });
        }
        if is_accent(character) && saw_accent {
            return Err(Error::InvalidOrthography {
                reason: "a letter cluster cannot carry more than one accent mark".into(),
            });
        }
        if is_accent(character) {
            saw_accent = true;
        }
        if ccc != 0 && previous_ccc > ccc {
            return Err(Error::InvalidOrthography {
                reason: "combining marks are not in canonical order".into(),
            });
        }
        if ccc != 0 {
            previous_ccc = ccc;
        }
    }
    Ok(())
}

fn is_accent(character: char) -> bool {
    matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}')
}

fn is_permitted_mark(character: char) -> bool {
    matches!(
        character as u32,
        0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
    )
}

fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f
            | 0x1c80..=0x1c8f
            | 0x2de0..=0x2dff
            | 0xa640..=0xa69f
            | 0x1e030..=0x1e08f
    )
}

fn is_private_use(character: char) -> bool {
    matches!(
        character as u32,
        0xe000..=0xf8ff | 0xf0000..=0xffffd | 0x100000..=0x10fffd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_historical_letters_and_marks() {
        let word = SynodalWord::parse("сло\u{0486}\u{0301}во").expect("valid word");
        assert_eq!(word.lookup_key(), "сло\u{0486}\u{0301}во");
    }

    #[test]
    fn rejects_accent_before_breathing() {
        let error = SynodalWord::parse("о\u{0301}\u{0486}").expect_err("invalid order");
        assert!(matches!(error, Error::InvalidOrthography { .. }));
    }

    #[test]
    fn rejects_multiple_accents_on_one_letter_cluster() {
        let error = SynodalWord::parse("а\u{0301}\u{0301}").expect_err("duplicate accents");
        assert!(matches!(error, Error::InvalidOrthography { .. }));
    }

    #[test]
    fn rejects_private_use_and_other_scripts() {
        assert!(SynodalWord::parse("сло\u{e000}во").is_err());
        assert!(SynodalWord::parse("slovo").is_err());
        assert!(SynodalWord::parse("слovo").is_err());
    }

    #[test]
    fn supports_standard_titlo_superscripts_payerok_and_kavyka() {
        for value in ["бг҃ъ", "б\u{2de1}\u{0487}", "слоꙿво", "сло꙾во"] {
            SynodalWord::parse(value).expect("standard encoded Church Slavonic spelling");
        }
    }

    #[test]
    fn positional_rendering_requires_an_explicit_compatible_choice() {
        let word = SynodalWord::parse("его").expect("expanded word");
        let report = apply_initial_presentation(&word, InitialPresentation::WideE)
            .expect("compatible presentation");
        assert_eq!(report.normalized, "єго");
        assert_eq!(report.losses.len(), 1);
        assert!(apply_initial_presentation(&word, InitialPresentation::BroadOn).is_err());
    }
}
