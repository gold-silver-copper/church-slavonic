#[allow(unused_imports)]
use super::*;

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

/// A typed reverse analysis of one fused Church Slavonic cardinal word.
///
/// Compound cardinals are grammatical constructions rather than synthetic
/// dictionary lexemes, so their numeric value, licensed cell, and composition
/// are preserved independently of [`Analysis`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardinalWordAnalysis {
    pub value: u32,
    pub cell: CompoundNumeralCell,
    pub construction: NumeralComposition,
    pub matched_text: String,
    pub source: AnalysisSource,
    pub confidence: synodal_church_slavonic_core::Confidence,
    pub evidence_ids: Vec<String>,
    pub assumptions: Vec<String>,
    pub contradictions: Vec<String>,
    pub warnings: Vec<String>,
    pub rule_trace: RuleTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextTokenAnalysis {
    pub token: TextToken,
    pub status: TokenStatus,
    pub analyses: Vec<Analysis>,
    /// A typed non-lexical analysis for a canonical Church Slavonic number.
    /// Numerals do not fabricate a dictionary lexeme, but a successfully
    /// parsed value is a real strict analysis and therefore counts in top-k.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeral: Option<CyrillicNumeral>,
    /// Typed analyses for fused lexical cardinal words such as
    /// `двана́десѧть` and `пѧтьдесѧ́тъ`; no artificial lexeme is created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cardinal_words: Vec<CardinalWordAnalysis>,
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

/// Composition of the covered tokens, as distinct from their count.
///
/// Strict top-k coverage answers "does this token have any analysis"; it
/// cannot distinguish a token the engine can inflect, generate, and reverse
/// from one that merely carries a reviewed headword. These measures make that
/// difference auditable so recall cannot be bought with morphology-free rows,
/// and so a fall in unique-reading counts can be attributed rather than
/// assumed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageIntegrity {
    /// Covered tokens whose analyses are *all* `LexicalForm` — a reviewed
    /// identity with no inflectional commitment at all. This is the cheapest
    /// route to coverage and the one that teaches the engine nothing.
    pub morphology_free_analyzed: usize,
    /// Covered tokens carrying at least one typed, inflectable cell.
    pub morphologically_typed_analyzed: usize,
    /// Covered tokens that resolve to exactly one lexical identity, whether or
    /// not several cells of that identity remain compatible. Unlike
    /// `top_1_analyzed` this is not capped by genuine syncretism.
    pub lemma_unique_analyzed: usize,
    /// Covered tokens with several readings of a single lexeme. This is
    /// syncretism, which the target recension has in abundance and which must
    /// be preserved rather than collapsed.
    pub within_lexeme_ambiguous: usize,
    /// Covered tokens whose readings span more than one lexeme. This is
    /// homonymy, and each pair needs its own justification.
    pub cross_lexeme_ambiguous: usize,
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

/// One complete, top-k-uncovered surface/status row for evidence review.
///
/// Unlike the bounded human-facing gap queue, this inventory includes every
/// uncovered surface whether or not it carries a [`GapOccurrence`]. Document
/// frequency is computed from the true union of source/passage identities;
/// contexts remain bounded review aids.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageFrontierItem {
    pub status: TokenStatus,
    pub kind: Option<GapKind>,
    pub normalized: String,
    pub sample: String,
    pub token_frequency: usize,
    pub document_frequency: usize,
    pub corpora: Vec<String>,
    pub source_ids: Vec<String>,
    pub partitions: Vec<String>,
    pub candidate_lexeme_ids: Vec<LexemeId>,
    pub requested_morphological_system: Option<String>,
    pub missing_metadata: Vec<MetadataField>,
    pub suggested_action: String,
    pub contexts: Vec<GapContext>,
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
    /// Composition of `summary.top_k_analyzed`. See [`CoverageIntegrity`].
    ///
    /// Additive to schema version 4: every field a version 4 reader knows keeps
    /// its meaning, and an older report deserializes with these counts zeroed,
    /// so the version is deliberately not bumped.
    #[serde(default)]
    pub integrity: CoverageIntegrity,
    /// How many sealed held-out types actually occur in this corpus.
    #[serde(default)]
    pub held_out_types: usize,
    /// Coverage restricted to tokens whose normalized type is held out.
    ///
    /// Read this together with [`Self::held_out_type_status`]: coverage that
    /// arrives as `exact-synodal-attestation` is a row citing the held-out type
    /// itself and is memorisation, while `synodal-productive-rule` and
    /// `synodal-normative-table` coverage is generalisation.
    #[serde(default)]
    pub held_out_type_coverage: CoverageSlice,
    #[serde(default)]
    pub held_out_type_status: BTreeMap<String, usize>,
    /// Diagnostic only: strict-unresolved tokens the exploratory segmentation
    /// tier can read (`prediction::predict`), by the top prediction's system.
    /// Never part of `summary.top_k_analyzed`; no sealed floor reads it.
    #[serde(default)]
    pub predicted_unresolved_by_system: BTreeMap<String, usize>,
    /// Diagnostic only: the same tokens by the top prediction's confidence
    /// bucket in basis points.
    #[serde(default)]
    pub predicted_unresolved_by_confidence: BTreeMap<String, usize>,
    /// Held-out tokens by morphological system, then by resolver status, so a
    /// wave aimed at one system can be seen landing there rather than
    /// somewhere else. Systems are attributed exactly as in
    /// [`Self::by_morphological_system`].
    #[serde(default)]
    pub held_out_type_status_by_system: BTreeMap<String, BTreeMap<String, usize>>,
    pub by_corpus: BTreeMap<String, CoverageSlice>,
    pub by_source: BTreeMap<String, CoverageSlice>,
    #[serde(default)]
    pub by_partition: BTreeMap<String, CoverageSlice>,
    #[serde(default)]
    pub by_source_partition: BTreeMap<String, CoverageSlice>,
    pub by_policy: BTreeMap<String, CoverageSlice>,
    pub by_lexeme: BTreeMap<String, CoverageSlice>,
    pub by_family: BTreeMap<String, CoverageSlice>,
    pub by_morphological_system: BTreeMap<String, CoverageSlice>,
    pub by_corpus_gap: BTreeMap<String, BTreeMap<String, usize>>,
    pub by_source_gap: BTreeMap<String, BTreeMap<String, usize>>,
    #[serde(default)]
    pub by_partition_gap: BTreeMap<String, BTreeMap<String, usize>>,
    #[serde(default)]
    pub by_source_partition_gap: BTreeMap<String, BTreeMap<String, usize>>,
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
    /// Complete in-memory frontier used to render the separately committed TSV.
    /// Keeping it out of JSON avoids duplicating tens of thousands of detailed
    /// rows in the already-large coverage report.
    #[serde(skip)]
    pub uncovered_frontier: Vec<CoverageFrontierItem>,
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
