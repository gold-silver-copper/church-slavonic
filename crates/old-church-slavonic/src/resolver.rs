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

const REVIEWED_VERB_ID_PREFIX: &str = "reviewed:ocs:verb:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewedVerbProfile {
    Unique(UniqueVerbFamilyMember),
    Irregular(IrregularVerbFamilyMember),
}

impl ReviewedVerbProfile {
    fn classify(lemma: &str) -> Option<Self> {
        if let Some(member) = UniqueVerbFamilyMember::classify_source_union_lemma(lemma) {
            return Some(Self::Unique(member));
        }
        if let Some(identity) = UniqueVerbIdentity::classify_source_union_lemma(lemma)
            && let Some(member) =
                UniqueVerbFamilyMember::classify_source_union_lemma(identity.canonical_lemma())
        {
            return Some(Self::Unique(member));
        }
        IrregularVerbFamilyMember::classify_source_lemma(lemma).map(Self::Irregular)
    }

    const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::Unique(member) => member.canonical_lemma(),
            Self::Irregular(member) => member.canonical_lemma(),
        }
    }

    fn analyses(self) -> Vec<ReviewedVerbAnalysis> {
        match self {
            Self::Unique(member) => vec![ReviewedVerbAnalysis::Unique(member)],
            Self::Irregular(member) => member
                .analyses()
                .iter()
                .copied()
                .map(|analysis| ReviewedVerbAnalysis::Irregular { member, analysis })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewedVerbAnalysis {
    Unique(UniqueVerbFamilyMember),
    Irregular {
        member: IrregularVerbFamilyMember,
        analysis: IrregularVerbAnalysis,
    },
}

impl ReviewedVerbAnalysis {
    fn lexeme(self) -> Result<VerbLexeme, InflectionError> {
        match self {
            Self::Unique(member) => Ok(member.lexeme()),
            Self::Irregular { member, analysis } => member
                .lexeme_for_analysis(analysis)
                .ok_or_else(|| InflectionError::InvalidInput {
                    reason: format!(
                        "reviewed analysis {} does not belong to {}",
                        analysis.code(),
                        member.canonical_lemma()
                    ),
                }),
        }
    }

    fn code(self) -> String {
        match self {
            Self::Unique(member) => format!(
                "unique:{}:{}",
                member.profile().canonical_lemma(),
                member.canonical_lemma()
            ),
            Self::Irregular { member, analysis } => {
                format!("irregular:{}:{}", member.canonical_lemma(), analysis.code())
            }
        }
    }

    fn authority(self) -> String {
        match self {
            Self::Unique(member) => format!(
                "Polivanova 2023 {}; official LMU LOVe principal-part crosscheck",
                member.source_section()
            ),
            Self::Irregular {
                member,
                analysis: IrregularVerbAnalysis::PolivanovaTable434,
            } => format!(
                "{} and {}, with official LMU LOVe principal-part crosscheck",
                IrregularVerbAnalysis::PolivanovaTable434.authority(),
                member.source_section()
            ),
            Self::Irregular { analysis, .. } => analysis.authority().to_string(),
        }
    }

    fn is_direct_source_cell(self, cell: VerbMorphologyCell, rule_id: RuleId) -> bool {
        match self {
            Self::Unique(_)
            | Self::Irregular {
                analysis: IrregularVerbAnalysis::PolivanovaTable434,
                ..
            } => rule_id == RuleId::VerbIrregularExact || cell == VerbMorphologyCell::Infinitive,
            Self::Irregular { analysis, .. } => {
                matches!(
                    analysis,
                    IrregularVerbAnalysis::LoveMetatiJePresent
                        | IrregularVerbAnalysis::LoveMetatiAjePresent
                ) && (cell == VerbMorphologyCell::Infinitive
                    || matches!(
                        cell,
                        VerbMorphologyCell::Finite(FiniteVerbCell {
                            tense: FiniteTense::Present,
                            person: Person::Third,
                            number: Number::Singular,
                        })
                    )
                    || matches!(
                        cell,
                        VerbMorphologyCell::Finite(FiniteVerbCell {
                            tense: FiniteTense::Aorist,
                            person: Person::Second | Person::Third,
                            number: Number::Singular,
                        })
                    ))
            }
        }
    }
}

fn reviewed_verb_id(profile: ReviewedVerbProfile) -> String {
    format!("{REVIEWED_VERB_ID_PREFIX}{}", profile.canonical_lemma())
}

fn reviewed_profile_from_id(id: &str) -> Option<ReviewedVerbProfile> {
    let lemma = id.strip_prefix(REVIEWED_VERB_ID_PREFIX)?;
    ReviewedVerbProfile::classify(lemma).filter(|profile| profile.canonical_lemma() == lemma)
}

fn reviewed_profile_for_dictionary_id(
    id: &str,
) -> Result<Option<ReviewedVerbProfile>, InflectionError> {
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?;
    if record.pos != PartOfSpeech::Verb.code() {
        return Err(InflectionError::InvalidInput {
            reason: format!("lexeme {id} is {}, not verb", record.pos),
        });
    }
    if let Some(profile) = ReviewedVerbProfile::classify(record.lemma) {
        return Ok(Some(profile));
    }

    let mut matches = Vec::new();
    for profile in UniqueVerbFamilyMember::all()
        .map(ReviewedVerbProfile::Unique)
        .chain(IrregularVerbFamilyMember::all().map(ReviewedVerbProfile::Irregular))
    {
        if lookup::lemma_maps_to_id(profile.canonical_lemma(), PartOfSpeech::Verb, id)?
            && !matches.contains(&profile)
        {
            matches.push(profile);
        }
    }
    match matches.as_slice() {
        [] => Ok(None),
        [profile] => Ok(Some(*profile)),
        _ => Err(InflectionError::InvalidInput {
            reason: format!("dictionary identity {id} aliases multiple reviewed verb profiles"),
        }),
    }
}

fn reviewed_dictionary_candidate(
    candidates: &[LexemeSummary],
    profile: ReviewedVerbProfile,
) -> Option<&LexemeSummary> {
    let mut exact = candidates
        .iter()
        .filter(|candidate| candidate.lemma == profile.canonical_lemma());
    let first = exact.next()?;
    exact.next().is_none().then_some(first)
}

fn resolve_queried_verb(
    query: &str,
    resolve: impl FnOnce(&str, Option<ReviewedVerbProfile>) -> Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    let normalized = orthography::lookup_key(query)?;
    let reviewed = ReviewedVerbProfile::classify(&normalized);
    let candidates = lookup(query, PartOfSpeech::Verb)?;
    if let Some(profile) = reviewed {
        if let Some(one) = reviewed_dictionary_candidate(&candidates, profile).or_else(|| {
            let [one] = candidates.as_slice() else {
                return None;
            };
            Some(one)
        }) {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated alias points at a missing lexeme".to_string(),
                })?;
            return queried_result(query, record, resolve(record.id, Some(profile)));
        }

        let mut result = resolve(&reviewed_verb_id(profile), Some(profile))?;
        if normalized != profile.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: profile.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
    match candidates.as_slice() {
        [one] => {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated alias points at a missing lexeme".to_string(),
                })?;
            queried_result(query, record, resolve(record.id, reviewed))
        }
        [] => Err(InflectionError::unknown_lemma(query, PartOfSpeech::Verb)),
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub(crate) fn resolve_verb_identity(query: &str) -> Result<(String, String), InflectionError> {
    let normalized = orthography::lookup_key(query)?;
    let reviewed = ReviewedVerbProfile::classify(&normalized);
    let candidates = lookup(query, PartOfSpeech::Verb)?;
    if let Some(profile) = reviewed {
        if let Some(one) = reviewed_dictionary_candidate(&candidates, profile).or_else(|| {
            let [one] = candidates.as_slice() else {
                return None;
            };
            Some(one)
        }) {
            return Ok((one.id.clone(), one.lemma.clone()));
        }
        return Ok((
            reviewed_verb_id(profile),
            profile.canonical_lemma().to_string(),
        ));
    }
    match candidates.as_slice() {
        [one] => Ok((one.id.clone(), one.lemma.clone())),
        [] => Err(InflectionError::unknown_lemma(query, PartOfSpeech::Verb)),
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub(crate) fn verb_identity_from_id(id: &str) -> Result<(String, String), InflectionError> {
    if let Some(profile) = reviewed_profile_from_id(id) {
        return Ok((id.to_string(), profile.canonical_lemma().to_string()));
    }
    let record = lexeme_identity(id, PartOfSpeech::Verb)?;
    Ok((record.id.to_string(), record.lemma.to_string()))
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
    supplied_profile: Option<ReviewedVerbProfile>,
    cell: VerbMorphologyCell,
    generate_reviewed: impl Fn(&VerbLexeme) -> Result<PredictedForm, InflectionError>,
    generate_metadata: impl FnOnce(&DictionaryVerbMetadata) -> Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    if let Some(profile) = reviewed_profile_from_id(id) {
        if supplied_profile.is_some_and(|supplied| supplied != profile) {
            return Err(InflectionError::InvalidInput {
                reason: format!("reviewed verb identity {id} conflicts with the supplied profile"),
            });
        }
        return reviewed_verb_form(profile, profile.canonical_lemma(), cell, generate_reviewed)
            .map_err(|error| error.with_lexeme_id(id));
    }

    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, feature) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, feature) {
        return Ok(form);
    }
    let profile = match supplied_profile {
        Some(profile) => Some(profile),
        None => reviewed_profile_for_dictionary_id(id)?,
    };
    if let Some(profile) = profile {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?;
        return reviewed_verb_form(profile, record.lemma, cell, generate_reviewed)
            .map_err(|error| error.with_lexeme_id(id));
    }
    let metadata = verb_metadata_by_id(id)?;
    generate_metadata(&metadata).map_err(|error| error.with_lexeme_id(id))
}

fn reviewed_verb_form(
    profile: ReviewedVerbProfile,
    display_lemma: &str,
    cell: VerbMorphologyCell,
    generate: impl Fn(&VerbLexeme) -> Result<PredictedForm, InflectionError>,
) -> Result<FormSet, InflectionError> {
    let mut analyses: Vec<FormAnalysis> = Vec::new();
    let mut any_prediction = false;
    for reviewed in profile.analyses() {
        let lexeme = reviewed.lexeme()?;
        let predicted = generate(&lexeme)?;
        let direct = reviewed.is_direct_source_cell(cell, predicted.rule_id);
        any_prediction |= !direct;
        let form = FormVariant {
            text: orthography::canonical_display(&predicted.text)?,
            romanization: None,
        };
        let source = FormSource::ReviewedGrammarTable {
            rule_id: predicted.rule_id,
        };
        let mut evidence = vec![MetadataEvidence {
            field: None,
            provenance: MetadataProvenance::ReviewedGrammarTable,
            source_feature: Some(format!("reviewed:verb:{}:{}", reviewed.code(), cell.key())),
            source_form: direct.then(|| form.text.clone()),
            crosscheck_features: Vec::new(),
            authority: Some(reviewed.authority()),
        }];
        if !direct {
            evidence.push(MetadataEvidence {
                field: None,
                provenance: MetadataProvenance::ProductiveRuleOutput,
                source_feature: Some(predicted.rule_id.code().to_string()),
                source_form: None,
                crosscheck_features: Vec::new(),
                authority: Some("docs/MORPHOLOGY_SPEC.md".to_string()),
            });
        }
        let candidate = FormAnalysis {
            variants: vec![form],
            source,
            evidence,
            trace: predicted.trace,
        };
        if let Some(existing) = analyses.iter_mut().find(|existing| {
            existing.variants == candidate.variants
                && existing.source == candidate.source
                && existing.trace == candidate.trace
        }) {
            for evidence in candidate.evidence {
                if !existing.evidence.contains(&evidence) {
                    existing.evidence.push(evidence);
                }
            }
        } else {
            analyses.push(candidate);
        }
    }

    let multiple = analyses.len() > 1;
    if multiple {
        for analysis in &mut analyses {
            for evidence in &mut analysis.evidence {
                if evidence.provenance == MetadataProvenance::ReviewedGrammarTable {
                    evidence.provenance = MetadataProvenance::DisputedGrammarTable;
                }
            }
        }
    }
    let mut variants = Vec::new();
    for analysis in &analyses {
        for variant in &analysis.variants {
            if !variants.contains(variant) {
                variants.push(variant.clone());
            }
        }
    }
    let Some(primary) = variants.first().cloned() else {
        return Err(InflectionError::InvalidInput {
            reason: "a reviewed verb profile produced no analysis".to_string(),
        });
    };
    let source = if multiple {
        FormSource::ReviewedGrammarAnalyses
    } else {
        analyses
            .first()
            .map(|analysis| analysis.source.clone())
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: "a reviewed verb profile produced no source".to_string(),
            })?
    };
    let trace = if multiple {
        Vec::new()
    } else {
        analyses
            .first()
            .map(|analysis| analysis.trace.clone())
            .unwrap_or_default()
    };
    let mut warnings = Vec::new();
    if any_prediction {
        warnings.push(InflectionWarning::PredictedNotDictionaryBacked);
    }
    if multiple {
        warnings.push(InflectionWarning::MultipleMorphologicalAnalyses);
        warnings.push(InflectionWarning::IncludesDisputedForms);
    }
    Ok(FormSet::new(
        orthography::canonical_display(display_lemma)?,
        primary,
        variants.into_iter().skip(1).collect(),
        source,
        warnings,
        trace,
        analyses,
    ))
}

