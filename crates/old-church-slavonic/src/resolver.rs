//! Canonical dictionary-backed cell resolvers.
//!
//! Dictionary table cells take precedence. When a known lexeme has sufficient
//! class metadata, missing cells may be produced by the pure core rules. Unknown
//! or ambiguous lexical facts are returned as typed errors.

use crate::{dictionary, lookup, metadata::*, paradigm::*};
use old_church_slavonic_core::adjective::{AdjectiveLexeme, ComparativeLexeme};
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::*;

fn cell_outcomes<C: Copy>(
    id: &str,
    cells: impl IntoIterator<Item = C>,
    mut resolve: impl FnMut(&str, C) -> Result<FormSet, InflectionError>,
) -> Vec<CellOutcome<C>> {
    cells
        .into_iter()
        .map(|cell| CellOutcome {
            cell,
            result: resolve(id, cell),
        })
        .collect()
}

fn resolve_queried_lemma(
    query: &str,
    part_of_speech: PartOfSpeech,
    resolve: impl FnOnce(&str) -> Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(query, part_of_speech)?;
    queried_result(query, record, resolve(record.id))
}

fn verb_metadata_form(
    id: &str,
    feature: &str,
    generate: impl FnOnce(&DictionaryVerbMetadata) -> Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, feature) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, feature) {
        return Ok(form);
    }
    generate(&metadata).map_err(|error| error.with_lexeme_id(id))
}

pub fn noun(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Noun, |id| noun_by_id(id, cell))
}

pub fn noun_by_id(id: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Noun)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Noun)))?;
    let lexeme = noun_lexeme(record)?;
    predicted_noun(&lexeme, cell, true).map_err(|error| error.with_lexeme_id(id))
}

pub fn noun_with(lexeme: &NounLexeme, cell: NounCell) -> Result<FormSet, InflectionError> {
    predicted_noun(lexeme, cell, false)
}

pub fn noun_paradigm_by_id(id: &str) -> Result<NounParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Noun)?;
    Ok(build_noun_paradigm(id, record.lemma))
}

pub(crate) fn build_noun_paradigm(id: &str, lemma: &str) -> NounParadigm {
    NounParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, NounCell::all(), noun_by_id),
    }
}

pub fn adjective(lemma: &str, cell: AdjectiveCell) -> Result<FormSet, InflectionError> {
    let candidates = lookup(lemma, PartOfSpeech::Adjective)?;
    match candidates.as_slice() {
        [] => {
            let normalized = orthography::lookup_key(lemma)?;
            let class = if normalized.ends_with('ъ') {
                AdjectiveClass::Hard
            } else if normalized.ends_with(['ь', 'и']) {
                AdjectiveClass::Soft
            } else {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::AdjectiveClass],
                });
            };
            let lexeme = AdjectiveLexeme {
                lemma: normalized,
                class,
            };
            let predicted = old_church_slavonic_core::adjective::decline(&lexeme, cell)?;
            Ok(predicted_set(&lexeme.lemma, predicted, FormSourceKind::Oov))
        }
        [one] => {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated lookup candidate is missing".to_string(),
                })?;
            queried_result(lemma, record, adjective_by_id(&one.id, cell))
        }
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub fn adjective_by_id(id: &str, cell: AdjectiveCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Adjective)))?;
    let class = parse_adjective_class(record.class).ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::AdjectiveClass],
        }
    })?;
    let lexeme = AdjectiveLexeme {
        lemma: record.lemma.to_string(),
        class,
    };
    let predicted = old_church_slavonic_core::adjective::decline(&lexeme, cell)
        .map_err(|error| error.with_lexeme_id(id))?;
    Ok(predicted_set(
        record.lemma,
        predicted,
        FormSourceKind::DictionaryMetadata,
    ))
}

pub fn adjective_paradigm_by_id(id: &str) -> Result<AdjectiveParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Adjective)?;
    Ok(build_adjective_paradigm(id, record.lemma))
}

pub(crate) fn build_adjective_paradigm(id: &str, lemma: &str) -> AdjectiveParadigm {
    AdjectiveParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, AdjectiveCell::all(), adjective_by_id),
    }
}

pub fn adjective_with(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::adjective::decline(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn comparative_with(
    lexeme: &ComparativeLexeme,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.positive_lemma,
        old_church_slavonic_core::adjective::decline_comparative(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn comparative_paradigm_with(lexeme: &ComparativeLexeme) -> ComparativeParadigm {
    ComparativeParadigm {
        lemma: lexeme.positive_lemma.clone(),
        syncopated_citation: lexeme.syncopated_citation.clone(),
        expanded_citation: lexeme.expanded_citation.clone(),
        cells: AdjectiveCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: comparative_with(lexeme, cell),
            })
            .collect(),
    }
}

pub fn pre_superlative_with(
    positive: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &positive.lemma,
        old_church_slavonic_core::adjective::decline_pre_superlative(positive, cell),
        FormSourceKind::Explicit,
    )
}

pub(crate) fn grammar_token(
    text: &str,
    rule_id: RuleId,
    reason: &'static str,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        text,
        Ok(PredictedForm {
            text: text.to_string(),
            rule_id,
            trace: vec![RuleStep {
                rule_id,
                before: text.to_string(),
                after: text.to_string(),
                reason,
            }],
        }),
        FormSourceKind::Explicit,
    )
}

/// Resolve one independently reviewed suppletive copular series without
/// conflating the OCS `ѥс-`, `бѫд-`, `бѣ-`, and `би-` systems.
pub fn copula(
    series: CopulaSeries,
    person: Person,
    number: Number,
) -> Result<FormSet, InflectionError> {
    let rule_id = series.rule_id();
    let variants = series
        .forms(person, number)
        .iter()
        .map(|variant| {
            let text = orthography::canonical_display(variant.text)?;
            let trace = if variant.status == CopulaVariantStatus::Reconstructed {
                vec![RuleStep {
                    rule_id,
                    before: format!("{}:{}", person.code(), number.code()),
                    after: text.clone(),
                    reason: "realize the explicitly reconstructed OCS copular cell",
                }]
            } else {
                Vec::new()
            };
            let form = FormVariant {
                text,
                romanization: None,
            };
            let source = match variant.status {
                CopulaVariantStatus::SourceBacked => FormSource::ReviewedGrammarTable { rule_id },
                CopulaVariantStatus::Reconstructed => FormSource::ExplicitMetadataRule { rule_id },
            };
            let analysis = FormAnalysis {
                variants: vec![form.clone()],
                source,
                evidence: vec![MetadataEvidence {
                    field: None,
                    provenance: match variant.status {
                        CopulaVariantStatus::SourceBacked => {
                            MetadataProvenance::ReviewedGrammarTable
                        }
                        CopulaVariantStatus::Reconstructed => {
                            MetadataProvenance::ProductiveRuleOutput
                        }
                    },
                    source_feature: Some(format!(
                        "copula:{series:?}:{}:{}",
                        person.code(),
                        number.code()
                    )),
                    source_form: (variant.status == CopulaVariantStatus::SourceBacked)
                        .then(|| form.text.clone()),
                    crosscheck_features: Vec::new(),
                    authority: Some(series.authority().to_string()),
                }],
                trace,
            };
            Ok((form, analysis, variant.status))
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let (primary, primary_analysis, _) =
        variants
            .first()
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: format!("the {series:?} copular cell has no reviewed forms"),
            })?;
    let warnings = variants
        .iter()
        .any(|(_, _, status)| *status == CopulaVariantStatus::Reconstructed)
        .then_some(InflectionWarning::IncludesReconstructedForms)
        .into_iter()
        .collect();
    Ok(FormSet::new(
        orthography::canonical_display(series.lemma())?,
        primary.clone(),
        variants
            .iter()
            .skip(1)
            .map(|(form, _, _)| form.clone())
            .collect(),
        primary_analysis.source.clone(),
        warnings,
        Vec::new(),
        variants
            .into_iter()
            .map(|(_, analysis, _)| analysis)
            .collect(),
    ))
}

