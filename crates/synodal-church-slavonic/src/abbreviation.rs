use synodal_church_slavonic_core::{
    Error, EvidenceId, GrammarCell, LexemeId, MetadataField, Result, RuleId, SynodalWord,
};

use crate::registry;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Abbreviation {
    pub lexeme_id: LexemeId,
    pub sense_id: String,
    pub cell: GrammarCell,
    /// Original registry key, retaining wildcard dimensions that have no
    /// representation in the productive request type.
    pub cell_key: String,
    pub expanded: String,
    pub printed: String,
    pub rule_id: RuleId,
    pub evidence_ids: Vec<EvidenceId>,
    pub reversible: bool,
    pub required_marks: Vec<String>,
    pub context_restrictions: Vec<String>,
    pub ambiguity: String,
    pub source_recension: String,
    pub target_recension: String,
}

pub fn contract(lemma: &str, sense_id: &str) -> Result<Abbreviation> {
    let candidates = contractions(lemma, sense_id)?;
    match candidates.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AbbreviationClass,
        }),
        candidates => Err(Error::AmbiguousVariant {
            count: candidates.len(),
        }),
    }
}

/// Returns every reviewed contraction for one semantic identity. Grammatical
/// cells remain explicit; callers must not contract an arbitrary surface.
pub fn contractions(lemma: &str, sense_id: &str) -> Result<Vec<Abbreviation>> {
    let summary = registry::resolve(&SynodalWord::parse(lemma)?)?;
    contractions_by_id(summary.id(), sense_id)
}

/// Returns all reviewed contractions for a stable lexical and semantic
/// identity without resolving a potentially ambiguous lemma.
pub fn contractions_by_id(id: &LexemeId, sense_id: &str) -> Result<Vec<Abbreviation>> {
    let _ = registry::from_id(id)?;
    registry::abbreviations_for(id, sense_id)
        .into_iter()
        .map(from_record)
        .collect()
}

pub fn contract_for_cell(lemma: &str, sense_id: &str, cell: GrammarCell) -> Result<Abbreviation> {
    let candidates: Vec<_> = contractions(lemma, sense_id)?
        .into_iter()
        .filter(|candidate| candidate.matches_cell(cell))
        .collect();
    match candidates.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(Error::UnsupportedCell {
            reason: "no reviewed abbreviation for this grammatical cell".into(),
        }),
        candidates => Err(Error::AmbiguousVariant {
            count: candidates.len(),
        }),
    }
}

impl Abbreviation {
    /// Whether this reviewed registry cell accepts the typed request,
    /// including explicit `any` wildcard dimensions.
    #[must_use]
    pub fn matches_cell(&self, cell: GrammarCell) -> bool {
        crate::resolver::exact_lookup_keys(cell).contains(&self.cell_key)
    }
}

pub fn expand(printed: &str) -> Result<Vec<Abbreviation>> {
    let printed = SynodalWord::parse(printed)?;
    let candidates: Vec<Abbreviation> = registry::abbreviations_for_printed(printed.canonical())
        .into_iter()
        .map(from_record)
        .collect::<Result<Vec<_>>>()?;
    if candidates.is_empty() {
        Err(Error::UnknownLemma {
            lookup: printed.lookup_key(),
        })
    } else {
        Ok(candidates)
    }
}

fn from_record(record: registry::AbbreviationRecord) -> Result<Abbreviation> {
    Ok(Abbreviation {
        lexeme_id: LexemeId::from(record.lexeme_id),
        sense_id: record.sense_id.into(),
        cell: parse_cell(record.cell)?,
        cell_key: record.cell.into(),
        expanded: record.expanded.into(),
        printed: record.printed.into(),
        rule_id: RuleId::from(record.rule_id),
        evidence_ids: split_list(record.evidence_id)
            .into_iter()
            .map(EvidenceId::from)
            .collect(),
        reversible: record.reversible,
        required_marks: split_list(record.required_marks),
        context_restrictions: split_list(record.context_restrictions),
        ambiguity: record.ambiguity.into(),
        source_recension: record.source_recension.into(),
        target_recension: record.target_recension.into(),
    })
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(Into::into)
        .collect()
}