fn exact_or_reviewed_verb_form(
    id: &str,
    feature: &str,
    supplied_profile: Option<ReviewedVerbProfile>,
    cell: VerbMorphologyCell,
    generate_reviewed: impl Fn(&VerbLexeme) -> Result<PredictedForm, InflectionError>,
) -> Result<FormSet, InflectionError> {
    if let Some(profile) = reviewed_profile_from_id(id) {
        if supplied_profile.is_some_and(|supplied| supplied != profile) {
            return Err(InflectionError::InvalidInput {
                reason: format!("reviewed verb identity {id} conflicts with the supplied profile"),
            });
        }
        return reviewed_verb_form(profile, profile.canonical_lemma(), cell, generate_reviewed)
            .map_err(|error| error.with_lexeme_id(id));
    }

    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, feature) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, feature) {
        return Ok(form);
    }
    let profile = match supplied_profile {
        Some(profile) => Some(profile),
        None => reviewed_profile_for_dictionary_id(id)?,
    };
    if let Some(profile) = profile {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?;
        return reviewed_verb_form(profile, record.lemma, cell, generate_reviewed)
            .map_err(|error| error.with_lexeme_id(id));
    }
    Err(InflectionError::unsupported(id, cell.requested()))
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
            if let Some(identity) =
                LongOnlyAdjectiveIdentity::classify_source_union_lemma(&normalized)
            {
                let mut result = long_only_adjective(identity, cell)?;
                if normalized != identity.canonical_lemma() {
                    result.add_warning(InflectionWarning::LexicalAliasUsed {
                        canonical: identity.canonical_lemma().to_string(),
                    });
                }
                return Ok(result);
            }
            let class = if normalized.ends_with('ъ')
                || normalized.ends_with("ꙑи")
                || normalized.ends_with("ыи")
            {
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
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Adjective)))?;
    let standard_pronominal = StandardPronominalIdentity::classify_source_union_lemma(record.lemma)
        .filter(|identity| identity.part_of_speech() == PartOfSpeech::Adjective);
    if let Some(identity) = standard_pronominal
        && cell.form == AdjectiveForm::Short
    {
        let mut result = standard_pronominal_form(identity, cell.case, cell.number, cell.gender)
            .map_err(|error| error.with_lexeme_id(id))?;
        if record.lemma != identity.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: identity.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    if standard_pronominal.is_some() {
        return Err(InflectionError::unsupported(
            id,
            RequestedCell::Adjective(cell),
        ));
    }
    if let Some(identity) = LongOnlyAdjectiveIdentity::classify_source_union_lemma(record.lemma) {
        let mut result =
            long_only_adjective(identity, cell).map_err(|error| error.with_lexeme_id(id))?;
        if record.lemma != identity.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: identity.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
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

fn long_only_adjective_form(
    identity: LongOnlyAdjectiveIdentity,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    let prediction = old_church_slavonic_core::adjective::decline_long_only(identity, cell)?;
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
                "adjective:2-a:plenum-tantum:{}:{}:{}:{}",
                cell.case.code(),
                cell.number.code(),
                cell.gender.code(),
                cell.animacy.code()
            )),
            source_form: None,
            crosscheck_features: Vec::new(),
            authority: Some("Polivanova 2023 §§285 and 303–305".to_string()),
        }],
        trace: trace.clone(),
    };
    Ok(FormSet::new(
        identity.canonical_lemma().to_string(),
        primary,
        Vec::new(),
        source,
        Vec::new(),
        trace,
        vec![analysis],
    ))
}

/// Resolve one cell of the exhaustive source-listed `plenum tantum`
/// adjective inventory.
pub fn long_only_adjective(
    identity: LongOnlyAdjectiveIdentity,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    long_only_adjective_form(identity, cell)
}

fn remap_determiner_error(
    error: InflectionError,
    lemma: &str,
    cell: DeterminerCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. } => {
            InflectionError::historically_invalid(lemma, RequestedCell::Determiner(cell))
        }
        InflectionError::UnsupportedCell { .. } => {
            InflectionError::unsupported(lemma, RequestedCell::Determiner(cell))
        }
        other => other,
    }
}

fn reviewed_adjectival_determiner(
    identity: DeterminerIdentity,
    cell: DeterminerCell,
) -> Result<FormSet, InflectionError> {
    let lexeme = identity
        .productive_lexeme()
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "the reviewed determiner identity has no productive adjectival profile"
                .to_string(),
        })?;
    let prediction = old_church_slavonic_core::determiner::decline(&lexeme.to_owned(), cell)?;
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
                "determiner:{}:{}:{}:{}:{}",
                lexeme.declension.code(),
                cell.case.code(),
                cell.number.code(),
                cell.gender.code(),
                cell.animacy.code()
            )),
            source_form: None,
            crosscheck_features: Vec::new(),
            authority: Some(
                match identity {
                    DeterminerIdentity::InterrogativeKotoryi => {
                        "Polivanova 2023 §§285, 303–305, and 316"
                    }
                    DeterminerIdentity::IndefiniteYeter => {
                        "Polivanova 2023 Paradigmatic Dictionary entry 343"
                    }
                    _ => {
                        return Err(InflectionError::InvalidInput {
                            reason: "the reviewed determiner identity is not adjectival"
                                .to_string(),
                        });
                    }
                }
                .to_string(),
            ),
        }],
        trace: trace.clone(),
    };
    Ok(FormSet::new(
        identity.canonical_lemma().to_string(),
        primary,
        Vec::new(),
        source,
        Vec::new(),
        trace,
        vec![analysis],
    ))
}

/// Resolve one member of the exhaustive reviewed determiner inventory.
pub fn reviewed_determiner(
    identity: DeterminerIdentity,
    cell: DeterminerCell,
) -> Result<FormSet, InflectionError> {
    let result = if let Some(standard) = identity.standard_pronominal() {
        standard_pronominal_form(standard, cell.case, cell.number, cell.gender)
    } else if let Some(irregular) = identity.irregular_agreeing() {
        irregular_agreeing(irregular, cell.case, cell.number, cell.gender)
    } else {
        reviewed_adjectival_determiner(identity, cell)
    };
    result.map_err(|error| remap_determiner_error(error, identity.canonical_lemma(), cell))
}

/// Resolve a determiner lemma through the source-exhaustive lexical inventory,
/// falling back to a future exact dictionary determiner only when it has no
/// reviewed grammatical identity.
pub fn determiner(lemma: &str, cell: DeterminerCell) -> Result<FormSet, InflectionError> {
    let candidates = lookup(lemma, PartOfSpeech::Determiner)?;
    match candidates.as_slice() {
        [one] => {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated lookup candidate is missing".to_string(),
                })?;
            queried_result(lemma, record, determiner_by_id(&one.id, cell))
        }
        [] => {
            let normalized = orthography::lookup_key(lemma)?;
            let identity = DeterminerIdentity::classify_source_union_lemma(&normalized)
                .ok_or_else(|| {
                    InflectionError::unknown_lemma(&normalized, PartOfSpeech::Determiner)
                })?;
            let mut result = reviewed_determiner(identity, cell)?;
            if normalized != identity.canonical_lemma() {
                result.add_warning(InflectionWarning::LexicalAliasUsed {
                    canonical: identity.canonical_lemma().to_string(),
                });
            }
            Ok(result)
        }
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

/// Generate a productive determiner from complete caller-supplied metadata.
pub fn determiner_with(
    lexeme: &DeterminerLexeme,
    cell: DeterminerCell,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::determiner::decline(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

/// Resolve one cell of the reviewed simple-cardinal inventory from one through
/// ten (including the independent cardinal `оба`).
pub fn reviewed_cardinal_numeral(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:cardinal:{}:{}:{}",
            cell.case.code(),
            cell.number.code(),
            cell.gender.map_or("none", Gender::code),
        ),
        old_church_slavonic_core::numeral::decline_cardinal(identity, cell)?,
    )
}

/// Resolve one adjective-agreement cell of a reviewed simple ordinal.
pub fn reviewed_ordinal_numeral(
    identity: OrdinalNumeralIdentity,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:ordinal:{}:{}:{}:{}:{}",
            cell.form.code(),
            cell.case.code(),
            cell.number.code(),
            cell.gender.code(),
            cell.animacy.code(),
        ),
        old_church_slavonic_core::numeral::decline_ordinal(identity, cell)?,
    )
}

