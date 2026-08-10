use std::{error, fmt};

use crate::{LexemeId, RecensionMappingId};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MetadataField {
    PresentStem,
    PresentFirstSingular,
    PresentThirdPlural,
    ImperfectStem,
    AoristStem,
    ImperativeStem,
    Infinitive,
    SupineStem,
    LParticipleStem,
    ParticipleStem,
    VerbalNounStem,
    ComparisonStem,
    AccentClass,
    SemanticIdentity,
    AbbreviationClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Error {
    InvalidUnicode {
        byte_index: usize,
        character: char,
        reason: String,
    },
    InvalidOrthography {
        reason: String,
    },
    EmptyInput,
    UnknownLemma {
        lookup: String,
    },
    AmbiguousLexeme {
        lexemes: Vec<LexemeId>,
    },
    MissingPrincipalPart {
        field: MetadataField,
    },
    ContradictoryMetadata {
        reason: String,
    },
    UnsupportedFormation {
        formation: String,
    },
    MissingRecensionMapping {
        source: LexemeId,
    },
    AmbiguousRecensionMapping {
        mappings: Vec<RecensionMappingId>,
    },
    SemanticAlignmentNotEstablished {
        mapping: RecensionMappingId,
    },
    InheritedEvidenceContradicted {
        mapping: RecensionMappingId,
    },
    HistoricallyInvalidCell {
        reason: String,
    },
    UnsupportedCell {
        reason: String,
    },
    OrthographicMetadataRequired {
        field: MetadataField,
    },
    EmptyFormSet,
    AmbiguousVariant {
        count: usize,
    },
    InvalidNumeral {
        reason: String,
    },
    OutOfRange {
        value: u32,
        maximum: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnicode {
                byte_index,
                character,
                reason,
            } => write!(
                formatter,
                "invalid Unicode character {character:?} at byte {byte_index}: {reason}"
            ),
            Self::InvalidOrthography { reason } => {
                write!(formatter, "invalid Synodal orthography: {reason}")
            }
            Self::EmptyInput => formatter.write_str("input must not be empty"),
            Self::UnknownLemma { lookup } => write!(formatter, "unknown lemma {lookup:?}"),
            Self::AmbiguousLexeme { lexemes } => {
                write!(formatter, "lemma resolves to {} lexemes", lexemes.len())
            }
            Self::MissingPrincipalPart { field } => {
                write!(formatter, "missing required principal part {field:?}")
            }
            Self::ContradictoryMetadata { reason } => {
                write!(formatter, "contradictory lexical metadata: {reason}")
            }
            Self::UnsupportedFormation { formation } => {
                write!(formatter, "unsupported formation {formation}")
            }
            Self::MissingRecensionMapping { source } => {
                write!(
                    formatter,
                    "no reviewed Synodal mapping for source lexeme {source}"
                )
            }
            Self::AmbiguousRecensionMapping { mappings } => write!(
                formatter,
                "several recension mappings remain compatible: {}",
                mappings.len()
            ),
            Self::SemanticAlignmentNotEstablished { mapping } => {
                write!(
                    formatter,
                    "semantic alignment is not established for {mapping}"
                )
            }
            Self::InheritedEvidenceContradicted { mapping } => write!(
                formatter,
                "Synodal evidence contradicts inherited mapping {mapping}"
            ),
            Self::HistoricallyInvalidCell { reason } => {
                write!(formatter, "historically invalid cell: {reason}")
            }
            Self::UnsupportedCell { reason } => {
                write!(
                    formatter,
                    "conceptually possible but unsupported cell: {reason}"
                )
            }
            Self::OrthographicMetadataRequired { field } => write!(
                formatter,
                "orthographic transformation requires lexical metadata {field:?}"
            ),
            Self::EmptyFormSet => formatter.write_str("a form set must be nonempty"),
            Self::AmbiguousVariant { count } => {
                write!(formatter, "expected one unique variant, found {count}")
            }
            Self::InvalidNumeral { reason } => {
                write!(formatter, "invalid Church Slavonic numeral: {reason}")
            }
            Self::OutOfRange { value, maximum } => {
                write!(
                    formatter,
                    "value {value} exceeds supported maximum {maximum}"
                )
            }
        }
    }
}

impl error::Error for Error {}
