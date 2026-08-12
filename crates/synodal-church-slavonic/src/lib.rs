#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abbreviation;
mod handles;
mod inflector;
mod kernel;
mod paradigm;
pub mod phrases;
mod registry;
mod resolver;
mod spec;

pub use abbreviation::Abbreviation;
pub use handles::{Adjective, Capabilities, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use inflector::{Inflector, InflectorBuilder};
pub use paradigm::{Paradigm, ParadigmIdentity, ParadigmRow, ParadigmStatus};
pub use registry::{
    AccentParadigmSummary, AccentSummary, AlignmentSummary, ExactFormSummary,
    IrregularOverrideSummary, LexemeSummary, LexicalMetadataSummary, PartOfSpeech,
    PositionalRuleSummary, PrincipalPartSummary, RecensionConflictSummary,
    TransformationRuleSummary,
};
pub use spec::{
    AdjectiveSpec, DefectKind, DefectiveCell, LexemeSpec, NounSpec, SpecificationSource,
    SpecifiedForm, VerbSpec, VerbSpecBuilder,
};
pub use synodal_church_slavonic_core as core;
pub use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope,
    ActiveParticipleShortFormation, AdjectiveClass, AoristFormation, Aspect, AuthorityRole,
    BreathingMark, BreathingRule, ComparisonFormation, EpistemicRole, Evidence, EvidenceId,
    EvidenceKind, ImperativeFormation, ImperfectFormation, NounDeclension, ParticiplePrincipalPart,
    RuleId, SourceId, VerbConjugation,
};
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

/// Returns the complete reviewable metadata associated with one target lexeme.
pub fn lexical_metadata(id: &LexemeId) -> Result<LexicalMetadataSummary> {
    registry::lexical_metadata(id)
}

/// Reports the systems currently available for a stable target lexeme.
pub fn capabilities_by_id(id: &LexemeId, inflector: Inflector) -> Result<Capabilities> {
    handles::capabilities_by_id(id, inflector)
}

/// Lists metadata that prevents otherwise represented systems from running.
pub fn missing_metadata_by_id(id: &LexemeId) -> Result<Vec<MetadataField>> {
    handles::missing_metadata_by_id(id)
}