/// Resolve the reviewed first- or second-person paradigm independently of the
/// duplicated personal-pronoun tables found on dictionary form pages.
pub fn personal_pronoun_with(
    identity: PersonalPronounIdentity,
    case: Case,
    number: Number,
    selection: PronounFormSelection,
) -> Result<FormSet, InflectionError> {
    let person = match identity {
        PersonalPronounIdentity::First => Person::First,
        PersonalPronounIdentity::Second => Person::Second,
        PersonalPronounIdentity::Reflexive | PersonalPronounIdentity::AnaphoricThird => {
            return Err(InflectionError::InvalidInput {
                reason:
                    "personal_pronoun_with requires the intrinsic first- or second-person identity"
                        .to_string(),
            });
        }
    };
    let cell = PersonalPronounCell {
        case,
        number,
        person,
    }
    .closed_class();
    let forms =
        old_church_slavonic_core::pronoun::personal_forms(identity, case, number, selection);
    reviewed_pronoun_set(
        identity,
        cell,
        forms,
        format!(
            "pronoun:{}:{}:{}:{}",
            person.code(),
            case.code(),
            number.code(),
            selection.code()
        ),
    )
}

/// Resolve the numberless reflexive pronoun. Number is intentionally absent
/// from this API because the same lexeme refers back to any subject number.
pub fn reflexive_pronoun(
    case: Case,
    selection: PronounFormSelection,
) -> Result<FormSet, InflectionError> {
    reviewed_pronoun_set(
        PersonalPronounIdentity::Reflexive,
        ClosedClassCell {
            case,
            number: Number::Singular,
            gender: None,
            person: None,
        },
        old_church_slavonic_core::pronoun::reflexive_forms(case, selection),
        format!("pronoun:reflexive:{}:{}", case.code(), selection.code()),
    )
}

/// Resolve the defective third-person anaphoric pronoun in either its free or
/// obligatorily prepositional `н҄-` realization.
pub fn anaphoric_pronoun(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> Result<FormSet, InflectionError> {
    let forms =
        old_church_slavonic_core::pronoun::anaphoric_form(case, number, gender, environment)
            .into_iter()
            .collect();
    reviewed_pronoun_set(
        PersonalPronounIdentity::AnaphoricThird,
        GenderedCell {
            case,
            number,
            gender,
        }
        .closed_class(),
        forms,
        format!(
            "pronoun:anaphoric-third:{}:{}:{}:{}",
            case.code(),
            number.code(),
            gender.code(),
            environment.code()
        ),
    )
}

/// Decline a regular pronominal lexeme from explicit caller-supplied class
/// metadata, independently of the bundled dictionary.
///
/// ```
/// use old_church_slavonic::{Case, Gender, Number};
/// use old_church_slavonic::advanced::rules::{
///     PronominalDeclension, PronominalLexeme, pronominal_with,
/// };
///
/// let lexeme = PronominalLexeme {
///     lemma: "такъ".to_string(),
///     declension: PronominalDeclension::Hard,
/// };
/// let form = pronominal_with(
///     &lexeme, Case::Nominative, Number::Plural, Gender::Masculine,
/// )?;
/// assert_eq!(form.primary_text(), "таци");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn pronominal_with(
    lexeme: &PronominalLexeme,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::pronoun::decline_pronominal(lexeme, case, number, gender),
        FormSourceKind::Explicit,
    )
}