fn parse_cell(value: &str) -> Result<GrammarCell> {
    value.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use synodal_church_slavonic_core::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, FiniteTense, FiniteVerbCell,
        Gender, NounCell, Number, ParticipleCell, ParticipleTense, ParticipleVoice, Person,
    };

    #[test]
    fn contraction_requires_semantic_identity() {
        assert!(contract("богъ", "wrong-sense").is_err());
        let nominative = GrammarCell::Noun(NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Animate,
        });
        assert!(matches!(
            contract_for_cell("богъ", "sense:deity:christian", nominative),
            Err(Error::AmbiguousVariant { count: 2 })
        ));
        let nominatives = contractions("богъ", "sense:deity:christian")
            .expect("reviewed semantic identity")
            .into_iter()
            .filter(|candidate| candidate.cell == nominative)
            .collect::<Vec<_>>();
        assert_eq!(nominatives.len(), 2);
        assert!(nominatives.iter().any(|result| result.printed == "бг҃ъ"));
        assert!(nominatives.iter().any(|result| result.printed == "Бг҃ъ"));
        assert!(matches!(
            contract("богъ", "sense:deity:christian"),
            Err(Error::AmbiguousVariant { count: 8 })
        ));
    }

    #[test]
    fn expansion_preserves_ambiguity_shape() {
        let candidates = expand("бг҃ъ").expect("known abbreviation");
        assert_eq!(candidates.len(), 1);
        assert!(!candidates[0].reversible);

        let oblique = expand("бг҃а").expect("reviewed homographic abbreviation");
        assert_eq!(oblique.len(), 2);
        assert!(oblique.iter().any(|candidate| {
            matches!(
                candidate.cell,
                GrammarCell::Noun(NounCell {
                    case: Case::Genitive,
                    ..
                })
            )
        }));
        assert!(oblique.iter().any(|candidate| {
            matches!(
                candidate.cell,
                GrammarCell::Noun(NounCell {
                    case: Case::Accusative,
                    ..
                })
            )
        }));
    }

    #[test]
    fn contraction_registry_preserves_cells_and_review_metadata() {
        let reviewed_contractions = contractions("господь", "sense:v03:ed67a3345df1")
            .expect("reviewed господь contractions");
        assert_eq!(reviewed_contractions.len(), 9);
        assert_eq!(
            reviewed_contractions
                .iter()
                .filter(|entry| entry.printed.starts_with('Г'))
                .count(),
            2
        );
        assert!(reviewed_contractions.iter().all(|entry| {
            !entry.reversible
                && entry.required_marks.iter().any(|mark| mark == "titlo")
                && !entry.context_restrictions.is_empty()
                && !entry.ambiguity.is_empty()
                && entry.source_recension == "synodal-russian"
                && entry.target_recension == "synodal-russian"
                && entry.evidence_ids.len() >= 2
        }));
        assert!(matches!(
            contract("господь", "sense:v03:ed67a3345df1"),
            Err(Error::AmbiguousVariant { count: 9 })
        ));

        let israel = contractions("израилевъ", "sense:v06:israel-adjective")
            .expect("reviewed wildcard adjective contractions");
        let animate_dative = GrammarCell::Adjective(AdjectiveCell {
            case: Case::Dative,
            number: Number::Plural,
            gender: Gender::Masculine,
            animacy: Animacy::Animate,
            form: AdjectiveForm::Long,
            comparison: Comparison::Positive,
        });
        assert!(israel.iter().any(|candidate| {
            candidate.cell_key == "adjective:dative:plural:masculine:any:long:positive"
                && candidate.matches_cell(animate_dative)
        }));
    }

    #[test]
    fn expansion_retains_the_exact_grammatical_analysis() {
        let candidates = expand("гдⷭ҇а").expect("reviewed accusative contraction");
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].cell,
            GrammarCell::Noun(NounCell {
                case: Case::Accusative,
                number: Number::Singular,
                animacy: Animacy::Animate,
            })
        );
        assert_eq!(candidates[0].expanded, "господа");
    }

    #[test]
    fn typed_cell_parser_covers_verbal_and_agreement_cells() {
        assert_eq!(
            parse_cell("aorist:first:singular").expect("finite verb cell"),
            GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            })
        );
        assert_eq!(
            parse_cell(
                "participle:present:active:nominative:singular:masculine:animate:short:positive",
            )
            .expect("participle cell"),
            GrammarCell::Participle(ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Animate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            })
        );
        assert!(parse_cell("aorist:fourth:singular").is_err());
    }

    #[test]
    fn expansion_rejects_missing_and_malformed_required_marks() {
        assert!(matches!(expand("гдса"), Err(Error::UnknownLemma { .. })));
        assert!(matches!(
            expand("\u{301}гдⷭ҇а"),
            Err(Error::InvalidOrthography { .. })
        ));
        assert!(matches!(
            expand("гдⷭ҇а\u{e000}"),
            Err(Error::InvalidUnicode { .. })
        ));
    }
}
