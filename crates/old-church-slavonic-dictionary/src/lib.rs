//! Offline Old Church Slavonic meaning lookup and game-vocabulary validation.
//!
//! The dictionary keeps every Wiktionary sense independent and exposes its
//! source ID. Search results are ranked suggestions, never automatic
//! translations.

#![forbid(unsafe_code)]

mod records;
mod vocabulary;

use old_church_slavonic_core::orthography;
use records::SENSES;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

pub use vocabulary::{
    VocabularyIssue, VocabularyIssueLevel, VocabularyReport, validate_vocabulary_tsv,
};

/// The exact machine-readable source represented by this release.
pub const SOURCE_NAME: &str = "English Wiktionary Old Church Slavonic via Kaikki";
pub const SOURCE_LICENSE: &str = "CC BY-SA 4.0";
pub const SOURCE_URL: &str = "https://kaikki.org/dictionary/Old%20Church%20Slavonic/kaikki.org-dictionary-OldChurchSlavonic.jsonl";
/// Complete pinned dump identity bundled in the published crate.
pub const SOURCE_MANIFEST: &str = include_str!("../SOURCE.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictionaryError {
    EmptyQuery,
    QueryTooLong,
    InvalidQuery(String),
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuery => formatter.write_str("the dictionary query is empty"),
            Self::QueryTooLong => {
                formatter.write_str("the dictionary query exceeds 256 characters")
            }
            Self::InvalidQuery(reason) => write!(formatter, "invalid dictionary query: {reason}"),
        }
    }
}

impl Error for DictionaryError {}

#[derive(Debug, Clone, Copy)]
pub struct Example {
    record: &'static records::ExampleRecord,
}

impl Example {
    pub fn text(self) -> &'static str {
        self.record.text
    }

    pub fn romanization(self) -> Option<&'static str> {
        nonempty(self.record.romanization)
    }

    pub fn translation(self) -> Option<&'static str> {
        nonempty(self.record.translation)
    }

    pub fn reference(self) -> Option<&'static str> {
        nonempty(self.record.reference)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sense {
    record: &'static records::SenseRecord,
}

impl Sense {
    pub fn id(self) -> &'static str {
        self.record.id
    }

    pub fn source_sense_id(self) -> Option<&'static str> {
        nonempty(self.record.source_sense_id)
    }

    pub fn lemma(self) -> &'static str {
        self.record.lemma
    }

    pub fn source_spelling(self) -> &'static str {
        self.record.page_word
    }

    pub fn part_of_speech(self) -> &'static str {
        self.record.part_of_speech
    }

    pub fn inflection_lexeme_id(self) -> Option<&'static str> {
        self.record.inflection_lexeme_id
    }

    pub fn glosses(self) -> &'static [&'static str] {
        self.record.glosses
    }

    pub fn raw_glosses(self) -> &'static [&'static str] {
        self.record.raw_glosses
    }

    pub fn tags(self) -> &'static [&'static str] {
        self.record.tags
    }

    pub fn topics(self) -> &'static [&'static str] {
        self.record.topics
    }

    pub fn examples(self) -> impl ExactSizeIterator<Item = Example> {
        self.record.examples.iter().map(|record| Example { record })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchKind {
    Lemma,
    ExactGloss,
    GlossPhrase,
    GlossWords,
    Topic,
}

#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    sense: Sense,
    score: u32,
    matched_on: MatchKind,
}

impl SearchResult {
    pub fn sense(self) -> Sense {
        self.sense
    }

    pub fn score(self) -> u32 {
        self.score
    }

    pub fn matched_on(self) -> MatchKind {
        self.matched_on
    }
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub part_of_speech: Option<String>,
    pub topic: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            part_of_speech: None,
            topic: None,
        }
    }
}

/// Look up a canonical lemma or the spelling used as its Wiktionary page title.
pub fn lookup(lemma: &str) -> Result<Vec<Sense>, DictionaryError> {
    let key = orthography::lookup_key(lemma)
        .map_err(|error| DictionaryError::InvalidQuery(error.to_string()))?;
    Ok(SENSES
        .iter()
        .filter(|record| record.key == key || record.page_key == key)
        .map(|record| Sense { record })
        .collect())
}

