//! Explicit, evidence-carrying reconstruction of OCS stress placement.
//!
//! Canonical Old Church Slavonic manuscripts do not supply a complete,
//! standardized accent orthography. This module therefore never infers stress
//! from a spelling and never labels its output as attested. It only renders a
//! caller-supplied, cell-scoped reconstruction. Stressed cells use an acute
//! accent as a neutral scholarly marker; atonic cells remain unmarked.

use std::collections::BTreeSet;

use unicode_normalization::UnicodeNormalization;

use crate::{InflectionError, MetadataField, RequestedCell, RuleId, RuleStep, Script};

const RECONSTRUCTED_STRESS_MARK: char = '\u{0301}';

/// Reconstructed stress placement in the complete generated wordform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccentPlacement {
    /// The complete wordform is explicitly reconstructed as atonic.
    Unaccented,
    /// Count vowels from the lexical left edge, starting at zero.
    VowelFromStart(u8),
    /// Count vowels from the word's right edge, starting at zero.
    VowelFromEnd(u8),
}

/// The grammatical cells to which one reconstructed placement applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccentScope {
    All,
    Cells(Vec<RequestedCell>),
}

impl AccentScope {
    fn applies_to(&self, cell: &RequestedCell) -> bool {
        match self {
            Self::All => true,
            Self::Cells(cells) => cells.contains(cell),
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::All, _) | (_, Self::All) => true,
            (Self::Cells(left), Self::Cells(right)) => left.iter().any(|cell| right.contains(cell)),
        }
    }
}

/// One placement rule in an OCS stress reconstruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccentRule {
    pub scope: AccentScope,
    pub placement: AccentPlacement,
}

/// How strongly the supplied stress reconstruction is supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccentReconstructionStatus {
    /// Comparative or historical evidence supports the reconstruction.
    Comparative,
    /// More than one source-backed reconstruction remains viable.
    Disputed,
}

/// Source identity and citation carried by every reconstructed output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccentEvidence {
    pub source_id: String,
    pub citation: String,
    pub status: AccentReconstructionStatus,
}

/// A complete, explicit accent contract for a set of grammatical cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccentParadigm {
    pub id: String,
    pub rules: Vec<AccentRule>,
    pub evidence: AccentEvidence,
}

impl AccentParadigm {
    /// Construct a fixed placement shared by every requested cell.
    pub fn fixed(
        id: impl Into<String>,
        placement: AccentPlacement,
        evidence: AccentEvidence,
    ) -> Self {
        Self {
            id: id.into(),
            rules: vec![AccentRule {
                scope: AccentScope::All,
                placement,
            }],
            evidence,
        }
    }

    /// Validate source identity, scopes, and non-overlap before generation.
    pub fn validate(&self) -> Result<(), InflectionError> {
        if self.id.trim().is_empty()
            || self.evidence.source_id.trim().is_empty()
            || self.evidence.citation.trim().is_empty()
            || self.rules.is_empty()
        {
            return Err(InflectionError::ContradictoryLexicalMetadata {
                fields: vec![MetadataField::AccentParadigm],
            });
        }
        for rule in &self.rules {
            if let AccentScope::Cells(cells) = &rule.scope {
                let unique = cells.iter().collect::<BTreeSet<_>>();
                if cells.is_empty() || unique.len() != cells.len() {
                    return Err(InflectionError::ContradictoryLexicalMetadata {
                        fields: vec![MetadataField::AccentParadigm],
                    });
                }
            }
        }
        for (index, left) in self.rules.iter().enumerate() {
            if self
                .rules
                .iter()
                .skip(index + 1)
                .any(|right| left.scope.overlaps(&right.scope))
            {
                return Err(InflectionError::ContradictoryLexicalMetadata {
                    fields: vec![MetadataField::AccentParadigm],
                });
            }
        }
        Ok(())
    }