/// Returns the stable review/evaluation key for a typed grammar cell.
#[must_use]
pub fn grammar_cell_key(cell: GrammarCell) -> String {
    resolver::cell_key(cell)
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
        assert_eq!(forms.primary_text(), "рабѡмъ");
        assert_eq!(forms.target_recension(), Recension::SynodalRussian);
    }

    #[test]
    fn irregular_byti_is_exact_table_first() {
        let forms =
            present("быти", Person::First, Number::Singular).expect("exact irregular present");
        assert_eq!(forms.primary_text(), "єсмь");
        assert!(matches!(
            forms.variants()[0].source,
            FormSource::SynodalIrregularOverride { .. }
        ));
        assert!(
            forms.variants()[0]
                .evidence
                .iter()
                .any(|evidence| { evidence.kind == core::EvidenceKind::ReviewedIrregularOverride })
        );
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
    fn registry_reusable_accent_paradigm_covers_non_exact_cells() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let adjective = Adjective::resolve_with("мꙋдръ", inflector).expect("known adjective");
        for case in [Case::Genitive, Case::Dative, Case::Instrumental] {
            let forms = adjective
                .form(AdjectiveCell {
                    case,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("accent paradigm cell");
            assert!(forms.primary_text().starts_with("мꙋ́др"));
            assert!(
                forms
                    .primary()
                    .evidence
                    .iter()
                    .any(|evidence| { evidence.kind == core::EvidenceKind::AccentParadigm })
            );
        }
        let exact = adjective
            .form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            })
            .expect("exact accent override");
        assert_eq!(exact.primary_text(), "мꙋ́дрый");
        assert!(
            exact
                .primary()
                .rule_trace
                .steps()
                .iter()
                .any(|step| { step.rule.as_str() == "SYN-ACCENT-REGISTRY" })
        );
    }

    #[test]
    fn partial_registered_irregular_uses_regular_background_only_outside_override() {
        let noun = Noun::resolve("сынъ").expect("reviewed partially irregular noun");
        let irregular = noun
            .form(Case::Dative, Number::Singular, Animacy::Animate)
            .expect("irregular override");
        assert_eq!(irregular.primary_text(), "сынови");
        assert!(matches!(
            irregular.primary().source,
            FormSource::SynodalIrregularOverride { .. }
        ));

        let regular = noun
            .form(Case::Genitive, Number::Dual, Animacy::Animate)
            .expect("explicitly classed regular background");
        assert_eq!(regular.primary_text(), "сынꙋ");
        assert!(matches!(
            regular.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));
    }

    #[test]
    fn byti_future_is_an_exact_normative_table() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let verb = Verb::resolve_with("быти", inflector).expect("known verb");
        let forms = verb
            .future(Person::Third, Number::Singular)
            .expect("reviewed simple future");
        assert_eq!(forms.primary_text(), "бꙋ́детъ");
        assert!(matches!(
            forms.variants()[0].source,
            core::FormSource::SynodalNormativeGeneration { .. }
        ));

        let unsupported = Verb::resolve("нести")
            .expect("known verb")
            .future(Person::Third, Number::Singular);
        assert!(matches!(unsupported, Err(Error::UnsupportedCell { .. })));
    }

    #[test]
    fn third_person_pronoun_preserves_case_distinguishing_accents() {
        let pronoun = Pronoun::resolve_with(
            "онъ",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed third-person pronoun");
        let genitive = pronoun
            .form(PronounCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Some(Gender::Masculine),
                person: Some(Person::Third),
                animacy: Animacy::Animate,
            })
            .expect("reviewed genitive");
        assert_eq!(genitive.primary_text(), "є҆гѡ̀");
    }

    #[test]
    fn positional_preposition_variants_keep_distinct_exact_cells() {
        let inflector = Inflector::default();
        let id = LexemeId::from("synodal:preposition:wikt-77998a1b179f");
        assert_eq!(
            inflector
                .form_by_id(&id, GrammarCell::Indeclinable)
                .expect("base preposition")
                .primary_text(),
            "къ"
        );
        assert_eq!(
            inflector
                .form_by_id(&id, GrammarCell::LexicalForm)
                .expect("reviewed positional variant")
                .primary_text(),
            "ко"
        );
    }

    #[test]
    fn paradigms_retain_failures() {
        let verb = Verb::resolve("быти").expect("known verb");
        let paradigm = verb.paradigm(FiniteTense::Imperfect);
        assert_eq!(paradigm.iter().count(), 9);
        assert_eq!(paradigm.failures().count(), 0);
        let third_singular = paradigm
            .form(GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Imperfect,
                person: Person::Third,
                number: Number::Singular,
            }))
            .expect("irregular table");
        assert_eq!(third_singular.variants().len(), 3);
        assert_eq!(
            third_singular
                .variants()
                .iter()
                .filter(|variant| variant.is_attested())
                .count(),
            1
        );
        assert_eq!(
            third_singular
                .variants()
                .iter()
                .filter(|variant| variant.is_predicted())
                .count(),
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
            noun("градъ", Case::Dative, Number::Dual, Animacy::Inanimate),
            Err(Error::UnsupportedCell { .. })
        ));

        let inflector = Inflector::builder()
            .generation_policy(GenerationPolicy::Productive)
            .build();
        let noun = Noun::resolve_with("градъ", inflector).expect("target lexeme");
        let forms = noun
            .form(Case::Dative, Number::Dual, Animacy::Inanimate)
            .expect("reviewed inherited analysis");
        assert_eq!(forms.primary_text(), "градома");
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
        assert!(capabilities.l_participle);
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
        assert_eq!(paradigm.iter().count(), 72);
        assert_eq!(paradigm.attested().count(), 0);
        assert_eq!(paradigm.predicted().count(), 72);
        assert_eq!(paradigm.failures().count(), 0);
    }

    #[test]
    fn v04_exact_families_abstain_outside_reviewed_cells() {
        let strict = Inflector::builder()
            .generation_policy(GenerationPolicy::Strict)
            .build();
        let ves = LexemeId::from("synodal:determiner:ves");
        let unsupported_dual = GrammarCell::Determiner(AdjectiveCell {
            case: Case::Nominative,
            number: Number::Dual,
            gender: Gender::Masculine,
            animacy: Animacy::Animate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
        });
        assert!(matches!(
            strict.form_by_id(&ves, unsupported_dual),
            Err(Error::UnsupportedCell { .. })
        ));

        let reshchi = LexemeId::from("synodal:verb:wikt-06af096688df");
        assert!(matches!(
            strict.form_by_id(
                &reshchi,
                GrammarCell::FiniteVerb(FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::Third,
                    number: Number::Singular,
                })
            ),
            Err(Error::UnsupportedCell { .. })
        ));
        assert!(matches!(
            strict.form_by_id(
                &reshchi,
                GrammarCell::Participle(ParticipleCell {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Active,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Feminine,
                        animacy: Animacy::Animate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                })
            ),
            Err(Error::UnsupportedCell { .. })
        ));
    }
}
