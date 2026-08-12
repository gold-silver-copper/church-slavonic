use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, EvidenceId, FiniteTense,
    FiniteVerbCell, Gender, GrammarCell, ImperativeCell, LParticipleCell, LexemeId, MetadataField,
    NounCell, Number, NumeralCell, NumeralKind, ParticipleCell, ParticipleTense, ParticipleVoice,
    Person, PronounCell, Result, RuleId, SynodalWord,
};

use crate::registry;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Abbreviation {
    pub lexeme_id: LexemeId,
    pub sense_id: String,
    pub cell: GrammarCell,
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
        .filter(|candidate| candidate.cell == cell)
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
    let fields: Vec<_> = value.split(':').collect();
    match fields.as_slice() {
        ["lexical-form"] => Ok(GrammarCell::LexicalForm),
        ["indeclinable"] => Ok(GrammarCell::Indeclinable),
        ["noun", case, number, animacy] => Ok(GrammarCell::Noun(NounCell {
            case: parse_case(case)?,
            number: parse_number(number)?,
            animacy: parse_animacy(animacy)?,
        })),
        ["verbal-noun", case, number, animacy] => Ok(GrammarCell::VerbalNoun(NounCell {
            case: parse_case(case)?,
            number: parse_number(number)?,
            animacy: parse_animacy(animacy)?,
        })),
        ["adjective", case, number, gender, animacy, form, comparison] => {
            Ok(GrammarCell::Adjective(parse_adjective_cell(
                case, number, gender, animacy, form, comparison,
            )?))
        }
        [
            "determiner",
            case,
            number,
            gender,
            animacy,
            form,
            comparison,
        ] => Ok(GrammarCell::Determiner(parse_adjective_cell(
            case, number, gender, animacy, form, comparison,
        )?)),
        [
            tense @ ("present" | "future" | "past" | "imperfect" | "aorist"),
            person,
            number,
        ] => Ok(GrammarCell::FiniteVerb(FiniteVerbCell {
            tense: parse_finite_tense(tense)?,
            person: parse_person(person)?,
            number: parse_number(number)?,
        })),
        ["imperative", person, number] => Ok(GrammarCell::Imperative(ImperativeCell {
            person: parse_person(person)?,
            number: parse_number(number)?,
        })),
        ["infinitive"] => Ok(GrammarCell::Infinitive),
        ["supine"] => Ok(GrammarCell::Supine),
        ["l-participle", gender, number] => Ok(GrammarCell::LParticiple(LParticipleCell {
            gender: parse_gender(gender)?,
            number: parse_number(number)?,
        })),
        [
            "participle",
            tense,
            voice,
            case,
            number,
            gender,
            animacy,
            form,
            comparison,
        ] => Ok(GrammarCell::Participle(ParticipleCell {
            tense: parse_participle_tense(tense)?,
            voice: parse_participle_voice(voice)?,
            agreement: parse_adjective_cell(case, number, gender, animacy, form, comparison)?,
        })),
        ["pronoun", case, number, gender, person, animacy] => {
            Ok(GrammarCell::Pronoun(PronounCell {
                case: parse_case(case)?,
                number: parse_number(number)?,
                gender: parse_optional_gender(gender)?,
                person: parse_optional_person(person)?,
                animacy: parse_animacy_with_any(animacy)?,
            }))
        }
        ["numeral", kind, case, number, gender, animacy] => Ok(GrammarCell::Numeral(NumeralCell {
            kind: parse_numeral_kind(kind)?,
            case: parse_case(case)?,
            number: parse_number(number)?,
            gender: parse_optional_gender(gender)?,
            animacy: parse_animacy_with_any(animacy)?,
        })),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unsupported abbreviation cell key {value:?}"),
        }),
    }
}

fn parse_adjective_cell(
    case: &str,
    number: &str,
    gender: &str,
    animacy: &str,
    form: &str,
    comparison: &str,
) -> Result<AdjectiveCell> {
    Ok(AdjectiveCell {
        case: parse_case(case)?,
        number: parse_number(number)?,
        gender: parse_gender(gender)?,
        animacy: parse_animacy_with_any(animacy)?,
        form: parse_adjective_form(form)?,
        comparison: parse_comparison(comparison)?,
    })
}

