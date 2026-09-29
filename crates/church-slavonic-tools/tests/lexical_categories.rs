#![allow(clippy::unwrap_used)]
use church_slavonic::{
    document::AnalysisDocument, matching::MatchPolicy, morphology::*, witness::Witness,
};
use church_slavonic_tools::{
    analysis_document::{self as codec, Record},
    model_artifact::{self, Artifact, Fixture},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/synodal-mudr-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(MODEL).unwrap().input
}
fn spelling() -> OrthographyId {
    OrthographyId("o-syn-mudr".into())
}

#[test]
fn adjective_and_adverb_keep_distinct_lexical_readings_and_a_sourced_relation() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    for (bytes, id, count) in [
        (
            include_bytes!("../../../data/rewrite/synodal-mudr-adjective.json").as_slice(),
            "l-000004",
            3,
        ),
        (
            include_bytes!("../../../data/rewrite/synodal-mudr-adverb.json").as_slice(),
            "l-000005",
            2,
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
            (count, count, count)
        );
        assert!(report.failures.is_empty());
    }
    let adjective = model.lexeme(&LexemeId("l-000004".into())).unwrap();
    assert_eq!(adjective.relations[0].kind, RelationKind::DerivedAdverb);
    let adverb = model.lexeme(&adjective.relations[0].target).unwrap();
    assert_eq!(adverb.categories[0].category, LexicalCategory::Adverb);
    assert_eq!(
        model
            .evidence(&adjective.relations[0].evidence[0])
            .unwrap()
            .review,
        ReviewStatus::Unverified
    );

    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("  мꙋ́дрѣ!", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let record = Record::from_document(&doc);
    assert_eq!(record.version, 9);
    let candidates = &record
        .segments
        .iter()
        .find(|s| !s.candidates.is_empty())
        .unwrap()
        .candidates;
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().any(|c| c.cell == "short.pos.m.sg.loc"
        && c.categories.as_ref().unwrap()[0].category == LexicalCategory::Adjective));
    assert!(candidates.iter().any(|c| c.cell == "word"
        && c.categories.as_ref().unwrap()[0].category == LexicalCategory::Adverb));
    let json = codec::to_json(&doc).unwrap();
    let back = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(back.witness().reproduce(), "  мꙋ́дрѣ!");
    let mut changed = record.clone();
    changed
        .segments
        .iter_mut()
        .find(|s| !s.candidates.is_empty())
        .unwrap()
        .candidates[0]
        .categories
        .as_mut()
        .unwrap()[0]
        .category = LexicalCategory::Particle;
    assert!(changed.resolve_model(&model).is_err());
}

#[test]
fn classifications_require_evidence_and_can_retain_competing_source_claims() {
    let mut data = input();
    data.lexemes[0].categories.clear();
    assert!(Model::build(data).is_err());
    let mut data = input();
    data.lexemes[0].categories[0].evidence = vec![EvidenceId("missing".into())];
    assert!(matches!(
        Model::build(data),
        Err(ModelError::MissingReference(_))
    ));
    let mut data = input();
    let duplicate = data.lexemes[0].categories[0].clone();
    data.lexemes[0].categories.push(duplicate);
    assert!(matches!(Model::build(data), Err(ModelError::Duplicate(_))));

    // Constructed classification disagreement, not a linguistic claim about this word.
    let mut data = input();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("test-category-disagreement".into());
    evidence.source = "test:constructed".into();
    evidence.claim = "Constructed competing classification to test preservation".into();
    evidence.review = ReviewStatus::Unverified;
    data.lexemes[1].categories.push(CategoryAssertion {
        category: LexicalCategory::Particle,
        evidence: vec![evidence.id.clone()],
    });
    data.evidence.push(evidence);
    let model = Model::build(data).unwrap();
    let result = model
        .analyze("мꙋ́дрѡ", &spelling(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].lexeme.categories.len(), 2);
}