pub fn sense_by_id(id: &str) -> Option<Sense> {
    SENSES
        .iter()
        .find(|record| record.id == id)
        .map(|record| Sense { record })
}

/// Search English glosses and OCS lemmas, returning explainable ranked senses.
pub fn search(query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>, DictionaryError> {
    let normalized = normalize_query(query)?;
    let words = normalized.split_whitespace().collect::<Vec<_>>();
    let part_of_speech = options
        .part_of_speech
        .as_deref()
        .map(normalize_part_of_speech);
    let topic = options.topic.as_deref().map(normalize_ascii_label);
    let ocs_key = (!query.chars().any(char::is_whitespace))
        .then(|| orthography::lookup_key(query).ok())
        .flatten();
    let mut results = SENSES
        .iter()
        .filter(|record| {
            part_of_speech
                .as_deref()
                .is_none_or(|wanted| record.part_of_speech == wanted)
        })
        .filter(|record| {
            topic.as_deref().is_none_or(|wanted| {
                record
                    .topics
                    .iter()
                    .any(|candidate| normalize_ascii_label(candidate) == wanted)
            })
        })
        .filter_map(|record| {
            score_record(record, &normalized, &words, ocs_key.as_deref()).map(
                |(score, matched_on)| SearchResult {
                    sense: Sense { record },
                    score,
                    matched_on,
                },
            )
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.sense.lemma().cmp(right.sense.lemma()))
            .then_with(|| {
                left.sense
                    .part_of_speech()
                    .cmp(right.sense.part_of_speech())
            })
            .then_with(|| left.sense.id().cmp(right.sense.id()))
    });
    results.truncate(options.limit);
    Ok(results)
}

pub fn sense_count() -> usize {
    SENSES.len()
}

#[derive(Debug, Clone)]
pub struct TokenAnalysis {
    pub senses: Vec<Sense>,
    pub forms: Vec<DictionaryFormMatch>,
    pub generated_forms: Vec<DictionaryFormMatch>,
    pub examples: Vec<ExampleTokenMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryFormMatch {
    pub lexeme_id: String,
    pub lemma: String,
    pub part_of_speech: old_church_slavonic::PartOfSpeech,
    pub feature: String,
    pub form: String,
    pub romanization: Option<String>,
}

static FORM_INDEX: OnceLock<BTreeMap<String, Vec<DictionaryFormMatch>>> = OnceLock::new();
static GENERATED_FORM_INDEX: OnceLock<BTreeMap<String, Vec<DictionaryFormMatch>>> = OnceLock::new();
static EXAMPLE_TOKEN_INDEX: OnceLock<BTreeMap<String, Vec<ExampleTokenMatch>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleTokenMatch {
    pub lemma: String,
    pub sense_id: String,
    pub reference: Option<String>,
}

/// Reverse-analyze an exact source-table form belonging to a semantic entry.
///
/// Productive predictions are deliberately excluded: a match means that the
/// pinned inflection snapshot contains this exact surface string.
pub fn analyze_dictionary_form(form: &str) -> Result<Vec<DictionaryFormMatch>, DictionaryError> {
    let key = orthography::lookup_key(form)
        .map_err(|error| DictionaryError::InvalidQuery(error.to_string()))?;
    Ok(form_index().get(&key).cloned().unwrap_or_default())
}

fn form_index() -> &'static BTreeMap<String, Vec<DictionaryFormMatch>> {
    FORM_INDEX.get_or_init(|| {
        let mut index: BTreeMap<String, Vec<DictionaryFormMatch>> = BTreeMap::new();
        let mut lexeme_ids = BTreeSet::new();
        for sense in SENSES {
            if let Some(id) = sense.inflection_lexeme_id {
                lexeme_ids.insert(id.to_string());
            }
            let Some(part_of_speech) = inflection_part_of_speech(sense.part_of_speech) else {
                continue;
            };
            for spelling in [sense.lemma, sense.page_word] {
                if let Ok(candidates) = old_church_slavonic::lookup(spelling, part_of_speech) {
                    lexeme_ids.extend(candidates.into_iter().map(|candidate| candidate.id));
                }
            }
        }
        for lexeme_id in lexeme_ids {
            let Ok(paradigm) =
                old_church_slavonic::advanced::raw_features::dictionary_paradigm_by_id(&lexeme_id)
            else {
                continue;
            };
            for (feature, forms) in paradigm.iter() {
                for variant in forms.variants() {
                    let Ok(key) = orthography::lookup_key(&variant.text) else {
                        continue;
                    };
                    index.entry(key).or_default().push(DictionaryFormMatch {
                        lexeme_id: paradigm.id().to_string(),
                        lemma: paradigm.lemma().to_string(),
                        part_of_speech: paradigm.part_of_speech(),
                        feature: feature.to_string(),
                        form: variant.text.clone(),
                        romanization: variant.romanization.clone(),
                    });
                }
            }
        }
        for matches in index.values_mut() {
            matches.sort_by(|left, right| {
                (&left.lexeme_id, &left.feature, &left.form).cmp(&(
                    &right.lexeme_id,
                    &right.feature,
                    &right.form,
                ))
            });
            matches.dedup();
        }
        index
    })
}

