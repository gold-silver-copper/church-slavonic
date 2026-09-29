#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Gender, document::AnalysisDocument, matching::MatchPolicy, morphology::*, witness::Witness,
};
use church_slavonic_tools::{analysis_document as codec, model_artifact::Artifact};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/synodal-mudr-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn id() -> LexemeId {
    LexemeId("l-000004".into())
}
fn profile() -> OrthographyId {
    OrthographyId("o-syn-mudr".into())
}
fn decision(model: &Model, select: impl Fn(&LexicalClaim) -> bool) -> LexicalAcceptance {
    LexicalAcceptance {
        lexeme: id(),
        claim_sha256: model
            .lexical_claims(&id())
            .unwrap()
            .iter()
            .find(|c| select(&c.claim))
            .unwrap()
            .claim_sha256
            .clone(),
        decision_id: "constructed-test-decision".into(),
        method: "Engineering control only, not scholarly endorsement".into(),
    }
}
fn relation(c: &LexicalClaim) -> bool {
    matches!(c, LexicalClaim::Relation(_))
}

#[test]
fn citation_labels_and_generation_acceptance_do_not_review_lexical_claims() {
    let mut data = input();
    for e in &mut data.evidence {
        e.review = ReviewStatus::ExpertReviewed {
            reviewer: "untrusted fixture label".into(),
        };
    }
    let verified = data.evidence.clone();
    let model =
        Model::build_with_verified_evidence(data, ModelLimits::default(), verified).unwrap();
    assert!(
        model
            .lexical_claims(&id())
            .unwrap()
            .iter()
            .all(|c| c.review == ClaimReview::Unreviewed)
    );
    let analysis = model
        .analyze("мꙋ́дрѣ", &profile(), MatchPolicy::Exact)
        .unwrap();
    let reading = analysis
        .candidates
        .iter()
        .find(|a| a.lexeme.id == id())
        .unwrap();
    let accept = GenerationAcceptance {
        lexeme: id(),
        cell: reading.cell,
        orthography: profile(),
        claim_sha256: reading.derivation.claim_sha256.clone(),
        decision_id: "generation-only".into(),
        method: "constructed".into(),
    };
    let model = model.with_generation_acceptance(vec![accept]).unwrap();
    assert!(
        model
            .lexical_claims(&id())
            .unwrap()
            .iter()
            .all(|c| c.review == ClaimReview::Unreviewed)
    );
}

