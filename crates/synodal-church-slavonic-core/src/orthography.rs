// The Synodal orthography engine — lookup normalization, word validation, and
// the positional/initial presentation operations — is shared through the
// `church-slavonic-orthography` crate (docs/REWRITE_PLAN.md, target layout).
// The re-exported types and functions keep exactly the contracts this module
// declared before the extraction; only the cell-scoped paradigm types that
// carry family evidence (`PositionalParadigm`, `PositionalRule`) and the
// family-error-typed word wrappers stay here as thin adapters.
pub use church_slavonic_orthography::synodal::{
    InitialPresentation, LetterOccurrence, Loss, NormalizationReport, OrthographyProfile,
    PositionalOperation, PositionalReplacement, normalize_lookup, normalize_lookup_accentless,
    present_initial_uk_digraph,
};

pub(crate) use church_slavonic_orthography::synodal::is_synodal_vowel;

use church_slavonic_orthography::synodal::{
    self as engine, SynodalOrthographyError, contains_prosodic_mark,
};
use unicode_normalization::UnicodeNormalization;

use crate::{
    AccentScope, AuthorityRole, Case, Error, Evidence, EvidenceKind, GrammarCell, MetadataField,
    Number, Recension, Result,
};

impl From<SynodalOrthographyError> for Error {
    fn from(error: SynodalOrthographyError) -> Self {
        match error {
            SynodalOrthographyError::EmptyInput => Self::EmptyInput,
            SynodalOrthographyError::InvalidUnicode {
                byte_index,
                character,
                reason,
            } => Self::InvalidUnicode {
                byte_index,
                character,
                reason,
            },
            SynodalOrthographyError::InvalidOrthography { reason } => {
                Self::InvalidOrthography { reason }
            }
            SynodalOrthographyError::ContradictoryMetadata { reason } => {
                Self::ContradictoryMetadata { reason }
            }
        }
    }
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
        engine::validate_word(&source).map_err(Error::from)?;
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
            if church_slavonic_orthography::text::is_private_use(character)
                || character.is_control()
            {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "controls and private-use characters are forbidden".into(),
                });
            }
            if character.is_alphabetic() && !engine::is_cyrillic(character) {
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
pub struct PositionalRule {
    pub scope: AccentScope,
    pub operations: Vec<PositionalOperation>,
}

/// Complete caller-supplied positional spelling decisions for a lexeme.
/// An empty operation list is an explicit preserve decision, not missing
/// metadata. Exactly one rule must cover every requested liturgical cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PositionalParadigm {
    pub id: String,
    pub rules: Vec<PositionalRule>,
    pub evidence: Evidence,
}

impl PositionalParadigm {
    #[must_use]
    pub fn preserve(id: impl Into<String>, scope: AccentScope, evidence: Evidence) -> Self {
        Self {
            id: id.into(),
            rules: vec![PositionalRule {
                scope,
                operations: vec![],
            }],
            evidence,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.rules.is_empty() {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires a stable ID and at least one scoped rule"
                    .into(),
            });
        }
        if self.evidence.id.as_str().trim().is_empty()
            || self.evidence.source.as_str().trim().is_empty()
            || self.evidence.citation.trim().is_empty()
            || self.evidence.source_recension != Recension::SynodalRussian
            || self.evidence.kind != EvidenceKind::OrthographicParadigm
            || !self
                .evidence
                .authority_roles
                .contains(&AuthorityRole::Orthographic)
        {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires nonempty Synodal orthographic evidence"
                    .into(),
            });
        }
        if self.rules.iter().any(|rule| rule.scope.is_empty()) {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional rule cannot have an empty cell scope".into(),
            });
        }
        Ok(())
    }

    pub fn apply(&self, cell: GrammarCell, expanded: &str) -> Result<String> {
        self.validate()?;
        if contains_prosodic_mark(expanded) {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires an unaccented, unbreathed expanded form"
                    .into(),
            });
        }
        let mut applicable = self.rules.iter().filter(|rule| rule.scope.applies_to(cell));
        let rule = applicable
            .next()
            .ok_or(Error::OrthographicMetadataRequired {
                field: MetadataField::PositionalParadigm,
            })?;
        if applicable.next().is_some() {
            return Err(Error::ContradictoryMetadata {
                reason: "more than one positional rule applies to the requested cell".into(),
            });
        }
        let mut output = expanded.to_owned();
        for operation in &rule.operations {
            output = apply_positional_operation(cell, &output, operation)?;
        }
        Ok(SynodalWord::parse(output)?.canonical().to_owned())
    }
}

