#[allow(unused_imports)]
use crate::*;
use synodal_church_slavonic::Case;

#[test]
fn v16_productive_nominal_families_round_trip_through_the_reverse_index() {
    for (surface, id, expected_cell) in [
        (
            "беззако́нїємъ",
            "synodal:noun:bezzakonie",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Dative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "є҆гѵ́птомъ",
            "synodal:proper-noun:egipet",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Dative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "є҆гѵ́петстїи",
            "synodal:adjective:egipetskii",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "є҆гѵ́петскихъ",
            "synodal:adjective:egipetskii",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "і҆ꙋ́дина",
            "synodal:adjective:iudin",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == id
                    && analysis.cell == Some(expected_cell)
                    && analysis.source == AnalysisSource::SynodalProductiveRule
            }),
            "missing productive reverse analysis for {surface}: {analyses:?}"
        );
    }

    for id in ["synodal:adjective:egipetskii", "synodal:adjective:iudin"] {
        let cells = analysis_cells_by_id(&LexemeId::from(id), Inflector::default())
            .expect("typed positive adjective inventory");
        assert!(cells.iter().any(|cell| matches!(
            cell,
            GrammarCell::Adjective(AdjectiveCell {
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
                ..
            })
        )));
        assert!(cells.iter().any(|cell| matches!(
            cell,
            GrammarCell::Adjective(AdjectiveCell {
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
                ..
            })
        )));
        assert!(!cells.iter().any(|cell| matches!(
            cell,
            GrammarCell::Adjective(AdjectiveCell {
                comparison: Comparison::Comparative | Comparison::Superlative,
                ..
            })
        )));
    }
}

#[test]
fn v17_productive_nominal_families_round_trip_through_the_reverse_index() {
    for (surface, id, expected_cell) in [
        (
            "человѣ́чими",
            "synodal:adjective:chelovech",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Instrumental,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "человѣ́ческими",
            "synodal:adjective:chelovecheskii",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Instrumental,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "і҆ѡ́сифомъ",
            "synodal:proper-noun:iosif",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Animate,
            }),
        ),
        (
            "і҆ѡ́сифова",
            "synodal:adjective:iosifov",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "і҆ѻрда́номъ",
            "synodal:proper-noun:iordan",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "і҆ѻрда́нскихъ",
            "synodal:adjective:iordanskii",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "леѵі́тꙋ",
            "synodal:noun:levit",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Dative,
                number: Number::Singular,
                animacy: Animacy::Animate,
            }),
        ),
        (
            "леѵі́тскихъ",
            "synodal:adjective:levitskii",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == id
                    && analysis.cell == Some(expected_cell)
                    && analysis.source == AnalysisSource::SynodalProductiveRule
            }),
            "missing productive reverse analysis for {surface}: {analyses:?}"
        );
    }

    let vne = analyze("внѣ̀").expect("reviewed invariant adverb");
    assert!(vne.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:adverb:vne"
            && analysis.cell == Some(GrammarCell::Indeclinable)
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let human_cells = analysis_cells_by_id(
        &LexemeId::from("synodal:adjective:chelovech"),
        Inflector::default(),
    )
    .expect("typed historical -jь adjective inventory");
    assert!(human_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
            ..
        })
    )));
    assert!(!human_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            form: AdjectiveForm::Long,
            ..
        })
    )));
}