/// Resolve a simple ordinal through the source-exhaustive first-through-tenth
/// inventory.
pub fn ordinal_numeral(lemma: &str, cell: AdjectiveCell) -> Result<FormSet, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = OrdinalNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    let mut result = reviewed_ordinal_numeral(identity, cell)?;
    if normalized != identity.canonical_lemma() {
        result.add_warning(InflectionWarning::LexicalAliasUsed {
            canonical: identity.canonical_lemma().to_string(),
        });
    }
    Ok(result)
}

/// Resolve one lexically compatible cell of a reviewed collective numeral.
pub fn reviewed_collective_numeral(
    identity: CollectiveNumeralIdentity,
    cell: CollectiveNumeralCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!("numeral:{}", cell.key()),
        old_church_slavonic_core::numeral::decline_collective(identity, cell)?,
    )
}

/// Resolve a collective numeral through the complete two-through-ten source
/// union, while retaining its pronominal-versus-adjectival cell type.
pub fn collective_numeral(
    lemma: &str,
    cell: CollectiveNumeralCell,
) -> Result<FormSet, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = CollectiveNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    let mut result = reviewed_collective_numeral(identity, cell)?;
    if normalized != identity.canonical_lemma() {
        result.add_warning(InflectionWarning::LexicalAliasUsed {
            canonical: identity.canonical_lemma().to_string(),
        });
    }
    Ok(result)
}

/// Resolve one noun cell of a source-listed OCS fractional numeral.
pub fn reviewed_fractional_numeral(
    identity: FractionalNumeralIdentity,
    cell: NounCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:fractional:1-{}:{}:{}",
            identity.denominator(),
            cell.case.code(),
            cell.number.code(),
        ),
        old_church_slavonic_core::numeral::decline_fractional(identity, cell)?,
    )
}

/// Resolve a fractional numeral through the period-bounded OCS source union.
pub fn fractional_numeral(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = FractionalNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    let mut result = reviewed_fractional_numeral(identity, cell)?;
    if normalized != identity.canonical_lemma() {
        result.add_warning(InflectionWarning::LexicalAliasUsed {
            canonical: identity.canonical_lemma().to_string(),
        });
    }
    Ok(result)
}

/// Resolve one noun cell of a source-listed OCS indefinite-quantity numeral.
pub fn reviewed_indefinite_numeral(
    identity: IndefiniteNumeralIdentity,
    cell: NounCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:indefinite-quantity:{}:{}",
            cell.case.code(),
            cell.number.code(),
        ),
        old_church_slavonic_core::numeral::decline_indefinite(identity, cell)?,
    )
}

/// Resolve an indefinite-quantity numeral through the closed OCS source union.
pub fn indefinite_numeral(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = IndefiniteNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    let mut result = reviewed_indefinite_numeral(identity, cell)?;
    if normalized != identity.canonical_lemma() {
        result.add_warning(InflectionWarning::LexicalAliasUsed {
            canonical: identity.canonical_lemma().to_string(),
        });
    }
    Ok(result)
}

/// Resolve one cell of a reviewed cardinal-magnitude head.
pub fn reviewed_cardinal_magnitude(
    identity: CardinalMagnitudeIdentity,
    cell: NumeralCell,
) -> Result<FormSet, InflectionError> {
    reviewed_numeral_variants(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:cardinal:magnitude:{}:{}:{}:{}",
            identity.rule_id().code(),
            cell.case.code(),
            cell.number.code(),
            cell.gender.map_or("none", Gender::code),
        ),
        old_church_slavonic_core::numeral::decline_magnitude(identity, cell)?,
    )
}

fn reviewed_numeral_variants(
    lemma: &str,
    authority: &str,
    source_feature: String,
    variants: Vec<NumeralVariant>,
) -> Result<FormSet, InflectionError> {
    let includes_reconstructed = variants
        .iter()
        .any(|variant| variant.status == NumeralVariantStatus::ReconstructedRule);
    let variants = variants
        .into_iter()
        .map(|variant| {
            let rule_id = variant.prediction.rule_id;
            let trace = variant.prediction.trace;
            let form = FormVariant {
                text: orthography::canonical_display(&variant.prediction.text)?,
                romanization: None,
            };
            let source = FormSource::ReviewedGrammarTable { rule_id };
            let analysis = FormAnalysis {
                variants: vec![form.clone()],
                source: source.clone(),
                evidence: vec![MetadataEvidence {
                    field: None,
                    provenance: match variant.status {
                        NumeralVariantStatus::ReviewedTable => {
                            MetadataProvenance::ReviewedGrammarTable
                        }
                        NumeralVariantStatus::ProductiveRule => {
                            MetadataProvenance::ProductiveRuleOutput
                        }
                        NumeralVariantStatus::CorpusAttestation => {
                            MetadataProvenance::CorpusEvaluationObservation
                        }
                        NumeralVariantStatus::PrimaryTextAttestation => {
                            MetadataProvenance::PrimaryTextAttestation
                        }
                        NumeralVariantStatus::ReconstructedRule => {
                            MetadataProvenance::ProductiveRuleOutput
                        }
                    },
                    source_feature: Some(format!("{source_feature}:{}", variant.status.code())),
                    source_form: matches!(
                        variant.status,
                        NumeralVariantStatus::ReviewedTable
                            | NumeralVariantStatus::CorpusAttestation
                            | NumeralVariantStatus::PrimaryTextAttestation
                    )
                    .then(|| form.text.clone()),
                    crosscheck_features: Vec::new(),
                    authority: Some(authority.to_string()),
                }],
                trace: trace.clone(),
            };
            Ok((form, analysis, trace))
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let (primary, primary_analysis, primary_trace) =
        variants
            .first()
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: format!("the {lemma:?} numeral cell has no reviewed forms"),
            })?;
    let mut surface_forms = Vec::with_capacity(variants.len());
    for (form, _, _) in &variants {
        if !surface_forms.contains(form) {
            surface_forms.push(form.clone());
        }
    }
    Ok(FormSet::new(
        orthography::canonical_display(lemma)?,
        primary.clone(),
        surface_forms.into_iter().skip(1).collect(),
        primary_analysis.source.clone(),
        includes_reconstructed
            .then_some(InflectionWarning::IncludesReconstructedForms)
            .into_iter()
            .collect(),
        primary_trace.clone(),
        variants
            .into_iter()
            .map(|(_, analysis, _)| analysis)
            .collect(),
    ))
}

const COMPOUND_CARDINAL_AUTHORITY: &str = "UT OCS Online §44.11–10,000; Polivanova 2023 \
    §§321–322, 345–351, 373–374, 383–384";

const COMPOUND_ORDINAL_AUTHORITY: &str = "Leuta and Havryliuk 2018 pp. 155, 161–162; \
    Gorshkov 2002 §§118–119; Polivanova OSD spreadsheet; Syntacticus/PROIEL \
    Suprasliensis ordinal crosscheck";

const DISTRIBUTIVE_CARDINAL_AUTHORITY: &str = "Leuta and Havryliuk 2018 pp. 154, \
    156, 164; UD OCS PROIEL r2.18 and native Syntacticus: Codex Zographensis \
    Mark 14:19 and 6:40, Codex Marianus Luke 9:14 and 10:1, John 8:9 and \
    21:25, and Codex Suprasliensis sentences 245344 and 253762";

/// Compose a reviewed cardinal from 11 through 10,000 while retaining correlated
/// multiword analyses and each word's own provenance.
pub fn compound_cardinal(
    value: u16,
    cell: CompoundCardinalCell,
    one_identity: CardinalNumeralIdentity,
) -> Result<RealizedCardinal, InflectionError> {
    compound_cardinal_with_options(
        value,
        cell,
        CardinalCompositionOptions {
            one_identity,
            ..CardinalCompositionOptions::DEFAULT
        },
    )
}

pub fn compound_cardinal_with_options(
    value: u16,
    cell: CompoundCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<RealizedCardinal, InflectionError> {
    validate_compound_cardinal_spec(value, options)?;

    let government = final_cardinal_digit(value)
        .map(|digit| digit_identity(digit, options.one_identity))
        .transpose()?
        .map_or(NumeralGovernment::GenitivePlural, |identity| {
            identity.government()
        });
    let expects_gender = matches!(government, NumeralGovernment::Agreement { .. });
    if cell.gender.is_some() != expects_gender {
        return Err(compound_cardinal_cell_error(value, cell));
    }

    let analyses = compose_cardinal_analyses(value, cell, options)
        .map_err(|error| remap_compound_cardinal_error(error, value, cell))?;

    RealizedCardinal::new(value, cell, government, analyses)
        .map_err(|error| remap_compound_cardinal_error(error, value, cell))
}

fn validate_compound_cardinal_spec(
    value: u16,
    options: CardinalCompositionOptions,
) -> Result<(), InflectionError> {
    if !(11..=10_000).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed compound-cardinal range is 11 through 10,000".to_string(),
        });
    }
    validate_cardinal_composition_options(options)
}

fn validate_cardinal_composition_options(
    options: CardinalCompositionOptions,
) -> Result<(), InflectionError> {
    if !matches!(
        options.one_identity,
        CardinalNumeralIdentity::OneYedin | CardinalNumeralIdentity::OneYedyn
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "compound one selection must be OneYedin or OneYedyn".to_string(),
        });
    }
    if !matches!(
        options.thousand_identity,
        CardinalMagnitudeIdentity::ThousandBackYus | CardinalMagnitudeIdentity::ThousandLittleYus
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "compound thousand selection must be ThousandBackYus or ThousandLittleYus"
                .to_string(),
        });
    }
    Ok(())
}

pub(crate) fn build_compound_cardinal_paradigm(
    value: u16,
    options: CardinalCompositionOptions,
) -> Result<CompoundCardinalParadigm, InflectionError> {
    validate_compound_cardinal_spec(value, options)?;
    Ok(CompoundCardinalParadigm {
        value,
        options,
        cells: CompoundCardinalCell::all()
            .map(|cell| CompoundCardinalOutcome {
                cell,
                result: compound_cardinal_with_options(value, cell, options),
            })
            .collect(),
    })
}

/// Compose source-backed distributive `по` with a dative cardinal from one
/// through 10,000. The construction is syntactic; all cardinal components
/// retain their ordinary inflection and provenance.
pub fn distributive_cardinal_with_options(
    value: u16,
    cell: DistributiveCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<RealizedDistributiveCardinal, InflectionError> {
    if !(1..=10_000).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed distributive-cardinal range is 1 through 10,000".to_string(),
        });
    }
    validate_cardinal_composition_options(options)?;

    let government = final_cardinal_digit(value)
        .map(|digit| digit_identity(digit, options.one_identity))
        .transpose()?
        .map_or(NumeralGovernment::GenitivePlural, |identity| {
            identity.government()
        });
    if cell.gender.is_some() != matches!(government, NumeralGovernment::Agreement { .. }) {
        return Err(distributive_cardinal_cell_error(value, cell));
    }

    let cardinal_analyses = if value <= 10 {
        lower_cardinal_analyses(value as u8, Case::Dative, cell.gender, options.one_identity)
    } else {
        compose_cardinal_analyses(
            value,
            CompoundCardinalCell {
                case: Case::Dative,
                gender: cell.gender,
            },
            options,
        )
    }
    .map_err(|error| remap_distributive_cardinal_error(error, value, cell))?;

    let preposition = reviewed_grammar_token(
        "по",
        RuleId::NumeralCardinalDistributive,
        "numeral:cardinal:distributive:po-dative",
        DISTRIBUTIVE_CARDINAL_AUTHORITY,
    )?;
    let analyses = cardinal_analyses
        .into_iter()
        .map(|analysis| {
            let mut tokens = Vec::with_capacity(analysis.tokens.len() + 1);
            tokens.push(PhraseToken {
                role: PhraseRole::Preposition,
                forms: preposition.clone(),
            });
            tokens.extend(analysis.tokens);
            DistributiveCardinalAnalysis { tokens }
        })
        .collect();

    RealizedDistributiveCardinal::new(value, cell, government, analyses)
        .map_err(|error| remap_distributive_cardinal_error(error, value, cell))
}

