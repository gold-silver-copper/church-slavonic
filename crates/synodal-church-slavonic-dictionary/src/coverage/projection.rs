#[allow(unused_imports)]
use super::*;

/// One distinct printed surface of the corpus with its token frequency, the
/// input to [`project_surface_counts`].
#[derive(Clone, Debug)]
pub struct SurfaceCount {
    /// The surface exactly as printed in the corpus.
    pub original: String,
    /// The tokenizer's normalized form of the surface (holdout key).
    pub normalized: String,
    /// How many corpus tokens print exactly this surface.
    pub frequency: usize,
}

/// The ledger-relevant totals of a projected coverage run.
///
/// Produced by [`project_surface_counts`], which reuses the same per-token
/// classification as [`coverage_with_type_holdout`], so on an identical
/// surface inventory the numbers match the full run by construction. A
/// projection carries no per-passage attribution and can never seal a wave.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverageProjection {
    pub total_tokens: usize,
    pub top_1_analyzed: usize,
    pub top_k_analyzed: usize,
    pub cross_lexeme_ambiguous: usize,
    pub holdout_tokens: usize,
    pub holdout_top_k: usize,
    pub holdout_generalised: usize,
    pub holdout_memorised: usize,
}

/// Projects the ledger-relevant coverage totals from a distinct-surface
/// inventory, classifying each surface exactly once with the same
/// [`classify_token`] path the corpus loop uses and weighting by frequency.
pub fn project_surface_counts(
    analyzer: &Analyzer,
    surfaces: &[SurfaceCount],
    options: CheckTextOptions,
    held_out_types: &BTreeSet<String>,
) -> CoverageProjection {
    let mut summary = CoverageSlice::default();
    let mut integrity = CoverageIntegrity::default();
    let mut held_out_slice = CoverageSlice::default();
    let mut held_out_status: BTreeMap<String, usize> = BTreeMap::new();
    for surface in surfaces {
        let tokens = tokenize(&surface.original);
        let Some(token) = tokens.into_iter().next() else {
            continue;
        };
        let analysis = classify_token(analyzer, token, &options);
        let held = held_out_types.contains(&surface.normalized);
        for _ in 0..surface.frequency {
            update_slice(&mut summary, &analysis);
            update_integrity(&mut integrity, &analysis);
            if held {
                update_slice(&mut held_out_slice, &analysis);
                *held_out_status
                    .entry(status_label(analysis.status).into())
                    .or_default() += 1;
            }
        }
    }
    let holdout_generalised = GENERALISING_STATUSES
        .iter()
        .map(|status| held_out_status.get(*status).copied().unwrap_or_default())
        .sum();
    CoverageProjection {
        total_tokens: summary.total_tokens,
        top_1_analyzed: summary.top_1_analyzed,
        top_k_analyzed: summary.top_k_analyzed,
        cross_lexeme_ambiguous: integrity.cross_lexeme_ambiguous,
        holdout_tokens: held_out_slice.total_tokens,
        holdout_top_k: held_out_slice.top_k_analyzed,
        holdout_generalised,
        holdout_memorised: held_out_status
            .get(MEMORISING_STATUS)
            .copied()
            .unwrap_or_default(),
    }
}

pub fn coverage(
    analyzer: &Analyzer,
    passages: &[CoveragePassage],
    options: CheckTextOptions,
) -> CoverageReport {
    coverage_with_type_holdout(analyzer, passages, options, &BTreeSet::new())
}

