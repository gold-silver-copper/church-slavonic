use old_church_slavonic::adjective::AdjectiveLexeme;
use old_church_slavonic::noun::NounLexeme;
use old_church_slavonic::verb::VerbLexeme;
use old_church_slavonic::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, Case, ClosedClassCell,
    FiniteTense, FiniteVerbCell, FormSource, Gender, ImperativeCell, ImperativeFormation,
    ImperfectFormation, InflectionError, InflectionWarning, LParticipleCell, NounCell, NounClass,
    Number, NumberRestriction, PartOfSpeech, ParticipleCell, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, RuleId, VerbClass,
};

fn only_id(lemma: &str, part_of_speech: PartOfSpeech) -> String {
    let candidates = old_church_slavonic::lookup(lemma, part_of_speech).expect("valid lookup");
    assert_eq!(candidates.len(), 1, "fixture must remain unambiguous");
    candidates[0].id.clone()
}

#[test]
fn curated_revision_pinned_goldens_match_the_public_dictionary() {
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
                old_church_slavonic::dictionary_form_by_id(&candidate.id, columns[2])
                    .ok()
                    .filter(|forms| {
                        forms
                            .variants
                            .iter()
                            .map(|variant| variant.text.as_str())
                            .eq(expected.iter().copied())
                    })
                    .map(|_| candidate.id)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "golden row {} must identify exactly one source record: {matching:?}",
            line_index + 1
        );
        let (source, revision) = columns[4]
            .split_once('@')
            .expect("golden source has an immutable revision anchor");
        assert!(!source.is_empty() && !revision.is_empty());
    }
}

#[test]
fn dictionary_noun_cell_and_paradigm_share_the_resolver() {
    let id = only_id("обѣдъ", PartOfSpeech::Noun);
    let paradigm = old_church_slavonic::noun_paradigm(&id).expect("known noun");
    for number in Number::ALL {
        for case in Case::ALL {
            let cell = NounCell { case, number };
            assert_eq!(
                paradigm.get(cell).expect("enumerated cell"),
                &old_church_slavonic::noun_by_id(&id, cell)
            );
        }
    }
    let dual_dative = old_church_slavonic::noun(
        "обѣдъ",
        NounCell {
            case: Case::Dative,
            number: Number::Dual,
        },
    )
    .expect("dictionary cell");
    assert_eq!(dual_dative.variants[0].text, "обѣдома");
    assert_eq!(dual_dative.source, FormSource::DictionaryTable);
}

#[test]
fn adjective_short_and_long_are_separate_cells() {
    let common = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Short,
    };
    let short = old_church_slavonic::adjective("добръ", common).expect("short table");
    let long = old_church_slavonic::adjective(
        "добръ",
        AdjectiveCell {
            form: AdjectiveForm::Long,
            ..common
        },
    )
    .expect("long table");
    assert_eq!(short.variants[0].text, "добръ");
    assert_eq!(long.variants[0].text, "добрꙑи");

    let uppercase_oov = old_church_slavonic::adjective("КОДЕКСОВЪ", common)
        .expect("uppercase OOV uses the normalized lookup spelling");
    assert_eq!(uppercase_oov.lemma, "кодексовъ");
    assert_eq!(uppercase_oov.variants[0].text, "кодексовъ");
    assert!(matches!(
        uppercase_oov.source,
        FormSource::OovPrediction { .. }
    ));
}

#[test]
fn variants_and_aorists_keep_source_order() {
    let variants = old_church_slavonic::noun(
        "аблань",
        NounCell {
            case: Case::Genitive,
            number: Number::Dual,
        },
    )
    .expect("variant cell");
    assert_eq!(
        variants
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["абланью", "абланию"]
    );

    let aorist = old_church_slavonic::finite_verb(
        "бꙑти",
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("listed aorists");
    assert_eq!(
        aorist
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["бѣхъ", "бꙑхъ"]
    );
}

#[test]
fn safe_dictionary_verb_components_have_typed_apis() {
    let present = old_church_slavonic::finite_verb(
        "благословити",
        FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        },
    )
    .expect("present");
    assert_eq!(present.variants[0].text, "благословлѭ");

    let imperative = old_church_slavonic::imperative(
        "благословити",
        ImperativeCell {
            person: Person::Second,
            number: Number::Singular,
        },
    )
    .expect("imperative");
    assert_eq!(imperative.variants[0].text, "благослови");

    let l_form = old_church_slavonic::l_participle(
        "благословити",
        LParticipleCell {
            gender: Gender::Feminine,
            number: Number::Dual,
        },
    )
    .expect("l-participle");
    assert_eq!(l_form.variants[0].text, "благословилѣ");
    assert_eq!(
        old_church_slavonic::infinitive("благословити")
            .expect("infinitive")
            .variants[0]
            .text,
        "благословити"
    );
    assert_eq!(
        old_church_slavonic::supine("бости")
            .expect("root supine")
            .variants[0]
            .text,
        "бостъ"
    );
    assert_eq!(
        old_church_slavonic::verbal_noun("благословити")
            .expect("dictionary verbal noun")
            .variants[0]
            .text,
        "благословлѥниѥ"
    );

    for (lemma, expected) in [
        ("бости", "бодѫ"),
        ("гънати", "женѫ"),
        ("благословити", "благословлѭ"),
        ("блѣдѣти", "блѣждѫ"),
        ("боꙗти", "боѭ"),
        ("рещи", "рекѫ"),
    ] {
        assert_eq!(
            old_church_slavonic::finite_verb(
                lemma,
                FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::First,
                    number: Number::Singular,
                },
            )
            .expect("curated class/root present")
            .variants[0]
                .text,
            expected
        );
    }

    assert_eq!(
        old_church_slavonic::participle_citation("благословити", ParticipleKind::PastActive,)
            .expect("past active participle")
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["благословл҄ь", "благословивъ"]
    );
    assert_eq!(
        old_church_slavonic::participle_citation("благословити", ParticipleKind::PresentActive,)
            .expect("safely tagged present active participle")
            .variants[0]
            .text,
        "благословѧ"
    );
}

