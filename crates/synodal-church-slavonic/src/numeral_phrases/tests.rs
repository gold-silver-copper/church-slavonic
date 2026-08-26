use super::*;

fn cardinal_cell(case: Case, gender: Option<Gender>) -> CompoundNumeralCell {
    CompoundNumeralCell {
        case,
        gender,
        animacy: Animacy::Inanimate,
    }
}

fn ordinal_cell(case: Case, gender: Gender) -> NumeralCell {
    NumeralCell {
        kind: NumeralKind::Ordinal,
        case,
        number: Number::Singular,
        gender: Some(gender),
        animacy: Animacy::Inanimate,
    }
}

fn realizes_token_surfaces(analysis: &CardinalPhraseAnalysis, expected: &[&str]) -> bool {
    analysis.tokens.len() == expected.len()
        && analysis
            .tokens
            .iter()
            .zip(expected)
            .all(|(token, expected)| {
                token
                    .forms
                    .variants()
                    .iter()
                    .any(|variant| variant.expanded == *expected)
            })
}

fn realizes_printed_token_surface(analysis: &CardinalPhraseAnalysis, expected: &str) -> bool {
    analysis.tokens.len() == 1
        && analysis.tokens[0]
            .forms
            .variants()
            .iter()
            .any(|variant| variant.printed == expected)
}

#[test]
fn cardinals_cover_simple_teens_tens_hundreds_and_all_named_magnitudes() {
    assert_eq!(
        cardinal(2, cardinal_cell(Case::Nominative, Some(Gender::Masculine)))
            .expect("masculine nominative two")
            .primary_text(),
        "два"
    );
    let twelve = cardinal(12, cardinal_cell(Case::Genitive, Some(Gender::Masculine)))
        .expect("genitive twelve");
    assert!(twelve.analyses().len() >= 5);
    assert!(
        twelve
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text().contains("на"))
    );

    let ninety_three = cardinal(93, cardinal_cell(Case::Genitive, Some(Gender::Masculine)))
        .expect("genitive ninety-three");
    assert!(
        ninety_three
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text().contains(" и "))
    );

    for (value, expected) in [
        (100, "сто"),
        (1_000, "тысѧща"),
        (10_000, "тьма"),
        (100_000, "легеѡнъ"),
        (1_000_000, "леѡдръ"),
    ] {
        assert_eq!(
            cardinal(value, cardinal_cell(Case::Nominative, None))
                .expect("named magnitude")
                .primary_text(),
            expected
        );
    }
}

#[test]
fn cardinal_government_is_case_and_position_typed() {
    let five = cardinal(5, cardinal_cell(Case::Dative, None)).expect("dative five");
    assert_eq!(
        five.government(NumeralNounPosition::Following),
        [
            NumeralGovernment::Agreement {
                number: Number::Plural
            },
            NumeralGovernment::GenitivePlural
        ]
    );
    assert_eq!(
        five.government_evidence()[0].id,
        EvidenceId::from("normative:SYN-NUMERAL-GOVERNMENT-ALYPY-65-67")
    );
    let twelve = cardinal(12, cardinal_cell(Case::Nominative, Some(Gender::Feminine)))
        .expect("nominative twelve");
    assert!(twelve.government(NumeralNounPosition::Following).contains(
        &NumeralGovernment::Agreement {
            number: Number::Dual
        }
    ));
    assert!(
        twelve
            .government(NumeralNounPosition::Following)
            .contains(&NumeralGovernment::GenitivePlural)
    );
    assert_eq!(
        twelve.government(NumeralNounPosition::Preceding),
        [NumeralGovernment::Agreement {
            number: Number::Dual
        }]
    );

    for value in [100, 1_000] {
        let magnitude =
            cardinal(value, cardinal_cell(Case::Nominative, None)).expect("exact magnitude");
        assert!(
            magnitude
                .government(NumeralNounPosition::Preceding)
                .contains(&NumeralGovernment::GenitivePlural)
        );
        assert!(
            !magnitude
                .government(NumeralNounPosition::Preceding)
                .contains(&NumeralGovernment::Agreement {
                    number: Number::Singular
                })
        );
    }
}

