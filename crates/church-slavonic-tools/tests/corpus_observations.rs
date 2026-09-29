#![allow(clippy::unwrap_used)]
use church_slavonic_tools::sources::{
    observations::RecordKind,
    ud::{Corpus, append_conllu},
};
#[test]
fn originals_reconstruct_exactly_and_unsupported_labels_survive() {
    let text = "# sent_id = test\r\n1\tГРАДЪ\tГРАДЪ\tNOUN\tNb\tCase=Nom|Number=Sing\t0\troot\t_\t_\r\n2\tи\tи\tCCONJ\tC-\t_\t1\tcc\t_\tSpaceAfter=No\r\n\r\n";
    let mut corpus = Corpus::default();
    append_conllu(&mut corpus, "fixture.conllu".into(), text.into()).unwrap();
    let archive = &corpus.observations;
    let reproduced: String = archive
        .records()
        .iter()
        .map(|r| archive.raw(r).unwrap())
        .collect();
    assert_eq!(reproduced, text);
    assert_eq!(corpus.tokens, 2);
    assert_eq!(corpus.slots[0].lemma, "градъ");
    let noun = &archive.records()[corpus.sentences[0][0].source_record.unwrap()];
    assert!(archive.raw(noun).unwrap().contains("ГРАДЪ\tГРАДЪ"));
    let closed = &archive.records()[corpus.sentences[0][1].source_record.unwrap()];
    assert!(closed.mapped_slots.is_empty());
    assert_eq!(
        closed.mapping_issue.as_deref(),
        Some("part of speech outside the lexicon")
    );
    let mut bytes = Vec::new();
    corpus.write_observations(&mut bytes).unwrap();
    let rows: Vec<serde_json::Value> = String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let exported: String = rows
        .iter()
        .filter_map(|r| r.get("raw").and_then(|s| s.as_str()))
        .collect();
    assert_eq!(exported, text);
    assert!(rows.iter().any(|r| r["type"] == "mapped_slot"));
}
#[test]
fn multiword_empty_malformed_duplicate_and_bad_feature_records_are_accounted() {
    let text = "1-2\tab\t_\t_\t_\t_\t_\t_\t_\t_\n0.1\tx\tx\tX\t_\t_\t_\t_\t_\t_\nnot a row\n1\tградъ\tградъ\tNOUN\t_\tCase=Nom|Case=Acc|Number=Sing\t0\troot\t_\t_\n1\tградъ\tградъ\tNOUN\t_\tCase=Nom|Number=Sing\t0\troot\t_\t_\n";
    let mut corpus = Corpus::default();
    append_conllu(&mut corpus, "fixture".into(), text.into()).unwrap();
    let records = corpus.observations.records();
    assert_eq!(
        records.iter().map(|r| r.kind).collect::<Vec<_>>(),
        vec![
            RecordKind::Multiword,
            RecordKind::EmptyNode,
            RecordKind::Malformed,
            RecordKind::Word,
            RecordKind::Word
        ]
    );
    assert_eq!(corpus.tokens, 2);
    assert!(corpus.slots.is_empty());
    assert_eq!(
        records[3].mapping_issue.as_deref(),
        Some("invalid feature field")
    );
    assert_eq!(
        records[4].mapping_issue.as_deref(),
        Some("duplicate token ID")
    );
    assert_eq!(records.last().unwrap().end, text.len());
}
#[test]
fn appended_documents_keep_distinct_source_and_slot_addresses() {
    let text = "1\tградъ\tградъ\tNOUN\t_\tCase=Nom|Number=Sing\t0\troot\t_\t_";
    let mut corpus = Corpus::default();
    for path in ["one", "two"] {
        append_conllu(&mut corpus, path.into(), text.into()).unwrap();
    }
    assert_eq!(corpus.observations.documents().len(), 2);
    assert_eq!(corpus.observations.records()[1].source, 1);
    assert_eq!(corpus.observations.records()[1].mapped_slots, vec![1]);
    assert_eq!(corpus.sentences[1][0].source_record, Some(1));
    assert!(Corpus::default().write_observations(Vec::new()).is_err());
}

#[test]
fn rejected_append_preserves_existing_corpus_and_missing_mapping_is_explicit() {
    let mut corpus = Corpus::default();
    let text = "1\tградъ\tградъ\tNOUN\t_\tCase=Nom|Number=Sing\t0\troot\t_\t_\n";
    append_conllu(&mut corpus, "one".into(), text.into()).unwrap();
    let oversized = format!("{text}{}", "x".repeat(65537));
    assert!(append_conllu(&mut corpus, "bad".into(), oversized).is_err());
    assert_eq!(
        (
            corpus.tokens,
            corpus.slots.len(),
            corpus.sentences.len(),
            corpus.observations.documents().len()
        ),
        (1, 1, 1, 1)
    );
    let adjective = "1\tдобръ\tдобръ\tADJ\t_\tNumber=Sing\t0\troot\t_\t_\n";
    append_conllu(&mut corpus, "adj".into(), adjective.into()).unwrap();
    let record = corpus.observations.records().last().unwrap();
    assert!(record.mapped_slots.is_empty());
    assert!(record.mapping_issue.is_some());
}
