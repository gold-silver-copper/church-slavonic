//! The Synodal liturgical orthography engine, moved from
//! `synodal-church-slavonic-core::orthography` (docs/REWRITE_PLAN.md, target
//! layout): lookup normalization (uk digraph/monograph, broad on, wide e
//! folds), word validation over the Synodal Cyrillic repertoire, and the
//! positional/initial presentation operations. Cell-scoped paradigm types
//! that carry family evidence (`PositionalParadigm`, `PositionalRule`) stay
//! in the family core as thin adapters over these operations.

use church_slavonic_core::grammar::{Case, Number};
use core::fmt;
use unicode_normalization::{UnicodeNormalization, char::canonical_combining_class};

use crate::text::is_private_use;

/// A typed orthographic failure. The family core maps these onto its own
/// `Error` variants one-for-one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SynodalOrthographyError {
    EmptyInput,
    InvalidUnicode {
        byte_index: usize,
        character: char,
        reason: String,
    },
    InvalidOrthography {
        reason: String,
    },
    ContradictoryMetadata {
        reason: String,
    },
}

impl fmt::Display for SynodalOrthographyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "the input is empty"),
            Self::InvalidUnicode {
                byte_index,
                character,
                reason,
            } => write!(
                f,
                "invalid Unicode {character:?} at byte {byte_index}: {reason}"
            ),
            Self::InvalidOrthography { reason } => write!(f, "invalid orthography: {reason}"),
            Self::ContradictoryMetadata { reason } => {
                write!(f, "contradictory metadata: {reason}")
            }
        }
    }
}

impl std::error::Error for SynodalOrthographyError {}

type Result<T> = std::result::Result<T, SynodalOrthographyError>;

use SynodalOrthographyError as Error;

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

/// One closed positional-letter substitution licensed by source-specific
/// Synodal positional and number-antistich rules.
/// The expected input letter is part of the variant, so a rule cannot silently
/// rewrite an unrelated character.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PositionalReplacement {
    WideE,
    BroadOn,
    Omega,
    DecimalI,
    IotatedYa,
    Yeri,
    LittleYus,
}

impl PositionalReplacement {
    pub const ALL: [Self; 7] = [
        Self::WideE,
        Self::BroadOn,
        Self::Omega,
        Self::DecimalI,
        Self::IotatedYa,
        Self::Yeri,
        Self::LittleYus,
    ];

    const fn letters(self) -> (char, char) {
        match self {
            Self::WideE => ('е', 'є'),
            Self::BroadOn => ('о', 'ѻ'),
            Self::Omega => ('о', 'ѡ'),
            Self::DecimalI => ('и', 'ї'),
            Self::IotatedYa => ('ѧ', 'ꙗ'),
            Self::Yeri => ('и', 'ы'),
            Self::LittleYus => ('а', 'ѧ'),
        }
    }
}

/// Zero-based occurrence of the input letter selected by a positional rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LetterOccurrence {
    FromStart(u8),
    FromEnd(u8),
}

/// Explicit operations for one lexical/cell-conditioned printed spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PositionalOperation {
    Initial(InitialPresentation),
    DecimalIBeforeVowel,
    /// Applies Alypy §36's `-ѡвъ/-євъ` and `-ѡмъ/-ємъ` spelling when the
    /// selected source/class treats that general rule as controlling. Some
    /// reviewed noun tables retain ordinary vowels, so this is explicit.
    WidePluralEnding,
    Replace {
        replacement: PositionalReplacement,
        occurrence: LetterOccurrence,
    },
}

