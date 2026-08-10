use crate::{Confidence, EvidenceId, LexemeId, Recension, RecensionMappingId, RuleId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LexemeRelation {
    InheritedFrom,
    SameEtymon,
    BorrowedFrom,
    Uncertain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MappingStatus {
    Reviewed,
    AutomaticallyValidated,
    Exploratory,
    Rejected,
}

impl MappingStatus {
    #[must_use]
    pub const fn permits_productive(self) -> bool {
        matches!(self, Self::Reviewed | Self::AutomaticallyValidated)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MorphologyAlignment {
    Compatible,
    Transformed,
    Contradictory,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SemanticAlignment {
    Established,
    Narrowed,
    Shifted,
    FalseFriend,
    Unknown,
}

impl SemanticAlignment {
    #[must_use]
    pub const fn permits_generation(self) -> bool {
        matches!(self, Self::Established | Self::Narrowed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Transformation {
    pub rule: RuleId,
    pub description: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RecensionMapping {
    pub id: RecensionMappingId,
    pub source: LexemeId,
    pub target: LexemeId,
    pub source_recension: Recension,
    pub target_recension: Recension,
    pub relation: LexemeRelation,
    pub status: MappingStatus,
    pub morphology: MorphologyAlignment,
    pub semantics: SemanticAlignment,
    pub transformations: Vec<Transformation>,
    pub confidence: Confidence,
    pub evidence: Vec<EvidenceId>,
    pub contradictions: Vec<EvidenceId>,
    pub review_note: String,
}

impl RecensionMapping {
    #[must_use]
    pub fn permits_productive_generation(&self) -> bool {
        self.source_recension == Recension::OldChurchSlavonic
            && self.target_recension == Recension::SynodalRussian
            && self.status.permits_productive()
            && self.semantics.permits_generation()
            && matches!(
                self.morphology,
                MorphologyAlignment::Compatible | MorphologyAlignment::Transformed
            )
    }
}
