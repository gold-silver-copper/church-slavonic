use old_church_slavonic::advanced::cells::{
    AdjectiveCell, AdjectiveForm, ClosedClassCell, FiniteVerbCell, ImperativeCell, LParticipleCell,
    NounCell, ParticipleCell,
};
use old_church_slavonic::advanced::metadata as api_metadata;
use old_church_slavonic::advanced::raw_features;
use old_church_slavonic::advanced::rules::{
    AdjectiveClass, AdjectiveLexeme, AoristFormation, ComparativeFormation, ComparativeLexeme,
    ImperativeFormation, NounClass, NounLexeme, NumberRestriction, PronominalDeclension,
    PronominalLexeme, VerbClass, VerbLexeme, adjective_with, comparative_paradigm_with,
    comparative_with, finite_verb_with, imperative_with, noun_with, participle_with,
    productive_new_comparative, pronominal_with,
};
use old_church_slavonic::advanced::{by_id, participle_form};
use old_church_slavonic::trace::{MetadataField, MetadataProvenance, RuleId};
use old_church_slavonic::{
    Adjective, AnaphoricEnvironment, Animacy, CardinalCompositionOptions,
    CardinalMagnitudeIdentity, CardinalNumeralIdentity, Case, CollectiveNumeralCell,
    CollectiveNumeralDeclension, CollectiveNumeralIdentity, CompoundCardinalCell, Determiner,
    DeterminerCell, DeterminerIdentity, DistributiveCardinalCell, FiniteTense, FormSource,
    FractionalNumeralDeclension, FractionalNumeralIdentity, Gender, GenderedCell,
    ImpersonalVerbIdentity, ImpersonalVerbStatus, IndefiniteNumeralIdentity, InflectionError,
    InflectionWarning, InterrogativePronounIdentity, IrregularAgreeingIdentity, Lemma,
    LongOnlyAdjectiveIdentity, MAX_COMPOUND_ORDINAL_VALUE, MIN_COMPOUND_ORDINAL_VALUE, Noun,
    Number, Numeral, NumeralCell, OrdinalComposition, OrdinalNumeralIdentity, ParadigmLookupError,
    PartOfSpeech, ParticipleKind, Person, PersonalPronounCell, PersonalPronounIdentity, Pronoun,
    PronounFormSelection, RequestedCell, Script, StandardPronominalIdentity, UngenderedCell,
    VariantPolicy, Verb, adjective_paradigm, anaphoric_pronoun, aorist, cardinal_magnitude,
    cardinal_numeral_identity, cardinal_numeral_paradigm, collective_numeral,
    collective_numeral_identity, collective_numeral_paradigm, collective_numeral_paradigm_identity,
    compound_cardinal, compound_cardinal_paradigm, compound_cardinal_paradigm_with_options,
    compound_cardinal_with_one, compound_cardinal_with_options, compound_ordinal,
    compound_ordinal_paradigm, determiner, determiner_identity, determiner_paradigm,
    distributive_cardinal, distributive_cardinal_paradigm, distributive_cardinal_paradigm_with_one,
    distributive_cardinal_paradigm_with_options, distributive_cardinal_with_one,
    distributive_cardinal_with_options, finite, finite_paradigm, fractional_numeral,
    fractional_numeral_identity, fractional_numeral_paradigm, fractional_numeral_paradigm_identity,
    gendered_numeral, gendered_pronoun, imperative, imperative_paradigm, imperfect,
    indefinite_numeral, indefinite_numeral_identity, indefinite_numeral_paradigm,
    indefinite_numeral_paradigm_identity, infinitive, interrogative_pronoun, irregular_agreeing,
    l_participle, l_participle_paradigm, long_adjective, long_only_adjective, noun, noun_paradigm,
    numeral, ordinal_numeral, ordinal_numeral_identity, ordinal_numeral_paradigm,
    ordinal_numeral_paradigm_identity, participle_paradigm, past_active_participle,
    personal_pronoun, personal_pronoun_with, present, present_paradigm, pronoun, reflexive_pronoun,
    regular_pronominal, relative_pronoun, short_adjective, supine,
};

fn only_id(lemma: &str, part_of_speech: PartOfSpeech) -> String {
    let candidates = old_church_slavonic::lookup(lemma, part_of_speech).expect("valid lookup");
    assert_eq!(candidates.len(), 1, "fixture must remain unambiguous");
    candidates[0].id.clone()
}

