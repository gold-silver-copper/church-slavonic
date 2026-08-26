use crate::*;

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
    let forms = noun("рабъ", Case::Dative, Number::Plural, Animacy::Animate).expect("curated noun");
    assert_eq!(forms.primary_text(), "рабѡмъ");
    assert_eq!(forms.target_recension(), Recension::SynodalRussian);
}

#[test]
fn irregular_byti_is_exact_table_first() {
    let forms = present("быти", Person::First, Number::Singular).expect("exact irregular present");
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
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom,
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
        let noun = Noun::resolve_with(lemma, inflector).expect("reviewed productive soft -їе noun");
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
