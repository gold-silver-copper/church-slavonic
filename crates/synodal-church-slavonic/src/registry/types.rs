use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PartOfSpeech {
    Adverb,
    Preposition,
    Conjunction,
    Particle,
    Interjection,
    ProperNoun,
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Determiner,
    Numeral,
    Participle,
}

impl PartOfSpeech {
    pub const ALL: [Self; 13] = [
        Self::Adverb,
        Self::Preposition,
        Self::Conjunction,
        Self::Particle,
        Self::Interjection,
        Self::ProperNoun,
        Self::Noun,
        Self::Adjective,
        Self::Verb,
        Self::Pronoun,
        Self::Determiner,
        Self::Numeral,
        Self::Participle,
    ];

    /// Returns the stable code used by reviewed registries and CLI output.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Adverb => "adverb",
            Self::Preposition => "preposition",
            Self::Conjunction => "conjunction",
            Self::Particle => "particle",
            Self::Interjection => "interjection",
            Self::ProperNoun => "proper-noun",
            Self::Noun => "noun",
            Self::Adjective => "adjective",
            Self::Verb => "verb",
            Self::Pronoun => "pronoun",
            Self::Determiner => "determiner",
            Self::Numeral => "numeral",
            Self::Participle => "participle",
        }
    }

    /// Parses an exact stable registry code.
    #[must_use]
    pub fn from_code(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|part| part.code() == value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LexemeSummary {
    pub(crate) id: LexemeId,
    pub(crate) lemma: String,
    pub(crate) part_of_speech: PartOfSpeech,
    pub(crate) source_id: String,
}

impl LexemeSummary {
    pub(crate) fn new(
        id: LexemeId,
        lemma: String,
        part_of_speech: PartOfSpeech,
        source_id: String,
    ) -> Self {
        Self {
            id,
            lemma,
            part_of_speech,
            source_id,
        }
    }

    #[must_use]
    pub fn id(&self) -> &LexemeId {
        &self.id
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    #[must_use]
    pub const fn part_of_speech(&self) -> PartOfSpeech {
        self.part_of_speech
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AlignmentSummary {
    pub mapping_id: String,
    pub source_lexeme_id: String,
    pub target_lexeme_id: String,
    pub relation: String,
    pub status: String,
    pub morphology: String,
    pub semantics: String,
    pub confidence_basis_points: u16,
    pub transformations: Vec<String>,
    pub review_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TransformationRuleSummary {
    pub rule_id: String,
    pub source_recension: String,
    pub target_recension: String,
    pub operation: String,
    pub status: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RecensionConflictSummary {
    pub conflict_id: String,
    pub source_lexeme_id: String,
    pub target_lexeme_id: String,
    pub kind: String,
    pub status: String,
    pub supporting_evidence: String,
    pub contradicting_evidence: String,
    pub resolution: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PositionalRuleSummary {
    pub rule_id: String,
    pub input: String,
    pub context: String,
    pub output: String,
    pub exceptions: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IrregularOverrideSummary {
    pub lexeme_id: String,
    pub system: String,
    pub cell_set: String,
    pub evidence_id: String,
}

/// One exhaustively reviewed entry from Alypy §104's irregular-verb inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IrregularVerbInventorySummary {
    pub source_order: u8,
    pub headword: String,
    pub systems: Vec<String>,
    pub strategy: String,
    pub implementation_status: String,
    pub evidence_id: String,
    pub note: String,
}

/// Reviewable lexical metadata exposed without leaking the generated registry
/// representation. Empty source fields stay `None` rather than being guessed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LexicalMetadataSummary {
    pub lexeme_id: LexemeId,
    pub class: Option<String>,
    pub stem: Option<String>,
    pub gender: Option<String>,
    pub aspect: Option<String>,
    pub source_id: String,
    pub target_recension: String,
    pub noun_restriction: Option<NounRestrictionSummary>,
    pub principal_parts: Vec<PrincipalPartSummary>,
    pub exact_forms: Vec<ExactFormSummary>,
    pub accents: Vec<AccentSummary>,
    pub accent_paradigms: Vec<AccentParadigmSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounRestrictionSummary {
    pub number_inventory: String,
    pub animacy_inventory: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PrincipalPartSummary {
    pub system: String,
    pub value: String,
    pub formation: Option<String>,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ExactFormSummary {
    pub cell: String,
    pub expanded: String,
    pub printed: String,
    pub evidence_id: String,
    pub source_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentSummary {
    pub cell: String,
    pub expanded: String,
    pub accented: String,
    pub evidence_id: String,
    pub source_id: String,
    pub source_recension: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentParadigmSummary {
    pub paradigm_id: String,
    pub scope: String,
    pub placement: String,
    pub mark: String,
    pub breathing: Option<String>,
    pub evidence_id: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExactFormRecord {
    pub expanded: &'static str,
    pub printed: &'static str,
    pub evidence_id: &'static str,
    pub source_kind: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReviewedEvidenceRecord {
    pub id: &'static str,
    pub source_id: &'static str,
    pub source_recension: Recension,
    pub citation: &'static str,
    pub role: &'static str,
    pub note: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbbreviationRecord {
    pub lexeme_id: &'static str,
    pub sense_id: &'static str,
    pub cell: &'static str,
    pub expanded: &'static str,
    pub printed: &'static str,
    pub rule_id: &'static str,
    pub evidence_id: &'static str,
    pub reversible: bool,
    pub required_marks: &'static str,
    pub context_restrictions: &'static str,
    pub ambiguity: &'static str,
    pub source_recension: &'static str,
    pub target_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbbreviationFamilyRecord {
    pub lexeme_id: &'static str,
    pub sense_id: &'static str,
    pub expanded_prefix: &'static str,
    pub printed_prefix: &'static str,
    pub rule_id: &'static str,
    pub evidence_id: &'static str,
    pub reversible: bool,
    pub required_marks: &'static str,
    pub context_restrictions: &'static str,
    pub ambiguity: &'static str,
    pub source_recension: &'static str,
    pub target_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccentRecord {
    pub accented: &'static str,
    pub evidence_id: &'static str,
    pub source_id: &'static str,
    pub source_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InheritedAlignment {
    pub mapping_id: RecensionMappingId,
    pub source_lexeme_id: LexemeId,
    pub confidence: Confidence,
    pub evidence_ids: Vec<String>,
    pub transformations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DefectiveInventoryRecord {
    pub kind: crate::DefectKind,
    pub field: synodal_church_slavonic_core::MetadataField,
    pub reason: &'static str,
}
