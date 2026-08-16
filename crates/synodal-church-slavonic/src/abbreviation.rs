use std::collections::{BTreeMap, BTreeSet};

use synodal_church_slavonic_core::{
    Error, EvidenceId, GrammarCell, LexemeId, MetadataField, Result, RuleId, SynodalWord,
};

use crate::{Inflector, registry};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AbbreviationRealization {
    ReviewedExact,
    ProductiveFamily,
}

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
    pub realization: AbbreviationRealization,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AbbreviationPattern {
    pub expanded_prefix: String,
    pub printed_prefix: String,
}

/// A semantic, source-backed contraction family. Patterns are applied only
/// after stable lexical and sense identity are known; they are never global
/// substring rewrites.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AbbreviationFamily {
    pub lexeme_id: LexemeId,
    pub sense_id: String,
    pub patterns: Vec<AbbreviationPattern>,
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

/// Returns every reviewed semantic contraction family in stable identity
/// order. Multiple stem allomorphs remain patterns inside one family.
pub fn families() -> Result<Vec<AbbreviationFamily>> {
    let mut grouped = BTreeMap::<(String, String), Vec<registry::AbbreviationFamilyRecord>>::new();
    for record in registry::abbreviation_family_records() {
        grouped
            .entry((record.lexeme_id.into(), record.sense_id.into()))
            .or_default()
            .push(record);
    }
    grouped.into_values().map(from_family_records).collect()
}

pub fn family(lemma: &str, sense_id: &str) -> Result<AbbreviationFamily> {
    let summary = registry::resolve(&SynodalWord::parse(lemma)?)?;
    family_by_id(summary.id(), sense_id)
}

pub fn family_by_id(id: &LexemeId, sense_id: &str) -> Result<AbbreviationFamily> {
    let _ = registry::from_id(id)?;
    let records = registry::abbreviation_families_for(id, sense_id);
    if records.is_empty() {
        return Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AbbreviationClass,
        });
    }
    from_family_records(records)
}

/// Returns all contractions for a cell with reviewed exact rows first. A
/// productive family is used only when no exact row accepts the request.
pub fn contract_variants_for_cell(
    lemma: &str,
    sense_id: &str,
    cell: GrammarCell,
) -> Result<Vec<Abbreviation>> {
    let summary = registry::resolve(&SynodalWord::parse(lemma)?)?;
    contract_variants_for_cell_by_id_with(summary.id(), sense_id, cell, Inflector::default())
}

pub fn contract_variants_for_cell_by_id(
    id: &LexemeId,
    sense_id: &str,
    cell: GrammarCell,
) -> Result<Vec<Abbreviation>> {
    contract_variants_for_cell_by_id_with(id, sense_id, cell, Inflector::default())
}

pub fn contract_variants_for_cell_by_id_with(
    id: &LexemeId,
    sense_id: &str,
    cell: GrammarCell,
    inflector: Inflector,
) -> Result<Vec<Abbreviation>> {
    let exact = contractions_by_id(id, sense_id)?
        .into_iter()
        .filter(|candidate| candidate.matches_cell(cell))
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return Ok(exact);
    }

    let family = family_by_id(id, sense_id)?;
    let forms = inflector.form_by_id(id, cell)?;
    let mut seen = BTreeSet::new();
    let mut generated = Vec::new();
    for variant in forms.variants() {
        let printed = family.contract_expanded(&variant.expanded)?;
        if !seen.insert((variant.expanded.clone(), printed.clone())) {
            continue;
        }
        let mut evidence_ids = family.evidence_ids.clone();
        evidence_ids.extend(variant.evidence.iter().map(|evidence| evidence.id.clone()));
        evidence_ids.sort();
        evidence_ids.dedup();
        generated.push(Abbreviation {
            lexeme_id: id.clone(),
            sense_id: sense_id.into(),
            cell,
            cell_key: crate::resolver::cell_key(cell),
            expanded: variant.expanded.clone(),
            printed,
            rule_id: family.rule_id.clone(),
            evidence_ids,
            reversible: family.reversible,
            required_marks: family.required_marks.clone(),
            context_restrictions: family.context_restrictions.clone(),
            ambiguity: family.ambiguity.clone(),
            source_recension: family.source_recension.clone(),
            target_recension: family.target_recension.clone(),
            realization: AbbreviationRealization::ProductiveFamily,
        });
    }
    if generated.is_empty() {
        Err(Error::UnsupportedCell {
            reason: "the productive abbreviation family yielded no form for this cell".into(),
        })
    } else {
        Ok(generated)
    }
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