    /// Render the unique reconstructed stress rule licensed for one cell.
    pub fn apply(
        &self,
        cell: &RequestedCell,
        accentless: &str,
    ) -> Result<ReconstructedAccent, InflectionError> {
        self.validate()?;
        let canonical = crate::orthography::canonical_display(accentless)?;
        if crate::orthography::detect_script(&canonical) != Script::Cyrillic {
            return Err(InflectionError::InvalidInput {
                reason: "the reconstructed OCS accent profile requires a Cyrillic wordform"
                    .to_string(),
            });
        }
        if contains_source_prosodic_mark(&canonical) {
            return Err(InflectionError::ContradictoryLexicalMetadata {
                fields: vec![MetadataField::AccentParadigm],
            });
        }

        let mut applicable = self.rules.iter().filter(|rule| rule.scope.applies_to(cell));
        let rule = applicable
            .next()
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::AccentParadigm],
            })?;
        if applicable.next().is_some() {
            return Err(InflectionError::ContradictoryLexicalMetadata {
                fields: vec![MetadataField::AccentParadigm],
            });
        }

        let (text, reason) = match rule.placement {
            AccentPlacement::Unaccented => (
                canonical.clone(),
                "record an explicit reconstruction of OCS accentlessness",
            ),
            placement => {
                let accent_index = vowel_index(&canonical, placement)?;
                let mut text =
                    String::with_capacity(canonical.len() + RECONSTRUCTED_STRESS_MARK.len_utf8());
                for (index, character) in canonical.char_indices() {
                    text.push(character);
                    if index == accent_index {
                        text.push(RECONSTRUCTED_STRESS_MARK);
                    }
                }
                (
                    text.nfc().collect::<String>(),
                    "place an explicit evidence-carrying reconstruction of OCS stress",
                )
            }
        };
        crate::orthography::Lemma::parse(&text)?;
        Ok(ReconstructedAccent {
            text: text.clone(),
            paradigm_id: self.id.clone(),
            evidence: self.evidence.clone(),
            trace: vec![RuleStep {
                rule_id: RuleId::OrthographyReconstructedAccent,
                before: canonical,
                after: text,
                reason,
            }],
        })
    }
}

/// An accent-analyzed form whose reconstruction status and evidence are explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructedAccent {
    text: String,
    paradigm_id: String,
    evidence: AccentEvidence,
    trace: Vec<RuleStep>,
}

impl ReconstructedAccent {
    /// Return the accent-analyzed wordform, marked or explicitly atonic.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the caller-supplied identity of the reconstruction paradigm.
    pub fn paradigm_id(&self) -> &str {
        &self.paradigm_id
    }

    /// Return the comparative or disputed evidence attached to this result.
    pub fn evidence(&self) -> &AccentEvidence {
        &self.evidence
    }

    /// Return the auditable reconstruction trace.
    pub fn trace(&self) -> &[RuleStep] {
        &self.trace
    }
}

fn contains_source_prosodic_mark(value: &str) -> bool {
    value.nfd().any(|character| {
        matches!(
            character,
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0485}' | '\u{0486}'
        )
    })
}

fn vowel_index(value: &str, placement: AccentPlacement) -> Result<usize, InflectionError> {
    let vowels = value
        .char_indices()
        .filter_map(|(index, character)| is_vowel(character).then_some(index))
        .collect::<Vec<_>>();
    let selected = match placement {
        AccentPlacement::Unaccented => None,
        AccentPlacement::VowelFromStart(offset) => vowels.get(usize::from(offset)).copied(),
        AccentPlacement::VowelFromEnd(offset) => vowels
            .len()
            .checked_sub(usize::from(offset) + 1)
            .and_then(|index| vowels.get(index).copied()),
    };
    selected.ok_or_else(|| InflectionError::ContradictoryLexicalMetadata {
        fields: vec![MetadataField::AccentParadigm],
    })
}