pub(crate) fn build_distributive_cardinal_paradigm(
    value: u16,
    options: CardinalCompositionOptions,
) -> Result<DistributiveCardinalParadigm, InflectionError> {
    if !(1..=10_000).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed distributive-cardinal range is 1 through 10,000".to_string(),
        });
    }
    validate_cardinal_composition_options(options)?;
    Ok(DistributiveCardinalParadigm {
        value,
        options,
        cells: DistributiveCardinalCell::all()
            .map(|cell| DistributiveCardinalOutcome {
                cell,
                result: distributive_cardinal_with_options(value, cell, options),
            })
            .collect(),
    })
}

fn final_cardinal_digit(value: u16) -> Option<u8> {
    let remainder = value % 100;
    if (11..=19).contains(&remainder) {
        Some((remainder - 10) as u8)
    } else {
        match remainder % 10 {
            0 => None,
            digit => Some(digit as u8),
        }
    }
}

fn compose_cardinal_analyses(
    value: u16,
    cell: CompoundCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    if value == 10_000 {
        return myriad_analyses(cell.case, options.thousand_identity);
    }

    let mut chunks = Vec::new();
    let thousands = (value / 1_000) as u8;
    if thousands != 0 {
        chunks.push(magnitude_analyses(
            thousands,
            options.thousand_identity,
            cell.case,
        )?);
    }
    let hundreds = ((value % 1_000) / 100) as u8;
    if hundreds != 0 {
        chunks.push(magnitude_analyses(
            hundreds,
            CardinalMagnitudeIdentity::HundredSto,
            cell.case,
        )?);
    }
    let remainder = (value % 100) as u8;
    if remainder != 0 {
        chunks.push(lower_cardinal_analyses(
            remainder,
            cell.case,
            cell.gender,
            options.one_identity,
        )?);
    }

    combine_cardinal_chunks(chunks)
}

fn combine_cardinal_chunks(
    chunks: Vec<Vec<CardinalPhraseAnalysis>>,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let mut combined = vec![CardinalPhraseAnalysis { tokens: Vec::new() }];
    for chunk in chunks {
        let mut next = Vec::with_capacity(combined.len() * chunk.len());
        for prefix in &combined {
            for suffix in &chunk {
                let mut tokens = prefix.tokens.clone();
                if !tokens.is_empty() {
                    tokens.push(PhraseToken {
                        role: PhraseRole::Conjunction,
                        forms: additive_connector()?,
                    });
                }
                tokens.extend(suffix.tokens.clone());
                next.push(CardinalPhraseAnalysis { tokens });
            }
        }
        combined = next;
    }
    Ok(combined)
}

fn lower_cardinal_analyses(
    value: u8,
    case: Case,
    gender: Option<Gender>,
    one_identity: CardinalNumeralIdentity,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    match value {
        1..=9 => Ok(vec![CardinalPhraseAnalysis {
            tokens: vec![numeral_token(digit_component(
                value,
                one_identity,
                case,
                gender,
            )?)],
        }]),
        10 => Ok(vec![CardinalPhraseAnalysis {
            tokens: vec![numeral_token(reviewed_cardinal_numeral(
                CardinalNumeralIdentity::Ten,
                NumeralCell {
                    case,
                    number: Number::Singular,
                    gender: None,
                },
            )?)],
        }]),
        11..=19 => {
            let unit = digit_component(value - 10, one_identity, case, gender)?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![
                    numeral_token(unit),
                    PhraseToken {
                        role: PhraseRole::Preposition,
                        forms: reviewed_grammar_token(
                            "на",
                            RuleId::NumeralCardinalTeen,
                            "numeral:cardinal:teen:preposition",
                            COMPOUND_CARDINAL_AUTHORITY,
                        )?,
                    },
                    PhraseToken {
                        role: PhraseRole::Numeral,
                        forms: reviewed_grammar_token(
                            "десѧте",
                            RuleId::NumeralCardinalTeen,
                            "numeral:cardinal:teen:invariant-ten",
                            COMPOUND_CARDINAL_AUTHORITY,
                        )?,
                    },
                ],
            }])
        }
        20..=99 => {
            let tens_digit = value / 10;
            let final_digit = value % 10;
            let mut analyses = tens_analyses(tens_digit, case)?;
            if final_digit != 0 {
                let unit = digit_component(final_digit, one_identity, case, gender)?;
                let connector = additive_connector()?;
                for analysis in &mut analyses {
                    analysis.tokens.push(PhraseToken {
                        role: PhraseRole::Conjunction,
                        forms: connector.clone(),
                    });
                    analysis.tokens.push(numeral_token(unit.clone()));
                }
            }
            Ok(analyses)
        }
        _ => Err(InflectionError::InvalidInput {
            reason: "a lower cardinal chunk must be between one and ninety-nine".to_string(),
        }),
    }
}

fn magnitude_analyses(
    multiplier: u8,
    magnitude: CardinalMagnitudeIdentity,
    case: Case,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let magnitude_number = match multiplier {
        1 | 5..=9 => Number::Singular,
        2 => Number::Dual,
        3 | 4 => Number::Plural,
        _ => {
            return Err(InflectionError::InvalidInput {
                reason: "a magnitude multiplier must be between one and nine".to_string(),
            });
        }
    };
    let magnitude_case = if multiplier >= 5 {
        Case::Genitive
    } else {
        case
    };
    let magnitude_form = reviewed_cardinal_magnitude(
        magnitude,
        NumeralCell {
            case: magnitude_case,
            number: if multiplier >= 5 {
                Number::Plural
            } else {
                magnitude_number
            },
            gender: None,
        },
    )?;
    let tokens = if multiplier == 1 {
        vec![numeral_token(magnitude_form)]
    } else {
        let leading = digit_component(
            multiplier,
            CardinalNumeralIdentity::OneYedin,
            case,
            (multiplier <= 4).then_some(Gender::Feminine),
        )?;
        vec![numeral_token(leading), numeral_token(magnitude_form)]
    };
    Ok(vec![CardinalPhraseAnalysis { tokens }])
}

fn myriad_analyses(
    case: Case,
    thousand_identity: CardinalMagnitudeIdentity,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let ten_thousand = CardinalPhraseAnalysis {
        tokens: vec![
            numeral_token(reviewed_cardinal_numeral(
                CardinalNumeralIdentity::Ten,
                NumeralCell {
                    case,
                    number: Number::Singular,
                    gender: None,
                },
            )?),
            numeral_token(reviewed_cardinal_magnitude(
                thousand_identity,
                NumeralCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    gender: None,
                },
            )?),
        ],
    };
    let tma = CardinalPhraseAnalysis {
        tokens: vec![numeral_token(reviewed_cardinal_magnitude(
            CardinalMagnitudeIdentity::MyriadTma,
            NumeralCell {
                case,
                number: Number::Singular,
                gender: None,
            },
        )?)],
    };
    Ok(vec![ten_thousand, tma])
}

fn additive_connector() -> Result<FormSet, InflectionError> {
    reviewed_grammar_token(
        "и",
        RuleId::NumeralCardinalAdditive,
        "numeral:cardinal:additive-connector",
        COMPOUND_CARDINAL_AUTHORITY,
    )
}

fn tens_analyses(
    multiplier: u8,
    case: Case,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    match multiplier {
        2 => {
            let leading = digit_component(
                2,
                CardinalNumeralIdentity::OneYedin,
                case,
                Some(Gender::Masculine),
            )?;
            let ten = reviewed_numeral_variants(
                CardinalNumeralIdentity::Ten.canonical_lemma(),
                COMPOUND_CARDINAL_AUTHORITY,
                format!("numeral:cardinal:tens:twenty:{}", case.code()),
                old_church_slavonic_core::numeral::decline_counted_ten(case, Number::Dual)?,
            )?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![numeral_token(leading), numeral_token(ten)],
            }])
        }
        3 | 4 => {
            let identity = digit_identity(multiplier, CardinalNumeralIdentity::OneYedin)?;
            let variants =
                old_church_slavonic_core::numeral::decline_counted_ten(case, Number::Plural)?;
            if case == Case::Nominative {
                let mut variants = variants.into_iter();
                let primary_ten = variants
                    .next()
                    .ok_or_else(|| InflectionError::InvalidInput {
                        reason: "counted ten has no nominative plural form".to_string(),
                    })?;
                let primary = CardinalPhraseAnalysis {
                    tokens: vec![
                        numeral_token(digit_component(
                            multiplier,
                            CardinalNumeralIdentity::OneYedin,
                            case,
                            Some(Gender::Masculine),
                        )?),
                        numeral_token(reviewed_numeral_variants(
                            CardinalNumeralIdentity::Ten.canonical_lemma(),
                            COMPOUND_CARDINAL_AUTHORITY,
                            format!("numeral:cardinal:tens:{multiplier}:{}:primary", case.code()),
                            vec![primary_ten],
                        )?),
                    ],
                };
                let mut analyses = vec![primary];
                if let Some(alternative_ten) = variants.next() {
                    analyses.push(CardinalPhraseAnalysis {
                        tokens: vec![
                            numeral_token(reviewed_cardinal_numeral(
                                identity,
                                NumeralCell {
                                    case,
                                    number: Number::Plural,
                                    gender: Some(Gender::Feminine),
                                },
                            )?),
                            numeral_token(reviewed_numeral_variants(
                                CardinalNumeralIdentity::Ten.canonical_lemma(),
                                COMPOUND_CARDINAL_AUTHORITY,
                                format!(
                                    "numeral:cardinal:tens:{multiplier}:{}:alternative",
                                    case.code()
                                ),
                                vec![alternative_ten],
                            )?),
                        ],
                    });
                }
                Ok(analyses)
            } else {
                let leading = reviewed_cardinal_numeral(
                    identity,
                    NumeralCell {
                        case,
                        number: Number::Plural,
                        gender: Some(Gender::Masculine),
                    },
                )?;
                let ten = reviewed_numeral_variants(
                    CardinalNumeralIdentity::Ten.canonical_lemma(),
                    COMPOUND_CARDINAL_AUTHORITY,
                    format!("numeral:cardinal:tens:{multiplier}:{}", case.code()),
                    variants,
                )?;
                Ok(vec![CardinalPhraseAnalysis {
                    tokens: vec![numeral_token(leading), numeral_token(ten)],
                }])
            }
        }
        5..=9 => {
            let leading =
                digit_component(multiplier, CardinalNumeralIdentity::OneYedin, case, None)?;
            let ten = reviewed_grammar_token(
                "десѧтъ",
                RuleId::NumeralCardinalTens,
                "numeral:cardinal:tens:invariant-genitive-plural-ten",
                COMPOUND_CARDINAL_AUTHORITY,
            )?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![numeral_token(leading), numeral_token(ten)],
            }])
        }
        _ => Err(InflectionError::InvalidInput {
            reason: "a decimal tens multiplier must be between two and nine".to_string(),
        }),
    }
}