#[test]
fn ottudu_is_an_exact_mark_sensitive_indeclinable_adverb() {
    let marked = analyze("ѿтꙋ́дꙋ").expect("reviewed pronominal adverb");
    assert!(marked.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:adverb:ottudu"
            && analysis.cell == Some(GrammarCell::Indeclinable)
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let wrongly_accented = analyze("ѿтꙋдꙋ́").expect("orthographically valid negative control");
    assert!(
        wrongly_accented
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:adverb:ottudu")
    );
}

#[test]
fn dokole_is_an_exact_mark_sensitive_interrogative_adverb() {
    let marked = analyze("доко́лѣ").expect("reviewed interrogative temporal adverb");
    assert!(marked.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:adverb:dokole"
            && analysis.cell == Some(GrammarCell::Indeclinable)
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let wrongly_accented = analyze("доколѣ́").expect("orthographically valid negative control");
    assert!(
        wrongly_accented
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:adverb:dokole")
    );
}

#[test]
fn o_interjection_is_exact_and_distinct_from_the_o_preposition() {
    let interjection = analyze("ѽ").expect("reviewed exact interjection");
    assert!(interjection.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:interjection:o"
            && analysis.cell == Some(GrammarCell::Indeclinable)
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let preposition = analyze("ѡ҆").expect("reviewed exact preposition");
    assert!(
        preposition
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:interjection:o")
    );
}

#[test]
fn skonchanie_reverse_index_preserves_exact_and_productive_cells() {
    for (surface, case, number, source) in [
        (
            "сконча́нїи",
            Case::Locative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "сконча́нїємъ",
            Case::Dative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
    ] {
        let analyses = analyze(surface).expect("valid completion-noun form");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:noun:skonchanie"
                && analysis.cell
                    == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                        case,
                        number,
                        animacy: Animacy::Inanimate,
                    }))
                && analysis.source == source
        }));
    }
}

#[test]
fn reviewed_v21_soft_ie_nouns_preserve_exact_and_productive_reverse_cells() {
    for (lexeme_id, exact_surface, exact_case, productive_surface) in [
        (
            "synodal:noun:videnie",
            "видѣ́нїи",
            Case::Locative,
            "видѣ́нїємъ",
        ),
        (
            "synodal:noun:spasenie",
            "спасе́нїи",
            Case::Locative,
            "спасе́нїємъ",
        ),
        (
            "synodal:noun:ponoshenie",
            "поноше́нїи",
            Case::Locative,
            "поноше́нїємъ",
        ),
        (
            "synodal:noun:otmshchenie",
            "ѿмще́нїи",
            Case::Locative,
            "ѿмще́нїємъ",
        ),
    ] {
        let exact = analyze(exact_surface).expect("reviewed exact soft -їе form");
        assert!(exact.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell
                    == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                        case: exact_case,
                        number: Number::Singular,
                        animacy: Animacy::Inanimate,
                    }))
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));

        let productive =
            analyze(productive_surface).expect("reviewed productive soft -їе form");
        assert!(productive.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell
                    == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                        case: Case::Dative,
                        number: Number::Plural,
                        animacy: Animacy::Inanimate,
                    }))
                && analysis.source == AnalysisSource::SynodalProductiveRule
        }));
    }
}

#[test]
fn knyaz_reverse_index_preserves_exact_variants_and_productive_cells() {
    let lexeme_id = "synodal:noun:v07-345d6105fdd39fce";
    for (surface, case, number, source) in [
        (
            "кнѧзе́й",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧзє́мъ",
            Case::Dative,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧ́зїе",
            Case::Nominative,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧ́земъ",
            Case::Instrumental,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "кнѧ́зи",
            Case::Locative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧ̑зь",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧзе́хъ",
            Case::Locative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "кнѧ́зѣ",
            Case::Locative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "кнѧ̑зѧ",
            Case::Accusative,
            Number::Dual,
            AnalysisSource::ExactSynodalAttestation,
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == lexeme_id
                    && analysis.cell
                        == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                            case,
                            number,
                            animacy: Animacy::Animate,
                        }))
                    && analysis.source == source
            }),
            "missing {source:?} {case:?} {number:?} analysis for {surface}: {analyses:?}"
        );
    }

    for surface in ["кнѧ́зь", "кнѧ́зѧ", "кнѧ́земъ"] {
        let analyses = analyze(surface).expect("reviewed animate prince analysis");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && matches!(
                    analysis.cell,
                    Some(GrammarCell::Noun(cell)) if cell.animacy == Animacy::Animate
                )
        }));
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && matches!(
                    analysis.cell,
                    Some(GrammarCell::Noun(cell)) if cell.animacy == Animacy::Inanimate
                )
        }));
    }

    let wrong_accent = analyze("кнѧ́зей").expect("valid but wrongly accented surface");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != lexeme_id)
    );
}

