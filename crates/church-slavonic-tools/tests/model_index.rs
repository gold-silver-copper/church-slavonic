#![allow(clippy::unwrap_used)]
//! Runtime equivalence and cache isolation, not new linguistic gold.
use church_slavonic::{
    Cell, Pos,
    document::AnalysisDocument,
    matching::{self, MatchPolicy},
    morphology::*,
    witness::Witness,
};
use church_slavonic_tools::{analysis_document as codec, model_artifact::Artifact};
const DATA: &[u8] = include_bytes!("../../../data/rewrite/ocs-finite-model.json");
fn input() -> ModelInput {
    serde_json::from_slice::<Artifact>(DATA).unwrap().input
}
fn profile() -> OrthographyId {
    OrthographyId("o-lrc-finite-tables".into())
}
fn id() -> LexemeId {
    LexemeId("l-000026".into())
}
fn cell(s: &str) -> Cell {
    Cell::parse(Pos::Verb, s).unwrap()
}
fn forms(model: &Model, cell: Cell) -> Vec<Derivation> {
    let Availability::Licensed(forms) = model.generate(&id(), cell, &profile()).unwrap() else {
        panic!("licensed")
    };
    forms
}

#[test]
fn indexed_candidates_equal_independent_generation_enumeration_for_every_policy() {
    let model = Model::build(input()).unwrap();
    for policy in [
        MatchPolicy::Exact,
        MatchPolicy::UnicodeEquivalent,
        MatchPolicy::CaseInsensitive,
        MatchPolicy::AccentInsensitive,
        MatchPolicy::LegacyOrthographic,
    ] {
        for query in [
            "мол҄ꙗашє",
            "МОЛ҄ꙖАШЄ",
            "мол҄ѭ\u{301}",
            "глагол҄ѥтє",
            "unknown",
            "",
        ] {
            // Generation is independent of cache lookup; compare complete
            // derivations and matching traces, not only counts or surfaces.
            let mut expected = Vec::new();
            for entry in input().lexemes {
                for c in model.inventory(&entry.id).unwrap() {
                    let Availability::Licensed(fs) =
                        model.generate(&entry.id, c, &profile()).unwrap()
                    else {
                        panic!("fixture cell")
                    };
                    for d in fs {
                        if let Some(trace) = matching::compare(query, &d.surface, policy) {
                            expected.push((entry.id.clone(), c, d, trace));
                        }
                    }
                }
            }
            let actual = model.analyze(query, &profile(), policy).unwrap();
            let actual: Vec<_> = actual
                .candidates
                .into_iter()
                .map(|c| (c.lexeme.id.clone(), c.cell, c.derivation, c.trace))
                .collect();
            assert_eq!(actual, expected, "{policy:?} {query}");
        }
    }
    let warm = model.analysis_index_stats().unwrap();
    assert_eq!((warm.indexes, warm.forms), (5, 190));
    model
        .analyze("never-seen", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert_eq!(model.analysis_index_stats().unwrap(), warm);
}

#[test]
fn profiles_and_policy_changes_cannot_reuse_an_incompatible_generation() {
    let mut data = input();
    let mut spelling = data.orthographies[0].clone();
    spelling.id = OrthographyId("constructed-spelling".into());
    spelling.rules.push(SpellingRule {
        from: "мол".into(),
        to: "xyz".into(),
        evidence: vec![data.evidence[0].id.clone()],
    });
    let alternate = spelling.id.clone();
    data.orthographies.push(spelling);
    let model = Model::build(data).unwrap();
    assert_eq!(
        model
            .analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .len(),
        2
    );
    assert!(
        model
            .analyze("мол҄ꙗашє", &alternate, MatchPolicy::Exact)
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(
        model
            .analyze("xyz҄ꙗашє", &alternate, MatchPolicy::Exact)
            .unwrap()
            .candidates
            .len(),
        2
    );
    assert_eq!(model.analysis_index_stats().unwrap().indexes, 2);
    let c = cell("impf.2.sg");
    let claim = forms(&model, c).remove(0);
    let model = model
        .with_generation_acceptance(vec![GenerationAcceptance {
            lexeme: id(),
            cell: c,
            orthography: profile(),
            claim_sha256: claim.claim_sha256,
            decision_id: "constructed".into(),
            method: "API control, not expert review".into(),
        }])
        .unwrap();
    assert_eq!(model.analysis_index_stats().unwrap().indexes, 0);
    let readings = model
        .analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact)
        .unwrap();
    assert!(
        readings
            .candidates
            .iter()
            .any(|c| matches!(c.derivation.review, ClaimReview::CallerAccepted { .. }))
    );
    assert!(
        readings
            .candidates
            .iter()
            .any(|c| c.derivation.review == ClaimReview::Unreviewed)
    );
    let model = model.with_generation_acceptance(vec![]).unwrap();
    assert!(
        model
            .analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .iter()
            .all(|c| c.derivation.review == ClaimReview::Unreviewed)
    );
}

#[test]
fn missing_information_and_pending_restrictions_survive_unknown_queries_and_reload() {
    let mut data = input();
    data.lexemes[1].stems.retain(|s| s.name != "present");
    data.lexemes[0].unavailable.push(Restriction {
        cell: cell("inf"),
        reason: "unverified constructed restriction".into(),
        evidence: vec![data.evidence[0].id.clone()],
    });
    let model = Model::build(data).unwrap();
    for query in ["unknown", "молити"] {
        let result = model
            .analyze(query, &profile(), MatchPolicy::Exact)
            .unwrap();
        assert_eq!(result.unresolved_cells.len(), 9);
        assert_eq!(result.unresolved_restrictions.len(), 1);
    }
    let source = "  unknown\tмолити\n";
    let doc = AnalysisDocument::analyze_model(
        &model,
        Witness::new(source, None),
        profile(),
        MatchPolicy::Exact,
    )
    .unwrap();
    let text = codec::to_json(&doc).unwrap();
    let back = codec::from_json_model(text.as_bytes(), &model).unwrap();
    assert_eq!(back.segments(), doc.segments());
    assert_eq!(back.witness().reproduce(), source);
}

#[test]
fn failed_index_builds_publish_nothing_and_query_exhaustion_never_becomes_absence() {
    for limits in [
        ModelLimits {
            max_index_bytes: 1,
            ..ModelLimits::default()
        },
        ModelLimits {
            max_index_forms: 37,
            ..ModelLimits::default()
        },
        ModelLimits {
            max_index_checks: 1,
            ..ModelLimits::default()
        },
    ] {
        let model = Model::build_with_limits(input(), limits).unwrap();
        for _ in 0..2 {
            assert!(matches!(
                model.analyze("unknown", &profile(), MatchPolicy::Exact),
                Err(ModelError::ResourceLimit(_))
            ));
            assert_eq!(
                model.analysis_index_stats().unwrap(),
                AnalysisIndexStats::default()
            );
        }
        assert_eq!(forms(&model, cell("inf")).len(), 1);
    }
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_rule_checks: 2,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    // A successful no-match lookup can construct the index. A later ambiguous
    // lookup still enforces the query work budget on warm cached candidates.
    assert!(
        model
            .analyze("unknown", &profile(), MatchPolicy::Exact)
            .unwrap()
            .candidates
            .is_empty()
    );
    assert!(matches!(
        model.analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact),
        Err(ModelError::ResourceLimit(_))
    ));
}

