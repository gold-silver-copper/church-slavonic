#![allow(clippy::unwrap_used)]
use church_slavonic::{morphology::*, *};
use church_slavonic_tools::model_artifact::{self, Artifact, Fixture};

const ARTIFACT: &[u8] = include_bytes!("../../../data/rewrite/ocs-hard-noun-model.json");
const FIXTURE: &[u8] = include_bytes!("../../../data/rewrite/ocs-hard-noun.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(ARTIFACT).unwrap().input
}
fn lexeme() -> LexemeId {
    LexemeId("l-000001".into())
}
fn spelling() -> OrthographyId {
    OrthographyId("o-lrc-cyrillic".into())
}
fn fixture() -> Fixture {
    serde_json::from_slice(FIXTURE).unwrap()
}

#[test]
fn independent_ocs_table_generates_and_analyzes_with_traceable_support() {
    let model = model_artifact::from_bytes(ARTIFACT).unwrap();
    let report = model_artifact::check_fixture(&model, fixture(), &lexeme(), &spelling()).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (21, 21, 21)
    );
    assert!(report.failures.is_empty());
    let analyses = model
        .analyze("града", &spelling(), matching::MatchPolicy::Exact)
        .unwrap();
    assert_eq!(analyses.candidates.len(), 4); // gen.sg and nom/acc/voc.du
    assert!(analyses.unresolved_cells.is_empty());
    for analysis in analyses.candidates {
        assert!(!analysis.derivation.evidence.is_empty());
        for id in &analysis.derivation.evidence {
            assert_eq!(
                model.evidence(id).unwrap().review,
                ReviewStatus::SourceChecked
            );
        }
    }
}

#[test]
fn invalid_generation_cannot_disappear_from_the_fixture_denominator() {
    let mut data = input();
    data.paradigms[0]
        .rules
        .iter_mut()
        .find(|r| r.cell.name() == "ins.sg")
        .unwrap()
        .suffix = "омъ".into();
    let model = Model::build(data).unwrap();
    let report = model_artifact::check_fixture(&model, fixture(), &lexeme(), &spelling()).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (21, 20, 20)
    );
    assert!(!report.failures.is_empty());
    assert!(
        model_artifact::check_fixture(
            &model,
            Fixture {
                id: "empty".into(),
                forms: Vec::new()
            },
            &lexeme(),
            &spelling()
        )
        .is_err()
    );
}

#[test]
fn unavailable_missing_and_unsupported_cells_are_distinct() {
    let mut data = input();
    let loc = Cell::noun(Case::Locative, Number::Plural);
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("constructed-restriction".into());
    evidence.source = "test:constructed".into();
    evidence.source_sha256 = "0".repeat(64);
    evidence.claim = "Constructed API fixture only; no linguistic restriction asserted".into();
    evidence.review = ReviewStatus::SourceChecked;
    data.evidence.push(evidence);
    data.lexemes[0].unavailable.push(Restriction {
        cell: loc,
        reason: "Constructed restriction for an API test, not a claim about градъ".into(),
        evidence: vec![EvidenceId("constructed-restriction".into())],
    });
    data.lexemes[0].stems.clear();
    let verified = data
        .evidence
        .iter()
        .filter(|e| e.id.0 == "constructed-restriction")
        .cloned()
        .collect();
    let assertions = vec![RestrictionAssertion {
        lexeme: data.lexemes[0].id.clone(),
        restriction: data.lexemes[0].unavailable[0].clone(),
    }];
    let model =
        Model::build_with_verification(data, ModelLimits::default(), verified, assertions).unwrap();
    assert!(matches!(
        model.generate(&lexeme(), loc, &spelling()).unwrap(),
        Availability::Unavailable(_)
    ));
    assert!(matches!(
        model
            .generate(
                &lexeme(),
                Cell::noun(Case::Nominative, Number::Singular),
                &spelling()
            )
            .unwrap(),
        Availability::MissingStems(_)
    ));
    assert_eq!(
        model
            .generate(&lexeme(), Cell::infinitive(), &spelling())
            .unwrap(),
        Availability::UnsupportedCell
    );
    let result = model
        .analyze("градъ", &spelling(), matching::MatchPolicy::Exact)
        .unwrap();
    assert!(result.candidates.is_empty());
    assert_eq!(result.unresolved_cells.len(), 20);
}