#[test]
fn zhrets_reverse_index_preserves_exact_variants_and_productive_cells() {
    let lexeme_id = "synodal:noun:v11-332e30b022aa";
    for (surface, case, number, source) in [
        (
            "жре́цъ",
            Case::Nominative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "жерцꙋ̀",
            Case::Dative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "жерце́мъ",
            Case::Instrumental,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "жерцє́мъ",
            Case::Dative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "жерцє́въ",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "жрє́цъ",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "жерцѣ́хъ",
            Case::Locative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "жерца́ми",
            Case::Instrumental,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == lexeme_id
                    && analysis.cell
                        == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                            case,
                            number,
                            animacy: Animacy::Animate,
                        }))
                    && analysis.source == source
            }),
            "missing {source:?} {case:?} {number:?} analysis for {surface}: {analyses:?}"
        );
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && matches!(
                    analysis.cell,
                    Some(GrammarCell::Noun(cell)) if cell.animacy == Animacy::Inanimate
                )
        }));
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));
    }

    let wrong_accent = analyze("же́рцꙋ").expect("valid but wrongly accented surface");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != lexeme_id)
    );
}

#[test]
fn prestol_reverse_index_preserves_exact_wide_omega_and_inanimate_productivity() {
    let lexeme_id = "synodal:noun:prestol";
    for (surface, case, number, source) in [
        (
            "престо́лъ",
            Case::Nominative,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "престо́ломъ",
            Case::Instrumental,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "престо́лы",
            Case::Accusative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "престо́лѡвъ",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::ExactSynodalAttestation,
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == lexeme_id
                    && analysis.cell
                        == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                            case,
                            number,
                            animacy: Animacy::Inanimate,
                        }))
                    && analysis.source == source
            }),
            "missing {source:?} {case:?} {number:?} analysis for {surface}: {analyses:?}"
        );
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && matches!(
                    analysis.cell,
                    Some(GrammarCell::Noun(cell)) if cell.animacy == Animacy::Animate
                )
        }));
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));
    }

    let wrong_accent = analyze("пре́столомъ").expect("valid but wrongly accented surface");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != lexeme_id)
    );
}

#[test]
fn otrocha_reverse_index_preserves_exact_short_and_productive_extended_cells() {
    let lexeme_id = "synodal:noun:otrocha";
    for (surface, case, number, source) in [
        (
            "ѻ҆троча̀",
            Case::Nominative,
            Number::Singular,
            AnalysisSource::SynodalNormativeTable,
        ),
        (
            "Ѻ҆троча́",
            Case::Nominative,
            Number::Singular,
            AnalysisSource::ExactSynodalAttestation,
        ),
        (
            "ѻ҆троча́те",
            Case::Genitive,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "ѻ҆троча́ти",
            Case::Dative,
            Number::Singular,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "ѻ҆троча́та",
            Case::Nominative,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
        (
            "ѻ҆троча́тъ",
            Case::Genitive,
            Number::Plural,
            AnalysisSource::SynodalProductiveRule,
        ),
    ] {
        let analyses = analyze(surface).unwrap_or_else(|error| panic!("{surface}: {error}"));
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == lexeme_id
                    && analysis.cell
                        == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                            case,
                            number,
                            animacy: Animacy::Inanimate,
                        }))
                    && analysis.source == source
            }),
            "missing {source:?} {case:?} {number:?} analysis for {surface}: {analyses:?}"
        );
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && matches!(
                    analysis.cell,
                    Some(GrammarCell::Noun(cell)) if cell.animacy == Animacy::Animate
                )
        }));
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));
    }

    let wrong_accent = analyze("ѻ҆тро́чати").expect("valid but wrongly accented surface");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != lexeme_id)
    );
}

