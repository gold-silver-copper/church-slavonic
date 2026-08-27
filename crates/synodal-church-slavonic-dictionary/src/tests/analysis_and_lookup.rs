#[allow(unused_imports)]
use crate::*;
use synodal_church_slavonic::{Case, NounCell};

#[test]
fn semantic_lookup_keeps_source_recension_visible() {
    let entry = lookup("землѧ").expect("known entry");
    assert_eq!(entry.senses[0].source_recension, "mixed");
    assert_eq!(
        entry.senses[0].semantic_status,
        "reviewed-with-synodal-corpus"
    );
}

#[test]
fn analyzer_returns_typed_cells_without_guessing_one() {
    let analyses = analyze("є҆́смь").expect("valid input");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.lemma() == "быти"
            && matches!(analysis.cell, Some(GrammarCell::FiniteVerb(_)))
    }));
}

#[test]
fn verb_candidate_inventory_includes_every_represented_system() {
    let cells = candidate_cells(PartOfSpeech::Verb);
    assert_eq!(cells.len(), 1_116);
    assert!(cells.contains(&GrammarCell::Supine));
    assert!(
        cells
            .iter()
            .any(|cell| matches!(cell, GrammarCell::VerbalNoun(_)))
    );
}

#[test]
fn perfective_present_shaped_future_keeps_both_reverse_analyses() {
    for (id, surface) in [
        ("synodal:verb:v07-35ce5d83583f3639", "начнева"),
        ("synodal:verb:polozhiti", "положива"),
    ] {
        let id = LexemeId::from(id);
        let cells = analysis_cells_by_id(&id, Inflector::default())
            .expect("productive perfective inventory");
        for tense in [FiniteTense::Present, FiniteTense::Future] {
            assert!(cells.contains(&GrammarCell::FiniteVerb(FiniteVerbCell {
                tense,
                person: Person::First,
                number: Number::Dual,
            })));
        }

        let analyses = analyze(surface).expect("homographic present/future analysis");
        let tenses = analyses
            .iter()
            .filter(|analysis| analysis.lexeme.id() == &id)
            .filter_map(|analysis| match analysis.cell {
                Some(GrammarCell::FiniteVerb(cell)) => Some(cell.tense),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert!(tenses.contains(&FiniteTense::Present));
        assert!(tenses.contains(&FiniteTense::Future));
    }
}

#[test]
fn generated_reverse_index_covers_complete_verbal_noun_paradigms() {
    let id = LexemeId::from("synodal:verb:nesti");
    let cells = analysis_cells_by_id(&id, Inflector::default())
        .expect("reviewed productive verb inventory");
    let verbal_nouns = cells
        .iter()
        .filter(|cell| matches!(cell, GrammarCell::VerbalNoun(_)))
        .count();
    assert_eq!(verbal_nouns, 42);

    let analyses = analyze("несенїѧ").expect("productive reverse analysis");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.id() == &id
            && analysis.cell
                == Some(GrammarCell::VerbalNoun(core::NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn candidate_inventory_sizes_remain_exhaustive_and_stable() {
    for (part_of_speech, expected) in [
        (PartOfSpeech::Adverb, 2),
        (PartOfSpeech::Noun, 43),
        (PartOfSpeech::Adjective, 757),
        (PartOfSpeech::Pronoun, 673),
        (PartOfSpeech::Numeral, 841),
        (PartOfSpeech::Determiner, 757),
        (PartOfSpeech::Participle, 1),
    ] {
        let cells = candidate_cells(part_of_speech);
        assert_eq!(cells.len(), expected, "{part_of_speech:?}");
        assert_eq!(cells.last(), Some(&GrammarCell::LexicalForm));
    }
}

#[test]
fn every_productive_pronoun_identity_realizes_every_licensed_analysis_cell() {
    let inflector = Inflector::default();
    for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
        if lexeme.part_of_speech() != PartOfSpeech::Pronoun {
            continue;
        }
        let metadata = lexical_metadata(lexeme.id()).expect("pronoun metadata");
        if !metadata
            .class
            .as_deref()
            .is_some_and(|class| class.starts_with("pronoun-"))
        {
            continue;
        }
        let cells = analysis_cells_for_lexeme(&lexeme, inflector)
            .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
        assert!(
            cells
                .iter()
                .any(|cell| matches!(cell, GrammarCell::Pronoun(_))),
            "{} has no productive pronoun cells",
            lexeme.id()
        );
        for cell in cells {
            if !matches!(cell, GrammarCell::Pronoun(_)) {
                continue;
            }
            let forms = inflector
                .form_by_id(lexeme.id(), cell)
                .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
            assert!(
                !forms.variants().is_empty(),
                "{} {}",
                lexeme.id(),
                cell.key()
            );
        }
    }
}

#[test]
fn every_productive_determiner_identity_realizes_every_licensed_analysis_cell() {
    let inflector = Inflector::default();
    for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
        if lexeme.part_of_speech() != PartOfSpeech::Determiner {
            continue;
        }
        let metadata = lexical_metadata(lexeme.id()).expect("determiner metadata");
        if !metadata
            .class
            .as_deref()
            .is_some_and(|class| class.starts_with("determiner-"))
        {
            continue;
        }
        let cells = analysis_cells_for_lexeme(&lexeme, inflector)
            .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
        assert!(
            cells
                .iter()
                .any(|cell| matches!(cell, GrammarCell::Determiner(_))),
            "{} has no productive determiner cells",
            lexeme.id()
        );
        for cell in cells {
            if !matches!(cell, GrammarCell::Determiner(_)) {
                continue;
            }
            let forms = inflector
                .form_by_id(lexeme.id(), cell)
                .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
            assert!(
                !forms.variants().is_empty(),
                "{} {}",
                lexeme.id(),
                cell.key()
            );
        }
    }
}

#[test]
fn every_productive_numeral_identity_realizes_every_licensed_analysis_cell() {
    let inflector = Inflector::default();
    for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
        if lexeme.part_of_speech() != PartOfSpeech::Numeral {
            continue;
        }
        let metadata = lexical_metadata(lexeme.id()).expect("numeral metadata");
        if !metadata
            .class
            .as_deref()
            .is_some_and(|class| class.starts_with("numeral-") || class.starts_with("ordinal-"))
        {
            continue;
        }
        let cells = analysis_cells_for_lexeme(&lexeme, inflector)
            .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
        assert!(
            cells
                .iter()
                .any(|cell| matches!(cell, GrammarCell::Numeral(_))),
            "{} has no productive numeral cells",
            lexeme.id()
        );
        for cell in cells {
            if !matches!(cell, GrammarCell::Numeral(_)) {
                continue;
            }
            let forms = inflector
                .form_by_id(lexeme.id(), cell)
                .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
            assert!(
                !forms.variants().is_empty(),
                "{} {}",
                lexeme.id(),
                cell.key()
            );
        }
    }
}

#[test]
fn analyzer_uses_explicit_accents_to_disambiguate_homographs() {
    let conjunction = analyze("и҆").expect("valid conjunction");
    assert_eq!(conjunction.len(), 1);
    assert_eq!(
        conjunction[0].lexeme.part_of_speech(),
        PartOfSpeech::Conjunction
    );

    let pronoun = analyze("и҆̀").expect("valid pronoun");
    assert!(!pronoun.is_empty());
    assert!(
        pronoun
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() == "synodal:pronoun:on")
    );

    let unmarked = analyze("и").expect("valid unmarked spelling");
    assert!(unmarked.len() > 1, "unmarked homograph must stay ambiguous");

    let incompatible = analyze("и\u{301}").expect("valid incompatible accent");
    assert!(
        incompatible.is_empty(),
        "an explicit incompatible accent must not fall back to accentless analysis"
    );
}

#[test]
fn analyzer_canonicalizes_reviewed_conjunction_marks_without_restoring_rejected_identity() {
    let analyses = analyze("ꙗ҆́кѡ").expect("valid reviewed marked form");
    let identities: BTreeSet<_> = analyses
        .iter()
        .map(|analysis| analysis.lexeme.id().as_str())
        .collect();
    assert_eq!(identities.len(), 1);
    assert!(identities.contains("synodal:conjunction:wikt-47fa23a7ed6b"));
    assert!(!identities.contains("synodal:adverb:wikt-5471d4207f64"));
}

#[test]
fn ponomar_dictionary_reviews_admit_only_the_attested_exact_forms() {
    let reviewed = [
        ("жре́цъ", "synodal:noun:v11-332e30b022aa"),
        ("саꙋ́лъ", "synodal:proper-noun:v11-1c75360357d9"),
        ("со́нмъ", "synodal:noun:v11-ba59d1e727b5"),
        ("совѣ́тъ", "synodal:noun:v11-c3606bfd87d5"),
        ("і҆ѡнаѳа́нъ", "synodal:proper-noun:v11-4e8cbf1465b4"),
        ("ѕло̀", "synodal:noun:v11-112ca1130b42"),
        ("премꙋ́дрость", "synodal:noun:v11-160c9fb86c0f"),
        ("то́чїю", "synodal:adverb:v11-da873f6ae112"),
        ("прѧ́мѡ", "synodal:adverb:v11-8f3d7ab0c925"),
        ("сквозѣ̀", "synodal:preposition:v11-2b9f7f3a2990"),
    ];
    for (surface, expected_id) in reviewed {
        let analyses = analyze(surface).expect("valid reviewed Ponomar form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == expected_id
                    && analysis.source == AnalysisSource::ExactSynodalAttestation
            }),
            "missing exact analysis for {surface:?}"
        );
    }

    // Alypy §5: a word-final acute is the pre-enclitic print of the reviewed
    // grave-final surface (ѕло̀ / ѕло́ же), so it reaches the same review.
    for (surface, expected_id) in [
        ("ѕло́", "synodal:noun:v11-112ca1130b42"),
        ("сквозѣ́", "synodal:preposition:v11-2b9f7f3a2990"),
    ] {
        let analyses = analyze(surface).expect("valid pre-enclitic print");
        assert!(
            analyses
                .iter()
                .any(|analysis| analysis.lexeme.id().as_str() == expected_id),
            "pre-enclitic acute did not reach exact review {expected_id}"
        );
    }

    let (surface, excluded_id) = ("жрѐцъ", "synodal:noun:v11-332e30b022aa");
    let analyses = analyze(surface).expect("valid mark-sensitive negative control");
    assert!(
        analyses
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != excluded_id),
        "unattested marked variant reached exact review {excluded_id}"
    );
}

