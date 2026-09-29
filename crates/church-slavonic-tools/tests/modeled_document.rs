#![allow(clippy::unwrap_used)]
use church_slavonic::{
    document::{AnalysisDocument, Candidate},
    matching::MatchPolicy,
    morphology::*,
    witness::Witness,
    *,
};
use church_slavonic_tools::{
    analysis_document::{self as codec, Record, RecordError},
    model_artifact::{self, Artifact, Fixture},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/synodal-rab-model.json");
const FIXTURE: &[u8] = include_bytes!("../../../data/rewrite/synodal-rab.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(MODEL).unwrap().input
}
fn spelling() -> OrthographyId {
    OrthographyId("o-synodal-alypy-table".into())
}
fn lexeme() -> LexemeId {
    LexemeId("l-000002".into())
}

#[test]
fn synodal_source_forms_stress_and_source_documents_work_end_to_end() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    let fixture: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    let report = model_artifact::check_fixture(&model, fixture, &lexeme(), &spelling()).unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (21, 21, 21)
    );
    assert!(report.failures.is_empty());
    let source = "  ра́бъ,\tраба̀\nunknown";
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new(source, None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let restored = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(restored.witness().reproduce(), source);
    assert_eq!(doc.segments(), restored.segments());
    let mut positions = std::collections::BTreeSet::new();
    for candidate in restored.segments().iter().flat_map(|s| s.candidates()) {
        let Candidate::Modeled(candidate) = candidate else {
            panic!("model path must not use legacy candidates")
        };
        positions.insert(candidate.derivation.accent.predicted_vowel.unwrap());
        assert!(!candidate.derivation.evidence.is_empty());
    }
    assert_eq!(positions, [0, 1].into_iter().collect());
    let record = Record::from_document(&restored);
    assert_eq!(
        record.model_data_sha256.as_deref(),
        Some(model.data_sha256())
    );
    assert_eq!(record.engine_sha256.as_deref(), Some(model.engine_sha256()));
    assert_eq!(record.evidence.len(), 3);
}

