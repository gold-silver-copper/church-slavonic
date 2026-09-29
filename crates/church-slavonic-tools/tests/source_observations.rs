#![allow(clippy::unwrap_used)]
use church_slavonic::{
    Case, Cell, Number, document::AnalysisDocument, matching::MatchPolicy,
    morphology::observations::*, morphology::*, witness::Witness,
};
use church_slavonic_tools::{analysis_document as codec, model_artifact::Artifact};
fn fixture() -> (ModelInput, Vec<u8>) {
    let mut data: ModelInput = serde_json::from_slice::<Artifact>(include_bytes!(
        "../../../data/rewrite/ocs-hard-noun-model.json"
    ))
    .unwrap()
    .input;
    let raw = "<h>градъ</h><p>града</p>";
    let start = raw.find("града").unwrap();
    let mut evidence = data.evidence[0].clone();
    evidence.id = EvidenceId("test:annotation".into());
    evidence.source = "test:constructed".into();
    evidence.claim = "Constructed source annotation; no linguistic gold asserted".into();
    evidence.review = ReviewStatus::Unverified;
    evidence.kind = EvidenceKind::Annotation;
    data.evidence.push(evidence);
    data.observations = serde_json::from_value(serde_json::json!({
        "sources":[{"id":"s","uri":"test:constructed","sha256":Witness::new(raw,None).sha256()}],
        "witnesses":[{"id":"w","source":"s","start":start,"end":start+"града".len(),"location":"constructed paragraph text node"}],
        "observations":[{"id":"o","witness":"w","start":0,"end":"града".len(),"surface":"града","kind":"PedagogicalExample"}],
        "annotations":[
            {"id":"a-gen","observation":"o","original":{"case":"genitive","number":"singular"},"target":{"lexeme":"l-000001","cell":["n","gen.sg"],"orthography":"o-lrc-cyrillic"},"evidence":["test:annotation"]},
            {"id":"a-conflict","observation":"o","original":{"case":"nominative","note":"deliberately contradictory"},"target":{"lexeme":"l-000001","cell":["n","nom.sg"],"orthography":"o-lrc-cyrillic"},"evidence":["test:annotation"]},
            {"id":"a-unmapped","observation":"o","original":{"source_tag":"unsupported-tag"},"target":null,"evidence":["test:annotation"]}
        ]
    })).unwrap();
    (data, raw.as_bytes().to_vec())
}
fn spelling() -> OrthographyId {
    OrthographyId("o-lrc-cyrillic".into())
}

