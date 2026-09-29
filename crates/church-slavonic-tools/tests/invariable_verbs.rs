#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Cell, Pos,
    document::AnalysisDocument,
    matching::MatchPolicy,
    morphology::{Availability, LexemeId, OrthographyId},
    witness::Witness,
};
use church_slavonic_tools::{
    analysis_document,
    model_artifact::{self, Fixture},
    sources::{
        proiel::append_xml,
        ud::{Corpus, append_conllu},
    },
    treebank::{node, sexpr},
};
const MODEL: &[u8] = include_bytes!("../../../data/rewrite/ocs-invariable-model.json");
#[test]
fn supine_cell_and_legacy_tree_codec_preserve_the_distinction() {
    let cell = Cell::supine();
    assert_eq!(Cell::parse(Pos::Verb, "sup").unwrap(), cell);
    assert_ne!(cell, Cell::infinitive());
    assert_eq!(
        (cell.number(), cell.person(), cell.case(), cell.gender()),
        (None, None, None, None)
    );
    let tree = node::from_sexpr(&sexpr::parse_many("(v audit.v :form sup)").unwrap()[0]).unwrap();
    assert_eq!(node::from_sexpr(&node::to_sexpr(&tree)).unwrap(), tree);
    let node::Node::Lex { cells, .. } = tree else {
        panic!("lexical cell required")
    };
    assert!(cells.contains(cell));
}
#[test]
fn both_adapters_map_invariable_forms_without_requiring_number() {
    let mut corpus = Corpus::default();
    let text = "1\tбити\tбити\tVERB\t_\tVerbForm=Inf\t0\troot\t_\t_\n2\tбитъ\tбити\tVERB\t_\tVerbForm=Sup\t1\txcomp\t_\t_\n3\tбиеши\tбити\tVERB\t_\tVerbForm=Fin|Mood=Ind|Tense=Pres|Person=2\t0\troot\t_\t_\n";
    append_conllu(&mut corpus, "fixture".into(), text.into()).unwrap();
    assert_eq!(
        corpus.slots.iter().map(|s| s.cell).collect::<Vec<_>>(),
        vec![Cell::infinitive(), Cell::supine()]
    );
    assert_eq!(corpus.skipped["no number"], 1);
    let xml = "<proiel><source language='chu'><sentence><token form='бити' lemma='бити' part-of-speech='V-' morphology='--pna----i'/><token form='битъ' lemma='бити' part-of-speech='V-' morphology='---u--a--i'/></sentence></source></proiel>";
    append_xml(&mut corpus, "fixture.xml".into(), xml.into()).unwrap();
    assert_eq!(corpus.slots[2].cell, Cell::infinitive());
    assert_eq!(corpus.slots[3].cell, Cell::supine());
    let r = &corpus.observations.records()[corpus.sentences[1][1].source_record.unwrap()];
    assert!(corpus.observations.raw(r).unwrap().contains("---u--a--i"));
}
#[test]
fn sourced_pairs_use_generation_analysis_and_document_reload() {
    let model = model_artifact::from_bytes(MODEL).unwrap();
    let profile = OrthographyId("o-lrc-invariable".into());
    for (id, bytes) in [
        (
            "l-000008",
            include_bytes!("../../../data/rewrite/ocs-invariable-l-000008.json").as_slice(),
        ),
        (
            "l-000009",
            include_bytes!("../../../data/rewrite/ocs-invariable-l-000009.json").as_slice(),
        ),
        (
            "l-000010",
            include_bytes!("../../../data/rewrite/ocs-invariable-l-000010.json").as_slice(),
        ),
    ] {
        let fixture: Fixture = serde_json::from_slice(bytes).unwrap();
        let report =
            model_artifact::check_fixture(&model, fixture, &LexemeId(id.into()), &profile).unwrap();
        assert_eq!(
            (
                report.expected_forms,
                report.exact_generated,
                report.joint_analysis_retained
            ),
            (2, 2, 2)
        );
        assert!(report.failures.is_empty());
    }
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new("битъ мꙑтъ рєшть", None),
        profile,
        MatchPolicy::Exact,
    )
    .unwrap();
    let json = analysis_document::to_json(&doc).unwrap();
    let restored = analysis_document::from_json_model(json.as_bytes(), &model).unwrap();
    assert_eq!(doc.segments(), restored.segments());
    assert!(json.contains("sup"));
}
#[test]
fn adding_a_cell_does_not_license_it_in_unrelated_paradigms() {
    let syn = model_artifact::from_bytes(include_bytes!(
        "../../../data/rewrite/synodal-byti-model.json"
    ))
    .unwrap();
    assert!(matches!(
        syn.generate(
            &LexemeId("l-000003".into()),
            Cell::supine(),
            &OrthographyId("o-syn-byti-table".into())
        )
        .unwrap(),
        Availability::UnsupportedCell
    ));
    let ocs = model_artifact::from_bytes(MODEL).unwrap();
    let Availability::Licensed(forms) = ocs
        .generate(
            &LexemeId("l-000010".into()),
            Cell::supine(),
            &OrthographyId("o-lrc-invariable".into()),
        )
        .unwrap()
    else {
        panic!("fixture is licensed")
    };
    assert_eq!(forms[0].surface, "рєшть");
    assert_ne!(forms[0].surface, "рєштъ");
}