#[test]
fn explicit_verb_system_apis_expose_rules_traces_and_historical_cells() {
    let mut lexeme = VerbLexeme::new("нести", VerbClass::IA1);
    lexeme.stems.imperfect = Some("нес".to_string());
    lexeme.formations.imperfect = Some(ImperfectFormation::YatA);
    lexeme.stems.aorist = Some("рек".to_string());
    lexeme.formations.aorist = Some(AoristFormation::New);
    lexeme.stems.imperative = Some("нес".to_string());
    lexeme.formations.imperative = Some(ImperativeFormation::YatSeries);

    let imperfect = old_church_slavonic::finite_verb_with(
        &lexeme,
        FiniteVerbCell {
            tense: FiniteTense::Imperfect,
            person: Person::Third,
            number: Number::Dual,
        },
    )
    .expect("explicit imperfect");
    assert_eq!(imperfect.variants[0].text, "несѣашете");
    assert!(matches!(
        imperfect.source,
        FormSource::ExplicitMetadataRule {
            rule_id: RuleId::VerbImperfectYatA
        }
    ));
    assert_eq!(imperfect.trace.len(), 1);

    assert_eq!(
        old_church_slavonic::finite_verb_with(
            &lexeme,
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::Third,
                number: Number::Singular,
            },
        )
        .expect("explicit new aorist")
        .variants[0]
            .text,
        "рече"
    );
    assert_eq!(
        old_church_slavonic::imperative_with(
            &lexeme,
            ImperativeCell {
                person: Person::First,
                number: Number::Plural,
            },
        )
        .expect("explicit imperative")
        .variants[0]
            .text,
        "несѣмъ"
    );
    assert!(matches!(
        old_church_slavonic::imperative_with(
            &lexeme,
            ImperativeCell {
                person: Person::Third,
                number: Number::Plural,
            },
        ),
        Err(InflectionError::UnsupportedCell)
    ));

    lexeme.stems.present_active_participle = Some("нес".to_string());
    lexeme.formations.present_active_participle = Some(PresentActiveParticipleFormation::YushtHard);
    lexeme.stems.present_passive_participle = Some("нес".to_string());
    lexeme.formations.present_passive_participle = Some(PresentPassiveParticipleFormation::Om);
    lexeme.stems.past_active_participle = Some("нес".to_string());
    lexeme.formations.past_active_participle = Some(PastActiveParticipleFormation::Ush);
    lexeme.stems.past_passive_participle = Some("нес".to_string());
    lexeme.formations.past_passive_participle = Some(PastPassiveParticipleFormation::En);
    for kind in [
        ParticipleKind::PresentActive,
        ParticipleKind::PresentPassive,
        ParticipleKind::PastActive,
        ParticipleKind::PastPassive,
    ] {
        let result = old_church_slavonic::participle_with(
            &lexeme,
            ParticipleCell {
                kind,
                adjective: AdjectiveCell {
                    case: Case::Genitive,
                    number: Number::Dual,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                },
            },
        )
        .expect("all four productive participles");
        assert_eq!(result.trace.len(), 2);
        assert!(matches!(
            result.source,
            FormSource::ExplicitMetadataRule { .. }
        ));
    }
}

