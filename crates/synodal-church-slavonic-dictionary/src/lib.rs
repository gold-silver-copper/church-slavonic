#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    Gender, GrammarCell, ImperativeCell, Inflector, LParticipleCell, LexemeId, LexemeSummary,
    LexicalMetadataSummary, MetadataField, Number, NumeralCell, NumeralKind, PartOfSpeech,
    ParticipleCell, ParticipleTense, ParticipleVoice, Person, PronounCell, Result, abbreviation,
    capabilities_by_id, grammar_cell_registry_keys, lexemes, lexical_metadata,
    missing_metadata_by_id,
};
use synodal_church_slavonic_core::{
    Confidence, FormSource, RecensionMappingId, RuleTrace, SynodalWord, normalize_lookup_accentless,
};

pub mod coverage;

pub use synodal_church_slavonic as morphology;
pub use synodal_church_slavonic_core as core;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawSense(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawExample(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawSemanticAlignment(pub [&'static str; 6]);

include!("../generated/registry.rs");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Sense {
    pub id: String,
    pub gloss: String,
    pub domains: Vec<String>,
    pub source_id: String,
    pub source_recension: String,
    pub semantic_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceExample {
    pub id: String,
    pub lexeme_id: LexemeId,
    pub text: String,
    pub translation: String,
    pub source_id: String,
    pub passage: String,
    pub source_recension: String,
    pub target_recension: String,
    pub partition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub lexeme: LexemeSummary,
    pub senses: Vec<Sense>,
    pub examples: Vec<SourceExample>,
    pub metadata: LexicalMetadataSummary,
    pub capabilities: synodal_church_slavonic::Capabilities,
    pub missing_metadata: Vec<MetadataField>,
}

/// Stable identifier for a reviewed morphological family.
///
/// Reviewed families currently have a one-to-one relationship with a stable
/// Synodal lexeme identity.  Candidate family IDs produced by `xtask` are
/// deliberately not accepted by the runtime dictionary.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FamilyId(String);

impl FamilyId {
    #[must_use]
    pub fn for_lexeme(id: &LexemeId) -> Self {
        Self(format!("family:{}", id.as_str()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn lexeme_id(&self) -> Option<LexemeId> {
        self.0.strip_prefix("family:").map(LexemeId::from)
    }
}

impl std::fmt::Display for FamilyId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for FamilyId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Stable grammar-cell key. This remains a string so the review registry
    /// can expose exact cells not yet supported by productive generation.
    pub cell: String,
    pub expanded: String,
    pub printed: String,
    pub evidence_id: String,
    pub source_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FamilySummary {
    pub id: FamilyId,
    pub lexeme: LexemeSummary,
    pub senses: Vec<Sense>,
    pub members: Vec<FamilyMember>,
    pub class: Option<String>,
    pub stem: Option<String>,
    pub principal_parts: BTreeMap<String, String>,
    pub supported_systems: Vec<String>,
    pub missing_metadata: Vec<MetadataField>,
    /// Family-level requirements not represented by the low-level
    /// `MetadataField` enum (for example a nominal declension class).
    pub missing_family_metadata: Vec<String>,
    pub exact_only: bool,
    pub fully_classed: bool,
}

/// Finds reviewed morphological families by lemma or gloss. Ambiguous
/// homographs are returned independently and never collapsed to one identity.
pub fn families(query: &str) -> Result<Vec<FamilySummary>> {
    let options = SearchOptions {
        limit: usize::MAX,
        fuzzy: false,
        ..SearchOptions::default()
    };
    let mut ids: BTreeSet<LexemeId> = search(query, &options)?
        .into_iter()
        .map(|matched| matched.entry.lexeme.id().clone())
        .collect();
    if let Ok(exact) = lookup_all(query) {
        ids.extend(exact.into_iter().map(|entry| entry.lexeme.id().clone()));
    }
    if let Ok(analyses) = analyze(query) {
        ids.extend(
            analyses
                .into_iter()
                .map(|analysis| analysis.lexeme.id().clone()),
        );
    }
    ids.iter().map(family_for_lexeme).collect()
}

/// Returns one reviewed family by stable ID. Proposed family IDs remain part
/// of the review tooling and fail explicitly here.
pub fn show_family_by_id(id: &FamilyId) -> Result<FamilySummary> {
    let lexeme_id = id.lexeme_id().ok_or_else(|| Error::UnknownLemma {
        lookup: id.as_str().into(),
    })?;
    family_for_lexeme(&lexeme_id)
}

fn family_for_lexeme(id: &LexemeId) -> Result<FamilySummary> {
    let entry = lookup_by_id(id)?;
    let metadata = &entry.metadata;
    let mut members: Vec<_> = metadata
        .exact_forms
        .iter()
        .map(|form| FamilyMember {
            cell: form.cell.clone(),
            expanded: form.expanded.clone(),
            printed: form.printed.clone(),
            evidence_id: form.evidence_id.clone(),
            source_kind: form.source_kind.clone(),
        })
        .collect();
    for accent in &metadata.accents {
        let member = FamilyMember {
            cell: accent.cell.clone(),
            expanded: accent.expanded.clone(),
            printed: accent.accented.clone(),
            evidence_id: accent.evidence_id.clone(),
            source_kind: "accent-table".into(),
        };
        if !members.contains(&member) {
            members.push(member);
        }
    }
    for sense in &entry.senses {
        for contraction in abbreviation::contractions_by_id(id, &sense.id)? {
            members.push(FamilyMember {
                cell: contraction.cell_key,
                expanded: contraction.expanded,
                printed: contraction.printed,
                evidence_id: contraction
                    .evidence_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                source_kind: "abbreviation".into(),
            });
        }
    }
    members.sort_by(|left, right| {
        left.cell
            .cmp(&right.cell)
            .then_with(|| left.printed.cmp(&right.printed))
            .then_with(|| left.evidence_id.cmp(&right.evidence_id))
    });
    let capabilities = &entry.capabilities;
    let supported_systems = capabilities
        .supported_systems()
        .map(str::to_owned)
        .collect();
    let exact_complete_table = metadata.class.as_deref() == Some("exact-complete-pronoun-table");
    let exact_only = metadata
        .class
        .as_deref()
        .is_none_or(|class| matches!(class, "exact" | "exact-complete-pronoun-table"))
        && metadata.principal_parts.is_empty();
    let fully_classed = exact_complete_table
        || (!exact_only
            && match entry.lexeme.part_of_speech() {
                PartOfSpeech::Noun | PartOfSpeech::ProperNoun => {
                    metadata.class.is_some() && metadata.stem.is_some() && metadata.gender.is_some()
                }
                PartOfSpeech::Adjective | PartOfSpeech::Determiner => {
                    metadata.class.is_some() && metadata.stem.is_some()
                }
                PartOfSpeech::Verb => {
                    metadata.class.is_some() && metadata.stem.is_some() && metadata.aspect.is_some()
                }
                _ => true,
            });
    let mut missing_family_metadata = BTreeSet::new();
    if exact_only
        && !exact_complete_table
        && matches!(
            entry.lexeme.part_of_speech(),
            PartOfSpeech::Noun
                | PartOfSpeech::ProperNoun
                | PartOfSpeech::Adjective
                | PartOfSpeech::Determiner
                | PartOfSpeech::Verb
                | PartOfSpeech::Pronoun
                | PartOfSpeech::Numeral
        )
    {
        missing_family_metadata.insert("reviewed-inflection-class-or-exact-complete-table".into());
    }
    if metadata.stem.is_none()
        && matches!(
            entry.lexeme.part_of_speech(),
            PartOfSpeech::Noun
                | PartOfSpeech::ProperNoun
                | PartOfSpeech::Adjective
                | PartOfSpeech::Determiner
                | PartOfSpeech::Verb
        )
    {
        missing_family_metadata.insert("reviewed-stem-and-alternants".into());
    }
    if entry.lexeme.part_of_speech() == PartOfSpeech::Verb && metadata.principal_parts.is_empty() {
        missing_family_metadata.insert("independent-verb-principal-parts".into());
    }
    Ok(FamilySummary {
        id: FamilyId::for_lexeme(id),
        lexeme: entry.lexeme,
        senses: entry.senses,
        members,
        class: metadata.class.clone(),
        stem: metadata.stem.clone(),
        principal_parts: metadata
            .principal_parts
            .iter()
            .map(|part| (part.system.clone(), part.value.clone()))
            .collect(),
        supported_systems,
        missing_metadata: entry.missing_metadata,
        missing_family_metadata: missing_family_metadata.into_iter().collect(),
        exact_only,
        fully_classed,
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchMatchKind {
    ExactLemma,
    LemmaSubstring,
    ExactGloss,
    GlossPhrase,
    GlossWords,
    FuzzyLemma,
    FuzzyGloss,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    pub part_of_speech: Option<PartOfSpeech>,
    pub limit: usize,
    pub fuzzy: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            part_of_speech: None,
            limit: 20,
            fuzzy: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchMatch {
    pub entry: Entry,
    pub score: u16,
    pub matched_on: SearchMatchKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticAlignmentDecision {
    pub mapping_id: String,
    pub source_sense_id: String,
    pub target_sense_id: String,
    pub status: String,
    pub evidence_id: String,
    pub review_note: String,
}

#[must_use]
pub fn semantic_alignments() -> Vec<SemanticAlignmentDecision> {
    SEMANTIC_ALIGNMENTS
        .iter()
        .map(|row| SemanticAlignmentDecision {
            mapping_id: row.0[0].into(),
            source_sense_id: row.0[1].into(),
            target_sense_id: row.0[2].into(),
            status: row.0[3].into(),
            evidence_id: row.0[4].into(),
            review_note: row.0[5].into(),
        })
        .collect()
}

pub fn lookup(lemma: &str) -> Result<Entry> {
    let summary = morphology::lookup(lemma)?;
    entry_for(summary)
}

pub fn lookup_by_id(id: &LexemeId) -> Result<Entry> {
    let summary = morphology::advanced::lookup_by_id(id)?;
    entry_for(summary)
}

/// Returns every target entry matching a normalized lemma. Unlike `lookup`,
/// this operation preserves homographs instead of turning them into an error.
pub fn lookup_all(lemma: &str) -> Result<Vec<Entry>> {
    let parsed = SynodalWord::parse(lemma)?;
    let key = normalize_lookup_accentless(parsed.canonical());
    lexemes()?
        .into_iter()
        .filter(|lexeme| normalize_lookup_accentless(lexeme.lemma()) == key)
        .map(entry_for)
        .collect()
}

pub fn entries() -> Result<Vec<Entry>> {
    lexemes()?.into_iter().map(entry_for).collect()
}

pub fn search(query: &str, options: &SearchOptions) -> Result<Vec<SearchMatch>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(Error::EmptyInput);
    }
    let query_words: BTreeSet<&str> = query.split_whitespace().collect();
    let mut matches = Vec::new();
    for entry in entries()? {
        if options
            .part_of_speech
            .is_some_and(|part_of_speech| entry.lexeme.part_of_speech() != part_of_speech)
        {
            continue;
        }
        let lemma = entry.lexeme.lemma().to_lowercase();
        let mut best = if lemma == query {
            Some((10_000, SearchMatchKind::ExactLemma))
        } else if lemma.contains(&query) {
            Some((8_800, SearchMatchKind::LemmaSubstring))
        } else {
            None
        };
        for sense in &entry.senses {
            let gloss = sense.gloss.to_lowercase();
            let gloss_words: BTreeSet<&str> = gloss
                .split(|character: char| !character.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .collect();
            let candidate = if gloss == query {
                Some((9_800, SearchMatchKind::ExactGloss))
            } else if gloss.contains(&query) {
                Some((9_000, SearchMatchKind::GlossPhrase))
            } else if !query_words.is_empty() && query_words.is_subset(&gloss_words) {
                Some((8_200, SearchMatchKind::GlossWords))
            } else if options.fuzzy {
                fuzzy_score(&query, &gloss).map(|score| (score, SearchMatchKind::FuzzyGloss))
            } else {
                None
            };
            if candidate.is_some_and(|candidate| best.is_none_or(|best| candidate.0 > best.0)) {
                best = candidate;
            }
        }
        if best.is_none() && options.fuzzy {
            best = fuzzy_score(&query, &lemma).map(|score| (score, SearchMatchKind::FuzzyLemma));
        }
        if let Some((score, matched_on)) = best {
            matches.push(SearchMatch {
                entry,
                score,
                matched_on,
            });
        }
    }
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.matched_on.cmp(&right.matched_on))
            .then_with(|| left.entry.lexeme.id().cmp(right.entry.lexeme.id()))
    });
    matches.truncate(options.limit);
    Ok(matches)
}

pub fn search_gloss(query: &str) -> Result<Vec<Entry>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(Error::EmptyInput);
    }
    let matching_ids: BTreeSet<LexemeId> = SENSES
        .iter()
        .filter(|sense| {
            sense.0[2].to_lowercase().contains(&query)
                || sense.0[3]
                    .split(',')
                    .any(|domain| domain.to_lowercase().contains(&query))
        })
        .map(|sense| LexemeId::from(sense.0[0]))
        .collect();
    matching_ids.iter().map(lookup_by_id).collect()
}

pub fn concordance(id: &LexemeId) -> Vec<SourceExample> {
    EXAMPLES
        .iter()
        .filter(|example| example.0[1] == id.as_str())
        .map(source_example)
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AnalysisSource {
    ExactSynodalAttestation,
    SynodalIrregularOverride,
    SynodalNormativeTable,
    SynodalProductiveRule,
    CallerSpecifiedPrediction,
    InheritedPrediction,
    AnalogicalPrediction,
    AbbreviationExpansion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub lexeme: LexemeSummary,
    pub cell: Option<GrammarCell>,
    pub matched_text: String,
    pub source: AnalysisSource,
    pub recension_mapping: Option<RecensionMappingId>,
    pub confidence: Confidence,
    pub evidence_ids: Vec<String>,
    pub assumptions: Vec<String>,
    pub contradictions: Vec<String>,
    pub warnings: Vec<String>,
    pub rule_trace: RuleTrace,
}

/// Returns every compatible curated analysis of an expanded or printed word.
pub fn analyze(word: &str) -> Result<Vec<Analysis>> {
    coverage::default_analyzer()?.analyze_dictionary(word)
}

/// Returns every compatible curated analysis admitted by the caller's
/// generation and orthography policy. The default `analyze` remains Strict;
/// callers must opt into inherited or exploratory predictions explicitly.
pub fn analyze_with(word: &str, inflector: Inflector) -> Result<Vec<Analysis>> {
    if inflector == Inflector::default() {
        return analyze(word);
    }
    coverage::Analyzer::new(inflector)?.analyze_dictionary(word)
}

pub fn lemmatize(word: &str) -> Result<Vec<Entry>> {
    lemmatize_with(word, Inflector::default())
}

pub fn lemmatize_with(word: &str, inflector: Inflector) -> Result<Vec<Entry>> {
    let ids: BTreeSet<LexemeId> = analyze_with(word, inflector)?
        .into_iter()
        .map(|analysis| analysis.lexeme.id().clone())
        .collect();
    ids.iter().map(lookup_by_id).collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VocabularyManifest {
    pub entries: Vec<VocabularyItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub text: String,
    #[serde(default)]
    pub expected_lexeme_id: Option<LexemeId>,
    #[serde(default)]
    pub expected_part_of_speech: Option<PartOfSpeech>,
    #[serde(default)]
    pub required_sense_id: Option<String>,
    #[serde(default)]
    pub requested_cell: Option<GrammarCell>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VocabularyIssueKind {
    InvalidOrthography,
    UnknownVocabulary,
    UnexpectedPartOfSpeech,
    UnexpectedLexeme,
    MissingSemanticIdentity,
    AmbiguousSurfaceForm,
    MissingPrincipalPart,
    UnsupportedFormation,
    MissingOrthographicMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VocabularyIssue {
    pub index: usize,
    pub text: String,
    pub kind: VocabularyIssueKind,
    pub detail: String,
}

#[must_use]
pub fn lint_vocabulary(manifest: &VocabularyManifest) -> Vec<VocabularyIssue> {
    let analyzer = match coverage::default_analyzer() {
        Ok(analyzer) => analyzer,
        Err(error) => {
            let detail = error.to_string();
            return manifest
                .entries
                .iter()
                .enumerate()
                .map(|(index, item)| VocabularyIssue {
                    index,
                    text: item.text.clone(),
                    kind: VocabularyIssueKind::InvalidOrthography,
                    detail: detail.clone(),
                })
                .collect();
        }
    };
    lint_vocabulary_with(&analyzer, manifest)
}

/// Lints a batch with an already constructed analyzer so callers pay the
/// reverse-index cost at most once.
#[must_use]
pub fn lint_vocabulary_with(
    analyzer: &coverage::Analyzer,
    manifest: &VocabularyManifest,
) -> Vec<VocabularyIssue> {
    let mut issues = Vec::new();
    for (index, item) in manifest.entries.iter().enumerate() {
        match analyzer.analyze_dictionary(&item.text) {
            Err(error) => issues.push(VocabularyIssue {
                index,
                text: item.text.clone(),
                kind: VocabularyIssueKind::InvalidOrthography,
                detail: error.to_string(),
            }),
            Ok(analyses) if analyses.is_empty() => issues.push(VocabularyIssue {
                index,
                text: item.text.clone(),
                kind: VocabularyIssueKind::UnknownVocabulary,
                detail: "no curated Synodal analysis".into(),
            }),
            Ok(analyses) => {
                let lexeme_ids: BTreeSet<LexemeId> = analyses
                    .iter()
                    .map(|analysis| analysis.lexeme.id().clone())
                    .collect();
                if lexeme_ids.len() > 1 {
                    issues.push(VocabularyIssue {
                        index,
                        text: item.text.clone(),
                        kind: VocabularyIssueKind::AmbiguousSurfaceForm,
                        detail: format!("{} lexemes are compatible", lexeme_ids.len()),
                    });
                }
                if let Some(expected) = &item.expected_lexeme_id {
                    if !lexeme_ids.contains(expected) {
                        issues.push(VocabularyIssue {
                            index,
                            text: item.text.clone(),
                            kind: VocabularyIssueKind::UnexpectedLexeme,
                            detail: format!("expected lexeme {expected} is not compatible"),
                        });
                    }
                }
                if let Some(expected) = item.expected_part_of_speech {
                    if analyses
                        .iter()
                        .all(|analysis| analysis.lexeme.part_of_speech() != expected)
                    {
                        issues.push(VocabularyIssue {
                            index,
                            text: item.text.clone(),
                            kind: VocabularyIssueKind::UnexpectedPartOfSpeech,
                            detail: format!("no analysis has part of speech {expected:?}"),
                        });
                    }
                }
                if let Some(required) = &item.required_sense_id {
                    let has_sense = lexeme_ids
                        .iter()
                        .any(|id| senses_for(id).iter().any(|sense| sense.id == *required));
                    if !has_sense {
                        issues.push(VocabularyIssue {
                            index,
                            text: item.text.clone(),
                            kind: VocabularyIssueKind::MissingSemanticIdentity,
                            detail: format!("required sense {required:?} is not established"),
                        });
                    }
                }
                if let Some(cell) = item.requested_cell {
                    let ids: Vec<LexemeId> = item
                        .expected_lexeme_id
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .chain(lexeme_ids.iter().cloned())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    let mut errors = Vec::new();
                    let mut supported = false;
                    for id in ids {
                        match analyzer.inflector().form_by_id(&id, cell) {
                            Ok(_) => supported = true,
                            Err(error) => errors.push(error),
                        }
                    }
                    if !supported {
                        let kind = if errors
                            .iter()
                            .any(|error| matches!(error, Error::MissingPrincipalPart { .. }))
                        {
                            VocabularyIssueKind::MissingPrincipalPart
                        } else if errors.iter().any(|error| {
                            matches!(error, Error::OrthographicMetadataRequired { .. })
                        }) {
                            VocabularyIssueKind::MissingOrthographicMetadata
                        } else {
                            VocabularyIssueKind::UnsupportedFormation
                        };
                        issues.push(VocabularyIssue {
                            index,
                            text: item.text.clone(),
                            kind,
                            detail: errors
                                .first()
                                .map_or_else(|| "no compatible lexeme".into(), ToString::to_string),
                        });
                    }
                }
            }
        }
    }
    issues
}

fn entry_for(lexeme: LexemeSummary) -> Result<Entry> {
    let senses = senses_for(lexeme.id());
    if senses.is_empty() {
        return Err(Error::ContradictoryMetadata {
            reason: format!("lexeme {} has no reviewed semantic sense", lexeme.id()),
        });
    }
    let examples = concordance(lexeme.id());
    Ok(Entry {
        metadata: lexical_metadata(lexeme.id())?,
        capabilities: capabilities_by_id(lexeme.id(), Inflector::default())?,
        missing_metadata: missing_metadata_by_id(lexeme.id())?,
        lexeme,
        senses,
        examples,
    })
}

fn senses_for(id: &LexemeId) -> Vec<Sense> {
    SENSES
        .iter()
        .filter(|sense| sense.0[0] == id.as_str())
        .map(|sense| Sense {
            id: sense.0[1].into(),
            gloss: sense.0[2].into(),
            domains: split_list(sense.0[3]),
            source_id: sense.0[4].into(),
            source_recension: sense.0[5].into(),
            semantic_status: sense.0[6].into(),
        })
        .collect()
}

fn source_example(row: &RawExample) -> SourceExample {
    SourceExample {
        id: row.0[0].into(),
        lexeme_id: LexemeId::from(row.0[1]),
        text: row.0[2].into(),
        translation: row.0[3].into(),
        source_id: row.0[4].into(),
        passage: row.0[5].into(),
        source_recension: row.0[6].into(),
        target_recension: row.0[7].into(),
        partition: row.0[8].into(),
    }
}

fn analysis_source(source: &FormSource) -> AnalysisSource {
    match source {
        FormSource::SynodalAttestation { .. } => AnalysisSource::ExactSynodalAttestation,
        FormSource::SynodalIrregularOverride { .. } => AnalysisSource::SynodalIrregularOverride,
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-REGISTRY-NORMATIVE-TABLE" =>
        {
            AnalysisSource::SynodalNormativeTable
        }
        FormSource::SynodalNormativeGeneration { .. } => AnalysisSource::SynodalProductiveRule,
        FormSource::CallerSpecifiedPrediction { .. } => AnalysisSource::CallerSpecifiedPrediction,
        FormSource::InheritedPrediction { .. } => AnalysisSource::InheritedPrediction,
        FormSource::AnalogicalPrediction { .. } => AnalysisSource::AnalogicalPrediction,
    }
}

/// Enumerates cells that reverse analysis may attempt for a part of speech.
/// Unsupported cells still fail through the facade's typed error contract.
#[must_use]
pub fn candidate_cells(part_of_speech: PartOfSpeech) -> Vec<GrammarCell> {
    const OPTIONAL_GENDERS: [Option<Gender>; 4] = [
        None,
        Some(Gender::Masculine),
        Some(Gender::Feminine),
        Some(Gender::Neuter),
    ];
    const OPTIONAL_PERSONS: [Option<Person>; 4] = [
        None,
        Some(Person::First),
        Some(Person::Second),
        Some(Person::Third),
    ];
    let mut cells = match part_of_speech {
        PartOfSpeech::Adverb
        | PartOfSpeech::Preposition
        | PartOfSpeech::Conjunction
        | PartOfSpeech::Particle
        | PartOfSpeech::Interjection => {
            vec![GrammarCell::Indeclinable]
        }
        PartOfSpeech::Noun | PartOfSpeech::ProperNoun => core::NounCell::inventory(&Animacy::ALL)
            .into_iter()
            .map(GrammarCell::Noun)
            .collect(),
        PartOfSpeech::Adjective => {
            AdjectiveCell::inventory(&AdjectiveForm::ALL, &Comparison::ALL, |_| &Animacy::ALL)
                .into_iter()
                .map(GrammarCell::Adjective)
                .collect()
        }
        PartOfSpeech::Verb => verb_cells(),
        PartOfSpeech::Pronoun => PronounCell::inventory(
            &OPTIONAL_GENDERS
                .into_iter()
                .flat_map(|gender| {
                    OPTIONAL_PERSONS
                        .into_iter()
                        .map(move |person| (gender, person))
                })
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(GrammarCell::Pronoun)
        .collect(),
        PartOfSpeech::Numeral => NumeralCell::inventory(&NumeralKind::ALL, &OPTIONAL_GENDERS)
            .into_iter()
            .map(GrammarCell::Numeral)
            .collect(),
        PartOfSpeech::Determiner => {
            AdjectiveCell::inventory(&AdjectiveForm::ALL, &Comparison::ALL, |_| &Animacy::ALL)
                .into_iter()
                .map(GrammarCell::Determiner)
                .collect()
        }
        PartOfSpeech::Participle => Vec::new(),
    };
    cells.push(GrammarCell::LexicalForm);
    cells
}

/// Returns the exact-compatible and productively supported cells that can
/// contribute to reverse analysis for one stable lexeme.
///
/// This is deliberately narrower than [`candidate_cells`], which remains the
/// exhaustive typed inventory used by the independent correctness oracle.
pub fn analysis_cells_by_id(id: &LexemeId, inflector: Inflector) -> Result<Vec<GrammarCell>> {
    let lexeme = morphology::advanced::lookup_by_id(id)?;
    analysis_cells_for_lexeme(&lexeme, inflector)
}

pub(crate) fn analysis_cells_for_lexeme(
    lexeme: &LexemeSummary,
    inflector: Inflector,
) -> Result<Vec<GrammarCell>> {
    let metadata = lexical_metadata(lexeme.id())?;
    let capabilities = capabilities_by_id(lexeme.id(), inflector)?;
    let exact_keys: BTreeSet<&str> = metadata
        .exact_forms
        .iter()
        .map(|form| form.cell.as_str())
        .collect();
    let mut cells = BTreeSet::new();
    for cell in candidate_cells(lexeme.part_of_speech()) {
        if grammar_cell_registry_keys(cell)
            .iter()
            .any(|key| exact_keys.contains(key.as_str()))
            || productive_cell_is_supported(cell, &metadata, &capabilities)
        {
            cells.insert(cell);
        }
    }
    Ok(cells.into_iter().collect())
}

fn productive_cell_is_supported(
    cell: GrammarCell,
    metadata: &LexicalMetadataSummary,
    capabilities: &morphology::Capabilities,
) -> bool {
    let principal_part = |system: &str| {
        metadata
            .principal_parts
            .iter()
            .find(|part| part.system == system)
    };
    match cell {
        GrammarCell::Noun(cell) => {
            capabilities.productive_noun
                && metadata
                    .noun_restriction
                    .as_ref()
                    .is_none_or(|restriction| {
                        number_is_licensed(&restriction.number_inventory, cell.number)
                    })
        }
        GrammarCell::Adjective(cell) => {
            capabilities.productive_adjective
                && adjectival_cell_is_supported(cell, principal_part("comparative-stem").is_some())
        }
        GrammarCell::Determiner(cell) => {
            capabilities.productive_determiner
                && determiner_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::Pronoun(cell) => {
            capabilities.productive_pronoun
                && pronoun_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::Numeral(cell) => {
            capabilities.productive_numeral
                && numeral_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::FiniteVerb(cell) if productive_verb_class(metadata.class.as_deref()) => {
            match cell.tense {
                FiniteTense::Present => {
                    metadata.stem.is_some()
                        && ["present-first-singular", "present-third-plural"]
                            .into_iter()
                            .all(|system| principal_part(system).is_some())
                }
                FiniteTense::Imperfect => {
                    matches!(
                        metadata.aspect.as_deref(),
                        Some("imperfective" | "biaspectual")
                    ) && principal_part("imperfect-stem")
                        .and_then(|part| part.formation.as_deref())
                        .is_some_and(|formation| formation != "irregular")
                }
                FiniteTense::Aorist => principal_part("aorist-stem")
                    .and_then(|part| part.formation.as_deref())
                    .is_some_and(|formation| formation != "irregular"),
                FiniteTense::Future | FiniteTense::Past => false,
            }
        }
        GrammarCell::Imperative(_) if productive_verb_class(metadata.class.as_deref()) => {
            principal_part("imperative-stem")
                .and_then(|part| part.formation.as_deref())
                .is_some_and(|formation| formation != "irregular")
        }
        GrammarCell::Infinitive => productive_verb_class(metadata.class.as_deref()),
        GrammarCell::LParticiple(_) if productive_verb_class(metadata.class.as_deref()) => {
            principal_part("l-participle-stem").is_some()
        }
        GrammarCell::Participle(cell) if productive_verb_class(metadata.class.as_deref()) => {
            if cell.agreement.comparison != Comparison::Positive
                || (cell.tense == ParticipleTense::Present
                    && !matches!(
                        metadata.aspect.as_deref(),
                        Some("imperfective" | "biaspectual")
                    ))
            {
                return false;
            }
            let system = format!(
                "{}-{}-participle-{}-stem",
                cell.tense.code(),
                cell.voice.code(),
                cell.agreement.form.code()
            );
            let Some(part) = principal_part(&system) else {
                return false;
            };
            cell.voice != ParticipleVoice::Active
                || cell.agreement.form != AdjectiveForm::Short
                || part.formation.is_some()
        }
        GrammarCell::LexicalForm
        | GrammarCell::Indeclinable
        | GrammarCell::Supine
        | GrammarCell::VerbalNoun(_)
        | GrammarCell::FiniteVerb(_)
        | GrammarCell::Imperative(_)
        | GrammarCell::LParticiple(_)
        | GrammarCell::Participle(_) => false,
    }
}

fn pronoun_cell_is_supported(cell: PronounCell, class: Option<&str>) -> bool {
    if cell.case == synodal_church_slavonic::Case::Vocative {
        return false;
    }
    match class {
        Some("pronoun-personal-first") => {
            cell.gender.is_none() && cell.person == Some(Person::First)
        }
        Some("pronoun-personal-second") => {
            cell.gender.is_none() && cell.person == Some(Person::Second)
        }
        Some("pronoun-reflexive") => {
            cell.gender.is_none()
                && cell.person.is_none()
                && cell.number == Number::Singular
                && cell.case != synodal_church_slavonic::Case::Nominative
        }
        Some("pronoun-reflexive-clitic") => {
            cell.gender.is_none()
                && cell.person.is_none()
                && cell.number == Number::Singular
                && matches!(cell.case, Case::Dative | Case::Accusative)
        }
        Some("pronoun-third-person") => cell.gender.is_some() && cell.person == Some(Person::Third),
        Some("pronoun-third-person-demonstrative") => {
            cell.gender.is_some() && matches!(cell.person, None | Some(Person::Third))
        }
        Some("pronoun-relative-izhe")
        | Some(
            "pronoun-proximal-sei"
            | "pronoun-soft"
            | "pronoun-soft-i-alternating"
            | "pronoun-hard"
            | "pronoun-mixed-possessive"
            | "pronoun-short-hard"
            | "pronoun-short-ov-mixed"
            | "pronoun-short-velar"
            | "pronoun-quantity-velar"
            | "pronoun-full-hard"
            | "pronoun-full-soft"
            | "pronoun-full-velar"
            | "pronoun-interrogative-kii"
            | "pronoun-indefinite-kii"
            | "pronoun-negative-kii"
            | "pronoun-negative-full-hard"
            | "pronoun-kii-zhdo",
        ) => cell.gender.is_some() && cell.person.is_none(),
        Some(
            "pronoun-interrogative-who"
            | "pronoun-interrogative-what"
            | "pronoun-indefinite-who"
            | "pronoun-indefinite-what"
            | "pronoun-negative-who"
            | "pronoun-negative-what"
            | "pronoun-negative-who-zhe"
            | "pronoun-negative-what-zhe",
        ) => cell.gender.is_none() && cell.person.is_none() && cell.number == Number::Singular,
        _ => false,
    }
}

fn adjectival_cell_is_supported(cell: AdjectiveCell, has_comparative_stem: bool) -> bool {
    match (cell.comparison, cell.form) {
        (Comparison::Positive, _) => true,
        (Comparison::Comparative, _) => has_comparative_stem,
        (Comparison::Superlative, AdjectiveForm::Long) => has_comparative_stem,
        (Comparison::Superlative, AdjectiveForm::Short) => {
            has_comparative_stem && cell.case == Case::Nominative
        }
    }
}

fn determiner_cell_is_supported(cell: AdjectiveCell, class: Option<&str>) -> bool {
    if cell.comparison != Comparison::Positive {
        return false;
    }
    match class {
        Some("determiner-pronominal-hard") => true,
        Some("determiner-ves-mixed") => {
            cell.number != Number::Dual && cell.form == AdjectiveForm::Short
        }
        Some("determiner-vsyak-mixed") => cell.number != Number::Dual,
        Some("determiner-full-sk") => cell.form == AdjectiveForm::Long,
        _ => false,
    }
}

fn numeral_cell_is_supported(cell: NumeralCell, class: Option<&str>) -> bool {
    let nonvocative = cell.case != Case::Vocative;
    match class {
        Some("numeral-cardinal-one") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Singular
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-two" | "numeral-cardinal-both") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Dual
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-three" | "numeral-cardinal-four") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Plural
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-i-stem") => {
            cell.kind == NumeralKind::Cardinal
                && cell.gender.is_none()
                && nonvocative
                && (cell.number == Number::Singular
                    || cell.number == Number::Plural
                        && matches!(cell.case, Case::Genitive | Case::Dative | Case::Locative))
        }
        Some(
            "numeral-cardinal-ten"
            | "numeral-cardinal-hundred"
            | "numeral-cardinal-second-hard"
            | "numeral-cardinal-second-mixed"
            | "numeral-cardinal-first-hard-m"
            | "numeral-cardinal-third-f",
        ) => cell.kind == NumeralKind::Cardinal && cell.gender.is_none() && nonvocative,
        Some("ordinal-hard" | "ordinal-soft") => {
            cell.kind == NumeralKind::Ordinal && cell.gender.is_some()
        }
        Some("numeral-collective-agreeing" | "numeral-collective-hard-plural") => {
            cell.kind == NumeralKind::Collective
                && cell.number == Number::Plural
                && cell.gender.is_some()
        }
        Some("numeral-collective-governing-neuter") => {
            cell.kind == NumeralKind::Collective
                && cell.number == Number::Singular
                && cell.gender == Some(Gender::Neuter)
                && nonvocative
        }
        Some("numeral-multiplicative-hard" | "numeral-multiplicative-soft") => {
            cell.kind == NumeralKind::Multiplicative && cell.gender.is_some()
        }
        Some("numeral-fractional-hard") => {
            cell.kind == NumeralKind::Fractional && cell.gender.is_some()
        }
        Some(
            "numeral-fractional-first-u"
            | "numeral-fractional-second-hard"
            | "numeral-fractional-third-f",
        ) => cell.kind == NumeralKind::Fractional && cell.gender.is_none() && nonvocative,
        _ => false,
    }
}

fn productive_verb_class(class: Option<&str>) -> bool {
    matches!(
        class,
        Some("first-unpalatalized" | "first-palatalized" | "second" | "archaic")
    )
}

fn number_is_licensed(inventory: &str, number: Number) -> bool {
    match inventory {
        "singular-only" => number == Number::Singular,
        "dual-only" => number == Number::Dual,
        "plural-only" => number == Number::Plural,
        "singular-and-dual" => matches!(number, Number::Singular | Number::Dual),
        "singular-and-plural" => matches!(number, Number::Singular | Number::Plural),
        "dual-and-plural" => matches!(number, Number::Dual | Number::Plural),
        _ => true,
    }
}

fn verb_cells() -> Vec<GrammarCell> {
    let mut cells: Vec<GrammarCell> = FiniteVerbCell::inventory(&FiniteTense::ALL)
        .into_iter()
        .map(GrammarCell::FiniteVerb)
        .collect();
    cells.push(GrammarCell::Infinitive);
    cells.push(GrammarCell::Supine);
    for number in Number::ALL {
        for gender in Gender::ALL {
            cells.push(GrammarCell::LParticiple(LParticipleCell { gender, number }));
        }
        for person in Person::ALL {
            cells.push(GrammarCell::Imperative(ImperativeCell { person, number }));
        }
    }
    let agreements = AdjectiveCell::inventory(&AdjectiveForm::ALL, &[Comparison::Positive], |_| {
        &Animacy::ALL
    });
    cells.extend(
        ParticipleCell::inventory(&ParticipleTense::ALL, &ParticipleVoice::ALL, &agreements)
            .into_iter()
            .map(GrammarCell::Participle),
    );
    cells.extend(
        core::NounCell::inventory(&Animacy::ALL)
            .into_iter()
            .map(GrammarCell::VerbalNoun),
    );
    cells
}

fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect()
    }
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<u16> {
    let candidate_word = candidate
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .min_by_key(|word| levenshtein(query, word))
        .unwrap_or(candidate);
    let distance = levenshtein(query, candidate_word);
    let maximum = query.chars().count().max(candidate_word.chars().count());
    let allowed = 2_usize.max(maximum / 4);
    (distance <= allowed).then(|| {
        let penalty = (distance.saturating_mul(700)).min(3_000) as u16;
        7_000_u16.saturating_sub(penalty)
    })
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        current.push(left_index + 1);
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use synodal_church_slavonic::Case;

    #[test]
    fn semantic_lookup_keeps_source_recension_visible() {
        let entry = lookup("землѧ").expect("known entry");
        assert_eq!(entry.senses[0].source_recension, "mixed");
        assert_eq!(
            entry.senses[0].semantic_status,
            "reviewed-with-synodal-corpus"
        );
    }

    #[test]
    fn analyzer_returns_typed_cells_without_guessing_one() {
        let analyses = analyze("є҆́смь").expect("valid input");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.lemma() == "быти"
                && matches!(analysis.cell, Some(GrammarCell::FiniteVerb(_)))
        }));
    }

    #[test]
    fn verb_candidate_inventory_includes_every_represented_system() {
        let cells = candidate_cells(PartOfSpeech::Verb);
        assert_eq!(cells.len(), 1_116);
        assert!(cells.contains(&GrammarCell::Supine));
        assert!(
            cells
                .iter()
                .any(|cell| matches!(cell, GrammarCell::VerbalNoun(_)))
        );
    }

    #[test]
    fn candidate_inventory_sizes_remain_exhaustive_and_stable() {
        for (part_of_speech, expected) in [
            (PartOfSpeech::Adverb, 2),
            (PartOfSpeech::Noun, 43),
            (PartOfSpeech::Adjective, 757),
            (PartOfSpeech::Pronoun, 673),
            (PartOfSpeech::Numeral, 841),
            (PartOfSpeech::Determiner, 757),
            (PartOfSpeech::Participle, 1),
        ] {
            let cells = candidate_cells(part_of_speech);
            assert_eq!(cells.len(), expected, "{part_of_speech:?}");
            assert_eq!(cells.last(), Some(&GrammarCell::LexicalForm));
        }
    }

    #[test]
    fn every_productive_pronoun_identity_realizes_every_licensed_analysis_cell() {
        let inflector = Inflector::default();
        for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
            if lexeme.part_of_speech() != PartOfSpeech::Pronoun {
                continue;
            }
            let metadata = lexical_metadata(lexeme.id()).expect("pronoun metadata");
            if !metadata
                .class
                .as_deref()
                .is_some_and(|class| class.starts_with("pronoun-"))
            {
                continue;
            }
            let cells = analysis_cells_for_lexeme(&lexeme, inflector)
                .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
            assert!(
                cells
                    .iter()
                    .any(|cell| matches!(cell, GrammarCell::Pronoun(_))),
                "{} has no productive pronoun cells",
                lexeme.id()
            );
            for cell in cells {
                if !matches!(cell, GrammarCell::Pronoun(_)) {
                    continue;
                }
                let forms = inflector
                    .form_by_id(lexeme.id(), cell)
                    .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
                assert!(
                    !forms.variants().is_empty(),
                    "{} {}",
                    lexeme.id(),
                    cell.key()
                );
            }
        }
    }

    #[test]
    fn every_productive_determiner_identity_realizes_every_licensed_analysis_cell() {
        let inflector = Inflector::default();
        for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
            if lexeme.part_of_speech() != PartOfSpeech::Determiner {
                continue;
            }
            let metadata = lexical_metadata(lexeme.id()).expect("determiner metadata");
            if !metadata
                .class
                .as_deref()
                .is_some_and(|class| class.starts_with("determiner-"))
            {
                continue;
            }
            let cells = analysis_cells_for_lexeme(&lexeme, inflector)
                .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
            assert!(
                cells
                    .iter()
                    .any(|cell| matches!(cell, GrammarCell::Determiner(_))),
                "{} has no productive determiner cells",
                lexeme.id()
            );
            for cell in cells {
                if !matches!(cell, GrammarCell::Determiner(_)) {
                    continue;
                }
                let forms = inflector
                    .form_by_id(lexeme.id(), cell)
                    .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
                assert!(
                    !forms.variants().is_empty(),
                    "{} {}",
                    lexeme.id(),
                    cell.key()
                );
            }
        }
    }

    #[test]
    fn every_productive_numeral_identity_realizes_every_licensed_analysis_cell() {
        let inflector = Inflector::default();
        for lexeme in synodal_church_slavonic::lexemes().expect("registry") {
            if lexeme.part_of_speech() != PartOfSpeech::Numeral {
                continue;
            }
            let metadata = lexical_metadata(lexeme.id()).expect("numeral metadata");
            if !metadata
                .class
                .as_deref()
                .is_some_and(|class| class.starts_with("numeral-") || class.starts_with("ordinal-"))
            {
                continue;
            }
            let cells = analysis_cells_for_lexeme(&lexeme, inflector)
                .unwrap_or_else(|error| panic!("{} inventory: {error}", lexeme.id()));
            assert!(
                cells
                    .iter()
                    .any(|cell| matches!(cell, GrammarCell::Numeral(_))),
                "{} has no productive numeral cells",
                lexeme.id()
            );
            for cell in cells {
                if !matches!(cell, GrammarCell::Numeral(_)) {
                    continue;
                }
                let forms = inflector
                    .form_by_id(lexeme.id(), cell)
                    .unwrap_or_else(|error| panic!("{} {}: {error}", lexeme.id(), cell.key()));
                assert!(
                    !forms.variants().is_empty(),
                    "{} {}",
                    lexeme.id(),
                    cell.key()
                );
            }
        }
    }

    #[test]
    fn analyzer_uses_explicit_accents_to_disambiguate_homographs() {
        let conjunction = analyze("и҆").expect("valid conjunction");
        assert_eq!(conjunction.len(), 1);
        assert_eq!(
            conjunction[0].lexeme.part_of_speech(),
            PartOfSpeech::Conjunction
        );

        let pronoun = analyze("и҆̀").expect("valid pronoun");
        assert!(!pronoun.is_empty());
        assert!(
            pronoun
                .iter()
                .all(|analysis| analysis.lexeme.id().as_str() == "synodal:pronoun:on")
        );

        let unmarked = analyze("и").expect("valid unmarked spelling");
        assert!(unmarked.len() > 1, "unmarked homograph must stay ambiguous");

        let incompatible = analyze("и\u{301}").expect("valid incompatible accent");
        assert!(
            incompatible.is_empty(),
            "an explicit incompatible accent must not fall back to accentless analysis"
        );
    }

    #[test]
    fn analyzer_canonicalizes_reviewed_conjunction_marks_without_restoring_rejected_identity() {
        let analyses = analyze("ꙗ҆́кѡ").expect("valid reviewed marked form");
        let identities: BTreeSet<_> = analyses
            .iter()
            .map(|analysis| analysis.lexeme.id().as_str())
            .collect();
        assert_eq!(identities.len(), 1);
        assert!(identities.contains("synodal:conjunction:wikt-47fa23a7ed6b"));
        assert!(!identities.contains("synodal:adverb:wikt-5471d4207f64"));
    }

    #[test]
    fn analyzer_keeps_closed_class_variants_exact_and_collision_free() {
        let ko = analyze("ко").expect("valid positional preposition variant");
        assert!(ko.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:preposition:wikt-77998a1b179f"
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));

        let vo = analyze("во").expect("valid positional preposition variant");
        assert!(vo.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:preposition:wikt-9c77102d5441"
                && analysis.cell == Some(GrammarCell::LexicalForm)
        }));

        let so = analyze("со").expect("valid positional preposition variant");
        assert!(so.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:preposition:wikt-c63ef675e22e"
        }));

        let ubo = analyze("ᲂу҆̀бо").expect("valid positional conjunction variant");
        assert!(ubo.iter().any(|analysis| {
            analysis.lexeme.id().as_str() == "synodal:conjunction:wikt-a0dc1a363208"
        }));

        let soti = analyze("соти").expect("orthographically valid negative control");
        assert!(soti.iter().all(|analysis| {
            analysis.lexeme.id().as_str() != "synodal:preposition:wikt-c63ef675e22e"
        }));
        let liti = analyze("лити").expect("orthographically valid negative control");
        assert!(
            liti.iter()
                .all(|analysis| { analysis.lexeme.id().as_str() != "synodal:conjunction:li" })
        );
        let yuzhe = analyze("юже").expect("orthographically valid negative control");
        assert!(
            yuzhe.iter().all(|analysis| {
                analysis.lexeme.id().as_str() != "synodal:noun:wikt-f330683bc04d"
            })
        );
    }

    #[test]
    fn analyzer_expands_semantic_abbreviation() {
        let analyses = analyze("бг҃ъ").expect("valid abbreviation");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.lemma() == "богъ"
                && analysis.source == AnalysisSource::AbbreviationExpansion
        }));
    }

    #[test]
    fn analyzer_requires_explicit_policy_for_inherited_predictions() {
        assert!(analyze("градомъ").expect("valid input").is_empty());
        let analyses = analyze_with(
            "градомъ",
            Inflector::builder()
                .generation_policy(synodal_church_slavonic::GenerationPolicy::Productive)
                .build(),
        )
        .expect("valid productive analysis");
        assert!(analyses.iter().any(|analysis| {
            analysis.lexeme.lemma() == "градъ"
                && analysis.source == AnalysisSource::InheritedPrediction
                && analysis.recension_mapping.is_some()
        }));
    }

    #[test]
    fn vocabulary_lint_rejects_latin_and_missing_sense() {
        let issues = lint_vocabulary(&VocabularyManifest {
            entries: vec![
                VocabularyItem {
                    text: "slovo".into(),
                    expected_lexeme_id: None,
                    expected_part_of_speech: None,
                    required_sense_id: None,
                    requested_cell: None,
                },
                VocabularyItem {
                    text: "рабъ".into(),
                    expected_lexeme_id: None,
                    expected_part_of_speech: Some(PartOfSpeech::Noun),
                    required_sense_id: Some("missing".into()),
                    requested_cell: None,
                },
            ],
        });
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == VocabularyIssueKind::InvalidOrthography)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == VocabularyIssueKind::MissingSemanticIdentity)
        );
    }

    #[test]
    fn vocabulary_lint_uses_the_supplied_analyzer_policy_for_requested_cells() {
        let analyzer = coverage::Analyzer::new(
            Inflector::builder()
                .generation_policy(synodal_church_slavonic::GenerationPolicy::Productive)
                .build(),
        )
        .expect("productive analyzer");
        let issues = lint_vocabulary_with(
            &analyzer,
            &VocabularyManifest {
                entries: vec![VocabularyItem {
                    text: "граде".into(),
                    expected_lexeme_id: Some(LexemeId::from("synodal:noun:grad")),
                    expected_part_of_speech: Some(PartOfSpeech::Noun),
                    required_sense_id: None,
                    requested_cell: Some(GrammarCell::Noun(morphology::NounCell {
                        case: Case::Vocative,
                        number: Number::Singular,
                        animacy: Animacy::Inanimate,
                    })),
                }],
            },
        );
        assert!(
            issues
                .iter()
                .all(|issue| issue.kind != VocabularyIssueKind::UnsupportedFormation),
            "productive requested cell was rejected: {issues:?}"
        );
    }

    #[test]
    fn gloss_search_is_deterministic() {
        let results = search_gloss("religion").expect("search");
        assert!(
            results
                .windows(2)
                .all(|pair| pair[0].lexeme.id() < pair[1].lexeme.id())
        );
    }

    #[test]
    fn family_lookup_excludes_rejected_contextual_homograph() {
        let results = families("ꙗкѡ").expect("reviewed families");
        let identities: BTreeSet<_> = results
            .iter()
            .map(|family| (family.lexeme.id().as_str(), family.lexeme.part_of_speech()))
            .collect();
        assert!(!identities.contains(&("synodal:adverb:wikt-5471d4207f64", PartOfSpeech::Adverb)));
        assert!(identities.contains(&(
            "synodal:conjunction:wikt-47fa23a7ed6b",
            PartOfSpeech::Conjunction
        )));
    }

    #[test]
    fn kamen_exact_and_productive_analyses_share_one_stable_identity() {
        let analyses = analyze("камень").expect("reviewed камень analyses");
        let identities: BTreeSet<_> = analyses
            .iter()
            .map(|analysis| analysis.lexeme.id().as_str())
            .collect();

        assert_eq!(
            identities,
            BTreeSet::from(["synodal:noun:v07-c27905de175a0cde"])
        );
        assert!(
            analyses
                .iter()
                .any(|analysis| analysis.source == AnalysisSource::ExactSynodalAttestation)
        );
    }

    #[test]
    fn family_summary_exposes_exact_cells_and_productive_determiner_metadata() {
        let id = FamilyId::for_lexeme(&LexemeId::from("synodal:determiner:ves"));
        let family = show_family_by_id(&id).expect("reviewed весь family");
        assert_eq!(family.id.as_str(), "family:synodal:determiner:ves");
        assert!(!family.exact_only);
        assert!(family.fully_classed);
        assert_eq!(family.class.as_deref(), Some("determiner-ves-mixed"));
        assert_eq!(family.stem.as_deref(), Some("вс"));
        assert!(family.members.iter().any(|member| {
            member.cell == "determiner:nominative:singular:feminine:inanimate:short:positive"
                && member.printed == "всѧ̀"
        }));
        assert!(family.missing_family_metadata.is_empty());
    }

    #[test]
    fn family_supported_systems_cover_productive_and_exact_capabilities() {
        for (id, expected) in [
            ("synodal:determiner:sam", "determiner"),
            ("synodal:numeral:pervyi", "numeral"),
            ("synodal:verb:byti", "future"),
            ("synodal:verb:wikt-78da2d05497d", "past"),
        ] {
            let family = show_family_by_id(&FamilyId::for_lexeme(&LexemeId::from(id)))
                .expect("reviewed family");
            assert!(
                family
                    .supported_systems
                    .iter()
                    .any(|system| system == expected),
                "{id} should report {expected}: {:?}",
                family.supported_systems
            );
        }
    }

    #[test]
    fn complete_possessive_tables_are_truthfully_classed_and_productive() {
        for lexeme in ["moi", "tvoi", "svoi", "nash", "vash"] {
            let id = FamilyId::for_lexeme(&LexemeId::from(format!("synodal:pronoun:{lexeme}")));
            let family = show_family_by_id(&id).expect("reviewed possessive family");
            assert!(!family.exact_only);
            assert!(family.fully_classed);
            assert_eq!(family.members.len(), 57);
            assert!(family.missing_family_metadata.is_empty());
        }

        let vash = show_family_by_id(&FamilyId::for_lexeme(&LexemeId::from(
            "synodal:pronoun:vash",
        )))
        .expect("reviewed вашъ family");
        assert!(vash.members.iter().any(|member| {
            member.cell == "pronoun:dative:plural:masculine:none:any"
                && member.expanded == "вашымъ"
                && member.printed == "ва́шымъ"
        }));
        assert!(vash.members.iter().any(|member| {
            member.cell == "pronoun:accusative:singular:masculine:none:animate"
                && member.expanded == "вашего"
        }));
    }
}
