use super::*;

fn analyzer() -> Arc<Analyzer> {
    default_analyzer().expect("shared default analyzer")
}

#[test]
fn optimized_indexes_match_the_exhaustive_reference() {
    let configurations = [
        Inflector::default(),
        Inflector::builder()
            .generation_policy(GenerationPolicy::Productive)
            .build(),
        Inflector::builder()
            .generation_policy(GenerationPolicy::Exploratory)
            .productive_mapping_threshold_basis_points(0)
            .build(),
    ];
    for inflector in configurations {
        let optimized = Analyzer::new(inflector).expect("optimized analyzer");
        let exhaustive = Analyzer::new_exhaustive(inflector).expect("exhaustive analyzer");
        assert_index_matches(
            "expanded-marked",
            &optimized.expanded_marked,
            &exhaustive.expanded_marked,
        );
        assert_index_matches("expanded", &optimized.expanded, &exhaustive.expanded);
        assert_index_matches(
            "printed-marked",
            &optimized.printed_marked,
            &exhaustive.printed_marked,
        );
        assert_index_matches("printed", &optimized.printed, &exhaustive.printed);
        assert_cardinal_word_index_matches(
            "cardinal-expanded-marked",
            &optimized.cardinal_expanded_marked,
            &exhaustive.cardinal_expanded_marked,
        );
        assert_cardinal_word_index_matches(
            "cardinal-expanded",
            &optimized.cardinal_expanded,
            &exhaustive.cardinal_expanded,
        );
        assert_cardinal_word_index_matches(
            "cardinal-printed-marked",
            &optimized.cardinal_printed_marked,
            &exhaustive.cardinal_printed_marked,
        );
        assert_cardinal_word_index_matches(
            "cardinal-printed",
            &optimized.cardinal_printed,
            &exhaustive.cardinal_printed,
        );
        assert_eq!(
            optimized.spelling_candidates,
            exhaustive.spelling_candidates
        );
        assert!(optimized.indexed_cell_count() < exhaustive.indexed_cell_count());
    }
}

fn assert_index_matches(
    label: &str,
    optimized: &BTreeMap<String, Vec<Analysis>>,
    exhaustive: &BTreeMap<String, Vec<Analysis>>,
) {
    let keys = optimized
        .keys()
        .chain(exhaustive.keys())
        .collect::<BTreeSet<_>>();
    for key in keys {
        assert_eq!(
            optimized.get(key),
            exhaustive.get(key),
            "{label} differs for surface {key:?}"
        );
    }
}

fn assert_cardinal_word_index_matches(
    label: &str,
    optimized: &BTreeMap<String, Vec<CardinalWordAnalysis>>,
    exhaustive: &BTreeMap<String, Vec<CardinalWordAnalysis>>,
) {
    let keys = optimized
        .keys()
        .chain(exhaustive.keys())
        .collect::<BTreeSet<_>>();
    for key in keys {
        assert_eq!(
            optimized.get(key),
            exhaustive.get(key),
            "{label} differs for surface {key:?}"
        );
    }
}