/// Applies the selected §36 wide-ending rule. Selection remains explicit
/// because reviewed source tables do not realize the generalization uniformly.
/// `nominal` is the requested cell's number and case, or `None` for a
/// non-nominal cell (which is always preserved).
pub fn apply_wide_plural_ending(
    nominal: Option<(Number, Case)>,
    expanded: &str,
) -> Result<String> {
    if contains_prosodic_mark(expanded) {
        return Err(Error::ContradictoryMetadata {
            reason: "grammatical positional presentation requires an unaccented expanded form"
                .into(),
        });
    }
    let Some((number, case)) = nominal else {
        return Ok(expanded.to_owned());
    };
    if number != Number::Plural {
        return Ok(expanded.to_owned());
    }
    let suffixes = match case {
        Case::Genitive => [("овъ", "ѡвъ"), ("евъ", "євъ")],
        Case::Dative => [("омъ", "ѡмъ"), ("емъ", "ємъ")],
        _ => return Ok(expanded.to_owned()),
    };
    for (ordinary, wide) in suffixes {
        if let Some(stem) = expanded.strip_suffix(ordinary) {
            return canonicalize(format!("{stem}{wide}"));
        }
    }
    Ok(expanded.to_owned())
}

/// Validates one spelling and returns its NFC-canonical presentation — the
/// same projection `SynodalWord::parse(..).canonical()` produces in the
/// family core.
pub fn canonicalize(value: String) -> Result<String> {
    validate_word(&value)?;
    Ok(value.nfc().collect())
}

pub fn decimal_i_before_vowel(value: &str) -> Result<String> {
    let characters = value.char_indices().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    for (index, (_, character)) in characters.iter().copied().enumerate() {
        if character == 'и'
            && characters
                .iter()
                .skip(index + 1)
                .find(|(_, next)| canonical_combining_class(*next) == 0 && *next != '\u{034f}')
                .is_some_and(|(_, next)| is_synodal_vowel(*next))
        {
            output.push('ї');
        } else {
            output.push(character);
        }
    }
    canonicalize(output)
}



pub fn replace_occurrence(
    value: &str,
    replacement: PositionalReplacement,
    occurrence: LetterOccurrence,
) -> Result<String> {
    let (input, output) = replacement.letters();
    let matches = value
        .char_indices()
        .filter_map(|(index, character)| (character == input).then_some(index))
        .collect::<Vec<_>>();
    let selected = match occurrence {
        LetterOccurrence::FromStart(offset) => matches.get(usize::from(offset)).copied(),
        LetterOccurrence::FromEnd(offset) => matches
            .len()
            .checked_sub(usize::from(offset) + 1)
            .and_then(|index| matches.get(index).copied()),
    }
    .ok_or_else(|| Error::ContradictoryMetadata {
        reason: format!(
            "positional replacement {replacement:?} at {occurrence:?} is outside form {value:?}"
        ),
    })?;
    let mut replaced = String::with_capacity(value.len());
    for (index, character) in value.char_indices() {
        replaced.push(if index == selected { output } else { character });
    }
    canonicalize(replaced)
}



/// Whether a spelling carries any Synodal prosodic mark (accents, oxia,
/// varia, kamora, or breathing).
pub fn contains_prosodic_mark(value: &str) -> bool {
    value.nfd().any(|character| {
        matches!(
            character,
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0485}' | '\u{0486}'
        )
    })
}



/// The Synodal vowel-letter repertoire, case-insensitively.
pub fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѻ'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
            | 'ѥ'
            | 'ѫ'
            | 'ѭ'
            | 'ѵ'
    )
}



/// Applies one reviewed positional-letter decision and reports the change.
/// This is deliberately explicit: lexical semantics decide exceptions such as
/// the two spellings of `ꙗзыкъ`/`ѧзыкъ`, not a blind string rewrite.
/// `canonical` must be an already validated, NFC-canonical Synodal spelling.
pub fn apply_initial_presentation(
    canonical: &str,
    presentation: InitialPresentation,
) -> Result<NormalizationReport> {
    if presentation == InitialPresentation::Preserve {
        return Ok(NormalizationReport {
            original: canonical.into(),
            normalized: canonical.into(),
            losses: Vec::new(),
        });
    }
    let mut characters = canonical.chars();
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
    let normalized = canonicalize(normalized)?;
    Ok(NormalizationReport {
        original: canonical.into(),
        normalized,
        losses: vec![Loss {
            kind: "explicit-positional-presentation".into(),
            original: first.to_string(),
            replacement: replacement.into(),
        }],
    })
}