fn standard_pronominal_pronoun(
    identity: StandardPronominalIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<FormSet, InflectionError> {
    let prediction = old_church_slavonic_core::pronoun::decline_pronominal(
        &identity.lexeme(),
        case,
        number,
        gender,
    )?;
    let rule_id = prediction.rule_id;
    let trace = prediction.trace;
    let primary = FormVariant {
        text: orthography::canonical_display(&prediction.text)?,
        romanization: None,
    };
    let source = FormSource::ReviewedGrammarTable { rule_id };
    let analysis = FormAnalysis {
        variants: vec![primary.clone()],
        source: source.clone(),
        evidence: vec![MetadataEvidence {
            field: None,
            provenance: MetadataProvenance::ReviewedGrammarTable,
            source_feature: Some(format!(
                "pronoun:2-p:{}:{}:{}:{}",
                identity.declension().code(),
                case.code(),
                number.code(),
                gender.code()
            )),
            // The authorities license the productive terminal combination;
            // do not mislabel the generated surface as a corpus attestation.
            source_form: None,
            crosscheck_features: Vec::new(),
            authority: Some(
                "Polivanova 2023 §§287–299, 314–318; LMU OCS Reference Grammar §2.2.3".to_string(),
            ),
        }],
        trace: trace.clone(),
    };
    Ok(FormSet::new(
        orthography::canonical_display(identity.canonical_lemma())?,
        primary,
        Vec::new(),
        source,
        Vec::new(),
        trace,
        vec![analysis],
    ))
}

/// Resolve the complete agreeing relative pronoun `иже`, including its
/// obligatorily conditioned post-prepositional `н҄-` allomorph.
pub fn relative_pronoun(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> Result<FormSet, InflectionError> {
    let rule_id = match environment {
        AnaphoricEnvironment::Free => RuleId::PronounRelativeIzhe,
        AnaphoricEnvironment::AfterPreposition => RuleId::PronounRelativePrepositional,
    };
    let variants =
        old_church_slavonic_core::pronoun::relative_izhe_form(case, number, gender, environment)
            .map(|text| vec![(text, PronounVariantStatus::TablePrimary)])
            .unwrap_or_default();
    reviewed_closed_set(
        "иже",
        PartOfSpeech::Pronoun,
        GenderedCell {
            case,
            number,
            gender,
        }
        .closed_class(),
        variants,
        rule_id,
        format!(
            "pronoun:relative-izhe:{}:{}:{}:{}",
            case.code(),
            number.code(),
            gender.code(),
            environment.code()
        ),
        "Polivanova 2023 §318; UT OCS Online lesson 2 §8.3",
    )
}

/// Resolve one complete closed irregular agreeing paradigm. The identity owns
/// its grammatical part of speech: `кꙑи` is a determiner; the other identities
/// are pronouns.
pub fn irregular_agreeing(
    identity: IrregularAgreeingIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<FormSet, InflectionError> {
    let part_of_speech = identity.part_of_speech();
    let variants =
        old_church_slavonic_core::pronoun::irregular_agreeing_forms(identity, case, number, gender)
            .into_iter()
            .map(|variant| (variant.text.to_string(), variant.status))
            .collect();
    let authority = match identity {
        IrregularAgreeingIdentity::TotalVes | IrregularAgreeingIdentity::DemonstrativeSic => {
            "Polivanova 2023 §§319–320"
        }
        IrregularAgreeingIdentity::ProximalSi => "Polivanova 2023 §§377–378",
        IrregularAgreeingIdentity::InterrogativeKyi => "Polivanova 2023 §§375–376",
    };
    reviewed_closed_set(
        identity.canonical_lemma(),
        part_of_speech,
        GenderedCell {
            case,
            number,
            gender,
        }
        .closed_class(),
        variants,
        identity.rule_id(),
        format!(
            "{}:irregular-agreeing:{}:{}:{}",
            part_of_speech.code(),
            case.code(),
            number.code(),
            gender.code()
        ),
        authority,
    )
}

/// Resolve one case of numberless, genderless `къто` or `чьто`, preserving
/// every grammar-table variant in source order.
pub fn interrogative_pronoun(
    identity: InterrogativePronounIdentity,
    case: Case,
) -> Result<FormSet, InflectionError> {
    let variants = old_church_slavonic_core::pronoun::interrogative_forms(identity, case)
        .into_iter()
        .map(|variant| (variant.text.to_string(), variant.status))
        .collect();
    reviewed_closed_set(
        identity.canonical_lemma(),
        PartOfSpeech::Pronoun,
        ClosedClassCell {
            case,
            // This structural placeholder is not a grammatical singular: the
            // API deliberately has no number argument for these identities.
            number: Number::Singular,
            gender: None,
            person: None,
        },
        variants,
        identity.rule_id(),
        format!("pronoun:interrogative:numberless:{}", case.code()),
        "Polivanova 2023 §§379–380",
    )
}

fn reviewed_closed_set(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
    variants: Vec<(String, PronounVariantStatus)>,
    rule_id: RuleId,
    source_feature: String,
    authority: &'static str,
) -> Result<FormSet, InflectionError> {
    if variants.is_empty() {
        return Err(InflectionError::historically_invalid(
            lemma,
            RequestedCell::ClosedClass {
                part_of_speech,
                cell,
            },
        ));
    }
    let analyses = variants
        .iter()
        .map(|(text, status)| {
            let source = FormSource::ReviewedGrammarTable { rule_id };
            let form = FormVariant {
                text: orthography::canonical_display(text)?,
                romanization: None,
            };
            Ok((
                form.clone(),
                FormAnalysis {
                    variants: vec![form],
                    source,
                    evidence: vec![MetadataEvidence {
                        field: None,
                        provenance: MetadataProvenance::ReviewedGrammarTable,
                        source_feature: Some(format!(
                            "{source_feature}:{}",
                            pronoun_status_code(*status)
                        )),
                        source_form: Some(text.clone()),
                        crosscheck_features: Vec::new(),
                        authority: Some(authority.to_string()),
                    }],
                    trace: Vec::new(),
                },
            ))
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let (primary, primary_analysis) =
        analyses
            .first()
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: "a reviewed closed-class cell unexpectedly had no forms".to_string(),
            })?;
    Ok(FormSet::new(
        orthography::canonical_display(lemma)?,
        primary.clone(),
        analyses
            .iter()
            .skip(1)
            .map(|(variant, _)| variant.clone())
            .collect(),
        primary_analysis.source.clone(),
        Vec::new(),
        Vec::new(),
        analyses.into_iter().map(|(_, analysis)| analysis).collect(),
    ))
}

fn reviewed_pronoun_set(
    identity: PersonalPronounIdentity,
    cell: ClosedClassCell,
    variants: Vec<PronounVariant>,
    source_feature: String,
) -> Result<FormSet, InflectionError> {
    if variants.is_empty() {
        return Err(InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::ClosedClass {
                part_of_speech: PartOfSpeech::Pronoun,
                cell,
            },
        ));
    }
    let analyses = variants
        .iter()
        .map(|variant| {
            let rule_id = pronoun_variant_rule(identity, variant.status);
            let source = FormSource::ReviewedGrammarTable { rule_id };
            let text = orthography::canonical_display(variant.text)?;
            let form = FormVariant {
                text,
                romanization: None,
            };
            Ok((
                form.clone(),
                FormAnalysis {
                    variants: vec![form],
                    source,
                    evidence: vec![MetadataEvidence {
                        field: None,
                        provenance: if variant.status.is_disputed() {
                            MetadataProvenance::DisputedGrammarTable
                        } else {
                            MetadataProvenance::ReviewedGrammarTable
                        },
                        source_feature: Some(format!(
                            "{source_feature}:{}",
                            pronoun_status_code(variant.status)
                        )),
                        source_form: Some(variant.text.to_string()),
                        crosscheck_features: Vec::new(),
                        authority: Some(pronoun_authority(variant.status).to_string()),
                    }],
                    trace: Vec::new(),
                },
            ))
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let (primary, primary_analysis) =
        analyses
            .first()
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: "a reviewed pronoun cell unexpectedly had no forms".to_string(),
            })?;
    Ok(FormSet::new(
        orthography::canonical_display(identity.canonical_lemma())?,
        primary.clone(),
        analyses
            .iter()
            .skip(1)
            .map(|(variant, _)| variant.clone())
            .collect(),
        primary_analysis.source.clone(),
        variants
            .iter()
            .any(|variant| variant.status.is_disputed())
            .then_some(InflectionWarning::IncludesDisputedForms)
            .into_iter()
            .collect(),
        Vec::new(),
        analyses.into_iter().map(|(_, analysis)| analysis).collect(),
    ))
}

fn pronoun_variant_rule(identity: PersonalPronounIdentity, status: PronounVariantStatus) -> RuleId {
    match status {
        PronounVariantStatus::MarkedClitic | PronounVariantStatus::DisputedMarkedClitic => {
            RuleId::PronounPersonalClitic
        }
        PronounVariantStatus::Adprepositional => RuleId::PronounAnaphoricPrepositional,
        PronounVariantStatus::TablePrimary
        | PronounVariantStatus::TableVariant
        | PronounVariantStatus::FreeAnaphoric => identity.rule_id(),
    }
}

fn pronoun_status_code(status: PronounVariantStatus) -> &'static str {
    match status {
        PronounVariantStatus::TablePrimary => "table-primary",
        PronounVariantStatus::TableVariant => "table-variant",
        PronounVariantStatus::MarkedClitic => "marked-clitic",
        PronounVariantStatus::DisputedMarkedClitic => "disputed-marked-clitic",
        PronounVariantStatus::FreeAnaphoric => "free-anaphoric",
        PronounVariantStatus::Adprepositional => "adprepositional",
    }
}

fn pronoun_authority(status: PronounVariantStatus) -> &'static str {
    match status {
        PronounVariantStatus::DisputedMarkedClitic => {
            "UT OCS Online lesson 2 §8.1 lists first-person DDu на; Polivanova 2023 §382.3 says no OCS clitic is attested and compares Church Slavonic на"
        }
        PronounVariantStatus::FreeAnaphoric | PronounVariantStatus::Adprepositional => {
            "Polivanova 2023 §318; UT OCS Online lesson 2 §8.3"
        }
        PronounVariantStatus::TablePrimary
        | PronounVariantStatus::TableVariant
        | PronounVariantStatus::MarkedClitic => {
            "Polivanova 2023 §§381–382; UT OCS Online lesson 2 §§8.1–8.2"
        }
    }
}