#[test]
fn analyzer_cache_constructs_once_per_compatible_configuration() {
    let cache = Arc::new(AnalyzerCache::new());
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let analyzers = std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    cache.get(Inflector::default()).expect("cached analyzer")
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("analyzer thread"))
            .collect::<Vec<_>>()
    });
    assert_eq!(cache.construction_count(), 1);
    assert_eq!(cache.len(), 1);
    assert!(
        analyzers
            .iter()
            .all(|analyzer| Arc::ptr_eq(&analyzers[0], analyzer))
    );

    let custom = cache
        .get(
            Inflector::builder()
                .generation_policy(GenerationPolicy::Productive)
                .build(),
        )
        .expect("custom analyzer");
    assert!(!Arc::ptr_eq(&analyzers[0], &custom));

    let printed = cache
        .get(
            Inflector::builder()
                .orthography(OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("printed analyzer");
    assert!(!Arc::ptr_eq(&analyzers[0], &printed));

    let lower_threshold = cache
        .get(
            Inflector::builder()
                .productive_mapping_threshold_basis_points(0)
                .build(),
        )
        .expect("lower-threshold analyzer");
    assert!(!Arc::ptr_eq(&analyzers[0], &lower_threshold));
    assert_eq!(cache.construction_count(), 4);
    assert_eq!(cache.len(), 4);
}

fn recovery_candidate(
    id: &str,
    readiness: EvidenceReadiness,
    effort: ReviewEffort,
    preserves_ambiguity: bool,
    surfaces: &[(&str, usize)],
) -> RecoveryCandidateBatch {
    RecoveryCandidateBatch {
        id: id.into(),
        member_candidate_ids: vec![id.into()],
        label: id.into(),
        part_of_speech: "fixture".into(),
        recovery_route: "fixture".into(),
        document_frequency: 1,
        surfaces: surfaces
            .iter()
            .map(|(key, frequency)| RecoverySurfaceCandidate {
                key: (*key).into(),
                sample: (*key).into(),
                frequency: *frequency,
            })
            .collect(),
        compatible_lexeme_ids: Vec::new(),
        proposed_cells: Vec::new(),
        evidence_available: Vec::new(),
        missing_evidence: Vec::new(),
        contradictions: Vec::new(),
        assumptions: Vec::new(),
        confidence_basis_points: 5_000,
        review_status: "fixture".into(),
        review_reason: "fixture".into(),
        evidence_readiness: readiness,
        review_effort: effort,
        preserves_ambiguity,
    }
}

#[test]
fn marginal_recovery_is_overlap_adjusted_and_readiness_ranked() {
    let report = marginal_recovery_report(
        100,
        40,
        6_000,
        GenerationPolicy::Strict,
        OrthographyProfile::SynodalLiturgical,
        vec![
            recovery_candidate(
                "a",
                EvidenceReadiness::Ready,
                ReviewEffort::Medium,
                false,
                &[("shared", 5), ("a-only", 10)],
            ),
            recovery_candidate(
                "b",
                EvidenceReadiness::Ready,
                ReviewEffort::Small,
                true,
                &[("shared", 5), ("b-only", 5)],
            ),
            recovery_candidate(
                "blocked",
                EvidenceReadiness::Blocked,
                ReviewEffort::Small,
                false,
                &[("shared", 5)],
            ),
        ],
    );
    assert_eq!(report.target_top_k, 61);
    assert_eq!(report.tokens_needed_for_target, 21);
    assert_eq!(report.milestones.len(), 1);
    assert_eq!(report.milestones[0].percent, 60);
    assert_eq!(report.milestones[0].basis_points, 6_000);
    assert_eq!(report.milestones[0].target_top_k, 61);
    assert_eq!(report.milestones[0].tokens_needed, 21);
    assert_eq!(report.milestones[0].margin, 0);
    assert_eq!(report.diagnostic_recovery, 20);
    assert_eq!(report.batches[0].id, "b");
    assert_eq!(report.batches[0].expected_top_1_gain, 0);
    assert_eq!(report.batches[0].expected_ambiguity_gain, 10);
    assert_eq!(report.batches[1].id, "a");
    assert_eq!(report.batches[1].overlap_adjusted_tokens, 10);
    assert_eq!(report.batches[1].overlap_with_higher_batches[0].tokens, 5);
    assert_eq!(report.batches[2].id, "blocked");
    assert_eq!(report.batches[2].overlap_adjusted_tokens, 0);
}

#[test]
fn marginal_recovery_uses_maximum_duplicate_surface_membership() {
    let candidate = recovery_candidate(
        "duplicate",
        EvidenceReadiness::Ready,
        ReviewEffort::Small,
        false,
        &[("same", 3), ("same", 3)],
    );
    let report = marginal_recovery_report(
        10,
        0,
        6_001,
        GenerationPolicy::Strict,
        OrthographyProfile::SynodalLiturgical,
        vec![candidate],
    );
    assert_eq!(report.target_top_k, 7);
    assert_eq!(report.batches[0].raw_token_frequency, 6);
    assert_eq!(report.batches[0].unique_gap_tokens, 3);
    assert_eq!(report.batches[0].overlap_adjusted_tokens, 3);
}

#[test]
fn marginal_recovery_breaks_equal_scores_by_stable_id() {
    let candidates = vec![
        recovery_candidate(
            "z-candidate",
            EvidenceReadiness::Partial,
            ReviewEffort::Medium,
            false,
            &[("z", 5)],
        ),
        recovery_candidate(
            "a-candidate",
            EvidenceReadiness::Partial,
            ReviewEffort::Medium,
            false,
            &[("a", 5)],
        ),
    ];
    let report = marginal_recovery_report(
        20,
        0,
        6_000,
        GenerationPolicy::Strict,
        OrthographyProfile::SynodalLiturgical,
        candidates,
    );
    assert_eq!(report.batches[0].id, "a-candidate");
    assert_eq!(report.batches[1].id, "z-candidate");
}

#[test]
fn tokenizer_preserves_marks_spans_and_lines() {
    let text = "є҆́смь,\nбг҃ъ ҂а҃";
    let tokens = tokenize(text);
    assert_eq!(
        tokens
            .iter()
            .map(|token| token.original.as_str())
            .collect::<Vec<_>>(),
        ["є҆́смь", "бг҃ъ", "҂а҃"]
    );
    assert_eq!((tokens[1].line, tokens[1].column), (2, 1));
    assert_eq!(&text[tokens[0].byte_start..tokens[0].byte_end], "є҆́смь");
}

#[test]
fn tokenizer_retains_abbreviations_numerals_and_hostile_marks() {
    let tokens = tokenize("бг҃ъ; ҂а҃, \u{301} слоword");
    assert_eq!(
        tokens
            .iter()
            .map(|token| token.original.as_str())
            .collect::<Vec<_>>(),
        ["бг҃ъ", "҂а҃", "\u{301}", "слоword"]
    );
}

#[test]
fn gap_labels_are_exhaustive_and_stable() {
    assert_eq!(
        GapKind::ALL.map(GapKind::label),
        [
            "unknown-lexeme",
            "missing-declension-or-class",
            "missing-verb-principal-part",
            "unsupported-formation",
            "missing-accent-or-orthographic-metadata",
            "ambiguity-or-spelling-variant",
        ]
    );
}

#[test]
fn coverage_ranks_frequent_gaps_deterministically() {
    let analyzer = analyzer();
    let report = coverage(
        &analyzer,
        &[CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "evaluation".into(),
            source_recension: "synodal-russian".into(),
            text: "неизвѣстно неизвѣстно є҆́смь".into(),
        }],
        CheckTextOptions::default(),
    );
    assert_eq!(report.summary.total_tokens, 3);
    assert_eq!(report.review_queue[0].frequency, 2);
    assert_eq!(report.review_queue[0].kind, GapKind::UnknownLexeme);
    assert!(report.markdown().contains("unknown-lexeme"));
}