fn digit_component(
    digit: u8,
    one_identity: CardinalNumeralIdentity,
    case: Case,
    gender: Option<Gender>,
) -> Result<FormSet, InflectionError> {
    let identity = digit_identity(digit, one_identity)?;
    let number = match identity.government() {
        NumeralGovernment::Agreement { number } => number,
        NumeralGovernment::GenitivePlural => Number::Singular,
    };
    reviewed_cardinal_numeral(
        identity,
        NumeralCell {
            case,
            number,
            gender,
        },
    )
}

fn digit_identity(
    digit: u8,
    one_identity: CardinalNumeralIdentity,
) -> Result<CardinalNumeralIdentity, InflectionError> {
    match digit {
        1 => Ok(one_identity),
        2 => Ok(CardinalNumeralIdentity::TwoDva),
        3 => Ok(CardinalNumeralIdentity::Three),
        4 => Ok(CardinalNumeralIdentity::Four),
        5 => Ok(CardinalNumeralIdentity::Five),
        6 => Ok(CardinalNumeralIdentity::Six),
        7 => Ok(CardinalNumeralIdentity::Seven),
        8 => Ok(CardinalNumeralIdentity::Eight),
        9 => Ok(CardinalNumeralIdentity::Nine),
        _ => Err(InflectionError::InvalidInput {
            reason: "a compound-cardinal digit must be between one and nine".to_string(),
        }),
    }
}

fn numeral_token(forms: FormSet) -> PhraseToken {
    PhraseToken {
        role: PhraseRole::Numeral,
        forms,
    }
}

fn compound_cardinal_cell_error(value: u16, cell: CompoundCardinalCell) -> InflectionError {
    InflectionError::historically_invalid(
        value.to_string(),
        RequestedCell::CompoundCardinal { value, cell },
    )
}

fn distributive_cardinal_cell_error(value: u16, cell: DistributiveCardinalCell) -> InflectionError {
    InflectionError::historically_invalid(
        value.to_string(),
        RequestedCell::DistributiveCardinal { value, cell },
    )
}

fn remap_distributive_cardinal_error(
    error: InflectionError,
    value: u16,
    cell: DistributiveCardinalCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. }
        | InflectionError::UnsupportedCell { .. } => distributive_cardinal_cell_error(value, cell),
        other => other,
    }
}

fn remap_compound_cardinal_error(
    error: InflectionError,
    value: u16,
    cell: CompoundCardinalCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. }
        | InflectionError::UnsupportedCell { .. } => compound_cardinal_cell_error(value, cell),
        other => other,
    }
}

/// Compose a source-reviewed ordinal from 11 through 1,000 while retaining
/// each adjective component and connector as an independently sourced token.
pub fn compound_ordinal(
    value: u16,
    cell: AdjectiveCell,
) -> Result<RealizedOrdinal, InflectionError> {
    validate_compound_ordinal_value(value)?;
    let analyses = compose_ordinal_analyses(value, cell)
        .map_err(|error| remap_compound_ordinal_error(error, value, cell))?;
    RealizedOrdinal::new(value, cell, analyses)
        .map_err(|error| remap_compound_ordinal_error(error, value, cell))
}

pub(crate) fn build_compound_ordinal_paradigm(
    value: u16,
) -> Result<CompoundOrdinalParadigm, InflectionError> {
    validate_compound_ordinal_value(value)?;
    Ok(CompoundOrdinalParadigm {
        value,
        cells: AdjectiveCell::all()
            .map(|cell| CompoundOrdinalOutcome {
                cell,
                result: compound_ordinal(value, cell),
            })
            .collect(),
    })
}

fn validate_compound_ordinal_value(value: u16) -> Result<(), InflectionError> {
    if value < MIN_COMPOUND_ORDINAL_VALUE {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "compound ordinals begin at {MIN_COMPOUND_ORDINAL_VALUE}; values one through ten use the simple-ordinal API"
            ),
        });
    }
    if value > MAX_COMPOUND_ORDINAL_VALUE {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "the declared Old Church Slavonic source profile ends at {MAX_COMPOUND_ORDINAL_VALUE}; reviewed grammars do not determine higher-magnitude ordinal formation"
            ),
        });
    }
    Ok(())
}

fn compose_ordinal_analyses(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    if (11..=19).contains(&value) {
        return teen_ordinal_analyses(value, cell);
    }
    if is_compound_ordinal_head(value) {
        return Ok(vec![compound_ordinal_head_analysis(value, cell)?]);
    }

    let mut chunks = Vec::new();
    let hundreds = value / 100;
    if hundreds != 0 {
        chunks.push(vec![compound_ordinal_head_analysis(hundreds * 100, cell)?]);
    }
    let remainder = value % 100;
    if remainder != 0 {
        chunks.push(lower_ordinal_analyses(remainder, cell)?);
    }
    let mut analyses = combine_ordinal_chunks(chunks, value < 100)?;
    analyses.extend(first_component_asyndetic_analyses(value, cell)?);
    if (21..=29).contains(&value) {
        analyses.extend(twenty_first_through_twenty_ninth_turns(value, cell)?);
    }
    Ok(analyses)
}

fn lower_ordinal_analyses(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    match value {
        1..=10 => Ok(vec![simple_ordinal_analysis(value, cell)?]),
        11..=19 => teen_ordinal_analyses(value, cell),
        20..=99 if value % 10 == 0 => Ok(vec![compound_ordinal_head_analysis(value, cell)?]),
        21..=99 => combine_ordinal_chunks(
            vec![
                vec![compound_ordinal_head_analysis((value / 10) * 10, cell)?],
                vec![simple_ordinal_analysis(value % 10, cell)?],
            ],
            true,
        ),
        _ => Err(InflectionError::InvalidInput {
            reason: "a lower ordinal chunk must be between one and ninety-nine".to_string(),
        }),
    }
}

fn simple_ordinal_analysis(
    value: u16,
    cell: AdjectiveCell,
) -> Result<OrdinalPhraseAnalysis, InflectionError> {
    let identity = simple_ordinal_identity(value)?;
    Ok(OrdinalPhraseAnalysis {
        construction: OrdinalComposition::FusedStem,
        tokens: vec![numeral_token(reviewed_ordinal_numeral(identity, cell)?)],
    })
}

fn teen_ordinal_analyses(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    let unit = simple_ordinal_identity(value - 10)?;
    let analytic = OrdinalPhraseAnalysis {
        construction: OrdinalComposition::AnalyticTeen,
        tokens: vec![
            numeral_token(reviewed_ordinal_numeral(unit, cell)?),
            PhraseToken {
                role: PhraseRole::Preposition,
                forms: reviewed_grammar_token(
                    "на",
                    RuleId::NumeralOrdinalTeen,
                    "numeral:ordinal:teen:preposition",
                    COMPOUND_ORDINAL_AUTHORITY,
                )?,
            },
            PhraseToken {
                role: PhraseRole::Numeral,
                forms: reviewed_grammar_token(
                    "десѧте",
                    RuleId::NumeralOrdinalTeen,
                    "numeral:ordinal:teen:invariant-ten",
                    COMPOUND_ORDINAL_AUTHORITY,
                )?,
            },
        ],
    };
    Ok(vec![analytic, compound_ordinal_head_analysis(value, cell)?])
}

fn compound_ordinal_head_analysis(
    value: u16,
    cell: AdjectiveCell,
) -> Result<OrdinalPhraseAnalysis, InflectionError> {
    let lemma = old_church_slavonic_core::numeral::compound_ordinal_lemma(value)?;
    let forms = reviewed_numeral_variants(
        &lemma,
        COMPOUND_ORDINAL_AUTHORITY,
        format!("numeral:ordinal:compound:{value}:{}", cell.key()),
        old_church_slavonic_core::numeral::decline_compound_ordinal_stem(value, cell)?,
    )?;
    Ok(OrdinalPhraseAnalysis {
        construction: OrdinalComposition::FusedStem,
        tokens: vec![numeral_token(forms)],
    })
}

fn combine_ordinal_chunks(
    chunks: Vec<Vec<OrdinalPhraseAnalysis>>,
    prefer_conjunction_i: bool,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    if chunks.len() == 1 {
        return chunks
            .into_iter()
            .next()
            .filter(|chunk| !chunk.is_empty())
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: "a compound ordinal lost its only component".to_string(),
            });
    }
    let combinations = ordinal_chunk_combinations(chunks)?;

    let orders = if prefer_conjunction_i {
        [
            OrdinalComposition::ConjunctionI,
            OrdinalComposition::Asyndetic,
            OrdinalComposition::ConjunctionTi,
        ]
    } else {
        [
            OrdinalComposition::Asyndetic,
            OrdinalComposition::ConjunctionI,
            OrdinalComposition::ConjunctionTi,
        ]
    };
    let mut analyses = Vec::with_capacity(combinations.len() * orders.len());
    for chunks in combinations {
        for construction in orders {
            analyses.push(OrdinalPhraseAnalysis {
                construction,
                tokens: connect_ordinal_chunks(&chunks, construction)?,
            });
        }
    }
    Ok(analyses)
}

fn ordinal_chunk_combinations(
    chunks: Vec<Vec<OrdinalPhraseAnalysis>>,
) -> Result<Vec<Vec<OrdinalPhraseAnalysis>>, InflectionError> {
    if chunks.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "a compound ordinal requires at least one component".to_string(),
        });
    }
    if chunks.len() == 1 {
        return chunks
            .into_iter()
            .next()
            .map(|chunk| chunk.into_iter().map(|analysis| vec![analysis]).collect())
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: "a compound ordinal lost its only component".to_string(),
            });
    }

    let mut combinations = vec![Vec::<OrdinalPhraseAnalysis>::new()];
    for chunk in chunks {
        let mut next = Vec::with_capacity(combinations.len() * chunk.len());
        for prefix in &combinations {
            for suffix in &chunk {
                let mut analyses = prefix.clone();
                analyses.push(suffix.clone());
                next.push(analyses);
            }
        }
        combinations = next;
    }
    Ok(combinations)
}

