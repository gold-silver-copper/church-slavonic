use std::{
    error, fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use super::*;

/// Schema version for normalized Synodal registries.
pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

/// Sources admitted to the Synodal data pipeline, together with their locked
/// recension classification. New sources require an explicit code review so a
/// neutral manifest ID cannot bypass the source boundary.
pub const APPROVED_SOURCE_RECENSIONS: &[(&str, &str)] = &[
    ("unicode-tn41-revision-1", "mixed"),
    (
        "english-wiktionary-ocs-kaikki-2026-08-07",
        "old-church-slavonic",
    ),
    ("english-wiktionary-ocs-lineage-2026-08-07", "mixed"),
    ("polivanova-osd-source", "old-church-slavonic"),
    ("polivanova-fup-2023", "old-church-slavonic"),
    ("gorshkov-ocs-2002", "old-church-slavonic"),
    ("leuta-havryliuk-ocs-2018", "old-church-slavonic"),
    ("gorazd", "old-church-slavonic"),
    ("trager-kiev-fragment-1933", "old-church-slavonic"),
    ("ud-ocs-proiel-r2.18", "old-church-slavonic"),
    ("syntacticus-20230428", "old-church-slavonic"),
    ("ccmh-2021-04-23", "old-church-slavonic"),
    ("diacu-1.0", "mixed"),
    ("ponomar-elizabeth-bible-2026-08-09", "synodal-russian"),
    ("crosswire-csl-elizabeth-1.5.2", "synodal-russian"),
    (
        "wikisource-church-slavonic-bible-2026-08-09",
        "synodal-russian",
    ),
    ("ponomar-library-catalog-2026-08-09", "synodal-russian"),
    ("russian-national-corpus-church-slavonic", "mixed"),
    ("polyakov-church-slavonic-grammatical-dictionary", "mixed"),
    ("ponomar-modern-church-slavonic-corpus-2016", "mixed"),
    ("alypy-gamanovich-grammar-web-2023", "synodal-russian"),
    ("dyachenko-1900-scan", "mixed"),
];

/// Returns whether a source ID and recension are explicitly admitted to the
/// Synodal data pipeline.
#[must_use]
pub fn source_recension_is_approved(id: &str, source_recension: &str) -> bool {
    APPROVED_SOURCE_RECENSIONS.contains(&(id, source_recension))
}

pub(crate) const TARGET: &str = "synodal-russian";

#[derive(Debug)]
pub enum ExtractionError {
    Io(io::Error),
    InvalidHeader {
        file: PathBuf,
        expected: &'static str,
        actual: String,
    },
    InvalidRow {
        file: PathBuf,
        line: usize,
        reason: String,
    },
    DuplicateId {
        file: PathBuf,
        id: String,
    },
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidHeader {
                file,
                expected,
                actual,
            } => write!(
                formatter,
                "invalid header in {}: expected {expected:?}, got {actual:?}",
                file.display()
            ),
            Self::InvalidRow { file, line, reason } => {
                write!(formatter, "invalid {} row {line}: {reason}", file.display())
            }
            Self::DuplicateId { file, id } => {
                write!(formatter, "duplicate ID {id:?} in {}", file.display())
            }
        }
    }
}

impl error::Error for ExtractionError {}

