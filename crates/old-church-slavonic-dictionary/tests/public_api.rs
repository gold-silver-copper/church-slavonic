use old_church_slavonic_dictionary::{
    MatchKind, SOURCE_MANIFEST, SearchOptions, analyze_dictionary_form, lookup, search,
    sense_by_id, sense_count, validate_vocabulary_tsv,
};

#[test]
fn english_game_concepts_find_source_backed_ocs_senses() {
    let gold = search("gold coin", &SearchOptions::default()).expect("gold search");
    assert_eq!(gold[0].sense().lemma(), "златикъ");
    assert_eq!(gold[0].matched_on(), MatchKind::ExactGloss);

    let food = search("food", &SearchOptions::default()).expect("food search");
    assert!(food.iter().any(|result| result.sense().lemma() == "пищꙗ"));
}

#[test]
fn lemma_lookup_preserves_distinct_senses_and_examples() {
    let senses = lookup("чьсти").expect("read lookup");
    assert!(senses.iter().any(|sense| sense.glosses() == ["to read"]));
    let read = senses
        .iter()
        .find(|sense| sense.glosses() == ["to read"])
        .expect("read sense");
    assert!(
        read.examples()
            .any(|example| example.text().contains("чьтеші"))
    );
    assert_eq!(
        sense_by_id(read.id()).expect("sense by id").lemma(),
        "чьсти"
    );
}

#[test]
fn vocabulary_manifest_requires_evidence_and_semantic_alignment() {
    let gold = search("gold coin", &SearchOptions::default()).expect("gold search")[0].sense();
    let body = format!(
        "concept\tlemma\tpart_of_speech\tsense_id\tstatus\tnotes\n\
         gold coin\tзлатикъ\tnoun\t{}\tattested\t\n",
        gold.id()
    );
    let report = validate_vocabulary_tsv(&body);
    assert!(report.is_ok(), "{:?}", report.issues);
}

#[test]
fn generated_dictionary_has_real_coverage() {
    assert!(sense_count() > 5_000);
    assert!(SOURCE_MANIFEST.contains("fb20336e716d8f29d0c53bb4cc32f350"));
}

#[test]
fn reverse_dictionary_analysis_retains_identity_and_feature() {
    let analyses = analyze_dictionary_form("златици").expect("valid dictionary surface");
    assert!(analyses.iter().any(|analysis| {
        analysis.lemma == "златикъ"
            && analysis.part_of_speech == old_church_slavonic::PartOfSpeech::Noun
            && analysis.feature == "noun:nom:pl"
    }));

    let ambiguous = analyze_dictionary_form("града").expect("ambiguous dictionary surface");
    assert!(
        ambiguous
            .iter()
            .any(|analysis| analysis.lemma == "градъ" && analysis.feature == "noun:gen:sg")
    );
}

#[test]
fn representative_game_vocabulary_fixture_is_source_accountable() {
    let fixture = include_str!("fixtures/game-vocabulary.tsv");
    let report = validate_vocabulary_tsv(fixture);
    assert!(report.is_ok(), "{:?}", report.issues);
    assert_eq!(report.rows, 5);
    assert_eq!(report.attested, 3);
    assert_eq!(report.thematic, 1);
    assert_eq!(report.proper_names, 1);
}

#[test]
fn proper_name_filter_accepts_human_and_wire_spellings() {
    for spelling in ["proper-name", "proper name", "proper noun"] {
        let results = search(
            "Rome",
            &SearchOptions {
                part_of_speech: Some(spelling.to_string()),
                ..SearchOptions::default()
            },
        )
        .expect("proper-name search");
        assert!(!results.is_empty(), "no proper-name result for {spelling}");
        assert!(
            results
                .iter()
                .all(|result| result.sense().part_of_speech() == "proper-name")
        );
    }
}
