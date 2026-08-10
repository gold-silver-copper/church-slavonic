#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    Gender, GrammarCell, ImperativeCell, Inflector, LParticipleCell, LexemeId, LexemeSummary,
    Number, NumeralCell, NumeralKind, OrthographyProfile, PartOfSpeech, ParticipleCell,
    ParticipleTense, ParticipleVoice, Person, PronounCell, Result, abbreviation, lexemes,
};
use synodal_church_slavonic_core::{
    Confidence, FormSource, RecensionMappingId, SynodalWord, normalize_lookup_accentless,
};

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
}

/// Returns every compatible curated analysis of an expanded or printed word.
pub fn analyze(word: &str) -> Result<Vec<Analysis>> {
    analyze_with(word, Inflector::default())
}

/// Returns every compatible curated analysis admitted by the caller's
/// generation and orthography policy. The default `analyze` remains Strict;
/// callers must opt into inherited or exploratory predictions explicitly.
pub fn analyze_with(word: &str, inflector: Inflector) -> Result<Vec<Analysis>> {
    let word = SynodalWord::parse(word)?;
    let lookup = normalize_lookup_accentless(word.canonical());
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

    let mut analyses = Vec::new();
    let mut seen = BTreeSet::new();
    for lexeme in lexemes()? {
        for cell in candidate_cells(lexeme.part_of_speech()) {
            if let Ok(forms) = expanded_inflector.form_by_id(lexeme.id(), cell) {
                collect_matching(&lookup, &lexeme, cell, &forms, &mut seen, &mut analyses);
            }
            if let Ok(forms) = printed_inflector.form_by_id(lexeme.id(), cell) {
                collect_matching(&lookup, &lexeme, cell, &forms, &mut seen, &mut analyses);
            }
        }
    }

    if let Ok(expansions) = abbreviation::expand(word.canonical()) {
        for expansion in expansions {
            let lexeme = morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
            let key = (
                lexeme.id().clone(),
                None,
                AnalysisSource::AbbreviationExpansion,
                None,
            );
            if seen.insert(key) {
                analyses.push(Analysis {
                    lexeme,
                    cell: None,
                    matched_text: word.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: Confidence::CERTAIN,
                    evidence_ids: vec![expansion.evidence_id.to_string()],
                });
            }
        }
    }
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
    pub expected_part_of_speech: Option<PartOfSpeech>,
    pub required_sense_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VocabularyIssueKind {
    InvalidOrthography,
    UnknownVocabulary,
    UnexpectedPartOfSpeech,
    MissingSemanticIdentity,
    AmbiguousSurfaceForm,
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
    lookup: &str,
    lexeme: &LexemeSummary,
    cell: GrammarCell,
    forms: &synodal_church_slavonic_core::FormSet,
    seen: &mut BTreeSet<(
        LexemeId,
        Option<GrammarCell>,
        AnalysisSource,
        Option<RecensionMappingId>,
    )>,
    analyses: &mut Vec<Analysis>,
) {
    for variant in forms.variants() {
        let matches = [
            variant.expanded.as_str(),
            variant.printed.as_str(),
            variant.accented.as_deref().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|value| !value.is_empty())
        .any(|value| normalize_lookup_accentless(value) == lookup);
        let source = analysis_source(&variant.source);
        let key = (
            lexeme.id().clone(),
            Some(cell),
            source,
            variant.recension_mapping.clone(),
        );
        if matches && seen.insert(key) {
            analyses.push(Analysis {
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
            });
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

fn candidate_cells(part_of_speech: PartOfSpeech) -> Vec<GrammarCell> {
    match part_of_speech {
        PartOfSpeech::Noun => Number::ALL
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
                        Animacy::ALL.into_iter().map(move |animacy| {
                            GrammarCell::Pronoun(PronounCell {
                                case,
                                number,
                                gender,
                                person: None,
                                animacy,
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
    }
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
                    expected_part_of_speech: None,
                    required_sense_id: None,
                },
                VocabularyItem {
                    text: "рабъ".into(),
                    expected_part_of_speech: Some(PartOfSpeech::Noun),
                    required_sense_id: Some("missing".into()),
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
}