#[test]
fn edin_reverse_index_preserves_exact_witnesses_and_singular_only_productivity() {
    let lexeme_id = "synodal:numeral:edin";
    let instrumental = analyze("є҆ди́нѣмъ").expect("reviewed cardinal-one instrumental");
    assert!(instrumental.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == lexeme_id
            && matches!(
                analysis.cell,
                Some(GrammarCell::Numeral(NumeralCell {
                    kind: NumeralKind::Cardinal,
                    case: Case::Instrumental,
                    number: Number::Singular,
                    gender: Some(Gender::Neuter),
                    ..
                }))
            )
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    assert!(instrumental.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == lexeme_id
            && matches!(
                analysis.cell,
                Some(GrammarCell::Numeral(NumeralCell {
                    kind: NumeralKind::Cardinal,
                    case: Case::Instrumental,
                    number: Number::Singular,
                    gender: Some(Gender::Masculine),
                    ..
                }))
            )
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));

    let feminine = analyze("є҆ди́ною").expect("reviewed feminine cardinal-one instrumental");
    assert!(feminine.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == lexeme_id
            && matches!(
                analysis.cell,
                Some(GrammarCell::Numeral(NumeralCell {
                    kind: NumeralKind::Cardinal,
                    case: Case::Instrumental,
                    number: Number::Singular,
                    gender: Some(Gender::Feminine),
                    ..
                }))
            )
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    for analyses in [&instrumental, &feminine] {
        assert!(analyses.iter().all(|analysis| {
            analysis.lexeme.id().as_str() != lexeme_id
                || matches!(
                    analysis.cell,
                    Some(GrammarCell::Numeral(NumeralCell {
                        number: Number::Singular,
                        ..
                    }))
                )
        }));
        assert!(!analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));
    }

    let wrong_accent = analyze("є҆́динѣмъ").expect("valid but wrongly accented numeral");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != lexeme_id)
    );
}

#[test]
fn rab_wide_omega_plural_keeps_attested_genitive_and_normative_accusative() {
    let lexeme_id = "synodal:noun:rab";
    let analyses = analyze("рабѡ́въ").expect("reviewed wide-omega servant form");
    for (case, source) in [
        (Case::Genitive, AnalysisSource::ExactSynodalAttestation),
        (Case::Accusative, AnalysisSource::SynodalNormativeTable),
    ] {
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == lexeme_id
                && analysis.cell
                    == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                        case,
                        number: Number::Plural,
                        animacy: Animacy::Animate,
                    }))
                && analysis.source == source
        }));
    }
}

#[test]
fn analyzer_keeps_closed_class_variants_exact_and_collision_free() {
    let ko = analyze("ко").expect("valid positional preposition variant");
    assert!(ko.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:preposition:wikt-77998a1b179f"
            && analysis.cell == Some(GrammarCell::LexicalForm)
    }));

    let vo = analyze("во").expect("valid positional preposition variant");
    assert!(vo.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:preposition:wikt-9c77102d5441"
            && analysis.cell == Some(GrammarCell::LexicalForm)
    }));

    let so = analyze("со").expect("valid positional preposition variant");
    assert!(so.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:preposition:wikt-c63ef675e22e"
    }));

    let ubo = analyze("ᲂу҆̀бо").expect("valid positional conjunction variant");
    assert!(ubo.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:conjunction:wikt-a0dc1a363208"
    }));

    let soti = analyze("соти").expect("orthographically valid negative control");
    assert!(soti.iter().all(|analysis| {
        analysis.lexeme.id().as_str() != "synodal:preposition:wikt-c63ef675e22e"
    }));
    let liti = analyze("лити").expect("orthographically valid negative control");
    assert!(
        liti.iter()
            .all(|analysis| { analysis.lexeme.id().as_str() != "synodal:conjunction:li" })
    );
    let yuzhe = analyze("юже").expect("orthographically valid negative control");
    assert!(
        yuzhe.iter().all(|analysis| {
            analysis.lexeme.id().as_str() != "synodal:noun:wikt-f330683bc04d"
        })
    );
}