#[test]
fn vosled_preposition_is_exact_mark_sensitive_and_distinct_from_sled_noun() {
    for surface in ["в̾слѣ́дъ", "вослѣ́дъ", "вослѣдъ"] {
        let analyses = analyze(surface).expect("valid reviewed preposition form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:preposition:vosled"
                    && analysis.cell == Some(GrammarCell::Indeclinable)
            }),
            "missing exact preposition analysis for {surface:?}"
        );
    }

    let wrong_mark = analyze("вслѣ́дъ").expect("valid mark-sensitive negative control");
    assert!(
        wrong_mark
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:preposition:vosled")
    );

    let noun = analyze("слѣ́дъ").expect("valid noun control");
    assert!(
        noun.iter()
            .any(|analysis| { analysis.lexeme.id().as_str() == "synodal:noun:wikt-c96e00520110" })
    );
    assert!(
        noun.iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:preposition:vosled")
    );
}

#[test]
fn bez_preposition_is_exact_mark_sensitive_and_not_a_prefix_fallback() {
    for surface in ["без̾", "безъ"] {
        let analyses = analyze(surface).expect("reviewed primary preposition");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:preposition:bez"
                && analysis.cell == Some(GrammarCell::Indeclinable)
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));
    }

    for surface in ["без", "без̾мѣры"] {
        let analyses = analyze(surface).expect("valid mark-sensitive negative control");
        assert!(
            analyses
                .iter()
                .all(|analysis| analysis.lexeme.id().as_str() != "synodal:preposition:bez"),
            "unreviewed spelling or substring reached exact без̾ analysis: {surface:?}"
        );
    }
}

