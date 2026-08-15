//! Dictionary lookup and generated-cell access.

use crate::dictionary::{ALIASES, FORMS, LEXEMES, LexemeRecord, OVERRIDES};
use old_church_slavonic_core::{
    FormAnalysis, FormSet, FormSource, FormVariant, InflectionError, InflectionWarning,
    LexemeSummary, MetadataEvidence, MetadataProvenance, PartOfSpeech,
};

/// Return every dictionary identity matching one normalized lemma and part of speech.
///
/// ```
/// use old_church_slavonic::{lookup, PartOfSpeech};
/// let candidates = lookup("обѣдъ", PartOfSpeech::Noun)?;
/// assert_eq!(candidates.len(), 1);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn lookup(
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> Result<Vec<LexemeSummary>, InflectionError> {
    let lemma = old_church_slavonic_core::Lemma::parse(lemma)?;
    let key = old_church_slavonic_core::orthography::lookup_key(lemma.as_str())?;
    let start = ALIASES.partition_point(|alias| alias.key < key.as_str());
    let end = ALIASES.partition_point(|alias| alias.key <= key.as_str());
    let mut ids: Vec<&str> = ALIASES[start..end]
        .iter()
        .filter(|alias| alias.key == key)
        .map(|alias| alias.lexeme_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids
        .into_iter()
        .filter_map(find_lexeme)
        .filter(|record| record.pos == part_of_speech.code())
        .map(summary)
        .collect())
}

pub(crate) fn resolve_one(
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> Result<&'static LexemeRecord, InflectionError> {
    let candidates = lookup(lemma, part_of_speech)?;
    match candidates.as_slice() {
        [] => Err(InflectionError::unknown_lemma(lemma, part_of_speech)),
        [one] => find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
            reason: "generated alias points at a missing lexeme".to_string(),
        }),
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub(crate) fn find_lexeme(id: &str) -> Option<&'static LexemeRecord> {
    LEXEMES
        .binary_search_by_key(&id, |record| record.id)
        .ok()
        .map(|index| &LEXEMES[index])
}

pub(crate) fn table_form(id: &str, feature: &str) -> Option<FormSet> {
    let lexeme = find_lexeme(id)?;
    let requested = (id, feature);
    let start = FORMS.partition_point(|record| (record.lexeme_id, record.feature) < requested);
    let end = FORMS.partition_point(|record| (record.lexeme_id, record.feature) <= requested);
    let source = FormSource::DictionaryTable;
    let evidence = vec![MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::ExactDictionaryTableCell,
        source_feature: Some(feature.to_string()),
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some("wiktionary-kaikki-2026-07-06".to_string()),
    }];
    listed_form(
        lexeme.lemma,
        FORMS[start..end]
            .iter()
            .map(|row| (row.rank, row.form, row.romanization)),
        source,
        evidence,
        Vec::new(),
    )
}

pub(crate) fn override_form(id: &str, feature: &str) -> Option<FormSet> {
    let lexeme = find_lexeme(id)?;
    let requested = (id, feature);
    let start = OVERRIDES.partition_point(|record| (record.lexeme_id, record.feature) < requested);
    let end = OVERRIDES.partition_point(|record| (record.lexeme_id, record.feature) <= requested);
    let rows = OVERRIDES.get(start..end)?;
    let first = rows.first()?;
    let evidence = vec![MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::CuratedGrammarOverride,
        source_feature: Some(feature.to_string()),
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some(first.authority.to_string()),
    }];
    let trace = vec![old_church_slavonic_core::RuleStep {
        rule_id: old_church_slavonic_core::RuleId::VerbDictionaryMetadata,
        before: first.reason.to_string(),
        after: first.form.to_string(),
        reason: "apply a reviewed cell-specific override after exact-table lookup",
    }];
    listed_form(
        lexeme.lemma,
        rows.iter()
            .map(|row| (row.rank, row.form, row.romanization)),
        FormSource::ManualOverride,
        evidence,
        trace,
    )
}

fn listed_form<'a>(
    lemma: &str,
    rows: impl IntoIterator<Item = (u16, &'a str, &'a str)>,
    source: FormSource,
    evidence: Vec<MetadataEvidence>,
    trace: Vec<old_church_slavonic_core::RuleStep>,
) -> Option<FormSet> {
    let mut variants =
        rows.into_iter()
            .enumerate()
            .map(|(expected_rank, (rank, form, romanization))| {
                debug_assert_eq!(usize::from(rank), expected_rank);
                FormVariant {
                    text: form.into(),
                    romanization: (!romanization.is_empty()).then(|| romanization.into()),
                }
            });
    let primary = variants.next()?;
    let variants = variants.collect::<Vec<_>>();
    let warnings = (!variants.is_empty())
        .then_some(InflectionWarning::MultipleDictionaryVariants)
        .into_iter()
        .collect();
    let analysis_variants = std::iter::once(primary.clone())
        .chain(variants.iter().cloned())
        .collect();
    let analyses = vec![FormAnalysis {
        variants: analysis_variants,
        source: source.clone(),
        evidence,
        trace: trace.clone(),
    }];
    Some(FormSet::new(
        lemma, primary, variants, source, warnings, trace, analyses,
    ))
}

pub(crate) fn table_paradigm(id: &str) -> Option<Vec<(String, FormSet)>> {
    find_lexeme(id)?;
    let start = FORMS.partition_point(|record| record.lexeme_id < id);
    let end = FORMS.partition_point(|record| record.lexeme_id <= id);
    let mut out = Vec::new();
    let mut index = start;
    while index < end {
        let feature = FORMS[index].feature;
        let forms = table_form(id, feature)?;
        out.push((feature.to_string(), forms));
        index += FORMS[index..end].partition_point(|record| record.feature == feature);
    }
    Some(out)
}

pub(crate) fn summary(record: &LexemeRecord) -> LexemeSummary {
    LexemeSummary {
        id: record.id.to_string(),
        lemma: record.lemma.to_string(),
        lookup_key: record.key.to_string(),
        part_of_speech: parse_pos(record.pos).unwrap_or(PartOfSpeech::Noun),
        class: (!record.class.is_empty()).then(|| record.class.to_string()),
    }
}

pub(crate) fn parse_pos(value: &str) -> Option<PartOfSpeech> {
    match value {
        "noun" => Some(PartOfSpeech::Noun),
        "adj" => Some(PartOfSpeech::Adjective),
        "verb" => Some(PartOfSpeech::Verb),
        "pron" => Some(PartOfSpeech::Pronoun),
        "num" => Some(PartOfSpeech::Numeral),
        "det" => Some(PartOfSpeech::Determiner),
        _ => None,
    }
}