#[test]
fn closed_classes_retain_person_and_suppletion() {
    let first = old_church_slavonic::closed_class(
        "азъ",
        PartOfSpeech::Pronoun,
        ClosedClassCell {
            case: Case::Dative,
            number: Number::Singular,
            gender: None,
            person: Some(Person::First),
        },
    )
    .expect("first-person pronoun forms");
    assert_eq!(
        first
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["мьнѣ", "ми"]
    );

    let reflexive = old_church_slavonic::closed_class(
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
    assert_eq!(
        reflexive
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<Vec<_>>(),
        ["себѣ", "си"]
    );

    let numeral = old_church_slavonic::closed_class(
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
    assert_eq!(numeral.variants[0].text, "девѧти");

    let glagolitic_alias = old_church_slavonic::closed_class(
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
    assert!(glagolitic_alias.warnings.iter().any(|warning| matches!(
        warning,
        InflectionWarning::OrthographicAliasUsed { canonical } if canonical == "къто"
    )));
}

#[test]
fn ambiguity_and_unsupported_cells_are_typed() {
    assert!(matches!(
        old_church_slavonic::noun(
            "блѧдь",
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
            }
        ),
        Err(InflectionError::AmbiguousLexeme { .. })
    ));
    assert!(matches!(
        old_church_slavonic::finite_verb(
            "благословити",
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            }
        ),
        Err(InflectionError::UnsupportedCell)
    ));
    assert!(matches!(
        old_church_slavonic::dictionary_form_by_id("missing", "noun:nom:sg"),
        Err(InflectionError::UnknownLemma)
    ));
}

#[test]
fn source_backed_glagolitic_and_hostile_input_are_panic_free() {
    let glagolitic_id = old_church_slavonic::lookup("ⱁⰽⱁ", PartOfSpeech::Noun)
        .expect("Glagolitic lookup")
        .into_iter()
        .find(|candidate| candidate.lemma == "ⱁⰽⱁ")
        .expect("source-backed Glagolitic record")
        .id;
    let glagolitic = old_church_slavonic::noun_by_id(
        &glagolitic_id,
        NounCell {
            case: Case::Genitive,
            number: Number::Singular,
        },
    )
    .expect("source-backed Glagolitic paradigm");
    assert!(glagolitic.variants[0].text.contains('ⰵ'));

    for hostile in ["", "two words", ".", "\0", &"x".repeat(4_097)] {
        let result = std::panic::catch_unwind(|| exercise_public_surface(hostile));
        assert!(result.is_ok(), "public API panicked for {hostile:?}");
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
    let closed_cell = ClosedClassCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: None,
        person: None,
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
    verb_lexeme.stems.aorist = Some(hostile.to_string());

    let _ = old_church_slavonic::lookup(hostile, PartOfSpeech::Noun);
    let _ = old_church_slavonic::noun(hostile, noun_cell);
    let _ = old_church_slavonic::noun_by_id(hostile, noun_cell);
    let _ = old_church_slavonic::noun_paradigm(hostile);
    let _ = old_church_slavonic::noun_with(&noun_lexeme, noun_cell);
    let _ = old_church_slavonic::adjective(hostile, adjective_cell);
    let _ = old_church_slavonic::adjective_by_id(hostile, adjective_cell);
    let _ = old_church_slavonic::adjective_paradigm(hostile);
    let _ = old_church_slavonic::adjective_with(&adjective_lexeme, adjective_cell);
    let _ = old_church_slavonic::adjective_comparatives(hostile);
    let _ = old_church_slavonic::adjective_comparatives_by_id(hostile);
    let _ = old_church_slavonic::finite_verb(hostile, finite_cell);
    let _ = old_church_slavonic::finite_verb_by_id(hostile, finite_cell);
    let _ = old_church_slavonic::finite_verb_paradigm(hostile);
    let _ = old_church_slavonic::finite_verb_with(&verb_lexeme, finite_cell);
    let _ = old_church_slavonic::imperative(hostile, imperative_cell);
    let _ = old_church_slavonic::imperative_by_id(hostile, imperative_cell);
    let _ = old_church_slavonic::imperative_paradigm(hostile);
    let _ = old_church_slavonic::l_participle(hostile, l_cell);
    let _ = old_church_slavonic::l_participle_by_id(hostile, l_cell);
    let _ = old_church_slavonic::l_participle_paradigm(hostile);
    let _ = old_church_slavonic::l_participle_with(&verb_lexeme, l_cell);
    let _ = old_church_slavonic::participle(hostile, participle_cell);
    let _ = old_church_slavonic::participle_by_id(hostile, participle_cell);
    let _ = old_church_slavonic::participle_with(&verb_lexeme, participle_cell);
    let _ = old_church_slavonic::participle_paradigm(hostile, ParticipleKind::PresentActive);
    let _ = old_church_slavonic::participle_citation(hostile, ParticipleKind::PresentActive);
    let _ = old_church_slavonic::participle_citation_by_id(hostile, ParticipleKind::PresentActive);
    let _ = old_church_slavonic::infinitive(hostile);
    let _ = old_church_slavonic::infinitive_by_id(hostile);
    let _ = old_church_slavonic::infinitive_with(&verb_lexeme);
    let _ = old_church_slavonic::supine(hostile);
    let _ = old_church_slavonic::supine_by_id(hostile);
    let _ = old_church_slavonic::supine_with(&verb_lexeme);
    let _ = old_church_slavonic::verbal_noun(hostile);
    let _ = old_church_slavonic::verbal_noun_by_id(hostile);
    let _ = old_church_slavonic::closed_class(hostile, PartOfSpeech::Pronoun, closed_cell);
    let _ = old_church_slavonic::closed_class_by_id(hostile, PartOfSpeech::Pronoun, closed_cell);
    let _ = old_church_slavonic::dictionary_form_by_id(hostile, hostile);
    let _ = old_church_slavonic::dictionary_paradigm_by_id(hostile);
}