#[test]
fn analyzer_expands_semantic_abbreviation() {
    let analyses = analyze("бг҃ъ").expect("valid abbreviation");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.lemma() == "богъ"
            && analysis.source == AnalysisSource::AbbreviationExpansion
    }));
}

#[test]
fn analyzer_requires_explicit_policy_for_inherited_predictions() {
    assert!(analyze("градомъ").expect("valid input").is_empty());
    let analyses = analyze_with(
        "градомъ",
        Inflector::builder()
            .generation_policy(synodal_church_slavonic::GenerationPolicy::Productive)
            .build(),
    )
    .expect("valid productive analysis");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.lemma() == "градъ"
            && analysis.source == AnalysisSource::InheritedPrediction
            && analysis.recension_mapping.is_some()
    }));
}

#[test]
fn vocabulary_lint_rejects_latin_and_missing_sense() {
    let issues = lint_vocabulary(&VocabularyManifest {
        entries: vec![
            VocabularyItem {
                text: "slovo".into(),
                expected_lexeme_id: None,
                expected_part_of_speech: None,
                required_sense_id: None,
                requested_cell: None,
            },
            VocabularyItem {
                text: "рабъ".into(),
                expected_lexeme_id: None,
                expected_part_of_speech: Some(PartOfSpeech::Noun),
                required_sense_id: Some("missing".into()),
                requested_cell: None,
            },
        ],
    });
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == VocabularyIssueKind::InvalidOrthography)
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == VocabularyIssueKind::MissingSemanticIdentity)
    );
}

#[test]
fn vocabulary_lint_uses_the_supplied_analyzer_policy_for_requested_cells() {
    let analyzer = coverage::Analyzer::new(
        Inflector::builder()
            .generation_policy(synodal_church_slavonic::GenerationPolicy::Productive)
            .build(),
    )
    .expect("productive analyzer");
    let issues = lint_vocabulary_with(
        &analyzer,
        &VocabularyManifest {
            entries: vec![VocabularyItem {
                text: "граде".into(),
                expected_lexeme_id: Some(LexemeId::from("synodal:noun:grad")),
                expected_part_of_speech: Some(PartOfSpeech::Noun),
                required_sense_id: None,
                requested_cell: Some(GrammarCell::Noun(morphology::NounCell {
                    case: Case::Vocative,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                })),
            }],
        },
    );
    assert!(
        issues
            .iter()
            .all(|issue| issue.kind != VocabularyIssueKind::UnsupportedFormation),
        "productive requested cell was rejected: {issues:?}"
    );
}

#[test]
fn gloss_search_is_deterministic() {
    let results = search_gloss("religion").expect("search");
    assert!(
        results
            .windows(2)
            .all(|pair| pair[0].lexeme.id() < pair[1].lexeme.id())
    );
}

#[test]
fn family_lookup_excludes_rejected_contextual_homograph() {
    let results = families("ꙗкѡ").expect("reviewed families");
    let identities: BTreeSet<_> = results
        .iter()
        .map(|family| (family.lexeme.id().as_str(), family.lexeme.part_of_speech()))
        .collect();
    assert!(!identities.contains(&("synodal:adverb:wikt-5471d4207f64", PartOfSpeech::Adverb)));
    assert!(identities.contains(&(
        "synodal:conjunction:wikt-47fa23a7ed6b",
        PartOfSpeech::Conjunction
    )));
}

