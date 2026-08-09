use crate::{lookup, score_sense_for_concept, sense_by_id};
use serde::Serialize;
use std::collections::BTreeSet;

const HEADER: &str = "concept\tlemma\tpart_of_speech\tsense_id\tstatus\tnotes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VocabularyIssueLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VocabularyIssue {
    pub line: usize,
    pub level: VocabularyIssueLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VocabularyReport {
    pub rows: usize,
    pub attested: usize,
    pub thematic: usize,
    pub proper_names: usize,
    pub issues: Vec<VocabularyIssue>,
}

impl VocabularyReport {
    pub fn is_ok(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|issue| issue.level == VocabularyIssueLevel::Error)
    }
}

/// Validate a source-backed game vocabulary manifest.
///
/// The exact six-column header is:
/// `concept, lemma, part_of_speech, sense_id, status, notes`.
/// Supported statuses are `attested`, `thematic`, and `proper-name`.
pub fn validate_vocabulary_tsv(contents: &str) -> VocabularyReport {
    let mut report = VocabularyReport {
        rows: 0,
        attested: 0,
        thematic: 0,
        proper_names: 0,
        issues: Vec::new(),
    };
    let mut lines = contents.lines();
    if lines.next() != Some(HEADER) {
        report.issues.push(VocabularyIssue {
            line: 1,
            level: VocabularyIssueLevel::Error,
            message: format!("expected header: {HEADER}"),
        });
        return report;
    }

    let mut concepts = BTreeSet::new();
    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        report.rows += 1;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 6 {
            error(
                &mut report,
                line_number,
                format!("expected 6 tab-separated columns, found {}", columns.len()),
            );
            continue;
        }
        let (concept, lemma, part_of_speech, sense_id, status, notes) = (
            columns[0].trim(),
            columns[1].trim(),
            columns[2].trim(),
            columns[3].trim(),
            columns[4].trim(),
            columns[5].trim(),
        );
        if concept.is_empty() || lemma.is_empty() || part_of_speech.is_empty() || status.is_empty()
        {
            error(
                &mut report,
                line_number,
                "required field is empty".to_string(),
            );
            continue;
        }
        if !concepts.insert(concept.to_lowercase()) {
            error(
                &mut report,
                line_number,
                format!("duplicate game concept: {concept}"),
            );
        }

        match status {
            "attested" | "thematic" => {
                if status == "attested" {
                    report.attested += 1;
                } else {
                    report.thematic += 1;
                    if notes.is_empty() {
                        error(
                            &mut report,
                            line_number,
                            "thematic choices require a rationale in notes".to_string(),
                        );
                    }
                }
                let Some(sense) = sense_by_id(sense_id) else {
                    error(
                        &mut report,
                        line_number,
                        format!("unknown Wiktionary sense id: {sense_id}"),
                    );
                    continue;
                };
                let lemma_matches = lookup(lemma).is_ok_and(|senses| {
                    senses.iter().any(|candidate| candidate.id() == sense.id())
                });
                if !lemma_matches {
                    error(
                        &mut report,
                        line_number,
                        format!("sense {sense_id} does not belong to lemma {lemma}"),
                    );
                }
                if sense.part_of_speech() != part_of_speech {
                    error(
                        &mut report,
                        line_number,
                        format!(
                            "sense {sense_id} is {}, not {part_of_speech}",
                            sense.part_of_speech()
                        ),
                    );
                }
                if status == "attested" && score_sense_for_concept(sense, concept) == 0 {
                    error(
                        &mut report,
                        line_number,
                        format!(
                            "attested concept {concept:?} does not match the selected glosses: {}",
                            sense.glosses().join(", ")
                        ),
                    );
                }
            }
            "proper-name" => {
                report.proper_names += 1;
                if sense_id != "-" {
                    error(
                        &mut report,
                        line_number,
                        "proper-name rows must use '-' as sense_id".to_string(),
                    );
                }
                if notes.is_empty() {
                    error(
                        &mut report,
                        line_number,
                        "proper-name rows require a rationale in notes".to_string(),
                    );
                }
            }
            "unresolved" => error(
                &mut report,
                line_number,
                format!("unresolved game concept: {concept}"),
            ),
            other => error(
                &mut report,
                line_number,
                format!("unsupported vocabulary status: {other}"),
            ),
        }
    }
    report
}

fn error(report: &mut VocabularyReport, line: usize, message: String) {
    report.issues.push(VocabularyIssue {
        line,
        level: VocabularyIssueLevel::Error,
        message,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_manifest_fails_closed() {
        let report = validate_vocabulary_tsv("not the header\n");
        assert!(!report.is_ok());
    }
}
