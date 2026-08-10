//! Versioned Synodal Church Slavonic collation.
//!
//! The tailoring follows Unicode Technical Note #41, section 5.1.  It is a
//! deliberately small, dependency-free implementation for validated Synodal
//! words, rather than a general replacement for the Unicode Collation
//! Algorithm.  The individual levels remain visible so callers cannot confuse
//! primary sort equivalence with lexical identity.

use std::cmp::Ordering;

use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result, SynodalWord};

/// The immutable rule set used to construct a collation key.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CollationProfile {
    /// The Synodal tailoring printed in UTN #41, revision 1, section 5.1.
    Utn41Revision1,
}

impl CollationProfile {
    pub const ALL: [Self; 1] = [Self::Utn41Revision1];
}

/// The strongest collation level to consider during comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CollationStrength {
    /// Phonetic alphabetic order; positional glyph variants compare equal.
    Primary,
    /// Also compare accents, breathings, titla, and superscript letters.
    Secondary,
    /// Also compare case, with uppercase first as specified by UTN #41.
    Case,
    /// Also compare functional positional variants such as `о` and `ѡ`.
    Tertiary,
    /// Finally use normalized scalar values as a deterministic tie-breaker.
    Identical,
}

impl CollationStrength {
    pub const ALL: [Self; 5] = [
        Self::Primary,
        Self::Secondary,
        Self::Case,
        Self::Tertiary,
        Self::Identical,
    ];
}

/// An inspectable, deterministic key for one validated Synodal word.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CollationKey {
    profile: CollationProfile,
    primary: Vec<u16>,
    secondary: Vec<u16>,
    case: Vec<u8>,
    tertiary: Vec<u8>,
    identical: Vec<u32>,
}

impl CollationKey {
    #[must_use]
    pub const fn profile(&self) -> CollationProfile {
        self.profile
    }

    #[must_use]
    pub fn primary(&self) -> &[u16] {
        &self.primary
    }

    #[must_use]
    pub fn secondary(&self) -> &[u16] {
        &self.secondary
    }

    #[must_use]
    pub fn case(&self) -> &[u8] {
        &self.case
    }

    #[must_use]
    pub fn tertiary(&self) -> &[u8] {
        &self.tertiary
    }

    /// Compare this key with another key through the requested strength.
    ///
    /// Keys made with different profiles are ordered by profile first.  This
    /// prevents an accidental comparison after a future tailoring revision.
    #[must_use]
    pub fn compare_at(&self, other: &Self, strength: CollationStrength) -> Ordering {
        let profile = self.profile.cmp(&other.profile);
        if profile != Ordering::Equal {
            return profile;
        }
        let primary = self.primary.cmp(&other.primary);
        if primary != Ordering::Equal || strength == CollationStrength::Primary {
            return primary;
        }
        let secondary = self.secondary.cmp(&other.secondary);
        if secondary != Ordering::Equal || strength == CollationStrength::Secondary {
            return secondary;
        }
        let case = self.case.cmp(&other.case);
        if case != Ordering::Equal || strength == CollationStrength::Case {
            return case;
        }
        let tertiary = self.tertiary.cmp(&other.tertiary);
        if tertiary != Ordering::Equal || strength == CollationStrength::Tertiary {
            return tertiary;
        }
        self.identical.cmp(&other.identical)
    }

    #[must_use]
    pub fn equivalent_at(&self, other: &Self, strength: CollationStrength) -> bool {
        self.compare_at(other, strength) == Ordering::Equal
    }
}

impl Ord for CollationKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_at(other, CollationStrength::Identical)
    }
}

impl PartialOrd for CollationKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
struct Element {
    base: char,
    marks: Vec<char>,
}

/// Construct a collation key according to the selected Synodal tailoring.
pub fn collation_key(word: &SynodalWord, profile: CollationProfile) -> Result<CollationKey> {
    match profile {
        CollationProfile::Utn41Revision1 => utn41_revision_1_key(word),
    }
}

/// Compare two words directly at a chosen strength.
pub fn compare_synodal(
    left: &SynodalWord,
    right: &SynodalWord,
    profile: CollationProfile,
    strength: CollationStrength,
) -> Result<Ordering> {
    Ok(collation_key(left, profile)?.compare_at(&collation_key(right, profile)?, strength))
}

