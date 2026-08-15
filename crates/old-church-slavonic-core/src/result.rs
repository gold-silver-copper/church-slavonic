//! Structured inflection results and typed failures.

use crate::{PartOfSpeech, RequestedCell, RuleId, RuleStep};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormVariant {
    pub text: String,
    pub romanization: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSet {
    lemma: String,
    variants: Vec<FormVariant>,
    source: FormSource,
    warnings: Vec<InflectionWarning>,
    /// Ordered generation steps. Exact dictionary-table results have an empty
    /// trace; metadata-backed and productive results retain their operations.
    trace: Vec<RuleStep>,
    /// Provenance-preserving analyses. A dictionary table is one analysis with
    /// source-ordered variants; competing lexical analyses remain separate.
    analyses: Vec<FormAnalysis>,
}

impl FormSet {
    /// Construct a successful, necessarily nonempty set of ordered forms.
    pub fn new(
        lemma: impl Into<String>,
        primary: FormVariant,
        alternatives: Vec<FormVariant>,
        source: FormSource,
        warnings: Vec<InflectionWarning>,
        trace: Vec<RuleStep>,
        analyses: Vec<FormAnalysis>,
    ) -> Self {
        let mut variants = Vec::with_capacity(alternatives.len() + 1);
        variants.push(primary);
        variants.extend(alternatives);
        Self {
            lemma: lemma.into(),
            variants,
            source,
            warnings,
            trace,
            analyses,
        }
    }

    /// The canonical lemma associated with this result.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The first variant in deterministic source order.
    ///
    /// This is not a claim that the first spelling is linguistically superior.
    pub fn primary(&self) -> &FormVariant {
        // Construction requires a primary variant and the field is private.
        &self.variants[0]
    }

    /// The text of [`Self::primary`].
    pub fn primary_text(&self) -> &str {
        &self.primary().text
    }

    /// Select a form using an explicit variant policy.
    pub fn select(&self, policy: VariantPolicy) -> Result<&FormVariant, VariantSelectionError> {
        match policy {
            VariantPolicy::SourceFirst => Ok(self.primary()),
            VariantPolicy::RequireUnique if self.variants.len() == 1 => Ok(self.primary()),
            VariantPolicy::RequireUnique => Err(VariantSelectionError {
                lemma: self.lemma.clone(),
                variant_count: self.variants.len(),
            }),
        }
    }

    /// Return the only surface text, failing rather than discarding variants.
    pub fn unique_text(&self) -> Result<&str, VariantSelectionError> {
        self.select(VariantPolicy::RequireUnique)
            .map(|variant| variant.text.as_str())
    }

    /// All variants in deterministic source order.
    pub fn variants(&self) -> impl ExactSizeIterator<Item = &FormVariant> {
        self.variants.iter()
    }

    /// All surface strings in deterministic source order.
    pub fn texts(&self) -> impl ExactSizeIterator<Item = &str> {
        self.variants.iter().map(|variant| variant.text.as_str())
    }

    /// Consume the set and return its deterministic source-order primary text.
    pub fn into_primary_text(self) -> String {
        self.variants
            .into_iter()
            .next()
            .expect("FormSet construction guarantees a primary variant")
            .text
    }

    /// Consume the set and iterate over every ordered variant.
    pub fn into_variants(self) -> impl ExactSizeIterator<Item = FormVariant> {
        self.variants.into_iter()
    }

    /// The evidence class from which this result was resolved.
    pub fn source(&self) -> &FormSource {
        &self.source
    }

    /// Non-fatal properties of this result.
    pub fn warnings(&self) -> &[InflectionWarning] {
        &self.warnings
    }

    /// Ordered productive rule steps, when a single generated analysis exists.
    pub fn trace(&self) -> &[RuleStep] {
        &self.trace
    }

    /// Ordered source-backed morphological analyses.
    pub fn analyses(&self) -> &[FormAnalysis] {
        &self.analyses
    }

    /// Add a resolver warning while preserving the nonempty result invariant.
    pub fn add_warning(&mut self, warning: InflectionWarning) {
        self.warnings.push(warning);
    }
}

/// An explicit policy for selecting one source-ordered variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariantPolicy {
    /// Choose the first form in deterministic source order.
    SourceFirst,
    /// Accept only a result that contains exactly one form.
    RequireUnique,
}