#[test]
fn srede_preposition_is_exact_and_does_not_license_accentless_or_substring_forms() {
    // средѣ́ is the pre-enclitic print of the reviewed средѣ̀ (Alypy §5).
    for surface in ["средѣ̀", "средѣ", "средѣ́"] {
        let analyses = analyze(surface).expect("reviewed medial preposition");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:preposition:srede"
                && analysis.cell == Some(GrammarCell::Indeclinable)
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));
    }

    let surface = "посре́дѣ";
    let analyses = analyze(surface).expect("valid exact negative control");
    assert!(
        analyses
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:preposition:srede"),
        "unreviewed accent or substring reached exact средѣ̀ analysis: {surface:?}"
    );
}

#[test]
fn nasledie_soft_neuter_round_trips_exact_and_productive_cells() {
    for surface in ["наслѣ́дїе", "наслѣ́дїѧ", "наслѣ́дїю", "наслѣ́дїемъ", "наслѣ́дїи"]
    {
        let analyses = analyze(surface).expect("reviewed inheritance noun form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:nasledie"
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed inheritance-noun analysis for {surface:?}"
        );
    }

    for surface in ["наслѣ́дїе", "наслѣ́дїѧ", "наслѣ́дїемъ", "наслѣ́дїи"]
    {
        let analyses = analyze(surface).expect("source-backed inheritance noun form");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:noun:nasledie"
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));
    }

    let plural_kamora = analyze("наслѣ̑дїѧ").expect("reviewed plural kamora variant");
    assert!(plural_kamora.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:nasledie"
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let wrong_accent = analyze("на́слѣдїе").expect("valid accent negative control");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:nasledie")
    );
}

