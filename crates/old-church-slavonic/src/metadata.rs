//! Typed runtime view of the statically generated verb principal parts.

use crate::dictionary::VERB_METADATA;
use old_church_slavonic_core::{
    AoristFormation, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    InflectionError, MetadataEvidence, MetadataField, MetadataProvenance, PartOfSpeech,
    PastActiveParticipleFormation, PastPassiveParticipleFormation,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, VerbAspect, VerbClass,
    orthography::{Script, detect_script},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedMetadata<T> {
    pub value: T,
    pub evidence: MetadataEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentMetadataAnalysis {
    pub analysis_rank: u16,
    pub class: SourcedMetadata<VerbClass>,
    pub stem: SourcedMetadata<String>,
    pub first_singular_stem: Option<SourcedMetadata<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbSystemMetadata<F> {
    pub analysis_rank: u16,
    pub stem: SourcedMetadata<String>,
    pub formation: SourcedMetadata<F>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AoristMetadataAnalysis {
    pub analysis_rank: u16,
    pub stem: SourcedMetadata<String>,
    pub second_third_singular: Option<SourcedMetadata<String>>,
    pub formation: SourcedMetadata<AoristFormation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImperfectMetadataAnalysis {
    pub analysis_rank: u16,
    pub stem: SourcedMetadata<String>,
    pub formation: SourcedMetadata<ImperfectFormation>,
    pub variant_policy: SourcedMetadata<ImperfectVariantPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbStemMetadata {
    pub analysis_rank: u16,
    pub stem: SourcedMetadata<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryVerbMetadata {
    pub lexeme_id: String,
    pub lemma: String,
    pub aspect: Option<SourcedMetadata<VerbAspect>>,
    pub present: Vec<PresentMetadataAnalysis>,
    pub imperfect: Vec<ImperfectMetadataAnalysis>,
    pub aorist: Vec<AoristMetadataAnalysis>,
    pub imperative: Vec<VerbSystemMetadata<ImperativeFormation>>,
    pub l_participle: Vec<VerbStemMetadata>,
    pub present_active_participle: Vec<VerbSystemMetadata<PresentActiveParticipleFormation>>,
    pub present_passive_participle: Vec<VerbSystemMetadata<PresentPassiveParticipleFormation>>,
    pub past_active_participle: Vec<VerbSystemMetadata<PastActiveParticipleFormation>>,
    pub past_passive_participle: Vec<VerbSystemMetadata<PastPassiveParticipleFormation>>,
}

/// Reviewable interchange record used by the offline evaluator as well as the
/// generated static registry. Codes are validated into enums by
/// `DictionaryVerbMetadata::from_normalized_fields`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedVerbMetadataField {
    pub system: String,
    pub analysis_rank: u16,
    pub field: String,
    pub value: String,
    pub provenance: String,
    pub source_feature: String,
    pub source_form: String,
    pub crosscheck_features: Vec<String>,
    pub authority: String,
}

type FieldGroup = BTreeMap<String, NormalizedVerbMetadataField>;
type FieldGroups = BTreeMap<(String, u16), FieldGroup>;

pub fn verb_metadata_by_id(id: &str) -> Result<DictionaryVerbMetadata, InflectionError> {
    let lexeme = crate::lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?;
    if lexeme.pos != "verb" {
        return Err(InflectionError::InvalidInput {
            reason: format!("lexeme {id} is {}, not verb", lexeme.pos),
        });
    }
    let start = VERB_METADATA.partition_point(|row| row.lexeme_id < id);
    let end = VERB_METADATA.partition_point(|row| row.lexeme_id <= id);
    let fields = VERB_METADATA[start..end]
        .iter()
        .map(|row| NormalizedVerbMetadataField {
            system: row.system.to_string(),
            analysis_rank: row.analysis_rank,
            field: row.field.to_string(),
            value: row.value.to_string(),
            provenance: row.provenance.to_string(),
            source_feature: row.source_feature.to_string(),
            source_form: row.source_form.to_string(),
            crosscheck_features: row
                .crosscheck_features
                .split(" || ")
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            authority: row.authority.to_string(),
        });
    DictionaryVerbMetadata::from_normalized_fields(id, lexeme.lemma, fields)
}

impl DictionaryVerbMetadata {
    pub fn from_normalized_fields(
        lexeme_id: impl Into<String>,
        lemma: impl Into<String>,
        fields: impl IntoIterator<Item = NormalizedVerbMetadataField>,
    ) -> Result<Self, InflectionError> {
        let lexeme_id = lexeme_id.into();
        let lemma = old_church_slavonic_core::orthography::canonical_display(&lemma.into())?;
        let mut groups: FieldGroups = BTreeMap::new();
        let mut ranks: BTreeMap<String, BTreeSet<u16>> = BTreeMap::new();
        for field in fields {
            validate_field_shape(&field)?;
            ranks
                .entry(field.system.clone())
                .or_default()
                .insert(field.analysis_rank);
            let key = (field.system.clone(), field.analysis_rank);
            let field_name = field.field.clone();
            let group = groups.entry(key).or_default();
            if let Some(previous) = group.get(&field_name) {
                if previous.value != field.value {
                    return Err(InflectionError::ContradictoryLexicalMetadata {
                        fields: vec![metadata_field(&field.system, &field.field)],
                    });
                }
                return Err(InflectionError::InvalidInput {
                    reason: format!("duplicate normalized verb metadata field: {field_name}"),
                });
            }
            group.insert(field_name, field);
        }
        for (system, observed) in ranks {
            for (expected, rank) in observed.into_iter().enumerate() {
                if usize::from(rank) != expected {
                    return Err(InflectionError::InvalidInput {
                        reason: format!("non-contiguous normalized {system} analysis rank {rank}"),
                    });
                }
            }
        }

        let aspect = groups
            .get(&("aspect".to_string(), 0))
            .map(parse_aspect_group)
            .transpose()?;
        let present = parse_present_groups(&groups)?;
        if present.iter().any(|analysis| {
            matches!(
                analysis.class.value,
                VerbClass::II1 | VerbClass::II2 | VerbClass::II3
            ) && analysis.first_singular_stem.is_none()
        }) {
            return Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::PresentFirstSingularStem],
            });
        }
        let past_active_participle =
            parse_system_groups(&groups, "past-active-participle", parse_past_active)?;
        if past_active_participle.iter().any(|analysis| {
            analysis.formation.value == PastActiveParticipleFormation::VushAfterOvToU
                && !analysis.stem.value.ends_with("ов")
        }) {
            return Err(InflectionError::InvalidInput {
                reason: "ov-to-u metadata requires a past-active stem ending in -ов".to_string(),
            });
        }
        Ok(Self {
            lexeme_id,
            lemma,
            aspect,
            present,
            imperfect: parse_imperfect_groups(&groups)?,
            aorist: parse_aorist_groups(&groups)?,
            imperative: parse_system_groups(&groups, "imperative", parse_imperative)?,
            l_participle: parse_stem_groups(&groups, "l-participle")?,
            present_active_participle: parse_system_groups(
                &groups,
                "present-active-participle",
                parse_present_active,
            )?,
            present_passive_participle: parse_system_groups(
                &groups,
                "present-passive-participle",
                parse_present_passive,
            )?,
            past_active_participle,
            past_passive_participle: parse_system_groups(
                &groups,
                "past-passive-participle",
                parse_past_passive,
            )?,
        })
    }
}

fn validate_field_shape(field: &NormalizedVerbMetadataField) -> Result<(), InflectionError> {
    if field.system.is_empty()
        || field.field.is_empty()
        || field.value.is_empty()
        || field.provenance.is_empty()
        || field.source_feature.is_empty()
        || field.source_form.is_empty()
        || field.authority.is_empty()
    {
        return Err(InflectionError::InvalidInput {
            reason: "normalized verb metadata contains an empty required value".to_string(),
        });
    }
    if matches!(
        field.field.as_str(),
        "stem" | "first-singular-stem" | "second-third-singular"
    ) && old_church_slavonic_core::orthography::canonical_display(&field.value)? != field.value
    {
        return Err(InflectionError::InvalidInput {
            reason: "normalized verb metadata stem is not NFC".to_string(),
        });
    }
    if matches!(
        field.field.as_str(),
        "stem" | "first-singular-stem" | "second-third-singular"
    ) && detect_script(&field.value) != Script::Cyrillic
    {
        return Err(InflectionError::InvalidInput {
            reason: "normalized productive verb metadata stem is not Cyrillic".to_string(),
        });
    }
    if old_church_slavonic_core::orthography::canonical_display(&field.source_form)?
        != field.source_form
    {
        return Err(InflectionError::InvalidInput {
            reason: "normalized verb metadata source form is not NFC".to_string(),
        });
    }
    if !field.source_feature.starts_with("headword:")
        && !matches!(
            detect_script(&field.source_form),
            Script::Cyrillic | Script::Glagolitic
        )
    {
        return Err(InflectionError::InvalidInput {
            reason: "normalized verb metadata source form is not in an OCS script".to_string(),
        });
    }
    let field_is_valid = match field.system.as_str() {
        "aspect" => field.field == "aspect" && field.analysis_rank == 0,
        "present" => matches!(
            field.field.as_str(),
            "class" | "stem" | "first-singular-stem"
        ),
        "l-participle" => field.field == "stem",
        "imperfect" => matches!(
            field.field.as_str(),
            "stem" | "formation" | "variant-policy"
        ),
        "aorist" => matches!(
            field.field.as_str(),
            "stem" | "second-third-singular" | "formation"
        ),
        "imperative"
        | "present-active-participle"
        | "present-passive-participle"
        | "past-active-participle"
        | "past-passive-participle" => matches!(field.field.as_str(), "stem" | "formation"),
        _ => false,
    };
    if !field_is_valid {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "generated registry has an unknown or invalid verb metadata field: {}:{}",
                field.system, field.field
            ),
        });
    }
    Ok(())
}

