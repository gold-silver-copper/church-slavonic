#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Cell, Pos, document::AnalysisDocument, matching::MatchPolicy, morphology::*, witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec,
    model_artifact::{self, Artifact, Fixture},
};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-finite-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-finite-tables".into())
}
fn fixture(id: &str) -> Fixture {
    let bytes = match id {
        "l-000025" => include_bytes!("../../../data/rewrite/ocs-finite-l-000025.json").as_slice(),
        "l-000026" => include_bytes!("../../../data/rewrite/ocs-finite-l-000026.json").as_slice(),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_slice(bytes).unwrap()
}
fn forms(model: &Model, id: &str, cell: &str) -> Vec<Derivation> {
    let Availability::Licensed(forms) = model
        .generate(
            &LexemeId(id.into()),
            Cell::parse(Pos::Verb, cell).unwrap(),
            &profile(),
        )
        .unwrap()
    else {
        panic!("listed cell")
    };
    forms
}

#[test]
fn all_table_cells_and_infinitives_use_public_generation_analysis_and_reload() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    for id in ["l-000025", "l-000026"] {
        let report =
            model_artifact::check_fixture(&model, fixture(id), &LexemeId(id.into()), &profile())
                .unwrap();
        assert_eq!(
            (
                report.expected_forms,
                report.exact_generated,
                report.joint_analysis_retained
            ),
            (19, 19, 19)
        );
        assert!(report.failures.is_empty());
        assert!(
            model
                .lexical_claims(&LexemeId(id.into()))
                .unwrap()
                .iter()
                .all(|c| c.review == ClaimReview::Unreviewed)
        );
    }
    let text = "  глагол҄ѥтє\tмол҄ꙗашє молити\n";
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new(text, None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = codec::to_json(&doc).unwrap();
    let back = codec::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(back.witness().reproduce(), text);
    assert_eq!(back.segments(), doc.segments());
    let record = codec::Record::from_document(&doc);
    let cells: Vec<_> = record
        .segments
        .iter()
        .flat_map(|s| &s.candidates)
        .map(|c| c.cell.as_str())
        .collect();
    assert_eq!(cells.len(), 5);
    for cell in ["pres.3.du", "pres.2.pl", "impf.2.sg", "impf.3.sg", "inf"] {
        assert!(cells.contains(&cell));
    }
}

#[test]
fn palatalization_mark_is_preserved_separately_from_stress() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    let first = forms(&model, "l-000026", "pres.1.sg");
    assert_eq!(first[0].surface, "мол҄ѭ");
    assert_eq!(
        first[0].stem_change,
        Some(StemTrace {
            before: "моли".into(),
            after: "мол҄".into()
        })
    );
    assert_eq!(first[0].accent.predicted_vowel, None);
    assert_eq!(forms(&model, "l-000026", "pres.3.pl")[0].surface, "молѧтъ");
    let imperfect = forms(&model, "l-000026", "impf.1.sg");
    assert_eq!(imperfect[0].surface, "мол҄ꙗахъ");
    assert!(imperfect[0].stem_change.is_none()); // the imperfect allomorph was supplied
    // Matching-policy controls, not claims of historical ungrammaticality.
    assert!(
        model
            .analyze("молѭ", &profile(), MatchPolicy::AccentInsensitive)
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(
        model
            .analyze("мол҄ѭ\u{301}", &profile(), MatchPolicy::AccentInsensitive)
            .unwrap()
            .candidates
            .len(),
        1
    );
    assert!(
        model
            .analyze("мол҄ѭ\u{301}", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .is_empty()
    );
}

#[test]
fn missing_present_stem_is_not_inferred_from_the_infinitive() {
    let mut data = input();
    data.lexemes[1].stems.retain(|s| s.name != "present");
    let model = Model::build(data).unwrap();
    assert_eq!(forms(&model, "l-000026", "inf")[0].surface, "молити");
    assert!(matches!(
        model
            .generate(
                &LexemeId("l-000026".into()),
                Cell::parse(Pos::Verb, "pres.2.sg").unwrap(),
                &profile()
            )
            .unwrap(),
        Availability::MissingStems(_)
    ));
    let analysis = model
        .analyze("молиши", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert!(analysis.candidates.is_empty());
    assert_eq!(analysis.unresolved_cells.len(), 9);
    let report = model_artifact::check_fixture(
        &model,
        fixture("l-000026"),
        &LexemeId("l-000026".into()),
        &profile(),
    )
    .unwrap();
    assert_eq!(
        (
            report.expected_forms,
            report.exact_generated,
            report.joint_analysis_retained
        ),
        (19, 10, 10)
    );
}

#[test]
fn unspecified_tenses_and_prose_variants_are_not_silently_added() {
    let model = model_artifact::from_bytes(DATA).unwrap();
    for cell in ["aor.1.sg", "fut.1.sg", "sup"] {
        assert!(matches!(
            model
                .generate(
                    &LexemeId("l-000025".into()),
                    Cell::parse(Pos::Verb, cell).unwrap(),
                    &profile()
                )
                .unwrap(),
            Availability::UnsupportedCell
        ));
    }
    assert_eq!(
        forms(&model, "l-000025", "pres.3.du")[0].surface,
        "глагол҄ѥтє"
    );
    assert_eq!(
        forms(&model, "l-000025", "pres.2.du")[0].surface,
        "глагол҄ѥта"
    );
    assert_eq!(forms(&model, "l-000025", "pres.3.du").len(), 1);
}