fn utn41_revision_1_key(word: &SynodalWord) -> Result<CollationKey> {
    let expanded = expand_noncanonical_compatibility_characters(word.canonical());
    let mut elements = cluster(expanded.nfd())?;
    contract_uk(&mut elements);

    let mut primary = Vec::new();
    let mut case = Vec::new();
    let mut tertiary = Vec::new();
    for element in &elements {
        let (weights, variant) = primary_and_tertiary(element.base)?;
        primary.extend(weights.iter().copied());
        case.extend(std::iter::repeat_n(
            case_weight(element.base),
            weights.len(),
        ));
        tertiary.extend(std::iter::repeat_n(variant, weights.len()));
    }

    // UTN #41 requests backward secondary sorting. Keeping an explicit zero
    // for an unmarked cluster retains the position of otherwise ignorable base
    // characters while scanning the clusters from right to left.
    let secondary = elements
        .iter()
        .rev()
        .flat_map(|element| {
            if element.marks.is_empty() {
                vec![0]
            } else {
                element
                    .marks
                    .iter()
                    .filter_map(|mark| secondary_weight(*mark))
                    .collect()
            }
        })
        .collect();

    Ok(CollationKey {
        profile: CollationProfile::Utn41Revision1,
        primary,
        secondary,
        case,
        tertiary,
        identical: word.canonical().chars().map(u32::from).collect(),
    })
}

fn expand_noncanonical_compatibility_characters(value: &str) -> String {
    let mut expanded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            // Omega with psili and kamora has no canonical Unicode
            // decomposition, although UTN #41 requires this equivalence.
            'ѽ' => expanded.push_str("ꙍ\u{0486}\u{0311}"),
            'Ѽ' => expanded.push_str("Ꙍ\u{0486}\u{0311}"),
            // Ot is an expansion (omega + te), not omega with a superscript.
            'ѿ' => expanded.push_str("ѡт"),
            'Ѿ' => expanded.push_str("ѠТ"),
            // Unicode's precomposed uk characters are compatibility spellings
            // of the two-character digraph for Synodal collation.
            'ѹ' => expanded.push_str("оу"),
            'Ѹ' => expanded.push_str("ОУ"),
            _ => expanded.push(character),
        }
    }
    expanded
}

fn cluster(characters: impl Iterator<Item = char>) -> Result<Vec<Element>> {
    let mut elements: Vec<Element> = Vec::new();
    for character in characters {
        if unicode_normalization::char::canonical_combining_class(character) == 0
            && character != '\u{034f}'
        {
            elements.push(Element {
                base: character,
                marks: Vec::new(),
            });
        } else if let Some(element) = elements.last_mut() {
            element.marks.push(character);
        } else {
            return Err(Error::InvalidOrthography {
                reason: "collation input begins with a combining mark".into(),
            });
        }
    }
    Ok(elements)
}

fn contract_uk(elements: &mut Vec<Element>) {
    let mut contracted = Vec::with_capacity(elements.len());
    let mut index = 0;
    while index < elements.len() {
        if index + 1 < elements.len()
            && matches!(elements[index].base, 'о' | 'О' | 'ᲂ' | 'ᲃ')
            && matches!(elements[index + 1].base, 'у' | 'У')
            && !elements[index].marks.contains(&'\u{034f}')
        {
            let mut element = elements[index].clone();
            // The compatibility uk scalars are used here only as internal
            // sentinels after the public input has already been expanded.
            element.base = if element.base.is_uppercase() {
                'Ѹ'
            } else {
                'ѹ'
            };
            element.marks.extend(elements[index + 1].marks.iter());
            contracted.push(element);
            index += 2;
        } else {
            contracted.push(elements[index].clone());
            index += 1;
        }
    }
    *elements = contracted;
}

fn primary_and_tertiary(character: char) -> Result<(Vec<u16>, u8)> {
    let lowercase = character.to_lowercase().next().unwrap_or(character);
    let value = match lowercase {
        'а' => (vec![1], 0),
        'б' => (vec![2], 0),
        'в' => (vec![3], 0),
        'г' => (vec![4], 0),
        'д' | 'ᲁ' => (vec![5], 0),
        'е' => (vec![6], 0),
        'є' => (vec![6], 1),
        'ж' => (vec![7], 0),
        'ѕ' => (vec![8], 0),
        'з' | 'ꙁ' => (vec![9], 0),
        'и' => (vec![10], 0),
        'і' => (vec![11], 0),
        'к' => (vec![12], 0),
        'л' => (vec![13], 0),
        'м' => (vec![14], 0),
        'н' => (vec![15], 0),
        'ѻ' => (vec![16], 0),
        'о' | 'ᲂ' => (vec![16], 1),
        'ѡ' => (vec![16], 2),
        'ꙍ' => (vec![16], 3),
        'п' => (vec![17], 0),
        'р' => (vec![18], 0),
        'с' => (vec![19], 0),
        'т' => (vec![20], 0),
        'ѹ' => (vec![21], 0),
        'ꙋ' => (vec![21], 1),
        'у' => (vec![21], 2),
        'ф' => (vec![22], 0),
        'х' => (vec![23], 0),
        'ц' => (vec![24], 0),
        'ч' => (vec![25], 0),
        'ш' => (vec![26], 0),
        'щ' => (vec![27], 0),
        'ъ' => (vec![28], 0),
        'ы' => (vec![29], 0),
        'ь' => (vec![30], 0),
        'э' => (vec![31], 0),
        'ѣ' => (vec![32], 0),
        'ю' => (vec![33], 0),
        'ѫ' => (vec![34], 0),
        'я' => (vec![35], 0),
        'ꙗ' => (vec![36], 0),
        'ѧ' => (vec![36], 1),
        'ѯ' => (vec![37], 0),
        'ѱ' => (vec![38], 0),
        'ѳ' => (vec![39], 0),
        'ѵ' => (vec![40], 0),
        other => {
            return Err(Error::InvalidOrthography {
                reason: format!(
                    "character {other:?} is outside the UTN #41 Synodal collation alphabet"
                ),
            });
        }
    };
    Ok(value)
}