fn parse_aspect_group(group: &FieldGroup) -> Result<SourcedMetadata<VerbAspect>, InflectionError> {
    let row = required(group, "aspect", MetadataField::VerbAspect)?;
    let value = match row.value.as_str() {
        "perfective" => VerbAspect::Perfective,
        "imperfective" => VerbAspect::Imperfective,
        "biaspectual" => VerbAspect::Biaspectual,
        value => return invalid_code("aspect", value),
    };
    sourced(row, MetadataField::VerbAspect, value)
}

fn parse_present_groups(
    groups: &FieldGroups,
) -> Result<Vec<PresentMetadataAnalysis>, InflectionError> {
    let mut out = Vec::new();
    for ((system, rank), group) in groups {
        if system != "present" {
            continue;
        }
        let class_row = required(group, "class", MetadataField::VerbClass)?;
        let class = parse_class(&class_row.value)?;
        let stem = required(group, "stem", MetadataField::PresentStem)?;
        let first_singular_stem = group
            .get("first-singular-stem")
            .map(|row| {
                sourced(
                    row,
                    MetadataField::PresentFirstSingularStem,
                    row.value.clone(),
                )
            })
            .transpose()?;
        out.push(PresentMetadataAnalysis {
            analysis_rank: *rank,
            class: sourced(class_row, MetadataField::VerbClass, class)?,
            stem: sourced(stem, MetadataField::PresentStem, stem.value.clone())?,
            first_singular_stem,
        });
    }
    Ok(out)
}

