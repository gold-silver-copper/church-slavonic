//! Structured inflection results and typed failures.

use crate::{PartOfSpeech, RuleId, RuleStep};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormSource {
    DictionaryTable,
    DictionaryMetadataRule { rule_id: RuleId },
    DictionaryMetadataAnalyses,
    ExplicitMetadataRule { rule_id: RuleId },
    OovPrediction { rule_id: RuleId },
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
    InvalidInput {
        reason: String,
    },
    UnknownLemma,
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
    HistoricallyInvalidCell,
    UnsupportedCell,
}

impl fmt::Display for InflectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { reason } => write!(f, "invalid input: {reason}"),
            Self::UnknownLemma => f.write_str("unknown lemma"),
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
            Self::HistoricallyInvalidCell => f.write_str("historically invalid paradigm cell"),
            Self::UnsupportedCell => f.write_str("unsupported paradigm cell"),
        }
    }
}

impl std::error::Error for InflectionError {}
