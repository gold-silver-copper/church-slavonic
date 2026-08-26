use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentScope, AuthorityRole, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, GrammarCell, PositionalParadigm, PositionalRule, Recension, Result,
    SourceId,
};

/// Provenance attached to caller-supplied lexical metadata. It identifies a
/// prediction input; it never turns the resulting form into an attestation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpecificationSource {
    evidence_id: EvidenceId,
    source_id: SourceId,
    citation: String,
}

impl SpecificationSource {
    pub fn new(
        evidence_id: impl Into<EvidenceId>,
        source_id: impl Into<SourceId>,
        citation: impl Into<String>,
    ) -> Result<Self> {
        let evidence_id = evidence_id.into();
        let source_id = source_id.into();
        let citation = citation.into();
        if evidence_id.as_str().trim().is_empty()
            || source_id.as_str().trim().is_empty()
            || citation.trim().is_empty()
        {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "caller-supplied metadata requires nonempty evidence, source, and citation IDs"
                        .into(),
            });
        }
        Ok(Self {
            evidence_id,
            source_id,
            citation,
        })
    }

    #[must_use]
    pub fn evidence_id(&self) -> &EvidenceId {
        &self.evidence_id
    }

    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    #[must_use]
    pub fn citation(&self) -> &str {
        &self.citation
    }

    pub(crate) fn evidence(&self, kind: EvidenceKind) -> Evidence {
        let authority_roles = match kind {
            EvidenceKind::AccentParadigm => {
                vec![AuthorityRole::Accentual, AuthorityRole::Orthographic]
            }
            EvidenceKind::OrthographicParadigm => vec![AuthorityRole::Orthographic],
            _ => vec![AuthorityRole::Lexical, AuthorityRole::Morphological],
        };
        Evidence {
            id: self.evidence_id.clone(),
            source: self.source_id.clone(),
            source_recension: Recension::SynodalRussian,
            kind,
            authority_roles,
            epistemic_role: EpistemicRole::CallerSuppliedMetadata,
            citation: self.citation.clone(),
            note: Some("caller-supplied Synodal lexical metadata".into()),
        }
    }

    /// Builds an explicitly sourced fixed-stem accent paradigm suitable for a
    /// caller-supplied specification.
    #[must_use]
    pub fn fixed_stem_accent(
        &self,
        paradigm_id: impl Into<String>,
        scope: AccentScope,
        vowel_from_start: u8,
        mark: AccentMark,
    ) -> AccentParadigm {
        AccentParadigm::fixed_stem(
            paradigm_id,
            scope,
            vowel_from_start,
            mark,
            self.evidence(EvidenceKind::AccentParadigm),
        )
    }

    /// Builds a completely sourced positional-letter paradigm for an
    /// arbitrary caller-supplied lexeme.
    #[must_use]
    pub fn positional_paradigm(
        &self,
        paradigm_id: impl Into<String>,
        rules: Vec<PositionalRule>,
    ) -> PositionalParadigm {
        PositionalParadigm {
            id: paradigm_id.into(),
            rules,
            evidence: self.evidence(EvidenceKind::OrthographicParadigm),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum DefectKind {
    HistoricallyAbsent,
    EvidenceIncomplete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DefectiveCell {
    pub cell: GrammarCell,
    pub kind: DefectKind,
    pub field: synodal_church_slavonic_core::MetadataField,
    pub reason: String,
}

impl DefectiveCell {
    #[must_use]
    pub fn historically_absent(cell: GrammarCell, reason: impl Into<String>) -> Self {
        Self {
            cell,
            kind: DefectKind::HistoricallyAbsent,
            field: synodal_church_slavonic_core::MetadataField::IrregularOverride,
            reason: reason.into(),
        }
    }

    #[must_use]
    pub fn evidence_incomplete(
        cell: GrammarCell,
        field: synodal_church_slavonic_core::MetadataField,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            cell,
            kind: DefectKind::EvidenceIncomplete,
            field,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct SpecContext {
    pub source: SpecificationSource,
    pub accent: Option<AccentParadigm>,
    pub positional: Option<PositionalParadigm>,
    pub defective_cells: Vec<DefectiveCell>,
}

impl SpecContext {
    pub(crate) fn new(source: SpecificationSource) -> Self {
        Self {
            source,
            accent: None,
            positional: None,
            defective_cells: vec![],
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(accent) = &self.accent {
            accent.validate()?;
            if accent.evidence.source_recension != Recension::SynodalRussian {
                return Err(Error::ContradictoryMetadata {
                    reason:
                        "an explicit Synodal specification cannot use a non-Synodal accent paradigm"
                            .into(),
                });
            }
        }
        if let Some(positional) = &self.positional {
            positional.validate()?;
            if positional.evidence.source_recension != Recension::SynodalRussian {
                return Err(Error::ContradictoryMetadata {
                    reason:
                        "an explicit Synodal specification cannot use a non-Synodal positional paradigm"
                            .into(),
                });
            }
        }
        let mut defective = BTreeSet::new();
        for cell in &self.defective_cells {
            if cell.reason.trim().is_empty() {
                return Err(Error::ContradictoryMetadata {
                    reason: "a defective cell must include a nonempty diagnostic reason".into(),
                });
            }
            if !defective.insert(cell.cell) {
                return Err(Error::ContradictoryMetadata {
                    reason: "a defective cell cannot be listed twice".into(),
                });
            }
        }
        Ok(())
    }
}