pub fn adjective_comparatives(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Adjective, comparative_citation_by_id)
}

pub fn comparative_citation_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    lookup::table_form(id, "adj:comparative:citation")
        .or_else(|| lookup::override_form(id, "adj:comparative:citation"))
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::ComparativeCitation))
}

pub fn finite_verb(lemma: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, |id| finite_by_id(id, cell))
}

pub fn finite_by_id(id: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    verb_metadata_form(id, &cell.key(), |metadata| {
        generate_finite_from_metadata(metadata, cell)
    })
}

/// Generate through the same dictionary-metadata resolver after an offline
/// caller has already constructed and validated a metadata view. This does not
/// consult the bundled dictionary table and is used for leakage-controlled
/// held-cell evaluation.
pub fn finite_verb_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    generate_finite_from_metadata(metadata, cell)
}

pub fn finite_paradigm_by_id(id: &str) -> Result<FiniteVerbParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok(build_finite_paradigm(id, record.lemma))
}

pub(crate) fn build_finite_paradigm(id: &str, lemma: &str) -> FiniteVerbParadigm {
    FiniteVerbParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, FiniteVerbCell::all(), finite_by_id),
    }
}

pub(crate) fn build_present_paradigm(id: &str, lemma: &str) -> VerbParadigm {
    VerbParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(
            id,
            FiniteVerbCell::for_tense(FiniteTense::Present),
            finite_by_id,
        ),
    }
}

pub fn present_paradigm_by_id(id: &str) -> Result<VerbParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok(build_present_paradigm(id, record.lemma))
}

pub fn finite_verb_with(
    lexeme: &VerbLexeme,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::finite(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn imperative(lemma: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, |id| imperative_by_id(id, cell))
}

pub fn imperative_by_id(id: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    verb_metadata_form(id, &cell.key(), |metadata| {
        generate_imperative_from_metadata(metadata, cell)
    })
}

pub fn imperative_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    generate_imperative_from_metadata(metadata, cell)
}

pub fn imperative_with(
    lexeme: &VerbLexeme,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::imperative(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn imperative_paradigm_by_id(id: &str) -> Result<ImperativeParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok(build_imperative_paradigm(id, record.lemma))
}

pub(crate) fn build_imperative_paradigm(id: &str, lemma: &str) -> ImperativeParadigm {
    ImperativeParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, ImperativeCell::SUPPORTED, imperative_by_id),
    }
}

pub fn l_participle(lemma: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, |id| l_participle_by_id(id, cell))
}

pub fn l_participle_by_id(id: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    verb_metadata_form(id, &cell.key(), |metadata| {
        generate_l_participle_from_metadata(metadata, cell)
    })
}

pub fn l_participle_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    generate_l_participle_from_metadata(metadata, cell)
}

pub fn l_participle_with(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::l_participle(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn l_participle_paradigm_by_id(id: &str) -> Result<LParticipleParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok(build_l_participle_paradigm(id, record.lemma))
}

pub(crate) fn build_l_participle_paradigm(id: &str, lemma: &str) -> LParticipleParadigm {
    LParticipleParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, LParticipleCell::all(), l_participle_by_id),
    }
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, |id| participle_by_id(id, cell))
}

pub fn participle_by_id(id: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    verb_metadata_form(id, &cell.key(), |metadata| {
        generate_participle_from_metadata(metadata, cell)
    })
}

pub fn participle_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    generate_participle_from_metadata(metadata, cell)
}

pub fn participle_with(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::participle(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn participle_paradigm_by_id(
    id: &str,
    kind: ParticipleKind,
) -> Result<ParticipleParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok(build_participle_paradigm(id, record.lemma, kind))
}

pub(crate) fn build_participle_paradigm(
    id: &str,
    lemma: &str,
    kind: ParticipleKind,
) -> ParticipleParadigm {
    ParticipleParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        kind,
        cells: cell_outcomes(id, ParticipleCell::for_kind(kind), participle_by_id),
    }
}

pub fn participle_citation(lemma: &str, kind: ParticipleKind) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, |id| {
        participle_citation_by_id(id, kind)
    })
}

pub fn participle_citation_by_id(
    id: &str,
    kind: ParticipleKind,
) -> Result<FormSet, InflectionError> {
    let feature = format!("verb:participle:{}:citation", kind.code());
    verb_metadata_form(id, &feature, |metadata| {
        generate_participle_from_metadata(
            metadata,
            ParticipleCell {
                kind,
                adjective: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                },
            },
        )
    })
}

pub fn infinitive(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, infinitive_by_id)
}

pub fn infinitive_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:infinitive")
        .or_else(|| lookup::override_form(id, "verb:infinitive"))
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::Infinitive))
}

pub fn infinitive_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::infinitive(lexeme),
        FormSourceKind::Explicit,
    )
}

pub fn supine(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, supine_by_id)
}

pub fn supine_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:supine")
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::Supine))
}

pub fn supine_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::supine(lexeme),
        FormSourceKind::Explicit,
    )
}

pub fn verbal_noun(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_lemma(lemma, PartOfSpeech::Verb, verbal_noun_by_id)
}

pub fn verbal_noun_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:verbal-noun")
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::VerbalNoun))
}

pub fn closed_class(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "closed_class accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    resolve_queried_lemma(lemma, part_of_speech, |id| {
        closed_class_by_id(id, part_of_speech, cell)
    })
}

