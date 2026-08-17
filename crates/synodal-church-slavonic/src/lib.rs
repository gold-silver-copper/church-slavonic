#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abbreviation;
mod handles;
mod inflector;
mod kernel;
pub mod numeral_phrases;
mod paradigm;
pub mod phrases;
mod provider;
mod registry;
mod resolver;
mod spec;

pub use abbreviation::Abbreviation;
pub use handles::{Adjective, Capabilities, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use inflector::{Inflector, InflectorBuilder};
pub use numeral_phrases::{
    CardinalPhraseAnalysis, CompoundNumeralCell, NumeralComposition, NumeralGovernment,
    NumeralNounPosition, OrdinalPhraseAnalysis, RealizedCardinal, RealizedOrdinal, fraction,
    fractional_cardinal_parts, fractional_half_tenth_parts, fractional_ordinal_parts,
    multiplicative_krat, repeated_distributive,
};
pub use paradigm::{Paradigm, ParadigmIdentity, ParadigmRow, ParadigmStatus};
pub use provider::{
    BatchLexeme, BatchRequest, BatchResult, BatchRow, InMemoryLexemeProvider, LexemeProvider,
    Lexicon, ProviderLexeme, StaticLexemeProvider,
};
pub use registry::{
    AccentParadigmSummary, AccentSummary, AlignmentSummary, ExactFormSummary,
    IrregularOverrideSummary, IrregularVerbInventorySummary, LexemeSummary, LexicalMetadataSummary,
    NounRestrictionSummary, PartOfSpeech, PositionalRuleSummary, PrincipalPartSummary,
    RecensionConflictSummary, TransformationRuleSummary,
};
pub use spec::{
    AdjectiveSpec, DefectKind, DefectiveCell, DeterminerSpec, LexemeSpec, NounSpec, NumeralSpec,
    PronounSpec, SpecificationSource, SpecifiedForm, VerbSpec, VerbSpecBuilder,
};
pub use synodal_church_slavonic_core as core;
pub use synodal_church_slavonic_core::{
    AccentEnclitic, AccentEnvironment, AccentMark, AccentParadigm, AccentPlacement, AccentRule,
    AccentScope, ActiveParticipleShortFormation, AdjectiveClass, AoristFormation, Aspect,
    AuthorityRole, BreathingMark, BreathingRule, ComparisonFormation, EncliticParticle,
    EpistemicRole, Evidence, EvidenceId, EvidenceKind, ImperativeFormation, ImperfectFormation,
    NounDeclension, NounNumberInventory, NumeralDeclension, NumeralLexeme, NumeralNumberInventory,
    ParticiplePrincipalPart, PresentPrincipalParts, RuleId, ShortMasculineStemFormation, SourceId,
    VerbConjugation,
};
pub use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AdverbialParticipleFormation, AnalyticConstruction, Animacy,
    Case, CollationKey, CollationProfile, CollationStrength, Comparison, CompoundAuxiliaryOrder,
    CompoundFutureAuxiliary, ConditionalCopulaOrder, ConditionalFormation, Confidence,
    CopulaOmissionContext, CyrillicNumeral, DeterminerDeclension, DeterminerLexeme,
    DeterminerNumberInventory, Error, ErrorCode, FiniteTense, FiniteVerbCell, FormSet, FormSource,
    Gender, GenerationPolicy, GrammarCell, ImperativeCell, InitialPresentation, LParticipleCell,
    LetterOccurrence, LexemeId, Loss, MetadataField, ModalConditionalAuxiliary,
    NegativePronounBase, NounCell, NounLexeme, Number, NumeralCell, NumeralKind,
    OptativeFiniteSystem, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice,
    PassiveAgentGovernment, PassiveFormation, PerfectFormation, PeriphrasticFormation,
    PeriphrasticSemiAuxiliary, PeriphrasticTenseFormation, Person, PhraseFormation, PhraseOrder,
    PhraseRole, PhraseToken, PluperfectFormation, PositionalOperation, PositionalParadigm,
    PositionalReplacement, PositionalRule, PronounCell, PronounCliticProsody, PronounDeclension,
    PronounEnvironment, PronounFormSelection, PronounNumberInventory, PronounPostpositive,
    PronounPrefix, RealizedPhrase, Recension, RenderedText, Result, SynodalWord,
    TransliterationScheme, VariantPolicy, VerbSystem, VerbalNounFormation, VerbalNounPrincipalPart,
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
    cell.key()
}