/// Computes coverage while measuring a caller-supplied type-disjoint holdout
/// separately.
///
/// The corpus partition split is passage-disjoint, so most frontier surfaces
/// occur on both sides of it and an exact row sourced from a `source` passage
/// closes its own held-out twin. Holding out normalized *types* is what makes
/// generalization measurable. The set is supplied by the caller because the
/// runtime crates never read the filesystem.
pub fn coverage_with_type_holdout(
    analyzer: &Analyzer,
    passages: &[CoveragePassage],
    options: CheckTextOptions,
    held_out_types: &BTreeSet<String>,
) -> CoverageReport {
    let mut summary = CoverageSlice::default();
    let mut by_corpus = BTreeMap::new();
    let mut by_source = BTreeMap::new();
    let mut by_partition = BTreeMap::new();
    let mut by_source_partition = BTreeMap::new();
    let mut by_policy = BTreeMap::new();
    let mut by_lexeme = BTreeMap::new();
    let mut by_family = BTreeMap::new();
    let mut by_system = BTreeMap::new();
    let mut by_corpus_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_source_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_partition_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_source_partition_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    let mut integrity = CoverageIntegrity::default();
    let mut predicted_by_system: BTreeMap<String, usize> = BTreeMap::new();
    let mut predicted_by_confidence: BTreeMap<String, usize> = BTreeMap::new();
    let mut held_out_slice = CoverageSlice::default();
    let mut held_out_status = BTreeMap::new();
    let mut held_out_status_by_system: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut held_out_observed = BTreeSet::new();
    let mut by_gap = BTreeMap::new();
    let mut types = BTreeSet::new();
    let mut aggregates: BTreeMap<(GapKind, String), GapAggregate> = BTreeMap::new();
    let mut frontier_aggregates: BTreeMap<
        (String, TokenStatus, Option<GapKind>),
        CoverageFrontierAggregate,
    > = BTreeMap::new();
    let mut cache: BTreeMap<String, TextTokenAnalysis> = BTreeMap::new();
    let mut probable_aggregates: BTreeMap<String, ProbableFamilyAggregate> = BTreeMap::new();
    let mut recovery_estimates = BTreeMap::new();
    let mut abbreviation_family_tokens = 0_usize;
    let mut spelling_variant_family_tokens = 0_usize;
    let mut remaining_ungrouped_unknowns = 0_usize;

    for passage in passages {
        let document = format!("{}:{}", passage.source_id, passage.passage);
        let source_partition = format!("{}:{}", passage.source_id, passage.partition);
        for token in tokenize(&passage.text) {
            types.insert(token.normalized.clone());
            let template = cache
                .entry(token.original.clone())
                .or_insert_with(|| classify_token(analyzer, token.clone(), &options))
                .clone();
            let mut analysis = template;
            analysis.token = token;
            update_slice(&mut summary, &analysis);
            update_slice(
                by_corpus.entry(passage.corpus.clone()).or_default(),
                &analysis,
            );
            update_slice(
                by_source.entry(passage.source_id.clone()).or_default(),
                &analysis,
            );
            update_slice(
                by_partition.entry(passage.partition.clone()).or_default(),
                &analysis,
            );
            update_slice(
                by_source_partition
                    .entry(source_partition.clone())
                    .or_default(),
                &analysis,
            );
            update_slice(
                by_policy
                    .entry(policy_label(options.generation_policy).into())
                    .or_default(),
                &analysis,
            );
            for lexeme_id in analysis
                .analyses
                .iter()
                .map(|candidate| candidate.lexeme.id().to_string())
                .collect::<BTreeSet<_>>()
            {
                update_slice(by_lexeme.entry(lexeme_id).or_default(), &analysis);
            }
            for lexeme_id in analysis
                .analyses
                .iter()
                .map(|candidate| candidate.lexeme.id())
                .collect::<BTreeSet<_>>()
            {
                let family_id = FamilyId::for_lexeme(lexeme_id).to_string();
                update_slice(by_family.entry(family_id).or_default(), &analysis);
            }
            // A token is attributed to the first *typed* reading it carries;
            // a reviewed lexical-form row that happens to outrank a noun
            // reading in source precedence does not make the token
            // morphology-free.
            let system = if analysis.numeral.is_some() {
                "cyrillic-numeral".into()
            } else if !analysis.cardinal_words.is_empty() {
                "compound-cardinal-word".into()
            } else {
                let typed = analysis.analyses.iter().find_map(|candidate| {
                    candidate.cell.filter(|cell| {
                        !matches!(cell, GrammarCell::LexicalForm | GrammarCell::Indeclinable)
                    })
                });
                typed
                    .or_else(|| {
                        analysis
                            .analyses
                            .first()
                            .and_then(|candidate| candidate.cell)
                    })
                    .map_or_else(|| "unresolved".into(), morphological_system)
            };
            update_slice(by_system.entry(system.clone()).or_default(), &analysis);
            update_integrity(&mut integrity, &analysis);
            if held_out_types.contains(&analysis.token.normalized) {
                held_out_observed.insert(analysis.token.normalized.clone());
                update_slice(&mut held_out_slice, &analysis);
                *held_out_status
                    .entry(status_label(analysis.status).into())
                    .or_default() += 1;
                *held_out_status_by_system
                    .entry(system)
                    .or_default()
                    .entry(status_label(analysis.status).into())
                    .or_default() += 1;
            }
            *by_status
                .entry(status_label(analysis.status).into())
                .or_default() += 1;
            if !is_top_k_analyzed(&analysis) {
                if let Some(top) = crate::prediction::predict(&analysis.token.normalized)
                    .into_iter()
                    .next()
                {
                    *predicted_by_system
                        .entry(prediction_system(top.cell))
                        .or_default() += 1;
                    *predicted_by_confidence
                        .entry(confidence_bucket(top.confidence_bp).into())
                        .or_default() += 1;
                }
                let frontier_key = (
                    analysis.token.original.clone(),
                    analysis.status,
                    analysis.gap.as_ref().map(|gap| gap.kind),
                );
                frontier_aggregates
                    .entry(frontier_key)
                    .or_insert_with(|| CoverageFrontierAggregate::new(&analysis))
                    .observe(passage, &document, &analysis);
            }
            if analysis.status == TokenStatus::AbbreviationExpansion {
                abbreviation_family_tokens += 1;
            }
            if let Some(gap) = &analysis.gap {
                let (probable_id, route, assumption) = probable_family(&analysis, gap);
                if !is_top_k_analyzed(&analysis) {
                    *recovery_estimates.entry(route.label().into()).or_default() += 1;
                    if route == RecoveryRoute::AbbreviationRegistry {
                        abbreviation_family_tokens += 1;
                    }
                    if route == RecoveryRoute::SpellingVariant {
                        spelling_variant_family_tokens += 1;
                    }
                    if route == RecoveryRoute::UngroupedUnknown {
                        remaining_ungrouped_unknowns += 1;
                    }
                }
                probable_aggregates
                    .entry(probable_id.clone())
                    .or_insert_with(|| ProbableFamilyAggregate::new(probable_id, route, assumption))
                    .observe(&analysis, gap, &document);
                *by_gap.entry(gap.kind.label().into()).or_default() += 1;
                *by_corpus_gap
                    .entry(passage.corpus.clone())
                    .or_default()
                    .entry(gap.kind.label().into())
                    .or_default() += 1;
                *by_source_gap
                    .entry(passage.source_id.clone())
                    .or_default()
                    .entry(gap.kind.label().into())
                    .or_default() += 1;
                *by_partition_gap
                    .entry(passage.partition.clone())
                    .or_default()
                    .entry(gap.kind.label().into())
                    .or_default() += 1;
                *by_source_partition_gap
                    .entry(source_partition.clone())
                    .or_default()
                    .entry(gap.kind.label().into())
                    .or_default() += 1;
                aggregates
                    .entry((gap.kind, analysis.token.normalized.clone()))
                    .or_insert_with(|| GapAggregate::new(passage, &analysis, gap))
                    .observe(passage, &document, &analysis, gap);
            }
        }
    }

    let mut gaps: Vec<GapRecord> = aggregates
        .into_values()
        .map(|aggregate| aggregate.finish(&options))
        .collect();
    gaps.sort_by(|left, right| {
        right
            .frequency
            .cmp(&left.frequency)
            .then_with(|| right.document_frequency.cmp(&left.document_frequency))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.normalized.cmp(&right.normalized))
    });
    let total_gap_types = gaps.len();
    let mut gap_frequency_by_surface = BTreeMap::new();
    let mut top_k_uncovered_frequency_by_surface = BTreeMap::new();
    for gap in &gaps {
        *gap_frequency_by_surface
            .entry(gap.normalized.clone())
            .or_default() += gap.frequency;
        if gap.top_k_uncovered_frequency > 0 {
            *top_k_uncovered_frequency_by_surface
                .entry(gap.normalized.clone())
                .or_default() += gap.top_k_uncovered_frequency;
        }
    }
    let unresolved_by_probable_family = probable_aggregates
        .into_iter()
        .map(|(id, aggregate)| (id, aggregate.finish()))
        .collect();
    let mut uncovered_frontier: Vec<_> = frontier_aggregates
        .into_values()
        .map(CoverageFrontierAggregate::finish)
        .collect();
    uncovered_frontier.sort_by(|left, right| {
        right
            .token_frequency
            .cmp(&left.token_frequency)
            .then_with(|| right.document_frequency.cmp(&left.document_frequency))
            .then_with(|| left.status.cmp(&right.status))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.normalized.cmp(&right.normalized))
    });
    let review_queue = gaps
        .iter()
        .take(500)
        .enumerate()
        .map(|(index, gap)| ReviewQueueItem {
            rank: index + 1,
            kind: gap.kind,
            normalized: gap.normalized.clone(),
            sample: gap.original.clone(),
            frequency: gap.frequency,
            document_frequency: gap.document_frequency,
            candidate_lexeme_ids: gap.candidate_lexeme_ids.clone(),
            suggested_action: gap.suggested_action.clone(),
        })
        .collect();
    gaps.truncate(2_000);
    CoverageReport {
        schema_version: 4,
        target_recension: "synodal-russian".into(),
        generation_policy: options.generation_policy,
        orthography_profile: options.orthography_profile,
        passages: passages.len(),
        token_types: types.len(),
        summary,
        integrity,
        held_out_types: held_out_observed.len(),
        held_out_type_coverage: held_out_slice,
        held_out_type_status: held_out_status,
        predicted_unresolved_by_system: predicted_by_system,
        predicted_unresolved_by_confidence: predicted_by_confidence,
        held_out_type_status_by_system: held_out_status_by_system,
        by_corpus,
        by_source,
        by_partition,
        by_source_partition,
        by_policy,
        by_lexeme,
        by_family,
        by_morphological_system: by_system,
        by_corpus_gap,
        by_source_gap,
        by_partition_gap,
        by_source_partition_gap,
        by_status,
        by_gap,
        unresolved_by_probable_family,
        estimated_recovery_by_route: recovery_estimates,
        abbreviation_family_tokens,
        spelling_variant_family_tokens,
        remaining_ungrouped_unknowns,
        gap_frequency_by_surface,
        top_k_uncovered_frequency_by_surface,
        total_gap_types,
        gaps,
        review_queue,
        uncovered_frontier,
    }
}
