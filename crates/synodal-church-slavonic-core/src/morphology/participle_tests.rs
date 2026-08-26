use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FormSource,
    Gender, MetadataField, Number, OrthographyProfile, ParticipleCell, ParticipleTense,
    ParticipleVoice, Recension, VerbSystem,
};

use super::*;

use super::test_support::*;

#[test]
fn declines_independently_specified_participle_stems() {
    let lexeme = regular_verb();
    let short = decline_participle(
        &lexeme,
        ParticipleCell {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            agreement: AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            },
        },
        OrthographyProfile::Expanded,
    )
    .expect("reviewed participial principal part");
    assert_eq!(short.primary_text(), "несꙋщаѧ");

    let long = decline_participle(
        &lexeme,
        ParticipleCell {
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
        },
        OrthographyProfile::Expanded,
    )
    .expect("separate full-form stem");
    assert_eq!(long.primary_text(), "несенный");
}

#[test]
fn sibilant_long_participles_follow_the_synodal_mixed_series() {
    let lexeme = regular_verb();
    let long = |case, number, gender| {
        decline_participle(
            &lexeme,
            ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            },
            OrthographyProfile::Expanded,
        )
        .expect("hard sibilant participle stem")
        .primary_text()
        .to_owned()
    };
    // Genitive/locative and instrumental plural are never spelled with ы
    // after a sibilant; the plural dative and feminine keep ы so they stay
    // distinct from the singular instrumental and genitive.
    assert_eq!(
        long(Case::Genitive, Number::Plural, Gender::Masculine),
        "несꙋщихъ"
    );
    assert_eq!(
        long(Case::Instrumental, Number::Plural, Gender::Masculine),
        "несꙋщими"
    );
    assert_eq!(
        long(Case::Dative, Number::Plural, Gender::Masculine),
        "несꙋщымъ"
    );
    assert_eq!(
        long(Case::Nominative, Number::Plural, Gender::Feminine),
        "несꙋщыѧ"
    );
    assert_eq!(
        long(Case::Accusative, Number::Plural, Gender::Feminine),
        "несꙋщыѧ"
    );
    assert_eq!(
        long(Case::Instrumental, Number::Singular, Gender::Masculine),
        "несꙋщимъ"
    );
    assert_eq!(
        long(Case::Genitive, Number::Singular, Gender::Feminine),
        "несꙋщїѧ"
    );
    assert_eq!(
        long(Case::Dative, Number::Singular, Gender::Masculine),
        "несꙋщемꙋ"
    );
    assert_eq!(
        long(Case::Genitive, Number::Singular, Gender::Masculine),
        "несꙋщагѡ"
    );
    // Alypy §95: the masculine nominative singular contracts to -ый; the
    // uncontracted adjectival print remains as a later variant.
    assert_eq!(
        long(Case::Nominative, Number::Singular, Gender::Masculine),
        "несый"
    );
}

#[test]
fn rejects_comparison_for_participles() {
    let cell = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Long,
        comparison: Comparison::Comparative,
    };
    assert!(matches!(
        decline_participle(
            &regular_verb(),
            ParticipleCell {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Active,
                agreement: cell,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn missing_principal_part_diagnostics_include_typed_formations() {
    let mut verb = regular_verb();
    verb.imperfect_formation = None;
    assert_eq!(
        verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Imperfect)),
        vec![MetadataField::ImperfectFormation]
    );
    verb.aorist_formation = None;
    assert_eq!(
        verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Aorist)),
        vec![MetadataField::AoristFormation]
    );
    verb.imperative_formation = None;
    assert_eq!(
        verb.missing_principal_parts(VerbSystem::Imperative),
        vec![MetadataField::ImperativeFormation]
    );
    verb.present_active_participle
        .as_mut()
        .expect("test principal part")
        .short_formation = None;
    assert_eq!(
        verb.missing_principal_parts(VerbSystem::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        }),
        vec![MetadataField::ParticipleFormation]
    );
}

