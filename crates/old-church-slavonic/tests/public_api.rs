use old_church_slavonic::advanced::cells::{
    AdjectiveCell, AdjectiveForm, ClosedClassCell, FiniteVerbCell, ImperativeCell, LParticipleCell,
    NounCell, ParticipleCell,
};
use old_church_slavonic::advanced::metadata as api_metadata;
use old_church_slavonic::advanced::raw_features;
use old_church_slavonic::advanced::rules::{
    AdjectiveClass, AdjectiveLexeme, AoristFormation, ImperativeFormation, NounClass, NounLexeme,
    NumberRestriction, VerbClass, VerbLexeme, adjective_with, finite_verb_with, imperative_with,
    noun_with, participle_with,
};
use old_church_slavonic::advanced::{by_id, participle_form};
use old_church_slavonic::trace::{MetadataField, MetadataProvenance, RuleId};
use old_church_slavonic::{
    Adjective, Animacy, Case, FiniteTense, FormSource, Gender, InflectionError, InflectionWarning,
    Noun, Number, PartOfSpeech, ParticipleKind, Person, Verb, adjective, adjective_paradigm,
    aorist, finite_verb, finite_verb_paradigm, imperative, imperative_paradigm, imperfect,
    infinitive, l_participle, l_participle_paradigm, noun, noun_paradigm, participle_paradigm,
    past_active_participle, short_adjective, supine, verb, verb_paradigm,
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
        adjective(
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
        verb("благословити", Person::First, Number::Singular)
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
    let be = Verb::new("бꙑти").expect("unique verb");
    assert_eq!(
        be.aorist(Person::First, Number::Singular)
            .expect("handle aorists"),
        aorists
    );
    assert_eq!(
        by_id::finite_verb_by_id(
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
            .get(FiniteTense::Aorist, Person::First, Number::Singular,)
            .expect("paradigm aorist cell")
            .as_ref()
            .expect("successful paradigm aorists"),
        &aorists
    );

    let consumed = noun_variants.into_primary_text();
    assert_eq!(consumed, "абланью");
}

#[test]
fn resolved_noun_and_adjective_handles_share_free_and_by_id_paths() {
    let meal = Noun::new("обѣдъ").expect("unique noun");
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

    let good = Adjective::new("добръ").expect("unique adjective");
    let long = good.long(
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    assert_eq!(
        long,
        adjective(
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
            AdjectiveForm::Long => adjective(
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
    let bless = Verb::new("благословити").expect("unique verb");
    assert_eq!(Verb::from_id(bless.id()).expect("valid verb ID"), bless);
    assert_eq!(
        bless.present(Person::First, Number::Singular),
        verb("благословити", Person::First, Number::Singular)
    );
    assert_eq!(
        verb_paradigm("благословити").expect("present table"),
        bless.paradigm()
    );
    assert_eq!(
        by_id::verb_paradigm_by_id(bless.id()).expect("present table by ID"),
        bless.paradigm()
    );
    for outcome in bless.paradigm() {
        assert_eq!(outcome.cell.tense, FiniteTense::Present);
        assert_eq!(
            outcome.result,
            verb("благословити", outcome.cell.person, outcome.cell.number)
        );
    }

    let finite = finite_verb_paradigm("благословити").expect("finite paradigm");
    assert_eq!(finite.len(), 27);
    assert_eq!(finite, bless.finite_paradigm());
    for outcome in &finite {
        assert_eq!(
            &outcome.result,
            &finite_verb(
                "благословити",
                outcome.cell.tense,
                outcome.cell.person,
                outcome.cell.number,
            )
        );
        assert_eq!(
            &outcome.result,
            &by_id::finite_verb_by_id(bless.id(), outcome.cell)
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
    let handle_participle = Verb::new("благословити")
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

    explicit.formations.aorist = Some(AoristFormation::SigmaticPrimary);
    assert!(matches!(
        finite_verb_with(
            &explicit,
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            },
        ),
        Err(InflectionError::UnsupportedFormation { .. })
    ));
}

#[test]
fn ambiguity_unknown_missing_metadata_and_invalid_cells_remain_distinct() {
    assert!(matches!(
        noun("блѧдь", Case::Nominative, Number::Singular),
        Err(InflectionError::AmbiguousLexeme { .. })
    ));
    assert!(matches!(
        verb("несуществовати", Person::Third, Number::Singular),
        Err(InflectionError::UnknownLemma)
    ));
    assert!(matches!(
        finite_verb(
            "благословити",
            FiniteTense::Aorist,
            Person::First,
            Number::Singular,
        ),
        Err(InflectionError::MissingLexicalMetadata { needed })
            if needed == vec![MetadataField::AoristStem, MetadataField::AoristFormation]
    ));
    assert!(matches!(
        imperative("благословити", Person::Third, Number::Dual),
        Err(InflectionError::HistoricallyInvalidCell)
    ));
    let id = only_id("благословити", PartOfSpeech::Verb);
    assert!(matches!(
        raw_features::form_by_id(&id, "verb:finite:future:1:sg"),
        Err(InflectionError::InvalidInput { .. })
    ));
}

#[test]
fn reviewed_override_follows_exact_table_and_keeps_authority() {
    let id = only_id("бꙑти", PartOfSpeech::Verb);
    let by_id = by_id::finite_verb_by_id(
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
    let _ = adjective(
        hostile,
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    );
    let _ = verb(hostile, Person::First, Number::Singular);
    let _ = imperative(hostile, Person::Second, Number::Singular);
    let _ = Noun::new(hostile);
    let _ = Adjective::new(hostile);
    let _ = Verb::new(hostile);
    let _ = noun_paradigm(hostile);
    let _ = finite_verb_paradigm(hostile);

    let _ = by_id::noun_by_id(hostile, noun_cell);
    let _ = by_id::adjective_by_id(hostile, adjective_cell);
    let _ = by_id::finite_verb_by_id(hostile, finite_cell);
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