impl AbbreviationFamily {
    /// Contracts an unaccented expanded form with the longest matching stem
    /// allomorph. This private applicator is called only after the high-level
    /// API has resolved stable lexical and semantic identity.
    fn contract_expanded(&self, expanded: &str) -> Result<String> {
        let expanded = SynodalWord::parse(expanded)?;
        let pattern = self
            .patterns
            .iter()
            .filter_map(|pattern| {
                strip_family_prefix(expanded.canonical(), &pattern.expanded_prefix)
                    .map(|suffix| (pattern, suffix))
            })
            .max_by(|(left, _), (right, _)| {
                left.expanded_prefix
                    .len()
                    .cmp(&right.expanded_prefix.len())
                    .then_with(|| right.expanded_prefix.cmp(&left.expanded_prefix))
            })
            .ok_or_else(|| Error::UnsupportedCell {
                reason: format!(
                    "expanded form {:?} does not match abbreviation family {}",
                    expanded.canonical(),
                    self.rule_id.as_ref()
                ),
            })?;
        let printed = format!("{}{}", pattern.0.printed_prefix, pattern.1);
        SynodalWord::parse(printed.clone())?;
        Ok(printed)
    }
}

fn strip_family_prefix<'a>(expanded: &'a str, prefix: &str) -> Option<&'a str> {
    if let Some(suffix) = expanded.strip_prefix(prefix) {
        return Some(suffix);
    }
    let mut expanded_characters = expanded.char_indices();
    for prefix_character in prefix.chars() {
        let (_, expanded_character) = expanded_characters.next()?;
        if abbreviation_family_base(expanded_character)
            != abbreviation_family_base(prefix_character)
        {
            return None;
        }
    }
    let suffix_start = expanded_characters
        .next()
        .map_or(expanded.len(), |(offset, _)| offset);
    Some(&expanded[suffix_start..])
}

fn abbreviation_family_base(character: char) -> char {
    match character {
        'ѡ' | 'ѻ' | 'ꙍ' => 'о',
        'і' | 'ї' => 'и',
        'є' => 'е',
        'ꙋ' => 'у',
        'ꙗ' | 'я' => 'ѧ',
        other => other,
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
        realization: AbbreviationRealization::ReviewedExact,
    })
}

