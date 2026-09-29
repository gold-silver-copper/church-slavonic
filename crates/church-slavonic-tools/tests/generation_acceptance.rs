#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Case, Cell, Number, document::AnalysisDocument, matching::MatchPolicy, morphology::*,
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec,
    model_artifact::{self, Artifact},
};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-stem-replacement-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn id() -> LexemeId {
    LexemeId("l-000020".into())
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-otrok".into())
}
fn cell() -> Cell {
    Cell::noun(Case::Nominative, Number::Plural)
}
fn forms(model: &Model, cell: Cell) -> Vec<Derivation> {
    match model.generate(&id(), cell, &profile()).unwrap() {
        Availability::Licensed(forms) | Availability::Partial { forms, .. } => forms,
        other => panic!("expected model-relative forms, got {other:?}"),
    }
}
fn decision(model: &Model, cell: Cell) -> GenerationAcceptance {
    GenerationAcceptance {
        lexeme: id(),
        cell,
        orthography: profile(),
        claim_sha256: forms(model, cell)[0].claim_sha256.clone(),
        decision_id: "constructed-decision".into(),
        method: "Constructed API control, not linguistic expert review".into(),
    }
}

#[test]
fn reported_or_verified_citations_do_not_accept_positive_attachments() {
    let mut data = input();
    for e in &mut data.evidence {
        e.review = ReviewStatus::ExpertReviewed {
            reviewer: "untrusted label".into(),
        };
    }
    let verified = data.evidence.clone();
    let model =
        Model::build_with_verified_evidence(data, ModelLimits::default(), verified).unwrap();
    let form = forms(&model, cell()).remove(0);
    assert_eq!(form.review, GenerationReview::Unreviewed);
    assert_eq!(form.claim_sha256.len(), 64);
    let mut artifact: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    artifact["input"]["accepted_generation"] =
        serde_json::json!([{"claim_sha256": form.claim_sha256}]);
    assert!(model_artifact::from_bytes(&serde_json::to_vec(&artifact).unwrap()).is_err());
}