fn first_component_asyndetic_analyses(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    let citation = ordinal_component_citation_cell();
    let chunks = if value < 100 {
        vec![
            vec![compound_ordinal_head_analysis((value / 10) * 10, cell)?],
            vec![simple_ordinal_analysis(value % 10, citation)?],
        ]
    } else {
        vec![
            vec![compound_ordinal_head_analysis((value / 100) * 100, cell)?],
            lower_ordinal_analyses(value % 100, citation)?,
        ]
    };

    ordinal_chunk_combinations(chunks).map(|combinations| {
        combinations
            .into_iter()
            .map(|chunks| OrdinalPhraseAnalysis {
                construction: OrdinalComposition::AsyndeticFirstComponent,
                tokens: chunks.into_iter().flat_map(|chunk| chunk.tokens).collect(),
            })
            .collect()
    })
}

fn ordinal_component_citation_cell() -> AdjectiveCell {
    AdjectiveCell {
        form: AdjectiveForm::Short,
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
    }
}

fn twenty_first_through_twenty_ninth_turns(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<OrdinalPhraseAnalysis>, InflectionError> {
    let unit = simple_ordinal_identity(value - 20)?;
    let agreeing_unit = numeral_token(reviewed_ordinal_numeral(unit, cell)?);
    let grammar_token = |text, feature| {
        reviewed_grammar_token(
            text,
            RuleId::NumeralOrdinalCircumlocutive,
            feature,
            COMPOUND_ORDINAL_AUTHORITY,
        )
    };

    Ok(vec![
        OrdinalPhraseAnalysis {
            construction: OrdinalComposition::BetweenTens,
            tokens: vec![
                agreeing_unit.clone(),
                PhraseToken {
                    role: PhraseRole::Preposition,
                    forms: grammar_token(
                        "междю",
                        "numeral:ordinal:circumlocutive:between-tens:preposition",
                    )?,
                },
                PhraseToken {
                    role: PhraseRole::Numeral,
                    forms: grammar_token(
                        "десетма",
                        "numeral:ordinal:circumlocutive:between-tens:dual",
                    )?,
                },
            ],
        },
        OrdinalPhraseAnalysis {
            construction: OrdinalComposition::UnitWithinThirdTen,
            tokens: vec![
                agreeing_unit,
                PhraseToken {
                    role: PhraseRole::Numeral,
                    forms: grammar_token(
                        "третиаго",
                        "numeral:ordinal:circumlocutive:third-ten:ordinal",
                    )?,
                },
                PhraseToken {
                    role: PhraseRole::Numeral,
                    forms: grammar_token("десѧте", "numeral:ordinal:circumlocutive:third-ten:ten")?,
                },
            ],
        },
    ])
}

fn connect_ordinal_chunks(
    chunks: &[OrdinalPhraseAnalysis],
    construction: OrdinalComposition,
) -> Result<Vec<PhraseToken>, InflectionError> {
    let connector = match construction {
        OrdinalComposition::Asyndetic => None,
        OrdinalComposition::ConjunctionI => Some("и"),
        OrdinalComposition::ConjunctionTi => Some("ти"),
        OrdinalComposition::AnalyticTeen
        | OrdinalComposition::FusedStem
        | OrdinalComposition::AsyndeticFirstComponent
        | OrdinalComposition::BetweenTens
        | OrdinalComposition::UnitWithinThirdTen => {
            return Err(InflectionError::InvalidInput {
                reason: "only additive ordinal constructions can connect chunks".to_string(),
            });
        }
    };
    let Some(connector) = connector else {
        return Ok(chunks
            .iter()
            .flat_map(|chunk| chunk.tokens.clone())
            .collect());
    };

    let token_count = chunks.iter().map(|chunk| chunk.tokens.len()).sum::<usize>();
    let mut result = Vec::with_capacity(token_count + chunks.len().saturating_sub(1));
    for (index, chunk) in chunks.iter().enumerate() {
        if index != 0 {
            result.push(PhraseToken {
                role: PhraseRole::Conjunction,
                forms: reviewed_grammar_token(
                    connector,
                    RuleId::NumeralOrdinalAdditive,
                    if connector == "и" {
                        "numeral:ordinal:additive-connector:i"
                    } else {
                        "numeral:ordinal:additive-connector:ti"
                    },
                    COMPOUND_ORDINAL_AUTHORITY,
                )?,
            });
        }
        result.extend(chunk.tokens.clone());
    }
    Ok(result)
}

fn simple_ordinal_identity(value: u16) -> Result<OrdinalNumeralIdentity, InflectionError> {
    OrdinalNumeralIdentity::ALL
        .into_iter()
        .find(|identity| u16::from(identity.value()) == value)
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a simple ordinal component must be between one and ten".to_string(),
        })
}

fn is_compound_ordinal_head(value: u16) -> bool {
    (20..=90).contains(&value) && value % 10 == 0
        || (100..=900).contains(&value) && value % 100 == 0
        || value == 1_000
}

fn compound_ordinal_cell_error(value: u16, cell: AdjectiveCell) -> InflectionError {
    InflectionError::historically_invalid(
        value.to_string(),
        RequestedCell::CompoundOrdinal { value, cell },
    )
}

fn remap_compound_ordinal_error(
    error: InflectionError,
    value: u16,
    cell: AdjectiveCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. }
        | InflectionError::UnsupportedCell { .. } => compound_ordinal_cell_error(value, cell),
        other => other,
    }
}

/// Resolve a numeral lemma through the source-reviewed simple-cardinal
/// inventory, retaining exact-table fallback for other numeral types.
pub fn numeral(lemma: &str, cell: NumeralCell) -> Result<FormSet, InflectionError> {
    let candidates = lookup(lemma, PartOfSpeech::Numeral)?;
    match candidates.as_slice() {
        [one] => {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated lookup candidate is missing".to_string(),
                })?;
            queried_result(lemma, record, numeral_cell_by_id(&one.id, cell))
        }
        [] => {
            let normalized = orthography::lookup_key(lemma)?;
            let identity = CardinalNumeralIdentity::classify_source_union_lemma(&normalized)
                .ok_or_else(|| {
                    InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral)
                })?;
            let mut result = reviewed_cardinal_numeral(identity, cell)?;
            if normalized != identity.canonical_lemma() {
                result.add_warning(InflectionWarning::LexicalAliasUsed {
                    canonical: identity.canonical_lemma().to_string(),
                });
            }
            Ok(result)
        }
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
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

pub(crate) fn reviewed_grammar_token(
    text: &str,
    rule_id: RuleId,
    source_feature: &str,
    authority: &'static str,
) -> Result<FormSet, InflectionError> {
    let text = orthography::canonical_display(text)?;
    let form = FormVariant {
        text: text.clone(),
        romanization: None,
    };
    let source = FormSource::ReviewedGrammarTable { rule_id };
    Ok(FormSet::new(
        text.clone(),
        form.clone(),
        Vec::new(),
        source.clone(),
        Vec::new(),
        Vec::new(),
        vec![FormAnalysis {
            variants: vec![form],
            source,
            evidence: vec![MetadataEvidence {
                field: None,
                provenance: MetadataProvenance::ReviewedGrammarTable,
                source_feature: Some(source_feature.to_string()),
                source_form: Some(text),
                crosscheck_features: Vec::new(),
                authority: Some(authority.to_string()),
            }],
            trace: Vec::new(),
        }],
    ))
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

fn standard_pronominal_form(
    identity: StandardPronominalIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<FormSet, InflectionError> {
    let prediction = old_church_slavonic_core::pronoun::decline_standard_pronominal(
        identity, case, number, gender,
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
                "{}:2-p:{}:{}:{}:{}",
                identity.part_of_speech().code(),
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

/// Decline any reviewed regular identity in Polivanova's class `2/p`, including
/// identities whose primary lexical ownership is adjective, determiner, or
/// numeral rather than pronoun.
pub fn regular_pronominal(
    identity: StandardPronominalIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<FormSet, InflectionError> {
    standard_pronominal_form(identity, case, number, gender)
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
    resolve_queried_verb(lemma, |id, profile| {
        finite_by_id_with_profile(id, cell, profile)
    })
}

pub fn finite_by_id(id: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    finite_by_id_with_profile(id, cell, None)
}

fn finite_by_id_with_profile(
    id: &str,
    cell: FiniteVerbCell,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    verb_metadata_form(
        id,
        &cell.key(),
        profile,
        VerbMorphologyCell::Finite(cell),
        |lexeme| old_church_slavonic_core::verb::finite(lexeme, cell),
        |metadata| generate_finite_from_metadata(metadata, cell),
    )
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
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_finite_paradigm(id, &lemma))
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
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_present_paradigm(id, &lemma))
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

pub(crate) fn reviewed_finite_verb_with(
    lexeme: &VerbLexeme,
    cell: FiniteVerbCell,
    authority: &'static str,
) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::finite(lexeme, cell),
        FormSourceKind::ReviewedGrammar(authority),
    )
}

pub fn imperative(lemma: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, |id, profile| {
        imperative_by_id_with_profile(id, cell, profile)
    })
}

pub fn imperative_by_id(id: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    imperative_by_id_with_profile(id, cell, None)
}

fn imperative_by_id_with_profile(
    id: &str,
    cell: ImperativeCell,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    verb_metadata_form(
        id,
        &cell.key(),
        profile,
        VerbMorphologyCell::Imperative(cell),
        |lexeme| old_church_slavonic_core::verb::imperative(lexeme, cell),
        |metadata| generate_imperative_from_metadata(metadata, cell),
    )
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
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_imperative_paradigm(id, &lemma))
}

pub(crate) fn build_imperative_paradigm(id: &str, lemma: &str) -> ImperativeParadigm {
    ImperativeParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, ImperativeCell::SUPPORTED, imperative_by_id),
    }
}

pub fn l_participle(lemma: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, |id, profile| {
        l_participle_by_id_with_profile(id, cell, profile)
    })
}

pub fn l_participle_by_id(id: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    l_participle_by_id_with_profile(id, cell, None)
}

fn l_participle_by_id_with_profile(
    id: &str,
    cell: LParticipleCell,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    verb_metadata_form(
        id,
        &cell.key(),
        profile,
        VerbMorphologyCell::LParticiple(cell),
        |lexeme| old_church_slavonic_core::verb::l_participle(lexeme, cell),
        |metadata| generate_l_participle_from_metadata(metadata, cell),
    )
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
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_l_participle_paradigm(id, &lemma))
}

pub(crate) fn build_l_participle_paradigm(id: &str, lemma: &str) -> LParticipleParadigm {
    LParticipleParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, LParticipleCell::all(), l_participle_by_id),
    }
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, |id, profile| {
        participle_by_id_with_profile(id, cell, profile)
    })
}

pub fn participle_by_id(id: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    participle_by_id_with_profile(id, cell, None)
}

fn participle_by_id_with_profile(
    id: &str,
    cell: ParticipleCell,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    verb_metadata_form(
        id,
        &cell.key(),
        profile,
        VerbMorphologyCell::Participle(cell),
        |lexeme| old_church_slavonic_core::verb::participle(lexeme, cell),
        |metadata| generate_participle_from_metadata(metadata, cell),
    )
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
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_participle_paradigm(id, &lemma, kind))
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
    resolve_queried_verb(lemma, |id, profile| {
        participle_citation_by_id_with_profile(id, kind, profile)
    })
}