fn nominal_number_and_case(cell: GrammarCell) -> Option<(Number, Case)> {
    match cell {
        GrammarCell::Noun(cell) | GrammarCell::VerbalNoun(cell) => Some((cell.number, cell.case)),
        GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell) => {
            Some((cell.number, cell.case))
        }
        GrammarCell::Numeral(cell) => Some((cell.number, cell.case)),
        GrammarCell::Participle(cell) => Some((cell.agreement.number, cell.agreement.case)),
        GrammarCell::Pronoun(cell) => Some((cell.number, cell.case)),
        _ => None,
    }
}

fn apply_positional_operation(
    cell: GrammarCell,
    value: &str,
    operation: &PositionalOperation,
) -> Result<String> {
    match operation {
        PositionalOperation::Initial(presentation) => {
            Ok(apply_initial_presentation(&SynodalWord::parse(value)?, *presentation)?.normalized)
        }
        PositionalOperation::DecimalIBeforeVowel => {
            engine::decimal_i_before_vowel(value).map_err(Error::from)
        }
        PositionalOperation::WidePluralEnding => {
            engine::apply_wide_plural_ending(nominal_number_and_case(cell), value)
                .map_err(Error::from)
        }
        PositionalOperation::Replace {
            replacement,
            occurrence,
        } => engine::replace_occurrence(value, *replacement, *occurrence).map_err(Error::from),
    }
}