fn case_weight(character: char) -> u8 {
    if character.is_uppercase() { 0 } else { 1 }
}

fn secondary_weight(character: char) -> Option<u16> {
    Some(match character {
        // Pokrytie, combining/non-combining kavyka, and CGJ are ignorable.
        '\u{0487}' | '\u{a67c}' | '\u{a67e}' | '\u{034f}' => return None,
        '\u{0485}' => 1, // dasia
        '\u{0486}' => 2, // psili
        '\u{0301}' => 3, // acute
        '\u{0300}' => 4, // grave
        '\u{0311}' => 5, // kamora
        '\u{0483}' => 6, // titlo
        '\u{0306}' => 7, // breve
        '\u{0308}' | '\u{030f}' => 8,
        // Combining Cyrillic letters sort after ordinary diacritics. Their
        // codepoint order is stable but offset so it cannot collide above.
        '\u{2de0}'..='\u{2dff}' => 32 + (u32::from(character) - 0x2de0) as u16,
        '\u{a674}'..='\u{a67b}' => 96 + (u32::from(character) - 0xa674) as u16,
        '\u{a69e}'..='\u{a69f}' => 112 + (u32::from(character) - 0xa69e) as u16,
        '\u{033e}' | '\u{2e2f}' | '\u{a67d}' | '\u{a67f}' => 120,
        // Other validated combining marks are retained deterministically after
        // the tailored repertoire instead of being silently discarded.
        other => 256 + (u32::from(other) & 0x0fff) as u16,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: &str) -> CollationKey {
        collation_key(
            &SynodalWord::parse(value).expect("valid Synodal word"),
            CollationProfile::Utn41Revision1,
        )
        .expect("tailored character repertoire")
    }

    #[test]
    fn positional_o_variants_share_primary_but_have_tertiary_order() {
        let broad = key("ѻба");
        let ordinary = key("оба");
        let omega = key("ѡба");
        assert!(broad.equivalent_at(&ordinary, CollationStrength::Primary));
        assert!(ordinary.equivalent_at(&omega, CollationStrength::Primary));
        assert!(broad < ordinary);
        assert!(ordinary < omega);
    }

    #[test]
    fn digraph_uk_sorts_as_u_and_before_monograph_uk() {
        let digraph = key("ᲂу");
        let precomposed = key("ѹ");
        let monograph = key("ꙋ");
        let modern = key("у");
        assert!(digraph.equivalent_at(&modern, CollationStrength::Primary));
        assert!(digraph.equivalent_at(&precomposed, CollationStrength::Tertiary));
        assert!(digraph < monograph);
        assert!(monograph < modern);
    }

    #[test]
    fn ot_expands_to_omega_te() {
        assert!(key("ѿрокъ").equivalent_at(&key("ѡтрокъ"), CollationStrength::Tertiary));
    }

    #[test]
    fn accent_position_uses_backward_secondary_sorting() {
        assert!(key("дв҃а") < key("два̀"));
        assert!(key("а") < key("а́"));
    }

    #[test]
    fn uppercase_sorts_before_lowercase_after_secondary_level() {
        let upper = key("А");
        let lower = key("а");
        assert!(upper.equivalent_at(&lower, CollationStrength::Secondary));
        assert!(upper < lower);
    }

    #[test]
    fn canonical_equivalents_get_identical_keys() {
        assert_eq!(key("й"), key("й"));
        assert_eq!(key("ї"), key("ї"));
    }

    #[test]
    fn yat_remains_a_distinct_primary_letter() {
        assert!(!key("ѣ").equivalent_at(&key("е"), CollationStrength::Primary));
    }
}
