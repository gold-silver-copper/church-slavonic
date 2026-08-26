#[allow(unused_imports)]
use super::*;

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
