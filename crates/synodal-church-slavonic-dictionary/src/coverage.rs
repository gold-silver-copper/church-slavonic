//! Indexed text analysis and deterministic corpus-coverage reporting.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    GenerationPolicy, GrammarCell, Inflector, LexemeId, LexemeSummary, MetadataField,
    OrthographyProfile, PartOfSpeech, Result, SynodalWord, capabilities_by_id, lexemes,
    lexical_metadata, missing_metadata_by_id,
};
use synodal_church_slavonic_core::{
    RuleTrace, normalize_lookup, normalize_lookup_accentless, parse_cyrillic_numeral,
};
use unicode_normalization::char::is_combining_mark;

#[cfg(test)]
use crate::candidate_cells;
use crate::{Analysis, AnalysisSource, FamilyId, analysis_cells_for_lexeme, analysis_source};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GapKind {
    UnknownLexeme,
    MissingDeclensionOrClass,
    MissingVerbPrincipalPart,
    UnsupportedFormation,
    MissingAccentOrOrthographicMetadata,
    AmbiguityOrSpellingVariant,
}

impl GapKind {
    pub const ALL: [Self; 6] = [
        Self::UnknownLexeme,
        Self::MissingDeclensionOrClass,
        Self::MissingVerbPrincipalPart,
        Self::UnsupportedFormation,
        Self::MissingAccentOrOrthographicMetadata,
        Self::AmbiguityOrSpellingVariant,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::UnknownLexeme => "unknown-lexeme",
            Self::MissingDeclensionOrClass => "missing-declension-or-class",
            Self::MissingVerbPrincipalPart => "missing-verb-principal-part",
            Self::UnsupportedFormation => "unsupported-formation",
            Self::MissingAccentOrOrthographicMetadata => "missing-accent-or-orthographic-metadata",
            Self::AmbiguityOrSpellingVariant => "ambiguity-or-spelling-variant",
        }
    }

    /// Stable primary-gap precedence. Lower values win: ambiguity/variant,
    /// orthography, verb principal parts, nominal class, unsupported rule,
    /// then a genuinely unknown lexeme.
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::AmbiguityOrSpellingVariant => 0,
            Self::MissingAccentOrOrthographicMetadata => 1,
            Self::MissingVerbPrincipalPart => 2,
            Self::MissingDeclensionOrClass => 3,
            Self::UnsupportedFormation => 4,
            Self::UnknownLexeme => 5,
        }
    }
}

