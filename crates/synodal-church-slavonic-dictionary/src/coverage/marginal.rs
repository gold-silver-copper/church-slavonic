#[allow(unused_imports)]
use super::*;

/// Greedily ranks diagnostic recovery batches by overlap-adjusted unique
/// tokens, evidence readiness, and deterministic review effort. This function
/// never changes resolver output: its result is a review-planning diagnostic,
/// not analyzed coverage.
#[must_use]
pub fn marginal_recovery_report(
    total_tokens: usize,
    current_top_k: usize,
    target_basis_points: usize,
    generation_policy: GenerationPolicy,
    orthography_profile: OrthographyProfile,
    candidates: Vec<RecoveryCandidateBatch>,
) -> MarginalRecoveryReport {
    let target_top_k = total_tokens
        .saturating_mul(target_basis_points)
        .checked_div(10_000)
        .unwrap_or(total_tokens)
        .saturating_add(1);
    let mut remaining: BTreeMap<String, RecoveryCandidateBatch> = candidates
        .into_iter()
        .map(|candidate| (candidate.id.clone(), candidate))
        .collect();
    let mut covered: BTreeMap<String, usize> = BTreeMap::new();
    let mut selected: Vec<(String, BTreeMap<String, usize>)> = Vec::new();
    let mut batches = Vec::with_capacity(remaining.len());
    let mut cumulative = 0_usize;

    while !remaining.is_empty() {
        let next_id = remaining
            .values()
            .max_by(|left, right| {
                let left_marginal = marginal_tokens(left, &covered);
                let right_marginal = marginal_tokens(right, &covered);
                diagnostic_score(left_marginal, left)
                    .cmp(&diagnostic_score(right_marginal, right))
                    .then_with(|| left_marginal.cmp(&right_marginal))
                    .then_with(|| {
                        unique_surface_frequency(left).cmp(&unique_surface_frequency(right))
                    })
                    .then_with(|| right.id.cmp(&left.id))
            })
            .map(|candidate| candidate.id.clone())
            .expect("nonempty recovery candidate map");
        let mut candidate = remaining
            .remove(&next_id)
            .expect("selected recovery candidate exists");
        candidate.surfaces.sort_by(|left, right| {
            right
                .frequency
                .cmp(&left.frequency)
                .then_with(|| left.key.cmp(&right.key))
                .then_with(|| left.sample.cmp(&right.sample))
        });
        let membership = surface_membership(&candidate);
        let raw_token_frequency = candidate
            .surfaces
            .iter()
            .map(|surface| surface.frequency)
            .sum();
        let unique_gap_tokens = membership.values().sum();
        let overlap_adjusted_tokens = membership
            .iter()
            .map(|(key, frequency)| {
                frequency.saturating_sub(covered.get(key).copied().unwrap_or(0))
            })
            .sum();
        let overlap_with_higher_batches = selected
            .iter()
            .filter_map(|(higher_id, higher_membership)| {
                let tokens = membership
                    .iter()
                    .map(|(key, frequency)| frequency.min(higher_membership.get(key).unwrap_or(&0)))
                    .sum();
                (tokens > 0).then(|| MarginalOverlap {
                    higher_batch_id: higher_id.clone(),
                    tokens,
                })
            })
            .collect();
        for (key, frequency) in &membership {
            covered
                .entry(key.clone())
                .and_modify(|current| *current = (*current).max(*frequency))
                .or_insert(*frequency);
        }
        cumulative = cumulative.saturating_add(overlap_adjusted_tokens);
        let expected_ambiguity_gain = if candidate.preserves_ambiguity {
            overlap_adjusted_tokens
        } else {
            0
        };
        let expected_top_1_gain = if candidate.preserves_ambiguity {
            0
        } else {
            overlap_adjusted_tokens
        };
        let score = diagnostic_score(overlap_adjusted_tokens, &candidate);
        batches.push(MarginalRecoveryBatch {
            rank: batches.len() + 1,
            id: candidate.id.clone(),
            member_candidate_ids: candidate.member_candidate_ids,
            label: candidate.label,
            part_of_speech: candidate.part_of_speech,
            recovery_route: candidate.recovery_route,
            raw_token_frequency,
            unique_gap_tokens,
            document_frequency: candidate.document_frequency,
            overlap_with_higher_batches,
            overlap_adjusted_tokens,
            cumulative_overlap_adjusted_tokens: cumulative,
            diagnostic_score: score,
            expected_top_1_gain,
            expected_top_k_gain: overlap_adjusted_tokens,
            expected_ambiguity_gain,
            expected_abstention_reduction: overlap_adjusted_tokens,
            compatible_lexeme_ids: candidate.compatible_lexeme_ids,
            proposed_cells: candidate.proposed_cells,
            evidence_available: candidate.evidence_available,
            missing_evidence: candidate.missing_evidence,
            contradictions: candidate.contradictions,
            assumptions: candidate.assumptions,
            confidence_basis_points: candidate.confidence_basis_points,
            review_status: candidate.review_status,
            review_reason: candidate.review_reason,
            evidence_readiness: candidate.evidence_readiness,
            review_effort: candidate.review_effort,
            preserves_ambiguity: candidate.preserves_ambiguity,
            surfaces: candidate.surfaces,
        });
        selected.push((candidate.id, membership));
    }

    MarginalRecoveryReport {
        schema_version: 1,
        target_recension: "synodal-russian".into(),
        generation_policy,
        orthography_profile,
        total_tokens,
        current_top_k,
        target_top_k,
        tokens_needed_for_target: target_top_k.saturating_sub(current_top_k),
        milestones: vec![CoverageMilestone {
            percent: target_basis_points / 100,
            basis_points: target_basis_points,
            target_top_k,
            tokens_needed: target_top_k.saturating_sub(current_top_k),
            margin: current_top_k.saturating_sub(target_top_k),
        }],
        diagnostic_recovery: cumulative,
        diagnostic_projected_top_k: current_top_k.saturating_add(cumulative),
        batches,
    }
}

pub(crate) fn surface_membership(candidate: &RecoveryCandidateBatch) -> BTreeMap<String, usize> {
    let mut membership = BTreeMap::new();
    for surface in &candidate.surfaces {
        membership
            .entry(surface.key.clone())
            .and_modify(|frequency: &mut usize| *frequency = (*frequency).max(surface.frequency))
            .or_insert(surface.frequency);
    }
    membership
}

pub(crate) fn unique_surface_frequency(candidate: &RecoveryCandidateBatch) -> usize {
    surface_membership(candidate).values().sum()
}

pub(crate) fn marginal_tokens(candidate: &RecoveryCandidateBatch, covered: &BTreeMap<String, usize>) -> usize {
    surface_membership(candidate)
        .iter()
        .map(|(key, frequency)| frequency.saturating_sub(covered.get(key).copied().unwrap_or(0)))
        .sum()
}

pub(crate) fn diagnostic_score(marginal: usize, candidate: &RecoveryCandidateBatch) -> usize {
    marginal
        .saturating_mul(candidate.evidence_readiness.weight())
        .checked_div(candidate.review_effort.weight())
        .unwrap_or_default()
}