#[test]
fn acceptance_is_specific_to_cell_even_when_surface_and_source_are_shared() {
    let mut data = input();
    let genitive = Cell::noun(Case::Genitive, Number::Singular);
    let mut other = data.paradigms[0].rules[1].clone();
    other.id = RuleId("constructed-syncretism".into());
    other.cell = genitive;
    data.paradigms[0].rules.push(other);
    let wrong_model =
        Model::build(serde_json::from_slice(&serde_json::to_vec(&data).unwrap()).unwrap()).unwrap();
    let model = Model::build(data).unwrap();
    let accepted = decision(&model, cell());
    let mut wrong = accepted.clone();
    wrong.cell = genitive;
    assert!(wrong_model.with_generation_acceptance(vec![wrong]).is_err());
    let reviewed = model.with_generation_acceptance(vec![accepted]).unwrap();
    assert!(matches!(
        forms(&reviewed, cell())[0].review,
        GenerationReview::CallerAccepted { .. }
    ));
    assert_eq!(
        forms(&reviewed, genitive)[0].review,
        GenerationReview::Unreviewed
    );
    let readings = reviewed
        .analyze("отроци", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(readings.candidates.len(), 2);
    assert_eq!(
        readings
            .candidates
            .iter()
            .filter(|c| matches!(c.derivation.review, GenerationReview::CallerAccepted { .. }))
            .count(),
        1
    );
}

#[test]
fn altered_rules_evidence_or_lexical_attachments_invalidate_old_decisions() {
    let model = Model::build(input()).unwrap();
    let accepted = decision(&model, cell());
    for bad in 0..4 {
        let mut data = input();
        match bad {
            0 => data.paradigms[0].rules[1].suffix = "ꙑ".into(),
            1 => data.evidence[0].claim.push_str(" altered claim"),
            2 => data.evidence[0].location.push_str(" altered location"),
            _ => data.lexemes[0].displayed_lemma = "altered identity attachment".into(),
        }
        let model = Model::build(data).unwrap();
        assert_eq!(
            forms(&model, cell())[0].review,
            GenerationReview::Unreviewed
        );
        assert!(
            model
                .with_generation_acceptance(vec![accepted.clone()])
                .is_err(),
            "case {bad}"
        );
    }
}

#[test]
fn document_reload_requires_the_same_caller_policy_and_rejects_forged_review() {
    let plain = Model::build(input()).unwrap();
    let accepted = decision(&plain, cell());
    let reviewed = Model::build(input())
        .unwrap()
        .with_generation_acceptance(vec![accepted])
        .unwrap();
    assert_eq!(plain.data_sha256(), reviewed.data_sha256());
    assert_ne!(
        plain.generation_policy_sha256(),
        reviewed.generation_policy_sha256()
    );
    for text in ["отроци", ""] {
        let doc = AnalysisDocument::analyze_model(
            &reviewed,
            Witness::new(text, None),
            profile(),
            MatchPolicy::Exact,
        )
        .unwrap();
        let json = codec::to_json(&doc).unwrap();
        assert_eq!(
            codec::from_json_model(json.as_bytes(), &reviewed)
                .unwrap()
                .segments(),
            doc.segments()
        );
        assert!(codec::from_json_model(json.as_bytes(), &plain).is_err());
    }
    let doc = AnalysisDocument::analyze_model(
        &plain,
        Witness::new("отроци", None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let mut record = codec::Record::from_document(&doc);
    record
        .segments
        .iter_mut()
        .flat_map(|s| &mut s.candidates)
        .next()
        .unwrap()
        .derivation
        .as_mut()
        .unwrap()
        .review = GenerationReview::CallerAccepted {
        decision_id: "forged".into(),
        method: "copied label".into(),
    };
    assert!(record.resolve_model(&plain).is_err());
}

#[test]
fn policy_order_is_stable_and_invalid_or_duplicate_decisions_fail() {
    let model = Model::build(input()).unwrap();
    let a = decision(&model, cell());
    let b = decision(&model, Cell::noun(Case::Nominative, Number::Singular));
    let first = Model::build(input())
        .unwrap()
        .with_generation_acceptance(vec![a.clone(), b.clone()])
        .unwrap();
    let second = Model::build(input())
        .unwrap()
        .with_generation_acceptance(vec![b, a.clone()])
        .unwrap();
    assert_eq!(
        first.generation_policy_sha256(),
        second.generation_policy_sha256()
    );
    assert_eq!(forms(&first, cell()), forms(&second, cell()));
    assert!(
        Model::build(input())
            .unwrap()
            .with_generation_acceptance(vec![a.clone(), a.clone()])
            .is_err()
    );
    let mut bad = a.clone();
    bad.method.clear();
    assert!(
        Model::build(input())
            .unwrap()
            .with_generation_acceptance(vec![bad])
            .is_err()
    );
    let mut bad = a.clone();
    bad.method = "x".repeat(4097);
    assert!(
        Model::build(input())
            .unwrap()
            .with_generation_acceptance(vec![bad])
            .is_err()
    );
    assert!(matches!(
        Model::build(input())
            .unwrap()
            .with_generation_acceptance(vec![a; 1025]),
        Err(ModelError::ResourceLimit(_))
    ));
    let empty = first.with_generation_acceptance(vec![]).unwrap();
    assert_eq!(
        empty.generation_policy_sha256(),
        model.generation_policy_sha256()
    );
    assert_eq!(
        forms(&empty, cell())[0].review,
        GenerationReview::Unreviewed
    );
}

#[test]
fn accepting_one_alternative_keeps_other_forms_and_missing_information() {
    let mut data = input();
    let mut other = data.paradigms[0].rules[1].clone();
    other.id = RuleId("constructed-other-output".into());
    other.suffix = "ꙑ".into();
    data.paradigms[0].rules.push(other.clone());
    other.id = RuleId("constructed-missing-input".into());
    other.stem = "not-supplied".into();
    data.paradigms[0].rules.push(other);
    let model = Model::build(data).unwrap();
    let accepted = decision(&model, cell());
    let reviewed = model.with_generation_acceptance(vec![accepted]).unwrap();
    let Availability::Partial {
        forms,
        missing_stems,
    } = reviewed.generate(&id(), cell(), &profile()).unwrap()
    else {
        panic!("partial result must remain partial")
    };
    assert_eq!(missing_stems, vec!["not-supplied"]);
    assert_eq!(forms.len(), 2);
    assert_eq!(
        forms
            .iter()
            .filter(|f| matches!(f.review, GenerationReview::CallerAccepted { .. }))
            .count(),
        1
    );
    assert!(
        forms
            .iter()
            .any(|f| f.surface == "отроцꙑ" && f.review == GenerationReview::Unreviewed)
    );
}

#[test]
fn accepted_metadata_is_charged_to_generation_and_analysis_budgets() {
    let limit = 1024;
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_result_bytes: limit,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    let mut accepted = decision(&model, cell());
    let overhead = accepted.decision_id.len()
        + accepted.lexeme.0.len()
        + accepted.orthography.0.len()
        + 2 * accepted.claim_sha256.len();
    accepted.method = "x".repeat(limit - overhead - 1);
    let reviewed = model.with_generation_acceptance(vec![accepted]).unwrap();
    assert!(matches!(
        reviewed.generate(&id(), cell(), &profile()),
        Err(ModelError::ResourceLimit(_))
    ));
    assert!(matches!(
        reviewed.analyze("отроци", &profile(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
}