#[test]
fn predel_hard_masculine_preserves_exact_genitive_plural_kamora() {
    for surface in [
        "предѣ́лъ",
        "предѣ́ла",
        "предѣ́лы",
        "предѣ̑лъ",
        "предѣ́лѡвъ",
        "предѣ́лѣхъ",
    ] {
        let analyses = analyze(surface).expect("reviewed boundary noun form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:predel"
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed boundary-noun analysis for {surface:?}"
        );
    }

    let genitive_plural = analyze("предѣ̑лъ").expect("exact genitive plural kamora");
    assert!(genitive_plural.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:predel"
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    let genitive_plural_ov = analyze("предѣ́лѡвъ").expect("exact genitive plural -ov variant");
    assert!(genitive_plural_ov.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:predel"
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let wrong_accent = analyze("пре́дѣлы").expect("valid accent negative control");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:predel")
    );
}

#[test]
fn yazyk_velar_family_round_trips_productive_and_exact_target_cells() {
    for surface in [
        "ꙗ҆зы́къ",
        "ꙗ҆зы́кꙋ",
        "ꙗ҆зы́че",
        "ꙗ҆зы́цы",
        "ꙗ҆зы́кѡвъ",
        "ꙗ҆зы́кѡмъ",
        "ꙗ҆зы́цѣхъ",
    ] {
        let analyses = analyze(surface).expect("reviewed language or people noun form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:yazyk"
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed ꙗзыкъ-family analysis for {surface:?}"
        );
    }

    for surface in ["ꙗ҆зы́кѡвъ", "ꙗ҆зы́кѡмъ", "ꙗ҆зы́цѣхъ"] {
        let analyses = analyze(surface).expect("source-backed ꙗзыкъ target variant");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:noun:yazyk"
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));
    }

    let wrong_accent = analyze("ꙗ҆́зыкѡвъ").expect("valid accent negative control");
    assert!(
        wrong_accent
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:yazyk")
    );
}

#[test]
fn mesyats_productive_noun_and_typed_abbreviations_round_trip() {
    for surface in [
        "мѣ́сѧцъ",
        "мѣ́сѧца",
        "мѣ́сѧцꙋ",
        "мѣ́сѧцѣ",
        "мѣ́сѧцы",
        "мѣ́сѧцей",
        "мѣ́сѧцамъ",
        "мѣ́сѧцѣхъ",
    ] {
        let analyses = analyze(surface).expect("reviewed month noun form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:mesyats"
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed month analysis for {surface:?}"
        );
    }

    for surface in [
        "мцⷭ҇ъ",
        "мцⷭ҇а",
        "мцⷭ҇ꙋ",
        "мцⷭ҇ѣ",
        "мцⷭ҇ы",
        "мцⷭ҇ей",
        "мцⷭ҇євъ",
        "мцⷭ҇амъ",
        "мцⷭ҇ѣхъ",
    ] {
        let analyses = analyze(surface).expect("reviewed month abbreviation");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:mesyats"
                    && analysis.source == AnalysisSource::AbbreviationExpansion
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed month-abbreviation analysis for {surface:?}"
        );
    }

    let genitive = abbreviation::contract_variants_for_cell_by_id(
        &LexemeId::from("synodal:noun:mesyats"),
        "sense:v13:noun:mesyats",
        GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
            case: Case::Genitive,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        }),
    )
    .expect("typed month contraction");
    assert!(
        genitive
            .iter()
            .any(|form| { form.expanded == "мѣсѧца" && form.printed == "мцⷭ҇а" })
    );

    assert!(
        analyze("мцса")
            .expect("valid missing-mark negative control")
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:mesyats")
    );
}