#[test]
fn typed_numerals_are_covered_and_frontier_preserves_partition_slices() {
    let analyzer = analyzer();
    let passages = [
        CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "неизвѣстно ҂а҃".into(),
        },
        CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "2".into(),
            partition: "evaluation".into(),
            source_recension: "synodal-russian".into(),
            text: "неизвѣстно неизвѣ́стно".into(),
        },
    ];
    let report = coverage(&analyzer, &passages, CheckTextOptions::default());
    let numeral_report = check_text(&analyzer, "҂а҃", CheckTextOptions::default());

    assert_eq!(report.summary.total_tokens, 4);
    assert_eq!(report.summary.top_1_analyzed, 1);
    assert_eq!(report.summary.top_k_analyzed, 1);
    assert_eq!(report.by_status["cyrillic-numeral"], 1);
    assert_eq!(
        report.by_morphological_system["cyrillic-numeral"].top_k_analyzed,
        1
    );
    let numeral = numeral_report.tokens[0]
        .numeral
        .as_ref()
        .expect("typed numeral analysis");
    assert_eq!(numeral.value(), 1_000);
    assert_eq!(numeral.text(), "҂а҃");
    assert_eq!(numeral_report.summary.top_1_analyzed, 1);
    assert_eq!(numeral_report.summary.top_k_analyzed, 1);
    assert_eq!(
        report
            .uncovered_frontier
            .iter()
            .map(|item| item.token_frequency)
            .sum::<usize>(),
        report.summary.total_tokens - report.summary.top_k_analyzed
    );
    let unknown = report
        .uncovered_frontier
        .iter()
        .find(|item| item.kind == Some(GapKind::UnknownLexeme))
        .expect("unknown frontier row");
    assert_eq!(unknown.token_frequency, 2);
    assert_eq!(unknown.document_frequency, 2);
    assert_eq!(unknown.partitions, ["evaluation", "source"]);
    assert!(
        report
            .uncovered_frontier
            .iter()
            .all(|item| item.status != TokenStatus::CyrillicNumeral)
    );
    assert_eq!(report.by_partition["source"].total_tokens, 2);
    assert_eq!(report.by_partition["evaluation"].total_tokens, 2);
    assert_eq!(report.by_source_partition["fixture:source"].total_tokens, 2);
    assert_eq!(report.by_partition_gap["source"]["unknown-lexeme"], 1);
    assert_eq!(
        report.by_source_partition_gap["fixture:evaluation"]["unknown-lexeme"],
        2
    );
    let frontier = report.uncovered_frontier_tsv();
    assert!(frontier.contains("unknown-lexeme"));
    assert_eq!(frontier.lines().count(), 3);
    assert_eq!(
        report
            .uncovered_frontier
            .iter()
            .filter(|item| item.normalized == "неизвѣстно")
            .count(),
        2,
        "mark-distinct printed surfaces must remain separate frontier rows"
    );
}

