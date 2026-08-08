//! Dictionary lookup and generated-cell access.

use crate::dictionary::{ALIASES, FORMS, LEXEMES, LexemeRecord};
use old_church_slavonic_core::{
    FormSet, FormSource, FormVariant, InflectionError, InflectionWarning, LexemeSummary,
    PartOfSpeech,
};

pub fn lookup(
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> Result<Vec<LexemeSummary>, InflectionError> {
    let key = old_church_slavonic_core::orthography::lookup_key(lemma)?;
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
        [] => Err(InflectionError::UnknownLemma),
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
    let rows = &FORMS[start..end];
    if rows.is_empty() {
        return None;
    }
    let variants = rows
        .iter()
        .enumerate()
        .map(|(expected_rank, row)| {
            debug_assert_eq!(usize::from(row.rank), expected_rank);
            FormVariant {
                text: row.form.to_string(),
                romanization: (!row.romanization.is_empty()).then(|| row.romanization.to_string()),
            }
        })
        .collect::<Vec<_>>();
    let warnings = if variants.len() > 1 {
        vec![InflectionWarning::MultipleDictionaryVariants]
    } else {
        Vec::new()
    };
    Some(FormSet {
        lemma: lexeme.lemma.to_string(),
        variants,
        source: if rows[0].is_override {
            FormSource::ManualOverride
        } else {
            FormSource::DictionaryTable
        },
        warnings,
        trace: Vec::new(),
    })
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
