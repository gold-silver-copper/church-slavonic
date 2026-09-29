#![allow(clippy::unwrap_used)]
use church_slavonic_tools::sources::{observations::RecordKind, proiel::append_xml, ud::Corpus};
#[test]
fn xml_quotes_entities_and_missing_annotations_keep_original_evidence() {
    let xml = "<?xml version='1.0'?><proiel><source language='chu'><sentence id='s'><token id='1' form='&#x433;радъ' lemma='ГРАДЪ' part-of-speech='Nb' morphology='-s---mn--i'/><token id='2' form='&amp;lt;'/><token id='3' empty-token-sort='pro'/></sentence></source></proiel>";
    let mut corpus = Corpus::default();
    append_xml(&mut corpus, "fixture.xml".into(), xml.into()).unwrap();
    assert_eq!(corpus.tokens, 2);
    assert_eq!(corpus.sentences[0][0].surface, "градъ");
    assert_eq!(corpus.sentences[0][1].surface, "&lt;");
    assert_eq!(corpus.skipped["missing token annotation"], 1);
    let records = corpus.observations.records();
    assert_eq!(
        records
            .iter()
            .filter(|r| r.kind == RecordKind::XmlToken)
            .count(),
        3
    );
    assert!(
        records
            .iter()
            .any(|r| r.mapping_issue.as_deref() == Some("no source surface"))
    );
    assert_eq!(
        records
            .iter()
            .map(|r| corpus.observations.raw(r).unwrap())
            .collect::<String>(),
        xml
    );
    let first = &records[corpus.sentences[0][0].source_record.unwrap()];
    assert!(corpus.observations.raw(first).unwrap().contains("&#x433;"));
    assert_eq!(first.mapped_slots, vec![0]);
}
#[test]
fn language_and_sentence_boundaries_are_structural_and_other_sources_are_visible() {
    let mut corpus = Corpus::default();
    let xml = "<proiel><source language='lat'><sentence><token form='alien'/></sentence></source><source language='chu'><sentence><token form='a'/></sentence><sentence><token form='b'/></sentence></source></proiel>";
    append_xml(&mut corpus, "mixed.xml".into(), xml.into()).unwrap();
    assert_eq!((corpus.tokens, corpus.sentences.len()), (2, 2));
    assert!(
        corpus
            .observations
            .records()
            .iter()
            .any(|r| r.kind == RecordKind::OtherLanguageToken)
    );
    append_xml(
        &mut corpus,
        "latin.xml".into(),
        "<proiel><source language='lat'/></proiel>".into(),
    )
    .unwrap();
    assert_eq!(corpus.excluded_sources.len(), 1);
    assert_eq!(corpus.observations.documents().len(), 1);
}
#[test]
fn malformed_xml_duplicate_attributes_and_dtds_fail_without_partial_append() {
    let mut corpus = Corpus::default();
    let good =
        "<proiel><source language='chu'><sentence><token form='a'/></sentence></source></proiel>";
    append_xml(&mut corpus, "good".into(), good.into()).unwrap();
    for bad in [
        "<proiel/><proiel/>",
        "<wrong/>",
        "<proiel><source language='chu'><sentence><token><token form='a'/></token></sentence></source></proiel>",
        "<proiel><source language='chu'><sentence><token form='a'/>",
        "<proiel><source language='chu'><sentence><token form='a' form='b'/></sentence></source></proiel>",
        "<!DOCTYPE proiel SYSTEM 'file:///no-such-file'><proiel/>",
        "<proiel><source language='chu'><token form='a'/></source></proiel>",
    ] {
        assert!(append_xml(&mut corpus, "bad".into(), bad.into()).is_err());
        assert_eq!(
            (
                corpus.tokens,
                corpus.sentences.len(),
                corpus.observations.documents().len()
            ),
            (1, 1, 1)
        );
    }
}