fn parse_system_groups<F: Copy>(
    groups: &FieldGroups,
    system_name: &str,
    parse_formation: fn(&str) -> Result<F, InflectionError>,
) -> Result<Vec<VerbSystemMetadata<F>>, InflectionError> {
    let (stem_field, formation_field) = fields_for_system(system_name);
    let mut out = Vec::new();
    for ((system, rank), group) in groups {
        if system != system_name {
            continue;
        }
        let stem = required(group, "stem", stem_field)?;
        let formation = required(group, "formation", formation_field)?;
        out.push(VerbSystemMetadata {
            analysis_rank: *rank,
            stem: sourced(stem, stem_field, stem.value.clone())?,
            formation: sourced(
                formation,
                formation_field,
                parse_formation(&formation.value)?,
            )?,
        });
    }
    Ok(out)
}

fn parse_aorist_groups(
    groups: &FieldGroups,
) -> Result<Vec<AoristMetadataAnalysis>, InflectionError> {
    let mut out = Vec::new();
    for ((system, rank), group) in groups {
        if system != "aorist" {
            continue;
        }
        let stem = required(group, "stem", MetadataField::AoristStem)?;
        let formation = required(group, "formation", MetadataField::AoristFormation)?;
        let formation_value = parse_aorist(&formation.value)?;
        let second_third_singular = group
            .get("second-third-singular")
            .map(|row| {
                sourced(
                    row,
                    MetadataField::AoristSecondThirdSingular,
                    row.value.clone(),
                )
            })
            .transpose()?;
        let is_sigmatic = matches!(
            formation_value,
            AoristFormation::SigmaticPrimary
                | AoristFormation::SigmaticSecondary
                | AoristFormation::SigmaticVowel
        );
        if is_sigmatic && second_third_singular.is_none() {
            return Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::AoristSecondThirdSingular],
            });
        }
        if !is_sigmatic && second_third_singular.is_some() {
            return Err(InflectionError::ContradictoryLexicalMetadata {
                fields: vec![
                    MetadataField::AoristFormation,
                    MetadataField::AoristSecondThirdSingular,
                ],
            });
        }
        out.push(AoristMetadataAnalysis {
            analysis_rank: *rank,
            stem: sourced(stem, MetadataField::AoristStem, stem.value.clone())?,
            second_third_singular,
            formation: sourced(formation, MetadataField::AoristFormation, formation_value)?,
        });
    }
    Ok(out)
}

