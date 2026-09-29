#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Case, Cell, Number, document::AnalysisDocument, matching::MatchPolicy, morphology::*,
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec,
    model_artifact::{self, Artifact, Fixture},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/synodal-titlo-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(MODEL).unwrap().input
}
fn spelling() -> OrthographyId {
    OrthographyId("o-alypy-titlo-examples".into())
}
fn cell() -> Cell {
    Cell::noun(Case::Nominative, Number::Singular)
}

#[test]
fn registered_abbreviation_does_not_merge_a_distinct_uncontracted_lexeme() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    for (bytes, id) in [
        (
            include_bytes!("../../../data/rewrite/synodal-titlo-divine.json").as_slice(),
            "l-000006",
        ),
        (
            include_bytes!("../../../data/rewrite/synodal-titlo-idol.json").as_slice(),
            "l-000007",
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
            (1, 1, 1)
        );
        assert!(report.failures.is_empty());
    }
    let short = model
        .analyze("бг҃ъ", &spelling(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(short.candidates.len(), 1);
    assert_eq!(short.candidates[0].lexeme.id.0, "l-000006");
    let derivation = &short.candidates[0].derivation;
    let trace = derivation.abbreviation.as_ref().unwrap();
    assert_eq!((&*trace.expanded, &*trace.abbreviated), ("бо́гъ", "бг҃ъ"));
    assert!(!trace.evidence.is_empty());
    assert!(short.candidates[0].trace.query_steps.is_empty());
    let full = model
        .analyze("бо́гъ", &spelling(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(full.candidates.len(), 1);
    assert_eq!(full.candidates[0].lexeme.id.0, "l-000007");
    assert!(full.candidates[0].derivation.abbreviation.is_none());
    // No subsequence expansion, titlo deletion, or case folding in exact mode.
    for unknown in ["б҃ъ", "бгъ", "БГ҃Ъ"] {
        assert!(
            model
                .analyze(unknown, &spelling(), MatchPolicy::Exact)
                .unwrap()
                .candidates
                .is_empty()
        );
    }
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("  бг҃ъ, бо́гъ!", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let back = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(back.witness().reproduce(), "  бг҃ъ, бо́гъ!");
    let mut record = codec::Record::from_document(&doc);
    let candidate = record
        .segments
        .iter_mut()
        .flat_map(|s| &mut s.candidates)
        .find(|c| c.lexeme == "l-000006")
        .unwrap();
    candidate
        .derivation
        .as_mut()
        .unwrap()
        .abbreviation
        .as_mut()
        .unwrap()
        .expanded = "fabricated".into();
    assert!(record.resolve_model(&model).is_err());
}

#[test]
fn competing_registered_expansions_survive_and_rule_order_is_not_identity() {
    // Constructed second lexical interpretation: tests retention, not a source claim.
    let mut data = input();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("test:second-expansion".into());
    evidence.source = "test:constructed".into();
    evidence.review = ReviewStatus::Unverified;
    evidence.claim = "Synthetic alternative expansion only".into();
    let mut second = data.orthographies[0].abbreviations[0].clone();
    second.id = RuleId("test:second-expansion".into());
    second.lexeme = data.lexemes[1].id.clone();
    second.retain_expanded = true;
    second.evidence = vec![evidence.id.clone()];
    data.evidence.push(evidence);
    data.orthographies[0].abbreviations.push(second);
    let copy: ModelInput = serde_json::from_value(serde_json::to_value(&data).unwrap()).unwrap();
    let model = Model::build(copy).unwrap();
    data.orthographies[0].abbreviations.reverse();
    let reversed = Model::build(data).unwrap();
    let identities = |m: &Model| {
        m.analyze("бг҃ъ", &spelling(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .into_iter()
            .map(|a| {
                (
                    a.lexeme.id.clone(),
                    a.cell,
                    a.derivation.abbreviation.unwrap().rule,
                )
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(identities(&model).len(), 2);
    assert_eq!(identities(&model), identities(&reversed));
    assert_eq!(
        model
            .analyze("бо́гъ", &spelling(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .len(),
        1
    );
}

#[test]
fn malformed_rules_and_expansion_growth_are_rejected() {
    for invalid in ["бг҃ъ.", "бг҃ъ бо́гъ", "и\u{300}"] {
        let mut data = input();
        data.orthographies[0].abbreviations[0].abbreviated = invalid.into();
        assert!(matches!(
            Model::build(data),
            Err(ModelError::InvalidRule(_))
        ));
    }
    let mut data = input();
    data.orthographies[0].abbreviations[0].expanded = "not generated".into();
    assert!(matches!(
        Model::build(data),
        Err(ModelError::InvalidRule(_))
    ));
    let mut data = input();
    data.orthographies[0].abbreviations[0].evidence.clear();
    assert!(matches!(
        Model::build(data),
        Err(ModelError::MissingEvidence(_))
    ));
    let mut data = input();
    let mut conflicting = data.orthographies[0].abbreviations[0].clone();
    conflicting.id = RuleId("conflicting".into());
    conflicting.retain_expanded = true;
    data.orthographies[0].abbreviations.push(conflicting);
    assert!(matches!(
        Model::build(data),
        Err(ModelError::InvalidRule(_))
    ));
    let mut data = input();
    data.orthographies[0].abbreviations[0].abbreviated = "а".repeat(100);
    let limits = ModelLimits {
        max_form_bytes: 64,
        ..Default::default()
    };
    assert!(matches!(
        Model::build_with_limits(data, limits),
        Err(ModelError::ResourceLimit(_))
    ));
    let mut data = input();
    data.orthographies[0].abbreviations[0].retain_expanded = true;
    let model = Model::build_with_limits(
        data,
        ModelLimits {
            max_candidates: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.generate(&LexemeId("l-000006".into()), cell(), &spelling()),
        Err(ModelError::ResourceLimit(_))
    ));
}

#[test]
fn spelling_rules_do_not_revive_a_verified_unavailable_cell() {
    let mut data = input();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("test:restriction".into());
    evidence.source = "test:constructed".into();
    evidence.claim = "Synthetic unavailable cell; no linguistic assertion".into();
    let restriction = Restriction {
        cell: cell(),
        reason: "constructed restriction".into(),
        evidence: vec![evidence.id.clone()],
    };
    data.evidence.push(evidence.clone());
    data.lexemes[0].unavailable.push(restriction.clone());
    let assertion = RestrictionAssertion {
        lexeme: data.lexemes[0].id.clone(),
        restriction,
    };
    let model = Model::build_with_verification(
        data,
        ModelLimits::default(),
        vec![evidence],
        vec![assertion],
    )
    .unwrap();
    assert!(matches!(
        model
            .generate(&LexemeId("l-000006".into()), cell(), &spelling())
            .unwrap(),
        Availability::Unavailable(_)
    ));
    assert!(
        model
            .analyze("бг҃ъ", &spelling(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .is_empty()
    );
}