#[test]
fn locked_synodal_bible_compound_numerals_are_reproduced() {
    // 1 Chronicles 7:5, locked Wikisource revision 1355550, line 59.
    let fifty_four_thousand_four_hundred =
        cardinal(54_400, cardinal_cell(Case::Nominative, None)).expect("54,400");
    assert!(
        fifty_four_thousand_four_hundred
            .analyses()
            .iter()
            .any(|analysis| realizes_token_surfaces(
                analysis,
                &["пѧтьдесѧтъ", "и", "четыри", "тысѧщы", "и", "четыре", "ста"]
            ))
    );

    // 1 Chronicles 21:25, locked Wikisource revision 1355550, line 78.
    let six_hundred_three_thousand_five_hundred_fifty =
        cardinal(603_550, cardinal_cell(Case::Nominative, None)).expect("603,550");
    assert!(
        six_hundred_three_thousand_five_hundred_fifty
            .analyses()
            .iter()
            .any(|analysis| realizes_token_surfaces(
                analysis,
                &[
                    "шесть",
                    "сотъ",
                    "тысѧщъ",
                    "и",
                    "три",
                    "тысѧщы",
                    "и",
                    "пѧть",
                    "сотъ",
                    "и",
                    "пѧтьдесѧтъ"
                ]
            ))
    );

    // 3 Kingdoms 14:20, locked Wikisource revision 1355056, line 619.
    let twenty_two =
        cardinal(22, cardinal_cell(Case::Nominative, Some(Gender::Neuter))).expect("twenty-two");
    assert!(
        twenty_two
            .analyses()
            .iter()
            .any(|analysis| realizes_token_surfaces(analysis, &["двадесѧть", "два"]))
    );

    // 1 Chronicles 24:17–18, locked revision 1350049, lines 904–951.
    for (value, expected) in [(21, "двадесѧть первый"), (22, "двадесѧть вторый")]
    {
        let realized = ordinal(value, ordinal_cell(Case::Nominative, Gender::Masculine))
            .expect("compound ordinal");
        assert!(
            realized
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text() == expected)
        );
    }
}

#[test]
fn ordinals_cover_both_teen_placements_and_compounds_through_thousand() {
    let thirteenth = ordinal(13, ordinal_cell(Case::Nominative, Gender::Masculine))
        .expect("masculine thirteenth");
    assert_eq!(thirteenth.analyses().len(), 2);
    assert!(
        thirteenth
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text().starts_with("трет"))
    );
    assert!(
        thirteenth
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text().starts_with("тринадесѧт"))
    );

    let one_seventy_second = ordinal(172, ordinal_cell(Case::Accusative, Gender::Neuter))
        .expect("neuter accusative 172nd");
    assert!(
        one_seventy_second
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text().ends_with(" второе"))
    );
    assert!(ordinal(1_000, ordinal_cell(Case::Nominative, Gender::Masculine)).is_ok());
    assert!(matches!(
        ordinal(1_001, ordinal_cell(Case::Nominative, Gender::Masculine)),
        Err(Error::OutOfRange { .. })
    ));
}