pub fn closed_class_by_id(
    id: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "closed_class_by_id accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    ensure_pos(id, part_of_speech)?;
    if part_of_speech == PartOfSpeech::Pronoun {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Pronoun)))?;
        if let Some(identity) = PersonalPronounIdentity::classify_source_union_lemma(record.lemma) {
            let mut result = match identity {
                PersonalPronounIdentity::First | PersonalPronounIdentity::Second => {
                    match (cell.person, cell.gender, identity.person()) {
                        (Some(requested), None, Some(intrinsic)) if requested == intrinsic => {
                            personal_pronoun_with(
                                identity,
                                cell.case,
                                cell.number,
                                PronounFormSelection::All,
                            )
                        }
                        _ => Err(InflectionError::historically_invalid(
                            id,
                            RequestedCell::ClosedClass {
                                part_of_speech,
                                cell,
                            },
                        )),
                    }
                }
                PersonalPronounIdentity::Reflexive => {
                    if cell.person.is_none() && cell.gender.is_none() {
                        reflexive_pronoun(cell.case, PronounFormSelection::All)
                    } else {
                        Err(InflectionError::historically_invalid(
                            id,
                            RequestedCell::ClosedClass {
                                part_of_speech,
                                cell,
                            },
                        ))
                    }
                }
                PersonalPronounIdentity::AnaphoricThird => match (cell.person, cell.gender) {
                    (None, Some(gender)) => anaphoric_pronoun(
                        cell.case,
                        cell.number,
                        gender,
                        AnaphoricEnvironment::Free,
                    ),
                    _ => Err(InflectionError::historically_invalid(
                        id,
                        RequestedCell::ClosedClass {
                            part_of_speech,
                            cell,
                        },
                    )),
                },
            }?;
            if record.lemma != identity.canonical_lemma() {
                result.add_warning(InflectionWarning::LexicalAliasUsed {
                    canonical: identity.canonical_lemma().to_string(),
                });
            }
            return Ok(result);
        }
        if let Some(identity) =
            StandardPronominalIdentity::classify_source_union_lemma(record.lemma)
        {
            let gender = match (cell.person, cell.gender) {
                (None, Some(gender)) => gender,
                _ => {
                    return Err(InflectionError::historically_invalid(
                        id,
                        RequestedCell::ClosedClass {
                            part_of_speech,
                            cell,
                        },
                    ));
                }
            };
            let mut result = standard_pronominal_pronoun(identity, cell.case, cell.number, gender)
                .map_err(|error| error.with_lexeme_id(id))?;
            if record.lemma != identity.canonical_lemma() {
                result.add_warning(InflectionWarning::LexicalAliasUsed {
                    canonical: identity.canonical_lemma().to_string(),
                });
            }
            return Ok(result);
        }
        let reviewed = match record.lemma {
            "иже" => match (cell.person, cell.gender) {
                (None, Some(gender)) => {
                    relative_pronoun(cell.case, cell.number, gender, AnaphoricEnvironment::Free)
                }
                _ => Err(InflectionError::historically_invalid(
                    id,
                    RequestedCell::ClosedClass {
                        part_of_speech,
                        cell,
                    },
                )),
            },
            "сь" => match (cell.person, cell.gender) {
                (None, Some(gender)) => irregular_agreeing(
                    IrregularAgreeingIdentity::ProximalSi,
                    cell.case,
                    cell.number,
                    gender,
                ),
                _ => Err(InflectionError::historically_invalid(
                    id,
                    RequestedCell::ClosedClass {
                        part_of_speech,
                        cell,
                    },
                )),
            },
            "къто" | "чьто" => match (cell.person, cell.gender) {
                (None, None) => interrogative_pronoun(
                    if record.lemma == "къто" {
                        InterrogativePronounIdentity::Kto
                    } else {
                        InterrogativePronounIdentity::Chto
                    },
                    cell.case,
                ),
                _ => Err(InflectionError::historically_invalid(
                    id,
                    RequestedCell::ClosedClass {
                        part_of_speech,
                        cell,
                    },
                )),
            },
            _ => {
                return lookup::table_form(id, &cell.key(part_of_speech)).ok_or_else(|| {
                    InflectionError::unsupported(
                        id,
                        RequestedCell::ClosedClass {
                            part_of_speech,
                            cell,
                        },
                    )
                });
            }
        };
        return reviewed.map_err(|error| error.with_lexeme_id(id));
    }
    if part_of_speech == PartOfSpeech::Determiner {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Determiner)))?;
        if record.lemma == "кꙑи" {
            return match (cell.person, cell.gender) {
                (None, Some(gender)) => irregular_agreeing(
                    IrregularAgreeingIdentity::InterrogativeKyi,
                    cell.case,
                    cell.number,
                    gender,
                )
                .map_err(|error| error.with_lexeme_id(id)),
                _ => Err(InflectionError::historically_invalid(
                    id,
                    RequestedCell::ClosedClass {
                        part_of_speech,
                        cell,
                    },
                )),
            };
        }
    }
    lookup::table_form(id, &cell.key(part_of_speech)).ok_or_else(|| {
        InflectionError::unsupported(
            id,
            RequestedCell::ClosedClass {
                part_of_speech,
                cell,
            },
        )
    })
}

/// Resolve exactly one normalized source-table closed-class cell. This
/// diagnostic/raw path intentionally preserves duplicated source tables instead
/// of applying reviewed lexical ownership.
pub fn raw_closed_class(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "raw closed-class access accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    resolve_queried_lemma(lemma, part_of_speech, |id| {
        raw_closed_class_by_id(id, part_of_speech, cell)
    })
}

/// Resolve one normalized source-table cell by stable dictionary identity,
/// without applying grammar-table aliases or productive behavior.
pub fn raw_closed_class_by_id(
    id: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "raw closed-class access accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    ensure_pos(id, part_of_speech)?;
    lookup::table_form(id, &cell.key(part_of_speech)).ok_or_else(|| {
        InflectionError::unsupported(
            id,
            RequestedCell::ClosedClass {
                part_of_speech,
                cell,
            },
        )
    })
}

pub fn determiner_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Determiner, cell.closed_class())
}

pub fn pronoun_by_id(id: &str, cell: UngenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn personal_pronoun_by_id(
    id: &str,
    cell: PersonalPronounCell,
) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn gendered_pronoun_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn numeral_by_id(id: &str, cell: UngenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Numeral, cell.closed_class())
}

pub fn gendered_numeral_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Numeral, cell.closed_class())
}

fn lexeme_identity(
    id: &str,
    part_of_speech: PartOfSpeech,
) -> Result<&'static dictionary::LexemeRecord, InflectionError> {
    ensure_pos(id, part_of_speech)?;
    lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, Some(part_of_speech)))
}

pub fn determiner_paradigm_by_id(id: &str) -> Result<DeterminerParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Determiner)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Determiner,
    ))
}

pub fn pronoun_paradigm_by_id(id: &str) -> Result<PronounParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_ungendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Pronoun,
    ))
}

pub fn personal_pronoun_paradigm_by_id(
    id: &str,
) -> Result<PersonalPronounParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_personal_pronoun_paradigm(id, record.lemma))
}

pub fn gendered_pronoun_paradigm_by_id(
    id: &str,
) -> Result<GenderedPronounParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Pronoun,
    ))
}

pub fn numeral_paradigm_by_id(id: &str) -> Result<NumeralParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Numeral)?;
    Ok(build_ungendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Numeral,
    ))
}

pub fn gendered_numeral_paradigm_by_id(
    id: &str,
) -> Result<GenderedNumeralParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Numeral)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Numeral,
    ))
}

pub(crate) fn build_ungendered_closed_class_paradigm(
    id: &str,
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> ClosedClassParadigm<UngenderedCell> {
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech,
        cells: cell_outcomes(id, UngenderedCell::all(), |id, cell| {
            closed_class_by_id(id, part_of_speech, cell.closed_class())
        }),
    }
}