/// Applies one reviewed positional-letter decision and reports the change.
/// This is deliberately explicit: lexical semantics decide exceptions such as
/// the two spellings of `ꙗзыкъ`/`ѧзыкъ`, not a blind string rewrite.
pub fn apply_initial_presentation(
    word: &SynodalWord,
    presentation: InitialPresentation,
) -> Result<NormalizationReport> {
    engine::apply_initial_presentation(word.canonical(), presentation).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Animacy, EpistemicRole, EvidenceId, NounCell, SourceId};

    fn evidence() -> Evidence {
        Evidence {
            id: EvidenceId::from("alypy-2-positional-test"),
            source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::OrthographicParadigm,
            authority_roles: vec![AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::CallerSuppliedMetadata,
            citation: "Alypy §§2, 36".into(),
            note: None,
        }
    }

    fn noun_cell(case: Case, number: Number) -> GrammarCell {
        GrammarCell::Noun(NounCell {
            case,
            number,
            animacy: Animacy::Animate,
        })
    }

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
                apply_initial_presentation(
                    &SynodalWord::parse(input).expect("valid initial-presentation input"),
                    presentation,
                )
                .expect("compatible initial presentation")
                .normalized,
                expected
            );
        }
    }

    #[test]
    fn scoped_positional_paradigm_preserves_semantics_and_case_distinctions() {
        let singular = noun_cell(Case::Nominative, Number::Singular);
        let plural = noun_cell(Case::Genitive, Number::Plural);
        let paradigm = PositionalParadigm {
            id: "farisei-case-spelling".into(),
            rules: vec![
                PositionalRule {
                    scope: AccentScope::OtherCells(vec![singular]),
                    operations: vec![],
                },
                PositionalRule {
                    scope: AccentScope::OtherCells(vec![plural]),
                    operations: vec![PositionalOperation::Replace {
                        replacement: PositionalReplacement::WideE,
                        occurrence: LetterOccurrence::FromEnd(0),
                    }],
                },
            ],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(singular, "фарисей")
                .expect("singular preserve rule"),
            "фарисей"
        );
        assert_eq!(
            paradigm
                .apply(plural, "фарисей")
                .expect("plural wide-e rule"),
            "фарисєй"
        );

        let people = PositionalParadigm {
            id: "yazyk-people".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::Initial(InitialPresentation::IotatedYa)],
            }],
            evidence: evidence(),
        };
        let organ = PositionalParadigm::preserve("yazyk-organ", AccentScope::All, evidence());
        assert_eq!(
            people
                .apply(singular, "ѧзыкъ")
                .expect("people semantic spelling"),
            "ꙗзыкъ"
        );
        assert_eq!(
            organ
                .apply(singular, "ѧзыкъ")
                .expect("organ semantic spelling"),
            "ѧзыкъ"
        );
    }

    #[test]
    fn decimal_i_rule_is_opt_in_and_preserves_the_sion_exception() {
        let cell = noun_cell(Case::Genitive, Number::Singular);
        let regular = PositionalParadigm {
            id: "i-before-vowel".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::DecimalIBeforeVowel],
            }],
            evidence: evidence(),
        };
        let exception =
            PositionalParadigm::preserve("sion-king-exception", AccentScope::All, evidence());
        assert_eq!(
            regular
                .apply(cell, "сиона")
                .expect("regular decimal-i rule"),
            "сїона"
        );
        assert_eq!(
            exception.apply(cell, "сиѡна").expect("Sihon preserve rule"),
            "сиѡна"
        );
    }

    #[test]
    fn number_antistich_letter_pairs_are_all_source_selectable() {
        let cell = noun_cell(Case::Instrumental, Number::Plural);
        for (input, replacement, occurrence, expected) in [
            (
                "жене",
                PositionalReplacement::WideE,
                LetterOccurrence::FromEnd(0),
                "женє",
            ),
            (
                "мироносицы",
                PositionalReplacement::Omega,
                LetterOccurrence::FromStart(0),
                "мирѡносицы",
            ),
            (
                "мужи",
                PositionalReplacement::Yeri,
                LetterOccurrence::FromEnd(0),
                "мужы",
            ),
            (
                "пришедша",
                PositionalReplacement::LittleYus,
                LetterOccurrence::FromEnd(0),
                "пришедшѧ",
            ),
        ] {
            let paradigm = PositionalParadigm {
                id: "source-selected-number-antistich".into(),
                rules: vec![PositionalRule {
                    scope: AccentScope::All,
                    operations: vec![PositionalOperation::Replace {
                        replacement,
                        occurrence,
                    }],
                }],
                evidence: evidence(),
            };
            assert_eq!(
                paradigm
                    .apply(cell, input)
                    .expect("compatible number-antistich replacement"),
                expected
            );
        }
        assert_eq!(PositionalReplacement::ALL.len(), 7);
    }

    #[test]
    fn selected_plural_ending_rule_is_wide_but_singular_instrumentals_are_not() {
        let paradigm = PositionalParadigm {
            id: "alypy-36-wide-endings".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::WidePluralEnding],
            }],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Dative, Number::Plural), "рабомъ")
                .expect("plural dative wide ending"),
            "рабѡмъ"
        );
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Genitive, Number::Plural), "сыновъ")
                .expect("plural genitive wide ending"),
            "сынѡвъ"
        );
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Instrumental, Number::Singular), "рабомъ",)
                .expect("singular instrumental preserve"),
            "рабомъ"
        );
    }

    #[test]
    fn positional_paradigms_fail_typed_on_missing_overlap_and_bad_occurrence() {
        let cell = noun_cell(Case::Nominative, Number::Singular);
        let missing = PositionalParadigm::preserve(
            "missing",
            AccentScope::OtherCells(vec![noun_cell(Case::Genitive, Number::Plural)]),
            evidence(),
        );
        assert!(matches!(
            missing.apply(cell, "его"),
            Err(Error::OrthographicMetadataRequired {
                field: MetadataField::PositionalParadigm
            })
        ));

        let mut overlap = PositionalParadigm::preserve("overlap", AccentScope::All, evidence());
        overlap.rules.push(PositionalRule {
            scope: AccentScope::All,
            operations: vec![],
        });
        assert!(matches!(
            overlap.apply(cell, "его"),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let bad = PositionalParadigm {
            id: "bad-occurrence".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::Replace {
                    replacement: PositionalReplacement::Omega,
                    occurrence: LetterOccurrence::FromStart(2),
                }],
            }],
            evidence: evidence(),
        };
        assert!(matches!(
            bad.apply(cell, "слово"),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
