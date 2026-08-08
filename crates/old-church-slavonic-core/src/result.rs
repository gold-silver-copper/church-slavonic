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
    /// Ordered generation steps. Dictionary-backed results have an empty trace.
    pub trace: Vec<RuleStep>,
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
    ExplicitMetadataRule { rule_id: RuleId },
    OovPrediction { rule_id: RuleId },
    ManualOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflectionWarning {
    MultipleDictionaryVariants,
    OrthographicAliasUsed { canonical: String },
    PredictedNotDictionaryBacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataField {
    PartOfSpeech,
    NounClass,
    AdjectiveClass,
    Gender,
    Animacy,
    VerbClass,
    PresentStem,
    PresentFirstSingularStem,
    ImperfectStem,
    ImperfectFormation,
    AoristStem,
    AoristFormation,
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
    InvalidInput { reason: String },
    UnknownLemma,
    AmbiguousLexeme { candidates: Vec<LexemeSummary> },
    MissingLexicalMetadata { needed: Vec<MetadataField> },
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
            Self::UnsupportedCell => f.write_str("unsupported paradigm cell"),
        }
    }
}

impl std::error::Error for InflectionError {}