#[test]
fn unchanged_surface_does_not_hide_a_changed_model_snapshot() {
    let model = Model::build(input()).unwrap();
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("ра́бъ", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let original = Record::from_document(&doc);
    let mut changed = input();
    changed.evidence[0].claim.push_str(" Changed claim.");
    let changed = Model::build(changed).unwrap();
    assert!(matches!(
        original.resolve_model(&changed),
        Err(RecordError::SnapshotMismatch)
    ));
    let mut record = original.clone();
    record.engine_sha256 = Some("0".repeat(64));
    assert!(matches!(
        record.resolve_model(&model),
        Err(RecordError::SnapshotMismatch)
    ));
    let mut record = original.clone();
    record.source_sha256 = "0".repeat(64);
    assert!(matches!(
        record.resolve_model(&model),
        Err(RecordError::SourceMismatch)
    ));
    let mut record = original;
    record.segments[0].candidates[0]
        .derivation
        .as_mut()
        .unwrap()
        .evidence
        .clear();
    assert!(matches!(
        record.resolve_model(&model),
        Err(RecordError::AnalysisMismatch)
    ));
}

#[test]
fn accent_errors_are_rejected_during_model_construction() {
    let mut data = input();
    data.paradigms[0].rules[0].stress = StressInstruction::StemVowel(usize::MAX);
    assert!(matches!(Model::build(data), Err(ModelError::Accent(_))));
    let mut data = input();
    data.lexemes[0].stems[0].text = "ра́б".into();
    assert!(matches!(
        Model::build(data),
        Err(ModelError::Accent(
            accent::AccentError::ConflictingInputMarks
        ))
    ));
    let mut data = input();
    data.paradigms[0]
        .rules
        .iter_mut()
        .find(|r| r.cell.name() == "nom.du")
        .unwrap()
        .stress = StressInstruction::Unspecified;
    assert!(Model::build(data).is_err());
}

#[test]
fn unprinted_prediction_is_distinct_from_unknown_stress() {
    let mut data = input();
    data.orthographies[0].accent = AccentPolicy::PreserveInput;
    data.orthographies[0].accent_overrides.clear();
    let model = Model::build(data).unwrap();
    let Availability::Licensed(forms) = model
        .generate(
            &lexeme(),
            Cell::noun(Case::Genitive, Number::Singular),
            &spelling(),
        )
        .unwrap()
    else {
        panic!("licensed")
    };
    assert_eq!(forms[0].surface, "раба");
    assert_eq!(forms[0].accent.predicted_vowel, Some(1));
    assert_eq!(forms[0].accent.supplied_mark, None);
    let unknown = accent::render("ѝ", None, &AccentPolicy::PreserveInput, None).unwrap();
    assert_eq!(unknown.accented, "ѝ");
    assert_eq!(unknown.predicted_vowel, None);
    assert_eq!(unknown.supplied_mark, None);
    let breathing = accent::render(
        "а҆",
        Some(0),
        &AccentPolicy::MarkPredicted {
            internal: AccentMark::Acute,
            final_vowel: AccentMark::Grave,
        },
        None,
    )
    .unwrap();
    assert_eq!(breathing.accented, "а҆̀");
}

#[test]
fn incomplete_analysis_survives_document_serialization() {
    let mut data = input();
    data.lexemes[0].stems.clear();
    let model = Model::build(data).unwrap();
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("ра́бъ", None),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    assert!(doc.segments()[0].candidates().is_empty());
    assert_eq!(doc.segments()[0].uncertainties().len(), 21);
    let mut record = Record::from_document(&doc);
    assert!(record.resolve_model(&model).is_ok());
    record.segments[0].uncertainties.clear();
    assert!(matches!(
        record.resolve_model(&model),
        Err(RecordError::AnalysisMismatch)
    ));
}

#[test]
fn external_models_cannot_silently_exceed_caller_resource_limits() {
    let mut data = input();
    data.orthographies[0].rules = (0..20)
        .map(|_| SpellingRule {
            from: "р".into(),
            to: "рр".into(),
            evidence: vec![EvidenceId("e-syn-spelling".into())],
        })
        .collect();
    let model = Model::build_with_limits(
        data,
        ModelLimits {
            max_form_bytes: 100,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.generate(
            &lexeme(),
            Cell::noun(Case::Nominative, Number::Singular),
            &spelling()
        ),
        Err(ModelError::ResourceLimit("form bytes"))
    ));
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_candidates: 0,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.analyze("ра́бъ", &spelling(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_rule_checks: 0,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.analyze("unknown", &spelling(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit("rule checks"))
    ));
}

#[test]
fn work_limits_accumulate_across_document_tokens() {
    // Even no-match lookups consume query work across the document. Index
    // construction has its own budget; the former 800-check threshold counted
    // repeated generation, which warm queries no longer perform.
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_rule_checks: 1,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(
        AnalysisDocument::analyze_model(
            &model,
            Witness::new("unknown", None),
            spelling(),
            MatchPolicy::Exact
        )
        .is_ok()
    );
    assert!(matches!(
        AnalysisDocument::analyze_model(
            &model,
            Witness::new("unknown unknown", None),
            spelling(),
            MatchPolicy::Exact
        ),
        Err(ModelError::ResourceLimit("rule checks"))
    ));
    let model = Model::build(input()).unwrap();
    let oversized = " ".repeat(document::MAX_SOURCE_BYTES + 1);
    assert!(matches!(
        AnalysisDocument::analyze_model(
            &model,
            Witness::new(oversized, None),
            spelling(),
            MatchPolicy::Exact
        ),
        Err(ModelError::ResourceLimit("document source bytes"))
    ));
}

#[test]
fn retained_identifiers_and_large_queries_are_included_in_resource_limits() {
    let mut data = input();
    let long_id = OrthographyId("o".repeat(512));
    data.orthographies[0].id = long_id.clone();
    let model = Model::build_with_limits(
        data,
        ModelLimits {
            max_result_bytes: 300,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        model.generate(
            &lexeme(),
            Cell::noun(Case::Nominative, Number::Singular),
            &long_id
        ),
        Err(ModelError::ResourceLimit("generation results"))
    ));
    let model = Model::build(input()).unwrap();
    let source = "а".repeat(model.limits().max_form_bytes);
    assert!(matches!(
        model.analyze(&source, &spelling(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit("query bytes"))
    ));
}
