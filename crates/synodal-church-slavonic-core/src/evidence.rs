use crate::{EvidenceId, ModelId, Recension, RecensionMappingId, RuleId, SourceId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Confidence(u16);

impl Confidence {
    pub const ZERO: Self = Self(0);
    pub const CERTAIN: Self = Self(10_000);

    #[must_use]
    pub const fn from_basis_points(value: u16) -> Option<Self> {
        if value <= 10_000 {
            Some(Self(value))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AuthorityRole {
    Lexical,
    Grammatical,
    Morphological,
    Orthographic,
    Accentual,
    Abbreviation,
    Numeral,
    Semantic,
    ExactForm,
    Evaluation,
    Discovery,
}

impl AuthorityRole {
    pub const ALL: [Self; 11] = [
        Self::Lexical,
        Self::Grammatical,
        Self::Morphological,
        Self::Orthographic,
        Self::Accentual,
        Self::Abbreviation,
        Self::Numeral,
        Self::Semantic,
        Self::ExactForm,
        Self::Evaluation,
        Self::Discovery,
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum EpistemicRole {
    SynodalNormativeAuthority,
    ExactSynodalAttestation,
    InheritedOcsEvidence,
    OtherRecensionComparativeEvidence,
    EvaluationOnlyEvidence,
}

impl EpistemicRole {
    pub const ALL: [Self; 5] = [
        Self::SynodalNormativeAuthority,
        Self::ExactSynodalAttestation,
        Self::InheritedOcsEvidence,
        Self::OtherRecensionComparativeEvidence,
        Self::EvaluationOnlyEvidence,
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum EvidenceKind {
    ExactTableCell,
    NormativeRule,
    PrincipalPart,
    ReviewedIrregularOverride,
    CorpusObservation,
    RecensionTransformation,
    SemanticAlignment,
    ComparativeObservation,
    AccentMetadata,
}

impl EvidenceKind {
    pub const ALL: [Self; 9] = [
        Self::ExactTableCell,
        Self::NormativeRule,
        Self::PrincipalPart,
        Self::ReviewedIrregularOverride,
        Self::CorpusObservation,
        Self::RecensionTransformation,
        Self::SemanticAlignment,
        Self::ComparativeObservation,
        Self::AccentMetadata,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Evidence {
    pub id: EvidenceId,
    pub source: SourceId,
    pub source_recension: Recension,
    pub kind: EvidenceKind,
    pub authority_roles: Vec<AuthorityRole>,
    pub epistemic_role: EpistemicRole,
    pub citation: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Assumption {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Contradiction {
    pub evidence: EvidenceId,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum FormSource {
    SynodalAttestation {
        evidence: EvidenceId,
    },
    SynodalNormativeGeneration {
        rule: RuleId,
    },
    InheritedPrediction {
        source_recension: Recension,
        mapping: RecensionMappingId,
        rule: RuleId,
    },
    AnalogicalPrediction {
        model: ModelId,
    },
}

impl FormSource {
    #[must_use]
    pub const fn is_attested(&self) -> bool {
        matches!(self, Self::SynodalAttestation { .. })
    }

    #[must_use]
    pub const fn is_prediction(&self) -> bool {
        !self.is_attested()
    }

    #[must_use]
    pub const fn precedence(&self) -> u8 {
        match self {
            Self::SynodalAttestation { .. } => 0,
            Self::SynodalNormativeGeneration { .. } => 1,
            Self::InheritedPrediction { .. } => 2,
            Self::AnalogicalPrediction { .. } => 3,
        }
    }
}