pub(crate) fn build_gendered_closed_class_paradigm(
    id: &str,
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> ClosedClassParadigm<GenderedCell> {
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech,
        cells: cell_outcomes(id, GenderedCell::all(), |id, cell| {
            closed_class_by_id(id, part_of_speech, cell.closed_class())
        }),
    }
}

pub(crate) fn build_personal_pronoun_paradigm(id: &str, lemma: &str) -> PersonalPronounParadigm {
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: PartOfSpeech::Pronoun,
        cells: cell_outcomes(id, PersonalPronounCell::all(), personal_pronoun_by_id),
    }
}

pub fn dictionary_paradigm_by_id(id: &str) -> Result<DictionaryParadigm, InflectionError> {
    let record = lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    let part_of_speech =
        lookup::parse_pos(record.pos).ok_or_else(|| InflectionError::InvalidInput {
            reason: "generated lexeme has an invalid part of speech".to_string(),
        })?;
    let cells = lookup::table_paradigm(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    Ok(DictionaryParadigm {
        lexeme_id: id.to_string(),
        lemma: record.lemma.to_string(),
        part_of_speech,
        cells,
    })
}

pub fn dictionary_form_by_id(id: &str, feature: &str) -> Result<FormSet, InflectionError> {
    lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    lookup::table_form(id, feature)
        .or_else(|| lookup::override_form(id, feature))
        .ok_or_else(|| {
            InflectionError::unsupported(
                id,
                RequestedCell::RawFeature {
                    feature: feature.to_string(),
                },
            )
        })
}

/// Resolve an accepted normalized verb feature through the same table-first
/// public APIs as the typed entry points. Non-verb exact table/override keys
/// remain available, but productive normalized-key dispatch is intentionally
/// verb-only. Ordinary callers should prefer the typed cell APIs.
pub fn form_by_id(id: &str, feature: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    if let Some(form) = lookup::table_form(id, feature) {
        return Ok(form);
    }
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] if record.pos == "verb" => finite_by_id(
            id,
            FiniteVerbCell {
                tense: parse_feature_tense(tense)?,
                person: parse_feature_person(person)?,
                number: parse_feature_number(number)?,
            },
        ),
        ["verb", "imperative", person, number] if record.pos == "verb" => imperative_by_id(
            id,
            ImperativeCell {
                person: parse_feature_person(person)?,
                number: parse_feature_number(number)?,
            },
        ),
        ["verb", "l-participle", gender, number] if record.pos == "verb" => l_participle_by_id(
            id,
            LParticipleCell {
                gender: parse_feature_gender(gender)?,
                number: parse_feature_number(number)?,
            },
        ),
        [
            "verb",
            "participle",
            kind,
            "adj",
            form,
            case,
            number,
            gender,
            animacy,
        ] if record.pos == "verb" => participle_by_id(
            id,
            ParticipleCell {
                kind: parse_feature_participle_kind(kind)?,
                adjective: AdjectiveCell {
                    case: parse_feature_case(case)?,
                    number: parse_feature_number(number)?,
                    gender: parse_feature_gender(gender)?,
                    animacy: parse_feature_animacy(animacy)?,
                    form: parse_feature_adjective_form(form)?,
                },
            },
        ),
        ["verb", "participle", kind, "citation"] if record.pos == "verb" => {
            participle_citation_by_id(id, parse_feature_participle_kind(kind)?)
        }
        ["verb", "infinitive"] if record.pos == "verb" => infinitive_by_id(id),
        ["verb", "supine"] if record.pos == "verb" => supine_by_id(id),
        ["verb", "verbal-noun"] if record.pos == "verb" => verbal_noun_by_id(id),
        _ => lookup::override_form(id, feature).ok_or_else(|| {
            InflectionError::unsupported(
                id,
                RequestedCell::RawFeature {
                    feature: feature.to_string(),
                },
            )
        }),
    }
}

fn invalid_feature(segment: &str) -> InflectionError {
    InflectionError::InvalidInput {
        reason: format!("invalid normalized feature segment: {segment}"),
    }
}