/// Selects one deterministic primary reason while preserving all remaining
/// reasons as secondary diagnostics in the caller's original record.
#[must_use]
pub fn primary_gap(reasons: impl IntoIterator<Item = GapKind>) -> Option<GapKind> {
    reasons.into_iter().min_by_key(|reason| reason.precedence())
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenStatus {
    ExactSynodalAttestation,
    SynodalIrregularOverride,
    SynodalNormativeTable,
    SynodalProductiveRule,
    CallerSpecifiedPrediction,
    InheritedPrediction,
    AnalogicalPrediction,
    AbbreviationExpansion,
    SpellingVariant,
    Ambiguous,
    Unresolved,
    CyrillicNumeral,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextToken {
    pub original: String,
    pub normalized: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GapOccurrence {
    pub kind: GapKind,
    pub secondary_reasons: Vec<GapKind>,
    pub detail: String,
    pub candidate_lexeme_ids: Vec<LexemeId>,
    pub requested_morphological_system: Option<String>,
    pub missing_metadata: Vec<MetadataField>,
    pub resolver_trace: RuleTrace,
    pub suggested_action: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextTokenAnalysis {
    pub token: TextToken,
    pub status: TokenStatus,
    pub analyses: Vec<Analysis>,
    pub gap: Option<GapOccurrence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckTextOptions {
    pub generation_policy: GenerationPolicy,
    pub orthography_profile: OrthographyProfile,
}

impl Default for CheckTextOptions {
    fn default() -> Self {
        Self {
            generation_policy: GenerationPolicy::Strict,
            orthography_profile: OrthographyProfile::Expanded,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextSummary {
    pub total_tokens: usize,
    pub unique_tokens: usize,
    pub top_1_analyzed: usize,
    pub top_k_analyzed: usize,
    pub ambiguous_tokens: usize,
    pub unresolved_tokens: usize,
    pub numerals: usize,
    pub by_status: BTreeMap<String, usize>,
    pub by_gap: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextReport {
    pub schema_version: u8,
    pub options: CheckTextOptions,
    pub summary: TextSummary,
    pub tokens: Vec<TextTokenAnalysis>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoveragePassage {
    pub corpus: String,
    pub source_id: String,
    pub work: String,
    pub edition: String,
    pub passage: String,
    pub partition: String,
    pub source_recension: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GapContext {
    pub document: String,
    pub passage: String,
    pub line: usize,
    pub column: usize,
    pub excerpt: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageSlice {
    pub total_tokens: usize,
    pub top_1_analyzed: usize,
    pub top_k_analyzed: usize,
    pub ambiguous: usize,
    pub unresolved: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GapRecord {
    pub kind: GapKind,
    pub original: String,
    pub normalized: String,
    pub corpus: String,
    pub source_id: String,
    pub work: String,
    pub edition: String,
    pub passage: String,
    pub partition: String,
    pub source_recension: String,
    /// Every corpus represented by this frequency aggregate. Singular
    /// provenance fields above identify the retained sample occurrence.
    pub corpora: Vec<String>,
    pub source_ids: Vec<String>,
    pub editions: Vec<String>,
    pub partitions: Vec<String>,
    pub source_recensions: Vec<String>,
    /// Stable source-and-passage document identities represented by this
    /// aggregate. This permits family queues to calculate a true union rather
    /// than summing per-surface document frequencies.
    #[serde(default)]
    pub documents: Vec<String>,
    /// Bounded, deterministic samples showing the token in corpus context.
    /// These are review aids only and never establish a lexical identity.
    #[serde(default)]
    pub contexts: Vec<GapContext>,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
    pub column: usize,
    pub candidate_lexeme_ids: Vec<LexemeId>,
    pub requested_morphological_system: Option<String>,
    pub generation_policy: GenerationPolicy,
    pub orthography_profile: OrthographyProfile,
    pub resolver_trace: RuleTrace,
    pub missing_metadata: Vec<MetadataField>,
    pub secondary_reasons: Vec<GapKind>,
    pub detail: String,
    pub frequency: usize,
    pub document_frequency: usize,
    /// Occurrences in this aggregate that do not currently receive any
    /// policy-valid top-k analysis. This distinguishes uncovered spelling
    /// variants from genuine multi-lexeme ambiguity, which uses the same gap
    /// kind but is already covered in top-k.
    #[serde(default)]
    pub top_k_uncovered_frequency: usize,
    #[serde(default)]
    pub top_k_uncovered_documents: Vec<String>,
    pub suggested_action: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryRoute {
    ExactEvidence,
    ReviewedClass,
    ReviewedPrincipalPart,
    AbbreviationRegistry,
    SpellingVariant,
    UnsupportedFormation,
    UngroupedUnknown,
}

impl RecoveryRoute {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ExactEvidence => "exact-evidence",
            Self::ReviewedClass => "reviewed-class",
            Self::ReviewedPrincipalPart => "reviewed-principal-part",
            Self::AbbreviationRegistry => "abbreviation-registry",
            Self::SpellingVariant => "spelling-variant",
            Self::UnsupportedFormation => "unsupported-formation",
            Self::UngroupedUnknown => "ungrouped-unknown",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProbableFamilyDiagnostic {
    pub probable_family_id: String,
    pub token_frequency: usize,
    pub top_k_uncovered_token_frequency: usize,
    /// Source-and-passage identities represented by the top-k-uncovered
    /// occurrences, excluding documents that contribute only covered ambiguity.
    pub document_frequency: usize,
    pub surfaces: Vec<String>,
    pub candidate_lexeme_ids: Vec<LexemeId>,
    pub recovery_route: RecoveryRoute,
    pub assumption: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewQueueItem {
    pub rank: usize,
    pub kind: GapKind,
    pub normalized: String,
    pub sample: String,
    pub frequency: usize,
    pub document_frequency: usize,
    pub candidate_lexeme_ids: Vec<LexemeId>,
    pub suggested_action: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub schema_version: u8,
    pub target_recension: String,
    pub generation_policy: GenerationPolicy,
    pub orthography_profile: OrthographyProfile,
    pub passages: usize,
    pub token_types: usize,
    pub summary: CoverageSlice,
    pub by_corpus: BTreeMap<String, CoverageSlice>,
    pub by_source: BTreeMap<String, CoverageSlice>,
    pub by_policy: BTreeMap<String, CoverageSlice>,
    pub by_lexeme: BTreeMap<String, CoverageSlice>,
    pub by_family: BTreeMap<String, CoverageSlice>,
    pub by_morphological_system: BTreeMap<String, CoverageSlice>,
    pub by_corpus_gap: BTreeMap<String, BTreeMap<String, usize>>,
    pub by_source_gap: BTreeMap<String, BTreeMap<String, usize>>,
    pub by_status: BTreeMap<String, usize>,
    pub by_gap: BTreeMap<String, usize>,
    /// Diagnostic grouping only. Entries do not establish lexical identity and
    /// multiple candidates remain explicit.
    pub unresolved_by_probable_family: BTreeMap<String, ProbableFamilyDiagnostic>,
    pub estimated_recovery_by_route: BTreeMap<String, usize>,
    pub abbreviation_family_tokens: usize,
    pub spelling_variant_family_tokens: usize,
    pub remaining_ungrouped_unknowns: usize,
    /// Compact frequency membership for every surface that still carries a
    /// gap. Unlike `gaps`, this map is not truncated for human review output,
    /// so later milestone reports can attribute realized recovery exactly on
    /// an unchanged corpus denominator.
    pub gap_frequency_by_surface: BTreeMap<String, usize>,
    /// Complete per-surface frequency of occurrences that remain outside
    /// top-k. Unlike `gap_frequency_by_surface`, covered ambiguity is excluded.
    pub top_k_uncovered_frequency_by_surface: BTreeMap<String, usize>,
    pub total_gap_types: usize,
    pub gaps: Vec<GapRecord>,
    pub review_queue: Vec<ReviewQueueItem>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceReadiness {
    Blocked,
    Weak,
    Partial,
    Ready,
}

impl EvidenceReadiness {
    #[must_use]
    pub const fn weight(self) -> usize {
        match self {
            Self::Blocked => 0,
            Self::Weak => 1,
            Self::Partial => 3,
            Self::Ready => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewEffort {
    Small,
    Medium,
    Large,
}

impl ReviewEffort {
    #[must_use]
    pub const fn weight(self) -> usize {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoverySurfaceCandidate {
    /// Stable analyzer lookup key. The same key in two batches denotes the
    /// same corpus-token membership and must not be counted twice.
    pub key: String,
    pub sample: String,
    pub frequency: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryCandidateBatch {
    pub id: String,
    pub member_candidate_ids: Vec<String>,
    pub label: String,
    pub part_of_speech: String,
    pub recovery_route: String,
    pub document_frequency: usize,
    pub surfaces: Vec<RecoverySurfaceCandidate>,
    pub compatible_lexeme_ids: Vec<String>,
    pub proposed_cells: Vec<String>,
    pub evidence_available: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub contradictions: Vec<String>,
    pub assumptions: Vec<String>,
    pub confidence_basis_points: u16,
    pub review_status: String,
    pub review_reason: String,
    pub evidence_readiness: EvidenceReadiness,
    pub review_effort: ReviewEffort,
    /// True when independently supported alternatives are expected to remain
    /// visible instead of being forced into one top-1 analysis.
    pub preserves_ambiguity: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarginalOverlap {
    pub higher_batch_id: String,
    pub tokens: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarginalRecoveryBatch {
    pub rank: usize,
    pub id: String,
    pub member_candidate_ids: Vec<String>,
    pub label: String,
    pub part_of_speech: String,
    pub recovery_route: String,
    pub raw_token_frequency: usize,
    pub unique_gap_tokens: usize,
    pub document_frequency: usize,
    pub overlap_with_higher_batches: Vec<MarginalOverlap>,
    pub overlap_adjusted_tokens: usize,
    pub cumulative_overlap_adjusted_tokens: usize,
    pub diagnostic_score: usize,
    pub expected_top_1_gain: usize,
    pub expected_top_k_gain: usize,
    pub expected_ambiguity_gain: usize,
    pub expected_abstention_reduction: usize,
    pub compatible_lexeme_ids: Vec<String>,
    pub proposed_cells: Vec<String>,
    pub evidence_available: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub contradictions: Vec<String>,
    pub assumptions: Vec<String>,
    pub confidence_basis_points: u16,
    pub review_status: String,
    pub review_reason: String,
    pub evidence_readiness: EvidenceReadiness,
    pub review_effort: ReviewEffort,
    pub preserves_ambiguity: bool,
    pub surfaces: Vec<RecoverySurfaceCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageMilestone {
    pub percent: usize,
    pub basis_points: usize,
    pub target_top_k: usize,
    pub tokens_needed: usize,
    pub margin: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarginalRecoveryReport {
    pub schema_version: u8,
    pub target_recension: String,
    pub generation_policy: GenerationPolicy,
    pub orthography_profile: OrthographyProfile,
    pub total_tokens: usize,
    pub current_top_k: usize,
    pub target_top_k: usize,
    pub tokens_needed_for_target: usize,
    #[serde(default)]
    pub milestones: Vec<CoverageMilestone>,
    pub diagnostic_recovery: usize,
    pub diagnostic_projected_top_k: usize,
    pub batches: Vec<MarginalRecoveryBatch>,
}

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

fn surface_membership(candidate: &RecoveryCandidateBatch) -> BTreeMap<String, usize> {
    let mut membership = BTreeMap::new();
    for surface in &candidate.surfaces {
        membership
            .entry(surface.key.clone())
            .and_modify(|frequency: &mut usize| *frequency = (*frequency).max(surface.frequency))
            .or_insert(surface.frequency);
    }
    membership
}

fn unique_surface_frequency(candidate: &RecoveryCandidateBatch) -> usize {
    surface_membership(candidate).values().sum()
}

fn marginal_tokens(candidate: &RecoveryCandidateBatch, covered: &BTreeMap<String, usize>) -> usize {
    surface_membership(candidate)
        .iter()
        .map(|(key, frequency)| frequency.saturating_sub(covered.get(key).copied().unwrap_or(0)))
        .sum()
}

fn diagnostic_score(marginal: usize, candidate: &RecoveryCandidateBatch) -> usize {
    marginal
        .saturating_mul(candidate.evidence_readiness.weight())
        .checked_div(candidate.review_effort.weight())
        .unwrap_or_default()
}

/// Reusable reverse-analysis index. Building it is deliberately explicit so a
/// corpus run pays the paradigm-enumeration cost once rather than once per
/// token.
#[derive(Clone, Debug)]
pub struct Analyzer {
    inflector: Inflector,
    indexed_cells: usize,
    expanded_marked: BTreeMap<String, Vec<Analysis>>,
    expanded: BTreeMap<String, Vec<Analysis>>,
    printed_marked: BTreeMap<String, Vec<Analysis>>,
    printed: BTreeMap<String, Vec<Analysis>>,
    spelling_candidates: BTreeMap<String, BTreeSet<LexemeId>>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AnalyzerConfig {
    generation_policy: GenerationPolicy,
    orthography: OrthographyProfile,
    productive_mapping_threshold_basis_points: u16,
}

impl From<Inflector> for AnalyzerConfig {
    fn from(inflector: Inflector) -> Self {
        Self {
            generation_policy: inflector.generation_policy(),
            orthography: inflector.orthography(),
            productive_mapping_threshold_basis_points: inflector
                .productive_mapping_threshold_basis_points(),
        }
    }
}

/// Process-local cache for immutable analyzers. Callers choose its lifetime,
/// so custom configurations never leak into unrelated processes or tests.
#[derive(Debug, Default)]
pub struct AnalyzerCache {
    analyzers: Mutex<BTreeMap<AnalyzerConfig, Arc<Analyzer>>>,
    constructions: std::sync::atomic::AtomicUsize,
}

impl AnalyzerCache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            analyzers: Mutex::new(BTreeMap::new()),
            constructions: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn get(&self, inflector: Inflector) -> Result<Arc<Analyzer>> {
        let key = AnalyzerConfig::from(inflector);
        let mut analyzers = match self.analyzers.lock() {
            Ok(analyzers) => analyzers,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(analyzer) = analyzers.get(&key) {
            return Ok(Arc::clone(analyzer));
        }
        let analyzer = Arc::new(Analyzer::new(inflector)?);
        self.constructions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        analyzers.insert(key, Arc::clone(&analyzer));
        Ok(analyzer)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self.analyzers.lock() {
            Ok(analyzers) => analyzers.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of successful analyzer constructions performed by this cache.
    #[must_use]
    pub fn construction_count(&self) -> usize {
        self.constructions
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

static DEFAULT_ANALYZERS: OnceLock<AnalyzerCache> = OnceLock::new();

pub(crate) fn default_analyzer() -> Result<Arc<Analyzer>> {
    DEFAULT_ANALYZERS
        .get_or_init(AnalyzerCache::new)
        .get(Inflector::default())
}

impl Analyzer {
    pub fn new(inflector: Inflector) -> Result<Self> {
        let mut analyzer = Self {
            inflector,
            indexed_cells: 0,
            expanded_marked: BTreeMap::new(),
            expanded: BTreeMap::new(),
            printed_marked: BTreeMap::new(),
            printed: BTreeMap::new(),
            spelling_candidates: BTreeMap::new(),
        };
        let expanded_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::Expanded)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        let printed_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::SynodalLiturgical)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        for lexeme in lexemes()? {
            analyzer
                .spelling_candidates
                .entry(spelling_key(lexeme.lemma()))
                .or_default()
                .insert(lexeme.id().clone());
            for cell in analysis_cells_for_lexeme(&lexeme, inflector)? {
                analyzer.indexed_cells += 1;
                analyzer.index_cell(&lexeme, cell, expanded_inflector);
                analyzer.index_cell(&lexeme, cell, printed_inflector);
            }
        }
        for index in [
            &mut analyzer.expanded_marked,
            &mut analyzer.expanded,
            &mut analyzer.printed_marked,
            &mut analyzer.printed,
        ] {
            sort_index(index);
        }
        Ok(analyzer)
    }

    #[cfg(test)]
    fn new_exhaustive(inflector: Inflector) -> Result<Self> {
        let mut analyzer = Self {
            inflector,
            indexed_cells: 0,
            expanded_marked: BTreeMap::new(),
            expanded: BTreeMap::new(),
            printed_marked: BTreeMap::new(),
            printed: BTreeMap::new(),
            spelling_candidates: BTreeMap::new(),
        };
        let expanded_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::Expanded)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        let printed_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::SynodalLiturgical)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        for lexeme in lexemes()? {
            analyzer
                .spelling_candidates
                .entry(spelling_key(lexeme.lemma()))
                .or_default()
                .insert(lexeme.id().clone());
            for cell in candidate_cells(lexeme.part_of_speech()) {
                analyzer.indexed_cells += 1;
                analyzer.index_cell(&lexeme, cell, expanded_inflector);
                analyzer.index_cell(&lexeme, cell, printed_inflector);
            }
        }
        for index in [
            &mut analyzer.expanded_marked,
            &mut analyzer.expanded,
            &mut analyzer.printed_marked,
            &mut analyzer.printed,
        ] {
            sort_index(index);
        }
        Ok(analyzer)
    }

    #[must_use]
    pub const fn inflector(&self) -> Inflector {
        self.inflector
    }

    /// Number of per-lexeme typed cells admitted to this reverse index.
    #[must_use]
    pub const fn indexed_cell_count(&self) -> usize {
        self.indexed_cells
    }

    pub fn analyze(&self, word: &str) -> Result<Vec<Analysis>> {
        let mut analyses = self.analyze_profile(word, OrthographyProfile::Expanded)?;
        analyses.extend(self.analyze_profile(word, OrthographyProfile::SynodalLiturgical)?);
        deduplicate_analyses(&mut analyses);
        Ok(analyses)
    }

    pub(crate) fn analyze_dictionary(&self, word: &str) -> Result<Vec<Analysis>> {
        let parsed = SynodalWord::parse(word)?;
        let marked_key = normalize_lookup(parsed.canonical());
        let key = normalize_lookup_accentless(parsed.canonical());
        let allow_fallback = marked_key == key
            || self.inflector.orthography() == OrthographyProfile::ExpandedAccentless;
        let mut analyses = self
            .expanded_marked
            .get(&marked_key)
            .into_iter()
            .flatten()
            .cloned()
            .chain(
                self.printed_marked
                    .get(&marked_key)
                    .into_iter()
                    .flatten()
                    .cloned(),
            )
            .collect::<Vec<_>>();
        let used_fallback = analyses.is_empty() && allow_fallback;
        if used_fallback {
            analyses.extend(self.expanded.get(&key).into_iter().flatten().cloned());
            analyses.extend(self.printed.get(&key).into_iter().flatten().cloned());
        }
        if let Ok(expansions) = crate::morphology::abbreviation::expand(parsed.canonical()) {
            if used_fallback && !expansions.is_empty() {
                analyses.clear();
            }
            for expansion in expansions {
                let lexeme = crate::morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
                analyses.push(Analysis {
                    lexeme,
                    cell: Some(expansion.cell),
                    matched_text: parsed.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: synodal_church_slavonic_core::Confidence::CERTAIN,
                    evidence_ids: expansion
                        .evidence_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    assumptions: Vec::new(),
                    contradictions: Vec::new(),
                    warnings: Vec::new(),
                    rule_trace: RuleTrace::default(),
                });
            }
        }
        let mut best_by_analysis = BTreeMap::new();
        for mut analysis in analyses {
            if used_fallback && analysis.source != AnalysisSource::AbbreviationExpansion {
                analysis
                    .warnings
                    .push("analysis required accent-insensitive matching".into());
            }
            let key = (
                analysis.lexeme.id().clone(),
                analysis.cell,
                analysis.source,
                analysis.recension_mapping.clone(),
            );
            best_by_analysis.entry(key).or_insert(analysis);
        }
        Ok(best_by_analysis.into_values().collect())
    }

    pub fn analyze_profile(
        &self,
        word: &str,
        profile: OrthographyProfile,
    ) -> Result<Vec<Analysis>> {
        let parsed = SynodalWord::parse(word)?;
        let marked_key = normalize_lookup(parsed.canonical());
        let key = normalize_lookup_accentless(parsed.canonical());
        let allow_fallback = profile == OrthographyProfile::ExpandedAccentless || marked_key == key;
        let (marked, fallback) = match profile {
            OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                (&self.expanded_marked, &self.expanded)
            }
            OrthographyProfile::SynodalLiturgical => (&self.printed_marked, &self.printed),
        };
        let mut analyses = marked
            .get(&marked_key)
            .cloned()
            .filter(|analyses| !analyses.is_empty())
            .unwrap_or_else(|| {
                if allow_fallback {
                    fallback.get(&key).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            });
        if let Ok(expansions) = crate::morphology::abbreviation::expand(parsed.canonical()) {
            for expansion in expansions {
                let lexeme = crate::morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
                analyses.push(Analysis {
                    lexeme,
                    cell: Some(expansion.cell),
                    matched_text: parsed.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: synodal_church_slavonic_core::Confidence::CERTAIN,
                    evidence_ids: expansion
                        .evidence_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    assumptions: Vec::new(),
                    contradictions: Vec::new(),
                    warnings: Vec::new(),
                    rule_trace: RuleTrace::default(),
                });
            }
        }
        deduplicate_analyses(&mut analyses);
        Ok(analyses)
    }

    #[must_use]
    pub fn spelling_candidates(&self, word: &str) -> Vec<LexemeId> {
        self.spelling_candidates
            .get(&spelling_key(word))
            .map_or_else(Vec::new, |ids| ids.iter().cloned().collect())
    }

    fn index_cell(&mut self, lexeme: &LexemeSummary, cell: GrammarCell, inflector: Inflector) {
        let Ok(forms) = inflector.form_by_id(lexeme.id(), cell) else {
            return;
        };
        let profile = inflector.orthography();
        for variant in forms.variants() {
            let surface = match profile {
                OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                    &variant.expanded
                }
                OrthographyProfile::SynodalLiturgical => &variant.printed,
            };
            let Ok(canonical_surface) = SynodalWord::parse(surface) else {
                continue;
            };
            let marked_key = normalize_lookup(canonical_surface.canonical());
            let key = normalize_lookup_accentless(canonical_surface.canonical());
            let analysis = Analysis {
                lexeme: lexeme.clone(),
                cell: Some(cell),
                matched_text: surface.clone(),
                source: analysis_source(&variant.source),
                recension_mapping: variant.recension_mapping.clone(),
                confidence: variant.confidence,
                evidence_ids: variant
                    .evidence
                    .iter()
                    .map(|evidence| evidence.id.to_string())
                    .collect(),
                assumptions: variant
                    .assumptions
                    .iter()
                    .map(|assumption| assumption.detail.clone())
                    .collect(),
                contradictions: variant
                    .contradictions
                    .iter()
                    .map(|contradiction| contradiction.detail.clone())
                    .collect(),
                warnings: variant.warnings.clone(),
                rule_trace: variant.rule_trace.clone(),
            };
            let (marked, fallback) = match profile {
                OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                    (&mut self.expanded_marked, &mut self.expanded)
                }
                OrthographyProfile::SynodalLiturgical => {
                    (&mut self.printed_marked, &mut self.printed)
                }
            };
            marked.entry(marked_key).or_default().push(analysis.clone());
            fallback.entry(key).or_default().push(analysis);
        }
    }
}

pub fn check_text(analyzer: &Analyzer, text: &str, options: CheckTextOptions) -> TextReport {
    let mut analyses = Vec::new();
    let mut unique = BTreeSet::new();
    let mut summary = TextSummary::default();
    for token in tokenize(text) {
        unique.insert(token.normalized.clone());
        let analysis = classify_token(analyzer, token, &options);
        update_text_summary(&mut summary, &analysis);
        analyses.push(analysis);
    }
    summary.total_tokens = analyses.len();
    summary.unique_tokens = unique.len();
    TextReport {
        schema_version: 1,
        options,
        summary,
        tokens: analyses,
    }
}

pub fn coverage(
    analyzer: &Analyzer,
    passages: &[CoveragePassage],
    options: CheckTextOptions,
) -> CoverageReport {
    let mut summary = CoverageSlice::default();
    let mut by_corpus = BTreeMap::new();
    let mut by_source = BTreeMap::new();
    let mut by_policy = BTreeMap::new();
    let mut by_lexeme = BTreeMap::new();
    let mut by_family = BTreeMap::new();
    let mut by_system = BTreeMap::new();
    let mut by_corpus_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_source_gap: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    let mut by_gap = BTreeMap::new();
    let mut types = BTreeSet::new();
    let mut aggregates: BTreeMap<(GapKind, String), GapAggregate> = BTreeMap::new();
    let mut cache: BTreeMap<String, TextTokenAnalysis> = BTreeMap::new();
    let mut probable_aggregates: BTreeMap<String, ProbableFamilyAggregate> = BTreeMap::new();
    let mut recovery_estimates = BTreeMap::new();
    let mut abbreviation_family_tokens = 0_usize;
    let mut spelling_variant_family_tokens = 0_usize;
    let mut remaining_ungrouped_unknowns = 0_usize;

    for passage in passages {
        let document = format!("{}:{}", passage.source_id, passage.passage);
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
            let system = analysis
                .analyses
                .first()
                .and_then(|candidate| candidate.cell)
                .map_or_else(|| "unresolved".into(), morphological_system);
            update_slice(by_system.entry(system).or_default(), &analysis);
            *by_status
                .entry(status_label(analysis.status).into())
                .or_default() += 1;
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
        by_corpus,
        by_source,
        by_policy,
        by_lexeme,
        by_family,
        by_morphological_system: by_system,
        by_corpus_gap,
        by_source_gap,
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
    }
}

impl CoverageReport {
    #[must_use]
    pub fn markdown(&self) -> String {
        let basis_points = |value: usize, total: usize| {
            value
                .saturating_mul(10_000)
                .checked_div(total)
                .unwrap_or_default()
        };
        let mut output = format!(
            "# Synodal corpus coverage\n\n- Passages: {}\n- Tokens: {}\n- Types: {}\n- Top-1 analyzed: {} ({} bp)\n- Top-k analyzed: {} ({} bp)\n- Ambiguous: {}\n- Unresolved: {}\n\n## Gap categories\n\n| Category | Tokens |\n|---|---:|\n",
            self.passages,
            self.summary.total_tokens,
            self.token_types,
            self.summary.top_1_analyzed,
            basis_points(self.summary.top_1_analyzed, self.summary.total_tokens),
            self.summary.top_k_analyzed,
            basis_points(self.summary.top_k_analyzed, self.summary.total_tokens),
            self.summary.ambiguous,
            self.summary.unresolved,
        );
        for kind in GapKind::ALL {
            output.push_str(&format!(
                "| `{}` | {} |\n",
                kind.label(),
                self.by_gap.get(kind.label()).copied().unwrap_or_default()
            ));
        }
        output.push_str("\n## Estimated recovery routes\n\nThese are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.\n\n| Route | Tokens |\n|---|---:|\n");
        for route in [
            RecoveryRoute::ExactEvidence,
            RecoveryRoute::ReviewedClass,
            RecoveryRoute::ReviewedPrincipalPart,
            RecoveryRoute::AbbreviationRegistry,
            RecoveryRoute::SpellingVariant,
            RecoveryRoute::UnsupportedFormation,
            RecoveryRoute::UngroupedUnknown,
        ] {
            output.push_str(&format!(
                "| `{}` | {} |\n",
                route.label(),
                self.estimated_recovery_by_route
                    .get(route.label())
                    .copied()
                    .unwrap_or_default(),
            ));
        }
        output.push_str("\n## Unresolved tokens by probable family\n\n| Family diagnostic | Tokens | Documents | Route | Surfaces |\n|---|---:|---:|---|---|\n");
        let mut diagnostics: Vec<_> = self.unresolved_by_probable_family.values().collect();
        diagnostics.retain(|diagnostic| diagnostic.top_k_uncovered_token_frequency > 0);
        diagnostics.sort_by(|left, right| {
            right
                .top_k_uncovered_token_frequency
                .cmp(&left.top_k_uncovered_token_frequency)
                .then_with(|| left.probable_family_id.cmp(&right.probable_family_id))
        });
        for diagnostic in diagnostics.into_iter().take(100) {
            output.push_str(&format!(
                "| `{}` | {} | {} | `{}` | {} |\n",
                escape_markdown(&diagnostic.probable_family_id),
                diagnostic.top_k_uncovered_token_frequency,
                diagnostic.document_frequency,
                diagnostic.recovery_route.label(),
                escape_markdown(&diagnostic.surfaces.join(", ")),
            ));
        }
        output.push_str("\n## Coverage by corpus\n\n| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n");
        for (corpus, slice) in &self.by_corpus {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                escape_markdown(corpus),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Coverage by source\n\n| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (source, slice) in &self.by_source {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                escape_markdown(source),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Gap categories by source\n\n| Source | Category | Tokens |\n|---|---|---:|\n",
        );
        for (source, gaps) in &self.by_source_gap {
            for kind in GapKind::ALL {
                let count = gaps.get(kind.label()).copied().unwrap_or_default();
                if count > 0 {
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        escape_markdown(source),
                        kind.label(),
                        count,
                    ));
                }
            }
        }
        output.push_str(
            "\n## Review queue\n\n| Rank | Gap | Token | Frequency | Documents | Action |\n|---:|---|---|---:|---:|---|\n",
        );
        for item in &self.review_queue {
            output.push_str(&format!(
                "| {} | `{}` | `{}` | {} | {} | {} |\n",
                item.rank,
                item.kind.label(),
                escape_markdown(&item.sample),
                item.frequency,
                item.document_frequency,
                escape_markdown(&item.suggested_action),
            ));
        }
        output
    }

    #[must_use]
    pub fn gaps_tsv(&self) -> String {
        let mut output = String::from(
            "rank\tkind\tnormalized\tsample\tfrequency\tdocument_frequency\tcandidate_lexeme_ids\tsuggested_action\n",
        );
        for item in &self.review_queue {
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                item.rank,
                item.kind.label(),
                tsv_field(&item.normalized),
                tsv_field(&item.sample),
                item.frequency,
                item.document_frequency,
                item.candidate_lexeme_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                tsv_field(&item.suggested_action),
            ));
        }
        output
    }
}

#[derive(Clone, Debug)]
struct ProbableFamilyAggregate {
    id: String,
    frequency: usize,
    top_k_uncovered_frequency: usize,
    documents: BTreeSet<String>,
    surfaces: BTreeSet<String>,
    candidate_lexeme_ids: BTreeSet<LexemeId>,
    recovery_route: RecoveryRoute,
    assumption: String,
}

impl ProbableFamilyAggregate {
    fn new(id: String, recovery_route: RecoveryRoute, assumption: String) -> Self {
        Self {
            id,
            frequency: 0,
            top_k_uncovered_frequency: 0,
            documents: BTreeSet::new(),
            surfaces: BTreeSet::new(),
            candidate_lexeme_ids: BTreeSet::new(),
            recovery_route,
            assumption,
        }
    }

    fn observe(&mut self, analysis: &TextTokenAnalysis, gap: &GapOccurrence, document: &str) {
        self.frequency += 1;
        if !is_top_k_analyzed(analysis) {
            self.top_k_uncovered_frequency += 1;
            self.documents.insert(document.into());
        }
        self.surfaces.insert(analysis.token.original.clone());
        self.candidate_lexeme_ids
            .extend(gap.candidate_lexeme_ids.iter().cloned());
    }

    fn finish(self) -> ProbableFamilyDiagnostic {
        ProbableFamilyDiagnostic {
            probable_family_id: self.id,
            token_frequency: self.frequency,
            top_k_uncovered_token_frequency: self.top_k_uncovered_frequency,
            document_frequency: self.documents.len(),
            surfaces: self.surfaces.into_iter().collect(),
            candidate_lexeme_ids: self.candidate_lexeme_ids.into_iter().collect(),
            recovery_route: self.recovery_route,
            assumption: self.assumption,
        }
    }
}

fn probable_family(
    analysis: &TextTokenAnalysis,
    gap: &GapOccurrence,
) -> (String, RecoveryRoute, String) {
    let route = match gap.kind {
        GapKind::MissingDeclensionOrClass => RecoveryRoute::ReviewedClass,
        GapKind::MissingVerbPrincipalPart => RecoveryRoute::ReviewedPrincipalPart,
        GapKind::UnsupportedFormation if !gap.candidate_lexeme_ids.is_empty() => {
            RecoveryRoute::UnsupportedFormation
        }
        GapKind::MissingAccentOrOrthographicMetadata | GapKind::AmbiguityOrSpellingVariant => {
            RecoveryRoute::SpellingVariant
        }
        GapKind::UnknownLexeme if has_abbreviation_marks(&analysis.token.original) => {
            RecoveryRoute::AbbreviationRegistry
        }
        GapKind::UnknownLexeme => RecoveryRoute::UngroupedUnknown,
        GapKind::UnsupportedFormation => RecoveryRoute::ExactEvidence,
    };
    if gap.candidate_lexeme_ids.len() == 1 {
        return (
            FamilyId::for_lexeme(&gap.candidate_lexeme_ids[0]).to_string(),
            route,
            "one reviewed lexeme is compatible with this diagnostic; the requested cell still requires review".into(),
        );
    }
    if gap.candidate_lexeme_ids.len() > 1 {
        return (
            format!(
                "ambiguous-family:{}",
                gap.candidate_lexeme_ids
                    .iter()
                    .map(LexemeId::as_str)
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            route,
            "multiple reviewed lexemes remain compatible; no family was selected".into(),
        );
    }

    let normalized = analysis.token.normalized.as_str();
    let recognized = if normalized.starts_with("реч") || normalized.starts_with("рц") {
        Some((
            "diagnostic-family:рещи",
            "shared surface material suggests the reviewed рещи family; stem similarity alone does not prove identity",
        ))
    } else if matches!(
        normalized,
        "весь"
            | "всѧ"
            | "все"
            | "вси"
            | "всѣхъ"
            | "всѣмъ"
            | "всѣми"
            | "всю"
            | "всему"
            | "всей"
            | "всеѧ"
            | "всего"
            | "всею"
    ) {
        Some((
            "diagnostic-family:весь",
            "shared вс- material suggests весь; pronominal and unrelated identities must remain possible",
        ))
    } else if normalized.starts_with("сын") {
        Some((
            "diagnostic-family:сынъ",
            "shared сын- material suggests сынъ; the consonantal plural alternant still requires evidence",
        ))
    } else if normalized.starts_with("земл") || normalized.starts_with("земе") {
        Some((
            "diagnostic-family:землѧ",
            "shared земл-/земе- material suggests землѧ; the alternation and cell remain unproved",
        ))
    } else if normalized.starts_with("господ")
        || (normalized.starts_with("гд") && has_abbreviation_marks(&analysis.token.original))
    {
        Some((
            "diagnostic-family:господь",
            "expanded or contracted surface suggests господь; titlo scope and grammatical cell remain review requirements",
        ))
    } else if normalized.starts_with("ꙗкож") {
        Some((
            "diagnostic-family:ꙗкоже",
            "surface similarity groups the token for review while preserving adverb/conjunction ambiguity",
        ))
    } else if normalized == "ꙗкѡ" || normalized == "яко" {
        Some((
            "diagnostic-family:ꙗкѡ",
            "surface identity groups the token while preserving all reviewed syntactic identities",
        ))
    } else {
        None
    };
    if let Some((id, assumption)) = recognized {
        return (
            id.into(),
            if route == RecoveryRoute::UngroupedUnknown {
                RecoveryRoute::ExactEvidence
            } else {
                route
            },
            assumption.into(),
        );
    }
    (
        format!("ungrouped:{}", analysis.token.normalized),
        route,
        "no reviewed lexical identity or conservative high-impact family diagnostic is available"
            .into(),
    )
}

fn has_abbreviation_marks(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(character, '\u{0483}' | '\u{0487}')
            || ('\u{2de0}'..='\u{2dff}').contains(&character)
    })
}

#[derive(Clone, Debug)]
struct GapAggregate {
    record: GapRecord,
    documents: BTreeSet<String>,
    top_k_uncovered_documents: BTreeSet<String>,
    corpora: BTreeSet<String>,
    source_ids: BTreeSet<String>,
    editions: BTreeSet<String>,
    partitions: BTreeSet<String>,
    source_recensions: BTreeSet<String>,
    contexts: BTreeSet<(String, String, usize, usize, String)>,
}

impl GapAggregate {
    fn new(passage: &CoveragePassage, analysis: &TextTokenAnalysis, gap: &GapOccurrence) -> Self {
        Self {
            record: GapRecord {
                kind: gap.kind,
                original: analysis.token.original.clone(),
                normalized: analysis.token.normalized.clone(),
                corpus: passage.corpus.clone(),
                source_id: passage.source_id.clone(),
                work: passage.work.clone(),
                edition: passage.edition.clone(),
                passage: passage.passage.clone(),
                partition: passage.partition.clone(),
                source_recension: passage.source_recension.clone(),
                corpora: Vec::new(),
                source_ids: Vec::new(),
                editions: Vec::new(),
                partitions: Vec::new(),
                source_recensions: Vec::new(),
                documents: Vec::new(),
                contexts: Vec::new(),
                byte_start: analysis.token.byte_start,
                byte_end: analysis.token.byte_end,
                line: analysis.token.line,
                column: analysis.token.column,
                candidate_lexeme_ids: gap.candidate_lexeme_ids.clone(),
                requested_morphological_system: gap.requested_morphological_system.clone(),
                generation_policy: GenerationPolicy::Strict,
                orthography_profile: OrthographyProfile::Expanded,
                resolver_trace: gap.resolver_trace.clone(),
                missing_metadata: gap.missing_metadata.clone(),
                secondary_reasons: gap.secondary_reasons.clone(),
                detail: gap.detail.clone(),
                frequency: 0,
                document_frequency: 0,
                top_k_uncovered_frequency: 0,
                top_k_uncovered_documents: Vec::new(),
                suggested_action: gap.suggested_action.clone(),
            },
            documents: BTreeSet::new(),
            top_k_uncovered_documents: BTreeSet::new(),
            corpora: BTreeSet::new(),
            source_ids: BTreeSet::new(),
            editions: BTreeSet::new(),
            partitions: BTreeSet::new(),
            source_recensions: BTreeSet::new(),
            contexts: BTreeSet::new(),
        }
    }

    fn observe(
        &mut self,
        passage: &CoveragePassage,
        document: &str,
        analysis: &TextTokenAnalysis,
        gap: &GapOccurrence,
    ) {
        self.record.frequency += 1;
        self.documents.insert(document.into());
        if !is_top_k_analyzed(analysis) {
            self.record.top_k_uncovered_frequency += 1;
            self.top_k_uncovered_documents.insert(document.into());
        }
        self.corpora.insert(passage.corpus.clone());
        self.source_ids.insert(passage.source_id.clone());
        self.editions.insert(passage.edition.clone());
        self.partitions.insert(passage.partition.clone());
        self.source_recensions
            .insert(passage.source_recension.clone());
        if self.contexts.len() < 8 {
            self.contexts.insert((
                document.into(),
                passage.passage.clone(),
                analysis.token.line,
                analysis.token.column,
                context_excerpt(
                    &passage.text,
                    analysis.token.byte_start,
                    analysis.token.byte_end,
                ),
            ));
        }
        self.record
            .candidate_lexeme_ids
            .extend(gap.candidate_lexeme_ids.iter().cloned());
        self.record.candidate_lexeme_ids.sort();
        self.record.candidate_lexeme_ids.dedup();
        self.record
            .missing_metadata
            .extend(gap.missing_metadata.iter().copied());
        self.record.missing_metadata.sort();
        self.record.missing_metadata.dedup();
        self.record
            .secondary_reasons
            .extend(gap.secondary_reasons.iter().copied());
        self.record.secondary_reasons.sort();
        self.record.secondary_reasons.dedup();
    }

    fn finish(mut self, options: &CheckTextOptions) -> GapRecord {
        self.record.document_frequency = self.documents.len();
        self.record.documents = self.documents.into_iter().collect();
        self.record.top_k_uncovered_documents =
            self.top_k_uncovered_documents.into_iter().collect();
        self.record.contexts = self
            .contexts
            .into_iter()
            .take(5)
            .map(|(document, passage, line, column, excerpt)| GapContext {
                document,
                passage,
                line,
                column,
                excerpt,
            })
            .collect();
        self.record.corpora = self.corpora.into_iter().collect();
        self.record.source_ids = self.source_ids.into_iter().collect();
        self.record.editions = self.editions.into_iter().collect();
        self.record.partitions = self.partitions.into_iter().collect();
        self.record.source_recensions = self.source_recensions.into_iter().collect();
        self.record.generation_policy = options.generation_policy;
        self.record.orthography_profile = options.orthography_profile;
        self.record
    }
}

fn context_excerpt(text: &str, byte_start: usize, byte_end: usize) -> String {
    let start = text[..byte_start]
        .char_indices()
        .rev()
        .nth(8)
        .map_or(0, |(index, _)| index);
    let end = text[byte_end..]
        .char_indices()
        .nth(8)
        .map_or(text.len(), |(index, _)| byte_end + index);
    text[start..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn classify_token(
    analyzer: &Analyzer,
    token: TextToken,
    options: &CheckTextOptions,
) -> TextTokenAnalysis {
    if parse_cyrillic_numeral(&token.original).is_ok() {
        return TextTokenAnalysis {
            token,
            status: TokenStatus::CyrillicNumeral,
            analyses: Vec::new(),
            gap: None,
        };
    }
    if let Err(error) = SynodalWord::parse(&token.original) {
        let kind = if contains_cyrillic(&token.original)
            && !contains_non_cyrillic_alphabetic(&token.original)
        {
            GapKind::MissingAccentOrOrthographicMetadata
        } else {
            GapKind::UnknownLexeme
        };
        return TextTokenAnalysis {
            token,
            status: TokenStatus::Unresolved,
            analyses: Vec::new(),
            gap: Some(GapOccurrence {
                kind,
                secondary_reasons: Vec::new(),
                detail: error.to_string(),
                candidate_lexeme_ids: Vec::new(),
                requested_morphological_system: None,
                missing_metadata: Vec::new(),
                resolver_trace: RuleTrace::default(),
                suggested_action: if kind == GapKind::UnknownLexeme {
                    "replace Latin or later-language fallback text, or review a new target lexeme"
                        .into()
                } else {
                    "review malformed combining marks, titlo expansion, accents, and positional spelling"
                        .into()
                },
            }),
        };
    }
    let analyses = analyzer
        .analyze_profile(&token.original, options.orthography_profile)
        .unwrap_or_default();
    if !analyses.is_empty() {
        let ids: BTreeSet<&LexemeId> = analyses
            .iter()
            .map(|analysis| analysis.lexeme.id())
            .collect();
        if ids.len() > 1 {
            return TextTokenAnalysis {
                gap: Some(ambiguity_gap(
                    &analyses,
                    "several target lexemes match this surface",
                )),
                token,
                status: TokenStatus::Ambiguous,
                analyses,
            };
        }
        let status = analyses
            .iter()
            .map(|analysis| status_for_source(analysis.source))
            .min()
            .unwrap_or(TokenStatus::Unresolved);
        return TextTokenAnalysis {
            token,
            status,
            analyses,
            gap: None,
        };
    }

    if normalize_lookup(&token.original) != normalize_lookup_accentless(&token.original) {
        let accentless = analyzer
            .analyze_profile(&token.original, OrthographyProfile::ExpandedAccentless)
            .unwrap_or_default();
        if !accentless.is_empty() {
            return TextTokenAnalysis {
                token,
                status: TokenStatus::Unresolved,
                gap: Some(GapOccurrence {
                    kind: GapKind::MissingAccentOrOrthographicMetadata,
                    secondary_reasons: Vec::new(),
                    detail: "the accentless surface resolves, but the explicit presentation marks do not match reviewed evidence".into(),
                    candidate_lexeme_ids: analysis_ids(&accentless),
                    requested_morphological_system: accentless
                        .first()
                        .and_then(|analysis| analysis.cell)
                        .map(morphological_system),
                    missing_metadata: vec![MetadataField::AccentClass],
                    resolver_trace: accentless
                        .first()
                        .map_or_else(RuleTrace::default, |analysis| analysis.rule_trace.clone()),
                    suggested_action: "review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback".into(),
                }),
                analyses: accentless,
            };
        }
    }

    if options.orthography_profile == OrthographyProfile::SynodalLiturgical {
        let expanded = analyzer
            .analyze_profile(&token.original, OrthographyProfile::Expanded)
            .unwrap_or_default();
        if !expanded.is_empty() {
            let ids = analysis_ids(&expanded);
            return TextTokenAnalysis {
                token,
                status: TokenStatus::Unresolved,
                analyses: expanded,
                gap: Some(GapOccurrence {
                    kind: GapKind::MissingAccentOrOrthographicMetadata,
                    secondary_reasons: Vec::new(),
                    detail: "expanded morphology resolves, but the liturgical profile cannot realize this surface".into(),
                    candidate_lexeme_ids: ids,
                    requested_morphological_system: None,
                    missing_metadata: vec![MetadataField::AccentClass],
                    resolver_trace: RuleTrace::default(),
                    suggested_action: "review accent, breathing, and positional-letter evidence for the resolved lexeme".into(),
                }),
            };
        }
    }

    if let Ok(summary) = crate::morphology::lookup(&token.original) {
        let metadata = lexical_metadata(summary.id()).ok();
        let missing = missing_metadata_by_id(summary.id()).unwrap_or_default();
        let principal_missing: Vec<MetadataField> = missing
            .iter()
            .copied()
            .filter(|field| {
                matches!(
                    field,
                    MetadataField::PresentStem
                        | MetadataField::PresentFirstSingular
                        | MetadataField::PresentThirdPlural
                        | MetadataField::ImperfectStem
                        | MetadataField::AoristStem
                        | MetadataField::ImperativeStem
                        | MetadataField::LParticipleStem
                        | MetadataField::ParticipleStem
                        | MetadataField::SupineStem
                        | MetadataField::VerbalNounStem
                )
            })
            .collect();
        let capabilities = capabilities_by_id(summary.id(), analyzer.inflector()).ok();
        let kind =
            if summary.part_of_speech() == PartOfSpeech::Verb && !principal_missing.is_empty() {
                GapKind::MissingVerbPrincipalPart
            } else if metadata
                .as_ref()
                .is_none_or(|metadata| metadata.class.is_none())
                || capabilities.as_ref().is_some_and(|capabilities| {
                    matches!(summary.part_of_speech(), PartOfSpeech::Noun)
                        && !capabilities.productive_noun
                        || matches!(summary.part_of_speech(), PartOfSpeech::Adjective)
                            && !capabilities.productive_adjective
                })
            {
                GapKind::MissingDeclensionOrClass
            } else {
                GapKind::UnsupportedFormation
            };
        let suggested_action = match kind {
            GapKind::MissingVerbPrincipalPart => {
                "review the independently sourced principal part required by the repeated corpus form"
            }
            GapKind::MissingDeclensionOrClass => {
                "review the target declension or lexical class before enabling generation"
            }
            _ => "identify the requested cell and add a cited Synodal rule or exact evidence",
        };
        return TextTokenAnalysis {
            token,
            status: TokenStatus::Unresolved,
            analyses: Vec::new(),
            gap: Some(GapOccurrence {
                kind,
                secondary_reasons: Vec::new(),
                detail: format!(
                    "known target lemma {} does not analyze in this context",
                    summary.id()
                ),
                candidate_lexeme_ids: vec![summary.id().clone()],
                requested_morphological_system: None,
                missing_metadata: if kind == GapKind::MissingVerbPrincipalPart {
                    principal_missing
                } else {
                    missing
                },
                resolver_trace: RuleTrace::default(),
                suggested_action: suggested_action.into(),
            }),
        };
    }

    let spelling_candidates = analyzer.spelling_candidates(&token.original);
    if !spelling_candidates.is_empty() {
        return TextTokenAnalysis {
            token,
            status: TokenStatus::SpellingVariant,
            analyses: Vec::new(),
            gap: Some(GapOccurrence {
                kind: GapKind::AmbiguityOrSpellingVariant,
                secondary_reasons: Vec::new(),
                detail: "the diagnostic spelling key matches one or more reviewed lemmas".into(),
                candidate_lexeme_ids: spelling_candidates,
                requested_morphological_system: None,
                missing_metadata: Vec::new(),
                resolver_trace: RuleTrace::default(),
                suggested_action: "review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme".into(),
            }),
        };
    }

    TextTokenAnalysis {
        token,
        status: TokenStatus::Unresolved,
        analyses: Vec::new(),
        gap: Some(GapOccurrence {
            kind: GapKind::UnknownLexeme,
            secondary_reasons: Vec::new(),
            detail: "no reviewed target lexeme or compatible generated form".into(),
            candidate_lexeme_ids: Vec::new(),
            requested_morphological_system: None,
            missing_metadata: Vec::new(),
            resolver_trace: RuleTrace::default(),
            suggested_action: "review the token against target-recension evidence and create or reject a lexical candidate".into(),
        }),
    }
}

#[must_use]
pub fn tokenize(text: &str) -> Vec<TextToken> {
    let mut tokens = Vec::new();
    let mut start = None::<(usize, usize, usize)>;
    let mut line = 1_usize;
    let mut column = 1_usize;
    for (byte, character) in text.char_indices() {
        let component = character.is_alphabetic()
            || is_combining_mark(character)
            || character == '\u{0482}'
            || ('\u{2de0}'..='\u{2dff}').contains(&character);
        if component {
            start.get_or_insert((byte, line, column));
        } else if let Some((byte_start, token_line, token_column)) = start.take() {
            push_token(
                &mut tokens,
                text,
                byte_start,
                byte,
                token_line,
                token_column,
            );
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    if let Some((byte_start, token_line, token_column)) = start {
        push_token(
            &mut tokens,
            text,
            byte_start,
            text.len(),
            token_line,
            token_column,
        );
    }
    tokens
}

fn push_token(
    tokens: &mut Vec<TextToken>,
    text: &str,
    byte_start: usize,
    byte_end: usize,
    line: usize,
    column: usize,
) {
    let original = &text[byte_start..byte_end];
    tokens.push(TextToken {
        original: original.into(),
        normalized: normalize_lookup_accentless(original),
        byte_start,
        byte_end,
        line,
        column,
    });
}

fn sort_index(index: &mut BTreeMap<String, Vec<Analysis>>) {
    for analyses in index.values_mut() {
        deduplicate_analyses(analyses);
    }
}

fn deduplicate_analyses(analyses: &mut Vec<Analysis>) {
    analyses.sort_by(|left, right| {
        source_rank(left.source)
            .cmp(&source_rank(right.source))
            .then_with(|| left.lexeme.id().cmp(right.lexeme.id()))
            .then_with(|| left.cell.cmp(&right.cell))
            .then_with(|| left.matched_text.cmp(&right.matched_text))
    });
    analyses.dedup_by(|left, right| {
        left.lexeme.id() == right.lexeme.id()
            && left.cell == right.cell
            && left.source == right.source
            && left.recension_mapping == right.recension_mapping
            && left.matched_text == right.matched_text
    });
}

const fn source_rank(source: AnalysisSource) -> u8 {
    match source {
        AnalysisSource::ExactSynodalAttestation => 0,
        AnalysisSource::SynodalIrregularOverride => 1,
        AnalysisSource::SynodalNormativeTable => 2,
        AnalysisSource::SynodalProductiveRule => 3,
        AnalysisSource::CallerSpecifiedPrediction => 4,
        AnalysisSource::AbbreviationExpansion => 5,
        AnalysisSource::InheritedPrediction => 6,
        AnalysisSource::AnalogicalPrediction => 7,
    }
}

const fn status_for_source(source: AnalysisSource) -> TokenStatus {
    match source {
        AnalysisSource::ExactSynodalAttestation => TokenStatus::ExactSynodalAttestation,
        AnalysisSource::SynodalIrregularOverride => TokenStatus::SynodalIrregularOverride,
        AnalysisSource::SynodalNormativeTable => TokenStatus::SynodalNormativeTable,
        AnalysisSource::SynodalProductiveRule => TokenStatus::SynodalProductiveRule,
        AnalysisSource::CallerSpecifiedPrediction => TokenStatus::CallerSpecifiedPrediction,
        AnalysisSource::InheritedPrediction => TokenStatus::InheritedPrediction,
        AnalysisSource::AnalogicalPrediction => TokenStatus::AnalogicalPrediction,
        AnalysisSource::AbbreviationExpansion => TokenStatus::AbbreviationExpansion,
    }
}

const fn status_label(status: TokenStatus) -> &'static str {
    match status {
        TokenStatus::ExactSynodalAttestation => "exact-synodal-attestation",
        TokenStatus::SynodalIrregularOverride => "synodal-irregular-override",
        TokenStatus::SynodalNormativeTable => "synodal-normative-table",
        TokenStatus::SynodalProductiveRule => "synodal-productive-rule",
        TokenStatus::CallerSpecifiedPrediction => "caller-specified-prediction",
        TokenStatus::InheritedPrediction => "inherited-prediction",
        TokenStatus::AnalogicalPrediction => "analogical-prediction",
        TokenStatus::AbbreviationExpansion => "abbreviation-expansion",
        TokenStatus::SpellingVariant => "spelling-variant",
        TokenStatus::Ambiguous => "ambiguous",
        TokenStatus::Unresolved => "unresolved",
        TokenStatus::CyrillicNumeral => "cyrillic-numeral",
    }
}

const fn policy_label(policy: GenerationPolicy) -> &'static str {
    match policy {
        GenerationPolicy::Strict => "strict",
        GenerationPolicy::Productive => "productive",
        GenerationPolicy::Exploratory => "exploratory",
    }
}

fn ambiguity_gap(analyses: &[Analysis], detail: &str) -> GapOccurrence {
    GapOccurrence {
        kind: GapKind::AmbiguityOrSpellingVariant,
        secondary_reasons: Vec::new(),
        detail: detail.into(),
        candidate_lexeme_ids: analysis_ids(analyses),
        requested_morphological_system: analyses
            .first()
            .and_then(|analysis| analysis.cell)
            .map(morphological_system),
        missing_metadata: Vec::new(),
        resolver_trace: analyses
            .first()
            .map_or_else(RuleTrace::default, |analysis| analysis.rule_trace.clone()),
        suggested_action: "review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains".into(),
    }
}

fn analysis_ids(analyses: &[Analysis]) -> Vec<LexemeId> {
    analyses
        .iter()
        .map(|analysis| analysis.lexeme.id().clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn update_text_summary(summary: &mut TextSummary, analysis: &TextTokenAnalysis) {
    *summary
        .by_status
        .entry(status_label(analysis.status).into())
        .or_default() += 1;
    if analysis.status == TokenStatus::CyrillicNumeral {
        summary.numerals += 1;
    }
    if analysis.analyses.len() == 1 && analysis.gap.is_none() {
        summary.top_1_analyzed += 1;
    }
    if is_top_k_analyzed(analysis) {
        summary.top_k_analyzed += 1;
    }
    if analysis.status == TokenStatus::Ambiguous {
        summary.ambiguous_tokens += 1;
    }
    if let Some(gap) = &analysis.gap {
        *summary.by_gap.entry(gap.kind.label().into()).or_default() += 1;
        if gap.kind != GapKind::AmbiguityOrSpellingVariant {
            summary.unresolved_tokens += 1;
        }
    }
}

fn update_slice(slice: &mut CoverageSlice, analysis: &TextTokenAnalysis) {
    slice.total_tokens += 1;
    if analysis.analyses.len() == 1 && analysis.gap.is_none() {
        slice.top_1_analyzed += 1;
    }
    if is_top_k_analyzed(analysis) {
        slice.top_k_analyzed += 1;
    }
    if analysis.status == TokenStatus::Ambiguous {
        slice.ambiguous += 1;
    }
    if analysis
        .gap
        .as_ref()
        .is_some_and(|gap| gap.kind != GapKind::AmbiguityOrSpellingVariant)
    {
        slice.unresolved += 1;
    }
}

fn is_top_k_analyzed(analysis: &TextTokenAnalysis) -> bool {
    !analysis.analyses.is_empty()
        && (analysis.gap.is_none() || analysis.status == TokenStatus::Ambiguous)
}

fn morphological_system(cell: GrammarCell) -> String {
    match cell {
        GrammarCell::LexicalForm => "lexical-form",
        GrammarCell::Indeclinable => "indeclinable",
        GrammarCell::Noun(_) => "noun",
        GrammarCell::Adjective(_) => "adjective",
        GrammarCell::FiniteVerb(cell) => match cell.tense {
            synodal_church_slavonic::FiniteTense::Present => "present",
            synodal_church_slavonic::FiniteTense::Future => "future",
            synodal_church_slavonic::FiniteTense::Past => "past",
            synodal_church_slavonic::FiniteTense::Imperfect => "imperfect",
            synodal_church_slavonic::FiniteTense::Aorist => "aorist",
        },
        GrammarCell::Imperative(_) => "imperative",
        GrammarCell::Infinitive => "infinitive",
        GrammarCell::Supine => "supine",
        GrammarCell::LParticiple(_) => "l-participle",
        GrammarCell::Participle(cell) => match (cell.tense, cell.voice) {
            (
                synodal_church_slavonic::ParticipleTense::Present,
                synodal_church_slavonic::ParticipleVoice::Active,
            ) => "present-active-participle",
            (
                synodal_church_slavonic::ParticipleTense::Present,
                synodal_church_slavonic::ParticipleVoice::Passive,
            ) => "present-passive-participle",
            (
                synodal_church_slavonic::ParticipleTense::Past,
                synodal_church_slavonic::ParticipleVoice::Active,
            ) => "past-active-participle",
            (
                synodal_church_slavonic::ParticipleTense::Past,
                synodal_church_slavonic::ParticipleVoice::Passive,
            ) => "past-passive-participle",
        },
        GrammarCell::VerbalNoun(_) => "verbal-noun",
        GrammarCell::Pronoun(_) => "pronoun",
        GrammarCell::Determiner(_) => "determiner",
        GrammarCell::Numeral(_) => "numeral",
    }
    .into()
}

fn spelling_key(value: &str) -> String {
    normalize_lookup_accentless(value)
        .chars()
        .map(|character| match character {
            'є' => 'е',
            'ѡ' | 'ѻ' | 'ѽ' => 'о',
            'і' | 'ї' | 'ѵ' => 'и',
            'ꙋ' | 'ᲂ' | 'ѹ' => 'у',
            'ꙗ' => 'ѧ',
            'ѣ' => 'е',
            'ѳ' => 'ф',
            value => value,
        })
        .collect()
}

fn contains_cyrillic(value: &str) -> bool {
    value.chars().any(is_cyrillic)
}

fn contains_non_cyrillic_alphabetic(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_alphabetic() && !is_cyrillic(character))
}

fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f
            | 0x1c80..=0x1c8f
            | 0x2de0..=0x2dff
            | 0xa640..=0xa69f
            | 0x1e030..=0x1e08f
    )
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
    fn spelling_variant_family_tokens_are_not_double_counted() {
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
        assert_eq!(report.spelling_variant_family_tokens, 2);
        assert_eq!(report.gaps.len(), 1);
        assert_eq!(report.gaps[0].frequency, 2);
        assert_eq!(report.gaps[0].top_k_uncovered_frequency, 2);
        assert_eq!(report.gaps[0].top_k_uncovered_documents.len(), 1);
        assert_eq!(
            report
                .estimated_recovery_by_route
                .get(RecoveryRoute::SpellingVariant.label()),
            Some(&2)
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
}
