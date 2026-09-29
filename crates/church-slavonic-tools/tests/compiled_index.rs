#![allow(clippy::unwrap_used)]
//! Compiled-output trust, runtime equivalence, and hostile transport controls.
use church_slavonic::{
    Cell, Pos, document::AnalysisDocument, matching::MatchPolicy, morphology::*, witness::Witness,
};
use church_slavonic_tools::{
    analysis_document as codec, compiled_index,
    model_artifact::{self, Artifact},
};
use sha2::{Digest, Sha256};
use std::path::Path;
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-finite-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn model() -> Model {
    Model::build(input()).unwrap()
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-finite-tables".into())
}
fn runtime() -> String {
    "a".repeat(64)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn compile(m: &Model, policy: MatchPolicy) -> Vec<u8> {
    m.compile_analysis_index(&profile(), policy, &runtime())
        .unwrap()
}
fn install(
    m: &mut Model,
    bytes: &[u8],
    policy: MatchPolicy,
) -> Result<AnalysisIndexStats, ModelError> {
    m.install_compiled_analysis_index(bytes, &digest(bytes), &runtime(), &profile(), policy)
}
fn document(m: &Model, text: &str, policy: MatchPolicy) -> String {
    codec::to_json(
        &AnalysisDocument::analyze_model(m, Witness::new(text, None), profile(), policy).unwrap(),
    )
    .unwrap()
}
fn rows(bytes: &[u8]) -> Vec<serde_json::Value> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn encode(rows: &[serde_json::Value]) -> Vec<u8> {
    rows.iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>()
        .into_bytes()
}

#[test]
fn compiled_all_policies_reproduce_complete_documents_without_cold_generation() {
    let producer = model();
    let mut consumer = model();
    for policy in [
        MatchPolicy::Exact,
        MatchPolicy::UnicodeEquivalent,
        MatchPolicy::CaseInsensitive,
        MatchPolicy::AccentInsensitive,
        MatchPolicy::LegacyOrthographic,
    ] {
        let bytes = compile(&producer, policy);
        let stats = install(&mut consumer, &bytes, policy).unwrap();
        assert_eq!(stats.restored_indexes, stats.indexes);
        for source in [
            "  глагол҄ѥтє\tмол҄ꙗашє молити\n",
            "МОЛ҄ꙖАШЄ unknown",
            "мол҄ѭ\u{301}",
        ] {
            let expected = document(&producer, source, policy);
            let actual = document(&consumer, source, policy);
            assert_eq!(actual, expected);
            let back = codec::from_json_model(actual.as_bytes(), &consumer).unwrap();
            assert_eq!(back.witness().reproduce(), source);
        }
        assert_eq!(consumer.analysis_index_stats().unwrap(), stats);
    }
    assert_eq!(consumer.analysis_index_stats().unwrap().forms, 190);
}

#[test]
fn exact_caller_digest_and_current_runtime_model_and_policy_are_required() {
    let bytes = compile(&model(), MatchPolicy::Exact);
    let mut changed = bytes.clone();
    changed[0] = b'[';
    assert!(matches!(
        model().install_compiled_analysis_index(
            &changed,
            &digest(&bytes),
            &runtime(),
            &profile(),
            MatchPolicy::Exact
        ),
        Err(ModelError::ArtifactMismatch("compiled index digest"))
    ));
    assert!(install(&mut model(), &bytes, MatchPolicy::CaseInsensitive).is_err());
    assert!(
        model()
            .install_compiled_analysis_index(
                &bytes,
                &digest(&bytes),
                &"b".repeat(64),
                &profile(),
                MatchPolicy::Exact
            )
            .is_err()
    );
    let mut data = input();
    data.lexemes[0].displayed_lemma += " changed";
    assert!(install(&mut Model::build(data).unwrap(), &bytes, MatchPolicy::Exact).is_err());
    let mut accepted = model();
    let cell = Cell::parse(Pos::Verb, "impf.2.sg").unwrap();
    let id = LexemeId("l-000026".into());
    let Availability::Licensed(forms) = accepted.generate(&id, cell, &profile()).unwrap() else {
        panic!("form")
    };
    let decision = GenerationAcceptance {
        lexeme: id,
        cell,
        orthography: profile(),
        claim_sha256: forms[0].claim_sha256.clone(),
        decision_id: "test".into(),
        method: "constructed control".into(),
    };
    accepted = accepted
        .with_generation_acceptance(vec![decision.clone()])
        .unwrap();
    assert!(install(&mut accepted, &bytes, MatchPolicy::Exact).is_err());
    let reviewed = compile(&accepted, MatchPolicy::Exact);
    let mut altered = rows(&reviewed);
    let accepted_row = altered
        .iter_mut()
        .find(|r| r["Form"]["derivation"]["review"].is_object())
        .unwrap();
    accepted_row["Form"]["derivation"]["underlying"] = "changed claim content".into();
    let mut target = model()
        .with_generation_acceptance(vec![decision.clone()])
        .unwrap();
    assert!(matches!(
        install(&mut target, &encode(&altered), MatchPolicy::Exact),
        Err(ModelError::ArtifactMismatch(
            "compiled accepted claim content"
        ))
    ));
    assert_eq!(
        target.analysis_index_stats().unwrap(),
        AnalysisIndexStats::default()
    );

    let mut restored = model().with_generation_acceptance(vec![decision]).unwrap();
    install(&mut restored, &reviewed, MatchPolicy::Exact).unwrap();
    assert_eq!(
        document(&restored, "мол҄ꙗашє", MatchPolicy::Exact),
        document(&accepted, "мол҄ꙗашє", MatchPolicy::Exact)
    );
    let reset = restored.with_generation_acceptance(vec![]).unwrap();
    assert_eq!(
        reset.analysis_index_stats().unwrap(),
        AnalysisIndexStats::default()
    );
    // A reset policy cannot accept the previously reviewed compiled payload.
    assert!(
        accepted
            .with_generation_acceptance(vec![])
            .unwrap()
            .install_compiled_analysis_index(
                &reviewed,
                &digest(&reviewed),
                &runtime(),
                &profile(),
                MatchPolicy::Exact
            )
            .is_err()
    );
}

#[test]
fn malformed_trusted_payloads_are_rejected_without_partial_publication() {
    let bytes = compile(&model(), MatchPolicy::Exact);
    let baseline = rows(&bytes);
    for choice in 0..8 {
        let mut rows = baseline.clone();
        match choice {
            0 => rows[0]["version"] = 900.into(),
            1 => rows[0]["forms"] = serde_json::json!(u64::MAX),
            2 => {
                rows.pop();
            }
            3 => {
                rows.push(rows[1].clone());
                rows[0]["forms"] = 39.into();
            }
            4 => rows[1]["Form"]["lexeme"] = "missing-lexeme".into(),
            5 => rows[1]["Form"]["derivation"]["rule"] = "missing-rule".into(),
            6 => {
                rows[1]["Form"]["derivation"]["review"] =
                    serde_json::json!({"CallerAccepted":{"decision_id":"forged","method":"forged"}})
            }
            _ => rows[0]["engine_sha256"] = "b".repeat(64).into(),
        }
        let mut target = model();
        // Recomputed digest deliberately bypasses trust to exercise structural
        // validation; this does not establish completeness of arbitrary files.
        assert!(
            install(&mut target, &encode(&rows), MatchPolicy::Exact).is_err(),
            "case {choice}"
        );
        assert_eq!(
            target.analysis_index_stats().unwrap(),
            AnalysisIndexStats::default()
        );
    }
    let mut deep = b"[".repeat(65);
    deep.extend(b"]".repeat(65));
    deep.push(b'\n');
    assert!(matches!(
        install(&mut model(), &deep, MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
    let many = format!("[{}]\n", vec!["0"; 4098].join(","));
    assert!(matches!(
        install(&mut model(), many.as_bytes(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
    let large = vec![b' '; 1024 * 1024 + 1];
    let mut oversized = large;
    oversized.push(b'\n');
    assert!(matches!(
        install(&mut model(), &oversized, MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
    let mut invalid_utf8 = bytes.clone();
    invalid_utf8[0] = 0xff;
    assert!(install(&mut model(), &invalid_utf8, MatchPolicy::Exact).is_err());
    assert!(install(&mut model(), &bytes[..bytes.len() - 1], MatchPolicy::Exact).is_err());
}

#[test]
fn pending_restrictions_and_missing_stems_are_preserved_in_restored_queries() {
    let mut data = input();
    data.lexemes[1].stems.retain(|s| s.name != "present");
    data.lexemes[0].unavailable.push(Restriction {
        cell: Cell::parse(Pos::Verb, "inf").unwrap(),
        reason: "unreviewed control".into(),
        evidence: vec![data.evidence[0].id.clone()],
    });
    let encoded = serde_json::to_vec(&data).unwrap();
    let producer = Model::build(data).unwrap();
    let bytes = compile(&producer, MatchPolicy::Exact);
    let mut consumer = Model::build(serde_json::from_slice(&encoded).unwrap()).unwrap();
    install(&mut consumer, &bytes, MatchPolicy::Exact).unwrap();
    assert_eq!(
        document(&producer, "unknown молити", MatchPolicy::Exact),
        document(&consumer, "unknown молити", MatchPolicy::Exact)
    );
    let found = consumer
        .analyze("unknown", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(found.unresolved_cells.len(), 9);
    assert_eq!(found.unresolved_restrictions.len(), 1);
}

#[test]
fn actual_file_publication_and_loader_require_explicit_digest() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("model.json");
    std::fs::write(&input_path, DATA).unwrap();
    let output = dir.path().join("compiled.idx");
    let receipt =
        compiled_index::compile(&input_path, profile(), MatchPolicy::Exact, &output, &[]).unwrap();
    assert_eq!(receipt.forms, 38);
    let mut loaded = model_artifact::load(&input_path).unwrap();
    compiled_index::load_into(
        &mut loaded,
        &output,
        &receipt.sha256,
        &profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    assert_eq!(loaded.analysis_index_stats().unwrap().restored_indexes, 1);
    assert_eq!(
        document(&loaded, "мол҄ꙗашє", MatchPolicy::Exact),
        document(&model(), "мол҄ꙗашє", MatchPolicy::Exact)
    );
    let before = std::fs::read(&output).unwrap();
    assert!(
        compiled_index::compile(
            &input_path,
            OrthographyId("missing".into()),
            MatchPolicy::Exact,
            &output,
            &[]
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&output).unwrap(), before);
    assert!(
        compiled_index::compile(&input_path, profile(), MatchPolicy::Exact, &input_path, &[])
            .is_err()
    );
    assert_eq!(std::fs::read(&input_path).unwrap(), DATA);
    assert!(
        compiled_index::load_into(
            &mut model(),
            Path::new(&output),
            &"0".repeat(64),
            &profile(),
            MatchPolicy::Exact
        )
        .is_err()
    );
}

#[test]
fn restored_indexes_share_the_aggregate_model_budget() {
    let limited = || {
        Model::build_with_limits(
            input(),
            ModelLimits {
                max_index_forms: 38,
                ..ModelLimits::default()
            },
        )
        .unwrap()
    };
    let exact = compile(&limited(), MatchPolicy::Exact);
    let folded = compile(&limited(), MatchPolicy::CaseInsensitive);
    let mut consumer = limited();
    install(&mut consumer, &exact, MatchPolicy::Exact).unwrap();
    let stats = consumer.analysis_index_stats().unwrap();
    assert!(matches!(
        install(&mut consumer, &folded, MatchPolicy::CaseInsensitive),
        Err(ModelError::ResourceLimit(_))
    ));
    assert_eq!(consumer.analysis_index_stats().unwrap(), stats);
    assert!(install(&mut consumer, &exact, MatchPolicy::Exact).is_err());
    assert_eq!(
        consumer
            .analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .len(),
        2
    );
}

#[test]
fn synodal_accents_and_abbreviation_derivations_survive_compilation() {
    let fixtures: &[(&[u8], &str, &str)] = &[
        (
            include_bytes!("../../../data/rewrite/synodal-rab-model.json"),
            "o-synodal-alypy-table",
            "  ра́бъ раба̀\n",
        ),
        (
            include_bytes!("../../../data/rewrite/synodal-titlo-model.json"),
            "o-alypy-titlo-examples",
            "  бг҃ъ бо́гъ\n",
        ),
    ];
    for &(data, profile, text) in fixtures {
        let profile = OrthographyId(profile.into());
        let producer = model_artifact::from_bytes(data).unwrap();
        let bytes = producer
            .compile_analysis_index(&profile, MatchPolicy::Exact, &runtime())
            .unwrap();
        let mut consumer = model_artifact::from_bytes(data).unwrap();
        consumer
            .install_compiled_analysis_index(
                &bytes,
                &digest(&bytes),
                &runtime(),
                &profile,
                MatchPolicy::Exact,
            )
            .unwrap();
        let record = |m| {
            codec::to_json(
                &AnalysisDocument::analyze_model(
                    m,
                    Witness::new(text, None),
                    profile.clone(),
                    MatchPolicy::Exact,
                )
                .unwrap(),
            )
            .unwrap()
        };
        assert_eq!(record(&producer), record(&consumer));
        assert!(
            !consumer
                .analyze(
                    text.split_whitespace().next().unwrap(),
                    &profile,
                    MatchPolicy::Exact
                )
                .unwrap()
                .candidates
                .is_empty()
        );
    }
}