pub fn participle_citation_by_id(
    id: &str,
    kind: ParticipleKind,
) -> Result<FormSet, InflectionError> {
    participle_citation_by_id_with_profile(id, kind, None)
}

fn participle_citation_by_id_with_profile(
    id: &str,
    kind: ParticipleKind,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    let feature = format!("verb:participle:{}:citation", kind.code());
    let cell = ParticipleCell {
        kind,
        adjective: AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        },
    };
    verb_metadata_form(
        id,
        &feature,
        profile,
        VerbMorphologyCell::Participle(cell),
        |lexeme| old_church_slavonic_core::verb::participle(lexeme, cell),
        |metadata| generate_participle_from_metadata(metadata, cell),
    )
}

pub fn infinitive(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, infinitive_by_id_with_profile)
}

pub fn infinitive_by_id(id: &str) -> Result<FormSet, InflectionError> {
    infinitive_by_id_with_profile(id, None)
}

fn infinitive_by_id_with_profile(
    id: &str,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    exact_or_reviewed_verb_form(
        id,
        "verb:infinitive",
        profile,
        VerbMorphologyCell::Infinitive,
        old_church_slavonic_core::verb::infinitive,
    )
}

pub fn infinitive_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::infinitive(lexeme),
        FormSourceKind::Explicit,
    )
}

pub fn supine(lemma: &str) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, supine_by_id_with_profile)
}

pub fn supine_by_id(id: &str) -> Result<FormSet, InflectionError> {
    supine_by_id_with_profile(id, None)
}

fn supine_by_id_with_profile(
    id: &str,
    profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    exact_or_reviewed_verb_form(
        id,
        "verb:supine",
        profile,
        VerbMorphologyCell::Supine,
        old_church_slavonic_core::verb::supine,
    )
}

pub fn supine_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::supine(lexeme),
        FormSourceKind::Explicit,
    )
}

fn verbal_noun_citation_cell() -> NounCell {
    NounCell {
        case: Case::Nominative,
        number: Number::Singular,
    }
}

/// Resolve the source-listed or productively formed citation (nominative
/// singular) of an OCS verbal noun.
pub fn verbal_noun(lemma: &str) -> Result<FormSet, InflectionError> {
    verbal_noun_form(lemma, verbal_noun_citation_cell())
}

/// Resolve one declined cell of the derived soft-neuter noun.
pub fn verbal_noun_form(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    resolve_queried_verb(lemma, |id, profile| {
        verbal_noun_form_by_id_with_profile(id, cell, profile)
    })
}

pub fn verbal_noun_by_id(id: &str) -> Result<FormSet, InflectionError> {
    verbal_noun_form_by_id(id, verbal_noun_citation_cell())
}

pub fn verbal_noun_form_by_id(id: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    verbal_noun_form_by_id_with_profile(id, cell, None)
}

fn verbal_noun_form_by_id_with_profile(
    id: &str,
    cell: NounCell,
    supplied_profile: Option<ReviewedVerbProfile>,
) -> Result<FormSet, InflectionError> {
    if let Some(profile) = reviewed_profile_from_id(id) {
        if supplied_profile.is_some_and(|supplied| supplied != profile) {
            return Err(InflectionError::InvalidInput {
                reason: format!("reviewed verb identity {id} conflicts with the supplied profile"),
            });
        }
        return reviewed_verb_form(
            profile,
            profile.canonical_lemma(),
            VerbMorphologyCell::VerbalNoun(cell),
            |lexeme| old_church_slavonic_core::verb::verbal_noun(lexeme, cell),
        )
        .map_err(|error| error.with_lexeme_id(id));
    }

    ensure_pos(id, PartOfSpeech::Verb)?;
    let listed = lookup::table_form(id, "verb:verbal-noun")
        .or_else(|| lookup::override_form(id, "verb:verbal-noun"));
    if let Some(citation) = listed {
        if cell == verbal_noun_citation_cell() {
            return Ok(citation);
        }
        return decline_listed_verbal_noun(&citation, cell)
            .map_err(|error| error.with_lexeme_id(id));
    }

    let profile = match supplied_profile {
        Some(profile) => Some(profile),
        None => reviewed_profile_for_dictionary_id(id)?,
    };
    if let Some(profile) = profile {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?;
        return reviewed_verb_form(
            profile,
            record.lemma,
            VerbMorphologyCell::VerbalNoun(cell),
            |lexeme| old_church_slavonic_core::verb::verbal_noun(lexeme, cell),
        )
        .map_err(|error| error.with_lexeme_id(id));
    }
    let metadata = verb_metadata_by_id(id)?;
    generate_verbal_noun_from_metadata(&metadata, cell).map_err(|error| error.with_lexeme_id(id))
}

pub fn verbal_noun_with(lexeme: &VerbLexeme, cell: NounCell) -> Result<FormSet, InflectionError> {
    canonical_prediction(
        &lexeme.lemma,
        old_church_slavonic_core::verb::verbal_noun(lexeme, cell),
        FormSourceKind::Explicit,
    )
}

pub fn verbal_noun_paradigm_by_id(id: &str) -> Result<VerbalNounParadigm, InflectionError> {
    let (_, lemma) = verb_identity_from_id(id)?;
    Ok(build_verbal_noun_paradigm(id, &lemma))
}

pub(crate) fn build_verbal_noun_paradigm(id: &str, lemma: &str) -> VerbalNounParadigm {
    VerbalNounParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells: cell_outcomes(id, NounCell::all(), verbal_noun_form_by_id),
    }
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
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(part_of_speech)))?;
    if let Some(identity) = StandardPronominalIdentity::classify_source_union_lemma(record.lemma)
        .filter(|identity| identity.part_of_speech() == part_of_speech)
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
        let mut result = standard_pronominal_form(identity, cell.case, cell.number, gender)
            .map_err(|error| error.with_lexeme_id(id))?;
        if record.lemma != identity.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: identity.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
    if part_of_speech == PartOfSpeech::Pronoun {
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
            "къто" | "чьто" | "никъто" => match (cell.person, cell.gender) {
                (None, None) => interrogative_pronoun(
                    if record.lemma == "къто" || record.lemma == "никъто" {
                        InterrogativePronounIdentity::Kto
                    } else {
                        InterrogativePronounIdentity::Chto
                    },
                    cell.case,
                )
                .and_then(|base| {
                    if record.lemma == "никъто" {
                        crate::phrases::single_token_pronominal_family_with(
                            base,
                            cell.case,
                            PronominalFamilySpec {
                                prefix: Some(PronominalPrefix::Ni),
                                ..PronominalFamilySpec::default()
                            },
                        )
                    } else {
                        Ok(base)
                    }
                }),
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
    if part_of_speech == PartOfSpeech::Determiner && record.lemma == "кꙑи" {
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

pub fn determiner_by_id(id: &str, cell: DeterminerCell) -> Result<FormSet, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Determiner)?;
    if let Some(identity) = DeterminerIdentity::classify_source_union_lemma(record.lemma) {
        let mut result =
            reviewed_determiner(identity, cell).map_err(|error| error.with_lexeme_id(id))?;
        if record.lemma != identity.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: identity.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
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
    numeral_cell_by_id(
        id,
        NumeralCell {
            case: cell.case,
            number: cell.number,
            gender: None,
        },
    )
}

pub fn gendered_numeral_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    numeral_cell_by_id(
        id,
        NumeralCell {
            case: cell.case,
            number: cell.number,
            gender: Some(cell.gender),
        },
    )
}

fn numeral_cell_by_id(id: &str, cell: NumeralCell) -> Result<FormSet, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Numeral)?;
    if let Some(identity) = CardinalNumeralIdentity::classify_source_union_lemma(record.lemma) {
        let mut result =
            reviewed_cardinal_numeral(identity, cell).map_err(|error| error.with_lexeme_id(id))?;
        if record.lemma != identity.canonical_lemma() {
            result.add_warning(InflectionWarning::LexicalAliasUsed {
                canonical: identity.canonical_lemma().to_string(),
            });
        }
        return Ok(result);
    }
    closed_class_by_id(id, PartOfSpeech::Numeral, cell.closed_class())
}

pub(crate) fn build_cardinal_numeral_paradigm(
    identity: CardinalNumeralIdentity,
) -> CardinalNumeralParadigm {
    CardinalNumeralParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells: NumeralCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_cardinal_numeral(identity, cell),
            })
            .collect(),
    }
}

pub(crate) fn build_ordinal_numeral_paradigm(
    identity: OrdinalNumeralIdentity,
) -> OrdinalNumeralParadigm {
    OrdinalNumeralParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells: AdjectiveCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_ordinal_numeral(identity, cell),
            })
            .collect(),
    }
}

pub fn ordinal_numeral_paradigm(lemma: &str) -> Result<OrdinalNumeralParadigm, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = OrdinalNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    Ok(build_ordinal_numeral_paradigm(identity))
}

pub(crate) fn build_collective_numeral_paradigm(
    identity: CollectiveNumeralIdentity,
) -> CollectiveNumeralParadigm {
    let cells = match identity.declension() {
        CollectiveNumeralDeclension::Pronominal => GenderedCell::all()
            .map(CollectiveNumeralCell::Pronominal)
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_collective_numeral(identity, cell),
            })
            .collect(),
        CollectiveNumeralDeclension::Adjectival => AdjectiveCell::all()
            .map(CollectiveNumeralCell::Adjectival)
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_collective_numeral(identity, cell),
            })
            .collect(),
    };
    CollectiveNumeralParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells,
    }
}

pub fn collective_numeral_paradigm(
    lemma: &str,
) -> Result<CollectiveNumeralParadigm, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = CollectiveNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    Ok(build_collective_numeral_paradigm(identity))
}

pub(crate) fn build_fractional_numeral_paradigm(
    identity: FractionalNumeralIdentity,
) -> FractionalNumeralParadigm {
    FractionalNumeralParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells: NounCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_fractional_numeral(identity, cell),
            })
            .collect(),
    }
}

pub fn fractional_numeral_paradigm(
    lemma: &str,
) -> Result<FractionalNumeralParadigm, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = FractionalNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    Ok(build_fractional_numeral_paradigm(identity))
}

pub(crate) fn build_indefinite_numeral_paradigm(
    identity: IndefiniteNumeralIdentity,
) -> IndefiniteNumeralParadigm {
    IndefiniteNumeralParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells: NounCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_indefinite_numeral(identity, cell),
            })
            .collect(),
    }
}

