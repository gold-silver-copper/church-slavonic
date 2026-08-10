#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abbreviation;
mod handles;
mod inflector;
mod paradigm;
pub mod phrases;
mod registry;
mod resolver;

pub use abbreviation::Abbreviation;
pub use handles::{Adjective, Capabilities, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use inflector::{Inflector, InflectorBuilder};
pub use paradigm::{Paradigm, ParadigmRow, ParadigmStatus};
pub use registry::{
    AlignmentSummary, IrregularOverrideSummary, LexemeSummary, PartOfSpeech, PositionalRuleSummary,
    RecensionConflictSummary, TransformationRuleSummary,
};
pub use synodal_church_slavonic_core as core;
pub use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AnalyticConstruction, Animacy, Case, CollationKey,
    CollationProfile, CollationStrength, Comparison, Confidence, CyrillicNumeral, Error,
    FiniteTense, FiniteVerbCell, FormSet, FormSource, Gender, GenerationPolicy, GrammarCell,
    ImperativeCell, InitialPresentation, LParticipleCell, LexemeId, Loss, MetadataField, NounCell,
    Number, NumeralCell, NumeralKind, OrthographyProfile, ParticipleCell, ParticipleTense,
    ParticipleVoice, Person, PhraseRole, PhraseToken, PronounCell, RealizedPhrase, Recension,
    RenderedText, Result, SynodalWord, TransliterationScheme, VariantPolicy,
    apply_initial_presentation, collation_key, compare_synodal, format_cyrillic_numeral,
    normalize_lookup, normalize_lookup_accentless, parse_cyrillic_numeral, transliterate,
};

/// Resolves a lemma while retaining its stable target identity.
pub fn lookup(lemma: &str) -> Result<LexemeSummary> {
    handles::resolve_summary(lemma)
}

/// Returns every curated target lexeme in deterministic ID order.
pub fn lexemes() -> Result<Vec<LexemeSummary>> {
    registry::all_lexemes()
}

/// Returns the reviewed OCS-to-Synodal alignment gold registry, including
/// rejected negative rows.
pub fn recension_alignments() -> Result<Vec<AlignmentSummary>> {
    registry::alignments()
}

/// Returns the explicit, reviewed OCS-to-Synodal transformation-rule registry.
#[must_use]
pub fn recension_transformations() -> Vec<TransformationRuleSummary> {
    registry::transformation_rules()
}

/// Returns preserved conflicts and rejected alignment controls.
#[must_use]
pub fn recension_conflicts() -> Vec<RecensionConflictSummary> {
    registry::conflicts()
}

/// Returns the reviewable positional-letter rules and their exceptions.
#[must_use]
pub fn positional_rules() -> Vec<PositionalRuleSummary> {
    registry::positional_rules()
}

/// Returns the systems whose exact tables override productive formation.
#[must_use]
pub fn irregular_overrides() -> Vec<IrregularOverrideSummary> {
    registry::irregular_overrides()
}

pub fn noun(lemma: &str, case: Case, number: Number, animacy: Animacy) -> Result<FormSet> {
    Noun::resolve(lemma)?.form(case, number, animacy)
}

pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form: AdjectiveForm::Short,
        comparison: Comparison::Positive,
    })
}

pub fn long_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form: AdjectiveForm::Long,
        comparison: Comparison::Positive,
    })
}

/// Inflects an adjective in a fully specified grammatical cell.
///
/// Use [`short_adjective`] or [`long_adjective`] when only a positive-form
/// convenience call is needed.
pub fn adjective(lemma: &str, cell: AdjectiveCell) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(cell)
}

pub fn present(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.present(person, number)
}

pub fn imperfect(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.imperfect(person, number)
}

pub fn aorist(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.aorist(person, number)
}

pub fn imperative(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.imperative(person, number)
}

pub fn infinitive(lemma: &str) -> Result<FormSet> {
    Verb::resolve(lemma)?.infinitive()
}

pub fn l_participle(lemma: &str, gender: Gender, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.l_participle(gender, number)
}

pub fn pronoun(lemma: &str, cell: PronounCell) -> Result<FormSet> {
    Pronoun::resolve(lemma)?.form(cell)
}

pub fn numeral(lemma: &str, cell: NumeralCell) -> Result<FormSet> {
    Numeral::resolve(lemma)?.form(cell)
}