fn parse_imperfect_groups(
    groups: &FieldGroups,
) -> Result<Vec<ImperfectMetadataAnalysis>, InflectionError> {
    let mut out = Vec::new();
    for ((system, rank), group) in groups {
        if system != "imperfect" {
            continue;
        }
        let stem = required(group, "stem", MetadataField::ImperfectStem)?;
        let formation = required(group, "formation", MetadataField::ImperfectFormation)?;
        let variant_policy = required(
            group,
            "variant-policy",
            MetadataField::ImperfectVariantPolicy,
        )?;
        let variant_policy_value = match variant_policy.value.as_str() {
            "uncontracted-only" => ImperfectVariantPolicy::UncontractedOnly,
            "contracted-only" => ImperfectVariantPolicy::ContractedOnly,
            "iotated-only" => ImperfectVariantPolicy::IotatedOnly,
            value => return invalid_code("imperfect variant policy", value),
        };
        out.push(ImperfectMetadataAnalysis {
            analysis_rank: *rank,
            stem: sourced(stem, MetadataField::ImperfectStem, stem.value.clone())?,
            formation: sourced(
                formation,
                MetadataField::ImperfectFormation,
                parse_imperfect(&formation.value)?,
            )?,
            variant_policy: sourced(
                variant_policy,
                MetadataField::ImperfectVariantPolicy,
                variant_policy_value,
            )?,
        });
    }
    Ok(out)
}

fn parse_stem_groups(
    groups: &FieldGroups,
    system_name: &str,
) -> Result<Vec<VerbStemMetadata>, InflectionError> {
    let mut out = Vec::new();
    for ((system, rank), group) in groups {
        if system != system_name {
            continue;
        }
        let stem = required(group, "stem", MetadataField::LParticipleStem)?;
        out.push(VerbStemMetadata {
            analysis_rank: *rank,
            stem: sourced(stem, MetadataField::LParticipleStem, stem.value.clone())?,
        });
    }
    Ok(out)
}

fn required<'a>(
    group: &'a FieldGroup,
    name: &str,
    needed: MetadataField,
) -> Result<&'a NormalizedVerbMetadataField, InflectionError> {
    group
        .get(name)
        .ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![needed],
        })
}

fn sourced<T>(
    row: &NormalizedVerbMetadataField,
    field: MetadataField,
    value: T,
) -> Result<SourcedMetadata<T>, InflectionError> {
    let provenance = match row.provenance.as_str() {
        "dictionary-principal-part" | "dictionary-headword-metadata" => {
            MetadataProvenance::DictionaryPrincipalPart
        }
        "curated-grammar-override" => MetadataProvenance::CuratedGrammarOverride,
        value => return invalid_code("metadata provenance", value),
    };
    Ok(SourcedMetadata {
        value,
        evidence: MetadataEvidence {
            field: Some(field),
            provenance,
            source_feature: Some(row.source_feature.clone()),
            source_form: Some(row.source_form.clone()),
            crosscheck_features: row.crosscheck_features.clone(),
            authority: Some(row.authority.clone()),
        },
    })
}