/// A request for one surface form encountered multiple source variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantSelectionError {
    pub lemma: String,
    pub variant_count: usize,
}

impl fmt::Display for VariantSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lemma {:?} has {} source-ordered variants",
            self.lemma, self.variant_count
        )
    }
}

impl std::error::Error for VariantSelectionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormSource {
    DictionaryTable,
    ReviewedGrammarTable {
        rule_id: RuleId,
    },
    /// Multiple independently sourced reviewed analyses remain viable.
    ReviewedGrammarAnalyses,
    DictionaryMetadataRule {
        rule_id: RuleId,
    },
    DictionaryMetadataAnalyses,
    ExplicitMetadataRule {
        rule_id: RuleId,
    },
    OovPrediction {
        rule_id: RuleId,
    },
    ManualOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormAnalysis {
    pub variants: Vec<FormVariant>,
    pub source: FormSource,
    pub evidence: Vec<MetadataEvidence>,
    pub trace: Vec<RuleStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataProvenance {
    ExactDictionaryTableCell,
    ReviewedGrammarTable,
    DisputedGrammarTable,
    PrimaryTextAttestation,
    DictionaryPrincipalPart,
    CuratedGrammarOverride,
    ExplicitCallerMetadata,
    CorpusEvaluationObservation,
    ProductiveRuleOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataEvidence {
    pub field: Option<MetadataField>,
    pub provenance: MetadataProvenance,
    pub source_feature: Option<String>,
    pub source_form: Option<String>,
    pub crosscheck_features: Vec<String>,
    pub authority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflectionWarning {
    MultipleDictionaryVariants,
    OrthographicAliasUsed { canonical: String },
    PredictedNotDictionaryBacked,
    MultipleMorphologicalAnalyses,
    IncludesReconstructedForms,
    IncludesDisputedForms,
    LexicalAliasUsed { canonical: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataField {
    PartOfSpeech,
    NounClass,
    AdjectiveClass,
    Gender,
    Animacy,
    VerbClass,
    VerbAspect,
    PresentStem,
    PresentFirstSingularStem,
    ImperfectStem,
    ImperfectFormation,
    ImperfectVariantPolicy,
    AoristStem,
    AoristSecondThirdSingular,
    AoristFormation,
    LParticipleStem,
    ImperativeStem,
    ImperativeFormation,
    PresentActiveParticipleStem,
    PresentActiveParticipleFormation,
    PresentPassiveParticipleStem,
    PresentPassiveParticipleFormation,
    PastActiveParticipleStem,
    PastActiveParticipleFormation,
    PastPassiveParticipleStem,
    PastPassiveParticipleFormation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexemeSummary {
    pub id: String,
    pub lemma: String,
    pub lookup_key: String,
    pub part_of_speech: PartOfSpeech,
    pub class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflectionError {
    InvalidLemma {
        input: String,
        reason: String,
    },
    InvalidInput {
        reason: String,
    },
    UnknownLemma {
        lemma: String,
        part_of_speech: PartOfSpeech,
    },
    UnknownLexemeId {
        id: String,
        expected_part_of_speech: Option<PartOfSpeech>,
    },
    AmbiguousLexeme {
        candidates: Vec<LexemeSummary>,
    },
    MissingLexicalMetadata {
        needed: Vec<MetadataField>,
    },
    ContradictoryLexicalMetadata {
        fields: Vec<MetadataField>,
    },
    UnsupportedFormation {
        system: MetadataField,
        formation: String,
    },
    HistoricallyInvalidCell {
        /// Stable dictionary ID in facade calls; caller-supplied lemma in the
        /// rule-only core, which has no dictionary identity.
        lexeme_id: String,
        cell: RequestedCell,
    },
    /// A source-reviewed lexical gap for which neither attestation nor a
    /// defensible productive reconstruction is available. This is distinct
    /// from a historically impossible cell and an unimplemented engine path.
    UnattestedUnreconstructableCell {
        lexeme_id: String,
        cell: RequestedCell,
    },
    UnsupportedCell {
        /// Stable dictionary ID in facade calls; caller-supplied lemma in the
        /// rule-only core, which has no dictionary identity.
        lexeme_id: String,
        cell: RequestedCell,
    },
}

impl InflectionError {
    pub fn invalid_lemma(input: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidLemma {
            input: input.into(),
            reason: reason.into(),
        }
    }

    pub fn unknown_lemma(lemma: impl Into<String>, part_of_speech: PartOfSpeech) -> Self {
        Self::UnknownLemma {
            lemma: lemma.into(),
            part_of_speech,
        }
    }

    pub fn unknown_id(
        id: impl Into<String>,
        expected_part_of_speech: Option<PartOfSpeech>,
    ) -> Self {
        Self::UnknownLexemeId {
            id: id.into(),
            expected_part_of_speech,
        }
    }

    pub fn unsupported(lexeme_id: impl Into<String>, cell: RequestedCell) -> Self {
        Self::UnsupportedCell {
            lexeme_id: lexeme_id.into(),
            cell,
        }
    }

    pub fn historically_invalid(lexeme_id: impl Into<String>, cell: RequestedCell) -> Self {
        Self::HistoricallyInvalidCell {
            lexeme_id: lexeme_id.into(),
            cell,
        }
    }

    pub fn unattested_unreconstructable(lexeme_id: impl Into<String>, cell: RequestedCell) -> Self {
        Self::UnattestedUnreconstructableCell {
            lexeme_id: lexeme_id.into(),
            cell,
        }
    }

    /// Replace a rule-layer lemma context with a stable facade lexeme identity.
    pub fn with_lexeme_id(self, lexeme_id: impl Into<String>) -> Self {
        let lexeme_id = lexeme_id.into();
        match self {
            Self::HistoricallyInvalidCell { cell, .. } => {
                Self::HistoricallyInvalidCell { lexeme_id, cell }
            }
            Self::UnattestedUnreconstructableCell { cell, .. } => {
                Self::UnattestedUnreconstructableCell { lexeme_id, cell }
            }
            Self::UnsupportedCell { cell, .. } => Self::UnsupportedCell { lexeme_id, cell },
            other => other,
        }
    }
}

impl fmt::Display for InflectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLemma { input, reason } => {
                write!(f, "invalid lemma {input:?}: {reason}")
            }
            Self::InvalidInput { reason } => write!(f, "invalid input: {reason}"),
            Self::UnknownLemma {
                lemma,
                part_of_speech,
            } => write!(f, "unknown {part_of_speech} lemma {lemma:?}"),
            Self::UnknownLexemeId {
                id,
                expected_part_of_speech,
            } => match expected_part_of_speech {
                Some(part_of_speech) => {
                    write!(f, "unknown {part_of_speech} lexeme ID {id:?}")
                }
                None => write!(f, "unknown lexeme ID {id:?}"),
            },
            Self::AmbiguousLexeme { candidates } => {
                write!(f, "ambiguous lemma ({} candidates)", candidates.len())
            }
            Self::MissingLexicalMetadata { needed } => {
                write!(f, "missing lexical metadata: {needed:?}")
            }
            Self::ContradictoryLexicalMetadata { fields } => {
                write!(f, "contradictory lexical metadata: {fields:?}")
            }
            Self::UnsupportedFormation { system, formation } => {
                write!(f, "unsupported {system:?}: {formation}")
            }
            Self::HistoricallyInvalidCell { lexeme_id, cell } => {
                write!(f, "historically invalid cell {cell:?} for {lexeme_id:?}")
            }
            Self::UnattestedUnreconstructableCell { lexeme_id, cell } => write!(
                f,
                "unattested and unreconstructable cell {cell:?} for {lexeme_id:?}"
            ),
            Self::UnsupportedCell { lexeme_id, cell } => {
                write!(f, "unsupported cell {cell:?} for {lexeme_id:?}")
            }
        }
    }
}

impl std::error::Error for InflectionError {}