#[test]
fn curated_revision_pinned_goldens_match_the_raw_dictionary_api() {
    let fixture = include_str!("fixtures/goldens.tsv");
    for (line_index, line) in fixture.lines().enumerate().skip(1) {
        let columns = line.split('\t').collect::<Vec<_>>();
        assert_eq!(columns.len(), 5, "golden row {}", line_index + 1);
        let part_of_speech = match columns[1] {
            "noun" => PartOfSpeech::Noun,
            "adj" => PartOfSpeech::Adjective,
            "verb" => PartOfSpeech::Verb,
            "pron" => PartOfSpeech::Pronoun,
            "num" => PartOfSpeech::Numeral,
            other => panic!("unknown golden POS {other}"),
        };
        let expected = columns[3].split(" || ").collect::<Vec<_>>();
        let matching = old_church_slavonic::lookup(columns[0], part_of_speech)
            .expect("golden lookup is valid")
            .into_iter()
            .filter_map(|candidate| {
                raw_features::dictionary_form_by_id(&candidate.id, columns[2])
                    .ok()
                    .filter(|forms| forms.texts().eq(expected.iter().copied()))
                    .map(|_| candidate.id)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "golden row {} must identify one source record: {matching:?}",
            line_index + 1
        );
        let (source, revision) = columns[4]
            .split_once('@')
            .expect("golden source has an immutable revision anchor");
        assert!(!source.is_empty() && !revision.is_empty());
    }
}

#[test]
fn ordinary_calls_take_direct_grammar_and_keep_structured_results() {
    assert_eq!(
        noun("обѣдъ", Case::Dative, Number::Dual)
            .expect("dictionary noun")
            .primary_text(),
        "обѣдома"
    );
    assert_eq!(
        long_adjective(
            "добръ",
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("long adjective")
        .primary_text(),
        "добрꙑи"
    );
    assert_eq!(
        short_adjective(
            "добръ",
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("short adjective")
        .primary_text(),
        "добръ"
    );
    assert_eq!(
        present("благословити", Person::First, Number::Singular)
            .expect("present")
            .primary_text(),
        "благословлѭ"
    );
    assert_eq!(
        imperfect("бꙑти", Person::First, Number::Singular)
            .expect("reviewed imperfect")
            .primary_text(),
        "бѣахъ"
    );
    assert_eq!(
        imperative("благословити", Person::Second, Number::Singular)
            .expect("imperative")
            .primary_text(),
        "благослови"
    );
    assert_eq!(
        l_participle("благословити", Gender::Feminine, Number::Dual)
            .expect("l-participle")
            .primary_text(),
        "благословилѣ"
    );
    assert_eq!(
        infinitive("благословити")
            .expect("infinitive")
            .primary_text(),
        "благословити"
    );
    assert_eq!(
        supine("бости").expect("root supine").primary_text(),
        "бостъ"
    );
}

#[test]
fn long_only_adjectives_have_complete_reviewed_long_paradigms() {
    let selected = [
        (
            LongOnlyAdjectiveIdentity::InterrogativeKotoryi,
            "которꙑи",
            "котороую",
        ),
        (LongOnlyAdjectiveIdentity::OtherProkyi, "прокꙑи", "прокоую"),
        (LongOnlyAdjectiveIdentity::OtherProchii, "прочии", "прочоую"),
    ];
    for (identity, lemma, expected_dual_genitive) in selected {
        let mut cells = 0;
        for cell in AdjectiveCell::all().filter(|cell| cell.form == AdjectiveForm::Long) {
            let forms =
                long_only_adjective(identity, cell.case, cell.number, cell.gender, cell.animacy)
                    .expect("every long cell is represented");
            assert_eq!(forms.lemma(), lemma);
            assert!(matches!(
                forms.source(),
                FormSource::ReviewedGrammarTable { .. }
            ));
            assert_eq!(
                forms.analyses()[0].evidence[0].authority.as_deref(),
                Some("Polivanova 2023 §§285 and 303–305")
            );
            cells += 1;
        }
        assert_eq!(cells, 126);
        assert_eq!(
            long_adjective(
                lemma,
                Case::Genitive,
                Number::Dual,
                Gender::Masculine,
                Animacy::Inanimate,
            )
            .expect("ordinary long-only routing")
            .primary_text(),
            expected_dual_genitive
        );
        assert!(matches!(
            short_adjective(
                lemma,
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
            ),
            Err(InflectionError::HistoricallyInvalidCell { .. })
        ));
    }

    let source_spelling = long_adjective(
        "которыи",
        Case::Nominative,
        Number::Plural,
        Gender::Masculine,
        Animacy::Inanimate,
    )
    .expect("source spelling alias");
    assert_eq!(source_spelling.lemma(), "которꙑи");
    assert_eq!(source_spelling.primary_text(), "котории");
    assert!(
        source_spelling
            .warnings()
            .contains(&InflectionWarning::LexicalAliasUsed {
                canonical: "которꙑи".to_string(),
            })
    );

    let explicit = AdjectiveLexeme {
        lemma: "прочии".to_string(),
        class: AdjectiveClass::Soft,
    };
    let long_cell = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Neuter,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Long,
    };
    assert_eq!(
        adjective_with(&explicit, long_cell)
            .expect("explicit long citation")
            .primary_text(),
        "прочеѥ"
    );
    assert!(matches!(
        adjective_with(
            &explicit,
            AdjectiveCell {
                form: AdjectiveForm::Short,
                ..long_cell
            },
        ),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn determiner_inventory_is_exhaustive_across_all_real_declensional_profiles() {
    let nominative_plural = [
        (DeterminerIdentity::RelativeMannerYak, "ꙗци"),
        (DeterminerIdentity::RelativeQuantityYelik, "ѥлици"),
        (DeterminerIdentity::InterrogativeMannerKak, "каци"),
        (DeterminerIdentity::InterrogativeQuantityKolik, "колици"),
        (DeterminerIdentity::DemonstrativeQuantitySelik, "селици"),
        (DeterminerIdentity::DemonstrativeMannerTak, "таци"),
        (DeterminerIdentity::DemonstrativeQuantityTolik, "толици"),
        (DeterminerIdentity::InterrogativePossessiveChii, "чии"),
        (DeterminerIdentity::InterrogativeKyi, "ции"),
        (DeterminerIdentity::InterrogativeKotoryi, "котории"),
        (DeterminerIdentity::IndefiniteYeter, "ѥтери"),
    ];
    assert_eq!(DeterminerIdentity::ALL.len(), nominative_plural.len());

    for (identity, expected) in nominative_plural {
        let paradigm = determiner_paradigm(identity.canonical_lemma())
            .expect("every reviewed determiner has a paradigm");
        assert_eq!(paradigm.identity(), identity);
        assert_eq!(paradigm.lemma(), identity.canonical_lemma());
        assert_eq!(paradigm.len(), 126);

        let adjectival = matches!(
            identity,
            DeterminerIdentity::InterrogativeKotoryi | DeterminerIdentity::IndefiniteYeter
        );
        assert_eq!(
            paradigm.successes().count(),
            if adjectival { 126 } else { 108 }
        );
        assert_eq!(paradigm.failures().count(), if adjectival { 0 } else { 18 });
        assert!(
            paradigm.successes().all(|(_, forms)| matches!(
                forms.source(),
                FormSource::ReviewedGrammarTable { .. }
            ))
        );
        assert!(paradigm.failures().all(|(cell, error)| {
            cell.case == Case::Vocative
                && matches!(
                    error,
                    InflectionError::HistoricallyInvalidCell {
                        cell: RequestedCell::Determiner(requested),
                        ..
                    } if requested == cell
                )
        }));

        let direct = determiner_identity(
            identity,
            Case::Nominative,
            Number::Plural,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("reviewed identity route");
        assert_eq!(direct.primary_text(), expected);
        assert_eq!(
            paradigm
                .form(
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                    Animacy::Inanimate,
                )
                .expect("paradigm route"),
            &direct
        );
    }

    for (alias, canonical) in [("етеръ", "ѥтеръ"), ("которыи", "которꙑи")]
    {
        let forms = determiner(
            alias,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("source-union determiner alias");
        assert_eq!(forms.lemma(), canonical);
        assert!(
            forms
                .warnings()
                .contains(&InflectionWarning::LexicalAliasUsed {
                    canonical: canonical.to_string(),
                })
        );
    }
}

#[test]
fn simple_cardinals_have_exhaustive_typed_paradigms_and_evidence() {
    let expected_successes = [18, 18, 18, 18, 18, 18, 7, 7, 7, 7, 7, 21];
    assert_eq!(CardinalNumeralIdentity::ALL.len(), expected_successes.len());

    for (identity, expected_successes) in CardinalNumeralIdentity::ALL
        .into_iter()
        .zip(expected_successes)
    {
        let paradigm = cardinal_numeral_paradigm(identity);
        assert_eq!(paradigm.identity(), identity);
        assert_eq!(paradigm.lemma(), identity.canonical_lemma());
        assert_eq!(paradigm.len(), 84);
        assert_eq!(
            paradigm.successes().count(),
            expected_successes,
            "{identity:?}"
        );
        assert_eq!(
            paradigm.failures().count(),
            paradigm.len() - expected_successes,
            "{identity:?}"
        );
        assert!(
            paradigm.successes().all(|(_, forms)| matches!(
                forms.source(),
                FormSource::ReviewedGrammarTable { .. }
            ))
        );
        assert!(paradigm.failures().all(|(cell, error)| matches!(
            error,
            InflectionError::HistoricallyInvalidCell {
                cell: RequestedCell::Numeral(requested),
                ..
            } if requested == cell
        )));
    }

    let three = cardinal_numeral_identity(
        CardinalNumeralIdentity::Three,
        Case::Instrumental,
        Number::Plural,
        Some(Gender::Neuter),
    )
    .expect("reviewed cardinal-three cell");
    assert_eq!(three.primary_text(), "трьми");
    assert_eq!(three.trace()[0].rule_id, RuleId::NumeralCardinalThree);

    let ten = numeral("десѧть", Case::Nominative, Number::Plural)
        .expect("reviewed mixed cardinal-ten cell");
    assert_eq!(ten.texts().collect::<Vec<_>>(), ["десѧте", "десѧти"]);
    assert_eq!(ten.analyses().len(), 2);
    assert_eq!(
        ten.analyses()[0].evidence[0].provenance,
        MetadataProvenance::ReviewedGrammarTable
    );
    assert_eq!(
        ten.analyses()[1].evidence[0].provenance,
        MetadataProvenance::ProductiveRuleOutput
    );
    assert_eq!(
        ten.analyses()[0].evidence[0].source_form.as_deref(),
        Some("десѧте")
    );
    assert_eq!(ten.analyses()[1].evidence[0].source_form, None);

    let alias = gendered_numeral("единъ", Case::Genitive, Number::Singular, Gender::Feminine)
        .expect("source-union cardinal alias");
    assert_eq!(alias.lemma(), "ѥдинъ");
    assert_eq!(alias.primary_text(), "ѥдиноѩ");
    assert!(
        alias
            .warnings()
            .contains(&InflectionWarning::LexicalAliasUsed {
                canonical: "ѥдинъ".to_string(),
            })
    );

    let invalid = NumeralCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Some(Gender::Masculine),
    };
    assert!(matches!(
        cardinal_numeral_identity(
            CardinalNumeralIdentity::Three,
            invalid.case,
            invalid.number,
            invalid.gender,
        ),
        Err(InflectionError::HistoricallyInvalidCell {
            cell: RequestedCell::Numeral(requested),
            ..
        }) if requested == invalid
    ));
}

#[test]
fn simple_ordinals_have_complete_adjective_paradigms_and_evidence() {
    for identity in OrdinalNumeralIdentity::ALL {
        let paradigm = ordinal_numeral_paradigm_identity(identity);
        assert_eq!(paradigm.identity(), identity);
        assert_eq!(paradigm.lemma(), identity.canonical_lemma());
        assert_eq!(paradigm.len(), 252);
        assert_eq!(paradigm.successes().count(), 252, "{identity:?}");
        assert_eq!(paradigm.failures().count(), 0, "{identity:?}");
        assert!(paradigm.successes().all(|(_, forms)| matches!(
            forms.source(),
            FormSource::ReviewedGrammarTable { rule_id } if *rule_id == identity.rule_id()
        )));
    }

    let citation = ordinal_numeral_identity(
        OrdinalNumeralIdentity::Third,
        AdjectiveForm::Short,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )
    .expect("reviewed third-ordinal citation");
    assert_eq!(citation.primary_text(), "третии");
    assert_eq!(
        citation.trace().last().map(|step| step.rule_id),
        Some(RuleId::NumeralOrdinalJ)
    );
    assert_eq!(
        citation.analyses()[0].evidence[0].provenance,
        MetadataProvenance::ReviewedGrammarTable
    );
    assert_eq!(
        citation.analyses()[0].evidence[0].source_form.as_deref(),
        Some("третии")
    );

    let productive = ordinal_numeral_identity(
        OrdinalNumeralIdentity::Third,
        AdjectiveForm::Long,
        Case::Genitive,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )
    .expect("productive third-ordinal long cell");
    assert_eq!(
        productive.texts().collect::<Vec<_>>(),
        ["третиꙗѥго", "третиѣаго"]
    );
    assert_eq!(
        productive.analyses()[0].evidence[0].provenance,
        MetadataProvenance::ProductiveRuleOutput
    );
    assert_eq!(productive.analyses()[0].evidence[0].source_form, None);
    assert_eq!(
        productive.analyses()[1].evidence[0].provenance,
        MetadataProvenance::CorpusEvaluationObservation
    );
    assert_eq!(
        productive.analyses()[1].evidence[0].source_form.as_deref(),
        Some("третиѣаго")
    );

    let alias = ordinal_numeral(
        "трети",
        AdjectiveForm::Short,
        Case::Nominative,
        Number::Singular,
        Gender::Neuter,
        Animacy::Inanimate,
    )
    .expect("dictionary and corpus spelling alias");
    assert_eq!(alias.lemma(), "третии");
    assert_eq!(alias.primary_text(), "третиѥ");
    assert!(
        alias
            .warnings()
            .contains(&InflectionWarning::LexicalAliasUsed {
                canonical: "третии".to_string(),
            })
    );
    assert_eq!(
        ordinal_numeral_paradigm("трети")
            .expect("source-union ordinal paradigm")
            .identity(),
        OrdinalNumeralIdentity::Third
    );
}

#[test]
fn collective_numerals_preserve_their_two_real_inflectional_classes() {
    for identity in CollectiveNumeralIdentity::ALL {
        let paradigm = collective_numeral_paradigm_identity(identity);
        assert_eq!(paradigm.identity(), identity);
        assert_eq!(paradigm.lemma(), identity.canonical_lemma());
        match identity.declension() {
            CollectiveNumeralDeclension::Pronominal => {
                assert_eq!(paradigm.len(), 63, "{identity:?}");
                assert_eq!(paradigm.successes().count(), 54, "{identity:?}");
                assert_eq!(paradigm.failures().count(), 9, "{identity:?}");
            }
            CollectiveNumeralDeclension::Adjectival => {
                assert_eq!(paradigm.len(), 252, "{identity:?}");
                assert_eq!(paradigm.successes().count(), 252, "{identity:?}");
                assert_eq!(paradigm.failures().count(), 0, "{identity:?}");
            }
        }
        assert!(paradigm.successes().all(|(_, forms)| matches!(
            forms.source(),
            FormSource::ReviewedGrammarTable { rule_id } if *rule_id == identity.rule_id()
        )));
    }

    let low = collective_numeral_identity(
        CollectiveNumeralIdentity::Two,
        CollectiveNumeralCell::pronominal(Case::Accusative, Number::Singular, Gender::Neuter),
    )
    .expect("licensed collective-pronominal cell");
    assert_eq!(low.primary_text(), "дъвоѥ");
    assert_eq!(
        low.trace().last().map(|step| step.rule_id),
        Some(RuleId::NumeralCollectivePronominal)
    );
    assert_eq!(
        low.analyses()[0].evidence[0].provenance,
        MetadataProvenance::ProductiveRuleOutput
    );

    let direct = collective_numeral_identity(
        CollectiveNumeralIdentity::Four,
        CollectiveNumeralCell::adjectival(
            AdjectiveForm::Short,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        ),
    )
    .expect("directly cited collective adjective");
    assert_eq!(direct.texts().collect::<Vec<_>>(), ["четворъ", "четвѣръ"]);
    assert!(
        !direct
            .warnings()
            .contains(&InflectionWarning::IncludesReconstructedForms)
    );
    assert!(direct.analyses().iter().all(|analysis| {
        analysis.evidence[0].provenance == MetadataProvenance::ReviewedGrammarTable
            && analysis.evidence[0].source_form.is_some()
    }));

    let reconstructed = collective_numeral_identity(
        CollectiveNumeralIdentity::Five,
        CollectiveNumeralCell::adjectival(
            AdjectiveForm::Short,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        ),
    )
    .expect("historically reconstructed collective adjective");
    assert_eq!(
        reconstructed.texts().collect::<Vec<_>>(),
        ["пѧтеръ", "пѧторъ"]
    );
    assert!(
        reconstructed
            .warnings()
            .contains(&InflectionWarning::IncludesReconstructedForms)
    );
    assert!(reconstructed.analyses().iter().all(|analysis| {
        analysis.evidence[0].provenance == MetadataProvenance::ProductiveRuleOutput
            && analysis.evidence[0].source_form.is_none()
    }));

    let corpus = collective_numeral_identity(
        CollectiveNumeralIdentity::Ten,
        CollectiveNumeralCell::adjectival(
            AdjectiveForm::Short,
            Case::Accusative,
            Number::Singular,
            Gender::Neuter,
            Animacy::Inanimate,
        ),
    )
    .expect("collective corpus cell");
    assert_eq!(
        corpus.texts().collect::<Vec<_>>(),
        ["десѧторо", "десѧтеро", "десꙙторо"]
    );
    assert_eq!(
        corpus.analyses()[2].evidence[0].provenance,
        MetadataProvenance::CorpusEvaluationObservation
    );
    assert_eq!(
        corpus.analyses()[2].evidence[0].source_form.as_deref(),
        Some("десꙙторо")
    );

    let alias = collective_numeral(
        "четвѣръ",
        CollectiveNumeralCell::adjectival(
            AdjectiveForm::Short,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        ),
    )
    .expect("collective source-union alias");
    assert_eq!(alias.lemma(), "четворъ");
    assert!(
        alias
            .warnings()
            .contains(&InflectionWarning::LexicalAliasUsed {
                canonical: "четворъ".to_string(),
            })
    );
    assert_eq!(
        collective_numeral_paradigm("седмеръ")
            .expect("collective reconstructed alias")
            .identity(),
        CollectiveNumeralIdentity::Seven
    );
}

#[test]
fn fractional_numerals_are_complete_noun_paradigms_with_period_boundaries() {
    assert_eq!(
        FractionalNumeralIdentity::HalfPol.declension(),
        FractionalNumeralDeclension::UStem
    );
    assert_eq!(
        FractionalNumeralIdentity::Quarter.declension(),
        FractionalNumeralDeclension::IStem
    );
    assert_eq!(
        FractionalNumeralIdentity::Tenth.declension(),
        FractionalNumeralDeclension::AStem
    );
    for identity in FractionalNumeralIdentity::ALL {
        let paradigm = fractional_numeral_paradigm_identity(identity);
        assert_eq!(paradigm.identity(), identity);
        assert_eq!(paradigm.lemma(), identity.canonical_lemma());
        assert_eq!(paradigm.len(), 21, "{identity:?}");
        assert_eq!(paradigm.successes().count(), 21, "{identity:?}");
        assert_eq!(paradigm.failures().count(), 0, "{identity:?}");
        assert!(paradigm.successes().all(|(_, forms)| matches!(
            forms.source(),
            FormSource::ReviewedGrammarTable { rule_id }
                if *rule_id == RuleId::NumeralFractionalNoun
        )));
    }

    for (identity, case, number, expected) in [
        (
            FractionalNumeralIdentity::HalfPol,
            Case::Genitive,
            Number::Singular,
            "полоу",
        ),
        (
            FractionalNumeralIdentity::HalfPolovina,
            Case::Accusative,
            Number::Singular,
            "половинѫ",
        ),
        (
            FractionalNumeralIdentity::Quarter,
            Case::Instrumental,
            Number::Singular,
            "четврьтьѭ",
        ),
        (
            FractionalNumeralIdentity::Tenth,
            Case::Genitive,
            Number::Plural,
            "десѧтинъ",
        ),
    ] {
        let forms = fractional_numeral_identity(identity, case, number)
            .unwrap_or_else(|error| panic!("{identity:?}: {error}"));
        assert_eq!(forms.primary_text(), expected);
        assert_eq!(
            forms.trace().last().map(|step| step.rule_id),
            Some(RuleId::NumeralFractionalNoun)
        );
    }

    for (identity, lemma) in [
        (FractionalNumeralIdentity::HalfPol, "полъ"),
        (FractionalNumeralIdentity::Tenth, "десѧтина"),
    ] {
        for number in Number::ALL {
            for case in Case::ALL {
                let fractional = fractional_numeral_identity(identity, case, number)
                    .expect("licensed fractional noun cell");
                let dictionary = noun(lemma, case, number).expect("dictionary noun cell");
                assert_eq!(
                    fractional.primary_text(),
                    dictionary.primary_text(),
                    "{identity:?} {case:?} {number:?}"
                );
            }
        }
    }

    let half_attestation = fractional_numeral("полъ", Case::Accusative, Number::Singular)
        .expect("fractional half corpus cell");
    assert_eq!(half_attestation.texts().collect::<Vec<_>>(), ["полъ"]);
    assert_eq!(half_attestation.analyses().len(), 2);
    assert_eq!(
        half_attestation.analyses()[1].evidence[0].provenance,
        MetadataProvenance::CorpusEvaluationObservation
    );

    let tenth_attestation = fractional_numeral_identity(
        FractionalNumeralIdentity::Tenth,
        Case::Accusative,
        Number::Singular,
    )
    .expect("fractional tenth corpus cell");
    assert_eq!(tenth_attestation.texts().collect::<Vec<_>>(), ["десѧтинѫ"]);
    assert_eq!(tenth_attestation.analyses().len(), 2);
    assert_eq!(
        tenth_attestation.analyses()[1].evidence[0].provenance,
        MetadataProvenance::CorpusEvaluationObservation
    );

    assert_eq!(
        fractional_numeral_paradigm("четврьть")
            .expect("source-listed quarter")
            .identity(),
        FractionalNumeralIdentity::Quarter
    );
    for later in ["третина", "полътора", "полътретиꙗ"] {
        assert!(matches!(
            fractional_numeral(later, Case::Nominative, Number::Singular),
            Err(InflectionError::UnknownLemma { .. })
        ));
    }
}

#[test]
fn indefinite_quantity_numeral_is_fully_declined_but_never_an_exact_integer() {
    let identity = IndefiniteNumeralIdentity::Nesveda;
    assert_eq!(identity.noun_class(), NounClass::AHard);
    assert_eq!(identity.gender(), Gender::Feminine);
    assert_eq!(RuleId::NumeralIndefiniteNoun.code(), "NUM-INDEF-NOUN-01");

    let paradigm = indefinite_numeral_paradigm_identity(identity);
    assert_eq!(paradigm.identity(), identity);
    assert_eq!(paradigm.lemma(), "несъвѣда");
    assert_eq!(paradigm.len(), 21);
    assert_eq!(paradigm.successes().count(), 21);
    assert_eq!(paradigm.failures().count(), 0);
    assert_eq!(
        paradigm
            .form(Case::Accusative, Number::Singular)
            .expect("productive hard a-stem cell")
            .primary_text(),
        "несъвѣдѫ"
    );

    let attested = indefinite_numeral("несъвѣда", Case::Instrumental, Number::Plural)
        .expect("Suprasliensis instrumental plural");
    assert_eq!(attested.primary_text(), "несъвѣдами");
    assert_eq!(attested.analyses().len(), 1);
    assert_eq!(
        attested.analyses()[0].evidence[0].provenance,
        MetadataProvenance::PrimaryTextAttestation
    );
    assert_eq!(
        attested.analyses()[0].evidence[0].source_form.as_deref(),
        Some("несъвѣдами")
    );

    let predicted = indefinite_numeral_identity(identity, Case::Dative, Number::Dual)
        .expect("productive hard a-stem prediction");
    assert_eq!(predicted.primary_text(), "несъвѣдама");
    assert_eq!(
        predicted.analyses()[0].evidence[0].provenance,
        MetadataProvenance::ProductiveRuleOutput
    );
    assert!(predicted.analyses()[0].evidence[0].source_form.is_none());

    assert_eq!(
        indefinite_numeral_paradigm("несъвѣда")
            .expect("closed identity lookup")
            .identity(),
        identity
    );
    assert_eq!(
        CardinalMagnitudeIdentity::classify_source_union_lemma("несъвѣда"),
        None
    );
    assert!(matches!(
        indefinite_numeral("тъма", Case::Nominative, Number::Singular),
        Err(InflectionError::UnknownLemma {
            part_of_speech: PartOfSpeech::Numeral,
            ..
        })
    ));
}

#[test]
fn collective_numerals_reject_crossed_cell_classes() {
    let crossed = CollectiveNumeralCell::adjectival(
        AdjectiveForm::Long,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    assert!(matches!(
        collective_numeral_identity(CollectiveNumeralIdentity::Three, crossed),
        Err(InflectionError::HistoricallyInvalidCell {
            cell: RequestedCell::CollectiveNumeral(cell),
            ..
        }) if cell == crossed
    ));

    let low = collective_numeral_paradigm_identity(CollectiveNumeralIdentity::Both);
    assert!(matches!(
        low.adjectival_form(
            AdjectiveForm::Short,
            Case::Nominative,
            Number::Singular,
            Gender::Neuter,
            Animacy::Inanimate,
        ),
        Err(ParadigmLookupError::NotRepresented)
    ));
}

#[test]
fn reviewed_hard_ordinals_match_existing_adjective_tables() {
    for identity in [
        OrdinalNumeralIdentity::Fourth,
        OrdinalNumeralIdentity::Tenth,
    ] {
        for cell in AdjectiveCell::all() {
            // The legacy copied adjective feature rows have neither a distinct
            // vocative nor an animacy dimension. Cross-check the cells that
            // those rows can encode independently.
            if cell.case == Case::Vocative
                || (cell.case == Case::Accusative
                    && cell.gender == Gender::Masculine
                    && cell.animacy == Animacy::Animate
                    && matches!(cell.number, Number::Singular | Number::Plural))
            {
                continue;
            }
            let ordinal = ordinal_numeral_identity(
                identity,
                cell.form,
                cell.case,
                cell.number,
                cell.gender,
                cell.animacy,
            )
            .expect("reviewed ordinal cell");
            let adjective = match cell.form {
                AdjectiveForm::Short => short_adjective(
                    identity.canonical_lemma(),
                    cell.case,
                    cell.number,
                    cell.gender,
                    cell.animacy,
                ),
                AdjectiveForm::Long => long_adjective(
                    identity.canonical_lemma(),
                    cell.case,
                    cell.number,
                    cell.gender,
                    cell.animacy,
                ),
            }
            .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error}"));
            assert_eq!(
                ordinal.primary_text(),
                adjective.primary_text(),
                "{identity:?} {cell:?}"
            );
        }
    }
}

#[test]
fn compound_cardinals_through_ninety_nine_are_structured_and_exhaustive() {
    let goldens = [
        (
            12,
            Case::Genitive,
            Some(Gender::Masculine),
            "дъвою на десѧте",
        ),
        (15, Case::Nominative, None, "пѧть на десѧте"),
        (20, Case::Nominative, None, "дъва десѧти"),
        (20, Case::Genitive, None, "дъвою десѧту"),
        (30, Case::Nominative, None, "триѥ десѧте"),
        (40, Case::Nominative, None, "четыре десѧте"),
        (50, Case::Genitive, None, "пѧти десѧтъ"),
        (
            53,
            Case::Nominative,
            Some(Gender::Masculine),
            "пѧть десѧтъ и триѥ",
        ),
        (
            91,
            Case::Dative,
            Some(Gender::Feminine),
            "девѧти десѧтъ и ѥдинои",
        ),
    ];
    for (value, case, gender, expected) in goldens {
        let realized = compound_cardinal(value, case, gender).expect("licensed compound cardinal");
        assert_eq!(realized.value(), value);
        assert_eq!(realized.cell(), CompoundCardinalCell { case, gender });
        assert_eq!(realized.primary_text(), expected);
        assert!(!realized.analyses().is_empty());
    }

    let thirty =
        compound_cardinal(30, Case::Nominative, None).expect("thirty nominative alternatives");
    assert_eq!(thirty.analyses().len(), 2);
    assert_eq!(thirty.analyses()[0].primary_text(), "триѥ десѧте");
    assert_eq!(thirty.analyses()[1].primary_text(), "три десѧти");

    let alternate_one = compound_cardinal_with_one(
        21,
        CardinalNumeralIdentity::OneYedyn,
        Case::Dative,
        Some(Gender::Feminine),
    )
    .expect("explicit compound-one doublet");
    assert_eq!(alternate_one.primary_text(), "дъвѣма десѧтьма и ѥдьнои");

    let paradigm = compound_cardinal_paradigm(53).expect("compound-cardinal paradigm");
    assert_eq!(paradigm.value(), 53);
    assert_eq!(paradigm.len(), 28);
    assert_eq!(
        paradigm
            .form(Case::Nominative, Some(Gender::Masculine))
            .expect("licensed paradigm row")
            .primary_text(),
        "пѧть десѧтъ и триѥ"
    );

    for value in 11..=99 {
        let final_digit = if value < 20 { value - 10 } else { value % 10 };
        let requires_gender = matches!(final_digit, 1..=4);
        for cell in CompoundCardinalCell::all() {
            let gender_shape_valid = cell.gender.is_some() == requires_gender;
            let vocative_valid = cell.case != Case::Vocative
                || (value < 20 && final_digit >= 5)
                || (value >= 50 && (final_digit == 0 || final_digit >= 5));
            match compound_cardinal(value, cell.case, cell.gender) {
                Ok(realized) => {
                    assert!(gender_shape_valid && vocative_valid, "{value} {cell:?}");
                    assert_eq!(realized.cell(), cell);
                    assert!(realized.analyses().iter().all(|analysis| {
                        !analysis.tokens.is_empty()
                            && analysis
                                .tokens
                                .iter()
                                .all(|token| !token.forms.primary_text().is_empty())
                    }));
                }
                Err(error) => {
                    assert!(
                        !gender_shape_valid || !vocative_valid,
                        "{value} {cell:?}: {error}"
                    );
                    assert!(matches!(
                        error,
                        InflectionError::HistoricallyInvalidCell {
                            cell: RequestedCell::CompoundCardinal {
                                value: requested_value,
                                cell: requested_cell,
                            },
                            ..
                        } if requested_value == value && requested_cell == cell
                    ));
                }
            }
        }
    }
}

#[test]
fn compound_ordinals_through_one_thousand_are_structured_and_exhaustive() {
    let eighteenth = compound_ordinal(
        18,
        AdjectiveForm::Long,
        Case::Accusative,
        Number::Singular,
        Gender::Neuter,
        Animacy::Inanimate,
    )
    .expect("reviewed analytic and fused eighteenth");
    assert_eq!(eighteenth.primary_text(), "осмоѥ на десѧте");
    assert_eq!(eighteenth.analyses().len(), 2);
    assert_eq!(
        eighteenth.analyses()[0].construction,
        OrdinalComposition::AnalyticTeen
    );
    assert_eq!(
        eighteenth.analyses()[1].construction,
        OrdinalComposition::FusedStem
    );
    assert!(
        eighteenth.analyses()[1].tokens[0]
            .forms
            .texts()
            .any(|text| text == "осмонадесѧтоѥ")
    );
    assert_eq!(
        eighteenth.analyses()[1].tokens[0]
            .forms
            .texts()
            .filter(|text| *text == "осмонадесѧтоѥ")
            .count(),
        1,
        "identical productive and corpus surfaces retain separate evidence analyses, not duplicate choices"
    );
    assert_eq!(
        eighteenth.analyses()[1].tokens[0].forms.analyses().len(),
        3,
        "both reviewed fused stems and the exact corpus observation remain inspectable"
    );

    let twenty_eighth = compound_ordinal(
        28,
        AdjectiveForm::Long,
        Case::Accusative,
        Number::Singular,
        Gender::Neuter,
        Animacy::Inanimate,
    )
    .expect("attested conjunctive twenty-eighth");
    assert_eq!(twenty_eighth.primary_text(), "дъвадесѧтьноѥ и осмоѥ");
    assert_eq!(twenty_eighth.analyses().len(), 6);
    assert_eq!(
        twenty_eighth
            .analyses()
            .iter()
            .map(|analysis| analysis.construction)
            .collect::<Vec<_>>(),
        [
            OrdinalComposition::ConjunctionI,
            OrdinalComposition::Asyndetic,
            OrdinalComposition::ConjunctionTi,
            OrdinalComposition::AsyndeticFirstComponent,
            OrdinalComposition::BetweenTens,
            OrdinalComposition::UnitWithinThirdTen,
        ]
    );
    assert_eq!(
        twenty_eighth.analyses()[3].primary_text(),
        "дъвадесѧтьноѥ осмъ"
    );
    assert_eq!(
        twenty_eighth.analyses()[4].primary_text(),
        "осмоѥ междю десетма"
    );
    assert_eq!(
        twenty_eighth.analyses()[5].primary_text(),
        "осмоѥ третиаго десѧте"
    );
    assert_eq!(
        twenty_eighth.analyses()[4].tokens[1].forms.source(),
        &FormSource::ReviewedGrammarTable {
            rule_id: RuleId::NumeralOrdinalCircumlocutive,
        }
    );
    assert_eq!(
        twenty_eighth.analyses()[4].tokens[2].forms.analyses()[0].evidence[0]
            .source_form
            .as_deref(),
        Some("десетма")
    );
    assert_eq!(
        twenty_eighth.analyses()[0]
            .tokens
            .iter()
            .map(|token| token.role)
            .collect::<Vec<_>>(),
        [
            old_church_slavonic::PhraseRole::Numeral,
            old_church_slavonic::PhraseRole::Conjunction,
            old_church_slavonic::PhraseRole::Numeral,
        ]
    );
    assert!(
        twenty_eighth.analyses()[0].tokens[0]
            .forms
            .texts()
            .any(|text| text == "двадесꙙтъноѥ")
    );
    let mut mislabeled = twenty_eighth.analyses()[0].clone();
    mislabeled.construction = OrdinalComposition::Asyndetic;
    assert!(matches!(
        old_church_slavonic::RealizedOrdinal::new(
            twenty_eighth.value(),
            twenty_eighth.cell(),
            vec![mislabeled],
        ),
        Err(InflectionError::InvalidInput { .. })
    ));
    let mut wrong_turn = twenty_eighth.analyses()[4].clone();
    wrong_turn.construction = OrdinalComposition::UnitWithinThirdTen;
    assert!(matches!(
        old_church_slavonic::RealizedOrdinal::new(
            twenty_eighth.value(),
            twenty_eighth.cell(),
            vec![wrong_turn],
        ),
        Err(InflectionError::InvalidInput { .. })
    ));
    assert!(matches!(
        old_church_slavonic::RealizedOrdinal::new(
            30,
            twenty_eighth.cell(),
            vec![twenty_eighth.analyses()[4].clone()],
        ),
        Err(InflectionError::InvalidInput { .. })
    ));

    let hundred_fourth = compound_ordinal(
        104,
        AdjectiveForm::Long,
        Case::Genitive,
        Number::Singular,
        Gender::Neuter,
        Animacy::Inanimate,
    )
    .expect("attested asyndetic hundred-fourth");
    assert_eq!(hundred_fourth.primary_text(), "сътьнаѥго четврьтаѥго");
    assert_eq!(
        hundred_fourth.analyses()[0].construction,
        OrdinalComposition::Asyndetic
    );
    assert!(
        hundred_fourth.analyses()[0].tokens[0]
            .forms
            .texts()
            .any(|text| text == "сътънааго")
    );
    let first_only = hundred_fourth
        .analyses()
        .iter()
        .find(|analysis| analysis.construction == OrdinalComposition::AsyndeticFirstComponent)
        .expect("the competing first-component-only declension account");
    assert!(first_only.construction.is_disputed());
    assert!(!hundred_fourth.analyses()[0].construction.is_disputed());
    assert_eq!(first_only.primary_text(), "сътьнаѥго четврьтъ");

    let hundred_twenty_eighth = compound_ordinal(
        128,
        AdjectiveForm::Long,
        Case::Genitive,
        Number::Singular,
        Gender::Neuter,
        Animacy::Inanimate,
    )
    .expect("nested first-component-only account");
    let nested_first_only = hundred_twenty_eighth
        .analyses()
        .iter()
        .find(|analysis| analysis.construction == OrdinalComposition::AsyndeticFirstComponent)
        .expect("nested disputed asyndetic analysis");
    assert_eq!(
        nested_first_only.primary_text(),
        "сътьнаѥго дъвадесѧтьнъ и осмъ"
    );

    let reconstructed = compound_ordinal(
        700,
        AdjectiveForm::Short,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )
    .expect("productive inherited seven-hundredth");
    assert_eq!(reconstructed.primary_text(), "седмосътьнъ");
    assert!(
        reconstructed.analyses()[0].tokens[0]
            .forms
            .warnings()
            .contains(&InflectionWarning::IncludesReconstructedForms)
    );

    for value in 11..=1_000 {
        let paradigm = compound_ordinal_paradigm(value)
            .unwrap_or_else(|error| panic!("compound ordinal {value}: {error}"));
        assert_eq!(paradigm.value(), value);
        assert_eq!(paradigm.len(), 252, "{value}");
        assert_eq!(paradigm.successes().count(), 252, "{value}");
        assert_eq!(paradigm.failures().count(), 0, "{value}");
        assert!(paradigm.successes().all(|(cell, ordinal)| {
            ordinal.cell() == *cell
                && ordinal.value() == value
                && ordinal.analyses().iter().all(|analysis| {
                    !analysis.tokens.is_empty()
                        && analysis
                            .tokens
                            .iter()
                            .all(|token| !token.forms.primary_text().is_empty())
                })
        }));
        let is_head = ((20..=90).contains(&value) && value % 10 == 0)
            || ((100..=900).contains(&value) && value % 100 == 0)
            || value == 1_000;
        let expects_first_component_only = !(11..=19).contains(&value) && !is_head;
        let expects_rare_turns = (21..=29).contains(&value);
        assert!(paradigm.successes().all(|(_, ordinal)| {
            ordinal.analyses().iter().any(|analysis| {
                analysis.construction == OrdinalComposition::AsyndeticFirstComponent
            }) == expects_first_component_only
                && ordinal
                    .analyses()
                    .iter()
                    .any(|analysis| analysis.construction == OrdinalComposition::BetweenTens)
                    == expects_rare_turns
                && ordinal
                    .analyses()
                    .iter()
                    .any(|analysis| analysis.construction == OrdinalComposition::UnitWithinThirdTen)
                    == expects_rare_turns
        }));
        if expects_rare_turns {
            let unit = OrdinalNumeralIdentity::ALL[usize::from(value - 21)];
            for (cell, ordinal) in paradigm.successes() {
                let expected_unit = ordinal_numeral_identity(
                    unit,
                    cell.form,
                    cell.case,
                    cell.number,
                    cell.gender,
                    cell.animacy,
                )
                .expect("every simple ordinal has the complete adjective product");
                for construction in [
                    OrdinalComposition::BetweenTens,
                    OrdinalComposition::UnitWithinThirdTen,
                ] {
                    let turn = ordinal
                        .analyses()
                        .iter()
                        .find(|analysis| analysis.construction == construction)
                        .expect("the rare turn is licensed for every 21–29 agreement cell");
                    assert_eq!(turn.tokens[0].forms, expected_unit, "{value} {cell:?}");
                }
            }
        }
    }

    for invalid in [0, 10, 1_001, u16::MAX] {
        assert!(matches!(
            compound_ordinal_paradigm(invalid),
            Err(InflectionError::InvalidInput { .. })
        ));
    }
}

#[test]
fn compound_ordinal_source_boundary_is_explicit_and_final() {
    assert_eq!(MIN_COMPOUND_ORDINAL_VALUE, 11);
    assert_eq!(MAX_COMPOUND_ORDINAL_VALUE, 1_000);
    assert_eq!(RuleId::NumeralScopeBoundary.code(), "NUM-SCOPE-BOUNDARY-01");

    let low = compound_ordinal_paradigm(MIN_COMPOUND_ORDINAL_VALUE - 1)
        .expect_err("simple ordinals use their own closed API");
    assert!(matches!(
        low,
        InflectionError::InvalidInput { ref reason }
            if reason.contains("simple-ordinal API")
    ));

    let high = compound_ordinal_paradigm(MAX_COMPOUND_ORDINAL_VALUE + 1)
        .expect_err("higher OCS ordinal formation is source-underdetermined");
    assert!(matches!(
        high,
        InflectionError::InvalidInput { ref reason }
            if reason.contains("reviewed grammars do not determine")
                && !reason.contains("not implemented")
    ));
}

#[test]
fn cardinal_magnitudes_and_composition_through_ten_thousand_are_structured() {
    let thousand = cardinal_magnitude(
        CardinalMagnitudeIdentity::ThousandBackYus,
        Case::Nominative,
        Number::Plural,
    )
    .expect("thousand nominative plural");
    assert_eq!(thousand.primary_text(), "тꙑсѫщѩ");
    assert_eq!(
        thousand
            .variants()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["тꙑсѫщѩ", "тꙑсѫштѧ"]
    );

    let goldens = [
        (100, Case::Nominative, None, "съто"),
        (100, Case::Genitive, None, "съта"),
        (200, Case::Nominative, None, "дъвѣ сътѣ"),
        (300, Case::Nominative, None, "три съта"),
        (500, Case::Genitive, None, "пѧти сътъ"),
        (1_000, Case::Nominative, None, "тꙑсѫщи"),
        (2_000, Case::Nominative, None, "дъвѣ тꙑсѫщи"),
        (3_000, Case::Nominative, None, "три тꙑсѫщѩ"),
        (5_000, Case::Genitive, None, "пѧти тꙑсѫщь"),
        (
            153,
            Case::Nominative,
            Some(Gender::Masculine),
            "съто и пѧть десѧтъ и триѥ",
        ),
        (
            9_999,
            Case::Nominative,
            None,
            "девѧть тꙑсѫщь и девѧть сътъ и девѧть десѧтъ и девѧть",
        ),
        (10_000, Case::Nominative, None, "десѧть тꙑсѫщь"),
    ];
    for (value, case, gender, expected) in goldens {
        let realized = compound_cardinal(value, case, gender).expect("licensed higher cardinal");
        assert_eq!(realized.primary_text(), expected, "{value}");
        let remainder = value % 100;
        let final_digit = if (11..=19).contains(&remainder) {
            remainder - 10
        } else {
            remainder % 10
        };
        let expected_government = match final_digit {
            1 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Singular,
            },
            2 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Dual,
            },
            3 | 4 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Plural,
            },
            _ => old_church_slavonic::NumeralGovernment::GenitivePlural,
        };
        assert_eq!(realized.government(), expected_government);
    }

    let myriad =
        compound_cardinal(10_000, Case::Nominative, None).expect("ten-thousand alternatives");
    assert_eq!(myriad.analyses().len(), 2);
    assert_eq!(myriad.analyses()[0].primary_text(), "десѧть тꙑсѫщь");
    assert_eq!(myriad.analyses()[1].primary_text(), "тъма");

    let one_hundred_thirty =
        compound_cardinal(130, Case::Nominative, None).expect("higher correlated alternatives");
    assert_eq!(one_hundred_thirty.analyses().len(), 2);
    assert_eq!(
        one_hundred_thirty.analyses()[0].primary_text(),
        "съто и триѥ десѧте"
    );
    assert_eq!(
        one_hundred_thirty.analyses()[1].primary_text(),
        "съто и три десѧти"
    );

    let options = CardinalCompositionOptions {
        one_identity: CardinalNumeralIdentity::OneYedyn,
        thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
    };
    assert_eq!(
        compound_cardinal_with_options(1_001, options, Case::Dative, Some(Gender::Feminine),)
            .expect("selected compound lexical doublets")
            .primary_text(),
        "тꙑсѧщи и ѥдьнои",
    );
    let paradigm = compound_cardinal_paradigm_with_options(2_000, options)
        .expect("selected-thousand paradigm");
    assert_eq!(
        paradigm
            .form(Case::Nominative, None)
            .expect("licensed two-thousand cell")
            .primary_text(),
        "дъвѣ тꙑсѧщи",
    );
    assert_eq!(
        paradigm.thousand_identity(),
        CardinalMagnitudeIdentity::ThousandLittleYus
    );
    assert_eq!(paradigm.options(), options);

    assert!(matches!(
        compound_cardinal_with_options(
            1_000,
            CardinalCompositionOptions {
                one_identity: CardinalNumeralIdentity::OneYedin,
                thousand_identity: CardinalMagnitudeIdentity::HundredSto,
            },
            Case::Nominative,
            None,
        ),
        Err(InflectionError::InvalidInput { .. })
    ));
}

#[test]
fn every_integer_through_ten_thousand_has_a_well_formed_analysis() {
    for value in 11..=10_000 {
        let remainder = value % 100;
        let final_digit = if (11..=19).contains(&remainder) {
            remainder - 10
        } else {
            remainder % 10
        };
        let gender = matches!(final_digit, 1..=4).then_some(Gender::Masculine);
        let realized = compound_cardinal(value, Case::Nominative, gender)
            .unwrap_or_else(|error| panic!("{value} nominative failed: {error}"));
        assert_eq!(realized.value(), value);
        assert!(realized.analyses().iter().all(|analysis| {
            !analysis.tokens.is_empty()
                && analysis
                    .tokens
                    .iter()
                    .all(|token| !token.forms.primary_text().is_empty())
        }));
    }

    for value in [
        100, 101, 104, 105, 110, 111, 119, 120, 121, 130, 153, 199, 200, 300, 400, 500, 999, 1_000,
        1_001, 1_010, 1_011, 1_021, 1_100, 1_111, 2_000, 3_000, 4_000, 5_000, 9_999, 10_000,
    ] {
        let remainder = value % 100;
        let final_digit = if (11..=19).contains(&remainder) {
            remainder - 10
        } else {
            remainder % 10
        };
        let requires_gender = matches!(final_digit, 1..=4);
        for cell in CompoundCardinalCell::all().filter(|cell| cell.case != Case::Vocative) {
            let result = compound_cardinal(value, cell.case, cell.gender);
            assert_eq!(
                result.is_ok(),
                cell.gender.is_some() == requires_gender,
                "{value} {cell:?}: {result:?}",
            );
        }
    }
}

#[test]
fn distributive_cardinals_are_fixed_dative_structures_and_exhaustive() {
    assert_eq!(
        DistributiveCardinalCell {
            gender: Some(Gender::Feminine)
        }
        .key(),
        "num:distributive:dat:f"
    );
    let goldens = [
        (1, Some(Gender::Masculine), "по ѥдиному"),
        (2, Some(Gender::Masculine), "по дъвѣма"),
        (20, None, "по дъвѣма десѧтьма"),
        (50, None, "по пѧти десѧтъ"),
        (100, None, "по сътоу"),
        (10_000, None, "по десѧти тꙑсѫщь"),
    ];
    for (value, gender, expected) in goldens {
        let realized = distributive_cardinal(value, gender)
            .unwrap_or_else(|error| panic!("distributive {value}: {error}"));
        assert_eq!(realized.value(), value);
        assert_eq!(realized.cell(), DistributiveCardinalCell { gender });
        assert_eq!(realized.rule_id(), RuleId::NumeralCardinalDistributive);
        assert_eq!(realized.primary_text(), expected);
        assert!(realized.analyses().iter().all(|analysis| {
            analysis.tokens.len() >= 2
                && analysis.tokens[0].role == old_church_slavonic::PhraseRole::Preposition
                && analysis.tokens[0].forms.primary_text() == "по"
                && matches!(
                    analysis.tokens[0].forms.source(),
                    FormSource::ReviewedGrammarTable {
                        rule_id: RuleId::NumeralCardinalDistributive
                    }
                )
                && analysis.tokens[0]
                    .forms
                    .analyses()
                    .iter()
                    .any(|form_analysis| {
                        form_analysis.evidence.iter().any(|evidence| {
                            evidence.authority.as_deref().is_some_and(|authority| {
                                authority.contains("Mark 14:19")
                                    && authority.contains("10:1")
                                    && authority.contains("245344")
                            })
                        })
                    })
        }));
    }

    let valid_structure =
        distributive_cardinal(2, Some(Gender::Masculine)).expect("constructor-adversarial fixture");
    let mut missing_preposition = valid_structure.analyses()[0].clone();
    missing_preposition.tokens.remove(0);
    assert!(matches!(
        old_church_slavonic::RealizedDistributiveCardinal::new(
            valid_structure.value(),
            valid_structure.cell(),
            valid_structure.government(),
            vec![missing_preposition],
        ),
        Err(InflectionError::InvalidInput { .. })
    ));

    let alternate_one = distributive_cardinal_with_one(
        1,
        CardinalNumeralIdentity::OneYedyn,
        Some(Gender::Feminine),
    )
    .expect("selected distributive-one doublet");
    assert_eq!(alternate_one.primary_text(), "по ѥдьнои");
    assert_eq!(
        distributive_cardinal_paradigm_with_one(1, CardinalNumeralIdentity::OneYedyn)
            .expect("selected distributive-one paradigm")
            .form(Some(Gender::Feminine))
            .expect("selected feminine row")
            .primary_text(),
        "по ѥдьнои"
    );

    let options = CardinalCompositionOptions {
        one_identity: CardinalNumeralIdentity::OneYedyn,
        thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
    };
    let alternate = distributive_cardinal_with_options(1_001, options, Some(Gender::Feminine))
        .expect("selected distributive lexical doublets");
    assert_eq!(alternate.primary_text(), "по тꙑсѧщи и ѥдьнои");
    let alternate_paradigm = distributive_cardinal_paradigm_with_options(1_001, options)
        .expect("selected distributive paradigm");
    assert_eq!(alternate_paradigm.options(), options);
    assert_eq!(alternate_paradigm.one_identity(), options.one_identity);
    assert_eq!(
        alternate_paradigm.thousand_identity(),
        options.thousand_identity
    );

    let agreeing = distributive_cardinal_paradigm(2).expect("agreeing distributive paradigm");
    assert_eq!(agreeing.len(), 4);
    assert_eq!(agreeing.successes().count(), 3);
    assert_eq!(agreeing.failures().count(), 1);
    assert_eq!(
        agreeing
            .form(Some(Gender::Feminine))
            .expect("feminine distributive row")
            .primary_text(),
        "по дъвѣма"
    );

    let governing = distributive_cardinal_paradigm(50).expect("governing distributive paradigm");
    assert_eq!(governing.len(), 4);
    assert_eq!(governing.successes().count(), 1);
    assert_eq!(governing.failures().count(), 3);
    assert_eq!(
        governing
            .form(None)
            .expect("ungendered distributive row")
            .primary_text(),
        "по пѧти десѧтъ"
    );

    assert!(matches!(
        distributive_cardinal_with_options(
            1,
            CardinalCompositionOptions {
                one_identity: CardinalNumeralIdentity::TwoDva,
                thousand_identity: CardinalMagnitudeIdentity::ThousandBackYus,
            },
            Some(Gender::Masculine),
        ),
        Err(InflectionError::InvalidInput { .. })
    ));
    assert!(matches!(
        distributive_cardinal_with_options(
            1_000,
            CardinalCompositionOptions {
                one_identity: CardinalNumeralIdentity::OneYedin,
                thousand_identity: CardinalMagnitudeIdentity::HundredSto,
            },
            None,
        ),
        Err(InflectionError::InvalidInput { .. })
    ));

    for invalid in [0, 10_001, u16::MAX] {
        assert!(matches!(
            distributive_cardinal(invalid, None),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            distributive_cardinal_paradigm(invalid),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    for value in 1..=10_000 {
        let remainder = value % 100;
        let final_digit = if (11..=19).contains(&remainder) {
            remainder - 10
        } else {
            remainder % 10
        };
        let requires_gender = matches!(final_digit, 1..=4);
        let expected_government = match final_digit {
            1 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Singular,
            },
            2 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Dual,
            },
            3 | 4 => old_church_slavonic::NumeralGovernment::Agreement {
                number: Number::Plural,
            },
            _ => old_church_slavonic::NumeralGovernment::GenitivePlural,
        };
        for cell in DistributiveCardinalCell::all() {
            let valid = cell.gender.is_some() == requires_gender;
            match distributive_cardinal(value, cell.gender) {
                Ok(realized) => {
                    assert!(valid, "{value} {cell:?}");
                    assert_eq!(realized.cell(), cell);
                    assert_eq!(realized.government(), expected_government);
                    let dative_cardinal = if value <= 10 {
                        let (identity, number) = match value {
                            1 => (CardinalNumeralIdentity::OneYedin, Number::Singular),
                            2 => (CardinalNumeralIdentity::TwoDva, Number::Dual),
                            3 => (CardinalNumeralIdentity::Three, Number::Plural),
                            4 => (CardinalNumeralIdentity::Four, Number::Plural),
                            5 => (CardinalNumeralIdentity::Five, Number::Singular),
                            6 => (CardinalNumeralIdentity::Six, Number::Singular),
                            7 => (CardinalNumeralIdentity::Seven, Number::Singular),
                            8 => (CardinalNumeralIdentity::Eight, Number::Singular),
                            9 => (CardinalNumeralIdentity::Nine, Number::Singular),
                            10 => (CardinalNumeralIdentity::Ten, Number::Singular),
                            _ => unreachable!("loop bounds constrain simple cardinals"),
                        };
                        cardinal_numeral_identity(identity, Case::Dative, number, cell.gender)
                            .expect("licensed simple dative")
                            .primary_text()
                            .to_string()
                    } else {
                        compound_cardinal(value, Case::Dative, cell.gender)
                            .expect("licensed compound dative")
                            .primary_text()
                    };
                    assert_eq!(
                        realized.primary_text(),
                        format!("по {dative_cardinal}"),
                        "{value} {cell:?}"
                    );
                    assert!(realized.analyses().iter().all(|analysis| {
                        analysis.tokens.len() >= 2
                            && analysis.tokens[0].role
                                == old_church_slavonic::PhraseRole::Preposition
                            && analysis.tokens[0].forms.primary_text() == "по"
                            && analysis.tokens[1..].iter().all(|token| {
                                !token.forms.primary_text().is_empty()
                                    && (token.role != old_church_slavonic::PhraseRole::Preposition
                                        || token.forms.primary_text() == "на")
                            })
                    }));
                }
                Err(error) => {
                    assert!(!valid, "{value} {cell:?}: {error}");
                    assert!(matches!(
                        error,
                        InflectionError::HistoricallyInvalidCell {
                            cell: RequestedCell::DistributiveCardinal {
                                value: requested_value,
                                cell: requested_cell,
                            },
                            ..
                        } if requested_value == value && requested_cell == cell
                    ));
                }
            }
        }
    }
}

#[test]
fn explicit_determiner_metadata_supports_arbitrary_lexemes_without_guessing() {
    use old_church_slavonic::advanced::rules::{
        DeterminerDeclension, DeterminerLexeme, PronominalDeclension, determiner_with,
    };

    for (lexeme, cell, expected) in [
        (
            DeterminerLexeme {
                lemma: "новъ".to_string(),
                declension: DeterminerDeclension::Adjectival {
                    class: AdjectiveClass::Hard,
                    form: AdjectiveForm::Short,
                },
            },
            DeterminerCell {
                case: Case::Accusative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Animate,
            },
            "нова",
        ),
        (
            DeterminerLexeme {
                lemma: "вакъ".to_string(),
                declension: DeterminerDeclension::Pronominal(PronominalDeclension::Hard),
            },
            DeterminerCell {
                case: Case::Nominative,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
            },
            "ваци",
        ),
    ] {
        let forms = determiner_with(&lexeme, cell).expect("complete explicit metadata");
        assert_eq!(forms.primary_text(), expected);
        assert!(matches!(
            forms.source(),
            FormSource::ExplicitMetadataRule { .. }
        ));
        assert!(
            forms
                .warnings()
                .contains(&InflectionWarning::PredictedNotDictionaryBacked)
        );
    }
}

#[test]
fn source_order_primary_access_never_discards_alternatives() {
    let noun_variants = noun("аблань", Case::Genitive, Number::Dual).expect("variant noun cell");
    assert_eq!(noun_variants.primary_text(), "абланью");
    assert_eq!(
        noun_variants.texts().collect::<Vec<_>>(),
        ["абланью", "абланию"]
    );

    let aorists = aorist("бꙑти", Person::First, Number::Singular).expect("listed aorists");
    assert_eq!(aorists.primary().text, "бѣхъ");
    assert_eq!(aorists.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
    assert_eq!(aorists.source(), &FormSource::DictionaryTable);
    assert!(
        aorists
            .warnings()
            .contains(&InflectionWarning::MultipleDictionaryVariants)
    );
    let be = Verb::resolve("бꙑти").expect("unique verb");
    assert_eq!(
        be.aorist(Person::First, Number::Singular)
            .expect("handle aorists"),
        aorists
    );
    assert_eq!(
        by_id::finite_by_id(
            be.id(),
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            },
        )
        .expect("ID aorists"),
        aorists
    );
    assert_eq!(
        be.finite_paradigm()
            .form(FiniteTense::Aorist, Person::First, Number::Singular,)
            .expect("successful paradigm aorists"),
        &aorists
    );

    let consumed = noun_variants.into_primary_text();
    assert_eq!(consumed, "абланью");
}

#[test]
fn validated_lemmas_and_explicit_variant_selection_are_ergonomic() {
    let lemma = Lemma::parse("обѣдъ").expect("validated lemma");
    assert_eq!(lemma.script(), Script::Cyrillic);
    assert_eq!(
        noun(&lemma, Case::Dative, Number::Dual),
        noun("обѣдъ", Case::Dative, Number::Dual)
    );

    let normalized = Lemma::parse("И\u{306}").expect("NFC normalized lemma");
    assert_eq!(normalized.as_str(), "Й");
    for invalid in ["слоword", "слоα", "<слово>", "\u{301}слово"] {
        assert!(matches!(
            Lemma::parse(invalid),
            Err(InflectionError::InvalidLemma { input, .. }) if input == invalid
        ));
        assert!(matches!(
            noun(invalid, Case::Nominative, Number::Singular),
            Err(InflectionError::InvalidLemma { input, .. }) if input == invalid
        ));
    }

    let unique = noun("обѣдъ", Case::Dative, Number::Dual).expect("unique form");
    assert_eq!(unique.unique_text().expect("one form"), "обѣдома");
    assert_eq!(
        unique
            .select(VariantPolicy::SourceFirst)
            .expect("explicit source-first")
            .text,
        "обѣдома"
    );
    let variants = aorist("бꙑти", Person::First, Number::Singular).expect("variants");
    let error = variants.unique_text().expect_err("must reject two forms");
    assert_eq!(error.lemma, "бꙑти");
    assert_eq!(error.variant_count, 2);
}

#[test]
fn restrained_prelude_supports_an_ordinary_workflow() {
    use old_church_slavonic::prelude::*;

    let lemma = Lemma::parse("обѣдъ").expect("validated prelude lemma");
    let forms = noun(&lemma, Case::Dative, Number::Dual).expect("prelude inflection");
    assert_eq!(forms.unique_text().expect("unique table cell"), "обѣдома");
    assert_eq!(
        Noun::resolve(&lemma).expect("prelude handle").lemma(),
        lemma.as_str()
    );
}

#[test]
fn ordinary_closed_class_handles_share_direct_by_id_and_paradigm_paths() {
    let which = Determiner::resolve("кꙑи").expect("determiner");
    assert_eq!(
        Determiner::from_id(which.id()).expect("rebound determiner"),
        which
    );
    let determiner_cell = DeterminerCell {
        case: Case::Accusative,
        number: Number::Singular,
        gender: Gender::Feminine,
        animacy: Animacy::Inanimate,
    };
    let direct = determiner(
        "кꙑи",
        determiner_cell.case,
        determiner_cell.number,
        determiner_cell.gender,
        determiner_cell.animacy,
    );
    assert_eq!(
        direct,
        which.form(
            determiner_cell.case,
            determiner_cell.number,
            determiner_cell.gender,
            determiner_cell.animacy,
        )
    );
    assert_eq!(direct, by_id::determiner_by_id(which.id(), determiner_cell));
    assert_eq!(
        direct.as_ref().expect("determiner cell"),
        determiner_paradigm("кꙑи")
            .expect("determiner paradigm")
            .form(
                determiner_cell.case,
                determiner_cell.number,
                determiner_cell.gender,
                determiner_cell.animacy,
            )
            .expect("paradigm cell")
    );

    let reflexive = Pronoun::resolve("сѧ").expect("reflexive pronoun");
    assert_eq!(
        Pronoun::from_id(reflexive.id()).expect("rebound pronoun"),
        reflexive
    );
    let plain_cell = UngenderedCell {
        case: Case::Genitive,
        number: Number::Singular,
    };
    let direct = pronoun("сѧ", plain_cell.case, plain_cell.number);
    assert_eq!(direct, reflexive.form(plain_cell.case, plain_cell.number));
    assert_eq!(direct, by_id::pronoun_by_id(reflexive.id(), plain_cell));
    assert_eq!(
        direct.as_ref().expect("pronoun"),
        reflexive
            .paradigm()
            .form(plain_cell.case, plain_cell.number)
            .expect("row")
    );

    let first = Pronoun::resolve("азъ").expect("personal pronoun table");
    let personal_cell = PersonalPronounCell {
        case: Case::Dative,
        number: Number::Singular,
        person: Person::First,
    };
    let direct = personal_pronoun(
        "азъ",
        personal_cell.case,
        personal_cell.number,
        personal_cell.person,
    );
    assert_eq!(
        direct,
        first.personal(
            personal_cell.case,
            personal_cell.number,
            personal_cell.person
        )
    );
    assert_eq!(
        direct,
        by_id::personal_pronoun_by_id(first.id(), personal_cell)
    );
    assert_eq!(
        direct.as_ref().expect("personal pronoun"),
        first
            .personal_paradigm()
            .form(
                personal_cell.case,
                personal_cell.number,
                personal_cell.person
            )
            .expect("personal paradigm row")
    );

    let third = Pronoun::resolve("онъ").expect("gendered pronoun");
    let gendered_cell = GenderedCell {
        case: Case::Dative,
        number: Number::Singular,
        gender: Gender::Feminine,
    };
    let direct = gendered_pronoun(
        "онъ",
        gendered_cell.case,
        gendered_cell.number,
        gendered_cell.gender,
    );
    assert_eq!(
        direct,
        third.gendered(
            gendered_cell.case,
            gendered_cell.number,
            gendered_cell.gender,
        )
    );
    assert_eq!(
        direct,
        by_id::gendered_pronoun_by_id(third.id(), gendered_cell)
    );
    assert_eq!(
        direct.as_ref().expect("gendered pronoun"),
        third
            .gendered_paradigm()
            .form(
                gendered_cell.case,
                gendered_cell.number,
                gendered_cell.gender,
            )
            .expect("gendered paradigm row")
    );

    let nine = Numeral::resolve("девѧть").expect("cardinal numeral");
    assert_eq!(Numeral::from_id(nine.id()).expect("rebound numeral"), nine);
    let numeral_cell = UngenderedCell {
        case: Case::Genitive,
        number: Number::Singular,
    };
    let direct = numeral("девѧть", numeral_cell.case, numeral_cell.number);
    assert_eq!(direct, nine.form(numeral_cell.case, numeral_cell.number));
    assert_eq!(direct, by_id::numeral_by_id(nine.id(), numeral_cell));
    assert_eq!(
        direct.as_ref().expect("numeral"),
        nine.paradigm()
            .form(numeral_cell.case, numeral_cell.number)
            .expect("numeral paradigm row")
    );

    let first_ordinal = Numeral::resolve("прьвъ").expect("gendered numeral");
    let numeral_cell = GenderedCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Feminine,
    };
    let direct = gendered_numeral(
        "прьвъ",
        numeral_cell.case,
        numeral_cell.number,
        numeral_cell.gender,
    );
    assert_eq!(
        direct,
        first_ordinal.gendered(numeral_cell.case, numeral_cell.number, numeral_cell.gender,)
    );
    assert_eq!(
        direct,
        by_id::gendered_numeral_by_id(first_ordinal.id(), numeral_cell)
    );
    assert_eq!(
        direct.as_ref().expect("gendered numeral"),
        first_ordinal
            .gendered_paradigm()
            .form(numeral_cell.case, numeral_cell.number, numeral_cell.gender)
            .expect("gendered numeral paradigm row")
    );
}

#[test]
fn reviewed_personal_reflexive_and_anaphoric_pronouns_are_complete_and_typed() {
    let first_dative = personal_pronoun_with(
        PersonalPronounIdentity::First,
        Case::Dative,
        Number::Singular,
        PronounFormSelection::All,
    )
    .expect("reviewed first-person dative");
    assert_eq!(first_dative.texts().collect::<Vec<_>>(), ["мьнѣ", "ми"]);
    assert!(
        first_dative
            .analyses()
            .iter()
            .flat_map(|analysis| &analysis.evidence)
            .any(|evidence| {
                evidence
                    .source_feature
                    .as_deref()
                    .is_some_and(|feature| feature.ends_with("marked-clitic"))
            })
    );

    let disputed = personal_pronoun_with(
        PersonalPronounIdentity::First,
        Case::Dative,
        Number::Dual,
        PronounFormSelection::MarkedClitic,
    )
    .expect("source-disputed first-person dual clitic");
    assert_eq!(disputed.primary_text(), "на");
    assert!(
        disputed
            .warnings()
            .contains(&InflectionWarning::IncludesDisputedForms)
    );
    assert_eq!(
        disputed.analyses()[0].evidence[0].provenance,
        MetadataProvenance::DisputedGrammarTable
    );
    assert!(matches!(
        personal_pronoun_with(
            PersonalPronounIdentity::First,
            Case::Nominative,
            Number::Singular,
            PronounFormSelection::MarkedClitic,
        ),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));

    for case in [
        Case::Accusative,
        Case::Genitive,
        Case::Dative,
        Case::Instrumental,
        Case::Locative,
    ] {
        assert!(reflexive_pronoun(case, PronounFormSelection::All).is_ok());
    }
    assert!(matches!(
        reflexive_pronoun(Case::Nominative, PronounFormSelection::All),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        reflexive_pronoun(Case::Vocative, PronounFormSelection::All),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        reflexive_pronoun(Case::Genitive, PronounFormSelection::MarkedClitic),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));

    let mut anaphoric_cells = 0;
    for environment in [
        AnaphoricEnvironment::Free,
        AnaphoricEnvironment::AfterPreposition,
    ] {
        for number in Number::ALL {
            for gender in Gender::ALL {
                for case in [
                    Case::Accusative,
                    Case::Genitive,
                    Case::Dative,
                    Case::Instrumental,
                    Case::Locative,
                ] {
                    anaphoric_pronoun(case, number, gender, environment)
                        .expect("reviewed anaphoric cell");
                    anaphoric_cells += 1;
                }
                assert!(matches!(
                    anaphoric_pronoun(Case::Nominative, number, gender, environment),
                    Err(InflectionError::HistoricallyInvalidCell { .. })
                ));
                assert!(matches!(
                    anaphoric_pronoun(Case::Vocative, number, gender, environment),
                    Err(InflectionError::HistoricallyInvalidCell { .. })
                ));
            }
        }
    }
    assert_eq!(anaphoric_cells, 90);
    assert_eq!(
        anaphoric_pronoun(
            Case::Accusative,
            Number::Singular,
            Gender::Masculine,
            AnaphoricEnvironment::AfterPreposition,
        )
        .expect("prepositional anaphoric")
        .primary_text(),
        "н҄ь"
    );
}

#[test]
fn dictionary_form_pages_route_to_intrinsic_pronoun_identities() {
    assert!(matches!(
        personal_pronoun("азъ", Case::Nominative, Number::Singular, Person::Second,),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));

    for alias in ["азъ", "вѣ", "мꙑ", "наю"] {
        let result = personal_pronoun(alias, Case::Genitive, Number::Plural, Person::First)
            .expect("first-person source-union identity");
        assert_eq!(result.lemma(), "азъ");
        assert_eq!(result.primary_text(), "насъ");
        assert_eq!(
            result
                .warnings()
                .contains(&InflectionWarning::LexicalAliasUsed {
                    canonical: "азъ".to_string(),
                }),
            alias != "азъ"
        );
    }

    for alias in ["тꙑ", "ва", "вꙑ", "ваю"] {
        let result = personal_pronoun(alias, Case::Genitive, Number::Plural, Person::Second)
            .expect("second-person source-union identity");
        assert_eq!(result.lemma(), "тꙑ");
        assert_eq!(result.primary_text(), "васъ");
        assert_eq!(
            result
                .warnings()
                .contains(&InflectionWarning::LexicalAliasUsed {
                    canonical: "тꙑ".to_string(),
                }),
            alias != "тꙑ"
        );
    }

    let reflexive = pronoun("сѧ", Case::Instrumental, Number::Dual)
        .expect("numberless reflexive source-union identity");
    assert_eq!(reflexive.lemma(), "сѧ");
    assert_eq!(reflexive.primary_text(), "собоѭ");

    for alias in ["и", "ѥ", "ѭ", "ими"] {
        let result = gendered_pronoun(alias, Case::Genitive, Number::Singular, Gender::Masculine)
            .expect("anaphoric source-union identity");
        assert_eq!(result.lemma(), "и");
        assert_eq!(result.primary_text(), "ѥго");
        assert_eq!(
            result
                .warnings()
                .contains(&InflectionWarning::LexicalAliasUsed {
                    canonical: "и".to_string(),
                }),
            alias != "и"
        );
    }

    assert!(matches!(
        gendered_pronoun("ѥ", Case::Nominative, Number::Singular, Gender::Masculine,),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));

    let demonstrative =
        gendered_pronoun("онъ", Case::Nominative, Number::Singular, Gender::Masculine)
            .expect("independent demonstrative");
    assert_eq!(demonstrative.primary_text(), "онъ");
    assert_eq!(
        demonstrative.source(),
        &FormSource::ReviewedGrammarTable {
            rule_id: RuleId::PronounPronominalHard
        }
    );
}

#[test]
fn regular_pronominal_pronouns_use_reviewed_grammar_before_source_tables() {
    let goldens = [
        ("тъ", "тоѩ", "тъ"),
        ("онъ", "оноѩ", "онъ"),
        ("она", "оноѩ", "онъ"),
        ("оно", "оноѩ", "онъ"),
        ("вашь", "вашеѩ", "вашь"),
        ("нашь", "нашеѩ", "нашь"),
        ("мои", "моѥѩ", "мои"),
        ("твои", "твоѥѩ", "твои"),
        ("свои", "своѥѩ", "свои"),
        ("вьсѣкъ", "вьсѣкоѩ", "вьсѣкъ"),
    ];
    for (lemma, expected, canonical) in goldens {
        let result = gendered_pronoun(lemma, Case::Genitive, Number::Singular, Gender::Feminine)
            .expect("regular pronominal identity");
        assert_eq!(result.lemma(), canonical, "{lemma}");
        assert_eq!(result.primary_text(), expected, "{lemma}");
        assert!(matches!(
            result.source(),
            FormSource::ReviewedGrammarTable { .. }
        ));
        assert_eq!(
            result.analyses()[0].evidence[0].provenance,
            MetadataProvenance::ReviewedGrammarTable
        );
        assert_eq!(result.analyses()[0].evidence[0].source_form, None);
        assert!(
            result
                .warnings()
                .contains(&InflectionWarning::LexicalAliasUsed {
                    canonical: canonical.to_string(),
                })
                == (lemma != canonical)
        );
    }

    let on_id = only_id("онъ", PartOfSpeech::Pronoun);
    let cell = GenderedCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
    };
    assert_eq!(
        raw_features::closed_class_by_id(&on_id, PartOfSpeech::Pronoun, cell.closed_class())
            .expect("raw dictionary diagnostic")
            .source(),
        &FormSource::DictionaryTable
    );

    assert!(matches!(
        pronoun("тъ", Case::Nominative, Number::Singular),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        personal_pronoun("тъ", Case::Nominative, Number::Singular, Person::Third),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        gendered_pronoun("тъ", Case::Vocative, Number::Singular, Gender::Masculine),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn every_regular_class_2_p_identity_is_available_through_the_typed_api() {
    assert_eq!(StandardPronominalIdentity::ALL.len(), 32);
    let mut successes = 0;
    for identity in StandardPronominalIdentity::ALL {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    let result = regular_pronominal(identity, case, number, gender);
                    let supported = case != Case::Vocative
                        && (identity.number_restriction() == NumberRestriction::All
                            || number == Number::Dual);
                    if supported {
                        let result = result.unwrap_or_else(|error| {
                            panic!("{identity:?} {case:?} {number:?} {gender:?}: {error}")
                        });
                        assert_eq!(result.lemma(), identity.canonical_lemma());
                        assert!(matches!(
                            result.source(),
                            FormSource::ReviewedGrammarTable { .. }
                        ));
                        assert_eq!(result.analyses()[0].evidence[0].source_form, None);
                        successes += 1;
                    } else {
                        let Err(InflectionError::HistoricallyInvalidCell {
                            cell:
                                RequestedCell::ClosedClass {
                                    part_of_speech,
                                    cell,
                                },
                            ..
                        }) = result
                        else {
                            panic!(
                                "expected typed invalid cell for {identity:?} {case:?} {number:?} {gender:?}"
                            );
                        };
                        assert_eq!(part_of_speech, identity.part_of_speech());
                        assert_eq!(cell.case, case);
                        assert_eq!(cell.number, number);
                        assert_eq!(cell.gender, Some(gender));
                    }
                }
            }
        }
    }
    assert_eq!(successes, 1_656);

    assert_eq!(
        regular_pronominal(
            StandardPronominalIdentity::NumeralDva,
            Case::Nominative,
            Number::Dual,
            Gender::Feminine,
        )
        .expect("dual-only source citation")
        .primary_text(),
        "дъвѣ"
    );
    assert_eq!(
        regular_pronominal(
            StandardPronominalIdentity::NumeralTroi,
            Case::Nominative,
            Number::Plural,
            Gender::Masculine,
        )
        .expect("j-stem class member")
        .primary_text(),
        "трои"
    );
}

#[test]
fn pronominal_adjectives_use_reviewed_short_forms_and_preserve_long_and_raw_tables() {
    for (lemma, canonical, reviewed, copied) in [
        ("самъ", "самъ", "самого", "сама"),
        ("единъ", "ѥдинъ", "ѥдиного", "едина"),
        ("единакъ", "ѥдинакъ", "ѥдинакого", "единака"),
    ] {
        let reviewed_result = short_adjective(
            lemma,
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("reviewed pronominal adjective");
        assert_eq!(reviewed_result.lemma(), canonical, "{lemma}");
        assert_eq!(reviewed_result.primary_text(), reviewed, "{lemma}");
        assert!(matches!(
            reviewed_result.source(),
            FormSource::ReviewedGrammarTable { .. }
        ));

        let id = only_id(lemma, PartOfSpeech::Adjective);
        let raw = raw_features::dictionary_form_by_id(&id, "adj:short:gen:sg:m:in")
            .expect("copied diagnostic adjective cell");
        assert_eq!(raw.primary_text(), copied, "{lemma}");
    }

    assert_eq!(
        long_adjective(
            "самъ",
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("attested aberrant long form")
        .primary_text(),
        "самаѥго"
    );
    assert_eq!(
        noun("единакъ", Case::Genitive, Number::Singular)
            .expect("homonymous monk noun remains nominal")
            .primary_text(),
        "единака"
    );

    let vsek_id = only_id("вьсѣкъ", PartOfSpeech::Pronoun);
    assert_eq!(
        gendered_pronoun(
            "вьсѣкъ",
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
        )
        .expect("reviewed hard-pronominal form")
        .primary_text(),
        "вьсѣкого"
    );
    assert_eq!(
        raw_features::closed_class_by_id(
            &vsek_id,
            PartOfSpeech::Pronoun,
            GenderedCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Gender::Masculine,
            }
            .closed_class(),
        )
        .expect("copied diagnostic pronoun cell")
        .primary_text(),
        "вьсѣка"
    );
}

#[test]
fn explicit_pronominal_rules_support_oov_lexemes_and_trace_velar_palatalization() {
    let lexeme = PronominalLexeme {
        lemma: "такъ".to_string(),
        declension: PronominalDeclension::Hard,
    };
    let form = pronominal_with(&lexeme, Case::Nominative, Number::Plural, Gender::Masculine)
        .expect("explicit regular pronominal lexeme");
    assert_eq!(form.primary_text(), "таци");
    assert_eq!(
        form.source(),
        &FormSource::ExplicitMetadataRule {
            rule_id: RuleId::PronounPronominalHard
        }
    );
    assert_eq!(form.trace().len(), 2);
    assert_eq!(form.trace()[0].rule_id, RuleId::PronounPronominalVelar);
}

#[test]
fn exceptional_pronouns_use_complete_reviewed_inventories_and_keep_raw_tables() {
    let relative = gendered_pronoun("иже", Case::Genitive, Number::Singular, Gender::Feminine)
        .expect("free relative pronoun");
    assert_eq!(relative.primary_text(), "ѥѩже");
    assert_eq!(
        relative_pronoun(
            Case::Dative,
            Number::Singular,
            Gender::Feminine,
            AnaphoricEnvironment::AfterPreposition,
        )
        .expect("post-prepositional relative pronoun")
        .primary_text(),
        "н҄ѥиже"
    );

    let proximal = gendered_pronoun("сь", Case::Accusative, Number::Plural, Gender::Masculine)
        .expect("reviewed proximal pronoun");
    assert_eq!(proximal.primary_text(), "сиѩ");
    assert_eq!(
        proximal.source(),
        &FormSource::ReviewedGrammarTable {
            rule_id: RuleId::PronounUniqueSi
        }
    );
    let si_id = only_id("сь", PartOfSpeech::Pronoun);
    let raw_si = raw_features::closed_class_by_id(
        &si_id,
        PartOfSpeech::Pronoun,
        GenderedCell {
            case: Case::Accusative,
            number: Number::Plural,
            gender: Gender::Masculine,
        }
        .closed_class(),
    )
    .expect("raw source table remains inspectable");
    assert_eq!(raw_si.primary_text(), "сиꙗ");
    assert_eq!(raw_si.source(), &FormSource::DictionaryTable);

    for number in Number::ALL {
        assert_eq!(
            pronoun("къто", Case::Accusative, number)
                .expect("numberless interrogative")
                .primary_text(),
            "къто"
        );
    }
    assert_eq!(
        interrogative_pronoun(InterrogativePronounIdentity::Chto, Case::Genitive)
            .expect("variant-rich interrogative")
            .texts()
            .collect::<Vec<_>>(),
        ["чесо", "чьсо", "чесого"]
    );

    let kyi = determiner(
        "кꙑи",
        Case::Dative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )
    .expect("reviewed unique determiner");
    assert_eq!(kyi.primary_text(), "коѥму");
    assert_eq!(
        kyi.source(),
        &FormSource::ReviewedGrammarTable {
            rule_id: RuleId::DeterminerInterrogativeKyi
        }
    );

    assert!(matches!(
        pronoun("иже", Case::Nominative, Number::Singular),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        gendered_pronoun(
            "къто",
            Case::Nominative,
            Number::Singular,
            Gender::Masculine
        ),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));

    for lemma in ["иже", "сь"] {
        let paradigm = Pronoun::resolve(lemma)
            .expect("reviewed agreeing identity")
            .gendered_paradigm();
        assert_eq!(paradigm.successes().count(), 54, "{lemma}");
        assert_eq!(paradigm.failures().count(), 9, "{lemma}");
    }
    let kto = Pronoun::resolve("къто")
        .expect("numberless interrogative identity")
        .paradigm();
    assert_eq!(kto.successes().count(), 18);
    assert_eq!(kto.failures().count(), 3);
    let kyi = determiner_paradigm("кꙑи").expect("unique determiner paradigm");
    assert_eq!(kyi.successes().count(), 108);
    assert_eq!(kyi.failures().count(), 18);
}

#[test]
fn derived_negative_interrogative_uses_the_reviewed_family_not_the_raw_table() {
    let expected = [
        (Case::Nominative, "никъто"),
        (Case::Genitive, "никого"),
        (Case::Dative, "никому"),
        (Case::Accusative, "никъто"),
        (Case::Instrumental, "ницѣмь"),
        (Case::Locative, "никомь"),
    ];
    for number in Number::ALL {
        for (case, expected) in expected {
            assert_eq!(
                pronoun("никъто", case, number)
                    .expect("complete reviewed negative-interrogative family")
                    .primary_text(),
                expected,
            );
        }
        assert!(matches!(
            pronoun("никъто", Case::Vocative, number),
            Err(InflectionError::HistoricallyInvalidCell { .. })
        ));
    }

    let reviewed = pronoun("никъто", Case::Dative, Number::Singular)
        .expect("reviewed negative-interrogative family");
    assert_eq!(reviewed.primary_text(), "никому");
    assert_eq!(
        reviewed.source(),
        &FormSource::ReviewedGrammarTable {
            rule_id: old_church_slavonic::trace::RuleId::PronounDerivedFamily,
        }
    );
    assert!(reviewed.analyses()[0].evidence.iter().any(|evidence| {
        evidence
            .authority
            .as_deref()
            .is_some_and(|authority| authority.contains("Polivanova 2023 §§316, 380"))
    }));

    let id = only_id("никъто", PartOfSpeech::Pronoun);
    let raw = raw_features::closed_class_by_id(
        &id,
        PartOfSpeech::Pronoun,
        UngenderedCell {
            case: Case::Dative,
            number: Number::Singular,
        }
        .closed_class(),
    )
    .expect("raw copied table cell");
    assert_eq!(raw.primary_text(), "никомоу");
}

#[test]
fn no_dual_mixed_pronouns_and_ordered_doublets_are_typed() {
    let ves = irregular_agreeing(
        IrregularAgreeingIdentity::TotalVes,
        Case::Nominative,
        Number::Singular,
        Gender::Feminine,
    )
    .expect("mixed totalizing pronoun");
    assert_eq!(ves.texts().collect::<Vec<_>>(), ["вьса", "вьсѣ"]);
    assert_eq!(ves.analyses().len(), 2);
    assert!(matches!(
        irregular_agreeing(
            IrregularAgreeingIdentity::TotalVes,
            Case::Nominative,
            Number::Dual,
            Gender::Feminine,
        ),
        Err(InflectionError::HistoricallyInvalidCell { .. })
    ));
    assert_eq!(
        irregular_agreeing(
            IrregularAgreeingIdentity::DemonstrativeSic,
            Case::Instrumental,
            Number::Plural,
            Gender::Neuter,
        )
        .expect("mixed demonstrative pronoun")
        .primary_text(),
        "сицѣми"
    );
}

#[test]
fn reviewed_pronoun_paradigms_expose_every_valid_and_invalid_cell() {
    let first = Pronoun::resolve("азъ")
        .expect("first-person identity")
        .personal_paradigm();
    assert_eq!(first.successes().count(), 18);
    assert_eq!(first.failures().count(), 45);
    assert!(
        first
            .failures()
            .all(|(_, error)| matches!(error, InflectionError::HistoricallyInvalidCell { .. }))
    );

    let reflexive = Pronoun::resolve("сѧ")
        .expect("reflexive identity")
        .paradigm();
    assert_eq!(reflexive.successes().count(), 15);
    assert_eq!(reflexive.failures().count(), 6);
    assert!(
        reflexive
            .failures()
            .all(|(_, error)| matches!(error, InflectionError::HistoricallyInvalidCell { .. }))
    );

    let anaphoric = Pronoun::resolve("и")
        .expect("third-person anaphoric identity")
        .gendered_paradigm();
    assert_eq!(anaphoric.successes().count(), 45);
    assert_eq!(anaphoric.failures().count(), 18);
    assert!(
        anaphoric
            .failures()
            .all(|(_, error)| matches!(error, InflectionError::HistoricallyInvalidCell { .. }))
    );
}

#[test]
fn paradigm_access_distinguishes_unrepresented_and_failed_cells() {
    let imperative_table = imperative_paradigm("благословити").expect("imperative table");
    assert!(matches!(
        imperative_table.form(Person::Third, Number::Plural),
        Err(ParadigmLookupError::NotRepresented)
    ));

    let determiner_id = only_id("кꙑи", PartOfSpeech::Determiner);
    let requested = ClosedClassCell {
        case: Case::Vocative,
        number: Number::Dual,
        gender: Some(Gender::Masculine),
        person: None,
    };
    assert_eq!(
        determiner(
            "кꙑи",
            requested.case,
            requested.number,
            requested.gender.expect("gendered request"),
            Animacy::Inanimate,
        ),
        Err(InflectionError::HistoricallyInvalidCell {
            lexeme_id: determiner_id,
            cell: RequestedCell::Determiner(DeterminerCell {
                case: requested.case,
                number: requested.number,
                gender: requested.gender.expect("gendered request"),
                animacy: Animacy::Inanimate,
            }),
        })
    );

    let determiner_table = determiner_paradigm("кꙑи").expect("determiner table");
    assert!(matches!(
        determiner_table.form(
            Case::Vocative,
            Number::Dual,
            Gender::Masculine,
            Animacy::Inanimate,
        ),
        Err(ParadigmLookupError::Failed(
            InflectionError::HistoricallyInvalidCell { .. }
        ))
    ));
    assert!(!determiner_table.successes().collect::<Vec<_>>().is_empty());
    assert!(!determiner_table.failures().collect::<Vec<_>>().is_empty());
    assert_eq!(
        determiner_table.clone().into_rows().len(),
        determiner_table.len()
    );
}

#[test]
fn resolved_noun_and_adjective_handles_share_free_and_by_id_paths() {
    let meal = Noun::resolve("обѣдъ").expect("unique noun");
    let rebound = Noun::from_id(meal.id()).expect("valid noun ID");
    assert_eq!(meal, rebound);
    let direct = noun("обѣдъ", Case::Dative, Number::Dual);
    let method = meal.form(Case::Dative, Number::Dual);
    let by_id = by_id::noun_by_id(
        meal.id(),
        NounCell {
            case: Case::Dative,
            number: Number::Dual,
        },
    );
    assert_eq!(direct, method);
    assert_eq!(method, by_id);

    let paradigm = meal.paradigm();
    assert_eq!(paradigm.lemma(), "обѣдъ");
    assert_eq!(paradigm.id(), meal.id());
    assert_eq!(paradigm.len(), Case::ALL.len() * Number::ALL.len());
    for outcome in &paradigm {
        assert_eq!(
            &outcome.result,
            &noun("обѣдъ", outcome.cell.case, outcome.cell.number)
        );
        assert_eq!(
            &outcome.result,
            &meal.form(outcome.cell.case, outcome.cell.number)
        );
    }
    assert_eq!(noun_paradigm("обѣдъ").expect("lemma paradigm"), paradigm);
    assert_eq!(
        by_id::noun_paradigm_by_id(meal.id()).expect("ID paradigm"),
        paradigm
    );

    let good = Adjective::resolve("добръ").expect("unique adjective");
    let long = good.long(
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    assert_eq!(
        long,
        long_adjective(
            "добръ",
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
    );
    let adjective_table = adjective_paradigm("добръ").expect("adjective paradigm");
    assert_eq!(adjective_table, good.paradigm());
    for outcome in &adjective_table {
        let direct = match outcome.cell.form {
            AdjectiveForm::Long => long_adjective(
                "добръ",
                outcome.cell.case,
                outcome.cell.number,
                outcome.cell.gender,
                outcome.cell.animacy,
            ),
            AdjectiveForm::Short => short_adjective(
                "добръ",
                outcome.cell.case,
                outcome.cell.number,
                outcome.cell.gender,
                outcome.cell.animacy,
            ),
        };
        assert_eq!(&outcome.result, &direct);
        assert_eq!(
            &outcome.result,
            &by_id::adjective_by_id(good.id(), outcome.cell)
        );
    }
}

#[test]
fn resolved_verb_handle_and_every_paradigm_use_the_cell_resolver() {
    let bless = Verb::resolve("благословити").expect("unique verb");
    assert_eq!(Verb::from_id(bless.id()).expect("valid verb ID"), bless);
    assert_eq!(
        bless.present(Person::First, Number::Singular),
        present("благословити", Person::First, Number::Singular)
    );
    assert_eq!(
        present_paradigm("благословити").expect("present table"),
        bless.present_paradigm()
    );
    assert_eq!(
        by_id::present_paradigm_by_id(bless.id()).expect("present table by ID"),
        bless.present_paradigm()
    );
    for outcome in bless.present_paradigm() {
        assert_eq!(outcome.cell.tense, FiniteTense::Present);
        assert_eq!(
            outcome.result,
            present("благословити", outcome.cell.person, outcome.cell.number)
        );
    }

    let finite_table = finite_paradigm("благословити").expect("finite paradigm");
    assert_eq!(finite_table.len(), 27);
    assert_eq!(finite_table, bless.finite_paradigm());
    for outcome in &finite_table {
        assert_eq!(
            &outcome.result,
            &finite(
                "благословити",
                outcome.cell.tense,
                outcome.cell.person,
                outcome.cell.number,
            )
        );
        assert_eq!(
            &outcome.result,
            &by_id::finite_by_id(bless.id(), outcome.cell)
        );
    }

    let imperative_table = imperative_paradigm("благословити").expect("imperative paradigm");
    assert_eq!(imperative_table, bless.imperative_paradigm());
    for outcome in &imperative_table {
        assert_eq!(
            &outcome.result,
            &imperative("благословити", outcome.cell.person, outcome.cell.number,)
        );
        assert_eq!(
            &outcome.result,
            &by_id::imperative_by_id(bless.id(), outcome.cell)
        );
    }

    let l_table = l_participle_paradigm("благословити").expect("l-participle paradigm");
    assert_eq!(l_table, bless.l_participle_paradigm());
    for outcome in &l_table {
        assert_eq!(
            &outcome.result,
            &l_participle("благословити", outcome.cell.gender, outcome.cell.number,)
        );
        assert_eq!(
            &outcome.result,
            &by_id::l_participle_by_id(bless.id(), outcome.cell)
        );
    }
}

#[test]
fn named_participle_handle_preserves_competing_metadata_analyses() {
    let participle = past_active_participle("благословити").expect("past active participle");
    let handle_participle = Verb::resolve("благословити")
        .expect("unique verb")
        .past_active_participle()
        .expect("handle participle");
    assert_eq!(participle, handle_participle);
    assert_eq!(participle.kind(), ParticipleKind::PastActive);
    assert_eq!(
        participle
            .citation()
            .expect("citation")
            .texts()
            .collect::<Vec<_>>(),
        ["благословл҄ь", "благословивъ"]
    );

    let declined = participle
        .short(
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        )
        .expect("metadata-backed declined participle");
    assert_eq!(
        declined.texts().collect::<Vec<_>>(),
        ["благословл҄ьша", "благословивъша"]
    );
    assert_eq!(declined.source(), &FormSource::DictionaryMetadataAnalyses);
    assert_eq!(declined.analyses().len(), 2);
    assert!(declined.analyses().iter().all(|analysis| {
        analysis.evidence.iter().any(|evidence| {
            evidence.provenance == MetadataProvenance::DictionaryPrincipalPart
                && evidence.source_feature.as_deref()
                    == Some("verb:participle:past-active:citation")
        })
    }));

    let cell = ParticipleCell {
        kind: ParticipleKind::PastActive,
        adjective: AdjectiveCell {
            case: Case::Genitive,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        },
    };
    assert_eq!(
        by_id::participle_by_id(participle.id(), cell).expect("ID declined participle"),
        declined
    );
    assert_eq!(
        participle_form("благословити", cell).expect("lemma declined participle"),
        declined
    );

    let paradigm = participle.paradigm();
    assert_eq!(
        participle_paradigm("благословити", ParticipleKind::PastActive)
            .expect("lemma participle paradigm"),
        paradigm
    );
    assert_eq!(
        by_id::participle_paradigm_by_id(participle.id(), ParticipleKind::PastActive)
            .expect("ID participle paradigm"),
        paradigm
    );
    for outcome in &paradigm {
        assert_eq!(
            &outcome.result,
            &participle_form("благословити", outcome.cell)
        );
        assert_eq!(
            &outcome.result,
            &by_id::participle_by_id(participle.id(), outcome.cell)
        );
    }
}

#[test]
fn advanced_explicit_metadata_is_independent_and_typed() {
    let oov = noun_with(
        &NounLexeme {
            lemma: "роботъ".to_string(),
            class: NounClass::OMasculineHard,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        },
        NounCell {
            case: Case::Locative,
            number: Number::Plural,
        },
    )
    .expect("explicit noun metadata");
    assert_eq!(oov.primary_text(), "роботѣхъ");
    assert!(matches!(
        oov.source(),
        FormSource::ExplicitMetadataRule { .. }
    ));
    assert!(!oov.trace().is_empty());

    let predicted = adjective_with(
        &AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        },
        AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
        },
    )
    .expect("explicit adjective metadata");
    assert_eq!(predicted.primary_text(), "новꙑи");

    let mut explicit = VerbLexeme::new("рещи", VerbClass::IA1);
    explicit.stems.aorist = Some("рек".to_string());
    explicit.formations.aorist = Some(AoristFormation::New);
    let generated = finite_verb_with(
        &explicit,
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::Third,
            number: Number::Singular,
        },
    )
    .expect("explicit new aorist");
    assert_eq!(generated.primary_text(), "рече");

    explicit.stems.aorist = Some("рѣ".to_string());
    explicit.stems.aorist_second_third_singular = Some("рече".to_string());
    explicit.formations.aorist = Some(AoristFormation::SigmaticSecondary);
    let sigmatic = finite_verb_with(
        &explicit,
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("explicit old sigmatic 2 aorist");
    assert_eq!(sigmatic.primary_text(), "рѣхъ");
    assert!(matches!(
        sigmatic.source(),
        FormSource::ExplicitMetadataRule {
            rule_id: RuleId::VerbAoristSigmaticSecondary
        }
    ));
}

#[test]
fn productive_comparatives_have_complete_typed_public_paradigms() {
    let comparative = productive_new_comparative(&AdjectiveLexeme {
        lemma: "новъ".to_string(),
        class: AdjectiveClass::Hard,
    })
    .expect("productive new comparative");
    let cell = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Feminine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Long,
    };
    let form = comparative_with(&comparative, cell).expect("explicit comparative cell");
    assert_eq!(form.primary_text(), "новѣишиꙗ");
    assert!(matches!(
        form.source(),
        FormSource::ExplicitMetadataRule {
            rule_id: RuleId::AdjectiveComparativeNew
        }
    ));

    let paradigm = comparative_paradigm_with(&comparative);
    assert_eq!(paradigm.lemma(), "новъ");
    assert_eq!(paradigm.syncopated_citation(), "новѣи");
    assert_eq!(paradigm.expanded_citation(), "новѣиши");
    assert_eq!(paradigm.len(), 252);
    assert_eq!(paradigm.successes().count(), 252);
    assert_eq!(paradigm.failures().count(), 0);
    assert_eq!(
        paradigm
            .form(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
            )
            .expect("animate comparative accusative")
            .primary_text(),
        "новѣиша"
    );

    let old = ComparativeLexeme {
        positive_lemma: "грѫбъ".to_string(),
        syncopated_citation: "грѫбл҄ь".to_string(),
        expanded_citation: "грѫбл҄ьши".to_string(),
        formation: ComparativeFormation::Old,
    };
    assert_eq!(
        comparative_with(
            &old,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
            },
        )
        .expect("old comparative")
        .primary_text(),
        "грѫбл҄ии"
    );
}

#[test]
fn ambiguity_unknown_missing_metadata_and_invalid_cells_remain_distinct() {
    let ambiguity = noun("блѧдь", Case::Nominative, Number::Singular)
        .expect_err("fixture has multiple dictionary identities");
    let InflectionError::AmbiguousLexeme { candidates } = ambiguity else {
        panic!("expected ambiguity, got {ambiguity:?}");
    };
    assert!(candidates.len() > 1);
    assert!(candidates.iter().all(|candidate| !candidate.id.is_empty()));

    assert_eq!(
        present("несуществовати", Person::Third, Number::Singular),
        Err(InflectionError::UnknownLemma {
            lemma: "несуществовати".to_string(),
            part_of_speech: PartOfSpeech::Verb,
        })
    );
    assert!(matches!(
        finite(
            "благословити",
            FiniteTense::Aorist,
            Person::First,
            Number::Singular,
        ),
        Err(InflectionError::MissingLexicalMetadata { needed })
            if needed == vec![MetadataField::AoristStem, MetadataField::AoristFormation]
    ));
    let id = only_id("благословити", PartOfSpeech::Verb);
    assert_eq!(
        imperative("благословити", Person::Third, Number::Dual),
        Err(InflectionError::HistoricallyInvalidCell {
            lexeme_id: id.clone(),
            cell: RequestedCell::Imperative(ImperativeCell {
                person: Person::Third,
                number: Number::Dual,
            }),
        })
    );
    assert!(matches!(
        raw_features::form_by_id(&id, "verb:finite:future:1:sg"),
        Err(InflectionError::InvalidInput { .. })
    ));
}

#[test]
fn grammar_all_inventories_are_complete_and_stably_ordered() {
    assert_eq!(
        Case::ALL,
        [
            Case::Nominative,
            Case::Genitive,
            Case::Dative,
            Case::Accusative,
            Case::Instrumental,
            Case::Locative,
            Case::Vocative,
        ]
    );
    assert_eq!(
        Number::ALL,
        [Number::Singular, Number::Dual, Number::Plural]
    );
    assert_eq!(
        Gender::ALL,
        [Gender::Masculine, Gender::Feminine, Gender::Neuter]
    );
    assert_eq!(Animacy::ALL, [Animacy::Animate, Animacy::Inanimate]);
    assert_eq!(Person::ALL, [Person::First, Person::Second, Person::Third]);
    assert_eq!(
        AdjectiveForm::ALL,
        [AdjectiveForm::Short, AdjectiveForm::Long]
    );
    assert_eq!(
        FiniteTense::ALL,
        [
            FiniteTense::Present,
            FiniteTense::Imperfect,
            FiniteTense::Aorist,
        ]
    );
    assert_eq!(
        ParticipleKind::ALL,
        [
            ParticipleKind::PresentActive,
            ParticipleKind::PresentPassive,
            ParticipleKind::PastActive,
            ParticipleKind::PastPassive,
        ]
    );
    assert_eq!(ImperativeCell::SUPPORTED.len(), 6);
    assert!(
        ImperativeCell::SUPPORTED
            .iter()
            .copied()
            .all(ImperativeCell::is_supported)
    );
    assert_eq!(
        Number::ALL
            .into_iter()
            .flat_map(|number| Person::ALL
                .into_iter()
                .map(move |person| ImperativeCell { person, number }))
            .filter(|cell| cell.is_supported())
            .collect::<Vec<_>>(),
        ImperativeCell::SUPPORTED
    );
}

#[test]
fn reviewed_override_follows_exact_table_and_keeps_authority() {
    let id = only_id("бꙑти", PartOfSpeech::Verb);
    let by_id = by_id::finite_by_id(
        &id,
        FiniteVerbCell {
            tense: FiniteTense::Imperfect,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("reviewed suppletive override");
    let by_lemma = imperfect("бꙑти", Person::First, Number::Singular)
        .expect("lemma resolver uses the same override");
    assert_eq!(by_id, by_lemma);
    assert_eq!(by_id.primary_text(), "бѣахъ");
    assert_eq!(by_id.source(), &FormSource::ManualOverride);
    assert_eq!(
        by_id.analyses()[0].evidence[0].provenance,
        MetadataProvenance::CuratedGrammarOverride
    );
    assert!(
        by_id.analyses()[0].evidence[0]
            .authority
            .as_deref()
            .is_some_and(|source| source.contains("UT OCS Online lesson 1"))
    );

    let exact = imperfect("бꙑти", Person::Third, Number::Singular)
        .expect("exact source table precedes overrides");
    assert_eq!(exact.primary_text(), "бѣаше");
    assert_eq!(exact.source(), &FormSource::DictionaryTable);
}

#[test]
fn closed_classes_remain_lossless_in_the_advanced_dictionary_api() {
    let copied_second_person = raw_features::closed_class(
        "азъ",
        PartOfSpeech::Pronoun,
        ClosedClassCell {
            case: Case::Accusative,
            number: Number::Dual,
            gender: None,
            person: Some(Person::Second),
        },
    )
    .expect("raw source table preserves its copied second-person row");
    assert_eq!(copied_second_person.primary_text(), "ва");
    assert_eq!(copied_second_person.source(), &FormSource::DictionaryTable);

    let reflexive = raw_features::closed_class(
        "сѧ",
        PartOfSpeech::Pronoun,
        ClosedClassCell {
            case: Case::Dative,
            number: Number::Dual,
            gender: None,
            person: None,
        },
    )
    .expect("number-invariant reflexive pronoun");
    assert_eq!(reflexive.texts().collect::<Vec<_>>(), ["себѣ", "си"]);

    let numeral = raw_features::closed_class(
        "девѧть",
        PartOfSpeech::Numeral,
        ClosedClassCell {
            case: Case::Genitive,
            number: Number::Singular,
            gender: None,
            person: None,
        },
    )
    .expect("dictionary numeral");
    assert_eq!(numeral.primary_text(), "девѧти");

    let alias = raw_features::closed_class(
        "ⰽⱏⱅⱁ",
        PartOfSpeech::Pronoun,
        ClosedClassCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: None,
            person: None,
        },
    )
    .expect("source-listed Glagolitic alias");
    assert!(alias.warnings().iter().any(|warning| matches!(
        warning,
        InflectionWarning::OrthographicAliasUsed { canonical } if canonical == "къто"
    )));
}

#[test]
fn source_backed_glagolitic_and_hostile_inputs_are_panic_free() {
    let glagolitic_id = old_church_slavonic::lookup("ⱁⰽⱁ", PartOfSpeech::Noun)
        .expect("Glagolitic lookup")
        .into_iter()
        .find(|candidate| candidate.lemma == "ⱁⰽⱁ")
        .expect("source-backed Glagolitic record")
        .id;
    let glagolitic = by_id::noun_by_id(
        &glagolitic_id,
        NounCell {
            case: Case::Genitive,
            number: Number::Singular,
        },
    )
    .expect("source-backed Glagolitic paradigm");
    assert!(glagolitic.primary_text().contains('ⰵ'));

    let decomposed = "и\u{306}";
    let mixed_script = "слоword";
    for hostile in [
        "",
        "two words",
        ".",
        "\0",
        decomposed,
        mixed_script,
        &"x".repeat(4_097),
    ] {
        let result = std::panic::catch_unwind(|| exercise_public_surface(hostile));
        assert!(result.is_ok(), "public API panicked for {hostile:?}");
        assert!(
            noun(hostile, Case::Nominative, Number::Singular).is_err(),
            "hostile input unexpectedly produced a noun for {hostile:?}"
        );
    }
}

fn exercise_public_surface(hostile: &str) {
    let noun_cell = NounCell {
        case: Case::Nominative,
        number: Number::Singular,
    };
    let adjective_cell = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Short,
    };
    let finite_cell = FiniteVerbCell {
        tense: FiniteTense::Present,
        person: Person::First,
        number: Number::Singular,
    };
    let imperative_cell = ImperativeCell {
        person: Person::Second,
        number: Number::Singular,
    };
    let l_cell = LParticipleCell {
        gender: Gender::Masculine,
        number: Number::Singular,
    };
    let participle_cell = ParticipleCell {
        kind: ParticipleKind::PresentActive,
        adjective: adjective_cell,
    };
    let noun_lexeme = NounLexeme {
        lemma: hostile.to_string(),
        class: NounClass::OMasculineHard,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        number_restriction: NumberRestriction::All,
    };
    let adjective_lexeme = AdjectiveLexeme {
        lemma: hostile.to_string(),
        class: AdjectiveClass::Hard,
    };
    let mut verb_lexeme = VerbLexeme::new(hostile, VerbClass::II1);
    verb_lexeme.stems.present = Some(hostile.to_string());
    verb_lexeme.stems.present_first_singular = Some(hostile.to_string());
    verb_lexeme.stems.imperative = Some(hostile.to_string());
    verb_lexeme.formations.imperative = Some(ImperativeFormation::ISeries);

    let _ = old_church_slavonic::lookup(hostile, PartOfSpeech::Noun);
    let _ = noun(hostile, Case::Nominative, Number::Singular);
    let _ = long_adjective(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    let _ = short_adjective(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    let _ = determiner(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    let _ = pronoun(hostile, Case::Nominative, Number::Singular);
    let _ = personal_pronoun(hostile, Case::Nominative, Number::Singular, Person::First);
    let _ = gendered_pronoun(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
    );
    let _ = numeral(hostile, Case::Nominative, Number::Singular);
    let _ = gendered_numeral(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
    );
    let _ = present(hostile, Person::First, Number::Singular);
    let _ = imperfect(hostile, Person::First, Number::Singular);
    let _ = aorist(hostile, Person::First, Number::Singular);
    let _ = finite(
        hostile,
        FiniteTense::Present,
        Person::First,
        Number::Singular,
    );
    let _ = imperative(hostile, Person::Second, Number::Singular);
    let _ = l_participle(hostile, Gender::Masculine, Number::Singular);
    let _ = infinitive(hostile);
    let _ = supine(hostile);
    let _ = old_church_slavonic::verbal_noun(hostile);
    let _ = old_church_slavonic::comparative_citation(hostile);
    let _ = old_church_slavonic::present_active_participle(hostile);
    let _ = old_church_slavonic::present_passive_participle(hostile);
    let _ = old_church_slavonic::past_active_participle(hostile);
    let _ = old_church_slavonic::past_passive_participle(hostile);
    let _ = Noun::resolve(hostile);
    let _ = Adjective::resolve(hostile);
    let _ = Verb::resolve(hostile);
    let _ = Determiner::resolve(hostile);
    let _ = Pronoun::resolve(hostile);
    let _ = Numeral::resolve(hostile);
    let _ = noun_paradigm(hostile);
    let _ = adjective_paradigm(hostile);
    let _ = determiner_paradigm(hostile);
    let _ = old_church_slavonic::pronoun_paradigm(hostile);
    let _ = old_church_slavonic::personal_pronoun_paradigm(hostile);
    let _ = old_church_slavonic::gendered_pronoun_paradigm(hostile);
    let _ = old_church_slavonic::numeral_paradigm(hostile);
    let _ = old_church_slavonic::gendered_numeral_paradigm(hostile);
    let _ = present_paradigm(hostile);
    let _ = finite_paradigm(hostile);
    let _ = imperative_paradigm(hostile);
    let _ = l_participle_paradigm(hostile);
    let _ = participle_paradigm(hostile, ParticipleKind::PresentActive);

    let _ = by_id::noun_by_id(hostile, noun_cell);
    let _ = by_id::adjective_by_id(hostile, adjective_cell);
    let _ = by_id::finite_by_id(hostile, finite_cell);
    let _ = by_id::imperative_by_id(hostile, imperative_cell);
    let _ = by_id::l_participle_by_id(hostile, l_cell);
    let _ = by_id::participle_by_id(hostile, participle_cell);
    let _ = noun_with(&noun_lexeme, noun_cell);
    let _ = adjective_with(&adjective_lexeme, adjective_cell);
    let _ = finite_verb_with(&verb_lexeme, finite_cell);
    let _ = imperative_with(&verb_lexeme, imperative_cell);
    let _ = participle_with(&verb_lexeme, participle_cell);
    let _ = participle_form(hostile, participle_cell);
    let _ = raw_features::dictionary_form_by_id(hostile, hostile);
    let _ = raw_features::dictionary_paradigm_by_id(hostile);
}

#[test]
fn normalized_dictionary_metadata_remains_in_the_specialist_namespace() {
    let id = only_id("благословити", PartOfSpeech::Verb);
    let metadata = api_metadata::verb_metadata_by_id(&id).expect("typed dictionary metadata");
    assert_eq!(metadata.lexeme_id, id);
    assert!(!metadata.present.is_empty());
    assert!(
        metadata.present[0].stem.evidence.field == Some(MetadataField::PresentStem)
            || metadata.present[0].class.evidence.field == Some(MetadataField::VerbClass)
    );

    let generated = api_metadata::finite_verb_from_dictionary_metadata(
        &metadata,
        FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("production resolver accepts validated metadata");
    assert_eq!(generated.primary_text(), "благословлѭ");
    assert!(generated.trace().iter().any(|step| {
        step.rule_id == RuleId::VerbDictionaryMetadata
            || matches!(
                generated.source(),
                FormSource::DictionaryMetadataRule { .. }
            )
    }));
}

#[test]
fn normalized_sigmatic_metadata_reaches_the_production_resolver() {
    let field = |name: &str,
                 value: &str,
                 source_feature: &str,
                 source_form: &str|
     -> api_metadata::NormalizedVerbMetadataField {
        api_metadata::NormalizedVerbMetadataField {
            system: "aorist".to_string(),
            analysis_rank: 0,
            field: name.to_string(),
            value: value.to_string(),
            provenance: "dictionary-principal-part".to_string(),
            source_feature: source_feature.to_string(),
            source_form: source_form.to_string(),
            crosscheck_features: vec!["verb:finite:aorist:3:pl".to_string()],
            authority: "UT OCS Online lesson 3 §14.2; Polivanova 2023 §§476–480".to_string(),
        }
    };
    let metadata = api_metadata::DictionaryVerbMetadata::from_normalized_fields(
        "fixture:рєшти",
        "рєшти",
        [
            field("stem", "рѣ", "verb:finite:aorist:1:sg", "рѣхъ"),
            field(
                "second-third-singular",
                "рєчє",
                "verb:finite:aorist:3:sg",
                "рєчє",
            ),
            field(
                "formation",
                "sigmatic-secondary",
                "verb:finite:aorist:1:sg",
                "рѣхъ",
            ),
        ],
    )
    .expect("validated sigmatic metadata");

    let first_singular = api_metadata::finite_verb_from_dictionary_metadata(
        &metadata,
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("metadata-driven old sigmatic 2 main subbundle");
    assert_eq!(first_singular.primary_text(), "рѣхъ");

    let third_singular = api_metadata::finite_verb_from_dictionary_metadata(
        &metadata,
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::Third,
            number: Number::Singular,
        },
    )
    .expect("metadata-driven independent singular subbundle");
    assert_eq!(third_singular.primary_text(), "рєчє");
    assert!(
        third_singular
            .analyses()
            .iter()
            .flat_map(|analysis| &analysis.evidence)
            .any(|evidence| { evidence.field == Some(MetadataField::AoristSecondThirdSingular) })
    );
}

#[test]
fn normalized_imperfect_variants_reach_the_production_resolver_in_source_order() {
    let field = |analysis_rank: u16,
                 name: &str,
                 value: &str,
                 source_form: &str|
     -> api_metadata::NormalizedVerbMetadataField {
        api_metadata::NormalizedVerbMetadataField {
            system: "imperfect".to_string(),
            analysis_rank,
            field: name.to_string(),
            value: value.to_string(),
            provenance: "dictionary-principal-part".to_string(),
            source_feature: "verb:finite:imperfect:3:sg".to_string(),
            source_form: source_form.to_string(),
            crosscheck_features: vec!["verb:finite:imperfect:1:sg".to_string()],
            authority: "Polivanova 2023 §§455, 467–472 and 914–915".to_string(),
        }
    };
    let metadata = api_metadata::DictionaryVerbMetadata::from_normalized_fields(
        "fixture:нести",
        "нести",
        [
            field(0, "stem", "нес", "несѣше"),
            field(0, "formation", "yat-a", "несѣше"),
            field(0, "variant-policy", "contracted-only", "несѣше"),
            field(1, "stem", "нес", "несѣаше"),
            field(1, "formation", "yat-a", "несѣаше"),
            field(1, "variant-policy", "uncontracted-only", "несѣаше"),
        ],
    )
    .expect("validated source-ordered imperfect metadata");

    let generated = api_metadata::finite_verb_from_dictionary_metadata(
        &metadata,
        FiniteVerbCell {
            tense: FiniteTense::Imperfect,
            person: Person::Third,
            number: Number::Singular,
        },
    )
    .expect("metadata-driven contracted and uncontracted variants");
    assert_eq!(generated.primary_text(), "несѣше");
    assert_eq!(
        generated.texts().collect::<Vec<_>>(),
        vec!["несѣше", "несѣаше"]
    );
    assert_eq!(
        generated.analyses()[0]
            .trace
            .last()
            .expect("productive imperfect step")
            .rule_id,
        RuleId::VerbImperfectContractedYatA
    );
    assert_eq!(
        generated.analyses()[1]
            .trace
            .last()
            .expect("productive imperfect step")
            .rule_id,
        RuleId::VerbImperfectYatA
    );
    assert!(generated.analyses().iter().all(|analysis| {
        analysis
            .evidence
            .iter()
            .any(|evidence| evidence.field == Some(MetadataField::ImperfectVariantPolicy))
    }));
}

#[test]
fn impersonal_predicates_are_typed_without_deleting_word_morphology() {
    assert_eq!(
        ImpersonalVerbIdentity::Dostojati.status(),
        ImpersonalVerbStatus::LexicallyImpersonal
    );
    assert_eq!(
        old_church_slavonic::phrases::impersonal_predicate(
            ImpersonalVerbIdentity::Dostojati,
            FiniteTense::Present,
        )
        .expect("dictionary-backed impersonal predicate")
        .primary_text(),
        "достоитъ"
    );
    assert_eq!(
        old_church_slavonic::phrases::impersonal_predicate(
            ImpersonalVerbIdentity::MnetiReflexive,
            FiniteTense::Present,
        )
        .expect("reflexive impersonal sense")
        .primary_text(),
        "мьнитъ сѧ"
    );
    assert_eq!(
        present("мьнѣти", Person::First, Number::Singular)
            .expect("personal sense remains available")
            .primary_text(),
        "мьнѭ"
    );
}