fn from_family_records(
    records: Vec<registry::AbbreviationFamilyRecord>,
) -> Result<AbbreviationFamily> {
    let first = records.first().ok_or(Error::OrthographicMetadataRequired {
        field: MetadataField::AbbreviationClass,
    })?;
    if records.iter().any(|record| {
        record.lexeme_id != first.lexeme_id
            || record.sense_id != first.sense_id
            || record.rule_id != first.rule_id
            || record.evidence_id != first.evidence_id
            || record.reversible != first.reversible
            || record.required_marks != first.required_marks
            || record.context_restrictions != first.context_restrictions
            || record.ambiguity != first.ambiguity
            || record.source_recension != first.source_recension
            || record.target_recension != first.target_recension
    }) {
        return Err(Error::ContradictoryMetadata {
            reason: "one abbreviation family has inconsistent pattern metadata".into(),
        });
    }
    let mut patterns = records
        .iter()
        .map(|record| AbbreviationPattern {
            expanded_prefix: record.expanded_prefix.into(),
            printed_prefix: record.printed_prefix.into(),
        })
        .collect::<Vec<_>>();
    patterns.sort_by(|left, right| {
        right
            .expanded_prefix
            .len()
            .cmp(&left.expanded_prefix.len())
            .then_with(|| left.expanded_prefix.cmp(&right.expanded_prefix))
    });
    patterns.dedup();
    Ok(AbbreviationFamily {
        lexeme_id: LexemeId::from(first.lexeme_id),
        sense_id: first.sense_id.into(),
        patterns,
        rule_id: RuleId::from(first.rule_id),
        evidence_ids: split_list(first.evidence_id)
            .into_iter()
            .map(EvidenceId::from)
            .collect(),
        reversible: first.reversible,
        required_marks: split_list(first.required_marks),
        context_restrictions: split_list(first.context_restrictions),
        ambiguity: first.ambiguity.into(),
        source_recension: first.source_recension.into(),
        target_recension: first.target_recension.into(),
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
    use unicode_normalization::UnicodeNormalization;

    fn structural_shape(value: &str) -> String {
        value
            .nfd()
            .filter(|character| {
                !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0308}' | '\u{0311}')
            })
            .flat_map(char::to_lowercase)
            .map(|character| match character {
                'ѡ' | 'ѻ' | 'ꙍ' => 'о',
                'і' | 'ї' => 'и',
                'є' => 'е',
                'ꙋ' => 'у',
                'ꙗ' | 'я' => 'ѧ',
                other => other,
            })
            .nfc()
            .collect()
    }

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

    #[test]
    fn semantic_families_reproduce_every_reviewed_exact_contraction_shape() {
        let families = families().expect("reviewed abbreviation families");
        assert_eq!(families.len(), 45);
        assert_eq!(
            families
                .iter()
                .map(|family| family.patterns.len())
                .sum::<usize>(),
            50
        );
        for family in families {
            let exact = contractions_by_id(&family.lexeme_id, &family.sense_id)
                .expect("reviewed exact family cells");
            assert!(!exact.is_empty());
            for row in exact {
                let generated = family
                    .contract_expanded(&row.expanded)
                    .expect("family must accept exact expanded form");
                assert_eq!(
                    structural_shape(&generated),
                    structural_shape(&row.printed),
                    "{} {:?}",
                    family.rule_id.as_ref(),
                    row.cell
                );
            }
        }
    }

    #[test]
    fn productive_families_are_exact_first_and_semantically_bounded() {
        let nominative_singular = GrammarCell::Noun(NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Animate,
        });
        let exact = contract_variants_for_cell_by_id(
            &LexemeId::from("synodal:noun:bog"),
            "sense:deity:christian",
            nominative_singular,
        )
        .expect("reviewed exact contractions");
        assert_eq!(exact.len(), 2);
        assert!(
            exact
                .iter()
                .all(|entry| { entry.realization == AbbreviationRealization::ReviewedExact })
        );

        let mary = contract_variants_for_cell_by_id(
            &LexemeId::from("synodal:noun:mary"),
            "sense:proper:mary",
            nominative_singular,
        )
        .expect("reviewed Marian source-table contraction");
        assert_eq!(mary.len(), 1);
        assert_eq!(mary[0].expanded, "маріа");
        assert_eq!(mary[0].printed, "мр҃і́ѧ");
        assert_eq!(mary[0].realization, AbbreviationRealization::ReviewedExact);

        let nominative_plural = GrammarCell::Noun(NounCell {
            case: Case::Nominative,
            number: Number::Plural,
            animacy: Animacy::Animate,
        });
        let generated = contract_variants_for_cell_by_id(
            &LexemeId::from("synodal:noun:bog"),
            "sense:deity:christian",
            nominative_plural,
        )
        .expect("productive Christian-deity contraction");
        assert!(!generated.is_empty());
        assert!(generated.iter().all(|entry| {
            entry.realization == AbbreviationRealization::ProductiveFamily
                && entry.printed.contains('\u{0483}')
                && entry.printed != entry.expanded
        }));
        assert!(family("богъ", "sense:idol").is_err());
    }

    #[test]
    fn family_allomorphs_use_longest_prefix_without_blind_replacement() {
        let bog = family("богъ", "sense:deity:christian").expect("Christian God family");
        assert_eq!(
            bog.contract_expanded("бозѣ").expect("velar alternation"),
            "бз҃ѣ"
        );
        assert!(bog.contract_expanded("многобозѣ").is_err());

        let heaven = family("небо", "sense:v03:7790891c2704").expect("heaven family");
        assert_eq!(
            heaven.contract_expanded("небесемъ").expect("extended stem"),
            "нб҃семъ"
        );
    }

    #[test]
    fn source_listed_nominal_families_cover_every_productive_noun_cell() {
        let identities = [
            ("synodal:noun:molitva", "sense:prayer"),
            ("synodal:noun:apostol", "sense:title:apostle"),
            ("synodal:noun:miloserdie", "sense:mercy-compassion"),
            ("synodal:noun:muchenik", "sense:title:martyr"),
            ("synodal:noun:nedelya", "sense:week-sunday"),
            ("synodal:noun:pravednik", "sense:righteous-person"),
            ("synodal:noun:bogoroditsa", "sense:title:theotokos"),
            ("synodal:noun:resurrection", "sense:resurrection"),
            ("synodal:noun:prestol", "sense:throne-altar"),
            ("synodal:noun:vladyka", "sense:title:vladyka"),
            ("synodal:noun:vladychitsa", "sense:title:lady"),
            ("synodal:noun:svyatitel", "sense:title:hierarch"),
            ("synodal:noun:deva-title", "sense:title:virgin-theotokos"),
            ("synodal:noun:spas-title", "sense:title:savior"),
            ("synodal:noun:episkop", "sense:title:bishop"),
            ("synodal:noun:troitsa", "sense:trinity-christian"),
            ("synodal:noun:evangelie", "sense:gospel"),
            ("synodal:noun:krest", "sense:cross-religious"),
            ("synodal:noun:krestitel", "sense:title:baptist"),
            ("synodal:noun:mary", "sense:proper:mary"),
        ];
        for (lexeme_id, sense_id) in identities {
            for cell in NounCell::inventory(&Animacy::ALL) {
                let contractions = contract_variants_for_cell_by_id(
                    &LexemeId::from(lexeme_id),
                    sense_id,
                    GrammarCell::Noun(cell),
                )
                .unwrap_or_else(|error| panic!("{lexeme_id} {cell:?}: {error}"));
                assert!(!contractions.is_empty(), "{lexeme_id} {cell:?}");
                assert!(
                    contractions
                        .iter()
                        .all(|entry| entry.printed != entry.expanded),
                    "{lexeme_id} {cell:?}"
                );
            }
        }
    }
}