fn parse_case(value: &str) -> Result<Case> {
    match value {
        "nominative" => Ok(Case::Nominative),
        "genitive" => Ok(Case::Genitive),
        "dative" => Ok(Case::Dative),
        "accusative" => Ok(Case::Accusative),
        "instrumental" => Ok(Case::Instrumental),
        "locative" => Ok(Case::Locative),
        "vocative" => Ok(Case::Vocative),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation case {value:?}"),
        }),
    }
}

fn parse_number(value: &str) -> Result<Number> {
    match value {
        "singular" => Ok(Number::Singular),
        "dual" => Ok(Number::Dual),
        "plural" => Ok(Number::Plural),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation number {value:?}"),
        }),
    }
}

fn parse_animacy(value: &str) -> Result<Animacy> {
    match value {
        "animate" => Ok(Animacy::Animate),
        "inanimate" => Ok(Animacy::Inanimate),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation animacy {value:?}"),
        }),
    }
}

fn parse_animacy_with_any(value: &str) -> Result<Animacy> {
    match value {
        // The public cell model has no third animacy value. In non-accusative
        // agreement cells, `any` is the serialized spelling for the neutral
        // (inanimate) representative used throughout the exact registry.
        "any" => Ok(Animacy::Inanimate),
        _ => parse_animacy(value),
    }
}

fn parse_gender(value: &str) -> Result<Gender> {
    match value {
        "masculine" => Ok(Gender::Masculine),
        "feminine" => Ok(Gender::Feminine),
        "neuter" => Ok(Gender::Neuter),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation gender {value:?}"),
        }),
    }
}

fn parse_optional_gender(value: &str) -> Result<Option<Gender>> {
    match value {
        "any" | "none" => Ok(None),
        _ => parse_gender(value).map(Some),
    }
}

fn parse_person(value: &str) -> Result<Person> {
    match value {
        "first" => Ok(Person::First),
        "second" => Ok(Person::Second),
        "third" => Ok(Person::Third),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation person {value:?}"),
        }),
    }
}

fn parse_optional_person(value: &str) -> Result<Option<Person>> {
    match value {
        "any" | "none" => Ok(None),
        _ => parse_person(value).map(Some),
    }
}

fn parse_finite_tense(value: &str) -> Result<FiniteTense> {
    match value {
        "present" => Ok(FiniteTense::Present),
        "future" => Ok(FiniteTense::Future),
        "past" => Ok(FiniteTense::Past),
        "imperfect" => Ok(FiniteTense::Imperfect),
        "aorist" => Ok(FiniteTense::Aorist),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation finite tense {value:?}"),
        }),
    }
}

fn parse_adjective_form(value: &str) -> Result<AdjectiveForm> {
    match value {
        "short" => Ok(AdjectiveForm::Short),
        "long" => Ok(AdjectiveForm::Long),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation adjective form {value:?}"),
        }),
    }
}

fn parse_comparison(value: &str) -> Result<Comparison> {
    match value {
        "positive" => Ok(Comparison::Positive),
        "comparative" => Ok(Comparison::Comparative),
        "superlative" => Ok(Comparison::Superlative),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation comparison {value:?}"),
        }),
    }
}

fn parse_participle_tense(value: &str) -> Result<ParticipleTense> {
    match value {
        "present" => Ok(ParticipleTense::Present),
        "past" => Ok(ParticipleTense::Past),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation participle tense {value:?}"),
        }),
    }
}

fn parse_participle_voice(value: &str) -> Result<ParticipleVoice> {
    match value {
        "active" => Ok(ParticipleVoice::Active),
        "passive" => Ok(ParticipleVoice::Passive),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation participle voice {value:?}"),
        }),
    }
}

fn parse_numeral_kind(value: &str) -> Result<NumeralKind> {
    match value {
        "cardinal" => Ok(NumeralKind::Cardinal),
        "ordinal" => Ok(NumeralKind::Ordinal),
        "collective" => Ok(NumeralKind::Collective),
        _ => Err(Error::ContradictoryMetadata {
            reason: format!("unknown abbreviation numeral kind {value:?}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let contractions = contractions("господь", "sense:v03:ed67a3345df1")
            .expect("reviewed господь contractions");
        assert_eq!(contractions.len(), 9);
        assert_eq!(
            contractions
                .iter()
                .filter(|entry| entry.printed.starts_with('Г'))
                .count(),
            2
        );
        assert!(contractions.iter().all(|entry| {
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
