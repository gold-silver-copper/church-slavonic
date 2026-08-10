use synodal_church_slavonic_core::{
    Error, EvidenceId, LexemeId, MetadataField, Result, RuleId, SynodalWord,
};

use crate::registry;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Abbreviation {
    pub lexeme_id: LexemeId,
    pub sense_id: String,
    pub expanded: String,
    pub printed: String,
    pub rule_id: RuleId,
    pub evidence_id: EvidenceId,
    pub reversible: bool,
}

pub fn contract(lemma: &str, sense_id: &str) -> Result<Abbreviation> {
    let summary = registry::resolve(&SynodalWord::parse(lemma)?)?;
    let record = registry::abbreviation_for(summary.id(), sense_id).ok_or(
        Error::OrthographicMetadataRequired {
            field: MetadataField::AbbreviationClass,
        },
    )?;
    Ok(from_record(record))
}

pub fn expand(printed: &str) -> Result<Vec<Abbreviation>> {
    let printed = SynodalWord::parse(printed)?;
    let candidates: Vec<Abbreviation> = registry::abbreviations_for_printed(printed.canonical())
        .into_iter()
        .map(from_record)
        .collect();
    if candidates.is_empty() {
        Err(Error::UnknownLemma {
            lookup: printed.lookup_key(),
        })
    } else {
        Ok(candidates)
    }
}

fn from_record(record: registry::AbbreviationRecord) -> Abbreviation {
    Abbreviation {
        lexeme_id: LexemeId::from(record.lexeme_id),
        sense_id: record.sense_id.into(),
        expanded: record.expanded.into(),
        printed: record.printed.into(),
        rule_id: RuleId::from(record.rule_id),
        evidence_id: EvidenceId::from(record.evidence_id),
        reversible: record.reversible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contraction_requires_semantic_identity() {
        assert!(contract("богъ", "wrong-sense").is_err());
        let result = contract("богъ", "sense:deity:christian").expect("reviewed sense");
        assert_eq!(result.printed, "бг҃ъ");
    }

    #[test]
    fn expansion_preserves_ambiguity_shape() {
        let candidates = expand("бг҃ъ").expect("known abbreviation");
        assert_eq!(candidates.len(), 1);
        assert!(!candidates[0].reversible);
    }
}
