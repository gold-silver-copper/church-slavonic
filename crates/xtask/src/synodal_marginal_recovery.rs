use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use crate::report_io::{check_contents_for, read_json, write_if_changed_atomic};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use synodal_church_slavonic::{GenerationPolicy, OrthographyProfile};
use synodal_church_slavonic_dictionary::coverage::{
    CoverageMilestone, CoverageReport, EvidenceReadiness, GapKind, GapRecord,
    MarginalRecoveryReport, RecoveryCandidateBatch, RecoveryRoute, RecoverySurfaceCandidate,
    ReviewEffort, marginal_recovery_report,
};

const BASELINE_LOCK: &str = "reports/synodal-v04-baseline.json";
const BASELINE_MARGINAL: &str = "reports/synodal-v04-marginal-recovery.json";
const CURRENT_JSON: &str = "reports/synodal-marginal-recovery.json";
const CURRENT_MARKDOWN: &str = "reports/synodal-marginal-recovery.md";
const CURRENT_TSV: &str = "reports/synodal-marginal-recovery.tsv";
const TARGET_BASIS_POINTS: usize = 7_000;
const MILESTONE_BASIS_POINTS: [usize; 5] = [6_600, 6_700, 6_800, 6_900, 7_000];

const COVERAGE_SHA256: &str = "1fb8426b49d3a40f1b28c72f65a1d51f15916127fd85c22f88cdf71d12d01dbf";
const EVALUATION_SHA256: &str = "2d83efd560601e7ddcac2e6fc65ac4376eb1640b0615ff73c93d431cbbb5eed2";
const FAMILY_QUEUE_SHA256: &str =
    "2e168e226abd5d26b7f3e950666b2fe8fa116bdafec74fbacd52993b047e9b7b";
const FAMILY_REVIEWS_SHA256: &str =
    "1b596aee2c956e55200c9ab7b655e9febc571ffabbdc1fc8ad1139ea50db9704";
const LEXICAL_REVIEWS_SHA256: &str =
    "5c8e1b4b23ba35c3a6474dcd83a85f4338c80ee6966c23269f62683baef675bd";
const MORPHOLOGY_REGISTRY_SHA256: &str =
    "259b7a4b223eb8ce263c464e3d9b5945d58631340c53e2cb08f44f03328f03a2";
const DICTIONARY_REGISTRY_SHA256: &str =
    "a460c2c476749aa37290934eb075945279eaefff500cdde63b443363f6bac6d5";

const SOURCE_LOCKS: [(&str, &str, usize, usize); 2] = [
    (
        "data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl",
        "ef0323df940c93c9b72a3cbb6f7adfb062ba38ffcdcf401eff5cf369c4869c26",
        7_574,
        29_637,
    ),
    (
        "data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl",
        "913d9781ef511988d8bcc5d19b1b8c63c7582cd5e476f62469eff199e7c2c08f",
        7_481,
        29_438,
    ),
];