/// The Cyrillic letter uk `ѹ` (U+0479, with its capital U+0478) is a
/// single-codepoint presentation of the letter pair `оу`, exactly as the
/// printed word-initial digraph `ᲂу` is. The lookup projections expand it so
/// an `ѹ`-spelled token reaches the same key as its `оу`/`ᲂу` twins; marks
/// that followed the monograph land on the `у`, matching where the digraph
/// carries them.
fn fold_uk_monograph(character: char) -> impl Iterator<Item = char> {
    let (first, second) = match character {
        'ѹ' => ('о', Some('у')),
        other => (other, None),
    };
    core::iter::once(first).chain(second)
}

#[must_use]
pub fn normalize_lookup(value: &str) -> String {
    value
        .chars()
        .map(fold_digraph_uk)
        .flat_map(char::to_lowercase)
        .flat_map(fold_uk_monograph)
        .nfc()
        .collect()
}

/// Renders the expanded word-initial `оу` as the printed Synodal digraph
/// `ᲂу` (U+1C82 followed by `у`), leaving any marks on the `у` in place. A
/// capitalised `Оу` is already the digraph's sentence-initial presentation
/// and is preserved; every other initial is returned unchanged.
#[must_use]
pub fn present_initial_uk_digraph(printed: &str) -> String {
    let mut characters = printed.chars();
    match (characters.next(), characters.clone().next()) {
        (Some('о'), Some('у' | 'ꙋ')) => {
            let mut output = String::with_capacity(printed.len() + 2);
            output.push('\u{1c82}');
            output.extend(characters);
            output
        }
        _ => printed.to_owned(),
    }
}

/// The word-initial `ᲂу` digraph (U+1C82 followed by `у`) is a presentation
/// of the expanded letters `оу`, exactly as the capitalised `Оу` is. The
/// lookup projections fold the modifier-letter half back to `о` so a printed
/// digraph token reaches the same key as its expanded form.
const fn fold_digraph_uk(character: char) -> char {
    match character {
        '\u{1c82}' => 'о',
        // The broad on ѻ (U+047B, capital U+047A) is a word-initial
        // presentation of о, exactly as the digraph half above is a
        // presentation of о: the lookup projections fold it so an ѻ-spelled
        // token reaches the same key as its о twin.
        'ѻ' => 'о',
        'Ѻ' => 'О',
        // The wide е (є, U+0454, capital U+0404) marks position — endings
        // after a vowel and plural-distinguishing cells — over the same
        // letter е, so the lookup projections fold it likewise.
        'є' => 'е',
        'Є' => 'Е',
        other => other,
    }
}