#[test]
fn salvation_contraction_traditions_preserve_exact_reverse_cells() {
    for surface in [
        "спⷭ҇нїе",
        "спⷭ҇нїѧ",
        "спⷭ҇нїи",
        "спⷭ҇нїемъ",
        "спⷭ҇нїй",
        "сп҃се́нїе",
        "сп҃се́нїѧ",
        "сп҃се́нїи",
        "сп҃се́нїемъ",
        "сп҃се́нїю",
        "сп҃се́нїй",
        "Сп҃се́нїе",
    ] {
        let analyses = analyze(surface).expect("reviewed salvation abbreviation");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:spasenie"
                    && analysis.source == AnalysisSource::AbbreviationExpansion
                    && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
            }),
            "missing typed salvation-abbreviation analysis for {surface:?}"
        );
    }

    for surface in ["спⷭ҇нїе́", "спснїе"] {
        assert!(
            analyze(surface)
                .expect("orthographically valid negative control")
                .iter()
                .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:spasenie"),
            "unreviewed spelling must not reach the salvation identity: {surface:?}"
        );
    }
}

#[test]
fn newly_admitted_soft_ie_neuters_round_trip_productively() {
    for (id, surfaces) in [
        (
            "synodal:noun:vsesozhzhenie",
            &[
                "всесожже́нїе",
                "всесожже́нїѧ",
                "всесожже́нїю",
                "всесожже́нїемъ",
                "всесожжє́нїѧ",
            ][..],
        ),
        (
            "synodal:noun:sretenie",
            &["срѣ́тенїе", "срѣ́тенїѧ", "срѣ́тенїю", "срѣ́тенїемъ", "срѣ́тенїи"][..],
        ),
    ] {
        for surface in surfaces {
            let analyses = analyze(surface).expect("reviewed soft -їе noun form");
            assert!(
                analyses.iter().any(|analysis| {
                    analysis.lexeme.id().as_str() == id
                        && matches!(analysis.cell, Some(GrammarCell::Noun(_)))
                }),
                "missing {id} analysis for {surface:?}"
            );
        }
    }

    for (wrong, id) in [
        ("все́сожженїе", "synodal:noun:vsesozhzhenie"),
        ("срѣте́нїе", "synodal:noun:sretenie"),
    ] {
        assert!(
            analyze(wrong)
                .expect("valid wrong-accent negative control")
                .iter()
                .all(|analysis| analysis.lexeme.id().as_str() != id)
        );
    }
}

#[test]
fn vino_mobile_accent_and_polozhiti_systems_round_trip_productively() {
    for (id, surfaces) in [
        (
            "synodal:noun:vino",
            &["вїно̀", "вїна̀", "вїнꙋ̀", "вїно́мъ", "вїнѣ̀"][..],
        ),
        (
            "synodal:verb:polozhiti",
            &[
                "положи́ти",
                "положꙋ̀",
                "положи́ши",
                "положи́тъ",
                "положа́тъ",
                "положи́хъ",
                "положи́сте",
                "положи́ша",
                "положи́ла",
            ][..],
        ),
    ] {
        for surface in surfaces {
            assert!(
                analyze(surface)
                    .expect("reviewed productive surface")
                    .iter()
                    .any(|analysis| analysis.lexeme.id().as_str() == id),
                "missing {id} analysis for {surface:?}"
            );
        }
    }

    let polozhi = analyze("положѝ").expect("reviewed syncretic verb surface");
    let cells = polozhi
        .iter()
        .filter(|analysis| analysis.lexeme.id().as_str() == "synodal:verb:polozhiti")
        .filter_map(|analysis| analysis.cell)
        .collect::<BTreeSet<_>>();
    assert!(cells.contains(&GrammarCell::FiniteVerb(FiniteVerbCell {
        tense: FiniteTense::Aorist,
        person: Person::Third,
        number: Number::Singular,
    })));
    assert!(cells.contains(&GrammarCell::Imperative(ImperativeCell {
        person: Person::Second,
        number: Number::Singular,
    })));

    assert!(
        analyze("ви́на")
            .expect("valid wrong positional spelling")
            .iter()
            .all(|analysis| analysis.lexeme.id().as_str() != "synodal:noun:vino")
    );
}