pub fn determiner(lemma: &str, cell: AdjectiveCell) -> Result<FormSet> {
    Determiner::resolve(lemma)?.form(cell)
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet> {
    Participle::resolve(lemma)?.form(cell)
}

pub fn supine(lemma: &str) -> Result<FormSet> {
    let verb = Verb::resolve(lemma)?;
    Inflector::default().form_by_id(verb.id(), GrammarCell::Supine)
}

pub fn verbal_noun(lemma: &str, cell: NounCell) -> Result<FormSet> {
    let verb = Verb::resolve(lemma)?;
    Inflector::default().form_by_id(verb.id(), GrammarCell::VerbalNoun(cell))
}

/// Specialist stable-ID operations. These delegate to the same canonical cell
/// resolver as direct calls and resolved handles.
pub mod advanced {
    use super::*;

    pub fn form_by_id(id: &LexemeId, cell: GrammarCell) -> Result<FormSet> {
        Inflector::default().form_by_id(id, cell)
    }

    pub fn lookup_by_id(id: &LexemeId) -> Result<LexemeSummary> {
        Inflector::default().from_id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_api_uses_synodal_not_ocs_noun_endings() {
        let forms =
            noun("рабъ", Case::Dative, Number::Plural, Animacy::Animate).expect("curated noun");
        assert_eq!(forms.primary_text(), "рабомъ");
        assert_eq!(forms.target_recension(), Recension::SynodalRussian);
    }

    #[test]
    fn irregular_byti_is_exact_table_first() {
        let forms =
            present("быти", Person::First, Number::Singular).expect("exact irregular present");
        assert_eq!(forms.primary_text(), "єсмь");
        assert!(forms.variants()[0].evidence.len() == 1);
    }

    #[test]
    fn liturgical_profile_preserves_printed_form() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let verb = Verb::resolve_with("быти", inflector).expect("known verb");
        assert_eq!(
            verb.present(Person::First, Number::Singular)
                .expect("table cell")
                .primary_text(),
            "є҆́смь"
        );
    }

    #[test]
    fn paradigms_retain_failures() {
        let verb = Verb::resolve("быти").expect("known verb");
        let paradigm = verb.paradigm(FiniteTense::Imperfect);
        assert_eq!(paradigm.iter().count(), 9);
        assert_eq!(paradigm.failures().count(), 0);
        assert_eq!(
            paradigm
                .form(GrammarCell::FiniteVerb(FiniteVerbCell {
                    tense: FiniteTense::Imperfect,
                    person: Person::Third,
                    number: Number::Singular,
                }))
                .expect("irregular table")
                .variants()
                .len(),
            2
        );
    }

    #[test]
    fn rejected_alignment_remains_visible() {
        assert!(
            recension_alignments()
                .expect("alignment registry")
                .iter()
                .any(|mapping| mapping.status == "rejected")
        );
    }

    #[test]
    fn generation_policies_gate_inherited_class_evidence() {
        assert!(matches!(
            noun("градъ", Case::Dative, Number::Plural, Animacy::Inanimate),
            Err(Error::UnsupportedCell { .. })
        ));

        let inflector = Inflector::builder()
            .generation_policy(GenerationPolicy::Productive)
            .build();
        let noun = Noun::resolve_with("градъ", inflector).expect("target lexeme");
        let forms = noun
            .form(Case::Dative, Number::Plural, Animacy::Inanimate)
            .expect("reviewed inherited analysis");
        assert_eq!(forms.primary_text(), "градомъ");
        let variant = &forms.variants()[0];
        assert_eq!(variant.source_recension, Some(Recension::OldChurchSlavonic));
        assert!(variant.recension_mapping.is_some());
        assert!(matches!(
            variant.source,
            core::FormSource::InheritedPrediction { .. }
        ));
    }

    #[test]
    fn exact_registries_respect_animacy_and_any_gender_numerals() {
        let animate = pronoun(
            "той",
            PronounCell {
                case: Case::Accusative,
                number: Number::Singular,
                gender: Some(Gender::Masculine),
                person: None,
                animacy: Animacy::Animate,
            },
        )
        .expect("reviewed pronoun table");
        assert_eq!(animate.variants().len(), 1);
        assert_eq!(animate.primary_text(), "того");

        let inanimate = pronoun(
            "той",
            PronounCell {
                case: Case::Accusative,
                number: Number::Singular,
                gender: Some(Gender::Masculine),
                person: None,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("reviewed inanimate pronoun table");
        assert_eq!(inanimate.variants().len(), 1);
        assert_eq!(inanimate.primary_text(), "той");

        let numeral = numeral(
            "два",
            NumeralCell {
                kind: NumeralKind::Cardinal,
                case: Case::Genitive,
                number: Number::Dual,
                gender: Some(Gender::Feminine),
                animacy: Animacy::Inanimate,
            },
        )
        .expect("gender-independent table fallback");
        assert_eq!(numeral.variants().len(), 2);
    }

    #[test]
    fn determiner_handle_is_real_but_abstains_outside_exact_cells() {
        let nominative = determiner(
            "всѧкъ",
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            },
        )
        .expect("reviewed determiner cell");
        assert_eq!(nominative.primary_text(), "всѧкъ");

        assert!(matches!(
            determiner(
                "всѧкъ",
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            ),
            Err(Error::UnsupportedCell { .. })
        ));
    }

    #[test]
    fn expanded_productive_classes_and_personal_pronouns_are_available() {
        assert_eq!(
            noun("царь", Case::Genitive, Number::Plural, Animacy::Animate)
                .expect("reviewed soft masculine class")
                .primary_text(),
            "царей"
        );
        assert_eq!(
            adjective(
                "мꙋдръ",
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Comparative,
                }
            )
            .expect("reviewed comparison stem")
            .primary_text(),
            "мꙋдрѣйшїй"
        );
        assert_eq!(
            numeral(
                "первый",
                NumeralCell {
                    kind: NumeralKind::Ordinal,
                    case: Case::Genitive,
                    number: Number::Singular,
                    gender: Some(Gender::Masculine),
                    animacy: Animacy::Animate,
                }
            )
            .expect("productive ordinal")
            .primary_text(),
            "первагѡ"
        );
        assert_eq!(
            pronoun(
                "азъ",
                PronounCell {
                    case: Case::Genitive,
                    number: Number::Dual,
                    gender: None,
                    person: Some(Person::First),
                    animacy: Animacy::Inanimate,
                }
            )
            .expect("reviewed dual personal pronoun")
            .primary_text(),
            "наю"
        );
    }

    #[test]
    fn capabilities_report_actual_supported_systems() {
        let verb = Verb::resolve("быти").expect("known irregular verb");
        let capabilities = verb.capabilities();
        assert!(capabilities.present);
        assert!(capabilities.imperfect);
        assert!(capabilities.aorist);
        assert!(capabilities.imperative);
        assert!(capabilities.infinitive);
        assert!(!capabilities.l_participle);
        assert!(capabilities.participle);
        assert!(!capabilities.supine);
        assert!(!capabilities.verbal_noun);
        assert!(
            !verb
                .missing_metadata()
                .contains(&core::MetadataField::AccentClass)
        );

        let strict = Noun::resolve("градъ").expect("registered inherited-only noun");
        assert!(!strict.capabilities().productive_noun);
        let productive = Noun::resolve_with(
            "градъ",
            Inflector::builder()
                .generation_policy(GenerationPolicy::Productive)
                .build(),
        )
        .expect("registered inherited-only noun");
        assert!(productive.capabilities().productive_noun);

        let dati = Verb::resolve("дати").expect("reviewed archaic verb");
        assert!(dati.capabilities().participle);
        assert_eq!(
            dati.present(Person::Third, Number::Singular)
                .expect("reviewed simple-future table")
                .primary_text(),
            "дастъ"
        );
    }

    #[test]
    fn declined_participle_paradigm_uses_reviewed_principal_parts() {
        let participle = Participle::resolve("нести").expect("known verb");
        let paradigm = participle.paradigm(
            ParticipleTense::Present,
            ParticipleVoice::Active,
            AdjectiveForm::Long,
        );
        assert_eq!(paradigm.iter().count(), 126);
        assert_eq!(paradigm.attested().count(), 0);
        assert_eq!(paradigm.predicted().count(), 126);
        assert_eq!(paradigm.failures().count(), 0);
    }
}