/// Returns canonical and compatible wildcard registry keys in lookup order.
///
/// Dictionaries and other reverse-analysis layers should use this function
/// instead of reconstructing the facade's exact-form compatibility rules.
#[must_use]
pub fn grammar_cell_registry_keys(cell: GrammarCell) -> Vec<String> {
    resolver::exact_lookup_keys(cell)
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

/// Returns all 98 verb entries in Alypy §104's source order.
pub fn irregular_verb_inventory() -> Result<Vec<IrregularVerbInventorySummary>> {
    registry::irregular_verb_inventory()
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

pub fn future(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.future(person, number)
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
    fn registry_key_compatibility_is_canonical_and_ordered() {
        let pronoun = GrammarCell::Pronoun(PronounCell {
            case: Case::Accusative,
            number: Number::Plural,
            gender: Some(Gender::Feminine),
            person: Some(Person::Third),
            animacy: Animacy::Animate,
        });
        assert_eq!(
            grammar_cell_registry_keys(pronoun),
            [
                "pronoun:accusative:plural:feminine:third:animate",
                "pronoun:accusative:plural:feminine:third:any",
            ]
        );

        let numeral = GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Ordinal,
            case: Case::Accusative,
            number: Number::Plural,
            gender: Some(Gender::Masculine),
            animacy: Animacy::Animate,
        });
        assert_eq!(
            grammar_cell_registry_keys(numeral),
            [
                "numeral:ordinal:accusative:plural:masculine:animate",
                "numeral:ordinal:accusative:plural:any:animate",
                "numeral:ordinal:accusative:plural:masculine:any",
                "numeral:ordinal:accusative:plural:any:any",
            ]
        );
    }

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
    fn exact_attestations_preserve_distinct_evidence_provenance() {
        let verb =
            Verb::from_id(&LexemeId::from("synodal:verb:v06-vzeti")).expect("reviewed exact verb");
        let forms = verb
            .aorist(Person::Third, Number::Singular)
            .expect("reviewed exact aorist form");
        let variant = forms.primary();
        assert!(matches!(
            &variant.source,
            FormSource::SynodalAttestation { evidence }
                if evidence.as_str() == "v06-manual-target-vze"
        ));
        assert_eq!(
            variant
                .evidence
                .iter()
                .map(|evidence| evidence.id.as_str())
                .collect::<Vec<_>>(),
            [
                "v06-manual-semantic-vzeti",
                "v06-manual-alypy-vzeti",
                "v06-manual-target-vze",
            ]
        );
        let inherited = &variant.evidence[0];
        assert_eq!(
            inherited.source.as_str(),
            "english-wiktionary-ocs-kaikki-2026-08-07"
        );
        assert_eq!(inherited.source_recension, Recension::OldChurchSlavonic);
        assert_eq!(
            inherited.epistemic_role,
            core::EpistemicRole::InheritedOcsEvidence
        );
        let target = &variant.evidence[2];
        assert_eq!(target.source.as_str(), "ponomar-elizabeth-bible-2026-08-09");
        assert_eq!(target.source_recension, Recension::SynodalRussian);
        assert_eq!(
            target.epistemic_role,
            core::EpistemicRole::ExactSynodalAttestation
        );

        let plural = Inflector::default()
            .form_by_id(
                &LexemeId::from("synodal:verb:v06-c83e3264f4da24ce"),
                GrammarCell::Participle(ParticipleCell {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Passive,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Plural,
                        gender: Gender::Neuter,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                }),
            )
            .expect("reviewed cell-specific plural attestation");
        assert!(matches!(
            &plural.primary().source,
            FormSource::SynodalAttestation { evidence }
                if evidence.as_str() == "v06-target-fae0dde305c6fff8"
        ));
    }

    #[test]
    fn unified_registered_verb_paradigms_preserve_irregular_systems() {
        let verb = Verb::resolve("быти").expect("registered irregular verb");
        let present = verb.system_paradigm(VerbSystem::Finite(FiniteTense::Present));
        assert_eq!(present.iter().count(), 9);
        assert_eq!(present.failures().count(), 0);
        assert_eq!(
            present
                .with_status(ParadigmStatus::IrregularOverride)
                .count(),
            8
        );
        assert_eq!(present.with_status(ParadigmStatus::Attested).count(), 1);

        let future = verb.system_paradigm(VerbSystem::Finite(FiniteTense::Future));
        assert_eq!(future.successes().count(), 9);
        assert_eq!(future.failures().count(), 0);

        let nesti = Verb::resolve("нести").expect("productive verb");
        assert!(
            nesti
                .missing_principal_parts(VerbSystem::Finite(FiniteTense::Present))
                .expect("registered metadata")
                .is_empty()
        );
    }

    #[test]
    fn reviewed_regular_verbs_expose_complete_independent_finite_systems() {
        for lemma in ["нести", "писати", "любити"] {
            let verb = Verb::resolve(lemma).expect("reviewed productive verb");
            for tense in [
                FiniteTense::Present,
                FiniteTense::Imperfect,
                FiniteTense::Aorist,
            ] {
                assert!(
                    verb.missing_principal_parts(VerbSystem::Finite(tense))
                        .expect("registered metadata")
                        .is_empty(),
                    "{lemma} {tense:?}"
                );
                let paradigm = verb.system_paradigm(VerbSystem::Finite(tense));
                assert_eq!(paradigm.successes().count(), 9, "{lemma} {tense:?}");
                assert_eq!(paradigm.failures().count(), 0, "{lemma} {tense:?}");
            }
            assert!(
                verb.missing_principal_parts(VerbSystem::Imperative)
                    .expect("registered imperative metadata")
                    .is_empty()
            );
            let imperative = verb.system_paradigm(VerbSystem::Imperative);
            assert_eq!(imperative.successes().count(), 6, "{lemma} imperative");
            assert_eq!(
                imperative
                    .with_error_code(ErrorCode::HistoricallyInvalidCell)
                    .count(),
                3,
                "{lemma} imperative"
            );
            assert!(
                verb.missing_principal_parts(VerbSystem::LParticiple)
                    .expect("registered l-participle metadata")
                    .is_empty()
            );
        }

        let nesti = Verb::resolve("нести").expect("reviewed participial verb");
        for tense in ParticipleTense::ALL {
            for voice in ParticipleVoice::ALL {
                for form in AdjectiveForm::ALL {
                    assert!(
                        nesti
                            .missing_principal_parts(VerbSystem::Participle { tense, voice, form })
                            .expect("registered participle metadata")
                            .is_empty(),
                        "нести {tense:?} {voice:?} {form:?}"
                    );
                }
            }
        }
        let pisati = Verb::resolve("писати").expect("reviewed finite verb");
        assert_eq!(
            pisati
                .missing_principal_parts(VerbSystem::Participle {
                    tense: ParticipleTense::Present,
                    voice: ParticipleVoice::Passive,
                    form: AdjectiveForm::Short,
                })
                .expect("missing participle diagnostics"),
            vec![MetadataField::ParticipleStem]
        );
    }

    #[test]
    fn sotvoriti_promotes_reviewed_principal_parts_to_complete_typed_systems() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let verb = Verb::resolve_with("сотворити", inflector).expect("reviewed perfective verb");

        assert_eq!(
            verb.future(Person::First, Number::Plural)
                .expect("productive future")
                .primary_text(),
            "сотвори́мъ"
        );
        assert_eq!(
            verb.aorist(Person::Second, Number::Plural)
                .expect("productive aorist")
                .primary_text(),
            "сотвори́сте"
        );
        assert_eq!(
            verb.imperative(Person::First, Number::Plural)
                .expect("productive imperative")
                .primary_text(),
            "сотвори́мъ"
        );
        assert_eq!(
            verb.l_participle(Gender::Feminine, Number::Singular)
                .expect("productive l-participle")
                .primary_text(),
            "сотвори́ла"
        );

        assert_eq!(
            verb.future(Person::Second, Number::Plural)
                .expect("exact ending-stressed future")
                .primary_text(),
            "сотворитѐ"
        );
        assert_eq!(
            verb.imperative(Person::Second, Number::Plural)
                .expect("exact imperative")
                .primary_text(),
            "сотвори́те"
        );

        for system in [
            VerbSystem::Finite(FiniteTense::Future),
            VerbSystem::Finite(FiniteTense::Aorist),
            VerbSystem::Imperative,
            VerbSystem::LParticiple,
        ] {
            assert!(
                verb.missing_principal_parts(system)
                    .expect("typed metadata query")
                    .is_empty(),
                "{system:?}"
            );
        }
        assert_eq!(
            verb.system_paradigm(VerbSystem::Finite(FiniteTense::Future))
                .successes()
                .count(),
            9
        );
        assert_eq!(
            verb.system_paradigm(VerbSystem::Finite(FiniteTense::Aorist))
                .successes()
                .count(),
            9
        );
        assert_eq!(
            verb.system_paradigm(VerbSystem::Imperative)
                .successes()
                .count(),
            6
        );
        assert_eq!(
            verb.system_paradigm(VerbSystem::LParticiple)
                .successes()
                .count(),
            9
        );
    }

    #[test]
    fn additional_fourth_declension_nouns_are_productive_and_bounded() {
        let otrocha = Noun::resolve("ѻтроча").expect("registered at-stem noun");
        assert_eq!(otrocha.paradigm(Animacy::Inanimate).failures().count(), 0);
        assert_eq!(
            otrocha
                .form(Case::Genitive, Number::Singular, Animacy::Inanimate)
                .expect("extended oblique stem")
                .primary_text(),
            "ѻтрочате"
        );

        let svekry = Noun::resolve("свекры").expect("registered ov-stem noun");
        assert_eq!(
            svekry
                .form(Case::Accusative, Number::Plural, Animacy::Animate)
                .expect("ordered animate variants")
                .texts()
                .collect::<Vec<_>>(),
            ["свекровей", "свекрови"]
        );

        let kamen = Noun::resolve("камень").expect("registered en-stem noun");
        assert_eq!(kamen.id().as_str(), "synodal:noun:v07-c27905de175a0cde");
        assert!(matches!(
            kamen
                .form(Case::Nominative, Number::Singular, Animacy::Inanimate)
                .expect("reviewed exact citation")
                .primary()
                .source,
            FormSource::SynodalAttestation { .. }
        ));
        let ordinary_plural = kamen
            .form(Case::Nominative, Number::Plural, Animacy::Inanimate)
            .expect("ordinary plural");
        assert_eq!(
            ordinary_plural.texts().collect::<Vec<_>>(),
            ["камєни", "каменїѧ"]
        );
        assert!(ordinary_plural.texts().all(|form| form != "каменїе"));

        // The dative plural used to fail for want of an accent contract. It
        // is directly printed in the source partition at Ezek.6.3 in both
        // editions, in an unambiguous dative chain
        // (`гора́мъ и҆ холмѡ́мъ, и҆ ка́менємъ и҆ де́бремъ`), and the genitive
        // plural `ка́менїй` is printed 16 times, so a reviewed
        // `noun:dual,plural` paradigm now licenses them. The cells are
        // realised by that reviewed contract, not by an accentless fallback.
        // The paradigm is not exhaustive: Ex.28.12 also prints a kamora
        // nominative plural `ка̑мени` that this contract cannot produce.
        let liturgical = Noun::resolve_with(
            "камень",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("registered noun");
        let dative_plural = liturgical
            .form(Case::Dative, Number::Plural, Animacy::Inanimate)
            .expect("reviewed fixed-stem accent contract");
        assert_eq!(dative_plural.texts().collect::<Vec<_>>(), ["ка́менємъ"]);
        assert!(matches!(
            dative_plural.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));
        assert_eq!(
            liturgical
                .form(Case::Genitive, Number::Plural, Animacy::Inanimate)
                .expect("reviewed fixed-stem accent contract")
                .texts()
                .collect::<Vec<_>>(),
            ["ка́менїй"]
        );
    }

    #[test]
    fn otrocha_has_a_complete_mobile_at_stem_accent_paradigm() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let otrocha =
            Noun::resolve_with("ѻтроча", inflector).expect("reviewed fourth-neuter child noun");

        let nominative = otrocha
            .form(Case::Nominative, Number::Singular, Animacy::Inanimate)
            .expect("exact singular nominative variants");
        assert!(matches!(
            nominative.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));
        assert_eq!(nominative.texts().collect::<Vec<_>>(), ["ѻ҆троча̀", "Ѻ҆троча́"]);

        for (case, number, expected) in [
            (Case::Accusative, Number::Singular, "ѻ҆троча̀"),
            (Case::Genitive, Number::Singular, "ѻ҆троча́те"),
            (Case::Dative, Number::Singular, "ѻ҆троча́ти"),
            (Case::Instrumental, Number::Singular, "ѻ҆троча́темъ"),
            (Case::Nominative, Number::Plural, "ѻ҆троча́та"),
            (Case::Genitive, Number::Plural, "ѻ҆троча́тъ"),
        ] {
            assert_eq!(
                otrocha
                    .form(case, number, Animacy::Inanimate)
                    .expect("complete reviewed fourth-neuter cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(otrocha.paradigm(Animacy::Inanimate).failures().count(), 0);
        assert!(matches!(
            otrocha.form(Case::Dative, Number::Singular, Animacy::Animate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn alpy_42_44_irregular_noun_families_have_exact_first_complete_backgrounds() {
        let eye = Noun::resolve("око").expect("reviewed eye identity");
        assert_eq!(
            eye.form(Case::Nominative, Number::Dual, Animacy::Inanimate)
                .expect("reviewed paired dual")
                .primary_text(),
            "очи"
        );
        assert_eq!(
            eye.form(Case::Nominative, Number::Plural, Animacy::Inanimate)
                .expect("extended plural background")
                .primary_text(),
            "очеса"
        );

        let ear = Noun::resolve("ѹхо").expect("reviewed ear identity");
        assert_eq!(
            ear.form(Case::Genitive, Number::Dual, Animacy::Inanimate)
                .expect("suffixless paired dual")
                .primary_text(),
            "ушїю"
        );
        assert_eq!(ear.paradigm(Animacy::Inanimate).failures().count(), 0);

        let church = Noun::resolve("церковь").expect("reviewed church identity");
        let exact = church
            .form(Case::Genitive, Number::Singular, Animacy::Inanimate)
            .expect("target-attested exact form");
        assert_eq!(exact.primary_text(), "церкви");
        assert!(matches!(
            exact.primary().source,
            FormSource::SynodalAttestation { .. }
        ));
        assert_eq!(
            church
                .form(Case::Genitive, Number::Dual, Animacy::Inanimate)
                .expect("full-stem dual background")
                .primary_text(),
            "цєрковїю"
        );
        assert_eq!(
            church
                .form(Case::Dative, Number::Plural, Animacy::Inanimate)
                .expect("syncopated plural background")
                .primary_text(),
            "церквамъ"
        );

        let love = Noun::resolve("любовь").expect("one unified love identity");
        assert_eq!(love.id().as_str(), "synodal:noun:lyubov");
        assert_eq!(
            love.form(Case::Genitive, Number::Singular, Animacy::Inanimate)
                .expect("ordered exact variants")
                .texts()
                .collect::<Vec<_>>(),
            ["любве", "любве", "любви"]
        );
        assert_eq!(
            love.form(Case::Genitive, Number::Plural, Animacy::Inanimate)
                .expect("bounded productive background")
                .primary_text(),
            "любвей"
        );

        let daughter = Noun::resolve("дщерь").expect("reviewed daughter identity");
        assert_eq!(
            daughter
                .form(Case::Nominative, Number::Singular, Animacy::Animate)
                .expect("historical citation")
                .primary_text(),
            "дщи"
        );
        assert_eq!(
            daughter
                .form(Case::Genitive, Number::Plural, Animacy::Animate)
                .expect("complete daughter background")
                .texts()
                .collect::<Vec<_>>(),
            ["дщерей"]
        );

        for (lemma, expected) in [
            ("кровь", "кровей"),
            ("пламень", "пламенїй"),
            ("ремень", "ременїй"),
            ("кремень", "кременїй"),
            ("корень", "коренїй"),
        ] {
            let noun = Noun::resolve(lemma).expect("Alypy §44 named family member");
            assert_eq!(
                noun.paradigm(Animacy::Inanimate).failures().count(),
                0,
                "{lemma}"
            );
            assert_eq!(
                noun.form(Case::Genitive, Number::Plural, Animacy::Inanimate)
                    .expect("complete named-family background")
                    .primary_text(),
                expected,
                "{lemma}"
            );
        }

        let brethren = Noun::resolve("братїѧ").expect("distinct collective identity");
        assert_eq!(
            brethren
                .form(Case::Genitive, Number::Singular, Animacy::Animate)
                .expect("collective singular")
                .primary_text(),
            "братїи"
        );
        assert!(matches!(
            brethren.form(Case::Genitive, Number::Plural, Animacy::Animate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn alpy_37_44_remaining_named_noun_families_route_through_the_facade() {
        for (lemma, case, number, animacy, expected) in [
            (
                "галїлеанинъ",
                Case::Nominative,
                Number::Plural,
                Animacy::Animate,
                vec!["галїлеане"],
            ),
            (
                "ꙋдъ",
                Case::Instrumental,
                Number::Plural,
                Animacy::Inanimate,
                vec!["ꙋды", "ꙋдми", "ꙋдами", "ꙋдесы"],
            ),
            (
                "свидѣтель",
                Case::Nominative,
                Number::Plural,
                Animacy::Animate,
                vec!["свидѣтели", "свидѣтеле", "свидѣтелїе"],
            ),
            (
                "соборище",
                Case::Locative,
                Number::Plural,
                Animacy::Inanimate,
                vec!["соборищахъ", "соборищихъ", "соборищехъ"],
            ),
            (
                "чꙋдо",
                Case::Genitive,
                Number::Singular,
                Animacy::Inanimate,
                vec!["чꙋдесе", "чꙋда"],
            ),
            (
                "день",
                Case::Dative,
                Number::Singular,
                Animacy::Inanimate,
                vec!["дни", "дневи"],
            ),
            (
                "адѡнаі",
                Case::Instrumental,
                Number::Dual,
                Animacy::Animate,
                vec!["адѡнаі"],
            ),
            (
                "исаїа",
                Case::Instrumental,
                Number::Singular,
                Animacy::Animate,
                vec!["исаїемъ"],
            ),
            (
                "молнїѧ",
                Case::Nominative,
                Number::Plural,
                Animacy::Inanimate,
                vec!["молнїѧ"],
            ),
            (
                "кормчїй",
                Case::Dative,
                Number::Singular,
                Animacy::Animate,
                vec!["кормчїю"],
            ),
            (
                "пастырь",
                Case::Nominative,
                Number::Plural,
                Animacy::Animate,
                vec!["пастыри", "пастырїе"],
            ),
        ] {
            let noun = Noun::resolve(lemma).expect("normative named-family identity");
            assert_eq!(
                noun.form(case, number, animacy)
                    .expect("complete productive cell")
                    .texts()
                    .collect::<Vec<_>>(),
                expected,
                "{lemma}"
            );
            assert_eq!(noun.paradigm(animacy).failures().count(), 0, "{lemma}");
        }

        let lord = Noun::resolve("господь").expect("reviewed lord identity");
        let dative = lord
            .form(Case::Dative, Number::Singular, Animacy::Animate)
            .expect("normative dative variants");
        assert!(matches!(
            dative.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));
        assert_eq!(dative.texts().collect::<Vec<_>>(), ["господꙋ", "господеви"]);
        assert!(matches!(
            lord.form(Case::Vocative, Number::Singular, Animacy::Animate)
                .expect("reviewed vocative")
                .primary()
                .source,
            FormSource::SynodalAttestation { .. }
        ));
        assert_eq!(lord.paradigm(Animacy::Animate).failures().count(), 0);
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
    fn zlyi_mobile_o_and_comparison_stems_cover_productive_paradigms() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let adjective = Adjective::resolve_with("ѕлый", inflector).expect("registered adjective");

        let form = |cell| {
            adjective
                .form(cell)
                .expect("productive adjective cell")
                .primary_text()
                .to_owned()
        };
        assert_eq!(
            form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
            "ѕо́лъ"
        );
        assert_eq!(
            form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
            "ѕлы́й"
        );
        for (case, animacy) in [
            (Case::Nominative, Animacy::Inanimate),
            (Case::Accusative, Animacy::Inanimate),
        ] {
            assert_eq!(
                form(AdjectiveCell {
                    case,
                    number: Number::Plural,
                    gender: Gender::Neuter,
                    animacy,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                }),
                "ѕла̑ѧ"
            );
        }
        assert_eq!(
            form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Comparative,
            }),
            "ѕлѣ́йшїй"
        );

        assert_eq!(
            adjective.paradigm(AdjectiveForm::Short).failures().count(),
            0
        );
        assert_eq!(
            adjective.paradigm(AdjectiveForm::Long).failures().count(),
            0
        );
    }

    #[test]
    fn blagii_uses_the_complete_cell_conditioned_alypy_57_accent_paradigm() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let adjective = Adjective::resolve_with("благъ", inflector).expect("registered adjective");
        let form = |case, number, gender, animacy| {
            adjective
                .form(AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("productive §57 adjective cell")
                .primary_text()
                .to_owned()
        };
        use Case::{
            Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins,
            Locative as Loc, Nominative as Nom,
        };
        use Gender::{Feminine as F, Masculine as M, Neuter as N};
        use Number::{Dual as Du, Plural as Pl, Singular as Sg};

        let expected = [
            (Nom, Sg, M, Animacy::Inanimate, "благї́й"),
            (Loc, Sg, M, Animacy::Inanimate, "бла́зѣмъ"),
            (Nom, Du, M, Animacy::Inanimate, "блага̑ѧ"),
            (Nom, Du, F, Animacy::Inanimate, "блазѣ́и"),
            (Gen, Du, N, Animacy::Inanimate, "благꙋ̑ю"),
            (Dat, Du, M, Animacy::Inanimate, "благи́ма"),
            (Nom, Pl, M, Animacy::Inanimate, "блазї́и"),
            (Nom, Pl, F, Animacy::Inanimate, "благї̑ѧ"),
            (Nom, Pl, N, Animacy::Inanimate, "блага̑ѧ"),
            (Gen, Pl, F, Animacy::Inanimate, "благи́хъ"),
            (Dat, Pl, N, Animacy::Inanimate, "благи̑мъ"),
            (Acc, Pl, M, Animacy::Animate, "благї́ѧ"),
            (Acc, Pl, F, Animacy::Inanimate, "благї̑ѧ"),
            (Acc, Pl, N, Animacy::Inanimate, "блага̑ѧ"),
            (Ins, Pl, M, Animacy::Inanimate, "благи́ми"),
        ];
        let actual = expected
            .iter()
            .map(|(case, number, gender, animacy, _)| form(*case, *number, *gender, *animacy))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            expected
                .iter()
                .map(|(_, _, _, _, expected)| (*expected).to_owned())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            adjective.paradigm(AdjectiveForm::Long).failures().count(),
            0
        );
    }

    #[test]
    fn mertv_has_complete_fixed_stem_short_and_long_paradigms() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let adjective = Adjective::resolve_with("мертвъ", inflector).expect("registered adjective");
        let form = |case, number, gender, animacy, adjective_form| {
            adjective
                .form(AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy,
                    form: adjective_form,
                    comparison: Comparison::Positive,
                })
                .expect("productive adjective cell")
                .primary_text()
                .to_owned()
        };

        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                AdjectiveForm::Short,
            ),
            "ме́ртвъ"
        );
        assert_eq!(
            form(
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
                AdjectiveForm::Short,
            ),
            "ме́ртва"
        );
        assert_eq!(
            form(
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                AdjectiveForm::Long,
            ),
            "ме́ртвыхъ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                AdjectiveForm::Long,
            ),
            "ме́ртвїи"
        );
        assert_eq!(
            adjective.paradigm(AdjectiveForm::Short).failures().count(),
            0
        );
        assert_eq!(
            adjective.paradigm(AdjectiveForm::Long).failures().count(),
            0
        );
    }

    #[test]
    fn dusha_exact_cells_overlay_a_complete_mobile_accent_paradigm() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::resolve_with("дꙋша", inflector).expect("upgraded stable noun identity");

        for (case, number, expected) in [
            (Case::Genitive, Number::Singular, "дꙋшѝ"),
            (Case::Accusative, Number::Plural, "дꙋ́шы"),
            (Case::Genitive, Number::Plural, "дꙋ́шъ"),
            (Case::Locative, Number::Plural, "дꙋша́хъ"),
            (Case::Instrumental, Number::Plural, "дꙋша́ми"),
            (Case::Nominative, Number::Plural, "дꙋ́ши"),
            (Case::Nominative, Number::Dual, "дꙋши̑"),
            (Case::Vocative, Number::Singular, "дꙋшѐ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Inanimate)
                    .expect("complete mixed-declension cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
    }

    #[test]
    fn adonai_is_a_fully_accented_indeclinable_noun() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::resolve_with("адѡнаі", inflector).expect("classed divine title");

        for number in Number::ALL {
            for case in Case::ALL {
                assert_eq!(
                    noun.form(case, number, Animacy::Animate)
                        .expect("complete indeclinable cell")
                        .primary_text(),
                    "а҆дѡнаі̀",
                    "{case:?} {number:?}"
                );
            }
        }
        assert_eq!(noun.paradigm(Animacy::Animate).failures().count(), 0);
    }

    #[test]
    fn zhena_preserves_wide_e_plural_and_narrow_e_genitive_surfaces() {
        let noun = Noun::resolve_with(
            "жена",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("classed feminine noun");

        assert_eq!(
            noun.form(Case::Nominative, Number::Plural, Animacy::Animate)
                .expect("reviewed nominative plural")
                .primary_text(),
            "жєны̀"
        );
        assert_eq!(
            noun.form(Case::Accusative, Number::Plural, Animacy::Animate)
                .expect("reviewed accusative plural")
                .primary_text(),
            "жєны̀"
        );
        assert_eq!(
            noun.form(Case::Genitive, Number::Singular, Animacy::Animate)
                .expect("reviewed genitive singular")
                .primary_text(),
            "жены̀"
        );
    }

    #[test]
    fn svidenie_has_a_complete_fixed_accent_soft_ie_paradigm() {
        let noun = Noun::resolve_with(
            "свидѣнїе",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed testimony noun");

        for (case, number, expected) in [
            (Case::Accusative, Number::Singular, "свидѣ́нїе"),
            (Case::Genitive, Number::Singular, "свидѣ́нїѧ"),
            (Case::Nominative, Number::Plural, "свидѣ́нїѧ"),
            (Case::Dative, Number::Plural, "свидѣ́нїємъ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Inanimate)
                    .expect("complete soft -їе noun cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
    }

    #[test]
    fn skonchanie_has_a_complete_fixed_accent_soft_ie_paradigm() {
        let noun = Noun::resolve_with(
            "скончанїе",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed completion noun");

        for (case, number, expected) in [
            (Case::Nominative, Number::Singular, "сконча́нїе"),
            (Case::Genitive, Number::Singular, "сконча́нїѧ"),
            (Case::Locative, Number::Singular, "сконча́нїи"),
            (Case::Nominative, Number::Plural, "сконча́нїѧ"),
            (Case::Dative, Number::Plural, "сконча́нїємъ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Inanimate)
                    .expect("complete soft -їе noun cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
    }

    #[test]
    fn reviewed_v21_soft_ie_nouns_have_complete_fixed_accent_paradigms() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();

        for (lemma, representative_forms) in [
            (
                "видѣнїе",
                [
                    (Case::Nominative, Number::Singular, "видѣ́нїе"),
                    (Case::Locative, Number::Singular, "видѣ́нїи"),
                    (Case::Accusative, Number::Plural, "видѣ̑нїѧ"),
                    (Case::Dative, Number::Plural, "видѣ́нїємъ"),
                ],
            ),
            (
                "спасенїе",
                [
                    (Case::Nominative, Number::Singular, "спасе́нїе"),
                    (Case::Genitive, Number::Singular, "спасе́нїѧ"),
                    (Case::Locative, Number::Singular, "спасе́нїи"),
                    (Case::Dative, Number::Plural, "спасе́нїємъ"),
                ],
            ),
            (
                "поношенїе",
                [
                    (Case::Nominative, Number::Singular, "поноше́нїе"),
                    (Case::Genitive, Number::Singular, "поноше́нїѧ"),
                    (Case::Locative, Number::Singular, "поноше́нїи"),
                    (Case::Dative, Number::Plural, "поноше́нїємъ"),
                ],
            ),
            (
                "ѿмщенїе",
                [
                    (Case::Nominative, Number::Singular, "ѿмще́нїе"),
                    (Case::Genitive, Number::Singular, "ѿмще́нїѧ"),
                    (Case::Locative, Number::Singular, "ѿмще́нїи"),
                    (Case::Dative, Number::Plural, "ѿмще́нїємъ"),
                ],
            ),
        ] {
            let noun =
                Noun::resolve_with(lemma, inflector).expect("reviewed productive soft -їе noun");
            for (case, number, expected) in representative_forms {
                assert_eq!(
                    noun.form(case, number, Animacy::Inanimate)
                        .expect("complete soft -їе noun cell")
                        .primary_text(),
                    expected,
                    "{lemma}: {case:?} {number:?}"
                );
            }
            assert_eq!(
                noun.paradigm(Animacy::Inanimate).failures().count(),
                0,
                "{lemma}"
            );
        }
    }

    #[test]
    fn knyaz_is_a_complete_mobile_soft_masculine_with_bounded_variants() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::from_id_with(
            &LexemeId::from("synodal:noun:v07-345d6105fdd39fce"),
            liturgical,
        )
        .expect("reviewed productive prince noun");

        let texts = |case, number| {
            noun.form(case, number, Animacy::Animate)
                .expect("complete prince-noun cell")
                .texts()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };

        assert!(texts(Case::Nominative, Number::Singular).contains(&"кнѧ́зь".to_owned()));
        assert_eq!(texts(Case::Instrumental, Number::Singular), ["кнѧ́земъ"]);
        assert!(texts(Case::Genitive, Number::Plural).contains(&"кнѧзе́й".to_owned()));
        assert!(texts(Case::Genitive, Number::Plural).contains(&"кнѧ̑зь".to_owned()));
        assert_eq!(texts(Case::Dative, Number::Plural), ["кнѧзє́мъ"]);
        assert!(texts(Case::Locative, Number::Singular).contains(&"кнѧ́зи".to_owned()));
        assert!(texts(Case::Locative, Number::Singular).contains(&"кнѧ́зѣ".to_owned()));
        assert!(texts(Case::Nominative, Number::Plural).contains(&"Кнѧ̑зи".to_owned()));
        assert!(texts(Case::Nominative, Number::Plural).contains(&"кнѧ́зїе".to_owned()));
        assert_eq!(texts(Case::Accusative, Number::Dual), ["кнѧ̑зѧ"]);
        assert_eq!(noun.paradigm(Animacy::Animate).failures().count(), 0);
        assert!(matches!(
            noun.form(Case::Instrumental, Number::Singular, Animacy::Inanimate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn zhrets_is_a_complete_animate_mixed_ts_noun_with_bounded_variants() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::from_id_with(&LexemeId::from("synodal:noun:v11-332e30b022aa"), liturgical)
            .expect("reviewed productive priest noun");

        let texts = |case, number| {
            noun.form(case, number, Animacy::Animate)
                .expect("complete priest-noun cell")
                .texts()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };

        let nominative = noun
            .form(Case::Nominative, Number::Singular, Animacy::Animate)
            .expect("reviewed exact priest nominative");
        assert!(matches!(
            &nominative.primary().source,
            FormSource::SynodalAttestation { evidence }
                if evidence.as_str() == "review:v11:332e30b022aa"
        ));
        assert_eq!(texts(Case::Nominative, Number::Singular), ["жре́цъ"]);
        assert_eq!(texts(Case::Instrumental, Number::Singular), ["жерце́мъ"]);
        for expected in ["жерцꙋ̀", "жрецꙋ̀", "жерце́ви"] {
            assert!(
                texts(Case::Dative, Number::Singular).contains(&expected.to_owned()),
                "missing dative variant {expected}"
            );
        }
        for expected in ["жерцы̀", "жерцы́", "жєрцы̀"] {
            assert!(
                texts(Case::Nominative, Number::Plural).contains(&expected.to_owned()),
                "missing nominative-plural variant {expected}"
            );
        }
        for expected in ["жерцє́въ", "жерцѡ́въ", "жрє́цъ"] {
            assert!(
                texts(Case::Genitive, Number::Plural).contains(&expected.to_owned()),
                "missing genitive-plural variant {expected}"
            );
        }
        for expected in ["жерцы̀", "жерцьмѝ", "жерца́ми"] {
            assert!(
                texts(Case::Instrumental, Number::Plural).contains(&expected.to_owned()),
                "missing instrumental-plural variant {expected}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Animate).failures().count(), 0);
        assert!(matches!(
            noun.form(Case::Instrumental, Number::Singular, Animacy::Inanimate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn prestol_is_a_complete_inanimate_fixed_accent_hard_masculine() {
        let noun = Noun::from_id_with(
            &LexemeId::from("synodal:noun:prestol"),
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed productive throne noun");

        let texts = |case, number| {
            noun.form(case, number, Animacy::Inanimate)
                .expect("complete throne-noun cell")
                .texts()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };

        assert_eq!(texts(Case::Nominative, Number::Singular), ["престо́лъ"]);
        assert_eq!(texts(Case::Genitive, Number::Singular), ["престо́ла"]);
        assert!(texts(Case::Dative, Number::Singular).contains(&"престо́лꙋ".to_owned()));
        assert_eq!(texts(Case::Instrumental, Number::Singular), ["престо́ломъ"]);
        assert_eq!(texts(Case::Locative, Number::Singular), ["престо́лѣ"]);
        assert_eq!(texts(Case::Nominative, Number::Plural), ["престо́ли"]);
        assert_eq!(texts(Case::Accusative, Number::Plural), ["престо́лы"]);
        assert!(texts(Case::Instrumental, Number::Plural).contains(&"престо́лами".to_owned()));

        let genitive_plural = noun
            .form(Case::Genitive, Number::Plural, Animacy::Inanimate)
            .expect("complete throne genitive plural");
        assert_eq!(genitive_plural.primary_text(), "престо́лѡвъ");
        assert!(matches!(
            &genitive_plural.primary().source,
            FormSource::SynodalAttestation { evidence }
                if evidence.as_str()
                    == "ponomar-iv-kings-25-28-prestol-genitive-plural-wide-omega"
        ));
        assert!(genitive_plural.texts().any(|form| form == "престо́ловъ"));
        assert!(genitive_plural.texts().any(|form| form == "престо́лъ"));

        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
        assert!(matches!(
            noun.form(Case::Instrumental, Number::Singular, Animacy::Animate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn rab_preserves_the_bounded_wide_omega_animate_plural_variant() {
        let noun = Noun::from_id_with(
            &LexemeId::from("synodal:noun:rab"),
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed servant noun");

        for case in [Case::Genitive, Case::Accusative] {
            let forms = noun
                .form(case, Number::Plural, Animacy::Animate)
                .expect("reviewed animate plural cell");
            assert!(
                forms.texts().any(|surface| surface == "рабѡ́въ"),
                "missing wide-omega {case:?} plural"
            );
        }
    }

    #[test]
    fn dshcher_has_a_complete_fixed_oblique_accent_paradigm() {
        let noun = Noun::resolve_with(
            "дщерь",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed daughter noun");

        for (case, number, expected) in [
            (Case::Nominative, Number::Singular, "Дщѝ"),
            (Case::Genitive, Number::Singular, "дще́ре"),
            (Case::Instrumental, Number::Singular, "дще́рїю"),
            (Case::Nominative, Number::Plural, "дщє́ри"),
            (Case::Genitive, Number::Plural, "дще́рей"),
            (Case::Instrumental, Number::Plural, "дще́рьми"),
            (Case::Locative, Number::Plural, "дще́рехъ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Animate)
                    .expect("complete daughter cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Animate).failures().count(), 0);
    }

    #[test]
    fn sosud_has_a_complete_fixed_accent_hard_masculine_paradigm() {
        let noun = Noun::resolve_with(
            "сосꙋдъ",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed vessel noun");

        for (case, number, expected) in [
            (Case::Nominative, Number::Singular, "сосꙋ́дъ"),
            (Case::Genitive, Number::Singular, "сосꙋ́да"),
            (Case::Instrumental, Number::Singular, "сосꙋ́домъ"),
            (Case::Nominative, Number::Plural, "сосꙋ́ди"),
            (Case::Genitive, Number::Plural, "сосꙋ́дѡвъ"),
            (Case::Locative, Number::Plural, "сосꙋ́дѣхъ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Inanimate)
                    .expect("complete vessel cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
    }

    #[test]
    fn iuda_has_a_complete_masculine_second_declension_paradigm() {
        let noun = Noun::resolve_with(
            "іꙋда",
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("reviewed Judah/Judas identity");

        for (case, number, expected) in [
            (Case::Nominative, Number::Singular, "і҆ꙋ́да"),
            (Case::Genitive, Number::Singular, "і҆ꙋ́ды"),
            (Case::Dative, Number::Singular, "і҆ꙋ́дѣ"),
            (Case::Accusative, Number::Singular, "і҆ꙋ́дꙋ"),
            (Case::Instrumental, Number::Singular, "і҆ꙋ́дою"),
            (Case::Vocative, Number::Singular, "і҆ꙋ́до"),
            (Case::Genitive, Number::Plural, "і҆ꙋ́дъ"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Animate)
                    .expect("complete Judah/Judas cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Animate).failures().count(), 0);
    }

    #[test]
    fn alpy_43_registry_accent_paradigms_cover_complete_productive_tables() {
        fn assert_accented_paradigm(
            id: &str,
            lemma: &str,
            stem: &str,
            gender: Gender,
            declension: NounDeclension,
            animacy: Animacy,
            expected: &[&str],
        ) {
            let id = LexemeId::from(id);
            let seed = GrammarCell::Noun(NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy,
            });
            let accent = registry::accent_paradigm_for(&id, seed)
                .expect("valid accent metadata")
                .expect("registered accent paradigm");
            let source = SpecificationSource::new(
                format!("test-metadata:{}", id.as_str()),
                "alypy-gamanovich-grammar-web-2023",
                "Alypy (Gamanovich), §43",
            )
            .expect("source metadata");
            let positional = source.positional_paradigm(
                format!("test-positional:{}", id.as_str()),
                vec![PositionalRule {
                    scope: AccentScope::All,
                    operations: vec![],
                }],
            );
            let spec = NounSpec::new(lemma, stem, gender, declension, source)
                .expect("typed noun")
                .with_accent_paradigm(accent)
                .expect("accent contract")
                .with_positional_paradigm(positional)
                .expect("positional contract");
            let paradigm = spec.paradigm_with(
                Inflector::builder()
                    .orthography(OrthographyProfile::SynodalLiturgical)
                    .build(),
                animacy,
            );
            assert_eq!(expected.len(), Number::ALL.len() * Case::ALL.len());
            for (index, (number, case)) in Number::ALL
                .into_iter()
                .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
                .enumerate()
            {
                let cell = GrammarCell::Noun(NounCell {
                    case,
                    number,
                    animacy,
                });
                assert_eq!(
                    paradigm
                        .form(cell)
                        .unwrap_or_else(|error| panic!("{lemma} {number:?} {case:?}: {error}"))
                        .primary_text(),
                    expected[index],
                    "{lemma} {number:?} {case:?}"
                );
            }
        }

        assert_accented_paradigm(
            "synodal:noun:wikt-551a03f1df94",
            "имѧ",
            "имен",
            Gender::Neuter,
            NounDeclension::FourthNeuterEn,
            Animacy::Inanimate,
            &[
                "и҆́мѧ",
                "и҆́мене",
                "и҆́мени",
                "и҆́мѧ",
                "и҆́менемъ",
                "и҆́мени",
                "и҆́мѧ",
                "и҆́мєни",
                "и҆менꙋ̀",
                "и҆мене́ма",
                "и҆́мєни",
                "и҆мене́ма",
                "и҆менꙋ̀",
                "и҆́мєни",
                "и҆мена̀",
                "и҆ме́нъ",
                "и҆́менємъ",
                "и҆мена̀",
                "и҆мены̀",
                "и҆́менѣхъ",
                "и҆мена̀",
            ],
        );
        assert_accented_paradigm(
            "synodal:noun:wikt-7790891c2704",
            "небо",
            "небес",
            Gender::Neuter,
            NounDeclension::FourthNeuterEs,
            Animacy::Inanimate,
            &[
                "не́бо",
                "небесѐ",
                "небесѝ",
                "не́бо",
                "небесе́мъ",
                "небесѝ",
                "не́бо",
                "небєсѝ",
                "небесꙋ̀",
                "небесе́ма",
                "небєсѝ",
                "небесе́ма",
                "небесꙋ̀",
                "небєсѝ",
                "небеса̀",
                "небе́съ",
                "небесє́мъ",
                "небеса̀",
                "небесы̀",
                "небесѣ́хъ",
                "небеса̀",
            ],
        );
        assert_accented_paradigm(
            "synodal:noun:wikt-a0a33dfa77c7",
            "мати",
            "матер",
            Gender::Feminine,
            NounDeclension::FourthFeminineEr,
            Animacy::Animate,
            &[
                "ма́ти",
                "ма́тере",
                "ма́тери",
                "ма́терь",
                "ма́терїю",
                "ма́тери",
                "ма́ти",
                "ма́тєри",
                "ма́тєрїю",
                "ма́терема",
                "ма́тєри",
                "ма́терема",
                "ма́тєрїю",
                "ма́тєри",
                "ма́тєри",
                "ма́терїй",
                "ма́теремъ",
                "ма́терей",
                "ма́терьми",
                "ма́терехъ",
                "ма́тєри",
            ],
        );
    }

    #[test]
    fn exact_noun_table_precedes_reusable_accent_and_productive_background() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::from_id_with(&LexemeId::from("synodal:noun:wikt-551a03f1df94"), inflector)
            .expect("registered noun");
        let forms = noun
            .form(Case::Nominative, Number::Singular, Animacy::Inanimate)
            .expect("exact table cell");
        assert_eq!(forms.primary_text(), "и҆́мѧ");
        assert_eq!(
            forms.primary().rule_trace.steps()[0].rule.as_str(),
            "SYN-REGISTRY-NORMATIVE-TABLE"
        );
        assert!(
            forms
                .primary()
                .evidence
                .iter()
                .all(|evidence| { evidence.kind != core::EvidenceKind::AccentParadigm })
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
    fn upgraded_mixed_noun_is_exact_first_with_a_productive_background() {
        let noun = Noun::from_id(&LexemeId::from("synodal:noun:man")).expect("reviewed mixed noun");
        assert!(noun.capabilities().exact_forms);
        assert!(noun.capabilities().productive_noun);

        let exact = noun
            .form(Case::Nominative, Number::Singular, Animacy::Animate)
            .expect("reviewed exact cell");
        assert_eq!(exact.primary_text(), "мꙋжъ");
        assert!(matches!(
            exact.primary().source,
            FormSource::SynodalAttestation { .. }
        ));

        let productive = noun
            .form(Case::Dative, Number::Dual, Animacy::Animate)
            .expect("licensed mixed background");
        assert_eq!(productive.primary_text(), "мꙋжема");
        assert!(matches!(
            productive.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));
        assert_eq!(
            productive.primary().rule_trace.steps()[0].rule.as_str(),
            "SYN-NOUN-I-MIXED-M-ALYPY-33-34"
        );
    }

    #[test]
    fn registered_plural_only_noun_exposes_restriction_and_productive_cells() {
        let id = LexemeId::from("synodal:noun:people");
        let noun = Noun::from_id(&id).expect("reviewed plural-only noun");
        assert!(noun.capabilities().productive_noun);

        let genitive = noun
            .form(Case::Genitive, Number::Plural, Animacy::Animate)
            .expect("licensed plural background");
        assert_eq!(
            genitive
                .variants()
                .iter()
                .map(|variant| variant.printed.as_str())
                .collect::<Vec<_>>(),
            ["людей", "людій"]
        );
        assert!(matches!(
            noun.form(Case::Genitive, Number::Singular, Animacy::Animate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let metadata = lexical_metadata(&id).expect("reviewable metadata");
        let restriction = metadata
            .noun_restriction
            .expect("noun restriction metadata");
        assert_eq!(restriction.number_inventory, "plural-only");
        assert_eq!(restriction.animacy_inventory, "any");
        assert_eq!(restriction.evidence_id, "alypy-32-41-people-table");
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
        assert!(matches!(
            unsupported,
            Err(Error::EvidenceIncompleteCell { .. })
        ));
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
    fn o_interjection_is_an_exact_invariant_identity() {
        let form = Inflector::default()
            .form_by_id(
                &LexemeId::from("synodal:interjection:o"),
                GrammarCell::Indeclinable,
            )
            .expect("reviewed exact interjection");

        assert_eq!(form.primary_text(), "ѽ");
        assert!(matches!(
            &form.primary().source,
            FormSource::SynodalAttestation { .. }
        ));
    }

    #[test]
    fn dokole_is_an_exact_interrogative_temporal_adverb() {
        let form = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build()
            .form_by_id(
                &LexemeId::from("synodal:adverb:dokole"),
                GrammarCell::Indeclinable,
            )
            .expect("reviewed invariant temporal adverb");

        assert_eq!(form.texts().collect::<Vec<_>>(), ["доко́лѣ"]);
        assert!(
            form.variants()
                .iter()
                .all(|variant| matches!(&variant.source, FormSource::SynodalAttestation { .. }))
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
    fn numeral_exact_attestation_precedes_productive_background() {
        let forms = numeral(
            "первый",
            NumeralCell {
                kind: NumeralKind::Ordinal,
                case: Case::Genitive,
                number: Number::Singular,
                gender: Some(Gender::Masculine),
                animacy: Animacy::Inanimate,
            },
        )
        .expect("reviewed first-ordinal cell");
        assert_eq!(forms.primary_text(), "первагѡ");
        assert!(forms.primary().is_attested());

        let productive = numeral(
            "первый",
            NumeralCell {
                kind: NumeralKind::Ordinal,
                case: Case::Dative,
                number: Number::Dual,
                gender: Some(Gender::Feminine),
                animacy: Animacy::Inanimate,
            },
        )
        .expect("productive ordinal background");
        assert!(matches!(
            productive.primary().source,
            core::FormSource::SynodalNormativeGeneration { .. }
        ));
    }

    #[test]
    fn determiner_handle_generates_reviewed_short_and_long_cells() {
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

        let long = determiner(
            "всѧкъ",
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            },
        )
        .expect("reviewed full determiner cell");
        assert_eq!(long.primary_text(), "всѧкїй");

        assert!(matches!(
            determiner(
                "всѧкъ",
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Dual,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn productive_determiner_background_contains_every_reviewed_exact_surface() {
        for id in [
            "synodal:determiner:sam",
            "synodal:determiner:ves",
            "synodal:determiner:vsyak",
        ] {
            let id = LexemeId::from(id);
            let lexeme = registry::determiner_lexeme(&id).expect("productive determiner metadata");
            for cell in
                AdjectiveCell::inventory(&AdjectiveForm::ALL, &[Comparison::Positive], |_| {
                    &Animacy::ALL
                })
            {
                let Ok(predicted) =
                    core::decline_determiner(&lexeme, cell, OrthographyProfile::Expanded)
                else {
                    continue;
                };
                let predicted = predicted.texts().collect::<Vec<_>>();
                for key in grammar_cell_registry_keys(GrammarCell::Determiner(cell)) {
                    for exact in registry::exact_forms(&id, &key) {
                        assert!(
                            predicted.contains(&exact.expanded),
                            "{} {key} exact {:?} absent from {predicted:?}",
                            id,
                            exact.expanded,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn vino_has_complete_mobile_accent_hard_neuter_paradigm() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let noun = Noun::resolve_with("вїно", inflector).expect("reviewed wine noun");

        for (case, number, expected) in [
            (Case::Nominative, Number::Singular, "вїно̀"),
            (Case::Genitive, Number::Singular, "вїна̀"),
            (Case::Dative, Number::Singular, "вїнꙋ̀"),
            (Case::Instrumental, Number::Singular, "вїно́мъ"),
            (Case::Locative, Number::Singular, "вїнѣ̀"),
            (Case::Nominative, Number::Plural, "вї́на"),
            (Case::Genitive, Number::Plural, "вї́нъ"),
            (Case::Instrumental, Number::Plural, "вї́ны"),
        ] {
            assert_eq!(
                noun.form(case, number, Animacy::Inanimate)
                    .expect("complete hard-neuter cell")
                    .primary_text(),
                expected,
                "{case:?} {number:?}"
            );
        }
        assert_eq!(noun.paradigm(Animacy::Inanimate).failures().count(), 0);
    }

    #[test]
    fn polozhiti_has_complete_reviewed_perfective_systems() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let verb = Verb::resolve_with("положити", inflector).expect("reviewed perfective verb");

        assert_eq!(
            verb.future(Person::First, Number::Singular)
                .expect("future first singular")
                .primary_text(),
            "положꙋ̀"
        );
        assert_eq!(
            verb.future(Person::Third, Number::Plural)
                .expect("future third plural")
                .primary_text(),
            "положа́тъ"
        );
        assert_eq!(
            verb.aorist(Person::Second, Number::Plural)
                .expect("productive vowel aorist")
                .primary_text(),
            "положи́сте"
        );
        assert_eq!(
            verb.imperative(Person::Second, Number::Singular)
                .expect("exact imperative")
                .primary_text(),
            "положѝ"
        );
        assert_eq!(
            verb.l_participle(Gender::Feminine, Number::Singular)
                .expect("productive l-participle")
                .primary_text(),
            "положи́ла"
        );

        for system in [
            VerbSystem::Finite(FiniteTense::Future),
            VerbSystem::Finite(FiniteTense::Aorist),
            VerbSystem::Imperative,
            VerbSystem::LParticiple,
        ] {
            assert!(
                verb.missing_principal_parts(system)
                    .expect("registered system metadata")
                    .is_empty(),
                "{system:?}"
            );
        }
    }

    #[test]
    fn high_frequency_v15_families_are_productive_and_source_bounded() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();

        let tsar = Noun::resolve_with("царь", inflector).expect("reviewed tsar noun");
        assert!(
            tsar.form(Case::Nominative, Number::Plural, Animacy::Animate)
                .expect("reviewed -їе nominative plural")
                .texts()
                .any(|text| text == "ца́рїе")
        );

        let sushchym = inflector
            .form_by_id(
                &LexemeId::from("synodal:verb:byti"),
                GrammarCell::Participle(ParticipleCell {
                    tense: ParticipleTense::Present,
                    voice: ParticipleVoice::Active,
                    agreement: AdjectiveCell {
                        case: Case::Dative,
                        number: Number::Plural,
                        gender: Gender::Masculine,
                        animacy: Animacy::Animate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                }),
            )
            .expect("productive present-active participle accent");
        assert_eq!(sushchym.primary_text(), "сꙋ́щымъ");

        let vzeti = Verb::resolve_with("възѧти", inflector).expect("reviewed vzeti verb");
        assert_eq!(
            vzeti
                .future(Person::Third, Number::Singular)
                .expect("future third singular")
                .primary_text(),
            "во́зметъ"
        );
        assert_eq!(
            vzeti
                .future(Person::Second, Number::Plural)
                .expect("productive future second plural")
                .primary_text(),
            "во́змете"
        );

        let iziti = Verb::resolve_with("изити", inflector).expect("reviewed iziti verb");
        assert_eq!(
            iziti
                .present(Person::Third, Number::Plural)
                .expect("productive perfective finite third plural")
                .primary_text(),
            "и҆зы́дꙋтъ"
        );

        let tsarstvo = Noun::resolve_with("царство", inflector).expect("reviewed kingdom noun");
        assert_eq!(
            tsarstvo
                .form(Case::Genitive, Number::Singular, Animacy::Inanimate)
                .expect("productive hard-neuter genitive")
                .primary_text(),
            "ца́рства"
        );

        let otechestvo = Noun::resolve_with("ѻтечество", inflector).expect("reviewed lineage noun");
        assert_eq!(
            otechestvo
                .form(Case::Genitive, Number::Plural, Animacy::Inanimate)
                .expect("productive zero-ending genitive plural")
                .primary_text(),
            "ѻ҆те́чествъ"
        );
        assert_eq!(
            otechestvo.paradigm(Animacy::Inanimate).failures().count(),
            0
        );
    }

    #[test]
    fn high_frequency_v16_nominal_families_use_complete_typed_paradigms() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();

        let lawlessness =
            Noun::resolve_with("беззаконїе", inflector).expect("reviewed lawlessness noun");
        assert_eq!(
            lawlessness
                .form(Case::Dative, Number::Plural, Animacy::Inanimate)
                .expect("productive soft -їе dative plural")
                .primary_text(),
            "беззако́нїємъ"
        );
        assert_eq!(
            lawlessness
                .form(Case::Accusative, Number::Plural, Animacy::Inanimate)
                .expect("reviewed exact wide-omega plural")
                .primary_text(),
            "беззакѡ́нїѧ"
        );
        assert_eq!(
            lawlessness.paradigm(Animacy::Inanimate).failures().count(),
            0
        );

        let egypt = Noun::from_id_with(&LexemeId::from("synodal:proper-noun:egipet"), inflector)
            .expect("reviewed place name");
        assert_eq!(
            egypt
                .form(Case::Accusative, Number::Singular, Animacy::Inanimate)
                .expect("exact fleeting-vowel citation cell")
                .primary_text(),
            "є҆гѵ́петъ"
        );
        assert_eq!(
            egypt
                .form(Case::Dative, Number::Plural, Animacy::Inanimate)
                .expect("productive oblique-stem cell")
                .primary_text(),
            "є҆гѵ́птомъ"
        );

        let egyptian =
            Adjective::resolve_with("єгѵпетскїй", inflector).expect("reviewed -ск- adjective");
        for (case, number, gender, form, expected) in [
            (
                Case::Genitive,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Short,
                "є҆гѵ́петски",
            ),
            (
                Case::Locative,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Long,
                "є҆гѵ́петстѣй",
            ),
            (
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                AdjectiveForm::Long,
                "є҆гѵ́петстїи",
            ),
            (
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                AdjectiveForm::Long,
                "є҆гѵ́петскихъ",
            ),
        ] {
            assert_eq!(
                egyptian
                    .form(AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Inanimate,
                        form,
                        comparison: Comparison::Positive,
                    })
                    .expect("productive -ск- cell")
                    .primary_text(),
                expected
            );
        }
        assert_eq!(
            egyptian.paradigm(AdjectiveForm::Short).failures().count(),
            0
        );
        assert_eq!(egyptian.paradigm(AdjectiveForm::Long).failures().count(), 0);

        let judahite =
            Adjective::resolve_with("іꙋдинъ", inflector).expect("reviewed -ин- adjective");
        for (case, number, gender, form, expected) in [
            (
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                AdjectiveForm::Short,
                "і҆ꙋ́дина",
            ),
            (
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                AdjectiveForm::Long,
                "і҆ꙋ́динымъ",
            ),
            (
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                AdjectiveForm::Long,
                "і҆ꙋ́диныхъ",
            ),
        ] {
            assert_eq!(
                judahite
                    .form(AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Inanimate,
                        form,
                        comparison: Comparison::Positive,
                    })
                    .expect("productive -ин- cell")
                    .primary_text(),
                expected
            );
        }
        assert_eq!(
            judahite.paradigm(AdjectiveForm::Short).failures().count(),
            0
        );
        assert_eq!(judahite.paradigm(AdjectiveForm::Long).failures().count(), 0);
    }

    #[test]
    fn high_frequency_v17_nominal_families_use_source_bounded_typed_paradigms() {
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();

        let human = Adjective::resolve_with("человѣчь", inflector)
            .expect("reviewed historical -jь possessive adjective");
        for (case, number, gender, expected) in [
            (
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                "человѣ́ча",
            ),
            (
                Case::Instrumental,
                Number::Plural,
                Gender::Masculine,
                "человѣ́чими",
            ),
        ] {
            assert_eq!(
                human
                    .form(AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    })
                    .expect("productive historical -jь cell")
                    .primary_text(),
                expected
            );
        }
        assert_eq!(human.paradigm(AdjectiveForm::Short).failures().count(), 0);
        assert!(
            human
                .form(AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .is_err()
        );

        let human_relational = Adjective::resolve_with("человѣческїй", inflector)
            .expect("reviewed human relational adjective");
        assert_eq!(
            human_relational
                .form(AdjectiveCell {
                    case: Case::Instrumental,
                    number: Number::Plural,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("productive human relational cell")
                .primary_text(),
            "человѣ́ческими"
        );

        let joseph = Noun::from_id_with(&LexemeId::from("synodal:proper-noun:iosif"), inflector)
            .expect("reviewed Joseph proper name");
        assert_eq!(
            joseph
                .form(Case::Instrumental, Number::Singular, Animacy::Animate)
                .expect("productive Joseph instrumental")
                .primary_text(),
            "і҆ѡ́сифомъ"
        );
        let josephs = Adjective::resolve_with("іѡсифовъ", inflector)
            .expect("reviewed Joseph possessive adjective");
        assert_eq!(
            josephs
                .form(AdjectiveCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                })
                .expect("productive Joseph possessive cell")
                .primary_text(),
            "і҆ѡ́сифова"
        );

        let jordan = Noun::from_id_with(&LexemeId::from("synodal:proper-noun:iordan"), inflector)
            .expect("reviewed Jordan river name");
        assert_eq!(
            jordan
                .form(Case::Instrumental, Number::Singular, Animacy::Inanimate)
                .expect("productive Jordan instrumental")
                .primary_text(),
            "і҆ѻрда́номъ"
        );
        let jordanian = Adjective::resolve_with("іѻрданскїй", inflector)
            .expect("reviewed Jordan relational adjective");
        assert_eq!(
            jordanian
                .form(AdjectiveCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("productive Jordan relational cell")
                .primary_text(),
            "і҆ѻрда́нскихъ"
        );

        let levite = Noun::resolve_with("леѵітъ", inflector).expect("reviewed Levite noun");
        assert_eq!(
            levite
                .form(Case::Dative, Number::Singular, Animacy::Animate)
                .expect("productive Levite dative")
                .primary_text(),
            "леѵі́тꙋ"
        );
        let levitical =
            Adjective::resolve_with("леѵітскїй", inflector).expect("reviewed Levitical adjective");
        assert_eq!(
            levitical
                .form(AdjectiveCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("productive Levitical cell")
                .primary_text(),
            "леѵі́тскихъ"
        );
    }

    #[test]
    fn determiner_exact_cells_precede_the_complete_productive_background() {
        let vsyak = Determiner::resolve("всѧкъ").expect("reviewed determiner");
        let exact = vsyak
            .form(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            })
            .expect("reviewed exact cell");
        assert_eq!(exact.texts().collect::<Vec<_>>(), ["всѧкаго", "всѧкагѡ"]);
        assert!(
            exact
                .variants()
                .iter()
                .all(|variant| !matches!(variant.source, FormSource::SynodalNormativeGeneration { ref rule } if rule.as_ref().starts_with("SYN-DETERMINER-")))
        );

        let generated = vsyak
            .form(AdjectiveCell {
                case: Case::Dative,
                number: Number::Plural,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            })
            .expect("productive background cell");
        assert_eq!(generated.primary_text(), "всѧкимъ");
        assert!(matches!(
            &generated.primary().source,
            FormSource::SynodalNormativeGeneration { rule }
                if rule.as_ref() == "SYN-DETERMINER-VSYAK-MIXED-ALYPY-45-48-57"
        ));
    }

    #[test]
    fn determiner_liturgical_output_uses_reviewed_accent_or_fails_typed() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let vsyak =
            Determiner::resolve_with("всѧкъ", liturgical).expect("fixed-stress mixed determiner");
        assert_eq!(
            vsyak
                .form(AdjectiveCell {
                    case: Case::Dative,
                    number: Number::Plural,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("reusable accent paradigm")
                .primary_text(),
            "всѧ́кимъ"
        );
        assert_eq!(
            Determiner::resolve_with("всѧческїй", liturgical)
                .expect("fixed-stress full determiner")
                .form(AdjectiveCell {
                    case: Case::Locative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                })
                .expect("reviewed -ск-/-ст- accent realization")
                .primary_text(),
            "всѧ́честѣмъ"
        );

        for (id, cell) in [
            (
                "synodal:determiner:sam",
                AdjectiveCell {
                    case: Case::Dative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            ),
            (
                "synodal:determiner:ves",
                AdjectiveCell {
                    case: Case::Locative,
                    number: Number::Plural,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            ),
        ] {
            assert!(matches!(
                Determiner::from_id_with(&LexemeId::from(id), liturgical)
                    .expect("reviewed determiner")
                    .form(cell),
                Err(Error::OrthographicMetadataRequired {
                    field: MetadataField::AccentParadigm
                })
            ));
        }
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
        assert!(capabilities.future);
        assert!(!capabilities.past);
        assert!(capabilities.imperfect);
        assert!(capabilities.aorist);
        assert!(capabilities.imperative);
        assert!(capabilities.infinitive);
        assert!(capabilities.l_participle);
        assert!(capabilities.participle);
        assert!(!capabilities.supine);
        assert!(!capabilities.verbal_noun);
        assert!(
            verb.missing_principal_parts(VerbSystem::Supine)
                .expect("represented absent system")
                .is_empty()
        );
        assert!(!verb.missing_metadata().contains(&MetadataField::SupineStem));
        assert!(matches!(
            supine("быти"),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let nesti = Verb::resolve("нести").expect("reviewed regular verb");
        assert!(nesti.capabilities().verbal_noun);
        assert!(
            nesti
                .missing_principal_parts(VerbSystem::VerbalNoun {
                    animacy: Animacy::Inanimate,
                })
                .expect("represented productive system")
                .is_empty()
        );
        for animacy in Animacy::ALL {
            let paradigm = nesti.system_paradigm(VerbSystem::VerbalNoun { animacy });
            assert_eq!(paradigm.iter().count(), 21);
            assert_eq!(paradigm.successes().count(), 21);
            assert_eq!(paradigm.failures().count(), 0);
        }
        assert_eq!(
            verbal_noun(
                "нести",
                NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                },
            )
            .expect("past-passive-platform verbal noun")
            .primary_text(),
            "несенїѧ"
        );
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
        assert!(!dati.capabilities().present);
        assert!(dati.capabilities().future);
        assert_eq!(
            dati.future(Person::Third, Number::Singular)
                .expect("reviewed simple-future table")
                .primary_text(),
            "дастъ"
        );
        assert!(matches!(
            dati.present(Person::Third, Number::Singular),
            Err(Error::MissingPrincipalPart {
                field: MetadataField::PresentStem,
            })
        ));

        let reclassified_past = Verb::from_id(&LexemeId::from("synodal:verb:wikt-78da2d05497d"))
            .expect("reviewed reclassified-past verb");
        assert!(!reclassified_past.capabilities().past);
        assert!(reclassified_past.capabilities().aorist);
        assert!(!reclassified_past.capabilities().future);
        assert!(!reclassified_past.capabilities().infinitive);
        assert!(matches!(
            Inflector::default().form_by_id(reclassified_past.id(), GrammarCell::Supine),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let typed_irregular = Verb::from_id(&LexemeId::from("synodal:verb:v06-vzeti"))
            .expect("reviewed typed irregular verb");
        let missing = typed_irregular.missing_metadata();
        assert!(!missing.contains(&core::MetadataField::AoristStem));
        assert!(!missing.contains(&core::MetadataField::ImperativeStem));
        assert!(!missing.contains(&core::MetadataField::LParticipleStem));
        assert!(!missing.contains(&core::MetadataField::ParticipleStem));
        assert!(!missing.contains(&core::MetadataField::VerbalNounStem));

        assert!(
            Determiner::from_id(&LexemeId::from("synodal:determiner:sam"))
                .expect("productive determiner")
                .capabilities()
                .productive_determiner
        );
        assert!(
            Numeral::from_id(&LexemeId::from("synodal:numeral:pervyi"))
                .expect("productive ordinal")
                .capabilities()
                .productive_numeral
        );
        assert!(
            Numeral::from_id(&LexemeId::from("synodal:numeral:dva"))
                .expect("exact cardinal")
                .capabilities()
                .productive_numeral
        );
    }

    #[test]
    fn high_frequency_v18_numerals_realize_complete_source_backed_patterns() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let oba = Numeral::from_id_with(&LexemeId::from("synodal:numeral:oba"), liturgical)
            .expect("reviewed cardinal both");
        let third = Numeral::from_id_with(&LexemeId::from("synodal:numeral:tretii"), liturgical)
            .expect("reviewed third ordinal");
        let seventh = Numeral::from_id_with(&LexemeId::from("synodal:numeral:sedmyi"), liturgical)
            .expect("reviewed seventh ordinal");

        let form = |numeral: &Numeral, kind, case, number, gender, animacy| -> String {
            numeral
                .form(NumeralCell {
                    kind,
                    case,
                    number,
                    gender,
                    animacy,
                })
                .expect("licensed numeral cell")
                .primary_text()
                .into()
        };

        assert_eq!(
            form(
                &oba,
                NumeralKind::Cardinal,
                Case::Nominative,
                Number::Dual,
                Some(Gender::Masculine),
                Animacy::Inanimate,
            ),
            "ѻ҆́ба"
        );
        assert_eq!(
            form(
                &oba,
                NumeralKind::Cardinal,
                Case::Genitive,
                Number::Dual,
                Some(Gender::Feminine),
                Animacy::Inanimate,
            ),
            "ѻ҆бою̀"
        );
        assert_eq!(
            form(
                &oba,
                NumeralKind::Cardinal,
                Case::Dative,
                Number::Dual,
                Some(Gender::Neuter),
                Animacy::Inanimate,
            ),
            "ѻ҆бѣ́ма"
        );
        assert_eq!(
            form(
                &third,
                NumeralKind::Ordinal,
                Case::Genitive,
                Number::Singular,
                Some(Gender::Masculine),
                Animacy::Inanimate,
            ),
            "тре́тїѧгѡ"
        );
        assert_eq!(
            form(
                &third,
                NumeralKind::Ordinal,
                Case::Accusative,
                Number::Singular,
                Some(Gender::Feminine),
                Animacy::Inanimate,
            ),
            "тре́тїю"
        );
        assert_eq!(
            form(
                &seventh,
                NumeralKind::Ordinal,
                Case::Genitive,
                Number::Singular,
                Some(Gender::Masculine),
                Animacy::Inanimate,
            ),
            "седма́гѡ"
        );
        assert_eq!(
            form(
                &seventh,
                NumeralKind::Ordinal,
                Case::Nominative,
                Number::Singular,
                Some(Gender::Neuter),
                Animacy::Inanimate,
            ),
            "седмо́е"
        );
    }

    #[test]
    fn cardinal_one_has_a_complete_source_bounded_singular_accent_paradigm() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let one = Numeral::from_id_with(&LexemeId::from("synodal:numeral:edin"), liturgical)
            .expect("reviewed cardinal one");
        let cell = |case, gender| NumeralCell {
            kind: NumeralKind::Cardinal,
            case,
            number: Number::Singular,
            gender: Some(gender),
            animacy: Animacy::Inanimate,
        };

        let neuter_instrumental = one
            .form(cell(Case::Instrumental, Gender::Neuter))
            .expect("source-attested neuter instrumental");
        assert_eq!(neuter_instrumental.primary_text(), "є҆ди́нѣмъ");
        assert!(neuter_instrumental.primary().is_attested());

        let feminine_instrumental = one
            .form(cell(Case::Instrumental, Gender::Feminine))
            .expect("source-attested feminine instrumental");
        assert_eq!(feminine_instrumental.primary_text(), "є҆ди́ною");
        assert!(feminine_instrumental.primary().is_attested());

        let masculine_instrumental = one
            .form(cell(Case::Instrumental, Gender::Masculine))
            .expect("productive masculine instrumental");
        assert_eq!(masculine_instrumental.primary_text(), "є҆ди́нѣмъ");
        assert!(matches!(
            masculine_instrumental.primary().source,
            core::FormSource::SynodalNormativeGeneration { .. }
        ));

        assert_eq!(
            one.form(cell(Case::Genitive, Gender::Feminine))
                .expect("productive feminine genitive")
                .primary_text(),
            "є҆ди́ноѧ"
        );
        assert_eq!(
            one.form(cell(Case::Locative, Gender::Masculine))
                .expect("productive masculine locative")
                .primary_text(),
            "є҆ди́номъ"
        );

        let dual = NumeralCell {
            number: Number::Dual,
            ..cell(Case::Instrumental, Gender::Masculine)
        };
        assert!(matches!(
            one.form(dual),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn high_frequency_v19_peter_name_has_complete_mobile_paradigm() {
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let peter = Noun::from_id_with(&LexemeId::from("synodal:proper-noun:petr"), liturgical)
            .expect("reviewed Peter proper noun");

        let form = |case| {
            peter
                .form(case, Number::Singular, Animacy::Animate)
                .expect("licensed Peter singular cell")
        };
        assert_eq!(form(Case::Nominative).primary_text(), "пе́тръ");
        assert_eq!(form(Case::Genitive).primary_text(), "петра̀");
        assert_eq!(form(Case::Accusative).primary_text(), "петра̀");
        assert_eq!(form(Case::Instrumental).primary_text(), "петро́мъ");
        assert_eq!(form(Case::Vocative).primary_text(), "пе́тре");
        assert_eq!(
            form(Case::Dative).texts().collect::<Vec<_>>(),
            vec!["петро́ви", "петрꙋ̀"]
        );

        let paradigm = peter.paradigm(Animacy::Animate);
        assert_eq!(paradigm.iter().count(), 21);
        assert_eq!(paradigm.failures().count(), 0);
    }

    #[test]
    fn personal_pronoun_paradigms_use_reviewed_person_and_gender_profiles() {
        let cases = [
            (
                "азъ",
                PronounCell {
                    case: Case::Genitive,
                    number: Number::Dual,
                    gender: None,
                    person: Some(Person::First),
                    animacy: Animacy::Inanimate,
                },
                "наю",
            ),
            (
                "ты",
                PronounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: None,
                    person: Some(Person::Second),
                    animacy: Animacy::Inanimate,
                },
                "ты",
            ),
            (
                "онъ",
                PronounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Some(Gender::Masculine),
                    person: Some(Person::Third),
                    animacy: Animacy::Inanimate,
                },
                "онъ",
            ),
        ];

        for (lemma, cell, expected) in cases {
            let paradigm = Pronoun::resolve(lemma)
                .expect("reviewed personal pronoun")
                .paradigm();
            assert_eq!(
                paradigm
                    .form(GrammarCell::Pronoun(cell))
                    .expect("profile-derived paradigm cell")
                    .primary_text(),
                expected
            );
        }
    }

    #[test]
    fn reviewed_pronouns_are_exact_first_with_productive_complete_backgrounds() {
        let relative = Pronoun::resolve("иже").expect("reviewed relative pronoun");
        let generated_dual = relative
            .form(PronounCell {
                case: Case::Dative,
                number: Number::Dual,
                gender: Some(Gender::Feminine),
                person: None,
                animacy: Animacy::Inanimate,
            })
            .expect("source-licensed dual relative cell");
        assert_eq!(
            generated_dual.texts().collect::<Vec<_>>(),
            ["имаже", "нимаже"]
        );
        assert!(matches!(
            &generated_dual.primary().source,
            FormSource::SynodalNormativeGeneration { rule }
                if rule.as_ref() == "SYN-PRONOUN-DERIVED-ALYPY-46-48"
        ));

        let exact = relative
            .form(PronounCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Some(Gender::Masculine),
                person: None,
                animacy: Animacy::Inanimate,
            })
            .expect("reviewed exact relative cell");
        assert!(matches!(
            exact.primary().source,
            FormSource::SynodalNormativeGeneration { ref rule }
                if rule.as_ref() == "SYN-REGISTRY-NORMATIVE-TABLE"
        ));

        let negative = Pronoun::resolve("никтоже").expect("reviewed negative pronoun");
        assert_eq!(
            negative
                .form(PronounCell {
                    case: Case::Instrumental,
                    number: Number::Singular,
                    gender: None,
                    person: None,
                    animacy: Animacy::Animate,
                })
                .expect("derived negative instrumental")
                .primary_text(),
            "никимъже"
        );
        assert!(negative.capabilities().productive_pronoun);

        assert_eq!(
            Pronoun::resolve("что")
                .expect("reviewed interrogative")
                .form(PronounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    gender: None,
                    person: None,
                    animacy: Animacy::Inanimate,
                })
                .expect("complete §48 genitive variants")
                .texts()
                .collect::<Vec<_>>(),
            ["чегѡ", "чесѡ", "чесогѡ"]
        );
    }

    #[test]
    fn alpy_45_48_source_union_pronouns_route_through_productive_classes() {
        let cases = [
            (
                "synodal:pronoun:sei",
                PronounCell {
                    case: Case::Dative,
                    number: Number::Dual,
                    gender: Some(Gender::Feminine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["сима"],
            ),
            (
                "synodal:pronoun:v07-97002c43d9dd87c3",
                PronounCell {
                    case: Case::Instrumental,
                    number: Number::Singular,
                    gender: Some(Gender::Neuter),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["овѣмъ"],
            ),
            (
                "synodal:pronoun:wikt-abc6b7472112",
                PronounCell {
                    case: Case::Locative,
                    number: Number::Plural,
                    gender: Some(Gender::Masculine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["инѣхъ"],
            ),
            (
                "synodal:pronoun:elik",
                PronounCell {
                    case: Case::Locative,
                    number: Number::Singular,
                    gender: Some(Gender::Masculine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["єлицѣ", "єлицѣмъ", "єликомъ"],
            ),
            (
                "synodal:pronoun:kiizhdo",
                PronounCell {
                    case: Case::Dative,
                    number: Number::Plural,
                    gender: Some(Gender::Neuter),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["кіимъждо"],
            ),
            (
                "synodal:pronoun:nekii",
                PronounCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    gender: Some(Gender::Feminine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["нѣкіихъ", "нѣкихъ"],
            ),
            (
                "synodal:pronoun:yakov",
                PronounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    gender: Some(Gender::Masculine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                vec!["ꙗкова", "ꙗковогѡ"],
            ),
        ];
        for (id, cell, expected) in cases {
            let pronoun = Pronoun::from_id(&LexemeId::from(id)).expect("source-union pronoun");
            assert!(pronoun.capabilities().productive_pronoun, "{id}");
            assert_eq!(
                pronoun
                    .form(cell)
                    .expect("source-licensed productive cell")
                    .texts()
                    .collect::<Vec<_>>(),
                expected,
                "{id}"
            );
        }

        let agreeing_citations = [
            ("synodal:pronoun:chii", "чій"),
            // Exact target evidence precedes the productive demonstrative
            // citation and preserves its source positional omega.
            ("synodal:pronoun:on", "ѡнъ"),
            ("synodal:pronoun:demonstrative-onyi", "оный"),
            ("synodal:pronoun:elikii", "єликїй"),
            ("synodal:pronoun:inyi", "иный"),
            ("synodal:pronoun:kakii", "какій"),
            ("synodal:pronoun:kakov", "каковъ"),
            ("synodal:pronoun:kakovyi", "каковый"),
            ("synodal:pronoun:kolik", "коликъ"),
            ("synodal:pronoun:kolikii", "коликїй"),
            ("synodal:pronoun:kotoryi", "который"),
            ("synodal:pronoun:nikotoryi", "никоторый"),
            ("synodal:pronoun:ovyi", "овый"),
            ("synodal:pronoun:sitsevyi", "сицевый"),
            ("synodal:pronoun:takii", "такій"),
            ("synodal:pronoun:takov", "таковъ"),
            ("synodal:pronoun:takovyi", "таковый"),
            ("synodal:pronoun:tolik", "толикъ"),
            ("synodal:pronoun:tolikii", "толикїй"),
            ("synodal:pronoun:yak", "ꙗкъ"),
            ("synodal:pronoun:yakii", "ꙗкій"),
            ("synodal:pronoun:yakov", "ꙗковъ"),
            ("synodal:pronoun:yakovyi", "ꙗковый"),
        ];
        for (id, citation) in agreeing_citations {
            let pronoun = Pronoun::from_id(&LexemeId::from(id)).expect("Alypy §45 identity");
            assert_eq!(
                pronoun
                    .form(PronounCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Some(Gender::Masculine),
                        person: None,
                        animacy: Animacy::Inanimate,
                    })
                    .expect("citation cell")
                    .primary_text(),
                citation,
                "{id}"
            );
        }

        let clitic = Pronoun::from_id(&LexemeId::from("synodal:pronoun:wikt-7c6914eff782"))
            .expect("reviewed reflexive clitic");
        assert_eq!(
            clitic
                .form(PronounCell {
                    case: Case::Dative,
                    number: Number::Singular,
                    gender: None,
                    person: None,
                    animacy: Animacy::Inanimate,
                })
                .expect("reflexive dative clitic")
                .primary_text(),
            "си"
        );
    }

    #[test]
    fn demonstrative_siya_is_not_attached_to_the_reflexive_pronoun() {
        let contaminated_cell = PronounCell {
            case: Case::Accusative,
            number: Number::Singular,
            gender: None,
            person: Some(Person::Third),
            animacy: Animacy::Inanimate,
        };
        assert!(matches!(
            Pronoun::resolve("себе")
                .expect("reviewed reflexive pronoun")
                .form(contaminated_cell),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let demonstrative = Pronoun::from_id(&LexemeId::from("synodal:pronoun:sei"))
            .expect("reviewed demonstrative pronoun");
        assert_eq!(
            demonstrative
                .form(PronounCell {
                    case: Case::Accusative,
                    number: Number::Plural,
                    gender: Some(Gender::Feminine),
                    person: None,
                    animacy: Animacy::Inanimate,
                })
                .expect("reviewed demonstrative cell")
                .primary_text(),
            "сїѧ"
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
            Err(Error::HistoricallyInvalidCell { .. })
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

    #[test]
    fn registered_impersonal_verbs_preserve_exact_first_typed_defectiveness() {
        let podobati = Verb::from_id(&LexemeId::from("synodal:verb:v06-7572c074fcdb7753"))
            .expect("reviewed impersonal verb");
        assert_eq!(
            podobati
                .present(Person::Third, Number::Singular)
                .expect("reviewed exact form")
                .primary_text(),
            "подобаетъ"
        );
        assert_eq!(
            podobati
                .infinitive()
                .expect("cited lexical infinitive")
                .primary_text(),
            "подобати"
        );
        assert!(matches!(
            podobati.present(Person::First, Number::Singular),
            Err(Error::HistoricallyInvalidCell { reason })
                if reason.contains("§104") && reason.contains("only third-singular")
        ));
        assert_eq!(
            podobati
                .paradigm(FiniteTense::Present)
                .with_status(ParadigmStatus::HistoricallyInvalid)
                .count(),
            8
        );

        let dovleti = Verb::from_id(&LexemeId::from("synodal:verb:v07-15e3f31915cf0144"))
            .expect("reviewed evidence-bounded verb");
        assert_eq!(
            dovleti
                .present(Person::Third, Number::Singular)
                .expect("reviewed exact form")
                .primary_text(),
            "довлѣетъ"
        );
        assert_eq!(
            dovleti
                .infinitive()
                .expect("cited lexical infinitive")
                .primary_text(),
            "довлѣти"
        );
        assert!(matches!(
            dovleti.present(Person::Third, Number::Plural),
            Err(Error::EvidenceIncompleteCell {
                field: MetadataField::IrregularOverride,
                reason,
            }) if reason.contains("prints only the singular surface")
        ));
        assert!(matches!(
            dovleti.imperfect(Person::Third, Number::Singular),
            Err(Error::EvidenceIncompleteCell {
                field: MetadataField::IrregularOverride,
                reason,
            }) if reason.contains("all other uncited cells")
        ));
    }

    #[test]
    fn alpy_103_archaic_verbs_have_closed_exact_present_and_imperative_tables() {
        let cases = [
            (
                "synodal:verb:wikt-6ceeefbe4e9e",
                "ꙗмъ",
                "ꙗдѧтъ",
                "ꙗждь",
                "ꙗдитѣ",
            ),
            (
                "synodal:verb:wikt-8a084860d2ef",
                "вѣмъ",
                "вѣдѧтъ",
                "вѣждь",
                "вѣдитѣ",
            ),
            (
                "synodal:verb:wikt-0c6c8db63b7c",
                "имамъ",
                "имꙋтъ",
                "имѣй",
                "имѣитѣ",
            ),
            ("synodal:verb:imati", "имамъ", "имꙋтъ", "имѣй", "имѣитѣ"),
        ];
        for (id, first_singular, third_plural, imperative_singular, imperative_dual_variant) in
            cases
        {
            let verb = Verb::from_id(&LexemeId::from(id)).expect("reviewed archaic identity");
            let present = verb.paradigm(FiniteTense::Present);
            assert_eq!(present.iter().count(), 9, "{id}");
            assert_eq!(present.failures().count(), 0, "{id}");
            assert_eq!(
                verb.present(Person::First, Number::Singular)
                    .expect("first singular")
                    .primary_text(),
                first_singular,
                "{id}"
            );
            assert_eq!(
                verb.present(Person::Third, Number::Plural)
                    .expect("third plural")
                    .primary_text(),
                third_plural,
                "{id}"
            );
            let imperative = verb.system_paradigm(VerbSystem::Imperative);
            assert_eq!(imperative.successes().count(), 7, "{id}");
            assert_eq!(imperative.failures().count(), 2, "{id}");
            assert_eq!(
                verb.imperative(Person::Second, Number::Singular)
                    .expect("second singular imperative")
                    .primary_text(),
                imperative_singular,
                "{id}"
            );
            let dual = verb
                .imperative(Person::Third, Number::Dual)
                .expect("source-licensed third-dual imperative");
            assert_eq!(dual.variants().len(), 2, "{id}");
            assert!(
                dual.variants()
                    .iter()
                    .any(|variant| variant.expanded == imperative_dual_variant),
                "{id}"
            );
        }

        let dati = Verb::from_id(&LexemeId::from("synodal:verb:dati")).expect("дати");
        assert_eq!(
            dati.future(Person::First, Number::Dual)
                .expect("dual future")
                .variants()
                .len(),
            2
        );
        let byti = Verb::from_id(&LexemeId::from("synodal:verb:byti")).expect("быти");
        assert_eq!(
            byti.system_paradigm(VerbSystem::Imperative)
                .successes()
                .count(),
            6
        );
        assert_eq!(
            byti.present(Person::First, Number::Dual)
                .expect("dual present")
                .variants()
                .len(),
            2
        );

        for id in ["synodal:verb:imati", "synodal:verb:wikt-0c6c8db63b7c"] {
            let verb = Verb::from_id(&LexemeId::from(id)).expect("имати/имѣти identity");
            let passive = verb.system_paradigm(VerbSystem::Participle {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Passive,
                form: AdjectiveForm::Long,
            });
            assert_eq!(passive.successes().count(), 0, "{id}");
            assert_eq!(
                passive
                    .with_status(ParadigmStatus::HistoricallyInvalid)
                    .count(),
                72,
                "{id}"
            );
            assert!(passive.iter().all(|row| matches!(
                row.outcome(),
                Err(Error::HistoricallyInvalidCell { reason })
                    if reason.contains("§103") && reason.contains("present passive")
            )));
        }
    }

    #[test]
    fn archaic_principal_parts_cover_every_source_licensed_participle_system() {
        let yasti = Verb::from_id(&LexemeId::from("synodal:verb:wikt-6ceeefbe4e9e")).expect("ꙗсти");
        for (tense, voice) in [
            (ParticipleTense::Present, ParticipleVoice::Active),
            (ParticipleTense::Present, ParticipleVoice::Passive),
            (ParticipleTense::Past, ParticipleVoice::Active),
            (ParticipleTense::Past, ParticipleVoice::Passive),
        ] {
            let paradigm = yasti.system_paradigm(VerbSystem::Participle {
                tense,
                voice,
                form: AdjectiveForm::Long,
            });
            assert_eq!(paradigm.successes().count(), 72, "{tense:?} {voice:?}");
            assert_eq!(paradigm.failures().count(), 0, "{tense:?} {voice:?}");
        }
        let cited = Participle::from_id(yasti.id())
            .expect("participle handle")
            .form(ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            })
            .expect("short present active citation");
        assert_eq!(cited.primary_text(), "ꙗдый");

        let vedeti = Participle::from_id(&LexemeId::from("synodal:verb:wikt-8a084860d2ef"))
            .expect("вѣдѣти participle");
        assert_eq!(
            vedeti
                .form(ParticipleCell {
                    tense: ParticipleTense::Present,
                    voice: ParticipleVoice::Passive,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                })
                .expect("present passive citation")
                .primary_text(),
            "вѣдомъ"
        );
    }

    #[test]
    fn alpy_104_irregular_inventory_is_public_exhaustive_and_source_ordered() {
        let inventory = irregular_verb_inventory().expect("validated generated inventory");
        assert_eq!(inventory.len(), 98);
        assert_eq!(inventory.first().map(|entry| entry.source_order), Some(2));
        assert_eq!(inventory.last().map(|entry| entry.source_order), Some(100));
        assert!(inventory.windows(2).all(|pair| {
            pair[0].source_order < pair[1].source_order && pair[1].source_order != 97
        }));
        assert_eq!(
            inventory
                .iter()
                .filter(|entry| entry.implementation_status == "source-evidence-incomplete")
                .map(|entry| entry.source_order)
                .collect::<Vec<_>>(),
            vec![4, 32, 55, 73]
        );
        assert!(inventory.iter().all(|entry| {
            matches!(
                entry.implementation_status.as_str(),
                "implemented-bundled"
                    | "implemented-by-metadata-contract"
                    | "source-evidence-incomplete"
            )
        }));
        let yati = inventory
            .iter()
            .find(|entry| entry.source_order == 100)
            .expect("prefixed ꙗти family");
        assert!(yati.systems.contains(&"stem-alternation".into()));
        assert_eq!(yati.strategy, "caller-exact-principal-parts");

        let systems = |order| {
            inventory
                .iter()
                .find(|entry| entry.source_order == order)
                .map(|entry| entry.systems.as_slice())
                .expect("reviewed source order")
        };
        assert_eq!(systems(20), ["future", "aorist"]);
        assert!(systems(11).contains(&"past-active-participle".into()));
        for order in [18, 19, 22, 23, 72] {
            assert!(systems(order).contains(&"present".into()), "order {order}");
        }
        for order in [18, 41, 92] {
            assert!(
                systems(order).contains(&"stem-alternation".into()),
                "order {order}"
            );
        }
        assert!(systems(69).contains(&"defectiveness".into()));
    }

    #[test]
    fn curated_possessives_israel_and_thousand_use_productive_backgrounds() {
        let bozhii = Adjective::from_id(&LexemeId::from("synodal:adjective:bozhii"))
            .expect("typed -їй possessive");
        assert_eq!(
            bozhii
                .form(AdjectiveCell {
                    case: Case::Genitive,
                    number: Number::Dual,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                })
                .expect("productive source-table cell")
                .primary_text(),
            "божїю"
        );

        let gospoden = Adjective::from_id(&LexemeId::from("synodal:adjective:gospoden"))
            .expect("typed soft possessive");
        assert_eq!(
            gospoden
                .form(AdjectiveCell {
                    case: Case::Dative,
                    number: Number::Plural,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                })
                .expect("productive soft possessive cell")
                .primary_text(),
            "господнимъ"
        );

        let israel_adjective = Adjective::from_id(&LexemeId::from("synodal:adjective:v06-israel"))
            .expect("typed hard possessive");
        let exact = israel_adjective
            .form(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            })
            .expect("independently attested exceptional compound cell");
        assert_eq!(exact.primary_text(), "израилевыхъ");
        assert!(matches!(
            exact.primary().source,
            FormSource::SynodalAttestation { .. }
        ));
        assert!(matches!(
            israel_adjective.form(AdjectiveCell {
                case: Case::Instrumental,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let israel = Noun::from_id(&LexemeId::from("synodal:noun:v06-israel"))
            .expect("typed proper-name noun");
        assert_eq!(
            israel
                .form(Case::Vocative, Number::Singular, Animacy::Animate)
                .expect("productive singular vocative")
                .primary_text(),
            "израилю"
        );
        assert!(matches!(
            israel.form(Case::Nominative, Number::Plural, Animacy::Animate),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let thousand = Noun::from_id(&LexemeId::from("synodal:noun:v06-tysyashcha"))
            .expect("typed magnitude noun");
        assert_eq!(
            thousand
                .form(Case::Instrumental, Number::Singular, Animacy::Inanimate)
                .expect("productive second-declension cell")
                .primary_text(),
            "тысѧщою"
        );
        assert_eq!(
            thousand
                .form(Case::Genitive, Number::Plural, Animacy::Inanimate)
                .expect("exact source cell remains first")
                .primary_text(),
            "тысѧщъ"
        );
    }

    #[test]
    fn alpy_104_remaining_curated_verbs_have_typed_complete_backgrounds() {
        let dostoyati = Verb::from_id(&LexemeId::from("synodal:verb:dostoyati"))
            .expect("typed defective modal");
        assert_eq!(
            dostoyati
                .present(Person::Third, Number::Singular)
                .expect("source-listed present")
                .primary_text(),
            "достоитъ"
        );
        assert_eq!(
            dostoyati
                .imperfect(Person::Third, Number::Singular)
                .expect("source-listed imperfect")
                .primary_text(),
            "достоѧше"
        );
        assert!(matches!(
            dostoyati.present(Person::First, Number::Singular),
            Err(Error::HistoricallyInvalidCell { reason })
                if reason.contains("§104") && reason.contains("third-person singular")
        ));
        assert!(matches!(
            dostoyati.imperative(Person::Second, Number::Singular),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let iziti = Verb::from_id(&LexemeId::from("synodal:verb:v06-iziti"))
            .expect("typed prefixed motion verb");
        assert_eq!(
            iziti
                .future(Person::Second, Number::Singular)
                .expect("productive future")
                .primary_text(),
            "изыдеши"
        );
        assert_eq!(
            iziti
                .aorist(Person::First, Number::Plural)
                .expect("productive consonant aorist")
                .primary_text(),
            "изыдохомъ"
        );
        assert_eq!(
            iziti
                .imperative(Person::Second, Number::Plural)
                .expect("productive imperative")
                .primary_text(),
            "изыдите"
        );
        assert_eq!(
            iziti
                .l_participle(Gender::Masculine, Number::Singular)
                .expect("mobile-vowel masculine l-participle")
                .primary_text(),
            "изшелъ"
        );
        assert_eq!(
            iziti
                .l_participle(Gender::Feminine, Number::Singular)
                .expect("zero-grade feminine l-participle")
                .primary_text(),
            "изшла"
        );
        let iziti_participle = Participle::from_id(iziti.id()).expect("past-active handle");
        assert_eq!(
            iziti_participle
                .form(ParticipleCell {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Active,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Feminine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                })
                .expect("productive past-active participle")
                .primary_text(),
            "изшедши"
        );

        let vzyti = Verb::from_id(&LexemeId::from("synodal:verb:v06-vzyti"))
            .expect("typed prefixed motion verb");
        assert_eq!(
            vzyti
                .future(Person::First, Number::Plural)
                .expect("productive future")
                .primary_text(),
            "взыдемъ"
        );
        assert_eq!(
            vzyti
                .aorist(Person::Second, Number::Plural)
                .expect("productive consonant aorist")
                .primary_text(),
            "взыдосте"
        );
        assert_eq!(
            vzyti
                .l_participle(Gender::Masculine, Number::Plural)
                .expect("zero-grade plural l-participle")
                .primary_text(),
            "возшли"
        );

        let vzeti = Verb::from_id(&LexemeId::from("synodal:verb:v06-vzeti"))
            .expect("typed suppletive take verb");
        assert_eq!(
            vzeti
                .present(Person::First, Number::Singular)
                .expect("suppletive present first singular")
                .primary_text(),
            "вземлю"
        );
        assert_eq!(
            vzeti
                .present(Person::Second, Number::Singular)
                .expect("suppletive present medial cell")
                .primary_text(),
            "вземлеши"
        );
        assert_eq!(
            vzeti
                .present(Person::Third, Number::Plural)
                .expect("suppletive present third plural")
                .primary_text(),
            "вземлютъ"
        );
        assert_eq!(
            vzeti
                .future(Person::First, Number::Singular)
                .expect("suppletive future first singular")
                .primary_text(),
            "возмꙋ"
        );
        assert_eq!(
            vzeti
                .future(Person::Third, Number::Singular)
                .expect("productive suppletive future")
                .primary_text(),
            "возметъ"
        );
        assert_eq!(
            vzeti
                .future(Person::Third, Number::Plural)
                .expect("suppletive future third plural")
                .primary_text(),
            "возмꙋтъ"
        );
        assert_eq!(
            vzeti
                .aorist(Person::Second, Number::Plural)
                .expect("productive vowel aorist")
                .primary_text(),
            "взѧсте"
        );
        assert_eq!(
            vzeti
                .imperative(Person::Second, Number::Plural)
                .expect("productive suppletive imperative")
                .primary_text(),
            "возмите"
        );
        assert_eq!(
            vzeti
                .imperative(Person::First, Number::Plural)
                .expect("first-conjugation imperative series")
                .primary_text(),
            "возмемъ"
        );
        assert_eq!(
            vzeti
                .l_participle(Gender::Feminine, Number::Singular)
                .expect("productive l-participle")
                .primary_text(),
            "взѧла"
        );
        let vzeti_participle = Participle::from_id(vzeti.id()).expect("participle handle");
        assert_eq!(
            vzeti_participle
                .form(ParticipleCell {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Passive,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                })
                .expect("productive past-passive participle")
                .primary_text(),
            "взѧтый"
        );
        assert_eq!(
            verbal_noun(
                "възѧти",
                NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                },
            )
            .expect("productive -їе verbal noun")
            .primary_text(),
            "взѧтїѧ"
        );

        for verb in [&iziti, &vzyti] {
            for system in [
                VerbSystem::Finite(FiniteTense::Future),
                VerbSystem::Finite(FiniteTense::Aorist),
                VerbSystem::Imperative,
                VerbSystem::LParticiple,
                VerbSystem::Participle {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Active,
                    form: AdjectiveForm::Short,
                },
                VerbSystem::Participle {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Active,
                    form: AdjectiveForm::Long,
                },
            ] {
                assert!(
                    verb.missing_principal_parts(system)
                        .expect("typed metadata query")
                        .is_empty(),
                    "{} {system:?}",
                    verb.id()
                );
            }
        }
        for system in [
            VerbSystem::Finite(FiniteTense::Future),
            VerbSystem::Finite(FiniteTense::Aorist),
            VerbSystem::Imperative,
            VerbSystem::LParticiple,
            VerbSystem::Participle {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Active,
                form: AdjectiveForm::Short,
            },
            VerbSystem::Participle {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Active,
                form: AdjectiveForm::Long,
            },
            VerbSystem::Participle {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                form: AdjectiveForm::Short,
            },
            VerbSystem::Participle {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                form: AdjectiveForm::Long,
            },
            VerbSystem::VerbalNoun {
                animacy: Animacy::Inanimate,
            },
        ] {
            assert!(
                vzeti
                    .missing_principal_parts(system)
                    .expect("typed metadata query")
                    .is_empty(),
                "{system:?}"
            );
        }
    }
}