#[test]
fn a_missing_alternative_does_not_hide_a_known_form() {
    let mut data = input();
    let mut alternative = data.paradigms[0].rules[0].clone();
    alternative.id = RuleId("missing-alternative".into());
    alternative.stem = "principal-part-not-supplied".into();
    data.paradigms[0].rules.push(alternative);
    let model = Model::build(data).unwrap();
    let Availability::Partial {
        forms,
        missing_stems,
    } = model
        .generate(
            &lexeme(),
            Cell::noun(Case::Nominative, Number::Singular),
            &spelling(),
        )
        .unwrap()
    else {
        panic!("partial generation")
    };
    assert_eq!(forms[0].surface, "градъ");
    assert_eq!(missing_stems, ["principal-part-not-supplied"]);
    let result = model
        .analyze("градъ", &spelling(), matching::MatchPolicy::Exact)
        .unwrap();
    assert!(!result.candidates.is_empty());
    assert_eq!(result.unresolved_cells.len(), 1);
    let report = model_artifact::check_fixture(&model, fixture(), &lexeme(), &spelling()).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (21, 21, 21)
    );
    assert!(report.failures.is_empty());
    assert_eq!(report.inventory_issues.len(), 1);
    assert_eq!(
        report.inventory_issues[0].details,
        ["principal-part-not-supplied"]
    );
}

#[test]
fn unknown_requested_forms_remain_errors_while_unrelated_unknowns_are_coverage() {
    let mut data = input();
    data.paradigms[0]
        .rules
        .iter_mut()
        .find(|r| r.cell.name() == "ins.sg")
        .unwrap()
        .stem = "missing".into();
    let model = Model::build(data).unwrap();
    let report = model_artifact::check_fixture(&model, fixture(), &lexeme(), &spelling()).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (21, 20, 20)
    );
    assert_eq!(report.failures.len(), 2);
    assert_eq!(report.inventory_issues.len(), 1);
    assert_eq!(report.inventory_issues[0].cell, "ins.sg");
}

#[test]
fn irregular_verb_principal_parts_use_the_same_fixture_and_document_path() {
    use church_slavonic::{document::AnalysisDocument, witness::Witness};
    use church_slavonic_tools::analysis_document as codec;
    let model = model_artifact::from_bytes(include_bytes!(
        "../../../data/rewrite/synodal-byti-model.json"
    ))
    .unwrap();
    let fixture: Fixture =
        serde_json::from_slice(include_bytes!("../../../data/rewrite/synodal-byti.json")).unwrap();
    let id = LexemeId("l-000003".into());
    let orthography = OrthographyId("o-syn-byti-table".into());
    let report = model_artifact::check_fixture(&model, fixture, &id, &orthography).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (10, 10, 10)
    );
    assert!(report.failures.is_empty());
    let witness = Witness::new("  є҆́смь, сꙋ́ть", None);
    let document = AnalysisDocument::analyze_model(
        &model,
        witness,
        orthography.clone(),
        matching::MatchPolicy::Exact,
    )
    .unwrap();
    let encoded = codec::to_json(&document).unwrap();
    let restored = codec::from_json_model(encoded.as_bytes(), &model).unwrap();
    assert_eq!(restored.witness().reproduce(), "  є҆́смь, сꙋ́ть");
    assert_eq!(
        restored
            .segments()
            .iter()
            .map(|s| s.candidates().len())
            .sum::<usize>(),
        2
    );
}

