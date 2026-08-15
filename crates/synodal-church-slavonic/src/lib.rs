#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abbreviation;
mod handles;
mod inflector;
mod kernel;
mod paradigm;
pub mod phrases;
mod provider;
mod registry;
mod resolver;
mod spec;

pub use abbreviation::Abbreviation;
pub use handles::{Adjective, Capabilities, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use inflector::{Inflector, InflectorBuilder};
pub use paradigm::{Paradigm, ParadigmIdentity, ParadigmRow, ParadigmStatus};
pub use provider::{
    BatchLexeme, BatchRequest, BatchResult, BatchRow, InMemoryLexemeProvider, LexemeProvider,
    Lexicon, ProviderLexeme, StaticLexemeProvider,
};
pub use registry::{
    AccentParadigmSummary, AccentSummary, AlignmentSummary, ExactFormSummary,
    IrregularOverrideSummary, LexemeSummary, LexicalMetadataSummary, NounRestrictionSummary,
    PartOfSpeech, PositionalRuleSummary, PrincipalPartSummary, RecensionConflictSummary,
    TransformationRuleSummary,
};
pub use spec::{
    AdjectiveSpec, DefectKind, DefectiveCell, LexemeSpec, NounSpec, PronounSpec,
    SpecificationSource, SpecifiedForm, VerbSpec, VerbSpecBuilder,
};
pub use synodal_church_slavonic_core as core;
pub use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope,
    ActiveParticipleShortFormation, AdjectiveClass, AoristFormation, Aspect, AuthorityRole,
    BreathingMark, BreathingRule, ComparisonFormation, EpistemicRole, Evidence, EvidenceId,
    EvidenceKind, ImperativeFormation, ImperfectFormation, NounDeclension, NounNumberInventory,
    ParticiplePrincipalPart, PresentPrincipalParts, RuleId, SourceId, VerbConjugation,
};
pub use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AnalyticConstruction, Animacy, Case, CollationKey,
    CollationProfile, CollationStrength, Comparison, Confidence, CyrillicNumeral, Error, ErrorCode,
    FiniteTense, FiniteVerbCell, FormSet, FormSource, Gender, GenerationPolicy, GrammarCell,
    ImperativeCell, InitialPresentation, LParticipleCell, LexemeId, Loss, MetadataField,
    NegativePronounBase, NounCell, Number, NumeralCell, NumeralKind, OrthographyProfile,
    ParticipleCell, ParticipleTense, ParticipleVoice, Person, PhraseRole, PhraseToken, PronounCell,
    PronounCliticProsody, PronounDeclension, PronounEnvironment, PronounFormSelection,
    PronounNumberInventory, PronounPostpositive, PronounPrefix, RealizedPhrase, Recension,
    RenderedText, Result, SynodalWord, TransliterationScheme, VariantPolicy, VerbSystem,
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
            .past(Person::Third, Number::Singular)
            .expect("reviewed exact past form");
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
        assert!(matches!(
            Noun::resolve_with(
                "камень",
                Inflector::builder()
                    .orthography(OrthographyProfile::SynodalLiturgical)
                    .build(),
            )
            .expect("registered noun")
            .form(Case::Dative, Number::Plural, Animacy::Inanimate),
            Err(Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm
            })
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
            ["дщерїй", "дщерей"]
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
            let spec = NounSpec::new(lemma, stem, gender, declension, source)
                .expect("typed noun")
                .with_accent_paradigm(accent)
                .expect("accent contract");
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
            .expect("number restriction metadata");
        assert_eq!(restriction.number_inventory, "plural-only");
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

        let exact_past = Verb::from_id(&LexemeId::from("synodal:verb:wikt-78da2d05497d"))
            .expect("reviewed exact-past verb");
        assert!(exact_past.capabilities().past);
        assert!(!exact_past.capabilities().future);
        assert!(!exact_past.capabilities().infinitive);

        let sparse_exact = Verb::from_id(&LexemeId::from("synodal:verb:v06-vzeti"))
            .expect("reviewed sparse exact verb");
        let missing = sparse_exact.missing_metadata();
        assert!(missing.contains(&core::MetadataField::AoristStem));
        assert!(missing.contains(&core::MetadataField::ImperativeStem));
        assert!(missing.contains(&core::MetadataField::ParticipleStem));

        assert!(
            Determiner::from_id(&LexemeId::from("synodal:determiner:sam"))
                .expect("productive determiner")
                .capabilities()
                .productive_adjective
        );
        assert!(
            Numeral::from_id(&LexemeId::from("synodal:numeral:pervyi"))
                .expect("productive ordinal")
                .capabilities()
                .productive_adjective
        );
        assert!(
            !Numeral::from_id(&LexemeId::from("synodal:numeral:dva"))
                .expect("exact cardinal")
                .capabilities()
                .productive_adjective
        );
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