fn fields_for_system(system: &str) -> (MetadataField, MetadataField) {
    match system {
        "imperfect" => (
            MetadataField::ImperfectStem,
            MetadataField::ImperfectFormation,
        ),
        "aorist" => (MetadataField::AoristStem, MetadataField::AoristFormation),
        "imperative" => (
            MetadataField::ImperativeStem,
            MetadataField::ImperativeFormation,
        ),
        "present-active-participle" => (
            MetadataField::PresentActiveParticipleStem,
            MetadataField::PresentActiveParticipleFormation,
        ),
        "present-passive-participle" => (
            MetadataField::PresentPassiveParticipleStem,
            MetadataField::PresentPassiveParticipleFormation,
        ),
        "past-active-participle" => (
            MetadataField::PastActiveParticipleStem,
            MetadataField::PastActiveParticipleFormation,
        ),
        "past-passive-participle" => (
            MetadataField::PastPassiveParticipleStem,
            MetadataField::PastPassiveParticipleFormation,
        ),
        _ => (MetadataField::VerbClass, MetadataField::VerbClass),
    }
}

fn metadata_field(system: &str, field: &str) -> MetadataField {
    match (system, field) {
        ("aspect", "aspect") => MetadataField::VerbAspect,
        ("present", "class") => MetadataField::VerbClass,
        ("present", "stem") => MetadataField::PresentStem,
        ("present", "first-singular-stem") => MetadataField::PresentFirstSingularStem,
        ("imperfect", "variant-policy") => MetadataField::ImperfectVariantPolicy,
        ("aorist", "second-third-singular") => MetadataField::AoristSecondThirdSingular,
        ("l-participle", "stem") => MetadataField::LParticipleStem,
        (_, "stem") => fields_for_system(system).0,
        (_, "formation") => fields_for_system(system).1,
        _ => MetadataField::VerbClass,
    }
}

fn parse_class(value: &str) -> Result<VerbClass, InflectionError> {
    match value {
        "IA1" => Ok(VerbClass::IA1),
        "IA2" => Ok(VerbClass::IA2),
        "II1" => Ok(VerbClass::II1),
        "II2" => Ok(VerbClass::II2),
        "II3" => Ok(VerbClass::II3),
        value => invalid_code("present class", value),
    }
}

fn parse_imperfect(value: &str) -> Result<ImperfectFormation, InflectionError> {
    match value {
        "a" => Ok(ImperfectFormation::A),
        "yat-a" => Ok(ImperfectFormation::YatA),
        "palatalized-a" => Ok(ImperfectFormation::PalatalizedA),
        "present-a" => Ok(ImperfectFormation::PresentA),
        "present-yat-a" => Ok(ImperfectFormation::PresentYatA),
        value => invalid_code("imperfect formation", value),
    }
}

fn parse_aorist(value: &str) -> Result<AoristFormation, InflectionError> {
    match value {
        "asigmatic" => Ok(AoristFormation::Asigmatic),
        "new" => Ok(AoristFormation::New),
        "sigmatic-primary" => Ok(AoristFormation::SigmaticPrimary),
        "sigmatic-secondary" => Ok(AoristFormation::SigmaticSecondary),
        "sigmatic-vowel" => Ok(AoristFormation::SigmaticVowel),
        value => invalid_code("aorist formation", value),
    }
}

fn parse_imperative(value: &str) -> Result<ImperativeFormation, InflectionError> {
    match value {
        "i-series" => Ok(ImperativeFormation::ISeries),
        "yat-series" => Ok(ImperativeFormation::YatSeries),
        value => invalid_code("imperative formation", value),
    }
}

fn parse_present_active(value: &str) -> Result<PresentActiveParticipleFormation, InflectionError> {
    match value {
        "yusht-hard" => Ok(PresentActiveParticipleFormation::YushtHard),
        "yusht-soft" => Ok(PresentActiveParticipleFormation::YushtSoft),
        "yesht-soft" => Ok(PresentActiveParticipleFormation::YeshtSoft),
        "mixed-yusht-soft" => Ok(PresentActiveParticipleFormation::MixedYushtSoft),
        "iotated-yusht-soft" => Ok(PresentActiveParticipleFormation::IotatedYushtSoft),
        value => invalid_code("present active participle formation", value),
    }
}

fn parse_present_passive(
    value: &str,
) -> Result<PresentPassiveParticipleFormation, InflectionError> {
    match value {
        "im" => Ok(PresentPassiveParticipleFormation::Im),
        "em" => Ok(PresentPassiveParticipleFormation::Em),
        "iotated-em" => Ok(PresentPassiveParticipleFormation::IotatedEm),
        "om" => Ok(PresentPassiveParticipleFormation::Om),
        value => invalid_code("present passive participle formation", value),
    }
}