#[test]
fn fused_cardinal_words_are_typed_mark_sensitive_and_top_k_covered() {
    let analyzer = analyzer();
    let indexed_values = analyzer
        .cardinal_printed_marked
        .values()
        .flatten()
        .map(|analysis| analysis.value)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        indexed_values,
        BTreeSet::from([
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300,
            400, 500, 600, 700, 800, 900,
        ])
    );
    let options = CheckTextOptions {
        generation_policy: GenerationPolicy::Strict,
        orthography_profile: OrthographyProfile::SynodalLiturgical,
    };
    let report = check_text(
        &analyzer,
        "двана́десѧть три́десѧть пѧтьдесѧ́тъ се́дмьдесѧтъ пѧ́тьдесѧтъ неизвѣстно",
        options.clone(),
    );
    assert_eq!(report.summary.total_tokens, 6);
    assert_eq!(report.summary.top_k_analyzed, 4);
    for token in &report.tokens[..4] {
        assert!(!token.cardinal_words.is_empty(), "{}", token.token.original);
        assert!(token.cardinal_words.iter().all(|analysis| {
            analysis
                .evidence_ids
                .iter()
                .any(|id| id.contains("SYN-NUMERAL-CARDINAL"))
        }));
        assert!(token.gap.is_none());
    }
    assert!(
        report.tokens[0]
            .cardinal_words
            .iter()
            .all(|analysis| analysis.value == 12)
    );
    assert!(
        report.tokens[2]
            .cardinal_words
            .iter()
            .all(|analysis| analysis.value == 50)
    );
    assert!(
        report.tokens[4].cardinal_words.is_empty(),
        "marked input with the wrong stress must not use accentless fallback"
    );
    assert_eq!(report.tokens[4].status, TokenStatus::Unresolved);

    let unmarked = check_text(&analyzer, "пѧтьдесѧтъ", options);
    assert_eq!(unmarked.summary.top_k_analyzed, 1);
    assert!(
        unmarked.tokens[0]
            .cardinal_words
            .iter()
            .all(|analysis| analysis.value == 50)
    );
    assert!(unmarked.tokens[0].cardinal_words.len() > 1);
}

