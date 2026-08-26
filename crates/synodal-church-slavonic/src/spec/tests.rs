use super::*;
use crate::ParadigmStatus;
use crate::{Inflector, Paradigm};
use std::collections::BTreeSet;
use synodal_church_slavonic_core::{
    AccentMark, AccentPlacement, AccentRule, AccentScope, AdjectiveCell, Animacy, Case, FormSource,
    InitialPresentation, MetadataField, NounCell, Number, NumeralCell, NumeralKind, ParticipleCell,
    Person, PositionalOperation,
};
use synodal_church_slavonic_core::{
    AccentParadigm, ActiveParticipleShortFormation, AdjectiveClass, AdjectiveForm, Aspect,
    AuthorityRole, Comparison, ComparisonFormation, DeterminerDeclension,
    DeterminerNumberInventory, EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind,
    FiniteTense, Gender, GrammarCell, ImperativeFormation, ImperfectFormation, NounDeclension,
    NumeralDeclension, OrthographyProfile, ParticiplePrincipalPart, ParticipleTense,
    ParticipleVoice, PositionalParadigm, PositionalRule, PronounDeclension, PronounFormSelection,
    PronounPostpositive, Recension, SourceId, SynodalWord, VerbConjugation, VerbSystem,
};

fn source() -> SpecificationSource {
    SpecificationSource::new(
        "caller-lexicon-entry",
        "caller-reviewed-lexicon",
        "caller lexicon, entry 1",
    )
    .expect("source")
}

fn preserve_positional() -> PositionalParadigm {
    source().positional_paradigm(
        "caller-positional-preserve",
        vec![PositionalRule {
            scope: AccentScope::All,
            operations: vec![],
        }],
    )
}

fn mudr_accent() -> AccentParadigm {
    AccentParadigm {
        id: "synodal-accent:mudr-fixed-stem".into(),
        accent_rules: vec![AccentRule {
            scope: AccentScope::Adjective {
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
                numbers: vec![Number::Singular],
            },
            placement: AccentPlacement::StemVowelFromStart(0),
            mark: AccentMark::Acute,
        }],
        breathing_rules: vec![],
        evidence: Evidence {
            id: EvidenceId::from("alypy-57-mudryi"),
            source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::AccentParadigm,
            authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: "Alypy (Gamanovich), §57, мꙋ́дръ adjective paradigm".into(),
            note: None,
        },
    }
}

#[test]
fn unregistered_noun_uses_typed_metadata_without_lookup() {
    let spec = NounSpec::new(
        "псалтирникъ",
        "псалтирник",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source(),
    )
    .expect("valid spec");
    assert!(Inflector::default().resolve("псалтирникъ").is_err());
    let forms = spec
        .form(NounCell {
            case: Case::Dative,
            number: Number::Plural,
            animacy: Animacy::Animate,
        })
        .expect("productive form");
    assert_eq!(forms.primary_text(), "псалтирникомъ");
    assert!(matches!(
        forms.primary().source,
        FormSource::CallerSpecifiedPrediction { .. }
    ));
    assert_eq!(spec.paradigm(Animacy::Animate).iter().count(), 21);
}