fn parse_past_active(value: &str) -> Result<PastActiveParticipleFormation, InflectionError> {
    match value {
        "ush" => Ok(PastActiveParticipleFormation::Ush),
        "ish" => Ok(PastActiveParticipleFormation::Ish),
        "vush-after-j-deletion" => Ok(PastActiveParticipleFormation::VushAfterJDeletion),
        "vush-after-ov-to-u" => Ok(PastActiveParticipleFormation::VushAfterOvToU),
        "vush" => Ok(PastActiveParticipleFormation::Vush),
        value => invalid_code("past active participle formation", value),
    }
}

fn parse_past_passive(value: &str) -> Result<PastPassiveParticipleFormation, InflectionError> {
    match value {
        "t" => Ok(PastPassiveParticipleFormation::T),
        "n" => Ok(PastPassiveParticipleFormation::N),
        "en" => Ok(PastPassiveParticipleFormation::En),
        value => invalid_code("past passive participle formation", value),
    }
}

fn invalid_code<T>(kind: &str, value: &str) -> Result<T, InflectionError> {
    Err(InflectionError::InvalidInput {
        reason: format!("generated registry has an unknown {kind} code: {value}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(system: &str, rank: u16, name: &str, value: &str) -> NormalizedVerbMetadataField {
        NormalizedVerbMetadataField {
            system: system.to_string(),
            analysis_rank: rank,
            field: name.to_string(),
            value: value.to_string(),
            provenance: "dictionary-principal-part".to_string(),
            source_feature: "verb:participle:past-active:citation".to_string(),
            source_form: "правль".to_string(),
            crosscheck_features: vec!["verb:l-participle:m:sg".to_string()],
            authority: "fixture-dictionary".to_string(),
        }
    }

    #[test]
    fn rejects_unknown_system_field_and_formation_codes() {
        for fields in [
            vec![field("unknown", 0, "stem", "правл")],
            vec![field("l-participle", 0, "unknown", "правл")],
            vec![
                field("past-active-participle", 0, "stem", "правл"),
                field("past-active-participle", 0, "formation", "guess"),
            ],
            vec![
                field("imperfect", 0, "stem", "прав"),
                field("imperfect", 0, "formation", "a"),
                field("imperfect", 0, "variant-policy", "guess"),
            ],
        ] {
            assert!(
                DictionaryVerbMetadata::from_normalized_fields("fixture", "правити", fields)
                    .is_err()
            );
        }
    }

    #[test]
    fn rejects_empty_non_nfc_and_duplicate_stems() {
        let mut empty = field("l-participle", 0, "stem", "прави");
        empty.value.clear();
        assert!(
            DictionaryVerbMetadata::from_normalized_fields("fixture", "правити", [empty]).is_err()
        );
        let non_nfc = field("l-participle", 0, "stem", "И\u{306}");
        assert!(
            DictionaryVerbMetadata::from_normalized_fields("fixture", "правити", [non_nfc])
                .is_err()
        );
        let duplicate = field("l-participle", 0, "stem", "прави");
        assert!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "правити",
                [duplicate.clone(), duplicate],
            )
            .is_err()
        );
        assert!(matches!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "правити",
                [
                    field("l-participle", 0, "stem", "прави"),
                    field("l-participle", 0, "stem", "правл"),
                ],
            ),
            Err(InflectionError::ContradictoryLexicalMetadata { .. })
        ));
        let latin_stem = field("l-participle", 0, "stem", "pravi");
        assert!(
            DictionaryVerbMetadata::from_normalized_fields("fixture", "правити", [latin_stem],)
                .is_err()
        );
        let mut latin_evidence = field("l-participle", 0, "stem", "прави");
        latin_evidence.source_form = "pravil".to_string();
        assert!(
            DictionaryVerbMetadata::from_normalized_fields("fixture", "правити", [latin_evidence],)
                .is_err()
        );
        assert!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "правити",
                [
                    field("present", 0, "class", "II1"),
                    field("present", 0, "stem", "прав"),
                ],
            )
            .is_err()
        );
        assert!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "пловати",
                [
                    field("past-active-participle", 0, "stem", "пла"),
                    field(
                        "past-active-participle",
                        0,
                        "formation",
                        "vush-after-ov-to-u",
                    ),
                ],
            )
            .is_err()
        );
    }

    #[test]
    fn multiple_analyses_keep_rank_and_provenance_order() {
        let metadata = DictionaryVerbMetadata::from_normalized_fields(
            "fixture",
            "правити",
            [
                field("past-active-participle", 0, "stem", "правл"),
                field("past-active-participle", 0, "formation", "ish"),
                field("past-active-participle", 1, "stem", "прави"),
                field("past-active-participle", 1, "formation", "vush"),
            ],
        )
        .expect("valid alternatives");
        assert_eq!(metadata.past_active_participle.len(), 2);
        assert_eq!(metadata.past_active_participle[0].analysis_rank, 0);
        assert_eq!(metadata.past_active_participle[0].stem.value, "правл");
        assert_eq!(metadata.past_active_participle[1].analysis_rank, 1);
        assert_eq!(metadata.past_active_participle[1].stem.value, "прави");
        assert!(metadata.past_active_participle.iter().all(|analysis| {
            analysis.stem.evidence.provenance == MetadataProvenance::DictionaryPrincipalPart
                && analysis.stem.evidence.authority.as_deref() == Some("fixture-dictionary")
        }));
    }

    #[test]
    fn sigmatic_aorist_metadata_requires_the_independent_singular_subbundle() {
        let metadata = DictionaryVerbMetadata::from_normalized_fields(
            "fixture",
            "рєшти",
            [
                field("aorist", 0, "stem", "рѣ"),
                field("aorist", 0, "second-third-singular", "рєчє"),
                field("aorist", 0, "formation", "sigmatic-secondary"),
            ],
        )
        .expect("complete sigmatic principal parts");
        assert_eq!(metadata.aorist.len(), 1);
        assert_eq!(metadata.aorist[0].stem.value, "рѣ");
        assert_eq!(
            metadata.aorist[0]
                .second_third_singular
                .as_ref()
                .expect("sigmatic singular subbundle")
                .value,
            "рєчє"
        );
        assert_eq!(
            metadata.aorist[0].formation.value,
            AoristFormation::SigmaticSecondary
        );

        assert!(matches!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "рєшти",
                [
                    field("aorist", 0, "stem", "рѣ"),
                    field("aorist", 0, "formation", "sigmatic-secondary"),
                ],
            ),
            Err(InflectionError::MissingLexicalMetadata { needed })
                if needed == vec![MetadataField::AoristSecondThirdSingular]
        ));
        assert!(matches!(
            DictionaryVerbMetadata::from_normalized_fields(
                "fixture",
                "рещи",
                [
                    field("aorist", 0, "stem", "рек"),
                    field("aorist", 0, "second-third-singular", "рече"),
                    field("aorist", 0, "formation", "new"),
                ],
            ),
            Err(InflectionError::ContradictoryLexicalMetadata { .. })
        ));
    }

    #[test]
    fn contracted_imperfect_policy_is_typed_and_source_ordered() {
        let metadata = DictionaryVerbMetadata::from_normalized_fields(
            "fixture",
            "нести",
            [
                field("imperfect", 0, "stem", "нес"),
                field("imperfect", 0, "formation", "yat-a"),
                field("imperfect", 0, "variant-policy", "contracted-only"),
                field("imperfect", 1, "stem", "нес"),
                field("imperfect", 1, "formation", "yat-a"),
                field("imperfect", 1, "variant-policy", "uncontracted-only"),
            ],
        )
        .expect("two separately sourced imperfect analyses");
        assert_eq!(metadata.imperfect.len(), 2);
        assert_eq!(metadata.imperfect[0].analysis_rank, 0);
        assert_eq!(
            metadata.imperfect[0].variant_policy.value,
            ImperfectVariantPolicy::ContractedOnly
        );
        assert_eq!(metadata.imperfect[1].analysis_rank, 1);
        assert_eq!(
            metadata.imperfect[1].variant_policy.value,
            ImperfectVariantPolicy::UncontractedOnly
        );
    }

    #[test]
    fn unique_participle_seams_have_stable_metadata_codes() {
        assert_eq!(
            parse_present_active("mixed-yusht-soft").expect("mixed root seam"),
            PresentActiveParticipleFormation::MixedYushtSoft
        );
        assert_eq!(
            parse_present_active("iotated-yusht-soft").expect("iotated vowel seam"),
            PresentActiveParticipleFormation::IotatedYushtSoft
        );
        assert_eq!(
            parse_present_passive("iotated-em").expect("iotated passive seam"),
            PresentPassiveParticipleFormation::IotatedEm
        );
    }
}