fn parse_feature_tense(value: &str) -> Result<FiniteTense, InflectionError> {
    match value {
        "present" => Ok(FiniteTense::Present),
        "imperfect" => Ok(FiniteTense::Imperfect),
        "aorist" => Ok(FiniteTense::Aorist),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_person(value: &str) -> Result<Person, InflectionError> {
    match value {
        "1" => Ok(Person::First),
        "2" => Ok(Person::Second),
        "3" => Ok(Person::Third),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_number(value: &str) -> Result<Number, InflectionError> {
    match value {
        "sg" => Ok(Number::Singular),
        "du" => Ok(Number::Dual),
        "pl" => Ok(Number::Plural),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_gender(value: &str) -> Result<Gender, InflectionError> {
    match value {
        "m" => Ok(Gender::Masculine),
        "f" => Ok(Gender::Feminine),
        "n" => Ok(Gender::Neuter),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_case(value: &str) -> Result<Case, InflectionError> {
    match value {
        "nom" => Ok(Case::Nominative),
        "gen" => Ok(Case::Genitive),
        "dat" => Ok(Case::Dative),
        "acc" => Ok(Case::Accusative),
        "ins" => Ok(Case::Instrumental),
        "loc" => Ok(Case::Locative),
        "voc" => Ok(Case::Vocative),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_animacy(value: &str) -> Result<Animacy, InflectionError> {
    match value {
        "an" => Ok(Animacy::Animate),
        "in" => Ok(Animacy::Inanimate),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_adjective_form(value: &str) -> Result<AdjectiveForm, InflectionError> {
    match value {
        "short" => Ok(AdjectiveForm::Short),
        "long" => Ok(AdjectiveForm::Long),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_participle_kind(value: &str) -> Result<ParticipleKind, InflectionError> {
    match value {
        "present-active" => Ok(ParticipleKind::PresentActive),
        "present-passive" => Ok(ParticipleKind::PresentPassive),
        "past-active" => Ok(ParticipleKind::PastActive),
        "past-passive" => Ok(ParticipleKind::PastPassive),
        _ => Err(invalid_feature(value)),
    }
}

#[derive(Clone)]
struct UsedMetadata {
    value: String,
    evidence: MetadataEvidence,
}

fn generate_finite_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    let mut analyses = Vec::new();
    match cell.tense {
        FiniteTense::Present => {
            if metadata.present.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::VerbClass, MetadataField::PresentStem],
                });
            }
            for analysis in &metadata.present {
                let mut lexeme = VerbLexeme::new(&metadata.lemma, analysis.class.value);
                lexeme.aspect = metadata.aspect.as_ref().map(|aspect| aspect.value);
                lexeme.stems.present = Some(analysis.stem.value.clone());
                lexeme.stems.present_first_singular = analysis
                    .first_singular_stem
                    .as_ref()
                    .map(|stem| stem.value.clone());
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                let mut selected = vec![used(&analysis.class), used(&analysis.stem)];
                if cell.person == Person::First && cell.number == Number::Singular {
                    if let Some(first) = &analysis.first_singular_stem {
                        selected.push(used(first));
                    }
                }
                analyses.push(metadata_analysis(predicted, selected));
            }
        }
        FiniteTense::Imperfect => {
            if metadata.imperfect.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![
                        MetadataField::ImperfectStem,
                        MetadataField::ImperfectFormation,
                    ],
                });
            }
            for analysis in &metadata.imperfect {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.imperfect = Some(analysis.stem.value.clone());
                lexeme.formations.imperfect = Some(analysis.formation.value);
                lexeme.formations.imperfect_variant_policy = Some(analysis.variant_policy.value);
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                analyses.push(metadata_analysis(
                    predicted,
                    vec![
                        used(&analysis.stem),
                        used(&analysis.formation),
                        used(&analysis.variant_policy),
                    ],
                ));
            }
        }
        FiniteTense::Aorist => {
            if metadata.aorist.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::AoristStem, MetadataField::AoristFormation],
                });
            }
            for analysis in &metadata.aorist {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.aorist = Some(analysis.stem.value.clone());
                lexeme.stems.aorist_second_third_singular = analysis
                    .second_third_singular
                    .as_ref()
                    .map(|principal_part| principal_part.value.clone());
                lexeme.formations.aorist = Some(analysis.formation.value);
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                let mut selected = vec![used(&analysis.stem), used(&analysis.formation)];
                if matches!(
                    (cell.person, cell.number),
                    (Person::Second | Person::Third, Number::Singular)
                ) {
                    if let Some(principal_part) = &analysis.second_third_singular {
                        selected.push(used(principal_part));
                    }
                }
                analyses.push(metadata_analysis(predicted, selected));
            }
        }
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_imperative_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    if !cell.is_supported() {
        return Err(InflectionError::historically_invalid(
            &metadata.lexeme_id,
            RequestedCell::Imperative(cell),
        ));
    }
    if metadata.imperative.is_empty() {
        return Err(InflectionError::MissingLexicalMetadata {
            needed: vec![
                MetadataField::ImperativeStem,
                MetadataField::ImperativeFormation,
            ],
        });
    }
    let mut analyses = Vec::new();
    for analysis in &metadata.imperative {
        let mut lexeme = metadata_verb(metadata);
        lexeme.stems.imperative = Some(analysis.stem.value.clone());
        lexeme.formations.imperative = Some(analysis.formation.value);
        let predicted = old_church_slavonic_core::verb::imperative(&lexeme, cell)?;
        analyses.push(metadata_analysis(
            predicted,
            vec![used(&analysis.stem), used(&analysis.formation)],
        ));
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_l_participle_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    if metadata.l_participle.is_empty() {
        return Err(InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::LParticipleStem],
        });
    }
    let mut analyses = Vec::new();
    for analysis in &metadata.l_participle {
        let mut lexeme = metadata_verb(metadata);
        lexeme.stems.aorist = Some(analysis.stem.value.clone());
        let predicted = old_church_slavonic_core::verb::l_participle(&lexeme, cell)?;
        analyses.push(metadata_analysis(predicted, vec![used(&analysis.stem)]));
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_participle_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    let mut analyses = Vec::new();
    match cell.kind {
        ParticipleKind::PresentActive => {
            if metadata.present_active_participle.is_empty() {
                return missing_participle(
                    MetadataField::PresentActiveParticipleStem,
                    MetadataField::PresentActiveParticipleFormation,
                );
            }
            for analysis in &metadata.present_active_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.present_active_participle = Some(analysis.stem.value.clone());
                lexeme.formations.present_active_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PresentPassive => {
            if metadata.present_passive_participle.is_empty() {
                return missing_participle(
                    MetadataField::PresentPassiveParticipleStem,
                    MetadataField::PresentPassiveParticipleFormation,
                );
            }
            for analysis in &metadata.present_passive_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.present_passive_participle = Some(analysis.stem.value.clone());
                lexeme.formations.present_passive_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PastActive => {
            if metadata.past_active_participle.is_empty() {
                return missing_participle(
                    MetadataField::PastActiveParticipleStem,
                    MetadataField::PastActiveParticipleFormation,
                );
            }
            for analysis in &metadata.past_active_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.past_active_participle = Some(analysis.stem.value.clone());
                lexeme.formations.past_active_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PastPassive => {
            if metadata.past_passive_participle.is_empty() {
                return missing_participle(
                    MetadataField::PastPassiveParticipleStem,
                    MetadataField::PastPassiveParticipleFormation,
                );
            }
            for analysis in &metadata.past_passive_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.past_passive_participle = Some(analysis.stem.value.clone());
                lexeme.formations.past_passive_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn missing_participle<T>(
    stem: MetadataField,
    formation: MetadataField,
) -> Result<T, InflectionError> {
    Err(InflectionError::MissingLexicalMetadata {
        needed: vec![stem, formation],
    })
}

fn metadata_verb(metadata: &DictionaryVerbMetadata) -> VerbLexeme {
    let class = metadata
        .present
        .first()
        .map_or(VerbClass::Irregular, |present| present.class.value);
    let mut lexeme = VerbLexeme::new(&metadata.lemma, class);
    lexeme.aspect = metadata.aspect.as_ref().map(|aspect| aspect.value);
    lexeme
}

trait TraceMetadataValue {
    fn trace_value(&self) -> String;
}

impl TraceMetadataValue for String {
    fn trace_value(&self) -> String {
        self.clone()
    }
}

macro_rules! trace_debug_value {
    ($($type:ty),+ $(,)?) => {
        $(impl TraceMetadataValue for $type {
            fn trace_value(&self) -> String {
                format!("{self:?}")
            }
        })+
    };
}

trace_debug_value!(
    VerbClass,
    ImperfectFormation,
    ImperfectVariantPolicy,
    AoristFormation,
    ImperativeFormation,
    PresentActiveParticipleFormation,
    PresentPassiveParticipleFormation,
    PastActiveParticipleFormation,
    PastPassiveParticipleFormation,
);

fn used<T: TraceMetadataValue>(metadata: &SourcedMetadata<T>) -> UsedMetadata {
    UsedMetadata {
        value: metadata.value.trace_value(),
        evidence: metadata.evidence.clone(),
    }
}

fn metadata_analysis(predicted: PredictedForm, used: Vec<UsedMetadata>) -> FormAnalysis {
    let source = FormSource::DictionaryMetadataRule {
        rule_id: predicted.rule_id,
    };
    let mut trace = used
        .iter()
        .map(|metadata| RuleStep {
            rule_id: RuleId::VerbDictionaryMetadata,
            before: metadata.evidence.source_form.clone().unwrap_or_default(),
            after: metadata.value.clone(),
            reason: "select a validated dictionary principal-part field",
        })
        .collect::<Vec<_>>();
    trace.extend(predicted.trace);
    let mut evidence = used
        .into_iter()
        .map(|metadata| metadata.evidence)
        .collect::<Vec<_>>();
    evidence.push(MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::ProductiveRuleOutput,
        source_feature: Some(predicted.rule_id.code().to_string()),
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some("docs/MORPHOLOGY_SPEC.md".to_string()),
    });
    FormAnalysis {
        variants: vec![FormVariant {
            text: predicted.text,
            romanization: None,
        }],
        source,
        evidence,
        trace,
    }
}

fn metadata_form_set(lemma: &str, analyses: Vec<FormAnalysis>) -> Result<FormSet, InflectionError> {
    let mut variants = Vec::new();
    for analysis in &analyses {
        for variant in &analysis.variants {
            if !variants.contains(variant) {
                variants.push(variant.clone());
            }
        }
    }
    let multiple = analyses.len() > 1;
    let source = if multiple {
        FormSource::DictionaryMetadataAnalyses
    } else {
        analyses
            .first()
            .map_or(FormSource::DictionaryMetadataAnalyses, |analysis| {
                analysis.source.clone()
            })
    };
    let trace = if multiple {
        Vec::new()
    } else {
        analyses
            .first()
            .map_or_else(Vec::new, |analysis| analysis.trace.clone())
    };
    let mut warnings = vec![InflectionWarning::PredictedNotDictionaryBacked];
    if multiple {
        warnings.push(InflectionWarning::MultipleMorphologicalAnalyses);
    }
    if variants.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "metadata generation produced no form analysis".to_string(),
        });
    }
    let primary = variants.remove(0);
    Ok(FormSet::new(
        lemma, primary, variants, source, warnings, trace, analyses,
    ))
}

fn ensure_pos(id: &str, expected: PartOfSpeech) -> Result<(), InflectionError> {
    let record =
        lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, Some(expected)))?;
    if record.pos == expected.code() {
        Ok(())
    } else {
        Err(InflectionError::InvalidInput {
            reason: format!("lexeme {id} is {}, not {expected}", record.pos),
        })
    }
}

fn add_alias_warning(
    query: &str,
    record: &dictionary::LexemeRecord,
    result: &mut FormSet,
) -> Result<(), InflectionError> {
    if orthography::lookup_key(query)? != record.key {
        result.add_warning(InflectionWarning::OrthographicAliasUsed {
            canonical: record.lemma.to_string(),
        });
    }
    Ok(())
}

fn queried_result(
    query: &str,
    record: &dictionary::LexemeRecord,
    result: Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    let mut result = result?;
    add_alias_warning(query, record, &mut result)?;
    Ok(result)
}

fn noun_lexeme(record: &dictionary::LexemeRecord) -> Result<NounLexeme, InflectionError> {
    let class =
        parse_noun_class(record.class).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::NounClass],
        })?;
    let gender =
        parse_gender(record.gender).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::Gender],
        })?;
    let animacy =
        parse_animacy(record.animacy).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::Animacy],
        })?;
    Ok(NounLexeme {
        lemma: record.lemma.to_string(),
        class,
        gender,
        animacy,
        number_restriction: parse_restriction(record.number_restriction),
    })
}