fn is_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'ꙑ'
            | 'ъ'
            | 'ь'
            | 'ѣ'
            | 'ю'
            | 'ꙗ'
            | 'ѧ'
            | 'ѩ'
            | 'ѫ'
            | 'ѭ'
            | 'ѥ'
            | 'ѵ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, NounCell, Number};

    fn evidence(status: AccentReconstructionStatus) -> AccentEvidence {
        AccentEvidence {
            source_id: "comparative-accentology".to_string(),
            citation: "explicit test reconstruction".to_string(),
            status,
        }
    }

    #[test]
    fn fixed_reconstruction_covers_all_twenty_one_noun_cells() {
        let paradigm = AccentParadigm::fixed(
            "fixed-first-vowel",
            AccentPlacement::VowelFromStart(0),
            evidence(AccentReconstructionStatus::Comparative),
        );
        for cell in NounCell::all() {
            let output = paradigm
                .apply(&RequestedCell::Noun(cell), "града")
                .expect("complete fixed reconstruction");
            assert_eq!(output.text(), "гра́да");
            assert_eq!(
                output.evidence().status,
                AccentReconstructionStatus::Comparative
            );
            assert_eq!(
                output.trace()[0].rule_id,
                RuleId::OrthographyReconstructedAccent
            );
        }
    }

    #[test]
    fn mobile_and_cell_specific_reconstructions_are_explicit() {
        let singular = NounCell::all()
            .filter(|cell| cell.number == Number::Singular)
            .map(RequestedCell::Noun)
            .collect();
        let non_singular = NounCell::all()
            .filter(|cell| cell.number != Number::Singular)
            .map(RequestedCell::Noun)
            .collect();
        let paradigm = AccentParadigm {
            id: "mobile-noun".to_string(),
            rules: vec![
                AccentRule {
                    scope: AccentScope::Cells(singular),
                    placement: AccentPlacement::VowelFromStart(0),
                },
                AccentRule {
                    scope: AccentScope::Cells(non_singular),
                    placement: AccentPlacement::VowelFromEnd(0),
                },
            ],
            evidence: evidence(AccentReconstructionStatus::Disputed),
        };
        for cell in NounCell::all() {
            let output = paradigm
                .apply(&RequestedCell::Noun(cell), "града")
                .expect("every noun cell is explicitly scoped");
            let expected = if cell.number == Number::Singular {
                "гра́да"
            } else {
                "града́"
            };
            assert_eq!(output.text(), expected);
        }
    }

    #[test]
    fn missing_overlapping_out_of_range_and_source_marks_fail_closed() {
        let citation = RequestedCell::Noun(NounCell {
            case: Case::Nominative,
            number: Number::Singular,
        });
        let missing = AccentParadigm {
            id: "missing".to_string(),
            rules: vec![AccentRule {
                scope: AccentScope::Cells(vec![RequestedCell::Noun(NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                })]),
                placement: AccentPlacement::VowelFromStart(0),
            }],
            evidence: evidence(AccentReconstructionStatus::Comparative),
        };
        assert_eq!(
            missing.apply(&citation, "градъ"),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::AccentParadigm],
            })
        );

        let overlapping = AccentParadigm {
            id: "overlap".to_string(),
            rules: vec![
                AccentRule {
                    scope: AccentScope::All,
                    placement: AccentPlacement::VowelFromStart(0),
                },
                AccentRule {
                    scope: AccentScope::Cells(vec![citation.clone()]),
                    placement: AccentPlacement::VowelFromEnd(0),
                },
            ],
            evidence: evidence(AccentReconstructionStatus::Comparative),
        };
        assert!(matches!(
            overlapping.apply(&citation, "градъ"),
            Err(InflectionError::ContradictoryLexicalMetadata { .. })
        ));

        let outside = AccentParadigm::fixed(
            "outside",
            AccentPlacement::VowelFromStart(9),
            evidence(AccentReconstructionStatus::Comparative),
        );
        assert!(matches!(
            outside.apply(&citation, "градъ"),
            Err(InflectionError::ContradictoryLexicalMetadata { .. })
        ));

        let fixed = AccentParadigm::fixed(
            "fixed",
            AccentPlacement::VowelFromStart(0),
            evidence(AccentReconstructionStatus::Comparative),
        );
        for source_marked in ["гра́дъ", "е҆лефантъ", "гра̀дъ", "гра̑дъ"]
        {
            assert!(matches!(
                fixed.apply(&citation, source_marked),
                Err(InflectionError::ContradictoryLexicalMetadata { .. })
            ));
        }
        assert!(matches!(
            fixed.apply(&citation, "ⰳⱃⰰⰴⱏ"),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn yers_are_valid_reconstructed_stress_bearers() {
        let paradigm = AccentParadigm::fixed(
            "strong-yer",
            AccentPlacement::VowelFromEnd(0),
            evidence(AccentReconstructionStatus::Comparative),
        );
        let cell = RequestedCell::RawFeature {
            feature: "accent-test".to_string(),
        };
        assert_eq!(
            paradigm
                .apply(&cell, "дьнь")
                .expect("yer-bearing reconstruction")
                .text(),
            "дьнь́"
        );
    }

    #[test]
    fn explicitly_unaccented_cells_and_vowelless_forms_are_supported() {
        let paradigm = AccentParadigm::fixed(
            "atonic-clitic",
            AccentPlacement::Unaccented,
            evidence(AccentReconstructionStatus::Comparative),
        );
        let cell = RequestedCell::RawFeature {
            feature: "atonic-test".to_string(),
        };
        for form in ["и", "ж"] {
            let output = paradigm
                .apply(&cell, form)
                .expect("explicit atonic reconstruction");
            assert_eq!(output.text(), form);
            assert_eq!(output.trace()[0].before, output.trace()[0].after);
        }
    }
}
