#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Case, Cell, Number, document::AnalysisDocument, matching::MatchPolicy, morphology::*,
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec,
    model_artifact::{self, Artifact, Fixture},
};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-o-stem-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-o-stems".into())
}
fn fixture(id: &str) -> Fixture {
    let bytes = match id {
        "l-000022" => include_bytes!("../../../data/rewrite/ocs-o-stem-l-000022.json").as_slice(),
        "l-000023" => include_bytes!("../../../data/rewrite/ocs-o-stem-l-000023.json").as_slice(),
        _ => include_bytes!("../../../data/rewrite/ocs-o-stem-l-000024.json").as_slice(),
    };
    serde_json::from_slice(bytes).unwrap()
}
fn generated(model: &Model, id: &str, case: Case, number: Number) -> Vec<Derivation> {
    let Availability::Licensed(forms) = model
        .generate(&LexemeId(id.into()), Cell::noun(case, number), &profile())
        .unwrap()
    else {
        panic!("listed cell")
    };
    forms
}

#[test]
fn complete_listed_tables_and_alternatives_use_ordinary_consumers() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    for (id, count) in [("l-000022", 22), ("l-000023", 21), ("l-000024", 21)] {
        let report =
            model_artifact::check_fixture(&model, fixture(id), &LexemeId(id.into()), &profile())
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
    let forms = generated(&model, "l-000022", Case::Dative, Number::Singular);
    assert_eq!(
        forms.iter().map(|f| f.surface.as_str()).collect::<Vec<_>>(),
        vec!["чловѣкѹ", "чловѣкови"]
    );
    assert!(forms.iter().all(|f| f.review == ClaimReview::Unreviewed));
    let text = "  чловѣкови\tчловѣчє вѣцѣ мѣсто\n";
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new(text, None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let restored = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(restored.witness().reproduce(), text);
    assert_eq!(restored.segments(), doc.segments());
    let record = codec::Record::from_document(&doc);
    let cells: Vec<_> = record
        .segments
        .iter()
        .flat_map(|s| &s.candidates)
        .filter(|c| c.lexeme == "l-000024")
        .map(|c| c.cell.as_str())
        .collect();
    assert_eq!(cells.len(), 4); // loc.sg and nom/acc/voc.du remain distinct
    for cell in ["loc.sg", "nom.du", "acc.du", "voc.du"] {
        assert!(cells.contains(&cell));
    }
}

#[test]
fn velar_change_is_specific_to_cell_not_all_front_vowels_or_plurals() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    let human = generated(&model, "l-000022", Case::Nominative, Number::Plural);
    assert_eq!(human[0].surface, "чловѣци");
    assert_eq!(human[0].stem_change.as_ref().unwrap().after, "чловѣц");
    let eyelid = generated(&model, "l-000024", Case::Nominative, Number::Plural);
    assert_eq!(eyelid[0].surface, "вѣка");
    assert!(eyelid[0].stem_change.is_none());
    assert_eq!(
        generated(&model, "l-000024", Case::Nominative, Number::Dual)[0].surface,
        "вѣцѣ"
    );
    assert_eq!(
        generated(&model, "l-000022", Case::Vocative, Number::Singular)[0]
            .stem_change
            .as_ref()
            .unwrap()
            .after,
        "чловѣч"
    );
    assert_eq!(
        generated(&model, "l-000023", Case::Locative, Number::Singular)[0].surface,
        "мѣстѣ"
    );
    assert!(
        generated(&model, "l-000023", Case::Locative, Number::Singular)[0]
            .stem_change
            .is_none()
    );
}

#[test]
fn frozen_otrok_rules_transfer_to_an_additional_supplied_stem() {
    // Same teaching work already consulted: supplemental transfer control,
    // not blind evaluation, stem induction, or independent scholarly gold.
    let mut donor: ModelInput = serde_json::from_slice::<Artifact>(include_bytes!(
        "../../../data/rewrite/ocs-stem-replacement-model.json"
    ))
    .unwrap()
    .input;
    let before = serde_json::to_vec(&donor.paradigms[0].rules).unwrap();
    let mut additional = input();
    let mut entry = additional.lexemes.remove(0);
    entry.paradigm = donor.paradigms[0].id.clone();
    donor.evidence.extend(additional.evidence);
    donor.lexemes.push(entry);
    donor.grammars[0]
        .description
        .push_str("; constructed transfer probe to a further supplied OCS stem");
    assert_eq!(
        serde_json::to_vec(&donor.paradigms[0].rules).unwrap(),
        before
    );
    let model = Model::build(donor).unwrap();
    for (number, expected) in [(Number::Singular, "чловѣкъ"), (Number::Plural, "чловѣци")]
    {
        let Availability::Licensed(forms) = model
            .generate(
                &LexemeId("l-000022".into()),
                Cell::noun(Case::Nominative, number),
                &OrthographyId("o-lrc-otrok".into()),
            )
            .unwrap()
        else {
            panic!("transfer")
        };
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].surface, expected);
        assert_eq!(forms[0].review, ClaimReview::Unreviewed);
    }
}

#[test]
fn removing_a_required_replacement_fails_the_source_fixture() {
    let mut data = input();
    data.paradigms[0]
        .rules
        .iter_mut()
        .find(|r| r.cell == Cell::noun(Case::Locative, Number::Singular))
        .unwrap()
        .stem_replacement = None;
    let model = Model::build(data).unwrap();
    let report = model_artifact::check_fixture(
        &model,
        fixture("l-000022"),
        &LexemeId("l-000022".into()),
        &profile(),
    )
    .unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (22, 21, 21)
    );
    assert!(!report.failures.is_empty());
}