#[test]
fn covered_exact_family_tokens_are_not_counted_as_spelling_variant_work() {
    let analyzer = analyzer();
    let report = coverage(
        &analyzer,
        &[CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "ѽ ѽ".into(),
        }],
        CheckTextOptions::default(),
    );
    assert_eq!(report.summary.top_k_analyzed, 2);
    assert_eq!(report.spelling_variant_family_tokens, 0);
    assert!(report.gaps.is_empty());
    assert_eq!(
        report
            .estimated_recovery_by_route
            .get(RecoveryRoute::SpellingVariant.label()),
        None
    );
}

#[test]
fn covered_ambiguity_is_not_marginal_uncovered_work() {
    let analyzer = analyzer();
    let report = coverage(
        &analyzer,
        &[CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "и".into(),
        }],
        CheckTextOptions {
            generation_policy: GenerationPolicy::Strict,
            orthography_profile: OrthographyProfile::SynodalLiturgical,
        },
    );
    assert_eq!(report.summary.top_k_analyzed, 1);
    assert_eq!(report.summary.ambiguous, 1);
    assert_eq!(report.gaps.len(), 1);
    assert_eq!(report.gaps[0].frequency, 1);
    assert_eq!(report.gaps[0].top_k_uncovered_frequency, 0);
    assert!(report.gaps[0].top_k_uncovered_documents.is_empty());
    assert!(report.uncovered_frontier.is_empty());
    let probable = report
        .unresolved_by_probable_family
        .values()
        .next()
        .expect("ambiguity diagnostic");
    assert_eq!(probable.token_frequency, 1);
    assert_eq!(probable.top_k_uncovered_token_frequency, 0);
    assert_eq!(probable.document_frequency, 0);
}

#[test]
fn coverage_preserves_contexts_and_true_document_unions() {
    let analyzer = analyzer();
    let passages = [
        CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "предъ неизвѣстно и неизвѣстно послѣ".into(),
        },
        CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "2".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "иное неизвѣстно окруженіе".into(),
        },
    ];
    let report = coverage(&analyzer, &passages, CheckTextOptions::default());
    let gap = report
        .gaps
        .iter()
        .find(|gap| gap.normalized == "неизвѣстно")
        .expect("unknown gap");
    assert_eq!(gap.frequency, 3);
    assert_eq!(gap.document_frequency, 2);
    assert_eq!(gap.documents.len(), 2);
    assert_eq!(gap.contexts.len(), 3);
    assert!(
        gap.contexts
            .iter()
            .all(|context| context.excerpt.contains("неизвѣстно"))
    );
    let probable = report
        .unresolved_by_probable_family
        .get("ungrouped:неизвѣстно")
        .expect("probable-family diagnostic");
    assert_eq!(probable.token_frequency, 3);
    assert_eq!(probable.document_frequency, 2);
}

#[test]
fn probable_families_do_not_swallow_unrelated_prefix_matches() {
    let analyzer = analyzer();
    let report = coverage(
        &analyzer,
        &[CoveragePassage {
            corpus: "fixture-corpus".into(),
            source_id: "fixture".into(),
            work: "fixture".into(),
            edition: "fixture".into(),
            passage: "1".into(),
            partition: "source".into(),
            source_recension: "synodal-russian".into(),
            text: "всегда гдѣ".into(),
        }],
        CheckTextOptions::default(),
    );
    assert!(
        !report
            .unresolved_by_probable_family
            .contains_key("diagnostic-family:весь")
    );
    assert!(
        !report
            .unresolved_by_probable_family
            .contains_key("diagnostic-family:господь")
    );
}

#[test]
fn indexed_analyzer_prefers_mark_sensitive_matches() {
    let analyzer = analyzer();
    let conjunction = analyzer
        .analyze_profile("и҆", OrthographyProfile::SynodalLiturgical)
        .expect("conjunction");
    assert_eq!(analysis_ids(&conjunction).len(), 1);
    assert_eq!(
        conjunction[0].lexeme.part_of_speech(),
        PartOfSpeech::Conjunction
    );

    let pronoun = analyzer
        .analyze_profile("и҆̀", OrthographyProfile::SynodalLiturgical)
        .expect("pronoun");
    assert_eq!(
        analysis_ids(&pronoun),
        vec![LexemeId::from("synodal:pronoun:on")]
    );

    let incompatible = analyzer
        .analyze_profile("и\u{301}", OrthographyProfile::SynodalLiturgical)
        .expect("valid but incompatible accent");
    assert!(incompatible.is_empty());

    let homographs = analyzer
        .analyze_profile("ꙗ҆́кѡ", OrthographyProfile::SynodalLiturgical)
        .expect("reviewed marked homographs");
    assert_eq!(analysis_ids(&homographs).len(), 1);
    assert_eq!(
        analysis_ids(&homographs),
        vec![LexemeId::from("synodal:conjunction:wikt-47fa23a7ed6b")]
    );
}

