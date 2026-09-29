#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Gender, document::AnalysisDocument, matching::MatchPolicy, morphology::*, witness::Witness,
};
use church_slavonic_tools::{
    analysis_document::{self as codec, Record},
    model_artifact::{self, Artifact, Fixture},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/ocs-noun-numeral-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(MODEL).unwrap().input
}
fn spelling() -> OrthographyId {
    OrthographyId("o-lrc-noun-numerals".into())
}

#[test]
fn listed_numerals_keep_lexical_gender_separate_from_case_and_number() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    for (id, bytes, n) in [
        (
            "l-000014",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000014.json").as_slice(),
            2,
        ),
        (
            "l-000015",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000015.json").as_slice(),
            2,
        ),
        (
            "l-000016",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000016.json").as_slice(),
            2,
        ),
        (
            "l-000017",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000017.json").as_slice(),
            2,
        ),
        (
            "l-000018",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000018.json").as_slice(),
            2,
        ),
        (
            "l-000019",
            include_bytes!("../../../data/rewrite/ocs-noun-numeral-l-000019.json").as_slice(),
            3,
        ),
    ] {
        let fixture: Fixture = serde_json::from_slice(bytes).unwrap();
        let report =
            model_artifact::check_fixture(&model, fixture, &LexemeId(id.into()), &spelling())
                .unwrap();
        assert_eq!(
            (
                report.expected_forms,
                report.exact_generated,
                report.joint_analysis_retained
            ),
            (n, n, n)
        );
        assert!(report.failures.is_empty());
    }
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("пѧти", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let candidates = doc.segments()[0].candidates();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].cell().name(), "gen.sg");
    assert_eq!(candidates[0].cell().gender(), None);
    assert_eq!(
        candidates[0].lexical_genders().unwrap()[0].gender,
        Gender::Feminine
    );
    assert!(
        candidates[0]
            .categories()
            .unwrap()
            .iter()
            .any(|c| c.category == LexicalCategory::Numeral)
    );
}

#[test]
fn missing_evidence_and_duplicates_fail_but_competing_assertions_survive() {
    let mut data = input();
    data.lexemes[0].genders[0].evidence.clear();
    assert!(Model::build(data).is_err());
    let mut data = input();
    let duplicate = data.lexemes[0].genders[0].clone();
    data.lexemes[0].genders.push(duplicate);
    assert!(Model::build(data).is_err());
    let mut data = input();
    data.lexemes[0].genders.clear();
    assert!(
        Model::build(data)
            .unwrap()
            .lexeme(&LexemeId("l-000014".into()))
            .unwrap()
            .genders
            .is_empty()
    );
    let mut data = input();
    let mut assertion = data.evidence[0].clone();
    assertion.id = EvidenceId("e-constructed-conflict".into());
    assertion.review = ReviewStatus::Unverified;
    assertion.claim =
        "Constructed conflicting assertion for software control; not a source transcription."
            .into();
    data.lexemes[0].genders.push(GenderAssertion {
        gender: Gender::Masculine,
        evidence: vec![assertion.id.clone()],
    });
    data.evidence.push(assertion);
    let model = Model::build(data).unwrap();
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("пѧти", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let restored = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(
        restored.segments()[0].candidates()[0]
            .lexical_genders()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn gender_and_document_version_cannot_be_relabelled_on_reload() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("пѧти", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let record = Record::from_document(&doc);
    assert_eq!(record.version, 9);
    let mut changed = record.clone();
    changed.segments[0].candidates[0]
        .lexical_genders
        .as_mut()
        .unwrap()[0]
        .gender = Gender::Masculine;
    assert!(changed.resolve_model(&model).is_err());
    let mut old = record;
    old.version = 8;
    assert!(matches!(
        old.resolve_model(&model),
        Err(codec::RecordError::UnsupportedVersion(8))
    ));
    let invalid = std::str::from_utf8(MODEL)
        .unwrap()
        .replace("\"gender\": \"f\"", "\"gender\": \"invalid\"");
    assert!(model_artifact::from_bytes(invalid.as_bytes()).is_err());
}
