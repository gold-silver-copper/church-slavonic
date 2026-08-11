#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    Gender, GrammarCell, ImperativeCell, Inflector, LParticipleCell, LexemeId, LexemeSummary,
    LexicalMetadataSummary, MetadataField, Number, NumeralCell, NumeralKind, OrthographyProfile,
    PartOfSpeech, ParticipleCell, ParticipleTense, ParticipleVoice, Person, PronounCell, Result,
    abbreviation, capabilities_by_id, lexemes, lexical_metadata, missing_metadata_by_id,
};
use synodal_church_slavonic_core::{
    Confidence, FormSource, RecensionMappingId, RuleTrace, SynodalWord, normalize_lookup,
    normalize_lookup_accentless,
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
                cell: morphology::grammar_cell_key(contraction.cell),
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
    let supported_systems = [
        (capabilities.productive_noun, "noun"),
        (capabilities.productive_adjective, "adjective"),
        (capabilities.present, "present"),
        (capabilities.imperfect, "imperfect"),
        (capabilities.aorist, "aorist"),
        (capabilities.imperative, "imperative"),
        (capabilities.infinitive, "infinitive"),
        (capabilities.l_participle, "l-participle"),
        (capabilities.participle, "participle"),
        (capabilities.supine, "supine"),
        (capabilities.verbal_noun, "verbal-noun"),
    ]
    .into_iter()
    .filter_map(|(supported, system)| supported.then_some(system.into()))
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
    SynodalNormativeTable,
    SynodalProductiveRule,
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

type AnalysisKey = (
    LexemeId,
    Option<GrammarCell>,
    AnalysisSource,
    Option<RecensionMappingId>,
);
type RankedAnalysis = (u8, AnalysisKey, Analysis);

/// Returns every compatible curated analysis of an expanded or printed word.
pub fn analyze(word: &str) -> Result<Vec<Analysis>> {
    analyze_with(word, Inflector::default())
}

/// Returns every compatible curated analysis admitted by the caller's
/// generation and orthography policy. The default `analyze` remains Strict;
/// callers must opt into inherited or exploratory predictions explicitly.
pub fn analyze_with(word: &str, inflector: Inflector) -> Result<Vec<Analysis>> {
    let word = SynodalWord::parse(word)?;
    let marked_lookup = normalize_lookup(word.canonical());
    let lookup = normalize_lookup_accentless(word.canonical());
    let allow_accentless = marked_lookup == lookup
        || inflector.orthography() == OrthographyProfile::ExpandedAccentless;
    let expanded_inflector = Inflector::builder()
        .generation_policy(inflector.generation_policy())
        .orthography(OrthographyProfile::Expanded)
        .productive_mapping_threshold_basis_points(
            inflector.productive_mapping_threshold_basis_points(),
        )
        .build();
    let printed_inflector = Inflector::builder()
        .generation_policy(inflector.generation_policy())
        .orthography(OrthographyProfile::SynodalLiturgical)
        .productive_mapping_threshold_basis_points(
            inflector.productive_mapping_threshold_basis_points(),
        )
        .build();

    let mut ranked = Vec::new();
    for lexeme in lexemes()? {
        for cell in candidate_cells(lexeme.part_of_speech()) {
            if let Ok(forms) = expanded_inflector.form_by_id(lexeme.id(), cell) {
                collect_matching(
                    &marked_lookup,
                    &lookup,
                    allow_accentless,
                    &lexeme,
                    cell,
                    &forms,
                    &mut ranked,
                );
            }
            if let Ok(forms) = printed_inflector.form_by_id(lexeme.id(), cell) {
                collect_matching(
                    &marked_lookup,
                    &lookup,
                    allow_accentless,
                    &lexeme,
                    cell,
                    &forms,
                    &mut ranked,
                );
            }
        }
    }

    if let Ok(expansions) = abbreviation::expand(word.canonical()) {
        for expansion in expansions {
            let lexeme = morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
            let key = (
                lexeme.id().clone(),
                Some(expansion.cell),
                AnalysisSource::AbbreviationExpansion,
                None,
            );
            ranked.push((
                2_u8,
                key,
                Analysis {
                    lexeme,
                    cell: Some(expansion.cell),
                    matched_text: word.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: Confidence::CERTAIN,
                    evidence_ids: expansion
                        .evidence_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    assumptions: Vec::new(),
                    contradictions: Vec::new(),
                    warnings: Vec::new(),
                    rule_trace: RuleTrace::default(),
                },
            ));
        }
    }
    let mut best_by_analysis = BTreeMap::new();
    for (quality, key, mut analysis) in ranked {
        if quality == 1 {
            analysis
                .warnings
                .push("analysis required accent-insensitive matching".into());
        }
        let replace = best_by_analysis
            .get(&key)
            .is_none_or(|(current, _)| quality > *current);
        if replace {
            best_by_analysis.insert(key, (quality, analysis));
        }
    }
    let best_quality = best_by_analysis
        .values()
        .map(|(quality, _)| *quality)
        .max()
        .unwrap_or_default();
    let mut analyses: Vec<_> = best_by_analysis
        .into_values()
        .filter_map(|(quality, analysis)| (quality == best_quality).then_some(analysis))
        .collect();
    analyses.sort_by(|left, right| {
        left.lexeme
            .id()
            .cmp(right.lexeme.id())
            .then_with(|| left.cell.cmp(&right.cell))
    });
    Ok(analyses)
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
    let mut issues = Vec::new();
    for (index, item) in manifest.entries.iter().enumerate() {
        match analyze(&item.text) {
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
                        match Inflector::default().form_by_id(&id, cell) {
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

fn collect_matching(
    marked_lookup: &str,
    lookup: &str,
    allow_accentless: bool,
    lexeme: &LexemeSummary,
    cell: GrammarCell,
    forms: &synodal_church_slavonic_core::FormSet,
    analyses: &mut Vec<RankedAnalysis>,
) {
    for variant in forms.variants() {
        let quality = [
            variant.expanded.as_str(),
            variant.printed.as_str(),
            variant.accented.as_deref().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|value| !value.is_empty())
        .filter_map(|value| {
            let canonical = SynodalWord::parse(value).ok()?;
            if normalize_lookup(canonical.canonical()) == marked_lookup {
                Some(2_u8)
            } else if allow_accentless
                && normalize_lookup_accentless(canonical.canonical()) == lookup
            {
                Some(1_u8)
            } else {
                None
            }
        })
        .max();
        let source = analysis_source(&variant.source);
        let key = (
            lexeme.id().clone(),
            Some(cell),
            source,
            variant.recension_mapping.clone(),
        );
        if let Some(quality) = quality {
            analyses.push((
                quality,
                key,
                Analysis {
                    lexeme: lexeme.clone(),
                    cell: Some(cell),
                    matched_text: variant.printed.clone(),
                    source,
                    recension_mapping: variant.recension_mapping.clone(),
                    confidence: variant.confidence,
                    evidence_ids: variant
                        .evidence
                        .iter()
                        .map(|evidence| evidence.id.to_string())
                        .collect(),
                    assumptions: variant
                        .assumptions
                        .iter()
                        .map(|assumption| assumption.detail.clone())
                        .collect(),
                    contradictions: variant
                        .contradictions
                        .iter()
                        .map(|contradiction| contradiction.detail.clone())
                        .collect(),
                    warnings: variant.warnings.clone(),
                    rule_trace: variant.rule_trace.clone(),
                },
            ));
        }
    }
}

fn analysis_source(source: &FormSource) -> AnalysisSource {
    match source {
        FormSource::SynodalAttestation { .. } => AnalysisSource::ExactSynodalAttestation,
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-REGISTRY-NORMATIVE-TABLE" =>
        {
            AnalysisSource::SynodalNormativeTable
        }
        FormSource::SynodalNormativeGeneration { .. } => AnalysisSource::SynodalProductiveRule,
        FormSource::InheritedPrediction { .. } => AnalysisSource::InheritedPrediction,
        FormSource::AnalogicalPrediction { .. } => AnalysisSource::AnalogicalPrediction,
    }
}

/// Enumerates cells that reverse analysis may attempt for a part of speech.
/// Unsupported cells still fail through the facade's typed error contract.
#[must_use]
pub fn candidate_cells(part_of_speech: PartOfSpeech) -> Vec<GrammarCell> {
    let mut cells = match part_of_speech {
        PartOfSpeech::Adverb
        | PartOfSpeech::Preposition
        | PartOfSpeech::Conjunction
        | PartOfSpeech::Particle
        | PartOfSpeech::Interjection => {
            vec![GrammarCell::Indeclinable]
        }
        PartOfSpeech::Noun | PartOfSpeech::ProperNoun => Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Animacy::ALL.into_iter().map(move |animacy| {
                        GrammarCell::Noun(core::NounCell {
                            case,
                            number,
                            animacy,
                        })
                    })
                })
            })
            .collect(),
        PartOfSpeech::Adjective => Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Gender::ALL.into_iter().flat_map(move |gender| {
                        Animacy::ALL.into_iter().flat_map(move |animacy| {
                            AdjectiveForm::ALL.into_iter().flat_map(move |form| {
                                Comparison::ALL.into_iter().map(move |comparison| {
                                    GrammarCell::Adjective(AdjectiveCell {
                                        case,
                                        number,
                                        gender,
                                        animacy,
                                        form,
                                        comparison,
                                    })
                                })
                            })
                        })
                    })
                })
            })
            .collect(),
        PartOfSpeech::Verb => verb_cells(),
        PartOfSpeech::Pronoun => Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    [
                        None,
                        Some(Gender::Masculine),
                        Some(Gender::Feminine),
                        Some(Gender::Neuter),
                    ]
                    .into_iter()
                    .flat_map(move |gender| {
                        [
                            None,
                            Some(Person::First),
                            Some(Person::Second),
                            Some(Person::Third),
                        ]
                        .into_iter()
                        .flat_map(move |person| {
                            Animacy::ALL.into_iter().map(move |animacy| {
                                GrammarCell::Pronoun(PronounCell {
                                    case,
                                    number,
                                    gender,
                                    person,
                                    animacy,
                                })
                            })
                        })
                    })
                })
            })
            .collect(),
        PartOfSpeech::Numeral => NumeralKind::ALL
            .into_iter()
            .flat_map(|kind| {
                Number::ALL.into_iter().flat_map(move |number| {
                    Case::ALL.into_iter().flat_map(move |case| {
                        [
                            None,
                            Some(Gender::Masculine),
                            Some(Gender::Feminine),
                            Some(Gender::Neuter),
                        ]
                        .into_iter()
                        .flat_map(move |gender| {
                            Animacy::ALL.into_iter().map(move |animacy| {
                                GrammarCell::Numeral(NumeralCell {
                                    kind,
                                    case,
                                    number,
                                    gender,
                                    animacy,
                                })
                            })
                        })
                    })
                })
            })
            .collect(),
        PartOfSpeech::Determiner => Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Gender::ALL.into_iter().flat_map(move |gender| {
                        Animacy::ALL.into_iter().flat_map(move |animacy| {
                            AdjectiveForm::ALL.into_iter().flat_map(move |form| {
                                Comparison::ALL.into_iter().map(move |comparison| {
                                    GrammarCell::Determiner(AdjectiveCell {
                                        case,
                                        number,
                                        gender,
                                        animacy,
                                        form,
                                        comparison,
                                    })
                                })
                            })
                        })
                    })
                })
            })
            .collect(),
        PartOfSpeech::Participle => Vec::new(),
    };
    cells.push(GrammarCell::LexicalForm);
    cells
}