/// Reverse-analyze a form generated by the inflection engine from a pinned
/// dictionary identity and its source-backed metadata.
pub fn analyze_generated_form(form: &str) -> Result<Vec<DictionaryFormMatch>, DictionaryError> {
    let key = orthography::lookup_key(form)
        .map_err(|error| DictionaryError::InvalidQuery(error.to_string()))?;
    Ok(generated_form_index()
        .get(&key)
        .cloned()
        .unwrap_or_default())
}

fn generated_form_index() -> &'static BTreeMap<String, Vec<DictionaryFormMatch>> {
    GENERATED_FORM_INDEX.get_or_init(|| {
        use old_church_slavonic::PartOfSpeech;
        use old_church_slavonic::advanced::by_id;

        let mut identities = BTreeMap::new();
        for sense in SENSES {
            if let (Some(id), Some(part_of_speech)) = (
                sense.inflection_lexeme_id,
                inflection_part_of_speech(sense.part_of_speech),
            ) {
                identities
                    .entry(id.to_string())
                    .or_insert((sense.lemma.to_string(), part_of_speech));
            }
        }

        let mut index: BTreeMap<String, Vec<DictionaryFormMatch>> = BTreeMap::new();
        for (id, (lemma, part_of_speech)) in identities {
            let mut add = |feature: String, forms: &old_church_slavonic::FormSet| {
                for variant in forms.variants() {
                    let Ok(key) = orthography::lookup_key(&variant.text) else {
                        continue;
                    };
                    index.entry(key).or_default().push(DictionaryFormMatch {
                        lexeme_id: id.clone(),
                        lemma: lemma.clone(),
                        part_of_speech,
                        feature: feature.clone(),
                        form: variant.text.clone(),
                        romanization: variant.romanization.clone(),
                    });
                }
            };
            match part_of_speech {
                PartOfSpeech::Noun => {
                    if let Ok(paradigm) = by_id::noun_paradigm_by_id(&id) {
                        for outcome in paradigm.iter() {
                            if let Ok(forms) = &outcome.result {
                                add(outcome.cell.key(), forms);
                            }
                        }
                    }
                }
                PartOfSpeech::Adjective => {
                    if let Ok(paradigm) = by_id::adjective_paradigm_by_id(&id) {
                        for outcome in paradigm.iter() {
                            if let Ok(forms) = &outcome.result {
                                add(outcome.cell.key(), forms);
                            }
                        }
                    }
                }
                PartOfSpeech::Verb => {
                    if let Ok(paradigm) = by_id::finite_paradigm_by_id(&id) {
                        for outcome in paradigm.iter() {
                            if let Ok(forms) = &outcome.result {
                                add(outcome.cell.key(), forms);
                            }
                        }
                    }
                    if let Ok(paradigm) = by_id::imperative_paradigm_by_id(&id) {
                        for outcome in paradigm.iter() {
                            if let Ok(forms) = &outcome.result {
                                add(outcome.cell.key(), forms);
                            }
                        }
                    }
                    if let Ok(paradigm) = by_id::l_participle_paradigm_by_id(&id) {
                        for outcome in paradigm.iter() {
                            if let Ok(forms) = &outcome.result {
                                add(outcome.cell.key(), forms);
                            }
                        }
                    }
                    for kind in [
                        old_church_slavonic::ParticipleKind::PresentActive,
                        old_church_slavonic::ParticipleKind::PresentPassive,
                        old_church_slavonic::ParticipleKind::PastActive,
                        old_church_slavonic::ParticipleKind::PastPassive,
                    ] {
                        if let Ok(paradigm) = by_id::participle_paradigm_by_id(&id, kind) {
                            for outcome in paradigm.iter() {
                                if let Ok(forms) = &outcome.result {
                                    add(outcome.cell.key(), forms);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for matches in index.values_mut() {
            matches.sort_by(|left, right| {
                (&left.lexeme_id, &left.feature, &left.form).cmp(&(
                    &right.lexeme_id,
                    &right.feature,
                    &right.form,
                ))
            });
            matches.dedup();
        }
        index
    })
}

/// Find exact tokens that occur in the pinned Wiktionary example corpus.
pub fn analyze_example_token(token: &str) -> Result<Vec<ExampleTokenMatch>, DictionaryError> {
    let key = orthography::lookup_key(token)
        .map_err(|error| DictionaryError::InvalidQuery(error.to_string()))?;
    Ok(example_token_index().get(&key).cloned().unwrap_or_default())
}

fn example_token_index() -> &'static BTreeMap<String, Vec<ExampleTokenMatch>> {
    EXAMPLE_TOKEN_INDEX.get_or_init(|| {
        let mut index: BTreeMap<String, Vec<ExampleTokenMatch>> = BTreeMap::new();
        for sense in SENSES {
            for example in sense.examples {
                for (token, _, _) in word_tokens(example.text) {
                    let Ok(key) = orthography::lookup_key(&token) else {
                        continue;
                    };
                    index.entry(key).or_default().push(ExampleTokenMatch {
                        lemma: sense.lemma.to_string(),
                        sense_id: sense.id.to_string(),
                        reference: (!example.reference.is_empty())
                            .then(|| example.reference.to_string()),
                    });
                }
            }
        }
        for matches in index.values_mut() {
            matches.sort_by(|left, right| {
                (&left.lemma, &left.sense_id, &left.reference).cmp(&(
                    &right.lemma,
                    &right.sense_id,
                    &right.reference,
                ))
            });
            matches.dedup();
        }
        index
    })
}

fn inflection_part_of_speech(value: &str) -> Option<old_church_slavonic::PartOfSpeech> {
    use old_church_slavonic::PartOfSpeech;
    match value {
        "adjective" => Some(PartOfSpeech::Adjective),
        "determiner" => Some(PartOfSpeech::Determiner),
        "noun" | "proper-name" => Some(PartOfSpeech::Noun),
        "numeral" => Some(PartOfSpeech::Numeral),
        "pronoun" => Some(PartOfSpeech::Pronoun),
        "verb" => Some(PartOfSpeech::Verb),
        _ => None,
    }
}

impl TokenAnalysis {
    pub fn is_known(&self) -> bool {
        !self.senses.is_empty()
            || !self.forms.is_empty()
            || !self.generated_forms.is_empty()
            || !self.examples.is_empty()
    }
}

pub fn analyze_token(token: &str) -> Result<TokenAnalysis, DictionaryError> {
    let senses = lookup(token)?;
    let forms = analyze_dictionary_form(token)?;
    let generated_forms = analyze_generated_form(token)?;
    let examples = analyze_example_token(token)?;
    Ok(TokenAnalysis {
        senses,
        forms,
        generated_forms,
        examples,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextTokenStatus {
    Citation,
    Inflected,
    Generated,
    ExampleAttested,
    Allowlisted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextTokenAnalysis {
    pub token: String,
    pub line: usize,
    pub column: usize,
    pub status: TextTokenStatus,
    pub lemmas: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextReport {
    pub total_tokens: usize,
    pub unique_tokens: usize,
    pub unknown_tokens: usize,
    pub analyses: Vec<TextTokenAnalysis>,
}

/// Analyze distinct word tokens in rendered OCS text against dictionary
/// citations and exact source-table forms.
pub fn check_text(text: &str, allowlisted: &BTreeSet<String>) -> TextReport {
    let mut total_tokens = 0;
    let mut first_occurrences: BTreeMap<String, (String, usize, usize)> = BTreeMap::new();
    for (token, line, column) in word_tokens(text) {
        total_tokens += 1;
        let key = orthography::lookup_key(&token).unwrap_or_else(|_| token.to_lowercase());
        first_occurrences
            .entry(key)
            .or_insert((token, line, column));
    }

    let normalized_allowlist = allowlisted
        .iter()
        .filter_map(|word| orthography::lookup_key(word).ok())
        .collect::<BTreeSet<_>>();
    let mut analyses = Vec::new();
    for (key, (token, line, column)) in first_occurrences {
        let analysis = analyze_token(&token).unwrap_or(TokenAnalysis {
            senses: Vec::new(),
            forms: Vec::new(),
            generated_forms: Vec::new(),
            examples: Vec::new(),
        });
        let status = if !analysis.senses.is_empty() {
            TextTokenStatus::Citation
        } else if !analysis.forms.is_empty() {
            TextTokenStatus::Inflected
        } else if !analysis.generated_forms.is_empty() {
            TextTokenStatus::Generated
        } else if !analysis.examples.is_empty() {
            TextTokenStatus::ExampleAttested
        } else if normalized_allowlist.contains(&key) {
            TextTokenStatus::Allowlisted
        } else {
            TextTokenStatus::Unknown
        };
        let mut lemmas = analysis
            .senses
            .iter()
            .map(|sense| sense.lemma().to_string())
            .chain(analysis.forms.iter().map(|form| form.lemma.clone()))
            .chain(
                analysis
                    .generated_forms
                    .iter()
                    .map(|form| form.lemma.clone()),
            )
            .chain(
                analysis
                    .examples
                    .iter()
                    .map(|example| example.lemma.clone()),
            )
            .collect::<Vec<_>>();
        lemmas.sort();
        lemmas.dedup();
        let mut features = analysis
            .forms
            .iter()
            .chain(analysis.generated_forms.iter())
            .map(|form| form.feature.clone())
            .collect::<Vec<_>>();
        features.sort();
        features.dedup();
        analyses.push(TextTokenAnalysis {
            token,
            line,
            column,
            status,
            lemmas,
            features,
        });
    }
    let unknown_tokens = analyses
        .iter()
        .filter(|analysis| analysis.status == TextTokenStatus::Unknown)
        .count();
    TextReport {
        total_tokens,
        unique_tokens: analyses.len(),
        unknown_tokens,
        analyses,
    }
}

pub(crate) fn score_sense_for_concept(sense: Sense, concept: &str) -> u32 {
    let Ok(normalized) = normalize_query(concept) else {
        return 0;
    };
    let words = normalized.split_whitespace().collect::<Vec<_>>();
    score_record(sense.record, &normalized, &words, None).map_or(0, |result| result.0)
}

fn score_record(
    record: &records::SenseRecord,
    query: &str,
    words: &[&str],
    ocs_key: Option<&str>,
) -> Option<(u32, MatchKind)> {
    if ocs_key.is_some_and(|key| key == record.key || key == record.page_key) {
        return Some((2_000, MatchKind::Lemma));
    }

    let mut best: Option<(u32, MatchKind)> = None;
    for gloss in record.glosses.iter().chain(record.raw_glosses.iter()) {
        let normalized = normalize_ascii_label(gloss);
        let candidate = if normalized == query {
            Some((1_200, MatchKind::ExactGloss))
        } else if normalized.contains(query) {
            Some((800, MatchKind::GlossPhrase))
        } else {
            let matches = words
                .iter()
                .filter(|word| normalized.split_whitespace().any(|token| token == **word))
                .count();
            if matches == words.len() && matches > 0 {
                Some((
                    600 + u32::try_from(matches).unwrap_or(0) * 20,
                    MatchKind::GlossWords,
                ))
            } else if matches > 0 {
                Some((
                    100 + u32::try_from(matches).unwrap_or(0) * 20,
                    MatchKind::GlossWords,
                ))
            } else {
                None
            }
        };
        if candidate.is_some_and(|candidate| best.is_none_or(|best| candidate.0 > best.0)) {
            best = candidate;
        }
    }
    for label in record.tags.iter().chain(record.topics.iter()) {
        if normalize_ascii_label(label) == query && best.is_none_or(|best| 350 > best.0) {
            best = Some((350, MatchKind::Topic));
        }
    }
    best
}

fn normalize_query(query: &str) -> Result<String, DictionaryError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(DictionaryError::EmptyQuery);
    }
    if query.chars().count() > 256 {
        return Err(DictionaryError::QueryTooLong);
    }
    if query.chars().any(char::is_control) {
        return Err(DictionaryError::InvalidQuery(
            "control characters are not allowed".to_string(),
        ));
    }
    Ok(normalize_ascii_label(query))
}

fn normalize_ascii_label(value: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for character in value.to_lowercase().chars() {
        if character.is_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(character);
        } else {
            pending_space = true;
        }
    }
    out
}

fn normalize_part_of_speech(value: &str) -> String {
    match normalize_ascii_label(value).as_str() {
        "proper name" | "proper noun" | "name" => "proper-name".to_string(),
        normalized => normalized.to_string(),
    }
}

fn nonempty(value: &'static str) -> Option<&'static str> {
    (!value.is_empty()).then_some(value)
}

fn word_tokens(text: &str) -> Vec<(String, usize, usize)> {
    let mut out = Vec::new();
    let mut token = String::new();
    let mut token_line = 1;
    let mut token_column = 1;
    let mut line = 1;
    let mut column = 1;
    for character in text.chars().chain(std::iter::once(' ')) {
        if character.is_alphabetic() || is_combining_mark(character) {
            if token.is_empty() {
                token_line = line;
                token_column = column;
            }
            token.push(character);
        } else if !token.is_empty() {
            out.push((std::mem::take(&mut token), token_line, token_column));
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    out
}

fn is_combining_mark(character: char) -> bool {
    matches!(
        u32::from(character),
        0x0300..=0x036f | 0x0483..=0x0489 | 0x1dc0..=0x1dff | 0x20d0..=0x20ff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_normalization_is_deterministic() {
        assert_eq!(
            normalize_query("  Gold-coin! ").expect("query"),
            "gold coin"
        );
        assert_eq!(normalize_query("\n"), Err(DictionaryError::EmptyQuery));
    }

    #[test]
    fn rendered_text_analysis_distinguishes_citations_forms_and_unknowns() {
        let report = check_text("златикъ златици оузьрѣнѫ безъ notocs", &BTreeSet::new());
        assert_eq!(report.total_tokens, 5);
        assert!(report.analyses.iter().any(|analysis| {
            analysis.token == "златикъ" && analysis.status == TextTokenStatus::Citation
        }));
        assert!(report.analyses.iter().any(|analysis| {
            analysis.token == "златици" && analysis.status == TextTokenStatus::Inflected
        }));
        assert!(report.analyses.iter().any(|analysis| {
            analysis.token == "оузьрѣнѫ" && analysis.status == TextTokenStatus::Generated
        }));
        assert!(report.analyses.iter().any(|analysis| {
            analysis.token == "безъ" && analysis.status == TextTokenStatus::ExampleAttested
        }));
        assert!(report.analyses.iter().any(|analysis| {
            analysis.token == "notocs" && analysis.status == TextTokenStatus::Unknown
        }));
    }

    #[test]
    fn generated_form_index_covers_every_participle_system() {
        let id = "благословити|verb|9d3c95ce56eb87f0";
        for (kind, feature_name) in [
            (
                old_church_slavonic::ParticipleKind::PresentActive,
                "present-active",
            ),
            (
                old_church_slavonic::ParticipleKind::PresentPassive,
                "present-passive",
            ),
            (
                old_church_slavonic::ParticipleKind::PastActive,
                "past-active",
            ),
            (
                old_church_slavonic::ParticipleKind::PastPassive,
                "past-passive",
            ),
        ] {
            let paradigm =
                old_church_slavonic::advanced::by_id::participle_paradigm_by_id(id, kind)
                    .expect("source-backed participle paradigm");
            let form = paradigm
                .iter()
                .find_map(|outcome| outcome.result.as_ref().ok())
                .expect("at least one supported participle cell")
                .primary_text();
            assert!(
                analyze_generated_form(form)
                    .expect("generated-form analysis")
                    .iter()
                    .any(|analysis| analysis.lexeme_id == id
                        && analysis.feature.contains(feature_name)),
                "missing {feature_name} analysis for {form}"
            );
        }
    }
}