#[test]
fn construction_rejects_dangling_evidence_and_rule_identity_does_not_depend_on_order() {
    let mut data = input();
    data.evidence.clear();
    assert!(matches!(
        Model::build(data),
        Err(ModelError::MissingReference(_))
    ));
    let model = Model::build(input()).unwrap();
    let mut reversed = input();
    reversed.paradigms[0].rules.reverse();
    let reversed = Model::build(reversed).unwrap();
    let rules = |m: &Model| {
        m.analyze("градъ", &spelling(), matching::MatchPolicy::Exact)
            .unwrap()
            .candidates
            .into_iter()
            .map(|a| a.derivation.rule)
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(rules(&model), rules(&reversed));
}

#[test]
fn grammar_and_spelling_profiles_are_checked_separately() {
    let mut data = input();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("constructed-spelling-rule".into());
    evidence.source = "test:constructed".into();
    evidence.source_sha256 = "0".repeat(64);
    evidence.claim =
        "Constructed replacement operation for an API test, not a source prescription".into();
    evidence.review = ReviewStatus::Unverified;
    data.evidence.push(evidence);
    let mut spelling_variant = data.orthographies[0].clone();
    spelling_variant.id = OrthographyId("constructed-spelling".into());
    spelling_variant.rules.push(SpellingRule {
        from: "ѹ".into(),
        to: "оу".into(),
        evidence: vec![EvidenceId("constructed-spelling-rule".into())],
    });
    data.orthographies.push(spelling_variant);
    let mut incompatible = data.orthographies[0].clone();
    incompatible.id = OrthographyId("no-compatible-grammar".into());
    incompatible.compatible_grammars.clear();
    data.orthographies.push(incompatible);
    let model = Model::build(data).unwrap();
    let cell = Cell::noun(Case::Dative, Number::Singular);
    let Availability::Licensed(forms) = model
        .generate(
            &lexeme(),
            cell,
            &OrthographyId("constructed-spelling".into()),
        )
        .unwrap()
    else {
        panic!("licensed")
    };
    assert_eq!(forms[0].underlying, "градѹ");
    assert_eq!(forms[0].surface, "градоу");
    assert_eq!(forms[0].spelling_steps, [("градѹ".into(), "градоу".into())]);
    assert_ne!(forms[0].grammar.0, forms[0].orthography.0);
    assert!(matches!(
        model.generate(
            &lexeme(),
            cell,
            &OrthographyId("no-compatible-grammar".into())
        ),
        Err(ModelError::IncompatibleProfiles)
    ));
}

#[test]
fn unverified_restrictions_never_suppress_generated_candidates() {
    let mut data = input();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("unverified-restriction".into());
    evidence.source = "test:constructed".into();
    evidence.claim = "Unsupported restriction, constructed to test uncertainty".into();
    evidence.review = ReviewStatus::Unverified;
    data.evidence.push(evidence);
    let cell = Cell::noun(Case::Nominative, Number::Singular);
    data.lexemes[0].unavailable.push(Restriction {
        cell,
        reason: "unverified exclusion".into(),
        evidence: vec![EvidenceId("unverified-restriction".into())],
    });
    let model = Model::build(data).unwrap();
    assert!(matches!(
        model.generate(&lexeme(), cell, &spelling()).unwrap(),
        Availability::UnresolvedRestriction { .. }
    ));
    let result = model
        .analyze("градъ", &spelling(), matching::MatchPolicy::Exact)
        .unwrap();
    assert!(result.candidates.iter().any(|a| a.cell == cell));
    assert_eq!(result.unresolved_restrictions.len(), 1);
}

#[test]
fn reported_review_status_cannot_authorize_exclusion() {
    let mut data = input();
    let cell = Cell::noun(Case::Nominative, Number::Singular);
    data.lexemes[0].unavailable.push(Restriction {
        cell,
        reason: "a poisoned claim".into(),
        evidence: vec![data.evidence[0].id.clone()],
    });
    assert_eq!(data.evidence[0].review, ReviewStatus::SourceChecked);
    let model = Model::build(data).unwrap();
    assert!(matches!(
        model.generate(&lexeme(), cell, &spelling()).unwrap(),
        Availability::UnresolvedRestriction { .. }
    ));
    assert!(
        model
            .analyze("градъ", &spelling(), matching::MatchPolicy::Exact)
            .unwrap()
            .candidates
            .iter()
            .any(|a| a.cell == cell)
    );
    let mut data = input();
    let verified = vec![data.evidence[0].clone()];
    data.evidence[0]
        .claim
        .push_str(" modified after verification");
    assert!(matches!(
        Model::build_with_verified_evidence(data, ModelLimits::default(), verified),
        Err(ModelError::VerificationMismatch(_))
    ));
}

#[test]
fn a_trusted_citation_does_not_verify_a_new_restriction_link() {
    let mut data = input();
    let verified = vec![data.evidence[0].clone()];
    let cell = Cell::noun(Case::Nominative, Number::Singular);
    data.lexemes[0].unavailable.push(Restriction {
        cell,
        reason: "not justified by the verified source claim".into(),
        evidence: vec![data.evidence[0].id.clone()],
    });
    let model =
        Model::build_with_verified_evidence(data, ModelLimits::default(), verified).unwrap();
    assert!(matches!(
        model.generate(&lexeme(), cell, &spelling()).unwrap(),
        Availability::UnresolvedRestriction { .. }
    ));
}