fn verb_cells() -> Vec<GrammarCell> {
    let mut cells: Vec<GrammarCell> = FiniteTense::ALL
        .into_iter()
        .flat_map(|tense| {
            Number::ALL.into_iter().flat_map(move |number| {
                Person::ALL.into_iter().map(move |person| {
                    GrammarCell::FiniteVerb(FiniteVerbCell {
                        tense,
                        person,
                        number,
                    })
                })
            })
        })
        .collect();
    cells.push(GrammarCell::Infinitive);
    for number in Number::ALL {
        for gender in Gender::ALL {
            cells.push(GrammarCell::LParticiple(LParticipleCell { gender, number }));
        }
        for person in Person::ALL {
            cells.push(GrammarCell::Imperative(ImperativeCell { person, number }));
        }
    }
    for tense in ParticipleTense::ALL {
        for voice in ParticipleVoice::ALL {
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        for animacy in Animacy::ALL {
                            for form in AdjectiveForm::ALL {
                                cells.push(GrammarCell::Participle(ParticipleCell {
                                    tense,
                                    voice,
                                    agreement: AdjectiveCell {
                                        case,
                                        number,
                                        gender,
                                        animacy,
                                        form,
                                        comparison: Comparison::Positive,
                                    },
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
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
    fn analyzer_canonicalizes_reviewed_variant_marks_without_erasing_ambiguity() {
        let analyses = analyze("ꙗ҆́кѡ").expect("valid reviewed marked form");
        let identities: BTreeSet<_> = analyses
            .iter()
            .map(|analysis| analysis.lexeme.id().as_str())
            .collect();
        assert_eq!(identities.len(), 2);
        assert!(identities.contains("synodal:adverb:wikt-5471d4207f64"));
        assert!(identities.contains("synodal:conjunction:wikt-47fa23a7ed6b"));
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
    fn gloss_search_is_deterministic() {
        let results = search_gloss("religion").expect("search");
        assert!(
            results
                .windows(2)
                .all(|pair| pair[0].lexeme.id() < pair[1].lexeme.id())
        );
    }

    #[test]
    fn family_lookup_preserves_reviewed_homographs() {
        let results = families("ꙗкѡ").expect("reviewed families");
        let identities: BTreeSet<_> = results
            .iter()
            .map(|family| (family.lexeme.id().as_str(), family.lexeme.part_of_speech()))
            .collect();
        assert!(identities.contains(&("synodal:adverb:wikt-5471d4207f64", PartOfSpeech::Adverb)));
        assert!(identities.contains(&(
            "synodal:conjunction:wikt-47fa23a7ed6b",
            PartOfSpeech::Conjunction
        )));
    }

    #[test]
    fn family_summary_exposes_exact_cells_and_missing_metadata() {
        let id = FamilyId::for_lexeme(&LexemeId::from("synodal:determiner:ves"));
        let family = show_family_by_id(&id).expect("reviewed весь family");
        assert_eq!(family.id.as_str(), "family:synodal:determiner:ves");
        assert!(family.exact_only);
        assert!(!family.fully_classed);
        assert!(family.members.iter().any(|member| {
            member.cell == "determiner:nominative:singular:feminine:inanimate:short:positive"
                && member.printed == "всѧ̀"
        }));
        assert!(
            family
                .missing_family_metadata
                .iter()
                .any(|field| field.contains("inflection-class"))
        );
    }

    #[test]
    fn complete_possessive_tables_are_truthfully_classed_and_closed() {
        for lexeme in ["moi", "tvoi", "svoi", "nash", "vash"] {
            let id = FamilyId::for_lexeme(&LexemeId::from(format!("synodal:pronoun:{lexeme}")));
            let family = show_family_by_id(&id).expect("reviewed possessive family");
            assert!(family.exact_only);
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