#[test]
fn no_yat_vsem_and_titlecase_month_abbreviations_remain_exactly_scoped() {
    let vsem = analyze("все́мъ").expect("reviewed no-yat locative");
    let ves_cells = vsem
        .iter()
        .filter(|analysis| analysis.lexeme.id().as_str() == "synodal:determiner:ves")
        .filter_map(|analysis| analysis.cell)
        .collect::<BTreeSet<_>>();
    assert_eq!(ves_cells.len(), 2);
    assert!(ves_cells.iter().all(|cell| matches!(
        cell,
        GrammarCell::Determiner(AdjectiveCell {
            case: Case::Locative,
            number: Number::Singular,
            ..
        })
    )));

    for (surface, case) in [("Мцⷭ҇а", Case::Genitive), ("Мцⷭ҇ъ", Case::Nominative)] {
        assert!(
            analyze(surface)
                .expect("reviewed titlecase abbreviation")
                .iter()
                .any(|analysis| {
                    analysis.lexeme.id().as_str() == "synodal:noun:mesyats"
                        && analysis.source == AnalysisSource::AbbreviationExpansion
                        && matches!(
                            analysis.cell,
                            Some(GrammarCell::Noun(NounCell {
                                case: found,
                                number: Number::Singular,
                                ..
                            })) if found == case
                        )
                }),
            "missing titlecase month analysis for {surface:?}"
        );
    }
}

#[test]
fn sotvoriti_productive_systems_round_trip_with_exact_precedence() {
    for (surface, expected_cells) in [
        ("сотвори́те", 1_usize),
        ("сотворитѐ", 1),
        ("сотвори́мъ", 2),
        ("сотвори́хомъ", 1),
        ("сотвори́ла", 2),
    ] {
        let analyses = analyze(surface).expect("reviewed сотвори́ти system surface");
        let matching = analyses
            .iter()
            .filter(|analysis| analysis.lexeme.id().as_str() == "synodal:verb:sotvoriti")
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), expected_cells, "{surface:?}");
    }

    let exact = analyze("сотворитѐ").expect("exact future accent variant");
    assert!(exact.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:verb:sotvoriti"
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    // Alypy §5: the word-final acute is the pre-enclitic print of the
    // reviewed grave variant (сотвори́те же), so it reaches the same lexeme.
    let pre_enclitic = analyze("сотворите́").expect("pre-enclitic acute print");
    assert!(
        pre_enclitic
            .iter()
            .any(|analysis| analysis.lexeme.id().as_str() == "synodal:verb:sotvoriti")
    );
}