#[test]
fn structurally_invalid_gender_and_vocative_fail_typed() {
    assert!(matches!(
        cardinal(21, cardinal_cell(Case::Nominative, None)),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        cardinal(50, cardinal_cell(Case::Nominative, Some(Gender::Masculine))),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
    assert!(matches!(
        cardinal(12, cardinal_cell(Case::Vocative, Some(Gender::Masculine))),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let twenty = cardinal_with(20, cardinal_cell(Case::Nominative, None), liturgical)
        .expect("§§62–64 license fused liturgical decades");
    assert!(
        twenty
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text() == "два́десѧть")
    );
}

#[test]
fn liturgical_fused_cardinals_follow_alypy_accent_rules() {
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let nominative = CompoundNumeralCell {
        case: Case::Nominative,
        gender: None,
        animacy: Animacy::Inanimate,
    };
    for (value, expected) in [
        (30, "три́десѧть"),
        (40, "четы́редесѧть"),
        (50, "пѧтьдесѧ́тъ"),
        (60, "шестьдесѧ́тъ"),
        (70, "се́дмьдесѧтъ"),
        (80, "ѻ҆́смьдесѧтъ"),
        (90, "де́вѧтьдесѧтъ"),
        (200, "двѣ́сти"),
        (300, "три́ста"),
        (400, "четы́реста"),
    ] {
        let realized = cardinal_with(value, nominative, liturgical)
            .unwrap_or_else(|error| panic!("{value}: {error}"));
        assert!(
            realized
                .analyses()
                .iter()
                .any(|analysis| realizes_printed_token_surface(analysis, expected)),
            "{value}: {:?}",
            realized
                .analyses()
                .iter()
                .map(CardinalPhraseAnalysis::primary_text)
                .collect::<Vec<_>>()
        );
    }

    let twelve = cardinal_with(
        12,
        CompoundNumeralCell {
            gender: Some(Gender::Masculine),
            ..nominative
        },
        liturgical,
    )
    .expect("Alypy §§63–64 license fused liturgical teens");
    assert!(
        twelve
            .analyses()
            .iter()
            .any(|analysis| analysis.primary_text() == "двана́десѧть")
    );
    for value in 11..=19 {
        cardinal_with(
            value,
            CompoundNumeralCell {
                gender: (value <= 14).then_some(Gender::Masculine),
                ..nominative
            },
            liturgical,
        )
        .unwrap_or_else(|error| panic!("liturgical teen {value}: {error}"));
    }

    for (value, expected) in [
        (50, "пѧти́десѧтъ"),
        (60, "шести́десѧтъ"),
        (70, "седми́десѧтъ"),
        (80, "ѻ҆сми́десѧтъ"),
        (90, "девѧти́десѧтъ"),
    ] {
        let realized = cardinal_with(
            value,
            CompoundNumeralCell {
                case: Case::Genitive,
                ..nominative
            },
            liturgical,
        )
        .unwrap_or_else(|error| panic!("genitive {value}: {error}"));
        assert!(
            realized
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text() == expected),
            "genitive {value}: {:?}",
            realized
                .analyses()
                .iter()
                .map(CardinalPhraseAnalysis::primary_text)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn distributive_multiplicative_and_fractional_constructions_remain_typed_tokens() {
    let two = cardinal_cell(Case::Nominative, Some(Gender::Masculine));
    let repeated = repeated_distributive(2, two).expect("two by two");
    assert_eq!(repeated[0].primary_text(), "два два");
    assert_eq!(
        repeated[0].construction(),
        AnalyticConstruction::RepeatedDistributive
    );
    assert!(
        repeated[0]
            .tokens()
            .iter()
            .all(|token| token.role == PhraseRole::Numeral)
    );

    let seven_times =
        multiplicative_krat(7, cardinal_cell(Case::Genitive, None)).expect("seven times");
    assert_eq!(seven_times[0].primary_text(), "седми кратъ");
    assert_eq!(
        seven_times[0].tokens().last().map(|token| token.role),
        Some(PhraseRole::MultiplicativeUnit)
    );

    let two_parts =
        fractional_cardinal_parts(2, Case::Nominative, Animacy::Inanimate).expect("two parts");
    assert_eq!(two_parts[0].primary_text(), "двѣ части");
    assert_eq!(
        two_parts[0].tokens().last().map(|token| token.role),
        Some(PhraseRole::FractionNoun)
    );

    let tenth_part = fractional_ordinal_parts(
        10,
        NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        },
    )
    .expect("tenth part");
    assert_eq!(tenth_part[0].primary_text(), "десѧтаѧ часть");

    let half_tenth = fractional_half_tenth_parts(NounCell {
        case: Case::Genitive,
        number: Number::Singular,
        animacy: Animacy::Inanimate,
    })
    .expect("half-tenth part");
    assert_eq!(half_tenth.primary_text(), "полдесѧтыѧ части");
    assert!(matches!(
        half_tenth.tokens()[0].forms.primary().source,
        FormSource::SynodalAttestation { .. }
    ));
    let predicted_half_tenth = fractional_half_tenth_parts(NounCell {
        case: Case::Dative,
        number: Number::Dual,
        animacy: Animacy::Inanimate,
    })
    .expect("productive half-tenth agreement");
    assert!(matches!(
        predicted_half_tenth.tokens()[0].forms.primary().source,
        FormSource::SynodalNormativeGeneration { .. }
    ));

    let two_fifths = fraction(2, 5, Case::Nominative, Animacy::Inanimate).expect("two fifth parts");
    assert_eq!(two_fifths[0].primary_text(), "двѣ пѧтѣи части");
}

#[test]
fn every_compositional_equivalence_class_covers_its_complete_cell_product() {
    let cardinal_values = [
        1, 2, 3, 4, 5, 10, 11, 12, 14, 15, 19, 20, 30, 40, 50, 90, 21, 55, 99, 100, 200, 400, 500,
        900, 101, 111, 114, 115, 120, 121, 999, 1_000, 2_000, 9_000, 10_000, 90_000, 100_000,
        900_000, 1_000_000,
    ];
    for value in cardinal_values {
        let genders: &[Option<Gender>] = if cardinal_requires_gender(value) {
            &[
                Some(Gender::Masculine),
                Some(Gender::Feminine),
                Some(Gender::Neuter),
            ]
        } else {
            &[None]
        };
        for case in Case::ALL.into_iter().filter(|case| *case != Case::Vocative) {
            for &gender in genders {
                for animacy in Animacy::ALL {
                    let realized = cardinal(
                        value,
                        CompoundNumeralCell {
                            case,
                            gender,
                            animacy,
                        },
                    )
                    .unwrap_or_else(|error| {
                        panic!("cardinal {value} {case:?} {gender:?} {animacy:?}: {error}")
                    });
                    assert!(!realized.analyses().is_empty());
                    assert!(realized.analyses().iter().all(|analysis| {
                        !analysis.tokens.is_empty()
                            && analysis
                                .tokens
                                .iter()
                                .all(|token| !token.forms.variants().is_empty())
                    }));
                }
            }
        }
    }

    let ordinal_values = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 60, 70,
        80, 90, 21, 99, 100, 200, 300, 400, 500, 600, 700, 800, 900, 101, 110, 111, 172, 999,
        1_000,
    ];
    for value in ordinal_values {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
                        let cell = NumeralCell {
                            kind: NumeralKind::Ordinal,
                            case,
                            number,
                            gender: Some(gender),
                            animacy,
                        };
                        let realized = ordinal(value, cell).unwrap_or_else(|error| {
                            panic!(
                                "ordinal {value} {case:?} {number:?} {gender:?} {animacy:?}: {error}"
                            )
                        });
                        assert!(!realized.analyses().is_empty());
                        assert!(
                            realized
                                .analyses()
                                .iter()
                                .all(|analysis| !analysis.tokens.is_empty())
                        );
                    }
                }
            }
        }
    }
}
