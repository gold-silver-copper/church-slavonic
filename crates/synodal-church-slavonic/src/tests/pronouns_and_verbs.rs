use crate::*;

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
    for (id, first_singular, third_plural, imperative_singular, imperative_dual_variant) in cases {
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
    assert!(
        inventory.windows(2).all(|pair| {
            pair[0].source_order < pair[1].source_order && pair[1].source_order != 97
        })
    );
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

    let israel =
        Noun::from_id(&LexemeId::from("synodal:noun:v06-israel")).expect("typed proper-name noun");
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
    let dostoyati =
        Verb::from_id(&LexemeId::from("synodal:verb:dostoyati")).expect("typed defective modal");
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
