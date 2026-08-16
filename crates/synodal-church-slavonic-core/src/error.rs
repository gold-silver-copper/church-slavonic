use std::{error, fmt};

use crate::{LexemeId, RecensionMappingId};

pub type Result<T> = std::result::Result<T, Error>;

/// Stable machine-readable classification for [`Error`]. Human-readable
/// diagnostics may improve without forcing callers to parse their wording.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ErrorCode {
    InvalidUnicode,
    InvalidOrthography,
    EmptyInput,
    UnknownLemma,
    AmbiguousLexeme,
    ProviderConflict,
    MissingPrincipalPart,
    MissingMetadata,
    ContradictoryMetadata,
    UnsupportedFormation,
    MissingRecensionMapping,
    AmbiguousRecensionMapping,
    SemanticAlignmentNotEstablished,
    InheritedEvidenceContradicted,
    HistoricallyInvalidCell,
    EvidenceIncompleteCell,
    UnsupportedCell,
    OrthographicMetadataRequired,
    EmptyFormSet,
    AmbiguousVariant,
    InvalidNumeral,
    OutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MetadataField {
    PresentStem,
    PresentFirstSingular,
    PresentThirdPlural,
    FutureStem,
    FutureFirstSingular,
    FutureThirdPlural,
    ImperfectStem,
    AoristStem,
    AoristFormation,
    ImperativeStem,
    ImperativeFormation,
    ImperfectFormation,
    Infinitive,
    SupineStem,
    LParticipleStem,
    ParticipleStem,
    ParticipleFormation,
    VerbalNounStem,
    ComparisonStem,
    ComparisonFormation,
    AccentClass,
    AccentParadigm,
    LexemeClass,
    Gender,
    Aspect,
    Formation,
    RegularBackground,
    IrregularOverride,
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
    ProviderConflict {
        lexeme: LexemeId,
        reason: String,
    },
    MissingPrincipalPart {
        field: MetadataField,
    },
    MissingMetadata {
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
    EvidenceIncompleteCell {
        field: MetadataField,
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

impl Error {
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidUnicode { .. } => ErrorCode::InvalidUnicode,
            Self::InvalidOrthography { .. } => ErrorCode::InvalidOrthography,
            Self::EmptyInput => ErrorCode::EmptyInput,
            Self::UnknownLemma { .. } => ErrorCode::UnknownLemma,
            Self::AmbiguousLexeme { .. } => ErrorCode::AmbiguousLexeme,
            Self::ProviderConflict { .. } => ErrorCode::ProviderConflict,
            Self::MissingPrincipalPart { .. } => ErrorCode::MissingPrincipalPart,
            Self::MissingMetadata { .. } => ErrorCode::MissingMetadata,
            Self::ContradictoryMetadata { .. } => ErrorCode::ContradictoryMetadata,
            Self::UnsupportedFormation { .. } => ErrorCode::UnsupportedFormation,
            Self::MissingRecensionMapping { .. } => ErrorCode::MissingRecensionMapping,
            Self::AmbiguousRecensionMapping { .. } => ErrorCode::AmbiguousRecensionMapping,
            Self::SemanticAlignmentNotEstablished { .. } => {
                ErrorCode::SemanticAlignmentNotEstablished
            }
            Self::InheritedEvidenceContradicted { .. } => ErrorCode::InheritedEvidenceContradicted,
            Self::HistoricallyInvalidCell { .. } => ErrorCode::HistoricallyInvalidCell,
            Self::EvidenceIncompleteCell { .. } => ErrorCode::EvidenceIncompleteCell,
            Self::UnsupportedCell { .. } => ErrorCode::UnsupportedCell,
            Self::OrthographicMetadataRequired { .. } => ErrorCode::OrthographicMetadataRequired,
            Self::EmptyFormSet => ErrorCode::EmptyFormSet,
            Self::AmbiguousVariant { .. } => ErrorCode::AmbiguousVariant,
            Self::InvalidNumeral { .. } => ErrorCode::InvalidNumeral,
            Self::OutOfRange { .. } => ErrorCode::OutOfRange,
        }
    }
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
            Self::ProviderConflict { lexeme, reason } => {
                write!(
                    formatter,
                    "lexical provider conflict for {lexeme}: {reason}"
                )
            }
            Self::MissingPrincipalPart { field } => {
                write!(formatter, "missing required principal part {field:?}")
            }
            Self::MissingMetadata { field } => {
                write!(formatter, "missing required lexical metadata {field:?}")
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
            Self::EvidenceIncompleteCell { field, reason } => {
                write!(formatter, "evidence is incomplete for {field:?}: {reason}")
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
