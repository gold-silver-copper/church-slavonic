#![allow(clippy::unwrap_used)]
use church_slavonic::{
    document::AnalysisDocument,
    matching::MatchPolicy,
    witness::{SourceAddress, Witness},
    *,
};
use church_slavonic_tools::analysis_document::{self as codec, Record, RecordError};

fn lexicon() -> Lexicon {
    let closed =
        "one.x\tже\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\ntwo.x\tже\tx\t-\t-\tconj\t-\t-\t-\t-\t-\t-\n";
    let mut entries = lexicon::parse(closed, Pos::Closed).unwrap();
    entries.extend(
        lexicon::parse(
            "test.n\tра́бъ\tn\tm\tinan\tN1t\ta\t-\t-\t-\t-\t-\n",
            Pos::Noun,
        )
        .unwrap(),
    );
    Lexicon::try_from_lexemes(Recension::Synodal, entries).unwrap()
}

#[test]
fn ambiguity_and_every_source_byte_survive_serialization() {
    let lex = lexicon();
    let source = "  же,\tра́бъ [unknown]\n";
    let doc = AnalysisDocument::analyze(
        &lex,
        Witness::new(
            source,
            Some(SourceAddress {
                witness: "fixture".into(),
                document: "test".into(),
                unit: "1".into(),
            }),
        ),
        MatchPolicy::Exact,
    );
    let json = codec::to_json(&doc).unwrap();
    let restored = codec::from_json(json.as_bytes(), &lex).unwrap();
    assert_eq!(doc.witness(), restored.witness());
    assert_eq!(doc.segments(), restored.segments());
    assert_eq!(
        (0..restored.segments().len())
            .map(|i| restored.span(i).unwrap().text().to_string())
            .collect::<String>(),
        source
    );
    let closed = restored
        .segments()
        .iter()
        .find(|s| s.candidates().len() == 2 && s.candidates()[0].cell() == Cell::Word)
        .unwrap();
    let ids: std::collections::BTreeSet<_> =
        closed.candidates().iter().map(|c| c.lexeme_id()).collect();
    assert_eq!(ids, ["one.x", "two.x"].into_iter().collect());
    assert!(
        restored
            .segments()
            .iter()
            .any(|s| s.candidates().len() == 2 && s.candidates()[0].cell() != Cell::Word)
    );
    assert_eq!(codec::to_json(&restored).unwrap(), json);
}

#[test]
fn source_candidates_survive_legacy_tree_mutation() {
    let lex = lexicon();
    let mut sentence = sentence::Sentence::parse(&lex, "же ра́бъ");
    let before = Record::from_document(&sentence.analysis_document(MatchPolicy::Exact));
    *sentence.tree_mut() = sentence::Node::W {
        surface: "replaced".into(),
        notes: Vec::new(),
    };
    let after = Record::from_document(&sentence.analysis_document(MatchPolicy::Exact));
    assert_eq!(before, after);
}

#[test]
fn changed_candidates_or_spans_are_rejected_and_order_is_not_a_choice() {
    let lex = lexicon();
    let doc = AnalysisDocument::analyze(&lex, Witness::new("же", None), MatchPolicy::Exact);
    let original = Record::from_document(&doc);
    let mut record = original.clone();
    record.segments[0].candidates.reverse();
    assert!(record.resolve(&lex).is_ok());
    let mut record = original.clone();
    record.segments[0].candidates.pop();
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::AnalysisMismatch)
    ));
    let mut record = original.clone();
    record.segments[0].candidates[0].lexeme = "fabricated".into();
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::AnalysisMismatch)
    ));
    let mut record = original.clone();
    record.segments[0].candidates[0].alternative = Some(usize::MAX);
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::AnalysisMismatch)
    ));
    let mut record = original.clone();
    record.segments[0].end = 1;
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::InvalidSpans)
    ));
    let empty = Lexicon::try_from_lexemes(Recension::Synodal, Vec::new()).unwrap();
    assert!(matches!(
        original.resolve(&empty),
        Err(RecordError::AnalysisMismatch)
    ));
}

#[test]
fn transformed_match_explanations_are_preserved_and_checked() {
    let lex = lexicon();
    let doc = AnalysisDocument::analyze(
        &lex,
        Witness::new("РАБЪ", None),
        MatchPolicy::AccentInsensitive,
    );
    let mut record = Record::from_document(&doc);
    assert!(!record.segments[0].candidates.is_empty());
    assert!(!record.segments[0].candidates[0].query_steps.is_empty());
    assert!(!record.segments[0].candidates[0].generated_steps.is_empty());
    assert!(record.resolve(&lex).is_ok());
    record.segments[0].candidates[0].query_steps.clear();
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::AnalysisMismatch)
    ));
}

#[test]
fn malformed_serialization_is_not_an_empty_success() {
    let lex = lexicon();
    for bytes in [
        b"".as_slice(),
        b"\xff",
        b"{}",
        b"{\"version\":1,\"version\":1}",
    ] {
        assert!(codec::from_json(bytes, &lex).is_err());
    }
    let bytes = vec![b' '; codec::MAX_RECORD_BYTES + 1];
    assert!(matches!(
        codec::from_json(&bytes, &lex),
        Err(RecordError::RecordTooLarge)
    ));
    let empty = AnalysisDocument::analyze(&lex, Witness::new("", None), MatchPolicy::Exact);
    let mut record = Record::from_document(&empty);
    assert!(record.resolve(&lex).is_ok());
    record.version = u32::MAX;
    assert!(matches!(
        record.resolve(&lex),
        Err(RecordError::UnsupportedVersion(_))
    ));
}