#[test]
fn unregistered_numeral_uses_the_same_typed_kernel_as_registry_numerals() {
    let spec = NumeralSpec::new(
        "девѧть",
        "девѧт",
        NumeralDeclension::CardinalIStem,
        source(),
    )
    .expect("valid numeral spec");
    let genitive = spec
        .form(NumeralCell {
            kind: NumeralKind::Cardinal,
            case: Case::Genitive,
            number: Number::Singular,
            gender: None,
            animacy: Animacy::Inanimate,
        })
        .expect("productive numeral cell");
    assert_eq!(genitive.primary_text(), "девѧти");
    assert!(matches!(
        genitive.primary().source,
        FormSource::CallerSpecifiedPrediction { .. }
    ));
    assert!(matches!(
        spec.form(NumeralCell {
            kind: NumeralKind::Cardinal,
            case: Case::Accusative,
            number: Number::Plural,
            gender: None,
            animacy: Animacy::Inanimate,
        }),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn explicit_short_comparison_and_active_participle_close_productive_gaps() {
    let adjective = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
        .expect("adjective")
        .comparison("мꙋдрѣйш", ComparisonFormation::LaterYat)
        .expect("comparison metadata");
    let comparison = adjective
        .form(AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Comparative,
        })
        .expect("short comparison");
    assert_eq!(comparison.primary_text(), "мꙋдрѣй");

    let short_superlative = adjective
        .form(AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Superlative,
        })
        .expect("predicate short superlative");
    assert_eq!(
        short_superlative.texts().collect::<Vec<_>>(),
        ["мꙋдрѣйшъ", "мꙋдрѣй"]
    );
    let short_superlative_paradigm =
        adjective.paradigm(AdjectiveForm::Short, Comparison::Superlative);
    assert_eq!(short_superlative_paradigm.successes().count(), 9);
    assert_eq!(
        short_superlative_paradigm
            .with_status(ParadigmStatus::HistoricallyInvalid)
            .count(),
        63
    );

    let present_part = ParticiplePrincipalPart {
        short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
        short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
        long_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
        class: AdjectiveClass::Hard,
    };
    let verb = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .present_stem("нес")
    .expect("stem")
    .present_first_singular("несꙋ")
    .expect("edge")
    .present_third_plural("несꙋтъ")
    .expect("edge")
    .present_active_participle(present_part)
    .build()
    .expect("verb");
    let finite = verb
        .form(GrammarCell::FiniteVerb(
            synodal_church_slavonic_core::FiniteVerbCell {
                tense: FiniteTense::Present,
                person: Person::First,
                number: Number::Singular,
            },
        ))
        .expect("present");
    assert_eq!(finite.primary_text(), "несꙋ");
    let participle = verb
        .form(GrammarCell::Participle(ParticipleCell {
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
        }))
        .expect("short active participle");
    assert_eq!(participle.texts().collect::<Vec<_>>(), ["несый", "несꙋщь"]);
}

#[test]
fn explicit_pronoun_specs_preserve_profiles_clitics_and_context() {
    let possessive = PronounSpec::regular("твой", "тво", PronounDeclension::Soft, source())
        .expect("regular soft pronoun");
    let form = possessive
        .form(synodal_church_slavonic_core::PronounCell {
            case: Case::Genitive,
            number: Number::Singular,
            gender: Some(Gender::Feminine),
            person: None,
            animacy: Animacy::Inanimate,
        })
        .expect("soft-pronoun genitive");
    assert_eq!(form.primary_text(), "твоеѧ");
    assert!(matches!(
        &form.primary().source,
        FormSource::CallerSpecifiedPrediction { .. }
    ));
    let paradigm = possessive.paradigm();
    assert_eq!(paradigm.successes().count(), 108);
    assert_eq!(
        paradigm
            .with_status(ParadigmStatus::HistoricallyInvalid)
            .count(),
        18
    );

    let clitic = PronounSpec::closed("азъ", PronounDeclension::PersonalFirst, source())
        .expect("first-person specification")
        .with_selection(PronounFormSelection::Enclitic)
        .expect("clitic selection");
    assert_eq!(
        clitic
            .form(synodal_church_slavonic_core::PronounCell {
                case: Case::Dative,
                number: Number::Singular,
                gender: None,
                person: Some(Person::First),
                animacy: Animacy::Inanimate,
            })
            .expect("first-person enclitic")
            .primary_text(),
        "ми"
    );

    let relative = PronounSpec::closed("иже", PronounDeclension::ThirdPerson, source())
        .expect("third-person base")
        .with_postpositive(PronounPostpositive::Zhe)
        .expect("relative composition");
    assert_eq!(
        relative
            .form(synodal_church_slavonic_core::PronounCell {
                case: Case::Nominative,
                number: Number::Plural,
                gender: Some(Gender::Feminine),
                person: None,
                animacy: Animacy::Inanimate,
            })
            .expect("relative nominative")
            .primary_text(),
        "ꙗже"
    );
}

#[test]
fn explicit_determiner_specs_preserve_class_and_number_restrictions() {
    let vsyak = DeterminerSpec::new("всѧкъ", "всѧк", DeterminerDeclension::VsyakMixed, source())
        .expect("mixed determiner specification");
    let generated = vsyak
        .form(AdjectiveCell {
            case: Case::Dative,
            number: Number::Singular,
            gender: Gender::Feminine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
        })
        .expect("licensed mixed cell");
    assert_eq!(generated.texts().collect::<Vec<_>>(), ["всѧцѣй", "всѧкой"]);
    assert!(matches!(
        generated.primary().source,
        FormSource::CallerSpecifiedPrediction { .. }
    ));
    let paradigm = vsyak.paradigm(AdjectiveForm::Short);
    assert_eq!(paradigm.successes().count(), 48);
    assert_eq!(
        paradigm
            .with_status(ParadigmStatus::HistoricallyInvalid)
            .count(),
        24
    );

    assert!(matches!(
        DeterminerSpec::new("всѧкъ", "всѧк", DeterminerDeclension::VsyakMixed, source(),)
            .expect("default no-dual specification")
            .with_number_inventory(DeterminerNumberInventory::All),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn productive_pronominal_tables_match_every_reviewed_exact_cell() {
    for (lemma, stem, declension) in [
        ("мой", "мо", PronounDeclension::Soft),
        ("твой", "тво", PronounDeclension::Soft),
        ("свой", "сво", PronounDeclension::Soft),
        ("нашъ", "наш", PronounDeclension::MixedPossessive),
        ("вашъ", "ваш", PronounDeclension::MixedPossessive),
        ("той", "т", PronounDeclension::Hard),
    ] {
        let explicit =
            PronounSpec::regular(lemma, stem, declension, source()).expect("regular pronoun spec");
        let reviewed = crate::Pronoun::resolve(lemma).expect("reviewed pronoun identity");
        for number in Number::ALL {
            for case in Case::ALL {
                if case == Case::Vocative {
                    continue;
                }
                for gender in Gender::ALL {
                    for animacy in if case == Case::Accusative {
                        Animacy::ALL.as_slice()
                    } else {
                        &[Animacy::Inanimate]
                    } {
                        let cell = synodal_church_slavonic_core::PronounCell {
                            case,
                            number,
                            gender: Some(gender),
                            person: None,
                            animacy: *animacy,
                        };
                        let mut predicted = explicit
                            .form(cell)
                            .expect("productive pronoun table cell")
                            .texts()
                            .map(str::to_owned)
                            .collect::<Vec<_>>();
                        let mut exact = reviewed
                            .form(cell)
                            .expect("reviewed exact pronoun table cell")
                            .texts()
                            .map(str::to_owned)
                            .collect::<Vec<_>>();
                        predicted.sort();
                        exact.sort();
                        assert_eq!(predicted, exact, "{lemma} {cell:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_accent_paradigm_realizes_multiple_cells() {
    let spec = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
        .expect("adjective")
        .with_accent_paradigm(mudr_accent())
        .expect("accent")
        .with_positional_paradigm(preserve_positional())
        .expect("positional contract");
    let inflector = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    for case in [Case::Genitive, Case::Dative, Case::Instrumental] {
        let forms = spec
            .form_with(
                inflector,
                AdjectiveCell {
                    case,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            )
            .expect("accented form");
        assert!(forms.primary_text().starts_with("мꙋ́др"));
        assert!(
            forms
                .primary()
                .evidence
                .iter()
                .any(|evidence| { evidence.kind == EvidenceKind::AccentParadigm })
        );
    }
}

#[test]
fn positional_realization_precedes_breathing_and_accent_for_arbitrary_lexemes() {
    let source = source();
    let accent = source.fixed_stem_accent(
        "ezero-initial-accent",
        AccentScope::Noun {
            numbers: Number::ALL.to_vec(),
        },
        0,
        AccentMark::Acute,
    );
    let positional = source.positional_paradigm(
        "ezero-initial-wide-e",
        vec![PositionalRule {
            scope: AccentScope::All,
            operations: vec![PositionalOperation::Initial(InitialPresentation::WideE)],
        }],
    );
    let spec = NounSpec::new(
        "езеро",
        "езер",
        Gender::Neuter,
        NounDeclension::FirstHardNeuter,
        source,
    )
    .expect("valid ezero noun specification")
    .with_accent_paradigm(accent)
    .expect("valid ezero accent paradigm")
    .with_positional_paradigm(positional)
    .expect("valid ezero positional paradigm");
    let cell = NounCell {
        case: Case::Nominative,
        number: Number::Singular,
        animacy: Animacy::Inanimate,
    };
    let forms = spec
        .form_with(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
            cell,
        )
        .expect("liturgical ezero form");
    assert_eq!(forms.primary().expanded, "езеро");
    assert_eq!(forms.primary_text(), "є҆́зеро");
    assert_eq!(
        forms
            .primary()
            .rule_trace
            .steps()
            .iter()
            .map(|step| step.stage.as_str())
            .filter(|stage| {
                matches!(
                    *stage,
                    "lexical-positional-realization" | "accent-paradigm-realization"
                )
            })
            .collect::<Vec<_>>(),
        [
            "lexical-positional-realization",
            "accent-paradigm-realization"
        ]
    );
}

#[test]
fn liturgical_specs_require_both_accent_and_positional_metadata() {
    let source = source();
    let accent =
        source.fixed_stem_accent("mudr-test-accent", AccentScope::All, 0, AccentMark::Acute);
    let spec = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source)
        .expect("valid mudr adjective specification")
        .with_accent_paradigm(accent)
        .expect("valid mudr accent paradigm");
    let error = spec
        .form_with(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            },
        )
        .expect_err("missing positional contract");
    assert!(matches!(
        error,
        Error::OrthographicMetadataRequired {
            field: MetadataField::PositionalParadigm
        }
    ));
    assert!(!LexemeSpec::from(spec).orthography_ready(OrthographyProfile::SynodalLiturgical));
}

#[test]
fn source_selected_wide_ending_is_applied_before_accent() {
    let source = source();
    let accent =
        source.fixed_stem_accent("rab-test-accent", AccentScope::All, 0, AccentMark::Acute);
    let positional = source.positional_paradigm(
        "rab-alypy-36-wide-ending",
        vec![PositionalRule {
            scope: AccentScope::All,
            operations: vec![PositionalOperation::WidePluralEnding],
        }],
    );
    let spec = NounSpec::new(
        "рабъ",
        "раб",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source,
    )
    .expect("valid rab noun specification")
    .with_accent_paradigm(accent)
    .expect("valid rab accent paradigm")
    .with_positional_paradigm(positional)
    .expect("valid rab positional paradigm");
    let forms = spec
        .form_with(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
            NounCell {
                case: Case::Dative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            },
        )
        .expect("liturgical rab dative plural");
    assert_eq!(forms.primary().expanded, "рабомъ");
    assert_eq!(forms.primary_text(), "ра́бѡмъ");
}

#[test]
fn noun_irregular_forms_cannot_bypass_lexical_inventories() {
    let inanimate = GrammarCell::Noun(NounCell {
        case: Case::Nominative,
        number: Number::Singular,
        animacy: Animacy::Inanimate,
    });
    let inanimate_form =
        SpecifiedForm::new(inanimate, "врагъ", None::<String>, source()).expect("form");
    let animate_only = NounSpec::new(
        "врагъ",
        "враг",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source(),
    )
    .expect("noun")
    .with_animacy_inventory(synodal_church_slavonic_core::NounAnimacyInventory::AnimateOnly)
    .expect("animate restriction");
    assert!(matches!(
        animate_only.with_irregular_form(inanimate_form),
        Err(Error::ContradictoryMetadata { .. })
    ));

    let plural = GrammarCell::Noun(NounCell {
        case: Case::Nominative,
        number: Number::Plural,
        animacy: Animacy::Animate,
    });
    let plural_form = SpecifiedForm::new(plural, "врази", None::<String>, source()).expect("form");
    let singular_only = NounSpec::new(
        "врагъ",
        "враг",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source(),
    )
    .expect("noun")
    .with_number_inventory(synodal_church_slavonic_core::NounNumberInventory::SingularOnly)
    .expect("singular restriction");
    assert!(matches!(
        singular_only.with_irregular_form(plural_form),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn exact_liturgical_override_precedes_reusable_positional_metadata() {
    let source = source();
    let cell = GrammarCell::Noun(NounCell {
        case: Case::Nominative,
        number: Number::Singular,
        animacy: Animacy::Inanimate,
    });
    let positional = source.positional_paradigm(
        "yazyk-people-rule",
        vec![PositionalRule {
            scope: AccentScope::All,
            operations: vec![PositionalOperation::Initial(InitialPresentation::IotatedYa)],
        }],
    );
    let exact = SpecifiedForm::new(cell, "ѧзыкъ", Some("ѧ҆зы́къ"), source.clone())
        .expect("valid exact yazyk form");
    let spec = NounSpec::new(
        "ѧзыкъ",
        "ѧзык",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source,
    )
    .expect("valid yazyk noun specification")
    .with_positional_paradigm(positional)
    .expect("valid yazyk positional paradigm")
    .with_irregular_form(exact)
    .expect("valid exact yazyk override");
    let forms = spec
        .form_with(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
            match cell {
                GrammarCell::Noun(cell) => cell,
                _ => unreachable!(),
            },
        )
        .expect("exact yazyk liturgical form");
    assert_eq!(forms.primary_text(), "ѧ҆зы́къ");
    assert!(forms.primary().rule_trace.steps().iter().all(|step| {
        step.stage != "lexical-positional-realization"
            && step.stage != "accent-paradigm-realization"
    }));
}

#[test]
fn partial_irregular_falls_back_and_defect_is_structured() {
    let dative = GrammarCell::Noun(NounCell {
        case: Case::Dative,
        number: Number::Singular,
        animacy: Animacy::Animate,
    });
    let defective = GrammarCell::Noun(NounCell {
        case: Case::Locative,
        number: Number::Dual,
        animacy: Animacy::Animate,
    });
    let spec = NounSpec::new(
        "сынъ",
        "сын",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source(),
    )
    .expect("noun")
    .with_irregular_form(
        SpecifiedForm::new(
            dative,
            "сынови",
            None::<String>,
            SpecificationSource::new(
                "alypy-37-synovi",
                "alypy-gamanovich-grammar-web-2023",
                "Alypy §37",
            )
            .expect("source"),
        )
        .expect("override"),
    )
    .expect("irregular")
    .with_defective_cell(DefectiveCell::evidence_incomplete(
        defective,
        MetadataField::IrregularOverride,
        "no reviewed dual locative override is supplied by this specification",
    ))
    .expect("defect");
    assert_eq!(
        spec.form(match dative {
            GrammarCell::Noun(cell) => cell,
            _ => unreachable!(),
        })
        .expect("override")
        .primary_text(),
        "сынови"
    );
    let fallback = spec
        .form(NounCell {
            case: Case::Genitive,
            number: Number::Dual,
            animacy: Animacy::Animate,
        })
        .expect("licensed regular background");
    assert_eq!(fallback.primary_text(), "сынꙋ");
    let paradigm = spec.paradigm(Animacy::Animate);
    let row = paradigm
        .iter()
        .find(|row| row.cell() == defective)
        .expect("defective row retained");
    assert_eq!(row.status(), crate::ParadigmStatus::EvidenceIncomplete);
}

#[test]
fn caller_irregular_variants_preserve_declared_order_per_cell() {
    let cell = GrammarCell::Noun(NounCell {
        case: Case::Genitive,
        number: Number::Singular,
        animacy: Animacy::Inanimate,
    });
    let first = SpecifiedForm::new(cell, "любве", Some("любве́"), source()).expect("first variant");
    let second =
        SpecifiedForm::new(cell, "любви", Some("любвѝ"), source()).expect("second variant");
    let spec = NounSpec::new(
        "любовь",
        "любв",
        Gender::Feminine,
        NounDeclension::FourthFeminineOvSyncopating,
        source(),
    )
    .expect("noun")
    .with_irregular_form(first.clone())
    .expect("first override")
    .with_irregular_form(second)
    .expect("ordered override");

    assert_eq!(
        spec.form(match cell {
            GrammarCell::Noun(cell) => cell,
            _ => unreachable!(),
        })
        .expect("ordered caller variants")
        .texts()
        .collect::<Vec<_>>(),
        ["любве", "любви"]
    );
    assert!(matches!(
        spec.clone().with_irregular_form(first),
        Err(Error::ContradictoryMetadata { .. })
    ));

    let liturgical = spec
        .form_with(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
            match cell {
                GrammarCell::Noun(cell) => cell,
                _ => unreachable!(),
            },
        )
        .expect("ordered accented caller variants");
    assert_eq!(liturgical.texts().collect::<Vec<_>>(), ["любве́", "любвѝ"]);
}

#[test]
fn paradigm_distinguishes_missing_metadata_from_invalid_cells() {
    let verb = VerbSpec::builder(
        "нести",
        Aspect::Perfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .build()
    .expect("verb");
    let finite = verb.finite_paradigm(FiniteTense::Aorist);
    assert!(finite.iter().all(|row| {
        matches!(
            row.status(),
            crate::ParadigmStatus::MissingMetadata | crate::ParadigmStatus::HistoricallyInvalid
        )
    }));
}

#[test]
fn specialized_paradigm_inventories_are_deterministic_and_duplicate_free() {
    fn assert_inventory(paradigm: &Paradigm, expected: usize) {
        let cells = paradigm.iter().map(|row| row.cell()).collect::<Vec<_>>();
        assert_eq!(cells.len(), expected);
        assert_eq!(
            cells.iter().copied().collect::<BTreeSet<_>>().len(),
            expected
        );
        let repeated = paradigm.iter().map(|row| row.cell()).collect::<Vec<_>>();
        assert_eq!(cells, repeated);
    }

    let noun = NounSpec::new(
        "псалтирникъ",
        "псалтирник",
        Gender::Masculine,
        NounDeclension::FirstHardMasculine,
        source(),
    )
    .expect("noun");
    assert_inventory(&noun.paradigm(Animacy::Animate), 21);

    let adjective =
        AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source()).expect("adjective");
    assert_inventory(
        &adjective.paradigm(AdjectiveForm::Long, Comparison::Positive),
        72,
    );

    let verb = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("verb")
    .build()
    .expect("verb spec");
    assert_inventory(&verb.finite_paradigm(FiniteTense::Present), 9);
    assert_inventory(
        &verb.participle_paradigm(
            ParticipleTense::Present,
            ParticipleVoice::Active,
            AdjectiveForm::Long,
        ),
        72,
    );
}

#[test]
fn typed_present_parts_and_unified_verb_system_paradigms_are_complete() {
    let incomplete = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .build()
    .expect("incomplete verb remains inspectable");
    assert_eq!(
        incomplete.missing_principal_parts(VerbSystem::Finite(FiniteTense::Present)),
        vec![
            MetadataField::PresentStem,
            MetadataField::PresentFirstSingular,
            MetadataField::PresentThirdPlural,
        ]
    );
    let missing_present = incomplete.system_paradigm(VerbSystem::Finite(FiniteTense::Present));
    assert_eq!(missing_present.iter().count(), 9);
    assert_eq!(
        missing_present
            .iter()
            .next()
            .expect("present row")
            .error_code(),
        Some(synodal_church_slavonic_core::ErrorCode::MissingPrincipalPart)
    );

    let verb = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .present_series("нес", "несꙋ", "несꙋтъ")
    .expect("present parts")
    .imperative("нес", ImperativeFormation::ISeries)
    .expect("imperative parts")
    .l_participle_stem("нес")
    .expect("l-participle part")
    .build()
    .expect("verb");
    assert!(
        verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Present))
            .is_empty()
    );
    assert_eq!(
        verb.system_paradigm(VerbSystem::Infinitive)
            .successes()
            .count(),
        1
    );
    assert_eq!(
        verb.system_paradigm(VerbSystem::LParticiple)
            .successes()
            .count(),
        9
    );
    let imperative = verb.system_paradigm(VerbSystem::Imperative);
    assert_eq!(imperative.iter().count(), 9);
    assert_eq!(
        imperative
            .with_status(crate::ParadigmStatus::HistoricallyInvalid)
            .count(),
        3
    );
    assert_eq!(verb.all_system_paradigms().len(), VerbSystem::ALL.len());

    let absent = GrammarCell::Imperative(synodal_church_slavonic_core::ImperativeCell {
        person: Person::Second,
        number: Number::Singular,
    });
    let defective = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .defective_cell(DefectiveCell::historically_absent(
        absent,
        "this caller-reviewed lexeme lacks an imperative",
    ))
    .build()
    .expect("defective verb");
    let paradigm = defective.system_paradigm(VerbSystem::Imperative);
    assert_eq!(
        paradigm
            .iter()
            .find(|row| row.cell() == absent)
            .expect("defective cell retained")
            .status(),
        crate::ParadigmStatus::HistoricallyInvalid
    );
}

#[test]
fn perfective_spec_exposes_the_complete_productive_simple_future() {
    let verb = VerbSpec::builder(
        "понести",
        Aspect::Perfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("builder")
    .present_series("понес", "понесꙋ", "понесꙋтъ")
    .expect("complete present-shaped principal parts")
    .build()
    .expect("perfective verb");

    assert!(
        verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Future))
            .is_empty()
    );
    let future = verb.finite_paradigm(FiniteTense::Future);
    assert_eq!(future.iter().count(), 9);
    assert_eq!(future.successes().count(), 9);
    assert_eq!(future.failures().count(), 0);
    assert_eq!(
        verb.form(GrammarCell::FiniteVerb(
            synodal_church_slavonic_core::FiniteVerbCell {
                tense: FiniteTense::Future,
                person: Person::Third,
                number: Number::Singular,
            }
        ))
        .expect("productive simple future")
        .primary_text(),
        "понесетъ"
    );

    let suppletive = VerbSpec::builder(
        "възѧти",
        Aspect::Perfective,
        VerbConjugation::FirstPalatalized,
        source(),
    )
    .expect("builder")
    .present_series("вземл", "вземлю", "вземлютъ")
    .expect("present series")
    .future_series("возм", "возмꙋ", "возмꙋтъ")
    .expect("future series")
    .build()
    .expect("suppletive future verb");
    assert_eq!(
        suppletive
            .form(GrammarCell::FiniteVerb(
                synodal_church_slavonic_core::FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::Second,
                    number: Number::Singular,
                }
            ))
            .expect("present form")
            .primary_text(),
        "вземлеши"
    );
    assert_eq!(
        suppletive
            .form(GrammarCell::FiniteVerb(
                synodal_church_slavonic_core::FiniteVerbCell {
                    tense: FiniteTense::Future,
                    person: Person::Second,
                    number: Number::Singular,
                }
            ))
            .expect("future form")
            .primary_text(),
        "возмеши"
    );

    let partial = VerbSpec::builder(
        "възѧти",
        Aspect::Perfective,
        VerbConjugation::FirstPalatalized,
        source(),
    )
    .expect("builder")
    .future_stem("возм")
    .expect("future stem")
    .build();
    assert!(matches!(partial, Err(Error::ContradictoryMetadata { .. })));
}

#[test]
fn absent_supine_and_productive_verbal_noun_are_distinguished() {
    let verb = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("verb")
    .verbal_noun_ie("молен")
    .expect("reviewed Alypy §27 platform")
    .build()
    .expect("verb spec");
    assert!(matches!(
        verb.form(GrammarCell::Supine),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
    assert_eq!(
        verb.form(GrammarCell::VerbalNoun(NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        }))
        .expect("productive verbal noun")
        .primary_text(),
        "моленїе"
    );

    let compatibility = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("verb")
    .irregular_form(
        SpecifiedForm::new(GrammarCell::Supine, "нестъ", None::<String>, source())
            .expect("explicit compatibility cell"),
    )
    .build()
    .expect("verb with explicit compatibility cell");
    assert_eq!(
        compatibility
            .form(GrammarCell::Supine)
            .expect("caller exact compatibility form")
            .primary_text(),
        "нестъ"
    );
}

#[test]
fn explicit_and_registry_routes_share_new_productive_rules() {
    let adjective = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
        .expect("adjective")
        .comparison("мꙋдрѣйш", ComparisonFormation::LaterYat)
        .expect("comparison");
    let adjective_cell = AdjectiveCell {
        case: Case::Dative,
        number: Number::Dual,
        gender: Gender::Feminine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Short,
        comparison: Comparison::Comparative,
    };
    let explicit = adjective.form(adjective_cell).expect("explicit");
    let paradigm = adjective.paradigm(AdjectiveForm::Short, Comparison::Comparative);
    assert_eq!(
        paradigm
            .form(GrammarCell::Adjective(adjective_cell))
            .expect("paradigm cell")
            .primary_text(),
        explicit.primary_text()
    );
    let registered = crate::Adjective::resolve("мꙋдръ")
        .expect("registered")
        .form(adjective_cell)
        .expect("registered form");
    assert_eq!(explicit.primary_text(), registered.primary_text());
    assert_eq!(
        explicit.primary().rule_trace.steps()[0].rule,
        registered.primary().rule_trace.steps()[0].rule
    );

    let part = ParticiplePrincipalPart {
        short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
        short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
        long_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
        class: AdjectiveClass::Hard,
    };
    let verb = VerbSpec::builder(
        "нести",
        Aspect::Imperfective,
        VerbConjugation::FirstUnpalatalized,
        source(),
    )
    .expect("verb")
    .present_active_participle(part)
    .build()
    .expect("verb spec");
    let participle_cell = ParticipleCell {
        tense: ParticipleTense::Present,
        voice: ParticipleVoice::Active,
        agreement: AdjectiveCell {
            case: Case::Instrumental,
            number: Number::Plural,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
        },
    };
    let explicit = verb
        .form(GrammarCell::Participle(participle_cell))
        .expect("explicit participle");
    let registered = crate::Participle::resolve("нести")
        .expect("registered verb")
        .form(participle_cell)
        .expect("registered participle");
    assert_eq!(explicit.primary_text(), registered.primary_text());
    assert_eq!(
        explicit.primary().rule_trace.steps()[0].rule,
        registered.primary().rule_trace.steps()[0].rule
    );
}

#[test]
fn explicit_specs_reject_hostile_unicode_without_panicking() {
    for lemma in ["слово\u{e000}", "slovo", "сло\u{0301}\u{0486}во"] {
        assert!(
            NounSpec::new(
                lemma,
                "слов",
                Gender::Neuter,
                NounDeclension::FirstHardNeuter,
                source(),
            )
            .is_err()
        );
    }
}

#[test]
fn typed_specs_reject_contradictory_formation_metadata() {
    assert!(matches!(
        NounSpec::new(
            "жена",
            "жен",
            Gender::Neuter,
            NounDeclension::SecondHard,
            source(),
        ),
        Err(Error::ContradictoryMetadata { .. })
    ));
    assert!(matches!(
        AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
            .expect("base adjective")
            .comparison("мꙋдрѣйш", ComparisonFormation::LaterAi),
        Err(Error::ContradictoryMetadata { .. })
    ));
    let part = ParticiplePrincipalPart {
        short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
        short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
        long_stem: None,
        class: AdjectiveClass::Hard,
    };
    assert!(matches!(
        VerbSpec::builder(
            "любити",
            Aspect::Imperfective,
            VerbConjugation::Second,
            source(),
        )
        .expect("builder")
        .present_active_participle(part)
        .build(),
        Err(Error::ContradictoryMetadata { .. })
    ));
    assert!(matches!(
        VerbSpec::builder(
            "сотворити",
            Aspect::Perfective,
            VerbConjugation::Second,
            source(),
        )
        .expect("builder")
        .imperfect("сотвор", ImperfectFormation::Ah)
        .expect("metadata")
        .build(),
        Err(Error::ContradictoryMetadata { .. })
    ));
    assert!(matches!(
        VerbSpec::builder(
            "изити",
            Aspect::Perfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .l_participle_masculine_singular_stem("изше")
        .expect("mobile-vowel edge")
        .build(),
        Err(Error::ContradictoryMetadata { reason })
            if reason.contains("general l-participle stem")
    ));
    let empty_reason = DefectiveCell::evidence_incomplete(
        GrammarCell::Noun(NounCell {
            case: Case::Dative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        }),
        MetadataField::IrregularOverride,
        "   ",
    );
    assert!(matches!(
        NounSpec::new(
            "слово",
            "слов",
            Gender::Neuter,
            NounDeclension::FirstHardNeuter,
            source(),
        )
        .expect("base noun")
        .with_defective_cell(empty_reason),
        Err(Error::ContradictoryMetadata { .. })
    ));
}
