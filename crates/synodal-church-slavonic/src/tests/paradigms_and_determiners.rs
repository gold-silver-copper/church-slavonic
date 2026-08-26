use crate::*;

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
        for cell in AdjectiveCell::inventory(&AdjectiveForm::ALL, &[Comparison::Positive], |_| {
            &Animacy::ALL
        }) {
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

    let judahite = Adjective::resolve_with("іꙋдинъ", inflector).expect("reviewed -ин- adjective");
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
