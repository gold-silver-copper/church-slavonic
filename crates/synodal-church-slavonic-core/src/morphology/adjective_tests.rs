use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, Gender, Number,
    OrthographyProfile,
};

use super::*;

use super::test_support::*;

#[test]
fn alpy_52_short_masculine_stem_formations_are_typed() {
    let blessed = AdjectiveLexeme {
        lemma: word("блаженъ"),
        stem: word("блаженн"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: Some(word("блажен")),
        short_masculine_formation: Some(ShortMasculineStemFormation::DoubleNReduction),
        comparative_stem: None,
        comparison_formation: None,
    };
    let venerable = AdjectiveLexeme {
        lemma: word("преподобенъ"),
        stem: word("преподобн"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: Some(word("преподобен")),
        short_masculine_formation: Some(ShortMasculineStemFormation::MobileEInsertion),
        comparative_stem: None,
        comparison_formation: None,
    };
    let evil = AdjectiveLexeme {
        lemma: word("ѕлый"),
        stem: word("ѕл"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: Some(word("ѕол")),
        short_masculine_formation: Some(ShortMasculineStemFormation::MobileOInsertion),
        comparative_stem: Some(word("ѕлѣйш")),
        comparison_formation: Some(ComparisonFormation::LaterYat),
    };
    for adjective in [&blessed, &venerable, &evil] {
        validate_adjective_lexeme(adjective).expect("typed positive principal part");
    }
    let form = |lexeme: &AdjectiveLexeme, gender, adjective_form| {
        decline_adjective(
            lexeme,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender,
                animacy: Animacy::Inanimate,
                form: adjective_form,
                comparison: Comparison::Positive,
            },
            OrthographyProfile::Expanded,
        )
        .expect("productive positive cell")
        .primary_text()
        .to_owned()
    };
    assert_eq!(
        form(&blessed, Gender::Masculine, AdjectiveForm::Short),
        "блаженъ"
    );
    assert_eq!(
        form(&blessed, Gender::Feminine, AdjectiveForm::Short),
        "блаженна"
    );
    assert_eq!(
        form(&blessed, Gender::Masculine, AdjectiveForm::Long),
        "блаженный"
    );
    assert_eq!(
        form(&venerable, Gender::Masculine, AdjectiveForm::Short),
        "преподобенъ"
    );
    assert_eq!(
        form(&venerable, Gender::Feminine, AdjectiveForm::Short),
        "преподобна"
    );
    assert_eq!(form(&evil, Gender::Masculine, AdjectiveForm::Short), "ѕолъ");
    assert_eq!(form(&evil, Gender::Neuter, AdjectiveForm::Short), "ѕло");
    assert_eq!(form(&evil, Gender::Masculine, AdjectiveForm::Long), "ѕлый");

    let mut contradictory = blessed.clone();
    contradictory.short_masculine_stem = Some(word("блаженн"));
    assert!(matches!(
        validate_adjective_lexeme(&contradictory),
        Err(Error::ContradictoryMetadata { .. })
    ));
    let mut contradictory_mobile_o = evil;
    contradictory_mobile_o.short_masculine_stem = Some(word("ѕел"));
    assert!(matches!(
        validate_adjective_lexeme(&contradictory_mobile_o),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn declines_long_hard_adjective_from_alypy_57() {
    let lexeme = AdjectiveLexeme {
        lemma: word("мꙋдръ"),
        stem: word("мꙋдр"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    let form = decline_adjective(
        &lexeme,
        AdjectiveCell {
            case: Case::Genitive,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Animate,
            form: AdjectiveForm::Long,
            comparison: Comparison::Positive,
        },
        OrthographyProfile::Expanded,
    )
    .expect("supported form");
    assert_eq!(form.primary_text(), "мꙋдрагѡ");
}

#[test]
fn alpy_57_velar_adjective_table_controls_endings_and_palatalization() {
    let good = AdjectiveLexeme {
        lemma: word("благъ"),
        stem: word("благ"),
        class: AdjectiveClass::Velar,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    validate_adjective_lexeme(&good).expect("velar stem");
    let form = |number, gender, case, adjective_form, animacy| {
        decline_adjective(
            &good,
            AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form: adjective_form,
                comparison: Comparison::Positive,
            },
            OrthographyProfile::Expanded,
        )
        .expect("Alypy velar cell")
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect::<Vec<_>>()
    };
    use AdjectiveForm::{Long, Short};
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Locative as Loc, Nominative as Nom,
        Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    for (number, gender, case, adjective_form, expected) in [
        (Sg, M, Nom, Short, &["благъ"][..]),
        (Sg, M, Voc, Short, &["блаже"]),
        (Sg, F, Dat, Short, &["блазѣ"]),
        (Pl, M, Nom, Short, &["блази"]),
        (Pl, F, Nom, Short, &["благи"]),
        (Sg, M, Nom, Long, &["благїй"]),
        (Sg, F, Gen, Long, &["благїѧ"]),
        (Sg, M, Loc, Long, &["блазѣмъ"]),
        (Du, F, Nom, Long, &["блазѣи"]),
        (Pl, M, Nom, Long, &["блазїи"]),
        (Pl, F, Nom, Long, &["благїѧ"]),
        (Pl, N, Nom, Long, &["благаѧ"]),
        (Pl, M, Acc, Long, &["благїѧ", "благихъ"]),
    ] {
        assert_eq!(
            form(
                number,
                gender,
                case,
                adjective_form,
                if (number, gender, case) == (Pl, M, Acc) {
                    Animacy::Animate
                } else {
                    Animacy::Inanimate
                },
            ),
            expected,
            "{number:?} {gender:?} {case:?} {adjective_form:?}"
        );
    }
    assert_eq!(
        form(Sg, M, Acc, Long, Animacy::Animate),
        ["благаго", "благїй"]
    );
    assert_eq!(
        form(Pl, M, Acc, Long, Animacy::Animate),
        ["благїѧ", "благихъ"]
    );

    let mut contradictory = good;
    contradictory.stem = word("мꙋдр");
    assert!(matches!(
        validate_adjective_lexeme(&contradictory),
        Err(Error::ContradictoryMetadata { .. })
    ));
    assert!(matches!(
        decline_adjective(
            &contradictory,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn declines_comparison_stem_with_alypy_58_mixed_endings() {
    let lexeme = AdjectiveLexeme {
        lemma: word("мꙋдръ"),
        stem: word("мꙋдр"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: Some(word("мꙋдрѣйш")),
        comparison_formation: Some(ComparisonFormation::LaterYat),
    };
    let form = |case, number, gender, animacy| {
        decline_adjective(
            &lexeme,
            AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form: AdjectiveForm::Long,
                comparison: Comparison::Comparative,
            },
            OrthographyProfile::Expanded,
        )
        .expect("reviewed comparison stem")
        .primary_text()
        .to_owned()
    };
    assert_eq!(
        form(
            Case::Nominative,
            Number::Singular,
            Gender::Feminine,
            Animacy::Inanimate
        ),
        "мꙋдрѣйшаѧ"
    );
    assert_eq!(
        form(
            Case::Nominative,
            Number::Singular,
            Gender::Neuter,
            Animacy::Inanimate
        ),
        "мꙋдрѣйшее"
    );
    assert_eq!(
        form(
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            Animacy::Animate
        ),
        "мꙋдрѣйшагѡ"
    );
    assert_eq!(
        form(
            Case::Dative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Animate
        ),
        "мꙋдрѣйшемꙋ"
    );
    assert_eq!(
        form(
            Case::Accusative,
            Number::Singular,
            Gender::Feminine,
            Animacy::Inanimate
        ),
        "мꙋдрѣйшꙋю"
    );
    assert_eq!(
        form(
            Case::Genitive,
            Number::Plural,
            Gender::Masculine,
            Animacy::Animate
        ),
        "мꙋдрѣйшихъ"
    );
}

#[test]
fn short_superlative_is_bounded_to_nominative_predicate_agreement() {
    let lexeme = AdjectiveLexeme {
        lemma: word("истиннъ"),
        stem: word("истинн"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: Some(word("истиннѣйш")),
        comparison_formation: Some(ComparisonFormation::LaterYat),
    };
    let expected = [
        (Number::Singular, Gender::Masculine, "истиннѣйшъ|истиннѣй"),
        (Number::Singular, Gender::Feminine, "истиннѣйши"),
        (Number::Singular, Gender::Neuter, "истиннѣе|истиннѣйше"),
        (Number::Dual, Gender::Masculine, "истиннѣйша"),
        (Number::Dual, Gender::Feminine, "истиннѣйши"),
        (Number::Dual, Gender::Neuter, "истиннѣйши"),
        (Number::Plural, Gender::Masculine, "истиннѣйше|истиннѣйши"),
        (Number::Plural, Gender::Feminine, "истиннѣйшѧ"),
        (Number::Plural, Gender::Neuter, "истиннѣйша"),
    ];
    for (number, gender, expected) in expected {
        let forms = decline_adjective(
            &lexeme,
            AdjectiveCell {
                case: Case::Nominative,
                number,
                gender,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Superlative,
            },
            OrthographyProfile::Expanded,
        )
        .expect("Alypy §59 predicate short superlative");
        assert_eq!(
            forms.texts().collect::<Vec<_>>(),
            expected.split('|').collect::<Vec<_>>()
        );
        assert_productive_contract(&forms);
    }

    for number in Number::ALL {
        for gender in Gender::ALL {
            for case in Case::ALL {
                if case == Case::Nominative {
                    continue;
                }
                assert!(matches!(
                    decline_adjective(
                        &lexeme,
                        AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy: Animacy::Inanimate,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Superlative,
                        },
                        OrthographyProfile::Expanded,
                    ),
                    Err(Error::HistoricallyInvalidCell { .. })
                ));
            }
        }
    }
}

#[test]
fn alpy_50_57_possessive_adjective_contracts_are_complete_and_bounded() {
    let bozhii = AdjectiveLexeme {
        lemma: word("божїй"),
        stem: word("бож"),
        class: AdjectiveClass::PossessiveIi,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    let short_goldens = [
        [
            "божїй",
            "божїѧ",
            "божїю",
            "божїѧ",
            "божїимъ",
            "божїи",
            "божїй",
            "божїѧ",
            "божїю",
            "божїима",
            "божїѧ",
            "божїима",
            "божїю",
            "божїѧ",
            "божїи",
            "божїихъ",
            "божїимъ",
            "божїи",
            "божїи",
            "божїихъ",
            "божїи",
        ],
        [
            "божїѧ",
            "божїѧ",
            "божїи",
            "божїю",
            "божїею",
            "божїи",
            "божїѧ",
            "божїи",
            "божїю",
            "божїима",
            "божїи",
            "божїима",
            "божїю",
            "божїи",
            "божїѧ",
            "божїихъ",
            "божїимъ",
            "божїѧ",
            "божїими",
            "божїихъ",
            "божїѧ",
        ],
        [
            "божїе",
            "божїѧ",
            "божїю",
            "божїе",
            "божїимъ",
            "божїи",
            "божїе",
            "божїи",
            "божїю",
            "божїима",
            "божїи",
            "божїима",
            "божїю",
            "божїи",
            "божїѧ",
            "божїихъ",
            "божїимъ",
            "божїѧ",
            "божїи",
            "божїихъ",
            "божїѧ",
        ],
    ];
    for (gender, expected) in Gender::ALL.into_iter().zip(short_goldens) {
        for ((number, case), expected) in Number::ALL
            .into_iter()
            .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
            .zip(expected)
        {
            assert_eq!(
                decline_adjective(
                    &bozhii,
                    AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Animate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("complete Alypy §56 short table")
                .primary_text(),
                expected,
                "{gender:?} {number:?} {case:?}"
            );
        }
    }
    for (case, gender, expected) in [
        (Case::Genitive, Gender::Masculine, "божїѧгѡ"),
        (Case::Dative, Gender::Masculine, "божїемꙋ"),
        (Case::Genitive, Gender::Feminine, "божїей"),
        (Case::Instrumental, Gender::Feminine, "божїею"),
        (Case::Locative, Gender::Neuter, "божїемъ"),
    ] {
        assert_eq!(
            decline_adjective(
                &bozhii,
                AdjectiveCell {
                    case,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy §56 compound possessive")
            .primary_text(),
            expected
        );
    }

    let gospoden = AdjectiveLexeme {
        lemma: word("господень"),
        stem: word("господн"),
        class: AdjectiveClass::PossessiveSoft,
        short_masculine_stem: Some(word("господен")),
        short_masculine_formation: Some(ShortMasculineStemFormation::MobileEInsertion),
        comparative_stem: None,
        comparison_formation: None,
    };
    let israel = AdjectiveLexeme {
        lemma: word("израилевъ"),
        stem: word("израилев"),
        class: AdjectiveClass::PossessiveHard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    for (lexeme, expected) in [(&gospoden, "господень"), (&israel, "израилевъ")] {
        assert_eq!(
            decline_adjective(
                lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("productive short possessive")
            .primary_text(),
            expected
        );
        assert!(matches!(
            decline_adjective(
                lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }
    for lexeme in [&gospoden, &israel] {
        for number in Number::ALL {
            for gender in Gender::ALL {
                for case in Case::ALL {
                    let cell = AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Animate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    };
                    assert_productive_contract(
                        &decline_adjective(lexeme, cell, OrthographyProfile::Expanded)
                            .expect("complete short possessive paradigm"),
                    );
                    assert!(matches!(
                        decline_adjective(
                            lexeme,
                            AdjectiveCell {
                                form: AdjectiveForm::Long,
                                ..cell
                            },
                            OrthographyProfile::Expanded,
                        ),
                        Err(Error::HistoricallyInvalidCell { .. })
                    ));
                }
            }
        }
    }
    for number in Number::ALL {
        for gender in Gender::ALL {
            for case in Case::ALL {
                assert_productive_contract(
                    &decline_adjective(
                        &bozhii,
                        AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy: Animacy::Animate,
                            form: AdjectiveForm::Long,
                            comparison: Comparison::Positive,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("complete compound -їй possessive paradigm"),
                );
            }
        }
    }
    let iudin = AdjectiveLexeme {
        lemma: word("іꙋдинъ"),
        stem: word("іꙋдин"),
        class: AdjectiveClass::PossessiveIn,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    let egipetskii = AdjectiveLexeme {
        lemma: word("єгѵпетскїй"),
        stem: word("єгѵпетск"),
        class: AdjectiveClass::PossessiveSk,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    for (lexeme, form, case, number, gender, expected) in [
        (
            &iudin,
            AdjectiveForm::Short,
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            "іꙋдина",
        ),
        (
            &iudin,
            AdjectiveForm::Long,
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            "іꙋдинагѡ",
        ),
        (
            &egipetskii,
            AdjectiveForm::Short,
            Case::Genitive,
            Number::Singular,
            Gender::Feminine,
            "єгѵпетски",
        ),
        (
            &egipetskii,
            AdjectiveForm::Short,
            Case::Locative,
            Number::Singular,
            Gender::Feminine,
            "єгѵпетстѣ",
        ),
        (
            &egipetskii,
            AdjectiveForm::Long,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            "єгѵпетскїй",
        ),
        (
            &egipetskii,
            AdjectiveForm::Long,
            Case::Genitive,
            Number::Singular,
            Gender::Feminine,
            "єгѵпетскїѧ",
        ),
        (
            &egipetskii,
            AdjectiveForm::Long,
            Case::Locative,
            Number::Singular,
            Gender::Feminine,
            "єгѵпетстѣй",
        ),
        (
            &egipetskii,
            AdjectiveForm::Long,
            Case::Nominative,
            Number::Plural,
            Gender::Masculine,
            "єгѵпетстїи",
        ),
        (
            &egipetskii,
            AdjectiveForm::Long,
            Case::Genitive,
            Number::Plural,
            Gender::Masculine,
            "єгѵпетскихъ",
        ),
    ] {
        assert_eq!(
            decline_adjective(
                lexeme,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("source-licensed -ин-/-ск- cell")
            .primary_text(),
            expected
        );
    }
    for lexeme in [&iudin, &egipetskii] {
        validate_adjective_lexeme(lexeme).expect("typed possessive suffix");
        for form in AdjectiveForm::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    for case in Case::ALL {
                        assert_productive_contract(
                            &decline_adjective(
                                lexeme,
                                AdjectiveCell {
                                    case,
                                    number,
                                    gender,
                                    animacy: Animacy::Animate,
                                    form,
                                    comparison: Comparison::Positive,
                                },
                                OrthographyProfile::Expanded,
                            )
                            .expect("complete short and long possessive paradigms"),
                        );
                    }
                }
            }
        }
        assert!(matches!(
            decline_adjective(
                lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Comparative,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }
    for (class, stem) in [
        (AdjectiveClass::PossessiveIn, "іꙋд"),
        (AdjectiveClass::PossessiveSk, "єгѵпет"),
    ] {
        assert!(matches!(
            validate_adjective_lexeme(&AdjectiveLexeme {
                lemma: word("неправиленъ"),
                stem: word(stem),
                class,
                short_masculine_stem: None,
                short_masculine_formation: None,
                comparative_stem: None,
                comparison_formation: None,
            }),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
    let chelovech = AdjectiveLexeme {
        lemma: word("человѣчь"),
        stem: word("человѣч"),
        class: AdjectiveClass::PossessiveJ,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    for (case, number, gender, expected) in [
        (
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            "человѣчь",
        ),
        (
            Case::Genitive,
            Number::Singular,
            Gender::Masculine,
            "человѣча",
        ),
        (
            Case::Instrumental,
            Number::Singular,
            Gender::Masculine,
            "человѣчимъ",
        ),
        (
            Case::Accusative,
            Number::Singular,
            Gender::Feminine,
            "человѣчꙋ",
        ),
        (
            Case::Nominative,
            Number::Singular,
            Gender::Neuter,
            "человѣчо",
        ),
        (
            Case::Nominative,
            Number::Plural,
            Gender::Masculine,
            "человѣчи",
        ),
        (
            Case::Nominative,
            Number::Plural,
            Gender::Feminine,
            "человѣчы",
        ),
        (Case::Genitive, Number::Plural, Gender::Neuter, "человѣчихъ"),
        (
            Case::Instrumental,
            Number::Plural,
            Gender::Feminine,
            "человѣчими",
        ),
    ] {
        assert_eq!(
            decline_adjective(
                &chelovech,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("source-licensed mixed -jь possessive cell")
            .primary_text(),
            expected
        );
    }
    for number in Number::ALL {
        for gender in Gender::ALL {
            for case in Case::ALL {
                assert_productive_contract(
                    &decline_adjective(
                        &chelovech,
                        AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy: Animacy::Animate,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Positive,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("complete mixed -jь possessive paradigm"),
                );
            }
        }
    }
    assert!(matches!(
        decline_adjective(
            &chelovech,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        validate_adjective_lexeme(&AdjectiveLexeme {
            lemma: word("неправильнь"),
            stem: word("человѣч"),
            class: AdjectiveClass::PossessiveJ,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: None,
            comparison_formation: None,
        }),
        Err(Error::ContradictoryMetadata { .. })
    ));
    assert!(matches!(
        decline_adjective(
            &bozhii,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Comparative,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}