#[test]
fn observations_do_not_attest_all_syncretic_cells_or_license_contradictory_forms() {
    let (data, raw) = fixture();
    let model = Model::build_with_sources(data, ModelLimits::default(), vec![raw]).unwrap();
    let result = model
        .analyze("града", &spelling(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(result.candidates.len(), 4);
    for candidate in result.candidates {
        if candidate.cell == Cell::noun(Case::Genitive, Number::Singular) {
            assert_eq!(
                candidate.derivation.source_annotations,
                [AnnotationId("a-gen".into())]
            );
        } else {
            assert!(candidate.derivation.source_annotations.is_empty());
        }
    }
    let Availability::Licensed(forms) = model
        .generate(
            &LexemeId("l-000001".into()),
            Cell::noun(Case::Nominative, Number::Singular),
            &spelling(),
        )
        .unwrap()
    else {
        panic!("known form")
    };
    assert_eq!(forms[0].surface, "градъ");
    assert!(forms[0].source_annotations.is_empty());
    assert_eq!(model.observations().annotations().count(), 3);
    assert!(
        model
            .observations()
            .annotation(&AnnotationId("a-unmapped".into()))
            .unwrap()
            .target
            .is_none()
    );
    let spec = model
        .observations()
        .witness_spec(&WitnessId("w".into()))
        .unwrap();
    assert_eq!(spec.end - spec.start, "града".len());
}

#[test]
fn registered_witnesses_preserve_observations_annotations_and_source_addresses_on_reload() {
    let (data, raw) = fixture();
    let model = Model::build_with_sources(data, ModelLimits::default(), vec![raw]).unwrap();
    let doc = AnalysisDocument::analyze_registered_witness(
        &model,
        WitnessId("w".into()),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    assert_eq!(doc.witness().reproduce(), "града");
    let record = codec::Record::from_document(&doc);
    assert_eq!(record.witness_annotations.len(), 3);
    assert_eq!(record.witness_observations.len(), 1);
    assert_eq!(
        record.observation_source.as_ref().unwrap().uri,
        "test:constructed"
    );
    let encoded = codec::to_json(&doc).unwrap();
    let back = codec::from_json_model(encoded.as_bytes(), &model).unwrap();
    assert_eq!(back.registered_witness(), Some(&WitnessId("w".into())));
    let mut changed = record.clone();
    changed.witness_observations[0].surface = "fabricated".into();
    assert!(changed.resolve_model(&model).is_err());
    let mut changed = record.clone();
    changed.witness_annotations[0]
        .original
        .insert("case".into(), "changed".into());
    assert!(changed.resolve_model(&model).is_err());
    let plain = AnalysisDocument::analyze_model(
        &model,
        doc.witness().clone(),
        spelling(),
        MatchPolicy::Exact,
    )
    .unwrap();
    assert!(plain.registered_witness().is_none());
    assert!(
        codec::Record::from_document(&plain)
            .witness_annotations
            .is_empty()
    );
}

#[test]
fn source_hashes_offsets_and_exact_surfaces_are_validated_without_trusting_review_flags() {
    let (data, _) = fixture();
    assert!(matches!(
        Model::build(data),
        Err(ModelError::MissingSource(_))
    ));
    let (mut data, mut raw) = fixture();
    data.evidence.last_mut().unwrap().review = ReviewStatus::ExpertReviewed {
        reviewer: "untrusted label".into(),
    };
    raw[0] = b'!';
    assert!(matches!(
        Model::build_with_sources(data, ModelLimits::default(), vec![raw]),
        Err(ModelError::MissingSource(_))
    ));
    let (mut data, raw) = fixture();
    data.observations.witnesses[0].start += 1;
    assert!(matches!(
        Model::build_with_sources(data, ModelLimits::default(), vec![raw]),
        Err(ModelError::InvalidObservation(_))
    ));
    let (mut data, raw) = fixture();
    data.observations.observations[0].surface = "градъ".into();
    assert!(matches!(
        Model::build_with_sources(data, ModelLimits::default(), vec![raw]),
        Err(ModelError::InvalidObservation(_))
    ));
    let (data, raw) = fixture();
    assert!(matches!(
        Model::build_with_sources(
            data,
            ModelLimits {
                max_observation_records: 0,
                ..Default::default()
            },
            vec![raw]
        ),
        Err(ModelError::ResourceLimit(_))
    ));
}

#[test]
fn annotation_links_require_the_exact_lexical_identity_and_profile() {
    let (mut data, raw) = fixture();
    // Synthetic homograph, not an assertion of another real lexical entry.
    let mut other = data.lexemes[0].clone();
    other.id = LexemeId("test:homograph".into());
    data.lexemes.push(other);
    let mut profile = data.orthographies[0].clone();
    profile.id = OrthographyId("test:other-profile".into());
    data.orthographies.push(profile);
    let model = Model::build_with_sources(data, ModelLimits::default(), vec![raw]).unwrap();
    let result = model
        .analyze("града", &spelling(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(result.candidates.len(), 8);
    let supported: Vec<_> = result
        .candidates
        .iter()
        .filter(|c| !c.derivation.source_annotations.is_empty())
        .collect();
    assert_eq!(supported.len(), 1);
    assert_eq!(supported[0].lexeme.id.0, "l-000001");
    let other = model
        .analyze(
            "града",
            &OrthographyId("test:other-profile".into()),
            MatchPolicy::Exact,
        )
        .unwrap();
    assert!(
        other
            .candidates
            .iter()
            .all(|c| c.derivation.source_annotations.is_empty())
    );
}

#[test]
fn archive_budgets_include_identifier_copies_and_supplied_source_records() {
    let (mut data, raw) = fixture();
    let id = SourceId("s".repeat(8192));
    data.observations.sources[0].id = id.clone();
    data.observations.witnesses[0].source = id;
    assert!(matches!(
        Model::build_with_sources(
            data,
            ModelLimits {
                max_observation_bytes: 8192 * 2 + 1000,
                ..Default::default()
            },
            vec![raw]
        ),
        Err(ModelError::ResourceLimit(_))
    ));
    let (data, raw) = fixture();
    assert!(matches!(
        Model::build_with_sources(
            data,
            ModelLimits {
                max_observation_records: 7,
                ..Default::default()
            },
            vec![raw.clone(), raw]
        ),
        Err(ModelError::ResourceLimit(_))
    ));
}