impl From<io::Error> for ExtractionError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, ExtractionError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationReport {
    pub lexemes: usize,
    pub principal_parts: usize,
    pub exact_forms: usize,
    pub accents: usize,
    pub alignments: usize,
    pub abbreviations: usize,
    pub positional_rules: usize,
    pub transformation_rules: usize,
    pub conflicts: usize,
    pub irregular_overrides: usize,
    pub defective_inventories: usize,
    pub irregular_inventory_entries: usize,
    pub output_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DictionaryGenerationReport {
    pub senses: usize,
    pub examples: usize,
    pub semantic_alignments: usize,
    pub output_sha256: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Table {
    pub(crate) rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateLink {
    pub(crate) source_id: String,
    pub(crate) target_recension: Option<String>,
    pub(crate) partition: Option<String>,
    pub(crate) passage: Option<String>,
    pub(crate) raw_spelling: String,
    pub(crate) normalized_spelling: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SourceInventory {
    #[serde(default)]
    pub(crate) source: Vec<SourceProvenance>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SourceProvenance {
    pub(crate) id: String,
    pub(crate) source_recension: String,
}

impl CandidateLink {
    pub(crate) fn is_target_corpus_source(&self) -> bool {
        matches!(
            self.source_id.as_str(),
            "ponomar-elizabeth-bible-2026-08-09" | "wikisource-church-slavonic-bible-2026-08-09"
        )
    }

    pub(crate) fn is_direct_target_corpus(&self) -> bool {
        self.is_target_corpus_source() && self.target_recension.as_deref() == Some(TARGET)
    }

    pub(crate) fn contains_exact(&self, form: &str) -> bool {
        let canonical_form = form.nfc().collect::<String>();
        contains_exact_token(
            &self.raw_spelling.nfc().collect::<String>(),
            &canonical_form,
        ) || contains_exact_token(
            &self.normalized_spelling.nfc().collect::<String>(),
            &canonical_form,
        )
    }
}

pub(crate) fn contains_exact_token(text: &str, token: &str) -> bool {
    !token.is_empty()
        && text.match_indices(token).any(|(start, matched)| {
            let end = start + matched.len();
            text[..start]
                .chars()
                .next_back()
                .is_none_or(|character| !is_token_component(character))
                && text[end..]
                    .chars()
                    .next()
                    .is_none_or(|character| !is_token_component(character))
        })
}

pub(crate) fn is_token_component(character: char) -> bool {
    character.is_alphabetic()
        || is_combining_mark(character)
        || character == '\u{0482}'
        || ('\u{2de0}'..='\u{2dff}').contains(&character)
}

pub(crate) const LEXICAL_REVIEW_HEADER: &str = "review_id\tlexeme_id\tsense_id\tlemma\tpart_of_speech\tcell\texpanded\tprinted\tgloss\tdomains\tsemantic_source_id\tsemantic_candidate_id\tattestation_source_id\tattestation_candidate_id\tcitation\tdecision\ttarget_recension\treview_note";

pub(crate) const V06_EXACT_REVIEW_HEADER: &str = "review_id\tdecision\troute\tfamily_id\tlexeme_id\tlemma\tpart_of_speech\tsurface\tfrequency\tcell\tsemantic_evidence_id\tmorphology_evidence_id\ttarget_evidence_id\tevaluation_candidate_id\tsource_passage\tevaluation_passage\tpredicted_unique_tokens\trealized_unique_tokens\tblocker\treview_note";

pub(crate) const PAST_CLASSIFICATION_REVIEW_HEADER: &str = "historical_review_id\tlexeme_id\tlemma\tobsolete_cell\tprinted\tdecision\treplacement_cells\tsource_passage\tevaluation_passage\treview_note";

pub(crate) const EVALUATION_HEADER: &str = "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";

pub(crate) fn read_table(
    path: &Path,
    expected_header: &'static str,
    columns: usize,
) -> Result<Table> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let actual_header = lines.next().unwrap_or_default();
    if actual_header != expected_header {
        return Err(ExtractionError::InvalidHeader {
            file: path.to_owned(),
            expected: expected_header,
            actual: actual_header.into(),
        });
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        let line_number = offset + 2;
        if line.is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split('\t').map(str::to_owned).collect();
        if fields.len() != columns {
            return Err(ExtractionError::InvalidRow {
                file: path.to_owned(),
                line: line_number,
                reason: format!("expected {columns} columns, found {}", fields.len()),
            });
        }
        for value in &fields {
            reject_forbidden_authority(path, line_number, value)?;
        }
        rows.push(fields);
    }
    Ok(Table { rows })
}