#[test]
fn zlyi_is_productive_mark_sensitive_and_preserves_zlo_homonymy() {
    for (surface, expected_cell) in [
        (
            "ѕла̑ѧ",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Plural,
                gender: Gender::Neuter,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            }),
        ),
        (
            "ѕо́лъ",
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
        ),
    ] {
        let analyses = analyze(surface).expect("valid reviewed adjective form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:adjective:zlyi"
                    && analysis.cell == Some(expected_cell)
                    && analysis.source == AnalysisSource::ExactSynodalAttestation
            }),
            "missing exact adjective analysis for {surface:?}"
        );
    }

    let wrong_mark = analyze("ѕла́ѧ").expect("valid mark-sensitive negative control");
    assert!(
        wrong_mark.iter().all(|analysis| {
            analysis.lexeme.id().as_str() != "synodal:adjective:zlyi"
                || analysis.source != AnalysisSource::ExactSynodalAttestation
        }),
        "the acute form must not inherit the exact kamora attestation: {wrong_mark:#?}"
    );

    let noun = analyze("ѕло̀").expect("valid noun control");
    assert!(
        noun.iter()
            .any(|analysis| { analysis.lexeme.id().as_str() == "synodal:noun:v11-112ca1130b42" })
    );
    assert!(noun.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:adjective:zlyi"
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn dusha_upgrade_preserves_exact_cells_and_exposes_productive_background() {
    for (surface, expected_cell) in [
        (
            "дꙋшѝ",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Genitive,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "дꙋ́шы",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Accusative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "дꙋши̑",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Nominative,
                number: Number::Dual,
                animacy: Animacy::Inanimate,
            }),
        ),
        (
            "дꙋ́ши",
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: Case::Nominative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            }),
        ),
    ] {
        let analyses = analyze(surface).expect("valid reviewed soul form");
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:v07-549dec12f8aeb0c9"
                    && analysis.cell == Some(expected_cell)
                    && analysis.source == AnalysisSource::ExactSynodalAttestation
            }),
            "missing exact soul analysis for {surface:?}"
        );
    }

    let productive = analyze("дꙋшѐ").expect("valid productive vocative");
    assert!(productive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:v07-549dec12f8aeb0c9"
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn adonai_analysis_preserves_exact_and_productive_indeclinable_readings() {
    let analyses = analyze("а҆дѡнаі̀").expect("reviewed divine title");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:adonai"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                }))
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:adonai"
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn zhena_wide_e_plural_keeps_both_exact_direct_case_analyses() {
    let analyses = analyze("жєны̀").expect("reviewed wide-e plural");
    for case in [Case::Nominative, Case::Accusative] {
        assert!(
            analyses.iter().any(|analysis| {
                analysis.lexeme.id().as_str() == "synodal:noun:zhena"
                    && analysis.cell
                        == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                            case,
                            number: Number::Plural,
                            animacy: Animacy::Animate,
                        }))
                    && analysis.source == AnalysisSource::ExactSynodalAttestation
            }),
            "missing {case:?} plural analysis"
        );
    }

    let narrow = analyze("жены̀").expect("reviewed narrow-e genitive");
    assert!(narrow.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:zhena"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                }))
    }));
}

#[test]
fn svidenie_analysis_combines_exact_genitive_and_productive_plural() {
    let analyses = analyze("свидѣ́нїѧ").expect("reviewed testimony form");
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:svidenie"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    assert!(analyses.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:svidenie"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn dshcher_analysis_combines_exact_and_productive_oblique_cells() {
    let exact = analyze("дщє́ри").expect("reviewed daughter plural");
    assert!(exact.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:v07-db06c7a6afdd2e88"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Animate,
                }))
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let productive = analyze("дще́рїй").expect("productive daughter genitive plural");
    assert!(productive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:v07-db06c7a6afdd2e88"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn sosud_analysis_preserves_exact_variants_and_productive_cells() {
    let ordinary = analyze("сосꙋ́ды").expect("reviewed vessel plural");
    for case in [Case::Nominative, Case::Accusative] {
        assert!(ordinary.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:noun:sosud"
                && analysis.cell
                    == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                        case,
                        number: Number::Plural,
                        animacy: Animacy::Inanimate,
                    }))
                && analysis.source == AnalysisSource::ExactSynodalAttestation
        }));
    }

    let alternative = analyze("сосꙋ́ди").expect("reviewed alternative nominative plural");
    assert!(alternative.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:sosud"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));

    let productive = analyze("сосꙋ́дꙋ").expect("productive vessel dative singular");
    assert!(productive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:noun:sosud"
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}

#[test]
fn iuda_analysis_preserves_exact_singular_and_productive_number_cells() {
    let genitive = analyze("і҆ꙋ́ды").expect("reviewed Judah/Judas genitive");
    assert!(genitive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:proper-noun:iuda"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                }))
            && analysis.source == AnalysisSource::ExactSynodalAttestation
    }));
    assert!(genitive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:proper-noun:iuda"
            && analysis.cell
                == Some(GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                }))
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));

    let productive = analyze("і҆ꙋ́дъ").expect("productive Judah/Judas genitive plural");
    assert!(productive.iter().any(|analysis| {
        analysis.lexeme.id().as_str() == "synodal:proper-noun:iuda"
            && analysis.source == AnalysisSource::SynodalProductiveRule
    }));
}
