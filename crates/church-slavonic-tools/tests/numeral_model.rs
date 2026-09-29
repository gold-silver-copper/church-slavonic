#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Cell, Pos,
    document::AnalysisDocument,
    matching::MatchPolicy,
    morphology::{Availability, LexemeId, LexicalCategory, OrthographyId},
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document,
    model_artifact::{self, Fixture},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/ocs-numeral-model.json");

#[test]
fn source_list_forms_inflect_as_numerals_without_an_invariant_fallback() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    let orthography = OrthographyId("o-lrc-numerals".into());
    for (id, bytes, count) in [
        (
            "l-000011",
            include_bytes!("../../../data/rewrite/ocs-numeral-l-000011.json").as_slice(),
            9,
        ),
        (
            "l-000012",
            include_bytes!("../../../data/rewrite/ocs-numeral-l-000012.json").as_slice(),
            18,
        ),
        (
            "l-000013",
            include_bytes!("../../../data/rewrite/ocs-numeral-l-000013.json").as_slice(),
            18,
        ),
    ] {
        let id = LexemeId(id.into());
        let fixture: Fixture = serde_json::from_slice(bytes).unwrap();
        let report = model_artifact::check_fixture(&model, fixture, &id, &orthography).unwrap();
        assert_eq!(
            (
                report.expected_forms,
                report.exact_generated,
                report.joint_analysis_retained
            ),
            (count, count, count)
        );
        assert!(report.failures.is_empty());
        assert!(
            model
                .lexeme(&id)
                .unwrap()
                .categories
                .iter()
                .any(|c| c.category == LexicalCategory::Numeral)
        );
        assert!(
            !model
                .lexeme(&id)
                .unwrap()
                .categories
                .iter()
                .any(|c| c.category == LexicalCategory::Pronoun)
        );
        assert!(matches!(
            model.generate(&id, Cell::Word, &orthography).unwrap(),
            Availability::UnsupportedCell
        ));
        assert!(matches!(
            model
                .generate(
                    &id,
                    Cell::parse(Pos::Pronoun, "m.sg.nom").unwrap(),
                    &orthography
                )
                .unwrap(),
            Availability::UnsupportedCell
        ));
    }
}

#[test]
fn syncretic_gender_and_case_alternatives_survive_document_reload() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("  три дъвоѭ\tчєтꙑрьми\n", None),
        OrthographyId("o-lrc-numerals".into()),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = analysis_document::to_json(&doc).unwrap();
    let restored = analysis_document::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(restored.witness().reproduce(), "  три дъвоѭ\tчєтꙑрьми\n");
    let candidates: Vec<_> = restored
        .segments()
        .iter()
        .filter(|s| !s.candidates().is_empty())
        .map(|s| s.candidates())
        .collect();
    assert_eq!(
        candidates.iter().map(|c| c.len()).collect::<Vec<_>>(),
        vec![5, 6, 3]
    );
    let tri: std::collections::BTreeSet<_> =
        candidates[0].iter().map(|c| c.cell().name()).collect();
    assert_eq!(
        tri,
        ["f.pl.nom", "n.pl.nom", "m.pl.acc", "f.pl.acc", "n.pl.acc"]
            .map(str::to_string)
            .into_iter()
            .collect()
    );
    assert!(candidates.iter().flat_map(|c| c.iter()).all(|c| {
        c.categories()
            .unwrap()
            .iter()
            .any(|a| a.category == LexicalCategory::Numeral)
    }));
    assert_eq!(analysis_document::to_json(&restored).unwrap(), json);
}
