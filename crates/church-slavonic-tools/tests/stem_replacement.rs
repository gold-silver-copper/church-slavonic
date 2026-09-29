#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Case, Cell, Number, document::AnalysisDocument, matching::MatchPolicy, morphology::*,
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec,
    model_artifact::{self, Artifact, Fixture},
};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-stem-replacement-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-otrok".into())
}
fn plural(model: &Model) -> Derivation {
    let Availability::Licensed(mut forms) = model
        .generate(
            &LexemeId("l-000020".into()),
            Cell::noun(Case::Nominative, Number::Plural),
            &profile(),
        )
        .unwrap()
    else {
        panic!("licensed development example")
    };
    forms.remove(0)
}

#[test]
fn supplied_pair_runs_through_rules_analysis_and_reload() {
    // Source pair was consulted to implement the rule: not a held-out test.
    let model = model_artifact::from_bytes(DATA).unwrap();
    let fixture: Fixture = serde_json::from_slice(include_bytes!(
        "../../../data/rewrite/ocs-stem-replacement.json"
    ))
    .unwrap();
    let report =
        model_artifact::check_fixture(&model, fixture, &LexemeId("l-000020".into()), &profile())
            .unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (2, 2, 2)
    );
    let form = plural(&model);
    assert_eq!(form.surface, "отроци");
    assert_eq!(
        form.stem_change,
        Some(StemTrace {
            before: "отрок".into(),
            after: "отроц".into()
        })
    );
    assert!(form.spelling_steps.is_empty());
    assert_eq!(form.accent.predicted_vowel, None);
    assert!(
        model
            .analyze("отроки", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .is_empty()
    );
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("  отрокъ\tотроци\n", None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let restored = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(doc.segments(), restored.segments());
    let changed = json.replace("отроц\"", "отрок\"");
    assert_ne!(json, changed);
    assert!(codec::from_json_model(changed.as_bytes(), &model).is_err());
}

#[test]
fn the_same_operation_supports_a_separate_supplied_verbal_alternation() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    let fixture: Fixture = serde_json::from_slice(include_bytes!(
        "../../../data/rewrite/ocs-stem-replacement-verb.json"
    ))
    .unwrap();
    let report =
        model_artifact::check_fixture(&model, fixture, &LexemeId("l-000021".into()), &profile())
            .unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (2, 2, 2)
    );
    let result = model
        .analyze("можєши", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(
        result.candidates[0].derivation.stem_change,
        Some(StemTrace {
            before: "мог".into(),
            after: "мож".into()
        })
    );
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("могѫ можєши", None),
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
}

#[test]
fn replacement_is_final_only_and_stress_addresses_transformed_stem() {
    // Constructed API controls, not additional historical forms.
    let mut data = input();
    data.lexemes[0].stems[0].text = "кок".into();
    assert_eq!(plural(&Model::build(data).unwrap()).surface, "коци");
    let mut data = input();
    let change = data.paradigms[0].rules[1]
        .stem_replacement
        .as_mut()
        .unwrap();
    change.from = "ок".into();
    change.to = "ц".into();
    data.paradigms[0].rules[1].stress = StressInstruction::SuffixVowel(0);
    let form = plural(&Model::build(data).unwrap());
    assert_eq!(form.underlying, "отрци");
    assert_eq!(form.accent.predicted_vowel, Some(1));
    let mut data = input();
    data.paradigms[0].rules[1]
        .stem_replacement
        .as_mut()
        .unwrap()
        .from = "ок".into();
    data.paradigms[0].rules[1].stress = StressInstruction::StemVowel(1);
    assert!(matches!(Model::build(data), Err(ModelError::Accent(_))));
}

#[test]
fn malformed_or_unjustified_operations_fail_before_model_is_usable() {
    for bad in 0..5 {
        let mut data = input();
        let change = data.paradigms[0].rules[1]
            .stem_replacement
            .as_mut()
            .unwrap();
        match bad {
            0 => change.from.clear(),
            1 => change.from = "г".into(),
            2 => change.to = change.from.clone(),
            3 => change.evidence.clear(),
            _ => change.evidence = vec![EvidenceId("missing".into())],
        }
        assert!(Model::build(data).is_err(), "case {bad}");
    }
    let mut data = input();
    data.lexemes[0].stems.clear();
    let model = Model::build(data).unwrap();
    assert!(matches!(
        model
            .generate(
                &LexemeId("l-000020".into()),
                Cell::noun(Case::Nominative, Number::Plural),
                &profile()
            )
            .unwrap(),
        Availability::MissingStems(_)
    ));
}

#[test]
fn transformed_stems_and_retained_traces_respect_small_limits() {
    let mut data = input();
    data.paradigms[0].rules[1]
        .stem_replacement
        .as_mut()
        .unwrap()
        .to = "ц".repeat(9);
    assert!(matches!(
        Model::build_with_limits(
            data,
            ModelLimits {
                max_form_bytes: 16,
                ..ModelLimits::default()
            }
        ),
        Err(ModelError::ResourceLimit(_))
    ));
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_result_bytes: 1,
            ..ModelLimits::default()
        },
    );
    // Construction may reject early; if built, generation must not report absence.
    match model {
        Err(ModelError::ResourceLimit(_)) => {}
        Ok(model) => assert!(matches!(
            model.generate(
                &LexemeId("l-000020".into()),
                Cell::noun(Case::Nominative, Number::Plural),
                &profile()
            ),
            Err(ModelError::ResourceLimit(_))
        )),
        Err(e) => panic!("unexpected {e}"),
    }
}