pub fn indefinite_numeral_paradigm(
    lemma: &str,
) -> Result<IndefiniteNumeralParadigm, InflectionError> {
    let normalized = orthography::lookup_key(lemma)?;
    let identity = IndefiniteNumeralIdentity::classify_source_union_lemma(&normalized)
        .ok_or_else(|| InflectionError::unknown_lemma(&normalized, PartOfSpeech::Numeral))?;
    Ok(build_indefinite_numeral_paradigm(identity))
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
    let identity =
        DeterminerIdentity::classify_source_union_lemma(record.lemma).ok_or_else(|| {
            InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::AdjectiveClass],
            }
        })?;
    let mut paradigm = build_determiner_paradigm(identity);
    for outcome in &mut paradigm.cells {
        if let Err(error) = &outcome.result {
            outcome.result = Err(error.clone().with_lexeme_id(id));
        }
    }
    Ok(paradigm)
}

pub fn determiner_paradigm(lemma: &str) -> Result<DeterminerParadigm, InflectionError> {
    let candidates = lookup(lemma, PartOfSpeech::Determiner)?;
    match candidates.as_slice() {
        [one] => determiner_paradigm_by_id(&one.id),
        [] => {
            let normalized = orthography::lookup_key(lemma)?;
            let identity = DeterminerIdentity::classify_source_union_lemma(&normalized)
                .ok_or_else(|| {
                    InflectionError::unknown_lemma(normalized, PartOfSpeech::Determiner)
                })?;
            Ok(build_determiner_paradigm(identity))
        }
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub(crate) fn build_determiner_paradigm(identity: DeterminerIdentity) -> DeterminerParadigm {
    DeterminerParadigm {
        identity,
        lemma: identity.canonical_lemma().to_string(),
        cells: DeterminerCell::all()
            .map(|cell| CellOutcome {
                cell,
                result: reviewed_determiner(identity, cell),
            })
            .collect(),
    }
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
    Ok(build_numeral_paradigm(id, record.lemma))
}

pub fn gendered_numeral_paradigm_by_id(
    id: &str,
) -> Result<GenderedNumeralParadigm, InflectionError> {
    let record = lexeme_identity(id, PartOfSpeech::Numeral)?;
    Ok(build_gendered_numeral_paradigm(id, record.lemma))
}

pub(crate) fn build_numeral_paradigm(id: &str, lemma: &str) -> NumeralParadigm {
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: PartOfSpeech::Numeral,
        cells: cell_outcomes(id, UngenderedCell::all(), numeral_by_id),
    }
}

pub(crate) fn build_gendered_numeral_paradigm(id: &str, lemma: &str) -> GenderedNumeralParadigm {
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: PartOfSpeech::Numeral,
        cells: cell_outcomes(id, GenderedCell::all(), gendered_numeral_by_id),
    }
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
        ["verb", "verbal-noun", case, number] if record.pos == "verb" => verbal_noun_form_by_id(
            id,
            NounCell {
                case: parse_feature_case(case)?,
                number: parse_feature_number(number)?,
            },
        ),
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
        lexeme.stems.l_participle = Some(analysis.stem.value.clone());
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

pub fn verbal_noun_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: NounCell,
) -> Result<FormSet, InflectionError> {
    generate_verbal_noun_from_metadata(metadata, cell)
}

fn generate_verbal_noun_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: NounCell,
) -> Result<FormSet, InflectionError> {
    if metadata.past_passive_participle.is_empty() {
        return Err(InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::VerbalNounStem],
        });
    }
    let mut analyses = Vec::new();
    for analysis in &metadata.past_passive_participle {
        let mut lexeme = metadata_verb(metadata);
        lexeme.stems.past_passive_participle = Some(analysis.stem.value.clone());
        lexeme.formations.past_passive_participle = Some(analysis.formation.value);
        analyses.push(metadata_analysis(
            old_church_slavonic_core::verb::verbal_noun(&lexeme, cell)?,
            vec![used(&analysis.stem), used(&analysis.formation)],
        ));
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

fn decline_listed_verbal_noun(
    citations: &FormSet,
    cell: NounCell,
) -> Result<FormSet, InflectionError> {
    let mut variants = Vec::new();
    let mut variant_traces = Vec::new();
    for citation in citations.variants() {
        let noun = NounLexeme {
            lemma: citation.text.clone(),
            class: NounClass::JoNeuterSoft,
            gender: Gender::Neuter,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        };
        let predicted = old_church_slavonic_core::noun::decline(&noun, cell)?;
        let variant = FormVariant {
            text: predicted.text,
            romanization: None,
        };
        if !variants.contains(&variant) {
            variants.push(variant);
            variant_traces.push(
                std::iter::once(RuleStep {
                    rule_id: RuleId::VerbVerbalNoun,
                    before: citations.lemma().to_string(),
                    after: citation.text.clone(),
                    reason: "select the source-listed derived noun as the declensional citation",
                })
                .chain(predicted.trace)
                .collect::<Vec<_>>(),
            );
        }
    }
    if variants.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "a listed verbal noun unexpectedly had no citation spelling".to_string(),
        });
    }

    let mut evidence = citations
        .analyses()
        .iter()
        .flat_map(|analysis| analysis.evidence.iter().cloned())
        .collect::<Vec<_>>();
    for (index, citation) in citations.variants().enumerate() {
        evidence.push(MetadataEvidence {
            field: Some(MetadataField::VerbalNounStem),
            provenance: match citations.source() {
                FormSource::ManualOverride => MetadataProvenance::CuratedGrammarOverride,
                _ => MetadataProvenance::ExactDictionaryTableCell,
            },
            source_feature: Some(format!("verb:verbal-noun:variant:{index}")),
            source_form: Some(citation.text.clone()),
            crosscheck_features: Vec::new(),
            authority: Some(
                match citations.source() {
                    FormSource::ManualOverride => "data/overrides.tsv",
                    _ => "wiktionary-kaikki-2026-07-06",
                }
                .to_string(),
            ),
        });
    }
    evidence.push(MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::ProductiveRuleOutput,
        source_feature: Some(RuleId::VerbVerbalNoun.code().to_string()),
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some(
            "UT OCS Online lesson 8 §36; Polivanova 2023 §§407, 483, and 865".to_string(),
        ),
    });
    let trace = if variant_traces.len() == 1 {
        variant_traces.remove(0)
    } else {
        Vec::new()
    };
    let analysis = FormAnalysis {
        variants,
        source: FormSource::DictionaryMetadataRule {
            rule_id: RuleId::VerbVerbalNoun,
        },
        evidence,
        trace,
    };
    let mut result = metadata_form_set(citations.lemma(), vec![analysis])?;
    for warning in citations.warnings() {
        if !result.warnings().contains(warning) {
            result.add_warning(warning.clone());
        }
    }
    Ok(result)
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
    ReviewedGrammar(&'static str),
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
        FormSourceKind::ReviewedGrammar(_) => FormSource::ReviewedGrammarTable {
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
            FormSourceKind::ReviewedGrammar(_) => MetadataProvenance::ReviewedGrammarTable,
        },
        source_feature: None,
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: match kind {
            FormSourceKind::ReviewedGrammar(authority) => Some(authority.to_string()),
            _ => None,
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_locked_verbal_noun_citation_seeds_all_twenty_one_cells() {
        let rows = crate::dictionary::FORMS
            .iter()
            .filter(|row| row.feature == "verb:verbal-noun")
            .collect::<Vec<_>>();
        let ids = rows
            .iter()
            .map(|row| row.lexeme_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(rows.len(), 191, "locked exact verbal-noun row count");
        assert_eq!(ids.len(), 191, "locked verbal-noun identity count");

        for row in rows {
            for cell in NounCell::all() {
                let forms = verbal_noun_form_by_id(row.lexeme_id, cell)
                    .unwrap_or_else(|error| panic!("{} {cell:?}: {error}", row.lexeme_id));
                let expected = old_church_slavonic_core::noun::decline(
                    &NounLexeme {
                        lemma: row.form.to_string(),
                        class: NounClass::JoNeuterSoft,
                        gender: Gender::Neuter,
                        animacy: Animacy::Inanimate,
                        number_restriction: NumberRestriction::All,
                    },
                    cell,
                )
                .expect("source-listed citation is a valid soft-neuter noun");
                assert!(
                    forms.texts().any(|text| text == expected.text),
                    "{} {cell:?}: expected {} in {:?}",
                    row.lexeme_id,
                    expected.text,
                    forms.texts().collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn every_dictionary_past_passive_platform_reproduces_its_exact_verbal_noun() {
        let ids = crate::dictionary::FORMS
            .iter()
            .filter(|row| row.feature == "verb:verbal-noun")
            .map(|row| row.lexeme_id)
            .collect::<BTreeSet<_>>();
        let citation = verbal_noun_citation_cell();
        let mut checked = 0;
        let mut exact_spelling = 0;
        let mut retained_jer_spelling = 0;
        for id in ids {
            let metadata = verb_metadata_by_id(id).expect("validated generated metadata");
            if metadata.past_passive_participle.is_empty() {
                continue;
            }
            let exact = lookup::table_form(id, "verb:verbal-noun").expect("locked exact citation");
            let generated = generate_verbal_noun_from_metadata(&metadata, citation)
                .expect("complete past-passive platform");
            let generated = generated.texts().collect::<Vec<_>>();
            let exact = exact.texts().collect::<Vec<_>>();
            if generated == exact {
                exact_spelling += 1;
            } else {
                assert_eq!(generated.len(), exact.len(), "{id}");
                for (productive, listed) in generated.into_iter().zip(exact) {
                    assert_eq!(
                        productive.strip_suffix("иѥ"),
                        listed.strip_suffix("ьѥ"),
                        "{id}: only the source-licensed tense-jer realization may differ"
                    );
                }
                retained_jer_spelling += 1;
            }
            checked += 1;
        }
        assert_eq!(checked, 134, "locked independently crosschecked platforms");
        assert_eq!(exact_spelling, 117, "locked canonical -иѥ spellings");
        assert_eq!(retained_jer_spelling, 17, "locked retained -ьѥ spellings");
    }

    #[test]
    fn exact_source_diacritics_remain_bounded_dictionary_spellings() {
        let acute = crate::dictionary::FORMS
            .iter()
            .filter(|row| row.form.contains('\u{0301}'))
            .collect::<Vec<_>>();
        let breathing = crate::dictionary::FORMS
            .iter()
            .filter(|row| row.form.contains('\u{0486}'))
            .collect::<Vec<_>>();
        assert_eq!(acute.len(), 231, "locked exact acute-marked rows");
        assert_eq!(
            acute
                .iter()
                .map(|row| row.lexeme_id)
                .collect::<BTreeSet<_>>()
                .len(),
            11,
            "locked acute-marked identities"
        );
        assert_eq!(breathing.len(), 21, "locked exact psili-marked rows");
        assert_eq!(
            breathing
                .iter()
                .map(|row| row.lexeme_id)
                .collect::<BTreeSet<_>>()
                .len(),
            1,
            "locked psili-marked identity"
        );
        assert!(crate::dictionary::FORMS.iter().all(|row| {
            !row.form.chars().any(|character| {
                matches!(
                    character,
                    '\u{0300}'
                        | '\u{0311}'
                        | '\u{0485}'
                        | '\u{0400}'
                        | '\u{0450}'
                        | '\u{040d}'
                        | '\u{045d}'
                )
            })
        }));
    }
}