/// Produces the explicit accent-insensitive lookup projection. Historical
/// letters remain distinct; only presentation accents and breathing are removed.
#[must_use]
pub fn normalize_lookup_accentless(value: &str) -> String {
    value
        .chars()
        .map(fold_digraph_uk)
        .flat_map(char::to_lowercase)
        .flat_map(fold_uk_monograph)
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


/// Validates one Synodal spelling: Cyrillic repertoire only, permitted
/// combining marks in canonical order, one accent per cluster, breathing
/// before accent.
pub fn validate_word(value: &str) -> Result<()> {
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

/// The Synodal Cyrillic letter repertoire, including the modifier-letter
/// digraph half U+1C82.
pub fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f
            | 0x1c80..=0x1c8f
            | 0x2de0..=0x2dff
            | 0xa640..=0xa69f
            | 0x1e030..=0x1e08f
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_initial_presentation_is_explicit_and_shape_checked() {
        for (input, presentation, expected) in [
            ("его", InitialPresentation::Preserve, "его"),
            ("его", InitialPresentation::WideE, "єго"),
            ("отецъ", InitialPresentation::BroadOn, "ѻтецъ"),
            ("ѧзыкъ", InitialPresentation::IotatedYa, "ꙗзыкъ"),
            ("ꙋченикъ", InitialPresentation::DigraphUk, "ᲂученикъ"),
        ] {
            assert_eq!(
                apply_initial_presentation(input, presentation)
                    .expect("compatible initial presentation")
                    .normalized,
                expected
            );
        }
        assert!(apply_initial_presentation("его", InitialPresentation::BroadOn).is_err());
    }

    #[test]
    fn wide_plural_endings_apply_only_to_plural_genitive_and_dative() {
        let plural_dative = Some((Number::Plural, Case::Dative));
        let plural_genitive = Some((Number::Plural, Case::Genitive));
        let singular = Some((Number::Singular, Case::Instrumental));
        assert_eq!(
            apply_wide_plural_ending(plural_dative, "рабомъ").expect("wide dative"),
            "рабѡмъ"
        );
        assert_eq!(
            apply_wide_plural_ending(plural_genitive, "сыновъ").expect("wide genitive"),
            "сынѡвъ"
        );
        assert_eq!(
            apply_wide_plural_ending(singular, "рабомъ").expect("preserved singular"),
            "рабомъ"
        );
        assert_eq!(
            apply_wide_plural_ending(None, "рабомъ").expect("preserved non-nominal"),
            "рабомъ"
        );
    }
}

#[cfg(test)]
mod digraph_lookup_tests {
    use super::{normalize_lookup, normalize_lookup_accentless};

    #[test]
    fn printed_presentation_writes_the_initial_uk_digraph() {
        use super::present_initial_uk_digraph;
        assert_eq!(
            present_initial_uk_digraph("оу\u{486}\u{301}мре"),
            "ᲂу\u{486}\u{301}мре"
        );
        assert_eq!(
            present_initial_uk_digraph("Оу\u{486}\u{301}мре"),
            "Оу\u{486}\u{301}мре"
        );
        assert_eq!(present_initial_uk_digraph("бои\u{301}тсѧ"), "бои\u{301}тсѧ");
        assert_eq!(present_initial_uk_digraph("о\u{486}трокъ"), "о\u{486}трокъ");
    }

    #[test]
    fn lookup_projections_fold_the_uk_digraph_to_its_expanded_letters() {
        assert_eq!(normalize_lookup("ᲂу҆́мре"), normalize_lookup("Оу҆́мре"));
        assert_eq!(normalize_lookup_accentless("ᲂу҆́мре"), "оумре");
        assert_eq!(normalize_lookup_accentless("ᲂумретъ"), "оумретъ");
        assert_eq!(normalize_lookup_accentless("Оу҆́мретъ"), "оумретъ");
        assert_ne!(normalize_lookup("ᲂу҆́мре"), normalize_lookup("ᲂумре"));
    }
}

#[cfg(test)]
mod uk_monograph_tests {

    #[test]
    fn wide_e_folds_to_e_in_both_projections() {
        assert_eq!(super::normalize_lookup("фарїсєй"), "фарїсей");
        assert_eq!(super::normalize_lookup("Єгѵпетъ"), "егѵпетъ");
        assert_eq!(
            super::normalize_lookup_accentless("словесє\u{0301}мъ"),
            super::normalize_lookup_accentless("словесе\u{0301}мъ"),
        );
    }

    #[test]
    fn broad_on_folds_to_o_in_both_projections() {
        assert_eq!(super::normalize_lookup("ѻдрѣ"), "одрѣ");
        assert_eq!(super::normalize_lookup("Ѻтроча"), "отроча");
        assert_eq!(
            super::normalize_lookup_accentless("ѻ\u{0486}дрѣ\u{0300}"),
            super::normalize_lookup_accentless("о\u{0486}дрѣ\u{0300}"),
        );
    }

    #[test]
    fn uk_monograph_folds_to_the_letter_pair_in_both_projections() {
        assert_eq!(super::normalize_lookup("ѹже"), "оуже");
        assert_eq!(super::normalize_lookup("Ѹже"), "оуже");
        assert_eq!(
            super::normalize_lookup_accentless("ѹ\u{0486}мре\u{0301}ти"),
            super::normalize_lookup_accentless("ᲂу\u{0486}мре\u{0301}ти"),
        );
        assert_eq!(
            super::normalize_lookup("ѹ\u{0486}мре\u{0301}ти"),
            super::normalize_lookup("оу\u{0486}мре\u{0301}ти"),
        );
    }
}
