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
    pub lemma: String,
    pub variants: Vec<FormVariant>,
    pub source: FormSource,
    pub warnings: Vec<InflectionWarning>,
    /// Ordered generation steps. Exact dictionary-table results have an empty
    /// trace; metadata-backed and productive results retain their operations.
    pub trace: Vec<RuleStep>,
    /// Provenance-preserving analyses. A dictionary table is one analysis with
    /// source-ordered variants; competing lexical analyses remain separate.
    pub analyses: Vec<FormAnalysis>,
}

impl FormSet {
    /// Returns the source-order primary variant. No linguistic preference is inferred.
    pub fn primary_source_order(&self) -> Option<&FormVariant> {
        self.variants.first()
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