#[test]
fn check_text_reports_malformed_and_incompatible_marks() {
    let analyzer = analyzer();
    let report = check_text(
        &analyzer,
        "а\u{301}\u{301} и\u{301}",
        CheckTextOptions {
            generation_policy: GenerationPolicy::Strict,
            orthography_profile: OrthographyProfile::SynodalLiturgical,
        },
    );
    assert_eq!(report.summary.unresolved_tokens, 2);
    assert!(report.tokens.iter().all(|token| {
        token.gap.as_ref().map(|gap| gap.kind)
            == Some(GapKind::MissingAccentOrOrthographicMetadata)
    }));
}

#[test]
fn all_gap_precedence_paths_are_constructible() {
    for kind in GapKind::ALL {
        let gap = GapOccurrence {
            kind,
            secondary_reasons: GapKind::ALL
                .into_iter()
                .filter(|candidate| *candidate != kind)
                .collect(),
            detail: "fixture".into(),
            candidate_lexeme_ids: Vec::new(),
            requested_morphological_system: None,
            missing_metadata: Vec::new(),
            resolver_trace: RuleTrace::default(),
            suggested_action: "fixture".into(),
        };
        assert_eq!(gap.kind, kind);
    }
    for primary in GapKind::ALL {
        for secondary in GapKind::ALL {
            assert_eq!(
                primary_gap([primary, secondary]),
                Some(if primary.precedence() <= secondary.precedence() {
                    primary
                } else {
                    secondary
                })
            );
        }
    }
}

#[test]
fn reflexive_surfaces_are_derived_from_registered_active_verbs_by_alypy_73() {
    let analyzer = default_analyzer().expect("analyzer");
    let analyses = analyzer.analyze("возвратисѧ").expect("analysis");
    assert!(
        !analyses.is_empty(),
        "возвратисѧ should analyse as a reflexive of возвратити"
    );
    assert!(analyses.iter().all(|analysis| analysis.reflexive));
    assert!(
        analyses
            .iter()
            .all(|analysis| analysis.source == AnalysisSource::SynodalProductiveRule)
    );
    assert!(analyses.iter().all(|analysis| {
        analysis
            .rule_trace
            .steps()
            .last()
            .is_some_and(|step| step.rule.as_str() == "SYN-VERB-REFLEXIVE-ALYPY-73")
    }));
    assert!(analyses.iter().all(|analysis| matches!(
        analysis.cell,
        Some(GrammarCell::FiniteVerb(_) | GrammarCell::Imperative(_))
    )));
    // The deleted jer is restored: да́стсѧ is the host да́стъ plus the enclitic.
    let printed = analyzer
        .analyze_profile("да́стсѧ", OrthographyProfile::SynodalLiturgical)
        .expect("analysis");
    assert!(printed.iter().any(|analysis| analysis.reflexive
        && matches!(analysis.cell, Some(GrammarCell::FiniteVerb(_)))));
    // A non-verbal host never yields a reflexive reading.
    assert!(analyzer.analyze("рабсѧ").expect("analysis").is_empty());
    assert!(analyzer.analyze("рабъсѧ").expect("analysis").is_empty());
}

#[test]
fn analysis_source_contract_remains_prediction_safe() {
    assert_eq!(
        analysis_source(
            &synodal_church_slavonic_core::FormSource::SynodalNormativeGeneration {
                rule: synodal_church_slavonic_core::RuleId::from("fixture")
            }
        ),
        AnalysisSource::SynodalProductiveRule
    );
}