#[test]
fn accepting_a_relation_does_not_accept_categories_or_generation() {
    let model = Model::build(input()).unwrap();
    let accepted = decision(&model, relation);
    let model = model.with_lexical_acceptance(vec![accepted]).unwrap();
    let claims = model.lexical_claims(&id()).unwrap();
    assert_eq!(
        claims
            .iter()
            .filter(|c| matches!(c.review, ClaimReview::CallerAccepted { .. }))
            .count(),
        1
    );
    assert!(
        claims
            .iter()
            .filter(|c| !relation(&c.claim))
            .all(|c| c.review == ClaimReview::Unreviewed)
    );
    let analysis = model
        .analyze("мꙋ́дрѣ", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(analysis.candidates.len(), 2);
    assert!(
        analysis
            .candidates
            .iter()
            .all(|a| a.derivation.review == ClaimReview::Unreviewed)
    );
    assert!(
        analysis
            .candidates
            .iter()
            .find(|a| a.lexeme.id == id())
            .unwrap()
            .lexical_claims
            .iter()
            .any(|c| relation(&c.claim) && matches!(c.review, ClaimReview::CallerAccepted { .. }))
    );
}

#[test]
fn altered_relation_target_kind_or_citation_rejects_a_copied_decision() {
    let model = Model::build(input()).unwrap();
    let accepted = decision(&model, relation);
    for bad in 0..3 {
        let mut data = input();
        match bad {
            0 => data.lexemes[0].relations[0].target = id(),
            1 => data.lexemes[0].relations[0].kind = RelationKind::OtherDerivation,
            _ => data.evidence[0].location.push_str(" altered attachment"),
        }
        let model = Model::build(data).unwrap();
        assert!(
            model
                .lexical_claims(&id())
                .unwrap()
                .iter()
                .all(|c| c.review == ClaimReview::Unreviewed)
        );
        assert!(
            model
                .with_lexical_acceptance(vec![accepted.clone()])
                .is_err()
        );
    }
}

#[test]
fn conflicting_gender_claims_survive_and_acceptance_does_not_transfer_between_lexemes() {
    // Constructed assertions about the API, not claims about the adjective.
    let mut data = input();
    let evidence = data.lexemes[0].evidence.clone();
    data.lexemes[0].genders = vec![
        GenderAssertion {
            gender: Gender::Masculine,
            evidence: evidence.clone(),
        },
        GenderAssertion {
            gender: Gender::Feminine,
            evidence,
        },
    ];
    data.lexemes[1].categories = data.lexemes[0].categories.clone();
    let bytes = serde_json::to_vec(&data).unwrap();
    let model = Model::build(data).unwrap();
    let category = decision(&model, |c| matches!(c, LexicalClaim::Category(_)));
    let mut moved = category.clone();
    moved.lexeme = LexemeId("l-000005".into());
    assert!(
        Model::build(serde_json::from_slice(&bytes).unwrap())
            .unwrap()
            .with_lexical_acceptance(vec![moved])
            .is_err()
    );
    let feminine = decision(
        &model,
        |c| matches!(c, LexicalClaim::Gender(g) if g.gender == Gender::Feminine),
    );
    let model = model
        .with_lexical_acceptance(vec![feminine, category])
        .unwrap();
    let genders: Vec<_> = model
        .lexical_claims(&id())
        .unwrap()
        .into_iter()
        .filter(|c| matches!(c.claim, LexicalClaim::Gender(_)))
        .collect();
    assert_eq!(genders.len(), 2);
    assert_eq!(
        genders
            .iter()
            .filter(|c| c.review == ClaimReview::Unreviewed)
            .count(),
        1
    );
}

#[test]
fn document_reload_binds_lexical_policy_and_rejects_forged_claims() {
    let plain = Model::build(input()).unwrap();
    let accepted = decision(&plain, relation);
    let model = Model::build(input())
        .unwrap()
        .with_lexical_acceptance(vec![accepted])
        .unwrap();
    assert_eq!(plain.data_sha256(), model.data_sha256());
    assert_eq!(
        plain.generation_policy_sha256(),
        model.generation_policy_sha256()
    );
    assert_ne!(plain.lexical_policy_sha256(), model.lexical_policy_sha256());
    for text in ["  мꙋ́дрѣ!", ""] {
        let doc = AnalysisDocument::analyze_model(
            &model,
            Witness::new(text, None),
            profile(),
            MatchPolicy::Exact,
        )
        .unwrap();
        let json = codec::to_json(&doc).unwrap();
        assert_eq!(
            codec::from_json_model(json.as_bytes(), &model)
                .unwrap()
                .segments(),
            doc.segments()
        );
        assert!(codec::from_json_model(json.as_bytes(), &plain).is_err());
    }
    let doc = AnalysisDocument::analyze_model(
        &plain,
        Witness::new("мꙋ́дрѣ", None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let mut record = codec::Record::from_document(&doc);
    let claim = &mut record
        .segments
        .iter_mut()
        .flat_map(|s| &mut s.candidates)
        .find(|c| c.lexeme == id().0)
        .unwrap()
        .lexical_claims
        .as_mut()
        .unwrap()[0];
    claim.review = ClaimReview::CallerAccepted {
        decision_id: "forged".into(),
        method: "copied label".into(),
    };
    assert!(record.resolve_model(&plain).is_err());
}

#[test]
fn policy_order_reset_duplicates_and_admission_limits_are_explicit() {
    let plain = Model::build(input()).unwrap();
    let a = decision(&plain, relation);
    let b = decision(&plain, |c| matches!(c, LexicalClaim::Category(_)));
    let first = Model::build(input())
        .unwrap()
        .with_lexical_acceptance(vec![a.clone(), b.clone()])
        .unwrap();
    let second = Model::build(input())
        .unwrap()
        .with_lexical_acceptance(vec![b, a.clone()])
        .unwrap();
    assert_eq!(
        first.lexical_policy_sha256(),
        second.lexical_policy_sha256()
    );
    assert_eq!(
        first.lexical_claims(&id()).unwrap(),
        second.lexical_claims(&id()).unwrap()
    );
    assert!(
        Model::build(input())
            .unwrap()
            .with_lexical_acceptance(vec![a.clone(), a.clone()])
            .is_err()
    );
    let mut bad = a.clone();
    bad.method.clear();
    assert!(
        Model::build(input())
            .unwrap()
            .with_lexical_acceptance(vec![bad])
            .is_err()
    );
    assert!(matches!(
        Model::build(input())
            .unwrap()
            .with_lexical_acceptance(vec![a; 1025]),
        Err(ModelError::ResourceLimit(_))
    ));
    let reset = first.with_lexical_acceptance(vec![]).unwrap();
    assert_eq!(reset.lexical_policy_sha256(), plain.lexical_policy_sha256());
    assert!(
        reset
            .lexical_claims(&id())
            .unwrap()
            .iter()
            .all(|c| c.review == ClaimReview::Unreviewed)
    );
}

#[test]
fn lexical_claim_work_and_review_output_are_bounded() {
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_candidates: 1,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.lexical_claims(&id()),
        Err(ModelError::ResourceLimit(_))
    ));
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_rule_checks: 1,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.lexical_claims(&id()),
        Err(ModelError::ResourceLimit(_))
    ));
    let limit = 1024;
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_result_bytes: limit,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    let mut accepted = decision(&model, relation);
    let overhead =
        accepted.decision_id.len() + accepted.lexeme.0.len() + 2 * accepted.claim_sha256.len();
    accepted.method = "x".repeat(limit - overhead - 1);
    let model = model.with_lexical_acceptance(vec![accepted]).unwrap();
    assert!(matches!(
        model.lexical_claims(&id()),
        Err(ModelError::ResourceLimit(_))
    ));
    assert!(matches!(
        model.analyze("мꙋ́дрѣ", &profile(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
}