#[test]
fn aggregate_cache_budget_and_concurrent_warm_reads_are_enforced() {
    let measured = Model::build(input()).unwrap();
    measured
        .analyze("unknown", &profile(), MatchPolicy::Exact)
        .unwrap();
    let bytes = measured.analysis_index_stats().unwrap().retained_bytes;
    let model = Model::build_with_limits(
        input(),
        ModelLimits {
            max_index_bytes: bytes,
            ..ModelLimits::default()
        },
    )
    .unwrap();
    model
        .analyze("unknown", &profile(), MatchPolicy::Exact)
        .unwrap();
    let stats = model.analysis_index_stats().unwrap();
    assert!(matches!(
        model.analyze("unknown", &profile(), MatchPolicy::CaseInsensitive),
        Err(ModelError::ResourceLimit(_))
    ));
    assert_eq!(model.analysis_index_stats().unwrap(), stats);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let model = &model;
            scope.spawn(move || {
                for _ in 0..10 {
                    assert_eq!(
                        model
                            .analyze("мол҄ꙗашє", &profile(), MatchPolicy::Exact)
                            .unwrap()
                            .candidates
                            .len(),
                        2
                    );
                }
            });
        }
    });
    assert_eq!(model.analysis_index_stats().unwrap(), stats);
}