#[test]
fn kamen_exact_and_productive_analyses_share_one_stable_identity() {
    let analyses = analyze("камень").expect("reviewed камень analyses");
    let identities: BTreeSet<_> = analyses
        .iter()
        .map(|analysis| analysis.lexeme.id().as_str())
        .collect();

    assert_eq!(
        identities,
        BTreeSet::from(["synodal:noun:v07-c27905de175a0cde"])
    );
    assert!(
        analyses
            .iter()
            .any(|analysis| analysis.source == AnalysisSource::ExactSynodalAttestation)
    );
}

#[test]
fn family_summary_exposes_exact_cells_and_productive_determiner_metadata() {
    let id = FamilyId::for_lexeme(&LexemeId::from("synodal:determiner:ves"));
    let family = show_family_by_id(&id).expect("reviewed весь family");
    assert_eq!(family.id.as_str(), "family:synodal:determiner:ves");
    assert!(!family.exact_only);
    assert!(family.fully_classed);
    assert_eq!(family.class.as_deref(), Some("determiner-ves-mixed"));
    assert_eq!(family.stem.as_deref(), Some("вс"));
    assert!(family.members.iter().any(|member| {
        member.cell == "determiner:nominative:singular:feminine:inanimate:short:positive"
            && member.printed == "всѧ̀"
    }));
    assert!(family.missing_family_metadata.is_empty());
}

#[test]
fn family_supported_systems_cover_productive_and_exact_capabilities() {
    for (id, expected) in [
        ("synodal:determiner:sam", "determiner"),
        ("synodal:numeral:pervyi", "numeral"),
        ("synodal:verb:byti", "future"),
        ("synodal:verb:wikt-78da2d05497d", "aorist"),
    ] {
        let family = show_family_by_id(&FamilyId::for_lexeme(&LexemeId::from(id)))
            .expect("reviewed family");
        assert!(
            family
                .supported_systems
                .iter()
                .any(|system| system == expected),
            "{id} should report {expected}: {:?}",
            family.supported_systems
        );
    }
}

#[test]
fn complete_possessive_tables_are_truthfully_classed_and_productive() {
    for lexeme in ["moi", "tvoi", "svoi", "nash", "vash"] {
        let id = FamilyId::for_lexeme(&LexemeId::from(format!("synodal:pronoun:{lexeme}")));
        let family = show_family_by_id(&id).expect("reviewed possessive family");
        assert!(!family.exact_only);
        assert!(family.fully_classed);
        assert_eq!(family.members.len(), 57);
        assert!(family.missing_family_metadata.is_empty());
    }

    let vash = show_family_by_id(&FamilyId::for_lexeme(&LexemeId::from(
        "synodal:pronoun:vash",
    )))
    .expect("reviewed вашъ family");
    assert!(vash.members.iter().any(|member| {
        member.cell == "pronoun:dative:plural:masculine:none:any"
            && member.expanded == "вашымъ"
            && member.printed == "ва́шымъ"
    }));
    assert!(vash.members.iter().any(|member| {
        member.cell == "pronoun:accusative:singular:masculine:none:animate"
            && member.expanded == "вашего"
    }));

    let gospoden_id = LexemeId::from("synodal:adjective:gospoden");
    let gospoden_cells = analysis_cells_by_id(&gospoden_id, Inflector::default())
        .expect("typed short possessive cells");
    assert!(gospoden_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
            ..
        })
    )));
    assert!(!gospoden_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            form: AdjectiveForm::Long,
            ..
        })
    )));

    let bozhii_id = LexemeId::from("synodal:adjective:bozhii");
    let bozhii_cells = analysis_cells_by_id(&bozhii_id, Inflector::default())
        .expect("typed -їй possessive cells");
    assert!(bozhii_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            form: AdjectiveForm::Long,
            comparison: Comparison::Positive,
            ..
        })
    )));
    assert!(!bozhii_cells.iter().any(|cell| matches!(
        cell,
        GrammarCell::Adjective(AdjectiveCell {
            comparison: Comparison::Comparative | Comparison::Superlative,
            ..
        })
    )));
}