#[derive(Clone, Debug, Deserialize)]
struct FamilySurfaceInput {
    original: String,
    normalized: String,
    frequency: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct FamilyProposalInput {
    candidate_id: String,
    proposed_lemma: String,
    part_of_speech: String,
    surfaces: Vec<FamilySurfaceInput>,
    token_frequency: usize,
    document_frequency: usize,
    documents: Vec<String>,
    possible_cells: Vec<String>,
    compatible_existing_lexemes: Vec<String>,
    dictionary_candidate_ids: Vec<String>,
    supporting_evidence: Vec<String>,
    contradicting_evidence: Vec<String>,
    missing_metadata: Vec<String>,
    confidence_basis_points: u16,
    assumptions: Vec<String>,
    review_status: String,
    review_reason: String,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineLock {
    schema_version: u8,
    milestone: &'static str,
    target_recension: &'static str,
    generation_policy: &'static str,
    orthography_profile: &'static str,
    tokenizer_contract: &'static str,
    corpus: BaselineCorpus,
    registry: BaselineRegistry,
    coverage: BaselineCoverage,
    evaluation: BaselineEvaluation,
    reviews: BaselineReviews,
    artifact_sha256: BTreeMap<&'static str, String>,
    sources: Vec<BaselineSource>,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineCorpus {
    passages: usize,
    tokens: usize,
    token_types: usize,
    source_ids: [&'static str; 2],
    partitions: [&'static str; 2],
}

#[derive(Clone, Debug, Serialize)]
struct BaselineRegistry {
    reviewed_lexemes: usize,
    reviewed_senses: usize,
    generated_exact_forms: usize,
    exact_only_lexemes: usize,
    fully_classed_lexemes: usize,
    partial_lexemes: usize,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineCoverage {
    top_1_analyzed: usize,
    top_k_analyzed: usize,
    ambiguous: usize,
    unresolved: usize,
    estimated_recovery_by_route: BTreeMap<&'static str, usize>,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineEvaluation {
    morphological_cells: usize,
    analytic_phrases: usize,
    abbreviation_cells: usize,
    expanded_top_k: usize,
    printed_top_k: usize,
    strict_top_k: usize,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineReviews {
    family_admitted: usize,
    family_deferred: usize,
    family_rejected: usize,
    current_top_200_decided: usize,
    lower_ranked_unreviewed: usize,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineSource {
    path: &'static str,
    sha256: &'static str,
    evaluation_passages: usize,
    source_passages: usize,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    let mut require_source_inputs = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            "--require-source-inputs" => require_source_inputs = true,
            value => {
                return Err(format!("unknown synodal-marginal-recovery argument {value:?}").into());
            }
        }
    }

    let coverage: CoverageReport = read_json(&root.join("reports/synodal-coverage.json"))?;
    validate_coverage_contract(&coverage)?;
    let proposals: Vec<FamilyProposalInput> =
        read_json(&root.join("reports/synodal-family-review-queue.json"))?;
    let candidates = proposals_to_candidates(&coverage, proposals)?;
    let mut report = marginal_recovery_report(
        coverage.summary.total_tokens,
        coverage.summary.top_k_analyzed,
        TARGET_BASIS_POINTS,
        coverage.generation_policy,
        coverage.orthography_profile,
        candidates,
    );
    report.milestones = coverage_milestones(&report);
    let json = format!("{}\n", serde_json::to_string_pretty(&report)?);
    let markdown = render_markdown(&report);
    let tsv = render_tsv(&report);

    let baseline_marginal_path = root.join(BASELINE_MARGINAL);
    if !baseline_marginal_path.is_file() {
        return Err(format!(
            "missing immutable {BASELINE_MARGINAL}; the sealed v0.4 baseline cannot be recreated from later state"
        )
        .into());
    }
    let baseline_marginal_sha256 = sha256_file(&baseline_marginal_path)?;
    let baseline_lock =
        render_baseline_lock(root, baseline_marginal_sha256, require_source_inputs)?;

    let outputs = [
        (root.join(CURRENT_JSON), json),
        (root.join(CURRENT_MARKDOWN), markdown),
        (root.join(CURRENT_TSV), tsv),
        (root.join(BASELINE_LOCK), baseline_lock),
    ];
    for (path, contents) in outputs {
        if check {
            check_contents_for(&path, &contents, "synodal-marginal-recovery")?;
        } else {
            write_if_changed_atomic(&path, &contents)?;
        }
    }

    println!(
        "Synodal marginal recovery: {} batches, {} overlap-adjusted diagnostic tokens, {} still needed for >70%",
        report.batches.len(),
        report.diagnostic_recovery,
        report.tokens_needed_for_target,
    );
    Ok(())
}

fn validate_coverage_contract(coverage: &CoverageReport) -> Result<(), Box<dyn Error>> {
    if coverage.target_recension != "synodal-russian"
        || coverage.generation_policy != GenerationPolicy::Strict
        || coverage.orthography_profile != OrthographyProfile::SynodalLiturgical
        || coverage.summary.total_tokens != 1_313_344
        || coverage.passages != 74_130
    {
        return Err(format!(
            "marginal recovery requires the locked v0.4 corpus contract; found target={}, policy={:?}, profile={:?}, passages={}, tokens={}",
            coverage.target_recension,
            coverage.generation_policy,
            coverage.orthography_profile,
            coverage.passages,
            coverage.summary.total_tokens,
        )
        .into());
    }
    Ok(())
}

fn proposals_to_candidates(
    coverage: &CoverageReport,
    proposals: Vec<FamilyProposalInput>,
) -> Result<Vec<RecoveryCandidateBatch>, Box<dyn Error>> {
    let mut gaps_by_surface: BTreeMap<&str, Vec<&GapRecord>> = BTreeMap::new();
    for gap in &coverage.gaps {
        gaps_by_surface
            .entry(gap.normalized.as_str())
            .or_default()
            .push(gap);
    }
    let gaps: BTreeMap<&str, &GapRecord> = gaps_by_surface
        .iter()
        .filter_map(|(surface, records)| records.first().map(|record| (*surface, *record)))
        .collect();
    let mut candidates = Vec::with_capacity(proposals.len());
    let mut documents_by_candidate = BTreeMap::new();
    for proposal in proposals {
        let calculated_frequency: usize = proposal
            .surfaces
            .iter()
            .map(|surface| surface.frequency)
            .sum();
        if calculated_frequency != proposal.token_frequency {
            return Err(format!(
                "family proposal {} reports {} tokens but surfaces sum to {}",
                proposal.candidate_id, proposal.token_frequency, calculated_frequency
            )
            .into());
        }
        if proposal.documents.len() > proposal.document_frequency {
            return Err(format!(
                "family proposal {} reports {} documents but lists {} representative documents",
                proposal.candidate_id,
                proposal.document_frequency,
                proposal.documents.len()
            )
            .into());
        }
        let route = dominant_route(&proposal, &gaps)?;
        let (evidence_readiness, review_effort) = review_priority(&proposal, route);
        let preserves_ambiguity = proposal.part_of_speech == "ambiguous"
            || proposal.compatible_existing_lexemes.len() > 1
            || proposal.contradicting_evidence.iter().any(|evidence| {
                evidence.contains("more than one lexical identity")
                    || evidence.contains("multiple lexical identities")
            });
        let mut recoverable_documents = BTreeSet::new();
        let recoverable_surfaces: Vec<_> = proposal
            .surfaces
            .into_iter()
            .filter_map(|surface| {
                let records = gaps_by_surface.get(surface.normalized.as_str())?;
                let mut frequency = 0_usize;
                for gap in records.iter() {
                    frequency = frequency.saturating_add(gap.top_k_uncovered_frequency);
                    recoverable_documents.extend(gap.top_k_uncovered_documents.iter().cloned());
                }
                (frequency > 0).then_some(RecoverySurfaceCandidate {
                    key: surface.normalized,
                    sample: surface.original,
                    frequency,
                })
            })
            .collect();
        if recoverable_surfaces.is_empty() {
            continue;
        }
        let candidate_id = proposal.candidate_id;
        let recoverable_document_frequency = recoverable_documents.len();
        documents_by_candidate.insert(
            candidate_id.clone(),
            recoverable_documents.into_iter().collect(),
        );
        candidates.push(RecoveryCandidateBatch {
            id: candidate_id.clone(),
            member_candidate_ids: vec![candidate_id],
            label: proposal.proposed_lemma,
            part_of_speech: proposal.part_of_speech,
            recovery_route: route.label().into(),
            document_frequency: recoverable_document_frequency,
            surfaces: recoverable_surfaces,
            compatible_lexeme_ids: proposal.compatible_existing_lexemes,
            proposed_cells: proposal.possible_cells,
            evidence_available: proposal.supporting_evidence,
            missing_evidence: proposal.missing_metadata,
            contradictions: proposal.contradicting_evidence,
            assumptions: proposal.assumptions,
            confidence_basis_points: proposal.confidence_basis_points,
            review_status: proposal.review_status,
            review_reason: proposal.review_reason,
            evidence_readiness,
            review_effort,
            preserves_ambiguity,
        });
    }
    add_consolidated_lexeme_batches(&mut candidates, &documents_by_candidate);
    Ok(candidates)
}

fn add_consolidated_lexeme_batches(
    candidates: &mut Vec<RecoveryCandidateBatch>,
    documents_by_candidate: &BTreeMap<String, Vec<String>>,
) {
    let mut groups: BTreeMap<String, Vec<RecoveryCandidateBatch>> = BTreeMap::new();
    for candidate in candidates.iter().filter(|candidate| {
        candidate.compatible_lexeme_ids.len() == 1
            && candidate.evidence_readiness != EvidenceReadiness::Blocked
    }) {
        groups
            .entry(candidate.compatible_lexeme_ids[0].clone())
            .or_default()
            .push(candidate.clone());
    }
    for (lexeme_id, mut members) in groups {
        if members.len() < 2 {
            continue;
        }
        members.sort_by(|left, right| left.id.cmp(&right.id));
        let member_candidate_ids: Vec<_> = members.iter().map(|member| member.id.clone()).collect();
        let documents: BTreeSet<_> = member_candidate_ids
            .iter()
            .flat_map(|id| {
                documents_by_candidate
                    .get(id)
                    .into_iter()
                    .flatten()
                    .cloned()
            })
            .collect();
        let mut route_tokens: BTreeMap<String, usize> = BTreeMap::new();
        for member in &members {
            *route_tokens
                .entry(member.recovery_route.clone())
                .or_default() += member
                .surfaces
                .iter()
                .map(|surface| surface.frequency)
                .sum::<usize>();
        }
        let recovery_route = route_tokens
            .into_iter()
            .max_by(|(left_route, left_tokens), (right_route, right_tokens)| {
                left_tokens
                    .cmp(right_tokens)
                    .then_with(|| right_route.cmp(left_route))
            })
            .map(|(route, _)| route)
            .unwrap_or_else(|| RecoveryRoute::UngroupedUnknown.label().into());
        let evidence_readiness = members
            .iter()
            .map(|member| member.evidence_readiness)
            .min()
            .unwrap_or(EvidenceReadiness::Weak);
        let review_effort = if members.len() <= 3 {
            ReviewEffort::Medium
        } else {
            ReviewEffort::Large
        };
        let mut assumptions = union_strings(&members, |member| &member.assumptions);
        assumptions.push(
            "consolidated batch overlap is diagnostic until every member identity and cell is reviewed"
                .into(),
        );
        assumptions.sort();
        assumptions.dedup();
        candidates.push(RecoveryCandidateBatch {
            id: format!("synodal:recovery-batch:lexeme:{lexeme_id}"),
            member_candidate_ids,
            label: format!("complete {lexeme_id} review batch"),
            part_of_speech: "mixed-family-batch".into(),
            recovery_route,
            document_frequency: documents.len(),
            surfaces: members
                .iter()
                .flat_map(|member| member.surfaces.iter().cloned())
                .collect(),
            compatible_lexeme_ids: vec![lexeme_id],
            proposed_cells: union_strings(&members, |member| &member.proposed_cells),
            evidence_available: union_strings(&members, |member| &member.evidence_available),
            missing_evidence: union_strings(&members, |member| &member.missing_evidence),
            contradictions: union_strings(&members, |member| &member.contradictions),
            assumptions,
            confidence_basis_points: members
                .iter()
                .map(|member| member.confidence_basis_points)
                .min()
                .unwrap_or_default(),
            review_status: "consolidated-diagnostic".into(),
            review_reason: format!(
                "Groups {} unresolved proposals linked to one reviewed lexeme so shared evidence can be reviewed once; member decisions remain authoritative.",
                members.len()
            ),
            evidence_readiness,
            review_effort,
            preserves_ambiguity: members.iter().any(|member| member.preserves_ambiguity),
        });
    }
}

fn union_strings(
    members: &[RecoveryCandidateBatch],
    values: impl Fn(&RecoveryCandidateBatch) -> &[String],
) -> Vec<String> {
    members
        .iter()
        .flat_map(values)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn dominant_route(
    proposal: &FamilyProposalInput,
    gaps: &BTreeMap<&str, &GapRecord>,
) -> Result<RecoveryRoute, Box<dyn Error>> {
    let mut frequencies: BTreeMap<RecoveryRoute, usize> = BTreeMap::new();
    for surface in &proposal.surfaces {
        let gap = gaps.get(surface.normalized.as_str()).ok_or_else(|| {
            format!(
                "family proposal {} surface {:?} is absent from the full gap membership",
                proposal.candidate_id, surface.normalized
            )
        })?;
        *frequencies.entry(route_for_gap(gap)).or_default() += surface.frequency;
    }
    frequencies
        .into_iter()
        .max_by(
            |(left_route, left_frequency), (right_route, right_frequency)| {
                left_frequency
                    .cmp(right_frequency)
                    .then_with(|| right_route.cmp(left_route))
            },
        )
        .map(|(route, _)| route)
        .ok_or_else(|| format!("family proposal {} has no surfaces", proposal.candidate_id).into())
}

fn route_for_gap(gap: &GapRecord) -> RecoveryRoute {
    match gap.kind {
        GapKind::MissingDeclensionOrClass => RecoveryRoute::ReviewedClass,
        GapKind::MissingVerbPrincipalPart => RecoveryRoute::ReviewedPrincipalPart,
        GapKind::UnsupportedFormation if !gap.candidate_lexeme_ids.is_empty() => {
            RecoveryRoute::UnsupportedFormation
        }
        GapKind::MissingAccentOrOrthographicMetadata | GapKind::AmbiguityOrSpellingVariant => {
            RecoveryRoute::SpellingVariant
        }
        GapKind::UnknownLexeme if has_abbreviation_marks(&gap.original) => {
            RecoveryRoute::AbbreviationRegistry
        }
        GapKind::UnknownLexeme => RecoveryRoute::UngroupedUnknown,
        GapKind::UnsupportedFormation => RecoveryRoute::ExactEvidence,
    }
}

fn review_priority(
    proposal: &FamilyProposalInput,
    route: RecoveryRoute,
) -> (EvidenceReadiness, ReviewEffort) {
    if proposal.review_status == "rejected" {
        return (EvidenceReadiness::Blocked, ReviewEffort::Small);
    }
    let linked = !proposal.compatible_existing_lexemes.is_empty();
    let only_orthography_missing = !proposal.missing_metadata.is_empty()
        && proposal.missing_metadata.iter().all(|field| {
            matches!(
                field.as_str(),
                "accent-or-printed-orthography" | "accentclass" | "positional-metadata"
            )
        });
    if linked && only_orthography_missing {
        return (EvidenceReadiness::Ready, ReviewEffort::Small);
    }
    if linked && proposal.compatible_existing_lexemes.len() == 1 {
        return (EvidenceReadiness::Partial, ReviewEffort::Medium);
    }
    if route == RecoveryRoute::AbbreviationRegistry {
        return (EvidenceReadiness::Partial, ReviewEffort::Medium);
    }
    if linked || !proposal.dictionary_candidate_ids.is_empty() {
        return (EvidenceReadiness::Partial, ReviewEffort::Large);
    }
    (EvidenceReadiness::Weak, ReviewEffort::Large)
}

fn has_abbreviation_marks(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(character, '\u{0483}' | '\u{0487}')
            || ('\u{2de0}'..='\u{2dff}').contains(&character)
    })
}

fn render_baseline_lock(
    root: &Path,
    baseline_marginal_sha256: String,
    require_source_inputs: bool,
) -> Result<String, Box<dyn Error>> {
    let mut sources = Vec::new();
    for (path, expected_sha256, evaluation_passages, source_passages) in SOURCE_LOCKS {
        let source_path = root.join(path);
        if source_path.exists() {
            let actual = sha256_file(&source_path)?;
            if actual != expected_sha256 {
                return Err(format!(
                    "locked v0.4 source input {path} drifted: expected {expected_sha256}, found {actual}"
                )
                .into());
            }
        } else if require_source_inputs {
            return Err(format!("locked v0.4 source input {path} is unavailable").into());
        }
        sources.push(BaselineSource {
            path,
            sha256: expected_sha256,
            evaluation_passages,
            source_passages,
        });
    }
    let mut estimated_recovery_by_route = BTreeMap::new();
    estimated_recovery_by_route.insert("abbreviation-registry", 42_115);
    estimated_recovery_by_route.insert("exact-evidence", 8_655);
    estimated_recovery_by_route.insert("reviewed-class", 1_092);
    estimated_recovery_by_route.insert("reviewed-principal-part", 0);
    estimated_recovery_by_route.insert("spelling-variant", 60_053);
    estimated_recovery_by_route.insert("ungrouped-unknown", 645_324);
    estimated_recovery_by_route.insert("unsupported-formation", 0);
    let mut artifact_sha256 = BTreeMap::new();
    artifact_sha256.insert("reports/synodal-coverage.json", COVERAGE_SHA256.into());
    artifact_sha256.insert("reports/synodal-evaluation.json", EVALUATION_SHA256.into());
    artifact_sha256.insert(
        "reports/synodal-family-review-queue.json",
        FAMILY_QUEUE_SHA256.into(),
    );
    artifact_sha256.insert(
        "reports/synodal-v04-marginal-recovery.json",
        baseline_marginal_sha256,
    );
    artifact_sha256.insert(
        "data/synodal/family_reviews.tsv",
        FAMILY_REVIEWS_SHA256.into(),
    );
    artifact_sha256.insert(
        "data/synodal/lexical_reviews.tsv",
        LEXICAL_REVIEWS_SHA256.into(),
    );
    artifact_sha256.insert(
        "crates/synodal-church-slavonic/generated/registry.rs",
        MORPHOLOGY_REGISTRY_SHA256.into(),
    );
    artifact_sha256.insert(
        "crates/synodal-church-slavonic-dictionary/generated/registry.rs",
        DICTIONARY_REGISTRY_SHA256.into(),
    );
    let lock = BaselineLock {
        schema_version: 1,
        milestone: "synodal-v0.4",
        target_recension: "synodal-russian",
        generation_policy: "strict",
        orthography_profile: "synodal-liturgical",
        tokenizer_contract: "synodal-dictionary-tokenize-v1",
        corpus: BaselineCorpus {
            passages: 74_130,
            tokens: 1_313_344,
            token_types: 57_476,
            source_ids: [
                "ponomar-elizabeth-bible-2026-08-09",
                "wikisource-church-slavonic-bible-2026-08-09",
            ],
            partitions: ["evaluation", "source"],
        },
        registry: BaselineRegistry {
            reviewed_lexemes: 506,
            reviewed_senses: 506,
            generated_exact_forms: 774,
            exact_only_lexemes: 457,
            fully_classed_lexemes: 46,
            partial_lexemes: 3,
        },
        coverage: BaselineCoverage {
            top_1_analyzed: 430_470,
            top_k_analyzed: 569_418,
            ambiguous: 13_510,
            unresolved: 742_721,
            estimated_recovery_by_route,
        },
        evaluation: BaselineEvaluation {
            morphological_cells: 445,
            analytic_phrases: 5,
            abbreviation_cells: 7,
            expanded_top_k: 445,
            printed_top_k: 445,
            strict_top_k: 444,
        },
        reviews: BaselineReviews {
            family_admitted: 5,
            family_deferred: 197,
            family_rejected: 3,
            current_top_200_decided: 200,
            lower_ranked_unreviewed: 800,
        },
        artifact_sha256,
        sources,
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&lock)?))
}

fn render_markdown(report: &MarginalRecoveryReport) -> String {
    let mut out = format!(
        "# Synodal marginal top-k recovery\n\nThis is a diagnostic counterfactual under `Strict` and `SynodalLiturgical`; it does not count a proposal as an analysis. Only canonical resolver output changes actual coverage.\n\n- Corpus tokens: {}\n- Current top-k: {}\n- Canonical target (>70%): {}\n- Tokens still needed for >70%: {}\n- Overlap-adjusted diagnostic potential in this queue: {}\n- Diagnostic projected top-k if every batch were valid: {}\n\n| Milestone | Minimum top-k | Tokens needed | Margin |\n|---:|---:|---:|---:|\n",
        report.total_tokens,
        report.current_top_k,
        report.target_top_k,
        report.tokens_needed_for_target,
        report.diagnostic_recovery,
        report.diagnostic_projected_top_k,
    );
    for milestone in coverage_milestones(report) {
        out.push_str(&format!(
            "| {}% | {} | {} | {} |\n",
            milestone.percent, milestone.target_top_k, milestone.tokens_needed, milestone.margin,
        ));
    }
    out.push_str("\n| Rank | Batch | Route | Raw | Unique | Marginal | Cumulative | Readiness | Effort | Review status |\n|---:|---|---|---:|---:|---:|---:|---|---|---|\n");
    for batch in &report.batches {
        out.push_str(&format!(
            "| {} | `{}` | `{}` | {} | {} | {} | {} | `{:?}` | `{:?}` | `{}` |\n",
            batch.rank,
            escape_markdown(&batch.label),
            batch.recovery_route,
            batch.raw_token_frequency,
            batch.unique_gap_tokens,
            batch.overlap_adjusted_tokens,
            batch.cumulative_overlap_adjusted_tokens,
            batch.evidence_readiness,
            batch.review_effort,
            escape_markdown(&batch.review_status),
        ));
    }
    out
}

fn coverage_milestones(report: &MarginalRecoveryReport) -> Vec<CoverageMilestone> {
    MILESTONE_BASIS_POINTS
        .into_iter()
        .map(|basis_points| {
            let target_top_k = strict_threshold(report.total_tokens, basis_points);
            CoverageMilestone {
                percent: basis_points / 100,
                basis_points,
                target_top_k,
                tokens_needed: target_top_k.saturating_sub(report.current_top_k),
                margin: report.current_top_k.saturating_sub(target_top_k),
            }
        })
        .collect()
}

fn strict_threshold(total_tokens: usize, basis_points: usize) -> usize {
    total_tokens
        .saturating_mul(basis_points)
        .checked_div(10_000)
        .unwrap_or(total_tokens)
        .saturating_add(1)
}

fn render_tsv(report: &MarginalRecoveryReport) -> String {
    let mut out = String::from(
        "rank\tbatch_id\tmember_candidate_ids\tlabel\tpart_of_speech\trecovery_route\traw_tokens\tunique_gap_tokens\toverlap_adjusted_tokens\tcumulative_tokens\tdiagnostic_score\tdocument_frequency\texpected_top_1_gain\texpected_top_k_gain\texpected_ambiguity_gain\texpected_abstention_reduction\tevidence_readiness\treview_effort\treview_status\tcompatible_lexeme_ids\tproposed_cells\tmissing_evidence\toverlaps\treview_reason\n",
    );
    for batch in &report.batches {
        let overlaps = batch
            .overlap_with_higher_batches
            .iter()
            .map(|overlap| format!("{}:{}", overlap.higher_batch_id, overlap.tokens))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            batch.rank,
            tsv_field(&batch.id),
            tsv_field(&batch.member_candidate_ids.join(",")),
            tsv_field(&batch.label),
            tsv_field(&batch.part_of_speech),
            batch.recovery_route,
            batch.raw_token_frequency,
            batch.unique_gap_tokens,
            batch.overlap_adjusted_tokens,
            batch.cumulative_overlap_adjusted_tokens,
            batch.diagnostic_score,
            batch.document_frequency,
            batch.expected_top_1_gain,
            batch.expected_top_k_gain,
            batch.expected_ambiguity_gain,
            batch.expected_abstention_reduction,
            batch.evidence_readiness,
            batch.review_effort,
            tsv_field(&batch.review_status),
            tsv_field(&batch.compatible_lexeme_ids.join(",")),
            tsv_field(&batch.proposed_cells.join(",")),
            tsv_field(&batch.missing_evidence.join(",")),
            tsv_field(&overlaps),
            tsv_field(&batch.review_reason),
        ));
    }
    out
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn tsv_field(value: &str) -> String {
    value
        .replace('\t', " ")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_prefers_linked_orthographic_work() {
        let proposal = FamilyProposalInput {
            candidate_id: "fixture".into(),
            proposed_lemma: "fixture".into(),
            part_of_speech: "noun".into(),
            surfaces: Vec::new(),
            token_frequency: 0,
            document_frequency: 0,
            documents: Vec::new(),
            possible_cells: Vec::new(),
            compatible_existing_lexemes: vec!["synodal:noun:fixture".into()],
            dictionary_candidate_ids: Vec::new(),
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
            missing_metadata: vec!["accentclass".into()],
            confidence_basis_points: 9_000,
            assumptions: Vec::new(),
            review_status: "deferred".into(),
            review_reason: "fixture".into(),
        };
        assert_eq!(
            review_priority(&proposal, RecoveryRoute::SpellingVariant),
            (EvidenceReadiness::Ready, ReviewEffort::Small)
        );
    }

    #[test]
    fn rejected_false_grouping_is_blocked() {
        let proposal = FamilyProposalInput {
            candidate_id: "fixture".into(),
            proposed_lemma: "fixture".into(),
            part_of_speech: "ambiguous".into(),
            surfaces: Vec::new(),
            token_frequency: 0,
            document_frequency: 0,
            documents: Vec::new(),
            possible_cells: Vec::new(),
            compatible_existing_lexemes: Vec::new(),
            dictionary_candidate_ids: Vec::new(),
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
            missing_metadata: Vec::new(),
            confidence_basis_points: 0,
            assumptions: Vec::new(),
            review_status: "rejected".into(),
            review_reason: "fixture".into(),
        };
        assert_eq!(
            review_priority(&proposal, RecoveryRoute::UngroupedUnknown),
            (EvidenceReadiness::Blocked, ReviewEffort::Small)
        );
    }

    #[test]
    fn target_contract_rejects_denominator_drift() {
        let report = CoverageReport {
            schema_version: 4,
            target_recension: "synodal-russian".into(),
            generation_policy: GenerationPolicy::Strict,
            orthography_profile: OrthographyProfile::SynodalLiturgical,
            passages: 74_130,
            token_types: 0,
            integrity: synodal_church_slavonic_dictionary::coverage::CoverageIntegrity::default(),
            held_out_types: 0,
            held_out_type_coverage:
                synodal_church_slavonic_dictionary::coverage::CoverageSlice::default(),
            held_out_type_status: BTreeMap::new(),
            summary: synodal_church_slavonic_dictionary::coverage::CoverageSlice {
                total_tokens: 1,
                ..Default::default()
            },
            by_corpus: BTreeMap::new(),
            by_source: BTreeMap::new(),
            by_partition: BTreeMap::new(),
            by_source_partition: BTreeMap::new(),
            by_policy: BTreeMap::new(),
            by_lexeme: BTreeMap::new(),
            by_family: BTreeMap::new(),
            by_morphological_system: BTreeMap::new(),
            by_corpus_gap: BTreeMap::new(),
            by_source_gap: BTreeMap::new(),
            by_partition_gap: BTreeMap::new(),
            by_source_partition_gap: BTreeMap::new(),
            by_status: BTreeMap::new(),
            by_gap: BTreeMap::new(),
            unresolved_by_probable_family: BTreeMap::new(),
            estimated_recovery_by_route: BTreeMap::new(),
            abbreviation_family_tokens: 0,
            spelling_variant_family_tokens: 0,
            remaining_ungrouped_unknowns: 0,
            gap_frequency_by_surface: BTreeMap::new(),
            top_k_uncovered_frequency_by_surface: BTreeMap::new(),
            total_gap_types: 0,
            gaps: Vec::new(),
            review_queue: Vec::new(),
            uncovered_frontier: Vec::new(),
        };
        assert!(validate_coverage_contract(&report).is_err());
    }
}