fn predicted_noun(
    lexeme: &NounLexeme,
    cell: NounCell,
    dictionary_metadata: bool,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::noun::decline(lexeme, cell),
        if dictionary_metadata {
            FormSourceKind::DictionaryMetadata
        } else {
            FormSourceKind::Explicit
        },
    )
}

#[derive(Clone, Copy)]
enum FormSourceKind {
    DictionaryMetadata,
    Explicit,
    Oov,
}

fn canonical_prediction(
    lemma: &str,
    predicted: Result<PredictedForm, InflectionError>,
    kind: FormSourceKind,
) -> Result<FormSet, InflectionError> {
    let predicted = predicted?;
    let lemma = orthography::canonical_display(lemma)?;
    Ok(predicted_set(&lemma, predicted, kind))
}

fn predicted_set(lemma: &str, predicted: PredictedForm, kind: FormSourceKind) -> FormSet {
    let trace = predicted.trace;
    let source = match kind {
        FormSourceKind::DictionaryMetadata => FormSource::DictionaryMetadataRule {
            rule_id: predicted.rule_id,
        },
        FormSourceKind::Explicit => FormSource::ExplicitMetadataRule {
            rule_id: predicted.rule_id,
        },
        FormSourceKind::Oov => FormSource::OovPrediction {
            rule_id: predicted.rule_id,
        },
    };
    let primary = FormVariant {
        text: predicted.text,
        romanization: None,
    };
    let evidence = vec![MetadataEvidence {
        field: None,
        provenance: match kind {
            FormSourceKind::DictionaryMetadata => MetadataProvenance::DictionaryPrincipalPart,
            FormSourceKind::Explicit => MetadataProvenance::ExplicitCallerMetadata,
            FormSourceKind::Oov => MetadataProvenance::ProductiveRuleOutput,
        },
        source_feature: None,
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: None,
    }];
    let analyses = vec![FormAnalysis {
        variants: vec![primary.clone()],
        source: source.clone(),
        evidence,
        trace: trace.clone(),
    }];
    FormSet::new(
        lemma,
        primary,
        Vec::new(),
        source,
        vec![InflectionWarning::PredictedNotDictionaryBacked],
        trace,
        analyses,
    )
}

fn parse_noun_class(value: &str) -> Option<NounClass> {
    match value {
        "o-m-hard" | "o-stem:m" => Some(NounClass::OMasculineHard),
        "o-n-hard" | "o-stem:n" => Some(NounClass::ONeuterHard),
        "jo-m-soft" => Some(NounClass::JoMasculineSoft),
        "jo-n-soft" => Some(NounClass::JoNeuterSoft),
        "a-hard" | "a-stem:f" => Some(NounClass::AHard),
        "ja-soft" => Some(NounClass::JaSoft),
        "i-f" => Some(NounClass::IFeminine),
        "i-m" => Some(NounClass::IMasculine),
        "u-m" => Some(NounClass::UMasculine),
        "n-m" => Some(NounClass::NMasculine),
        "n-n" => Some(NounClass::NNeuter),
        "nt-n" => Some(NounClass::NtNeuter),
        "r-n" => Some(NounClass::RStem),
        "s-n" => Some(NounClass::SNeuter),
        "v-f" => Some(NounClass::VFeminine),
        "indeclinable" => Some(NounClass::Indeclinable),
        _ => None,
    }
}

fn parse_adjective_class(value: &str) -> Option<AdjectiveClass> {
    match value {
        "adj-hard" => Some(AdjectiveClass::Hard),
        "adj-soft" => Some(AdjectiveClass::Soft),
        _ => None,
    }
}

fn parse_gender(value: &str) -> Option<Gender> {
    match value {
        "m" => Some(Gender::Masculine),
        "f" => Some(Gender::Feminine),
        "n" => Some(Gender::Neuter),
        _ => None,
    }
}

fn parse_animacy(value: &str) -> Option<Animacy> {
    match value {
        "an" => Some(Animacy::Animate),
        "in" => Some(Animacy::Inanimate),
        _ => None,
    }
}

fn parse_restriction(value: &str) -> NumberRestriction {
    match value {
        "sg" => NumberRestriction::SingularOnly,
        "du" => NumberRestriction::DualOnly,
        "pl" => NumberRestriction::PluralOnly,
        _ => NumberRestriction::All,
    }
}