#[test]
fn alpy_60_complete_short_comparison_golden() {
    struct Row {
        number: Number,
        gender: Gender,
        cells: [&'static str; 7],
    }

    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    // Case order is nominative, genitive, dative, accusative,
    // instrumental, locative, vocative. `@` denotes the typed §58
    // citation edge; the other entries are §60 suffixes.
    let rows = [
        Row {
            number: Sg,
            gender: M,
            cells: ["@", "а", "ꙋ", "ъ", "имъ", "и", "@"],
        },
        Row {
            number: Sg,
            gender: F,
            cells: ["@", "и", "и", "ꙋ", "ею", "и", "@"],
        },
        Row {
            number: Sg,
            gender: N,
            cells: ["@", "а", "ꙋ", "@", "имъ", "и", "@"],
        },
        Row {
            number: Du,
            gender: M,
            cells: ["а", "ꙋ", "има", "а", "има", "ꙋ", "а"],
        },
        Row {
            number: Du,
            gender: F,
            cells: ["и", "ꙋ", "има", "и", "има", "ꙋ", "и"],
        },
        Row {
            number: Du,
            gender: N,
            cells: ["и", "ꙋ", "има", "и", "има", "ꙋ", "и"],
        },
        Row {
            number: Pl,
            gender: M,
            cells: ["е|и", "ихъ", "ымъ", "ѧ", "ими", "ихъ", "е|и"],
        },
        Row {
            number: Pl,
            gender: F,
            cells: ["ѧ", "ихъ", "ымъ", "ѧ", "ими", "ихъ", "ѧ"],
        },
        Row {
            number: Pl,
            gender: N,
            cells: ["а", "ихъ", "ымъ", "а", "ими", "ихъ", "а"],
        },
    ];
    let lexeme = AdjectiveLexeme {
        lemma: word("мꙋдръ"),
        stem: word("мꙋдр"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: Some(word("мꙋдрѣйш")),
        comparison_formation: Some(ComparisonFormation::LaterYat),
    };

    for row in rows {
        for (case, cell_golden) in Case::ALL.into_iter().zip(row.cells) {
            for animacy in if case == Case::Accusative {
                Animacy::ALL.as_slice()
            } else {
                &[Animacy::Inanimate]
            } {
                let forms = decline_adjective(
                    &lexeme,
                    AdjectiveCell {
                        case,
                        number: row.number,
                        gender: row.gender,
                        animacy: *animacy,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Comparative,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("complete Alypy §60 short-comparison cell");
                let mut expected = if cell_golden == "@" {
                    match row.gender {
                        M => vec!["мꙋдрѣй".to_owned()],
                        F => vec!["мꙋдрѣйши".to_owned()],
                        N => vec!["мꙋдрѣе".to_owned(), "мꙋдрѣйше".to_owned()],
                    }
                } else {
                    cell_golden
                        .split('|')
                        .map(|suffix| format!("мꙋдрѣйш{suffix}"))
                        .collect::<Vec<_>>()
                };
                if case == Case::Accusative
                    && row.number == Sg
                    && row.gender == M
                    && *animacy == Animacy::Animate
                {
                    expected.push("мꙋдрѣйша".to_owned());
                }
                assert_eq!(
                    forms.texts().collect::<Vec<_>>(),
                    expected.iter().map(String::as_str).collect::<Vec<_>>()
                );
                assert!(forms.variants().iter().all(|variant| {
                    matches!(
                        &variant.source,
                        FormSource::SynodalNormativeGeneration { rule }
                            if rule.as_ref() == "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60"
                    ) && variant
                        .evidence
                        .iter()
                        .all(|evidence| evidence.citation.contains("§§58 and 60"))
                }));
            }
        }
    }
}

#[test]
fn alpy_98_complete_short_active_participle_goldens() {
    #[derive(Clone, Copy)]
    struct Golden {
        number: Number,
        gender: Gender,
        case: Case,
        variants: &'static [&'static str],
        animate_variants: Option<&'static [&'static str]>,
    }

    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    // Alypy §98's complete short-active-participle table, represented as
    // suffixes after the independently supplied participle stem. The three
    // singular nominative citation edges are checked separately.
    let goldens = [
        Golden {
            number: Sg,
            gender: M,
            case: Nom,
            variants: &[],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: M,
            case: Gen,
            variants: &["а"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: M,
            case: Dat,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: M,
            case: Acc,
            variants: &["ъ"],
            animate_variants: Some(&["ъ", "а"]),
        },
        Golden {
            number: Sg,
            gender: M,
            case: Ins,
            variants: &["имъ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: M,
            case: Loc,
            variants: &["емъ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Nom,
            variants: &[],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Gen,
            variants: &["и"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Dat,
            variants: &["и"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Acc,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Ins,
            variants: &["ею"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: F,
            case: Loc,
            variants: &["и"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Nom,
            variants: &[],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Gen,
            variants: &["а"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Dat,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Acc,
            variants: &["е"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Ins,
            variants: &["имъ"],
            animate_variants: None,
        },
        Golden {
            number: Sg,
            gender: N,
            case: Loc,
            variants: &["емъ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: M,
            case: Nom,
            variants: &["а"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: M,
            case: Gen,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: M,
            case: Dat,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: M,
            case: Acc,
            variants: &["а"],
            animate_variants: Some(&["а", "ꙋ"]),
        },
        Golden {
            number: Du,
            gender: M,
            case: Ins,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: M,
            case: Loc,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Nom,
            variants: &["ѣ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Gen,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Dat,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Acc,
            variants: &["ѣ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Ins,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: F,
            case: Loc,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Nom,
            variants: &["ѣ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Gen,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Dat,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Acc,
            variants: &["ѣ"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Ins,
            variants: &["ема"],
            animate_variants: None,
        },
        Golden {
            number: Du,
            gender: N,
            case: Loc,
            variants: &["ꙋ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: M,
            case: Nom,
            variants: &["е"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: M,
            case: Gen,
            variants: &["ихъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: M,
            case: Dat,
            variants: &["ымъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: M,
            case: Acc,
            variants: &["ѧ"],
            animate_variants: Some(&["ѧ", "ихъ"]),
        },
        Golden {
            number: Pl,
            gender: M,
            case: Ins,
            variants: &["ими"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: M,
            case: Loc,
            variants: &["ихъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: F,
            case: Nom,
            variants: &["ѧ", "е"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: F,
            case: Gen,
            variants: &["ихъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: F,
            case: Dat,
            variants: &["ымъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: F,
            case: Acc,
            variants: &["ѧ"],
            animate_variants: Some(&["ѧ", "ихъ"]),
        },
        Golden {
            number: Pl,
            gender: F,
            case: Ins,
            variants: &["ими"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: F,
            case: Loc,
            variants: &["ихъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Nom,
            variants: &["а"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Gen,
            variants: &["ихъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Dat,
            variants: &["ымъ"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Acc,
            variants: &["а"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Ins,
            variants: &["ими"],
            animate_variants: None,
        },
        Golden {
            number: Pl,
            gender: N,
            case: Loc,
            variants: &["ихъ"],
            animate_variants: None,
        },
    ];

    let verb = regular_verb();

    for golden in goldens {
        for animacy in if golden.case == Acc {
            Animacy::ALL.as_slice()
        } else {
            &[Animacy::Inanimate]
        } {
            let adjective_cell = AdjectiveCell {
                case: golden.case,
                number: golden.number,
                gender: golden.gender,
                animacy: *animacy,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            };
            let suffixes = if *animacy == Animacy::Animate {
                golden.animate_variants.unwrap_or(golden.variants)
            } else {
                golden.variants
            };
            for (tense, stem, citation) in [
                (
                    ParticipleTense::Present,
                    "несꙋщ",
                    ["несый|несꙋщь", "несꙋщи", "несый|несꙋще|несꙋщо"],
                ),
                (
                    ParticipleTense::Past,
                    "несш",
                    ["несъ|несшъ", "несши", "несъ|несше|несшо"],
                ),
            ] {
                let forms = decline_participle(
                    &verb,
                    ParticipleCell {
                        tense,
                        voice: ParticipleVoice::Active,
                        agreement: adjective_cell,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("Alypy §§95–96, 98 active-participle cell");
                let expected = if golden.number == Sg && golden.case == Nom {
                    citation[match golden.gender {
                        M => 0,
                        F => 1,
                        N => 2,
                    }]
                    .split('|')
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
                } else {
                    suffixes
                        .iter()
                        .map(|suffix| format!("{stem}{suffix}"))
                        .collect()
                };
                assert_eq!(
                    forms.texts().collect::<Vec<_>>(),
                    expected.iter().map(String::as_str).collect::<Vec<_>>()
                );
                assert!(forms.variants().iter().all(|variant| {
                    variant.target_recension == Recension::SynodalRussian
                        && !variant.evidence.is_empty()
                        && variant
                            .evidence
                            .iter()
                            .all(|evidence| evidence.citation.contains("Alypy"))
                        && !variant.rule_trace.steps().is_empty()
                }));
            }
        }
    }

    for number in Number::ALL {
        for gender in Gender::ALL {
            let agreement = AdjectiveCell {
                case: Case::Vocative,
                number,
                gender,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            };
            for tense in ParticipleTense::ALL {
                assert!(matches!(
                    decline_participle(
                        &verb,
                        ParticipleCell {
                            tense,
                            voice: ParticipleVoice::Active,
                            agreement,
                        },
                        OrthographyProfile::Expanded
                    ),
                    Err(Error::HistoricallyInvalidCell { .. })
                ));
            }
        }
    }
}

#[test]
fn active_participle_citation_formation_seams() {
    struct Citation<'a> {
        tense: ParticipleTense,
        formation: ActiveParticipleShortFormation,
        stem: &'a str,
        masculine: &'a [&'a str],
        feminine: &'a [&'a str],
        neuter: &'a [&'a str],
    }
    let citations = [
        Citation {
            tense: ParticipleTense::Present,
            formation: ActiveParticipleShortFormation::PresentFirstPalatalized,
            stem: "дѣлающ",
            masculine: &["дѣлаѧ", "дѣлающь"],
            feminine: &["дѣлающи"],
            neuter: &["дѣлаѧ", "дѣлающе", "дѣлающо"],
        },
        Citation {
            tense: ParticipleTense::Present,
            formation: ActiveParticipleShortFormation::PresentSecond,
            stem: "молѧщ",
            masculine: &["молѧ", "молѧщь"],
            feminine: &["молѧщи"],
            neuter: &["молѧ", "молѧще", "молѧщо"],
        },
        Citation {
            tense: ParticipleTense::Present,
            formation: ActiveParticipleShortFormation::PresentAfterSibilant,
            stem: "молчащ",
            masculine: &["молча", "молчѧ", "молчащь"],
            feminine: &["молчащи"],
            neuter: &["молча", "молчѧ", "молчаще", "молчащо"],
        },
        Citation {
            tense: ParticipleTense::Past,
            formation: ActiveParticipleShortFormation::PastVowel,
            stem: "дѣлавш",
            masculine: &["дѣлавъ", "дѣлавшъ"],
            feminine: &["дѣлавши"],
            neuter: &["дѣлавъ", "дѣлавше", "дѣлавшо"],
        },
        Citation {
            tense: ParticipleTense::Past,
            formation: ActiveParticipleShortFormation::PastIotated,
            stem: "сотворьш",
            masculine: &["сотворь"],
            feminine: &["сотворьши"],
            neuter: &["сотворь", "сотворьше"],
        },
    ];

    for citation in citations {
        let part = ParticiplePrincipalPart {
            short_stem: Some(word(citation.stem)),
            short_formation: Some(citation.formation),
            long_stem: None,
            class: AdjectiveClass::Hard,
        };
        let mut verb = regular_verb();
        match citation.tense {
            ParticipleTense::Present => verb.present_active_participle = Some(part),
            ParticipleTense::Past => verb.past_active_participle = Some(part),
        }
        for (gender, expected) in [
            (Gender::Masculine, citation.masculine),
            (Gender::Feminine, citation.feminine),
            (Gender::Neuter, citation.neuter),
        ] {
            let forms = decline_participle(
                &verb,
                ParticipleCell {
                    tense: citation.tense,
                    voice: ParticipleVoice::Active,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                },
                OrthographyProfile::Expanded,
            )
            .expect("source-backed citation edge");
            assert_eq!(forms.texts().collect::<Vec<_>>(), expected);
        }
    }
}

#[test]
fn comparison_citation_formation_seams() {
    struct Citation<'a> {
        formation: ComparisonFormation,
        stem: &'a str,
        masculine: &'a [&'a str],
        feminine: &'a [&'a str],
        neuter: &'a [&'a str],
    }
    let citations = [
        Citation {
            formation: ComparisonFormation::AncientHard,
            stem: "вышш",
            masculine: &["вышїй"],
            feminine: &["вышши"],
            neuter: &["выше", "вышше"],
        },
        Citation {
            formation: ComparisonFormation::AncientSoft,
            stem: "глꙋбльш",
            masculine: &["глꙋблїй"],
            feminine: &["глꙋбльши"],
            neuter: &["глꙋбле", "глꙋбльше"],
        },
        Citation {
            formation: ComparisonFormation::LaterAi,
            stem: "высочайш",
            masculine: &["высочай"],
            feminine: &["высочайши"],
            neuter: &["высочае", "высочайше"],
        },
    ];

    for citation in citations {
        let adjective = AdjectiveLexeme {
            lemma: word("высокъ"),
            stem: word("высок"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: Some(word(citation.stem)),
            comparison_formation: Some(citation.formation),
        };
        for (gender, expected) in [
            (Gender::Masculine, citation.masculine),
            (Gender::Feminine, citation.feminine),
            (Gender::Neuter, citation.neuter),
        ] {
            let forms = decline_adjective(
                &adjective,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Comparative,
                },
                OrthographyProfile::Expanded,
            )
            .expect("source-backed comparison citation edge");
            assert_eq!(forms.texts().collect::<Vec<_>>(), expected);
        }
    }

    let contradictory = AdjectiveLexeme {
        lemma: word("высокъ"),
        stem: word("высок"),
        class: AdjectiveClass::Hard,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: Some(word("высочайш")),
        comparison_formation: Some(ComparisonFormation::AncientSoft),
    };
    assert!(matches!(
        decline_adjective(
            &contradictory,
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
        Err(Error::ContradictoryMetadata { .. })
    ));
}
