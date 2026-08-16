//! Offline, deterministic source adapters for Synodal Russian Church Slavonic.
#![forbid(unsafe_code)]

pub mod adapters;
pub mod pipeline;

use std::{
    collections::{BTreeMap, BTreeSet},
    error, fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use synodal_church_slavonic_core::{GrammarCell, RenderedText, SynodalWord};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

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

const TARGET: &str = "synodal-russian";

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
struct Table {
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateLink {
    source_id: String,
    target_recension: Option<String>,
    partition: Option<String>,
    passage: Option<String>,
    raw_spelling: String,
    normalized_spelling: String,
}

#[derive(Debug, Deserialize)]
struct SourceInventory {
    #[serde(default)]
    source: Vec<SourceProvenance>,
}

#[derive(Debug, Deserialize)]
struct SourceProvenance {
    id: String,
    source_recension: String,
}

impl CandidateLink {
    fn is_target_corpus_source(&self) -> bool {
        matches!(
            self.source_id.as_str(),
            "ponomar-elizabeth-bible-2026-08-09" | "wikisource-church-slavonic-bible-2026-08-09"
        )
    }

    fn is_direct_target_corpus(&self) -> bool {
        self.is_target_corpus_source() && self.target_recension.as_deref() == Some(TARGET)
    }

    fn contains_exact(&self, form: &str) -> bool {
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

fn contains_exact_token(text: &str, token: &str) -> bool {
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

fn is_token_component(character: char) -> bool {
    character.is_alphabetic()
        || is_combining_mark(character)
        || character == '\u{0482}'
        || ('\u{2de0}'..='\u{2dff}').contains(&character)
}

const LEXICAL_REVIEW_HEADER: &str = "review_id\tlexeme_id\tsense_id\tlemma\tpart_of_speech\tcell\texpanded\tprinted\tgloss\tdomains\tsemantic_source_id\tsemantic_candidate_id\tattestation_source_id\tattestation_candidate_id\tcitation\tdecision\ttarget_recension\treview_note";

const V06_EXACT_REVIEW_HEADER: &str = "review_id\tdecision\troute\tfamily_id\tlexeme_id\tlemma\tpart_of_speech\tsurface\tfrequency\tcell\tsemantic_evidence_id\tmorphology_evidence_id\ttarget_evidence_id\tevaluation_candidate_id\tsource_passage\tevaluation_passage\tpredicted_unique_tokens\trealized_unique_tokens\tblocker\treview_note";

const PAST_CLASSIFICATION_REVIEW_HEADER: &str = "historical_review_id\tlexeme_id\tlemma\tobsolete_cell\tprinted\tdecision\treplacement_cells\tsource_passage\tevaluation_passage\treview_note";

const EVALUATION_HEADER: &str = "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";

fn read_lexical_reviews(data_directory: &Path) -> Result<Table> {
    read_table(
        &data_directory.join("lexical_reviews.tsv"),
        LEXICAL_REVIEW_HEADER,
        18,
    )
}

fn expected_past_classification(review_id: &str) -> &'static str {
    match review_id {
        "v06-exact-206a4cdecc4a38cd"
        | "v06-exact-3a3b6193679c4dea"
        | "v06-exact-6849b215c9f1b25b"
        | "v06-exact-bd469ab8bd4cf924" => "historical-invalid",
        "v06-exact-42beb1ca352eb0f0"
        | "v06-exact-6807a650d5010ffb"
        | "v06-exact-92d7b7c9ee19885f"
        | "v06-exact-cf7b435c4026e187" => "reclassified-imperfect",
        "v06-exact-ea4b694b16e6b4f9" => "split-contextual-homograph",
        _ => "reclassified-aorist",
    }
}

fn validate_past_classification_reviews(
    audit: (&Path, &Table),
    historical: (&Path, &Table),
    exact: (&Path, &Table),
    held_out: (&Path, &Table),
) -> Result<()> {
    let (path, reviews) = audit;
    let (historical_path, historical_reviews) = historical;
    let (exact_path, exact_forms) = exact;
    let (evaluation_path, evaluation) = held_out;
    let mut historical_past = BTreeMap::new();
    for (offset, row) in historical_reviews.rows.iter().enumerate() {
        if !row[9].starts_with("past:") {
            continue;
        }
        if historical_past.insert(row[0].as_str(), row).is_some() {
            return Err(ExtractionError::DuplicateId {
                file: historical_path.to_owned(),
                id: row[0].clone(),
            });
        }
        if row[1] != "admitted" {
            return invalid(
                historical_path,
                offset + 2,
                "historical past audit may cover only admitted v0.6 reviews",
            );
        }
    }

    let mut seen = BTreeSet::new();
    for (offset, row) in reviews.rows.iter().enumerate() {
        let line = offset + 2;
        if !seen.insert(row[0].as_str()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        let historical =
            historical_past
                .get(row[0].as_str())
                .ok_or_else(|| ExtractionError::InvalidRow {
                    file: path.to_owned(),
                    line,
                    reason: "audit row does not name a historical v0.6 past admission".into(),
                })?;
        if row[1] != historical[4]
            || row[2] != historical[5]
            || row[3] != historical[9]
            || row[4] != historical[7]
            || row[7] != historical[14]
            || row[8] != historical[15]
        {
            return invalid(
                path,
                line,
                "audit identity, form, or passage differs from its historical review",
            );
        }
        if row[9].is_empty() {
            return invalid(
                path,
                line,
                "past-classification audit requires a review note",
            );
        }

        let expected_decision = expected_past_classification(&row[0]);
        if row[5] != expected_decision {
            return invalid(
                path,
                line,
                "past-classification decision differs from the locked linguistic audit",
            );
        }
        let Some(suffix) = row[3].strip_prefix("past:") else {
            return invalid(path, line, "historical audit cell is not finite past");
        };
        let expected_replacements = match expected_decision {
            "historical-invalid" => String::new(),
            "reclassified-aorist" => format!("aorist:{suffix}"),
            "reclassified-imperfect" => format!("imperfect:{suffix}"),
            "split-contextual-homograph" => {
                format!("aorist:{suffix},imperfect:{suffix}")
            }
            _ => return invalid(path, line, "unknown past-classification decision"),
        };
        if row[6] != expected_replacements {
            return invalid(
                path,
                line,
                "replacement cells do not agree with the audited decision",
            );
        }
        for replacement in row[6].split(',').filter(|cell| !cell.is_empty()) {
            if !exact_forms
                .rows
                .iter()
                .any(|form| form[0] == row[1] && form[1] == replacement && form[3] == row[4])
            {
                return invalid(
                    exact_path,
                    1,
                    "audited finite-past replacement is absent from exact forms",
                );
            }
        }

        if expected_decision == "historical-invalid"
            && row[0].strip_prefix("v06-exact-").is_some_and(|suffix| {
                let evaluation_id = format!("eval:v06:exact-{suffix}");
                evaluation
                    .rows
                    .iter()
                    .any(|evaluation_row| evaluation_row[0] == evaluation_id)
            })
        {
            return invalid(
                evaluation_path,
                1,
                "evaluation retains a historically invalid finite-past admission",
            );
        }
    }

    if seen.len() != historical_past.len()
        || historical_past
            .keys()
            .any(|review_id| !seen.contains(review_id))
    {
        return invalid(
            path,
            1,
            "past-classification ledger does not exhaust historical v0.6 past admissions",
        );
    }
    if exact_forms
        .rows
        .iter()
        .any(|row| row[1].starts_with("past:"))
    {
        return invalid(
            exact_path,
            1,
            "target exact registry retains an underspecified finite-past cell",
        );
    }
    if evaluation
        .rows
        .iter()
        .any(|row| row[2].starts_with("past:"))
    {
        return invalid(
            evaluation_path,
            1,
            "evaluation retains an underspecified finite-past cell",
        );
    }
    Ok(())
}

fn validate_absent_target_cells(exact: (&Path, &Table), held_out: (&Path, &Table)) -> Result<()> {
    for (path, table, cell_column) in [(exact.0, exact.1, 1), (held_out.0, held_out.1, 2)] {
        for (offset, row) in table.rows.iter().enumerate() {
            if row[cell_column] == "supine" {
                return invalid(
                    path,
                    offset + 2,
                    "the Russian/Synodal target registry cannot contain the historically merged supine category",
                );
            }
        }
    }
    Ok(())
}

fn load_source_recensions(data_directory: &Path) -> Result<BTreeMap<String, String>> {
    let workspace = data_directory
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| ExtractionError::InvalidRow {
            file: data_directory.to_owned(),
            line: 1,
            reason: "Synodal data directory is not under a workspace data directory".into(),
        })?;
    let path = workspace.join("references/SOURCES.toml");
    let text = fs::read_to_string(&path)?;
    let inventory =
        toml::from_str::<SourceInventory>(&text).map_err(|error| ExtractionError::InvalidRow {
            file: path.clone(),
            line: 1,
            reason: format!("invalid source inventory: {error}"),
        })?;
    let mut recensions = BTreeMap::new();
    for source in inventory.source {
        if !source_recension_is_approved(&source.id, &source.source_recension) {
            return Err(ExtractionError::InvalidRow {
                file: path,
                line: 1,
                reason: format!(
                    "source {:?} with recension {:?} is not explicitly approved",
                    source.id, source.source_recension
                ),
            });
        }
        if recensions
            .insert(source.id.clone(), source.source_recension)
            .is_some()
        {
            return Err(ExtractionError::DuplicateId {
                file: path,
                id: source.id,
            });
        }
    }
    if recensions.len() != APPROVED_SOURCE_RECENSIONS.len() {
        return Err(ExtractionError::InvalidRow {
            file: path,
            line: 1,
            reason: "source inventory does not contain the complete approved source set".into(),
        });
    }
    Ok(recensions)
}

fn target_identity_is_adjudicated(
    ambiguities: &Table,
    candidate_id: &str,
    expanded: &str,
    printed: &str,
    left: (&str, &str),
    right: (&str, &str),
) -> bool {
    let mut analyses = [left, right];
    analyses.sort_unstable_by_key(|analysis| analysis.0);
    ambiguities.rows.iter().any(|row| {
        row[1] == candidate_id
            && row[2] == expanded
            && row[3] == printed
            && row[4] == analyses[0].0
            && row[5].split('|').any(|cell| cell == analyses[0].1)
            && row[6] == analyses[1].0
            && row[7].split('|').any(|cell| cell == analyses[1].1)
            && row[8] == "adjudicated"
    })
}

fn validate_target_identity_ambiguities(
    path: &Path,
    table: &Table,
    reviewed_evidence: &Table,
) -> Result<()> {
    let target_evidence: BTreeMap<&str, (&str, &str, &str)> = reviewed_evidence
        .rows
        .iter()
        .map(|row| {
            (
                row[0].as_str(),
                (row[1].as_str(), row[2].as_str(), row[4].as_str()),
            )
        })
        .collect();
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !row[0].starts_with("v")
            || !row[0].contains("-target-")
            || !row[1].starts_with("synodal:candidate:")
            || !row[4].starts_with("synodal:")
            || row[5].is_empty()
            || !row[6].starts_with("synodal:")
            || row[7].is_empty()
            || row[4] >= row[6]
            || row[8] != "adjudicated"
            || row[9].is_empty()
            || target_evidence.get(row[0].as_str()).is_none_or(
                |(candidate_id, source_id, decision)| {
                    *candidate_id != row[1]
                        || !is_target_corpus_source(source_id)
                        || *decision != "reviewed"
                },
            )
        {
            return invalid(
                path,
                line,
                "target identity ambiguities require stable target/candidate IDs, sorted distinct lexemes, an adjudicated decision, and a review note",
            );
        }
        validate_word(path, line, &row[2], "adjudicated expanded form")?;
        validate_word(path, line, &row[3], "adjudicated printed form")?;
        for cell in row[5].split('|').chain(row[7].split('|')) {
            cell.parse::<GrammarCell>()
                .map_err(|error| ExtractionError::InvalidRow {
                    file: path.to_owned(),
                    line,
                    reason: format!("invalid adjudicated grammar cell {cell:?}: {error}"),
                })?;
        }
        if !keys.insert(row[..9].to_vec()) {
            return invalid(path, line, "duplicate target identity ambiguity");
        }
    }
    Ok(())
}

fn validate_lexical_reviews(path: &Path, table: &Table, ambiguities: &Table) -> Result<()> {
    let mut review_ids = BTreeSet::new();
    let mut lexeme_ids = BTreeSet::new();
    let mut sense_ids = BTreeSet::new();
    let mut attested_tokens: BTreeMap<_, BTreeSet<(String, String)>> = BTreeMap::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !review_ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if !matches!(row[15].as_str(), "reviewed" | "rejected") {
            return invalid(path, line, "lexical decision must be reviewed or rejected");
        }
        validate_target(path, line, &row[16])?;
        if !row[11].starts_with("synodal:candidate:") || !row[13].starts_with("synodal:candidate:")
        {
            return invalid(
                path,
                line,
                "lexical reviews require stable semantic and attestation candidate IDs",
            );
        }
        if row[17].is_empty() {
            return invalid(
                path,
                line,
                "lexical reviews require an explicit review note",
            );
        }
        if row[15] == "rejected" {
            continue;
        }
        if !lexeme_ids.insert(row[1].clone()) || !sense_ids.insert(row[2].clone()) {
            return invalid(path, line, "reviewed lexeme and sense IDs must be unique");
        }
        if !row[1].starts_with("synodal:")
            || !row[2].starts_with("sense:")
            || row[8].is_empty()
            || row[10].is_empty()
            || row[12].is_empty()
            || row[14].is_empty()
        {
            return invalid(
                path,
                line,
                "reviewed lexical decisions require stable IDs, a gloss, both sources, and a citation",
            );
        }
        validate_word(path, line, &row[3], "reviewed lemma")?;
        validate_word(path, line, &row[6], "reviewed expanded form")?;
        validate_word(path, line, &row[7], "reviewed printed form")?;
        let attested_token = (row[13].clone(), row[6].clone(), row[7].clone());
        let identities = attested_tokens.entry(attested_token.clone()).or_default();
        for (previous_lexeme, previous_cell) in identities.iter() {
            if previous_lexeme != &row[1]
                && !target_identity_is_adjudicated(
                    ambiguities,
                    &row[13],
                    &row[6],
                    &row[7],
                    (previous_lexeme, previous_cell),
                    (&row[1], &row[5]),
                )
            {
                return invalid(
                    path,
                    line,
                    &format!(
                        "target candidate/token {attested_token:?} cannot confirm incompatible lexical identities without contextual adjudication"
                    ),
                );
            }
        }
        identities.insert((row[1].clone(), row[5].clone()));
        let closed = matches!(
            row[4].as_str(),
            "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
        );
        let inflectable = matches!(
            row[4].as_str(),
            "proper-noun"
                | "noun"
                | "adjective"
                | "verb"
                | "pronoun"
                | "determiner"
                | "numeral"
                | "participle"
        );
        if (!closed && !inflectable)
            || (closed && row[5] != "indeclinable")
            || (inflectable && row[5] != "lexical-form")
        {
            return invalid(
                path,
                line,
                "part of speech must use the matching exact-only lexical cell",
            );
        }
    }
    Ok(())
}

type AdmittedLexicalReviewRows = (Vec<Vec<String>>, Vec<Vec<String>>, Vec<Vec<String>>);

fn extend_missing_lexemes(
    path: &Path,
    lexemes: &mut Table,
    reviewed: Vec<Vec<String>>,
    reviewed_exact_forms: &[Vec<String>],
) -> Result<()> {
    let mut rows_by_id = lexemes
        .rows
        .iter()
        .enumerate()
        .map(|(offset, row)| (row[0].clone(), offset))
        .collect::<BTreeMap<_, _>>();
    for row in reviewed {
        if let Some(offset) = rows_by_id.get(&row[0]).copied() {
            let existing = &lexemes.rows[offset];
            let lemma_matches = existing[1] == row[1]
                || reviewed_exact_forms.iter().any(|form| {
                    form[0] == row[0] && form[1] == "lexical-form" && form[2] == existing[1]
                });
            if !lemma_matches || existing[2] != row[2] || existing[8] != row[8] {
                return invalid(
                    path,
                    offset + 2,
                    "a productive lexical upgrade must preserve the reviewed source or exact target citation, part of speech, and target recension",
                );
            }
            continue;
        }
        rows_by_id.insert(row[0].clone(), lexemes.rows.len());
        lexemes.rows.push(row);
    }
    Ok(())
}

fn extend_reviewed_exact_forms(
    path: &Path,
    exact_forms: &mut Table,
    reviewed: Vec<Vec<String>>,
) -> Result<()> {
    let mut rows_by_key = exact_forms
        .rows
        .iter()
        .enumerate()
        .map(|(offset, row)| {
            (
                (
                    row[0].clone(),
                    row[1].clone(),
                    row[2].clone(),
                    row[3].clone(),
                ),
                offset,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for row in reviewed {
        let key = (
            row[0].clone(),
            row[1].clone(),
            row[2].clone(),
            row[3].clone(),
        );
        if let Some(offset) = rows_by_key.get(&key).copied() {
            let existing = &mut exact_forms.rows[offset];
            if existing[5] != row[5] || existing[6] != row[6] {
                return invalid(
                    path,
                    offset + 2,
                    "reviewed lexical form conflicts with an exact row's source kind or target recension",
                );
            }
            let mut evidence = existing[4]
                .split(',')
                .map(str::to_owned)
                .collect::<Vec<_>>();
            for id in row[4].split(',') {
                if !evidence.iter().any(|existing| existing == id) {
                    evidence.push(id.to_owned());
                }
            }
            existing[4] = evidence.join(",");
        } else {
            rows_by_key.insert(key, exact_forms.rows.len());
            exact_forms.rows.push(row);
        }
    }
    Ok(())
}

fn admitted_lexical_review_rows(
    reviews: &Table,
    source_recensions: &BTreeMap<String, String>,
) -> Result<AdmittedLexicalReviewRows> {
    let mut lexemes = Vec::new();
    let mut exact_forms = Vec::new();
    let mut senses = Vec::new();
    for row in reviews.rows.iter().filter(|row| row[15] == "reviewed") {
        require_direct_target_source(&row[12], source_recensions)?;
        let source_recension =
            source_recensions
                .get(&row[10])
                .ok_or_else(|| ExtractionError::InvalidRow {
                    file: PathBuf::from("references/SOURCES.toml"),
                    line: 1,
                    reason: format!("reviewed semantic source {:?} is not registered", row[10]),
                })?;
        let semantic_status = match source_recension.as_str() {
            "old-church-slavonic" => "reviewed-ocs-inheritance",
            "mixed" => "reviewed-with-synodal-corpus",
            "synodal-russian" => "normative",
            value => {
                return Err(ExtractionError::InvalidRow {
                    file: PathBuf::from("references/SOURCES.toml"),
                    line: 1,
                    reason: format!(
                        "semantic source {:?} has unsupported recension {value:?}",
                        row[10]
                    ),
                });
            }
        };
        lexemes.push(vec![
            row[1].clone(),
            row[3].clone(),
            row[4].clone(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            row[12].clone(),
            row[16].clone(),
        ]);
        exact_forms.push(vec![
            row[1].clone(),
            row[5].clone(),
            row[6].clone(),
            row[7].clone(),
            row[0].clone(),
            "synodal-attestation".into(),
            row[16].clone(),
        ]);
        senses.push(vec![
            row[1].clone(),
            row[2].clone(),
            row[8].clone(),
            row[9].clone(),
            row[10].clone(),
            source_recension.clone(),
            semantic_status.into(),
        ]);
    }
    Ok((lexemes, exact_forms, senses))
}

/// Validates reviewable TSV and atomically writes the generated Rust registry.
pub fn generate_registry(data_directory: &Path, destination: &Path) -> Result<GenerationReport> {
    let lexeme_path = data_directory.join("lexemes.tsv");
    let noun_restriction_path = data_directory.join("noun_restrictions.tsv");
    let principal_path = data_directory.join("principal_parts.tsv");
    let exact_path = data_directory.join("exact_forms.tsv");
    let alignment_path = data_directory.join("alignments.tsv");
    let abbreviation_path = data_directory.join("abbreviations.tsv");
    let abbreviation_family_path = data_directory.join("abbreviation_families.tsv");
    let abbreviation_inventory_path = data_directory.join("abbreviation_inventory.tsv");
    let accent_path = data_directory.join("accents.tsv");
    let accent_paradigm_path = data_directory.join("accent_paradigms.tsv");
    let positional_path = data_directory.join("positional_rules.tsv");
    let transformation_path = data_directory.join("transformation_rules.tsv");
    let conflict_path = data_directory.join("conflicts.tsv");
    let irregular_path = data_directory.join("irregular_overrides.tsv");
    let defective_inventory_path = data_directory.join("verb_defectiveness.tsv");
    let irregular_inventory_path = data_directory.join("irregular_verb_inventory.tsv");
    let target_identity_ambiguity_path = data_directory.join("target_identity_ambiguities.tsv");
    let past_classification_review_path = data_directory.join("past_classification_reviews.tsv");
    let v06_exact_review_path = data_directory.join("v06_exact_reviews.tsv");
    let evaluation_path = data_directory.join("evaluation.tsv");

    let mut lexemes = read_table(
        &lexeme_path,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
    )?;
    let noun_restrictions = read_table(
        &noun_restriction_path,
        "lexeme_id\tnumber_inventory\tevidence_id\ttarget_recension",
        4,
    )?;
    let principal_parts = read_table(
        &principal_path,
        "lexeme_id\tsystem\tvalue\tformation\tevidence_id\ttarget_recension",
        6,
    )?;
    let mut exact_forms = read_table(
        &exact_path,
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension",
        7,
    )?;
    let alignments = read_table(
        &alignment_path,
        "mapping_id\tsource_lexeme_id\ttarget_lexeme_id\trelation\tstatus\tmorphology\tsemantics\tconfidence_bp\tevidence_ids\ttransformations\treview_note",
        11,
    )?;
    let abbreviations = read_table(
        &abbreviation_path,
        "lexeme_id\tsense_id\tcell\texpanded\tprinted\trule_id\tevidence_id\treversible\trequired_marks\tcontext_restrictions\tambiguity\tsource_recension\ttarget_recension",
        13,
    )?;
    let abbreviation_families = read_table(
        &abbreviation_family_path,
        "lexeme_id\tsense_id\texpanded_prefix\tprinted_prefix\trule_id\tevidence_id\treversible\trequired_marks\tcontext_restrictions\tambiguity\tsource_recension\ttarget_recension",
        12,
    )?;
    let abbreviation_inventory = read_table(
        &abbreviation_inventory_path,
        "source_order\tprinted_head\texpanded_head\tsemantic_scope\tdecision\tlexeme_id\tsense_id\trule_id\tevidence_id\treview_note\ttarget_recension",
        11,
    )?;
    let accents = read_table(
        &accent_path,
        "lexeme_id\tcell\texpanded\taccented\tevidence_id\tsource_id\tsource_recension\ttarget_recension",
        8,
    )?;
    let accent_paradigms = read_table(
        &accent_paradigm_path,
        "lexeme_id\tparadigm_id\tscope\tplacement\tmark\tbreathing\tevidence_id\tsource_id\tcitation\tsource_recension\ttarget_recension",
        11,
    )?;
    let positional_rules = read_table(
        &positional_path,
        "rule_id\tinput\tcontext\toutput\texceptions\tevidence_id\ttarget_recension",
        7,
    )?;
    let transformation_rules = read_table(
        &transformation_path,
        "rule_id\tsource_recension\ttarget_recension\toperation\tstatus\tevidence_id",
        6,
    )?;
    let conflicts = read_table(
        &conflict_path,
        "conflict_id\tsource_lexeme_id\ttarget_lexeme_id\tkind\tstatus\tsupporting_evidence\tcontradicting_evidence\tresolution",
        8,
    )?;
    let irregular_overrides = read_table(
        &irregular_path,
        "lexeme_id\tsystem\tcell_set\tevidence_id\ttarget_recension",
        5,
    )?;
    let defective_inventories = read_table(
        &defective_inventory_path,
        "lexeme_id\tmode\tselector\tkind\tmetadata_field\treason\tevidence_id\ttarget_recension",
        8,
    )?;
    let irregular_inventory = read_table(
        &irregular_inventory_path,
        "source_order\theadword\tsystems\tstrategy\timplementation_status\tevidence_id\tnote\ttarget_recension",
        8,
    )?;
    let reviewed_evidence = read_reviewed_evidence(data_directory)?;
    let past_classification_reviews = read_table(
        &past_classification_review_path,
        PAST_CLASSIFICATION_REVIEW_HEADER,
        10,
    )?;
    let v06_exact_reviews = read_table(&v06_exact_review_path, V06_EXACT_REVIEW_HEADER, 20)?;
    let evaluation = read_table(&evaluation_path, EVALUATION_HEADER, 9)?;
    let lexical_review_path = data_directory.join("lexical_reviews.tsv");
    let lexical_reviews = read_lexical_reviews(data_directory)?;
    let target_identity_ambiguities = read_table(
        &target_identity_ambiguity_path,
        "evidence_id\tcandidate_id\texpanded\tprinted\tleft_lexeme_id\tleft_cells\tright_lexeme_id\tright_cells\tdecision\treview_note",
        10,
    )?;
    validate_target_identity_ambiguities(
        &target_identity_ambiguity_path,
        &target_identity_ambiguities,
        &reviewed_evidence,
    )?;
    validate_lexical_reviews(
        &lexical_review_path,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    let source_recensions = load_source_recensions(data_directory)?;
    let evidence_provenance =
        evidence_provenance_rows(&reviewed_evidence, &lexical_reviews, &source_recensions)?;
    let (review_lexemes, review_exact_forms, _) =
        admitted_lexical_review_rows(&lexical_reviews, &source_recensions)?;
    // A later engine release may add independently reviewed productive
    // metadata for an identity first admitted by a lexical review. Preserve
    // that richer direct row instead of materializing a second exact-only
    // lexeme with the same stable ID.
    extend_missing_lexemes(
        &lexeme_path,
        &mut lexemes,
        review_lexemes,
        &review_exact_forms,
    )?;
    extend_reviewed_exact_forms(&exact_path, &mut exact_forms, review_exact_forms)?;

    validate_lexemes(&lexeme_path, &lexemes)?;
    validate_noun_restrictions(&noun_restriction_path, &noun_restrictions)?;
    validate_noun_restriction_lexemes(&noun_restriction_path, &noun_restrictions, &lexemes)?;
    validate_principal_parts(&principal_path, &principal_parts)?;
    validate_exact_forms(&exact_path, &exact_forms, &lexemes)?;
    validate_past_classification_reviews(
        (
            &past_classification_review_path,
            &past_classification_reviews,
        ),
        (&v06_exact_review_path, &v06_exact_reviews),
        (&exact_path, &exact_forms),
        (&evaluation_path, &evaluation),
    )?;
    validate_absent_target_cells((&exact_path, &exact_forms), (&evaluation_path, &evaluation))?;
    validate_exact_form_attestation_evidence(
        &exact_path,
        &exact_forms,
        &evidence_provenance,
        &reviewed_evidence,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    validate_alignments(&alignment_path, &alignments)?;
    validate_abbreviations(&abbreviation_path, &abbreviations, &lexemes)?;
    validate_abbreviation_families(
        &abbreviation_family_path,
        &abbreviation_families,
        &abbreviations,
        &lexemes,
    )?;
    validate_abbreviation_inventory(
        &abbreviation_inventory_path,
        &abbreviation_inventory,
        &abbreviation_families,
    )?;
    validate_accents(&accent_path, &accents)?;
    validate_accent_paradigms(&accent_paradigm_path, &accent_paradigms)?;
    validate_positional_rules(&positional_path, &positional_rules)?;
    validate_transformation_rules(&transformation_path, &transformation_rules)?;
    validate_conflicts(&conflict_path, &conflicts)?;
    validate_conflict_evidence(&conflict_path, &conflicts, &reviewed_evidence)?;
    validate_irregular_overrides(&irregular_path, &irregular_overrides)?;
    validate_defective_inventories(&defective_inventory_path, &defective_inventories, &lexemes)?;
    validate_irregular_verb_inventory(&irregular_inventory_path, &irregular_inventory)?;
    validate_morphology_evidence(
        data_directory,
        &reviewed_evidence,
        &lexical_reviews,
        [
            (&principal_parts, &[4_usize][..]),
            (&exact_forms, &[4][..]),
            (&alignments, &[8][..]),
            (&abbreviations, &[6][..]),
            (&abbreviation_families, &[5][..]),
            (&abbreviation_inventory, &[8][..]),
            (&accents, &[4][..]),
            (&accent_paradigms, &[6][..]),
            (&noun_restrictions, &[2][..]),
            (&positional_rules, &[5][..]),
            (&transformation_rules, &[5][..]),
            (&irregular_overrides, &[3][..]),
            (&defective_inventories, &[6][..]),
            (&irregular_inventory, &[5][..]),
        ],
    )?;
    validate_morphology_references(
        &lexeme_path,
        &lexemes,
        [
            (&principal_path, &principal_parts, 0),
            (&exact_path, &exact_forms, 0),
            (&abbreviation_path, &abbreviations, 0),
            (&abbreviation_family_path, &abbreviation_families, 0),
            (&accent_path, &accents, 0),
            (&accent_paradigm_path, &accent_paradigms, 0),
            (&noun_restriction_path, &noun_restrictions, 0),
            (&irregular_path, &irregular_overrides, 0),
            (&defective_inventory_path, &defective_inventories, 0),
        ],
    )?;
    validate_alignment_references(
        &alignment_path,
        &alignments,
        &lexemes,
        &transformation_rules,
        &conflict_path,
        &conflicts,
    )?;

    let output = emit_registry(RegistryTables {
        lexemes: lexemes.clone(),
        noun_restrictions: noun_restrictions.clone(),
        principal_parts: principal_parts.clone(),
        exact_forms: exact_forms.clone(),
        alignments: alignments.clone(),
        abbreviations: abbreviations.clone(),
        abbreviation_families: abbreviation_families.clone(),
        accents: accents.clone(),
        accent_paradigms: accent_paradigms.clone(),
        positional_rules: positional_rules.clone(),
        transformation_rules: transformation_rules.clone(),
        conflicts: conflicts.clone(),
        irregular_overrides: irregular_overrides.clone(),
        defective_inventories: defective_inventories.clone(),
        irregular_inventory: irregular_inventory.clone(),
        evidence_provenance,
    });
    let output_sha256 = hex_sha256(output.as_bytes());
    atomic_write(destination, output.as_bytes())?;

    Ok(GenerationReport {
        lexemes: lexemes.rows.len(),
        principal_parts: principal_parts.rows.len(),
        exact_forms: exact_forms.rows.len(),
        accents: accents.rows.len(),
        alignments: alignments.rows.len(),
        abbreviations: abbreviations.rows.len(),
        positional_rules: positional_rules.rows.len(),
        transformation_rules: transformation_rules.rows.len(),
        conflicts: conflicts.rows.len(),
        irregular_overrides: irregular_overrides.rows.len(),
        defective_inventories: defective_inventories.rows.len(),
        irregular_inventory_entries: irregular_inventory.rows.len(),
        output_sha256,
    })
}

/// Validates semantic/reference TSV and writes the dictionary's static registry.
pub fn generate_dictionary_registry(
    data_directory: &Path,
    destination: &Path,
) -> Result<DictionaryGenerationReport> {
    let sense_path = data_directory.join("senses.tsv");
    let example_path = data_directory.join("examples.tsv");
    let semantic_alignment_path = data_directory.join("semantic_alignments.tsv");
    let mut senses = read_table(
        &sense_path,
        "lexeme_id\tsense_id\tgloss\tdomains\tsource_id\tsource_recension\tsemantic_status",
        7,
    )?;
    let examples = read_table(
        &example_path,
        "example_id\tlexeme_id\ttext\ttranslation\tsource_id\tpassage\tsource_recension\ttarget_recension\tpartition",
        9,
    )?;
    let semantic_alignments = read_table(
        &semantic_alignment_path,
        "mapping_id\tsource_sense_id\ttarget_sense_id\tstatus\tevidence_id\treview_note",
        6,
    )?;
    let lexical_review_path = data_directory.join("lexical_reviews.tsv");
    let lexical_reviews = read_lexical_reviews(data_directory)?;
    let reviewed_evidence = read_reviewed_evidence(data_directory)?;
    let target_identity_ambiguity_path = data_directory.join("target_identity_ambiguities.tsv");
    let target_identity_ambiguities = read_table(
        &target_identity_ambiguity_path,
        "evidence_id\tcandidate_id\texpanded\tprinted\tleft_lexeme_id\tleft_cells\tright_lexeme_id\tright_cells\tdecision\treview_note",
        10,
    )?;
    validate_target_identity_ambiguities(
        &target_identity_ambiguity_path,
        &target_identity_ambiguities,
        &reviewed_evidence,
    )?;
    validate_lexical_reviews(
        &lexical_review_path,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    let source_recensions = load_source_recensions(data_directory)?;
    let (review_lexemes, review_exact_forms, review_senses) =
        admitted_lexical_review_rows(&lexical_reviews, &source_recensions)?;
    senses.rows.extend(review_senses);
    validate_senses(&sense_path, &senses, &source_recensions)?;
    validate_examples(&example_path, &examples)?;
    validate_semantic_alignments(&semantic_alignment_path, &semantic_alignments)?;
    validate_semantic_alignment_evidence(
        &semantic_alignment_path,
        &semantic_alignments,
        &reviewed_evidence,
    )?;
    let lexeme_path = data_directory.join("lexemes.tsv");
    let mut lexemes = read_table(
        &lexeme_path,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
    )?;
    extend_missing_lexemes(
        &lexeme_path,
        &mut lexemes,
        review_lexemes,
        &review_exact_forms,
    )?;
    let morphology_alignments = read_table(
        &data_directory.join("alignments.tsv"),
        "mapping_id\tsource_lexeme_id\ttarget_lexeme_id\trelation\tstatus\tmorphology\tsemantics\tconfidence_bp\tevidence_ids\ttransformations\treview_note",
        11,
    )?;
    validate_dictionary_references(
        &sense_path,
        &senses,
        &example_path,
        &examples,
        &semantic_alignment_path,
        &semantic_alignments,
        &lexemes,
        &morphology_alignments,
    )?;

    let output = emit_dictionary_registry(
        senses.clone(),
        examples.clone(),
        semantic_alignments.clone(),
    );
    let output_sha256 = hex_sha256(output.as_bytes());
    atomic_write(destination, output.as_bytes())?;
    Ok(DictionaryGenerationReport {
        senses: senses.rows.len(),
        examples: examples.rows.len(),
        semantic_alignments: semantic_alignments.rows.len(),
        output_sha256,
    })
}

fn read_reviewed_evidence(data_directory: &Path) -> Result<Table> {
    let path = data_directory.join("reviewed_evidence.tsv");
    let table = read_table(
        &path,
        "evidence_id\tcandidate_id\tsource_id\tcitation\tdecision\ttarget_recension\treview_note",
        7,
    )?;
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.clone(),
                id: row[0].clone(),
            });
        }
        if !row[1].starts_with("synodal:candidate:") {
            return invalid(
                &path,
                offset + 2,
                "review evidence requires a stable candidate ID",
            );
        }
        if !matches!(row[4].as_str(), "reviewed" | "rejected") {
            return invalid(
                &path,
                offset + 2,
                "review decision must be reviewed or rejected",
            );
        }
        validate_target(&path, offset + 2, &row[5])?;
    }
    Ok(table)
}

fn validate_morphology_evidence<const N: usize>(
    data_directory: &Path,
    reviewed: &Table,
    lexical_reviews: &Table,
    tables: [(&Table, &[usize]); N],
) -> Result<()> {
    let evidence_path = data_directory.join("reviewed_evidence.tsv");
    let known: BTreeSet<&str> = reviewed
        .rows
        .iter()
        .filter(|row| row[4] == "reviewed")
        .map(|row| row[0].as_str())
        .chain(
            lexical_reviews
                .rows
                .iter()
                .filter(|row| row[15] == "reviewed")
                .map(|row| row[0].as_str()),
        )
        .collect();
    for (table, columns) in tables {
        for row in &table.rows {
            for &column in columns {
                for evidence_id in row[column]
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    if !known.contains(evidence_id) {
                        return invalid(
                            &evidence_path,
                            1,
                            &format!(
                                "runtime or review fact has unregistered evidence {evidence_id:?}"
                            ),
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn evidence_provenance_rows(
    reviewed: &Table,
    lexical_reviews: &Table,
    source_recensions: &BTreeMap<String, String>,
) -> Result<Table> {
    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    for row in reviewed.rows.iter().filter(|row| row[4] == "reviewed") {
        let source_recension =
            source_recensions
                .get(&row[2])
                .ok_or_else(|| ExtractionError::InvalidRow {
                    file: PathBuf::from("references/SOURCES.toml"),
                    line: 0,
                    reason: format!(
                        "reviewed evidence {} uses unregistered source {}",
                        row[0], row[2]
                    ),
                })?;
        if !ids.insert(row[0].clone()) {
            return invalid(
                &PathBuf::from("data/synodal/reviewed_evidence.tsv"),
                0,
                "duplicate runtime evidence provenance ID",
            );
        }
        let role = if is_target_corpus_source(&row[2]) {
            "target-attestation"
        } else if source_recension == TARGET {
            "synodal-authority"
        } else if source_recension == "old-church-slavonic" {
            "ocs-evidence"
        } else {
            "comparative-evidence"
        };
        rows.push(vec![
            row[0].clone(),
            row[2].clone(),
            source_recension.clone(),
            row[3].clone(),
            role.into(),
            row[6].clone(),
        ]);
    }
    for row in lexical_reviews
        .rows
        .iter()
        .filter(|row| row[15] == "reviewed")
    {
        let source_recension = require_direct_target_source(&row[12], source_recensions)?;
        if !ids.insert(row[0].clone()) {
            return invalid(
                &PathBuf::from("data/synodal/lexical_reviews.tsv"),
                0,
                "duplicate runtime evidence provenance ID",
            );
        }
        rows.push(vec![
            row[0].clone(),
            row[12].clone(),
            source_recension.into(),
            row[14].clone(),
            format!("reviewed-cell:{}", row[5]),
            row[17].clone(),
        ]);
    }
    rows.sort();
    Ok(Table { rows })
}

fn validate_exact_form_attestation_evidence(
    path: &Path,
    exact_forms: &Table,
    evidence_provenance: &Table,
    reviewed_evidence: &Table,
    lexical_reviews: &Table,
    ambiguities: &Table,
) -> Result<()> {
    let roles: BTreeMap<&str, &str> = evidence_provenance
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row[4].as_str()))
        .collect();
    let target_candidates: BTreeMap<&str, &str> = reviewed_evidence
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row[1].as_str()))
        .collect();
    let reviewed_cell_owners: BTreeMap<&str, (&str, &str)> = lexical_reviews
        .rows
        .iter()
        .filter(|row| row[15] == "reviewed")
        .map(|row| (row[0].as_str(), (row[1].as_str(), row[5].as_str())))
        .collect();
    let mut attested_tokens: BTreeMap<_, BTreeSet<(String, String)>> = BTreeMap::new();
    for (offset, row) in exact_forms.rows.iter().enumerate() {
        let target_attestations = row[4]
            .split(',')
            .map(str::trim)
            .filter(|evidence_id| roles.get(evidence_id) == Some(&"target-attestation"))
            .collect::<Vec<_>>();
        for evidence_id in &target_attestations {
            let candidate_id = target_candidates.get(evidence_id).ok_or_else(|| {
                ExtractionError::InvalidRow {
                    file: path.to_owned(),
                    line: offset + 2,
                    reason: format!(
                        "target-attestation evidence {evidence_id} has no reviewed candidate provenance"
                    ),
                }
            })?;
            let attested_token = ((*candidate_id).to_owned(), row[2].clone(), row[3].clone());
            let identities = attested_tokens.entry(attested_token.clone()).or_default();
            for (previous_lexeme, previous_cell) in identities.iter() {
                if previous_lexeme != &row[0]
                    && !target_identity_is_adjudicated(
                        ambiguities,
                        candidate_id,
                        &row[2],
                        &row[3],
                        (previous_lexeme, previous_cell),
                        (&row[0], &row[1]),
                    )
                {
                    return invalid(
                        path,
                        offset + 2,
                        &format!(
                            "target evidence/token {attested_token:?} cannot license incompatible lexical identities without contextual adjudication"
                        ),
                    );
                }
            }
            identities.insert((row[0].clone(), row[1].clone()));
        }
        let has_target_attestation = !target_attestations.is_empty();
        let has_reviewed_cell_attestation = row[4].split(',').map(str::trim).any(|evidence_id| {
            reviewed_cell_owners.get(evidence_id) == Some(&(row[0].as_str(), row[1].as_str()))
        });
        if row[5] == "synodal-attestation"
            && !has_target_attestation
            && !has_reviewed_cell_attestation
        {
            return invalid(
                path,
                offset + 2,
                &format!(
                    "Synodal attestation {} {} requires distinct target-recension evidence (found {})",
                    row[0], row[1], row[4]
                ),
            );
        }
    }
    Ok(())
}

fn is_target_corpus_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "ponomar-elizabeth-bible-2026-08-09" | "wikisource-church-slavonic-bible-2026-08-09"
    )
}

fn require_direct_target_source<'a>(
    source_id: &str,
    source_recensions: &'a BTreeMap<String, String>,
) -> Result<&'a str> {
    let source_recension =
        source_recensions
            .get(source_id)
            .ok_or_else(|| ExtractionError::InvalidRow {
                file: PathBuf::from("references/SOURCES.toml"),
                line: 0,
                reason: format!("unregistered lexical attestation source {source_id:?}"),
            })?;
    if !is_target_corpus_source(source_id) || source_recension != TARGET {
        return Err(ExtractionError::InvalidRow {
            file: PathBuf::from("data/synodal/lexical_reviews.tsv"),
            line: 0,
            reason: format!(
                "lexical attestation source {source_id:?} is not an approved direct target corpus"
            ),
        });
    }
    Ok(source_recension)
}

/// Proves that committed review decisions still name candidates produced from
/// the current locked bytes. This runs only in a full bootstrap because partial
/// source bootstraps intentionally materialize a subset.
pub fn validate_candidate_links(data_directory: &Path, intermediate: &Path) -> Result<()> {
    let evidence = read_reviewed_evidence(data_directory)?;
    let lexical_reviews = read_lexical_reviews(data_directory)?;
    let evaluation = read_table(
        &data_directory.join("evaluation.tsv"),
        "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity",
        9,
    )?;
    let abbreviation_evaluation = read_table(
        &data_directory.join("abbreviation_evaluation.tsv"),
        "id\tlexeme_id\tsense_id\tcell\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity",
        9,
    )?;
    let evaluation_passages = evaluation
        .rows
        .iter()
        .chain(&abbreviation_evaluation.rows)
        .map(|row| (row[6].clone(), row[7].clone()))
        .collect::<BTreeSet<_>>();
    let wanted_candidates: BTreeSet<&str> = evidence
        .rows
        .iter()
        .map(|row| row[1].as_str())
        .chain(
            lexical_reviews
                .rows
                .iter()
                .flat_map(|row| [row[11].as_str(), row[13].as_str()]),
        )
        .collect();
    let mut candidates = BTreeMap::<String, CandidateLink>::new();
    for entry in fs::read_dir(intermediate)? {
        let path = entry?.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "jsonl")
        {
            continue;
        }
        for (offset, line) in fs::read_to_string(&path)?.lines().enumerate() {
            let value: serde_json::Value =
                serde_json::from_str(line).map_err(|error| ExtractionError::InvalidRow {
                    file: path.clone(),
                    line: offset + 1,
                    reason: format!("invalid candidate JSON: {error}"),
                })?;
            if let Some(candidate_id) = value
                .get("candidate_id")
                .and_then(serde_json::Value::as_str)
            {
                if !wanted_candidates.contains(candidate_id) {
                    continue;
                }
                let source_id = value
                    .get("source_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| ExtractionError::InvalidRow {
                        file: path.clone(),
                        line: offset + 1,
                        reason: "candidate has no source_id".into(),
                    })?;
                let target_recension = value
                    .get("target_recension")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                let metadata = CandidateLink {
                    source_id: source_id.to_owned(),
                    target_recension,
                    partition: value
                        .get("partition")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    passage: value
                        .get("passage")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    raw_spelling: value
                        .get("raw_spelling")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                    normalized_spelling: value
                        .get("normalized_spelling")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                };
                if candidates
                    .insert(candidate_id.to_owned(), metadata.clone())
                    .is_some_and(|previous| previous != metadata)
                {
                    return invalid(
                        &path,
                        offset + 1,
                        &format!("candidate {candidate_id} has conflicting source metadata"),
                    );
                }
            }
        }
    }
    let evidence_candidates: BTreeMap<&str, &CandidateLink> = evidence
        .rows
        .iter()
        .filter_map(|row| {
            candidates
                .get(&row[1])
                .map(|candidate| (row[0].as_str(), candidate))
        })
        .collect();
    for (offset, row) in evidence.rows.iter().enumerate() {
        let Some(candidate) = candidates.get(&row[1]) else {
            return invalid(
                &data_directory.join("reviewed_evidence.tsv"),
                offset + 2,
                &format!("reviewed candidate {} disappeared or changed", row[1]),
            );
        };
        let target_mismatch = candidate.target_recension.as_deref().map_or_else(
            || candidate.is_target_corpus_source(),
            |target| target != row[5],
        );
        if candidate.source_id != row[2] || target_mismatch {
            return invalid(
                &data_directory.join("reviewed_evidence.tsv"),
                offset + 2,
                &format!(
                    "reviewed candidate {} has mismatched source or target metadata",
                    row[1]
                ),
            );
        }
        if candidate.is_direct_target_corpus()
            && candidate.passage.as_deref() != Some(row[3].as_str())
        {
            return invalid(
                &data_directory.join("reviewed_evidence.tsv"),
                offset + 2,
                &format!(
                    "reviewed corpus evidence {} must cite its exact candidate passage",
                    row[0]
                ),
            );
        }
    }

    let exact_path = data_directory.join("exact_forms.tsv");
    let exact_forms = read_table(
        &exact_path,
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension",
        7,
    )?;
    for (offset, row) in exact_forms.rows.iter().enumerate() {
        if row[5] != "synodal-attestation" {
            continue;
        }
        let has_exact_source_witness = row[4]
            .split(',')
            .filter_map(|evidence_id| evidence_candidates.get(evidence_id))
            .any(|candidate| {
                candidate.is_direct_target_corpus()
                    && candidate.partition.as_deref() == Some("source")
                    && candidate.contains_exact(&row[3])
            });
        if !has_exact_source_witness {
            return invalid(
                &exact_path,
                offset + 2,
                "a Synodal attestation requires an exact source-partition corpus witness",
            );
        }
    }

    for evidence_id in runtime_evidence_ids(data_directory)? {
        let Some(candidate) = evidence_candidates.get(evidence_id.as_str()) else {
            continue;
        };
        if candidate.is_direct_target_corpus()
            && candidate.partition.as_deref() == Some("evaluation")
        {
            return invalid(
                &data_directory.join("reviewed_evidence.tsv"),
                1,
                &format!(
                    "runtime evidence {evidence_id:?} may not use an evaluation-partition corpus candidate"
                ),
            );
        }
        if candidate.is_direct_target_corpus()
            && candidate.passage.as_ref().is_some_and(|passage| {
                evaluation_passages.contains(&(candidate.source_id.clone(), passage.clone()))
            })
        {
            return invalid(
                &data_directory.join("reviewed_evidence.tsv"),
                1,
                &format!(
                    "runtime evidence {evidence_id:?} shares a passage with held-out evaluation"
                ),
            );
        }
    }

    let review_path = data_directory.join("lexical_reviews.tsv");
    for (offset, row) in lexical_reviews.rows.iter().enumerate() {
        for (candidate_column, source_column, require_target, label) in [
            (11_usize, 10_usize, false, "semantic"),
            (13_usize, 12_usize, true, "attestation"),
        ] {
            let Some(candidate) = candidates.get(&row[candidate_column]) else {
                return invalid(
                    &review_path,
                    offset + 2,
                    &format!(
                        "reviewed {label} candidate {} disappeared or changed",
                        row[candidate_column]
                    ),
                );
            };
            let target_matches_role = if require_target {
                candidate.target_recension.as_deref() == Some(TARGET)
            } else {
                // Semantic identity may come either from an inherited source
                // with no target claim or from an independently sourced
                // Synodal normative work. The latter still requires a
                // separate target-passage attestation below.
                candidate
                    .target_recension
                    .as_deref()
                    .is_none_or(|target| target == TARGET)
            };
            if candidate.source_id != row[source_column] || !target_matches_role {
                return invalid(
                    &review_path,
                    offset + 2,
                    &format!(
                        "reviewed {label} candidate {} has mismatched source or recension metadata",
                        row[candidate_column]
                    ),
                );
            }
            if require_target
                && row[15] == "reviewed"
                && (candidate.partition.as_deref() != Some("source")
                    || !candidate.contains_exact(&row[7])
                    || candidate.passage.as_deref() != Some(row[14].as_str()))
            {
                return invalid(
                    &review_path,
                    offset + 2,
                    "reviewed lexical attestation must match its exact source-partition form and passage",
                );
            }
            if require_target
                && row[15] == "reviewed"
                && candidate.passage.as_ref().is_some_and(|passage| {
                    evaluation_passages.contains(&(candidate.source_id.clone(), passage.clone()))
                })
            {
                return invalid(
                    &review_path,
                    offset + 2,
                    "reviewed lexical attestation shares a passage with held-out evaluation",
                );
            }
        }
        if row[10] == row[12] || row[11] == row[13] {
            return invalid(
                &review_path,
                offset + 2,
                "semantic identity and target attestation must be independently sourced",
            );
        }
    }
    Ok(())
}

fn runtime_evidence_ids(data_directory: &Path) -> Result<BTreeSet<String>> {
    let specifications: [(&str, &[usize]); 13] = [
        ("principal_parts.tsv", &[4]),
        ("exact_forms.tsv", &[4]),
        ("alignments.tsv", &[8]),
        ("abbreviations.tsv", &[6]),
        ("abbreviation_families.tsv", &[5]),
        ("accents.tsv", &[4]),
        ("accent_paradigms.tsv", &[6]),
        ("noun_restrictions.tsv", &[2]),
        ("positional_rules.tsv", &[5]),
        ("transformation_rules.tsv", &[5]),
        ("irregular_overrides.tsv", &[3]),
        ("verb_defectiveness.tsv", &[6]),
        ("irregular_verb_inventory.tsv", &[5]),
    ];
    let mut ids = BTreeSet::new();
    for (file_name, columns) in specifications {
        let path = data_directory.join(file_name);
        let text = fs::read_to_string(&path)?;
        let mut lines = text.lines();
        let header_columns = lines.next().unwrap_or_default().split('\t').count();
        for (offset, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != header_columns {
                return invalid(&path, offset + 2, "runtime evidence row has invalid width");
            }
            for &column in columns {
                for evidence_id in fields[column].split(',').filter(|value| !value.is_empty()) {
                    ids.insert(evidence_id.to_owned());
                }
            }
        }
    }
    let semantic_alignments = read_table(
        &data_directory.join("semantic_alignments.tsv"),
        "mapping_id\tsource_sense_id\ttarget_sense_id\tstatus\tevidence_id\treview_note",
        6,
    )?;
    for row in semantic_alignments
        .rows
        .iter()
        .filter(|row| row[3] != "false-friend")
    {
        ids.insert(row[4].clone());
    }
    Ok(ids)
}

fn read_table(path: &Path, expected_header: &'static str, columns: usize) -> Result<Table> {
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

fn validate_lexemes(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        validate_target(path, line, &row[8])?;
        validate_word(path, line, &row[1], "lemma")?;
        if !row[4].is_empty() {
            validate_word(path, line, &row[4], "stem")?;
        }
        if !row[0].starts_with("synodal:") {
            return invalid(
                path,
                line,
                "target lexeme IDs must use the synodal namespace",
            );
        }
        if !matches!(
            row[2].as_str(),
            "adverb"
                | "preposition"
                | "conjunction"
                | "particle"
                | "interjection"
                | "proper-noun"
                | "noun"
                | "adjective"
                | "verb"
                | "pronoun"
                | "determiner"
                | "numeral"
                | "participle"
        ) {
            return invalid(path, line, "unknown lexeme part of speech");
        }
        let valid_class = matches!(
            (row[2].as_str(), row[3].as_str()),
            (_, "" | "exact")
                | (
                    "noun",
                    "first-hard-m"
                        | "inherited-first-hard-m"
                        | "first-hard-u-stem-m"
                        | "first-hard-in-ethnonym-m"
                        | "first-hard-ud-es-m"
                        | "first-hard-velar-m"
                        | "first-mixed-m"
                        | "first-mixed-ts-m"
                        | "first-hard-n"
                        | "first-soft-m"
                        | "first-soft-agent-tel-m"
                        | "first-soft-lord-m"
                        | "first-soft-j-m"
                        | "first-soft-ey-m"
                        | "first-soft-n"
                        | "first-soft-ishche-n"
                        | "first-soft-ie-n"
                        | "second-hard"
                        | "second-hard-velar"
                        | "second-soft"
                        | "second-soft-postvocalic-ancient-pl"
                        | "second-soft-m-ia"
                        | "second-soft-f-ia"
                        | "second-mixed"
                        | "third-f"
                        | "third-m"
                        | "fourth-neuter-en"
                        | "fourth-neuter-es"
                        | "fourth-neuter-es-alt-first"
                        | "fourth-neuter-es-paired-dual"
                        | "fourth-neuter-at"
                        | "fourth-feminine-er"
                        | "fourth-feminine-er-daughter"
                        | "fourth-feminine-ov"
                        | "fourth-feminine-ov-syncopating"
                        | "fourth-masculine-en"
                        | "fourth-masculine-en-day"
                        | "fourth-masculine-en-kamen"
                        | "indeclinable",
                )
                | ("adjective", "hard-short" | "soft-short" | "velar-short")
                | (
                    "determiner",
                    "determiner-pronominal-hard"
                        | "determiner-ves-mixed"
                        | "determiner-vsyak-mixed"
                        | "determiner-full-sk"
                )
                | (
                    "numeral",
                    "numeral-cardinal-one"
                        | "numeral-cardinal-two"
                        | "numeral-cardinal-both"
                        | "numeral-cardinal-three"
                        | "numeral-cardinal-four"
                        | "numeral-cardinal-i-stem"
                        | "numeral-cardinal-ten"
                        | "numeral-cardinal-hundred"
                        | "numeral-cardinal-second-hard"
                        | "numeral-cardinal-second-mixed"
                        | "numeral-cardinal-first-hard-m"
                        | "numeral-cardinal-third-f"
                        | "ordinal-hard"
                        | "ordinal-soft"
                        | "numeral-collective-agreeing"
                        | "numeral-collective-governing-neuter"
                        | "numeral-collective-hard-plural"
                        | "numeral-multiplicative-hard"
                        | "numeral-multiplicative-soft"
                        | "numeral-fractional-hard"
                        | "numeral-fractional-first-u"
                        | "numeral-fractional-second-hard"
                        | "numeral-fractional-third-f"
                )
                | (
                    "pronoun",
                    "exact-complete-pronoun-table"
                        | "pronoun-personal-first"
                        | "pronoun-personal-second"
                        | "pronoun-reflexive"
                        | "pronoun-reflexive-clitic"
                        | "pronoun-third-person"
                        | "pronoun-third-person-demonstrative"
                        | "pronoun-relative-izhe"
                        | "pronoun-proximal-sei"
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
                        | "pronoun-interrogative-who"
                        | "pronoun-interrogative-what"
                        | "pronoun-indefinite-kii"
                        | "pronoun-indefinite-who"
                        | "pronoun-indefinite-what"
                        | "pronoun-negative-kii"
                        | "pronoun-negative-full-hard"
                        | "pronoun-negative-who"
                        | "pronoun-negative-what"
                        | "pronoun-kii-zhdo"
                        | "pronoun-negative-who-zhe"
                        | "pronoun-negative-what-zhe",
                )
                | (
                    "verb",
                    "first-unpalatalized" | "first-palatalized" | "second" | "archaic"
                )
        );
        if !valid_class {
            return invalid(path, line, "unknown class for lexeme part of speech");
        }
        if !matches!(row[5].as_str(), "" | "masculine" | "feminine" | "neuter") {
            return invalid(path, line, "unknown lexical gender");
        }
        if !matches!(
            row[6].as_str(),
            "" | "unknown" | "imperfective" | "perfective" | "biaspectual"
        ) {
            return invalid(path, line, "unknown lexical aspect");
        }
        if row[7].is_empty() {
            return invalid(path, line, "a lexeme requires a source ID");
        }
    }
    Ok(())
}

fn validate_noun_restrictions(path: &Path, table: &Table) -> Result<()> {
    let mut lexeme_ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !lexeme_ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if !matches!(
            row[1].as_str(),
            "singular-only"
                | "dual-only"
                | "plural-only"
                | "singular-and-dual"
                | "singular-and-plural"
                | "dual-and-plural"
        ) {
            return invalid(path, offset + 2, "unknown noun number inventory");
        }
        if row[2].is_empty() {
            return invalid(
                path,
                offset + 2,
                "a noun restriction requires normative evidence",
            );
        }
        validate_target(path, offset + 2, &row[3])?;
    }
    Ok(())
}

fn validate_noun_restriction_lexemes(
    path: &Path,
    restrictions: &Table,
    lexemes: &Table,
) -> Result<()> {
    let lexemes_by_id = lexemes
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for (offset, restriction) in restrictions.rows.iter().enumerate() {
        let Some(lexeme) = lexemes_by_id.get(restriction[0].as_str()) else {
            return invalid(
                path,
                offset + 2,
                "noun restriction references an unknown lexeme",
            );
        };
        if lexeme[2] != "noun" {
            return invalid(
                path,
                offset + 2,
                "noun restriction references a non-noun lexeme",
            );
        }
        if restriction[3] != lexeme[8] {
            return invalid(
                path,
                offset + 2,
                "noun restriction and lexeme target recensions disagree",
            );
        }
    }
    Ok(())
}

fn validate_principal_parts(path: &Path, table: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[5])?;
        validate_word(path, offset + 2, &row[2], "principal part")?;
        if row[1].is_empty() || row[4].is_empty() {
            return invalid(
                path,
                offset + 2,
                "principal parts require a system and normative evidence",
            );
        }
        if row[1] == "comparative-stem"
            && !matches!(
                row[3].as_str(),
                "ancient-hard" | "ancient-soft" | "later-yat" | "later-ai"
            )
        {
            return invalid(path, offset + 2, "unknown typed comparison formation");
        }
        if row[1] == "short-masculine-stem"
            && !matches!(row[3].as_str(), "double-n-reduction" | "mobile-e-insertion")
        {
            return invalid(path, offset + 2, "unknown typed short-masculine formation");
        }
        if row[1].ends_with("active-participle-short-stem") {
            let valid = matches!(
                row[3].as_str(),
                "hard:present-first-unpalatalized"
                    | "soft:present-first-unpalatalized"
                    | "hard:present-first-palatalized"
                    | "soft:present-first-palatalized"
                    | "hard:present-second"
                    | "soft:present-second"
                    | "hard:present-after-sibilant"
                    | "soft:present-after-sibilant"
                    | "hard:past-consonant"
                    | "soft:past-consonant"
                    | "hard:past-vowel"
                    | "soft:past-vowel"
                    | "hard:past-iotated"
                    | "soft:past-iotated"
            );
            if !valid {
                return invalid(
                    path,
                    offset + 2,
                    "active short participles require a class and closed typed formation",
                );
            }
        }
    }
    Ok(())
}

fn validate_exact_forms(path: &Path, table: &Table, lexemes: &Table) -> Result<()> {
    let mut runtime_keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !runtime_keys.insert((
            row[0].clone(),
            row[1].clone(),
            row[2].clone(),
            row[3].clone(),
        )) {
            return invalid(
                path,
                offset + 2,
                "duplicate lexeme/cell/expanded/printed exact-form tuple",
            );
        }
        validate_grammar_cell(path, offset + 2, &row[1])?;
        validate_cell_lexeme_pos(path, offset + 2, &row[0], &row[1], lexemes)?;
        validate_target(path, offset + 2, &row[6])?;
        validate_word(path, offset + 2, &row[2], "expanded form")?;
        validate_word(path, offset + 2, &row[3], "printed form")?;
        if !matches!(
            row[5].as_str(),
            "normative-table" | "normative-variant" | "synodal-attestation"
        ) {
            return invalid(path, offset + 2, "unknown exact-form source kind");
        }
        if row[5] == "synodal-attestation" && row[4].is_empty() {
            return invalid(
                path,
                offset + 2,
                "attestation rows require passage evidence",
            );
        }
    }
    Ok(())
}

fn validate_alignments(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if !row[2].starts_with("synodal:") {
            return invalid(
                path,
                offset + 2,
                "mapping target must be a Synodal lexeme ID",
            );
        }
        if !row[1].starts_with("ocs:") {
            return invalid(
                path,
                offset + 2,
                "mapping source must be a stable OCS lexeme ID",
            );
        }
        if !matches!(
            row[4].as_str(),
            "reviewed" | "automatically-validated" | "exploratory" | "rejected"
        ) {
            return invalid(path, offset + 2, "unknown mapping review status");
        }
        let confidence = row[7]
            .parse::<u16>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line: offset + 2,
                reason: "confidence must be integer basis points".into(),
            })?;
        if confidence > 10_000 {
            return invalid(
                path,
                offset + 2,
                "confidence cannot exceed 10000 basis points",
            );
        }
        if row[4] == "rejected" && confidence != 0 {
            return invalid(
                path,
                offset + 2,
                "rejected mappings must have zero confidence",
            );
        }
        if row[4] != "rejected" && (confidence == 0 || row[8].is_empty() || row[9].is_empty()) {
            return invalid(
                path,
                offset + 2,
                "admitted mappings require confidence, evidence, and explicit transformations",
            );
        }
    }
    Ok(())
}

fn validate_abbreviations(path: &Path, table: &Table, lexemes: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_grammar_cell(path, offset + 2, &row[2])?;
        validate_cell_lexeme_pos(path, offset + 2, &row[0], &row[2], lexemes)?;
        validate_target(path, offset + 2, &row[12])?;
        validate_word(path, offset + 2, &row[3], "expanded abbreviation")?;
        validate_word(path, offset + 2, &row[4], "printed abbreviation")?;
        if row[1].is_empty() {
            return invalid(
                path,
                offset + 2,
                "abbreviation rows require a semantic sense ID",
            );
        }
        if row[2].is_empty()
            || row[5].is_empty()
            || row[6].is_empty()
            || row[8].is_empty()
            || row[9].is_empty()
            || row[10].is_empty()
            || row[11] != TARGET
        {
            return invalid(
                path,
                offset + 2,
                "abbreviations require a cell, rule, evidence, marks, context, ambiguity, and Synodal source recension",
            );
        }
        if !matches!(row[7].as_str(), "true" | "false") {
            return invalid(path, offset + 2, "reversible must be true or false");
        }
    }
    Ok(())
}

fn validate_abbreviation_families(
    path: &Path,
    table: &Table,
    abbreviations: &Table,
    lexemes: &Table,
) -> Result<()> {
    let mut patterns = BTreeSet::new();
    let mut metadata = BTreeMap::<(String, String), Vec<String>>::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !lexemes.rows.iter().any(|lexeme| lexeme[0] == row[0]) {
            return invalid(
                path,
                line,
                &format!("abbreviation family references unknown lexeme {}", row[0]),
            );
        }
        validate_word(path, line, &row[2], "expanded abbreviation-family prefix")?;
        validate_word(path, line, &row[3], "printed abbreviation-family prefix")?;
        validate_target(path, line, &row[11])?;
        if row[1].is_empty()
            || row[2].is_empty()
            || row[3].is_empty()
            || row[4].is_empty()
            || row[5].is_empty()
            || row[7].is_empty()
            || row[8].is_empty()
            || row[9].is_empty()
            || row[10] != TARGET
        {
            return invalid(
                path,
                line,
                "abbreviation families require identity, nonempty prefixes, rule, evidence, marks, context, ambiguity, and Synodal recensions",
            );
        }
        if !matches!(row[6].as_str(), "true" | "false") {
            return invalid(path, line, "reversible must be true or false");
        }
        validate_abbreviation_family_marks(path, line, &row[3], &row[7])?;
        if !patterns.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        let key = (row[0].clone(), row[1].clone());
        let family_metadata = row[4..].to_vec();
        if let Some(first) = metadata.get(&key) {
            if first != &family_metadata {
                return invalid(
                    path,
                    line,
                    "all patterns in one abbreviation family must share rule and review metadata",
                );
            }
        } else {
            metadata.insert(key, family_metadata);
        }
    }

    for ((lexeme_id, sense_id), _) in metadata {
        let family_patterns = table
            .rows
            .iter()
            .filter(|row| row[0] == lexeme_id && row[1] == sense_id)
            .collect::<Vec<_>>();
        let exact = abbreviations
            .rows
            .iter()
            .filter(|row| row[0] == lexeme_id && row[1] == sense_id)
            .collect::<Vec<_>>();
        if exact.is_empty() {
            return invalid(
                path,
                1,
                &format!(
                    "abbreviation family {lexeme_id}:{sense_id} requires at least one reviewed exact cell"
                ),
            );
        }
        for exact_row in exact {
            let expected = normalize_abbreviation_family_shape(&exact_row[4]);
            let covered = family_patterns.iter().any(|pattern| {
                exact_row[3]
                    .strip_prefix(&pattern[2])
                    .map(|suffix| format!("{}{suffix}", pattern[3]))
                    .is_some_and(|generated| {
                        normalize_abbreviation_family_shape(&generated) == expected
                    })
            });
            if !covered {
                return invalid(
                    path,
                    1,
                    &format!(
                        "abbreviation family {lexeme_id}:{sense_id} does not reproduce reviewed exact shape {:?} -> {:?}",
                        exact_row[3], exact_row[4]
                    ),
                );
            }
        }
        for pattern in family_patterns {
            if !abbreviations.rows.iter().any(|exact_row| {
                exact_row[0] == lexeme_id
                    && exact_row[1] == sense_id
                    && abbreviation_pattern_covers(&exact_row[3], &exact_row[4], pattern)
            }) {
                return invalid(
                    path,
                    1,
                    &format!(
                        "abbreviation-family pattern {:?} -> {:?} has no reviewed exact witness",
                        pattern[2], pattern[3]
                    ),
                );
            }
        }
    }
    Ok(())
}

fn validate_abbreviation_family_marks(
    path: &Path,
    line: usize,
    printed_prefix: &str,
    required_marks: &str,
) -> Result<()> {
    let characters = printed_prefix.nfd().collect::<Vec<_>>();
    for mark in required_marks.split(',') {
        let present = match mark {
            "titlo" => characters.iter().any(|character| {
                *character == '\u{0483}' || ('\u{2de0}'..='\u{2dff}').contains(character)
            }),
            "initial-breathing" => characters.contains(&'\u{0486}'),
            "pokrytie" => characters.contains(&'\u{0487}'),
            "superscript-s" => characters.contains(&'\u{2ded}'),
            "superscript-g" => characters.contains(&'\u{2de2}'),
            "superscript-o" => characters.contains(&'\u{2dea}'),
            "superscript-d" => characters.contains(&'\u{2de3}'),
            _ => return invalid(path, line, "unknown required abbreviation-family mark"),
        };
        if !present {
            return invalid(
                path,
                line,
                &format!("printed family prefix is missing required mark {mark:?}"),
            );
        }
    }
    Ok(())
}

fn abbreviation_pattern_covers(expanded: &str, printed: &str, pattern: &[String]) -> bool {
    expanded
        .strip_prefix(&pattern[2])
        .map(|suffix| format!("{}{suffix}", pattern[3]))
        .is_some_and(|generated| {
            normalize_abbreviation_family_shape(&generated)
                == normalize_abbreviation_family_shape(printed)
        })
}

fn normalize_abbreviation_family_shape(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0308}' | '\u{0311}'))
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'ѡ' | 'ѻ' | 'ꙍ' => 'о',
            'і' | 'ї' => 'и',
            'є' => 'е',
            'ꙋ' => 'у',
            'ꙗ' | 'я' => 'ѧ',
            other => other,
        })
        .nfc()
        .collect()
}

fn validate_abbreviation_inventory(path: &Path, table: &Table, families: &Table) -> Result<()> {
    if table.rows.len() != 48 {
        return invalid(
            path,
            1,
            "Alypy §3.c abbreviation inventory must classify all 48 named entries",
        );
    }
    let mut orders = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        let order = row[0]
            .parse::<u8>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line,
                reason: "source_order must be an integer from 1 through 48".into(),
            })?;
        if !(1..=48).contains(&order) || !orders.insert(order) {
            return invalid(path, line, "source_order must uniquely cover 1 through 48");
        }
        validate_word(path, line, &row[1], "source abbreviation head")?;
        validate_word(path, line, &row[2], "source abbreviation expansion")?;
        validate_target(path, line, &row[10])?;
        if row[3].is_empty() || row[8].is_empty() || row[9].is_empty() {
            return invalid(
                path,
                line,
                "every abbreviation inventory row requires semantic scope, evidence, and a review note",
            );
        }
        match row[4].as_str() {
            "productive-family" => {
                if row[5].is_empty() || row[6].is_empty() || row[7].is_empty() {
                    return invalid(
                        path,
                        line,
                        "productive abbreviation decisions require lexeme, sense, and rule IDs",
                    );
                }
                if !families.rows.iter().any(|family| {
                    family[0] == row[5]
                        && family[1] == row[6]
                        && family[4] == row[7]
                        && abbreviation_inventory_pattern_covers(&row[2], &row[1], family)
                }) {
                    return invalid(
                        path,
                        line,
                        "productive abbreviation decision does not structurally match a generated family",
                    );
                }
            }
            "implementation-missing" => {
                if row[5..8].iter().any(|value| !value.is_empty()) {
                    return invalid(
                        path,
                        line,
                        "missing abbreviation decisions cannot claim runtime IDs",
                    );
                }
            }
            _ => {
                return invalid(
                    path,
                    line,
                    "abbreviation decision must be productive-family or implementation-missing",
                );
            }
        }
    }
    if orders != (1_u8..=48).collect() {
        return invalid(
            path,
            1,
            "source_order does not exhaustively cover 1 through 48",
        );
    }
    Ok(())
}

fn abbreviation_inventory_pattern_covers(
    expanded_head: &str,
    printed_head: &str,
    family: &[String],
) -> bool {
    let expanded = normalize_abbreviation_family_shape(expanded_head);
    let expanded_prefix = normalize_abbreviation_family_shape(&family[2]);
    expanded
        .strip_prefix(&expanded_prefix)
        .map(|suffix| {
            format!(
                "{}{suffix}",
                normalize_abbreviation_family_shape(&family[3])
            )
        })
        .is_some_and(|generated| generated == normalize_abbreviation_family_shape(printed_head))
}

fn validate_grammar_cell(path: &Path, line: usize, value: &str) -> Result<()> {
    value
        .parse::<GrammarCell>()
        .map(|_| ())
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: error.to_string(),
        })
}

fn validate_cell_lexeme_pos(
    path: &Path,
    line: usize,
    lexeme_id: &str,
    value: &str,
    lexemes: &Table,
) -> Result<()> {
    let part_of_speech = lexemes
        .rows
        .iter()
        .find(|row| row[0] == lexeme_id)
        .map(|row| row[2].as_str())
        .ok_or_else(|| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: format!("grammar cell references unknown lexeme {lexeme_id}"),
        })?;
    let cell = value
        .parse::<GrammarCell>()
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: error.to_string(),
        })?;
    let compatible = matches!(
        (cell, part_of_speech),
        (GrammarCell::LexicalForm, _)
            | (
                GrammarCell::Indeclinable,
                "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
            )
            | (GrammarCell::Noun(_), "noun" | "proper-noun")
            | (GrammarCell::Adjective(_), "adjective")
            | (GrammarCell::Determiner(_), "determiner")
            | (GrammarCell::Pronoun(_), "pronoun")
            | (GrammarCell::Numeral(_), "numeral")
            | (
                GrammarCell::FiniteVerb(_)
                    | GrammarCell::Imperative(_)
                    | GrammarCell::Infinitive
                    | GrammarCell::LParticiple(_)
                    | GrammarCell::Participle(_)
                    | GrammarCell::Supine
                    | GrammarCell::VerbalNoun(_),
                "verb"
            )
    );
    if compatible {
        Ok(())
    } else {
        invalid(
            path,
            line,
            &format!(
                "grammar cell {value} is incompatible with {part_of_speech} lexeme {lexeme_id}"
            ),
        )
    }
}

fn validate_accents(path: &Path, table: &Table) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[7])?;
        if row[5].is_empty() || row[6] != TARGET {
            return invalid(
                path,
                offset + 2,
                "accent evidence requires a source ID and Synodal source recension",
            );
        }
        validate_word(path, offset + 2, &row[2], "expanded accent form")?;
        validate_word(path, offset + 2, &row[3], "accented form")?;
        if !keys.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        if row[2] == row[3] {
            return invalid(
                path,
                offset + 2,
                "accent metadata must add a presentation mark",
            );
        }
    }
    Ok(())
}

fn validate_accent_paradigms(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[10])?;
        if row[1].is_empty()
            || row[6].is_empty()
            || row[7].is_empty()
            || row[8].is_empty()
            || row[9] != TARGET
        {
            return invalid(
                path,
                offset + 2,
                "accent paradigm requires stable IDs, evidence, a source, and Synodal source recension",
            );
        }
        if !ids.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if !matches!(row[4].as_str(), "acute" | "grave" | "kamora") {
            return invalid(path, offset + 2, "unknown accent-paradigm mark");
        }
        if !row[3].starts_with("stem-vowel-from-start:")
            && !row[3].starts_with("ending-vowel-from-end:")
        {
            return invalid(path, offset + 2, "unknown accent-paradigm placement");
        }
        validate_accent_placement_code(path, offset + 2, &row[3])?;
        validate_accent_scope_code(path, offset + 2, &row[2])?;
        if !row[5].is_empty() && !row[5].starts_with("psili@") {
            return invalid(path, offset + 2, "unknown accent-paradigm breathing rule");
        }
        if let Some(placement) = row[5].strip_prefix("psili@") {
            validate_accent_placement_code(path, offset + 2, placement)?;
        }
    }
    Ok(())
}

fn validate_accent_scope_code(path: &Path, line: usize, value: &str) -> Result<()> {
    let parts = value.split(':').collect::<Vec<_>>();
    let (numbers, cases) = match parts.as_slice() {
        ["all"] => return Ok(()),
        ["noun", numbers] => (*numbers, None),
        ["noun", numbers, cases] => (*numbers, Some(*cases)),
        ["adjective", form, comparison, numbers]
            if matches!(*form, "short" | "long")
                && matches!(*comparison, "positive" | "comparative" | "superlative") =>
        {
            (*numbers, None)
        }
        ["finite", tense, numbers]
            if matches!(
                *tense,
                "present" | "future" | "past" | "imperfect" | "aorist"
            ) =>
        {
            (*numbers, None)
        }
        _ => return invalid(path, line, "unknown accent-paradigm scope"),
    };
    if !numbers
        .split(',')
        .all(|number| matches!(number, "singular" | "dual" | "plural"))
    {
        return invalid(path, line, "unknown number in accent-paradigm scope");
    }
    if cases.is_some_and(|cases| {
        cases.split(',').any(|case| {
            !matches!(
                case,
                "nominative"
                    | "genitive"
                    | "dative"
                    | "accusative"
                    | "instrumental"
                    | "locative"
                    | "vocative"
            )
        })
    }) {
        return invalid(path, line, "unknown case in accent-paradigm scope");
    }
    Ok(())
}

fn validate_accent_placement_code(path: &Path, line: usize, value: &str) -> Result<()> {
    let Some((kind, offset)) = value.rsplit_once(':') else {
        return invalid(path, line, "invalid accent-paradigm placement");
    };
    if !matches!(kind, "stem-vowel-from-start" | "ending-vowel-from-end")
        || offset.parse::<u8>().is_err()
    {
        return invalid(path, line, "invalid accent-paradigm placement");
    }
    Ok(())
}

fn validate_positional_rules(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[6])?;
        validate_word(path, offset + 2, &row[1], "positional input")?;
        validate_word(path, offset + 2, &row[3], "positional output")?;
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

fn validate_transformation_rules(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if row[1] != "old-church-slavonic" || row[2] != TARGET {
            return invalid(
                path,
                offset + 2,
                "recension transformations must explicitly map OCS to Synodal Russian",
            );
        }
        if !matches!(row[4].as_str(), "reviewed" | "automatically-validated") {
            return invalid(path, offset + 2, "unknown transformation review status");
        }
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

fn validate_conflicts(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !row[1].starts_with("ocs:") || !row[2].starts_with("synodal:") {
            return invalid(
                path,
                offset + 2,
                "conflicts require stable OCS source and Synodal target IDs",
            );
        }
        if row[5].is_empty() || row[6].is_empty() || row[7].is_empty() {
            return invalid(
                path,
                offset + 2,
                "conflicts must retain support, contradiction, and resolution",
            );
        }
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

fn validate_conflict_evidence(path: &Path, conflicts: &Table, evidence: &Table) -> Result<()> {
    let known = evidence
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect::<BTreeSet<_>>();
    for (offset, row) in conflicts.rows.iter().enumerate() {
        for column in [5_usize, 6_usize] {
            for evidence_id in row[column].split(',').filter(|value| !value.is_empty()) {
                if !known.contains(evidence_id) {
                    return invalid(
                        path,
                        offset + 2,
                        &format!("conflict has unregistered evidence {evidence_id:?}"),
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_irregular_overrides(path: &Path, table: &Table) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[4])?;
        if !keys.insert((row[0].clone(), row[1].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if row[2] != "data/synodal/exact_forms.tsv" || row[3].is_empty() {
            return invalid(
                path,
                offset + 2,
                "irregular overrides must point to the exact-form registry and evidence",
            );
        }
    }
    Ok(())
}

fn validate_defective_inventories(path: &Path, table: &Table, lexemes: &Table) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        validate_target(path, line, &row[7])?;
        if !keys.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        let Some(lexeme) = lexemes.rows.iter().find(|lexeme| lexeme[0] == row[0]) else {
            return invalid(
                path,
                line,
                "defective inventory references an unknown lexeme",
            );
        };
        if lexeme[2] != "verb" {
            return invalid(
                path,
                line,
                "defective inventory references a non-verb lexeme",
            );
        }
        match row[1].as_str() {
            "outside-inventory" => {
                let mut cells = BTreeSet::new();
                for cell in row[2]
                    .split(',')
                    .map(str::trim)
                    .filter(|cell| !cell.is_empty())
                {
                    validate_grammar_cell(path, line, cell)?;
                    validate_cell_lexeme_pos(path, line, &row[0], cell, lexemes)?;
                    if !cells.insert(cell) {
                        return invalid(path, line, "defective inventory repeats an allowed cell");
                    }
                }
                if cells.is_empty() {
                    return invalid(
                        path,
                        line,
                        "defective inventory requires at least one allowed cell",
                    );
                }
            }
            "cell-prefix" => {
                if !matches!(
                    row[2].as_str(),
                    "present:"
                        | "future:"
                        | "past:"
                        | "imperfect:"
                        | "aorist:"
                        | "imperative:"
                        | "l-participle:"
                        | "participle:present:active:"
                        | "participle:present:passive:"
                        | "participle:past:active:"
                        | "participle:past:passive:"
                        | "verbal-noun:"
                ) {
                    return invalid(path, line, "unknown defective cell-system prefix");
                }
            }
            _ => return invalid(path, line, "unknown defective inventory mode"),
        }
        if !matches!(
            row[3].as_str(),
            "historically-absent" | "evidence-incomplete"
        ) {
            return invalid(path, line, "unknown defect kind");
        }
        if !matches!(
            row[4].as_str(),
            "present-stem"
                | "present-first-singular"
                | "present-third-plural"
                | "imperfect-stem"
                | "aorist-stem"
                | "aorist-formation"
                | "imperative-stem"
                | "imperative-formation"
                | "imperfect-formation"
                | "infinitive"
                | "supine-stem"
                | "l-participle-stem"
                | "participle-stem"
                | "participle-formation"
                | "verbal-noun-stem"
                | "aspect"
                | "formation"
                | "regular-background"
                | "irregular-override"
        ) {
            return invalid(path, line, "unknown defect metadata field");
        }
        if row[5].trim().is_empty() || row[6].trim().is_empty() {
            return invalid(
                path,
                line,
                "defective inventory requires a reason and evidence",
            );
        }
    }
    Ok(())
}

fn validate_irregular_verb_inventory(path: &Path, table: &Table) -> Result<()> {
    let expected_orders = (2_u8..=100)
        .filter(|order| *order != 97)
        .collect::<BTreeSet<_>>();
    let mut orders = BTreeSet::new();
    let mut headword_orders = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        validate_target(path, line, &row[7])?;
        let source_order = row[0]
            .parse::<u8>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line,
                reason: "irregular inventory source_order must be an integer".into(),
            })?;
        if !orders.insert(source_order) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if row[1].trim().is_empty() || !headword_orders.insert((row[1].clone(), source_order)) {
            return invalid(path, line, "irregular inventory requires a source headword");
        }
        let systems = row[2]
            .split(',')
            .map(str::trim)
            .filter(|system| !system.is_empty())
            .collect::<BTreeSet<_>>();
        if systems.is_empty()
            || systems.iter().any(|system| {
                !matches!(
                    *system,
                    "present"
                        | "future"
                        | "aorist"
                        | "imperfect"
                        | "imperative"
                        | "l-participle"
                        | "present-active-participle"
                        | "present-passive-participle"
                        | "past-active-participle"
                        | "past-passive-participle"
                        | "stem-alternation"
                        | "defectiveness"
                )
            })
        {
            return invalid(path, line, "irregular inventory has an unknown system code");
        }
        if !matches!(
            row[3].as_str(),
            "bundled-exact-and-productive"
                | "bundled-exact-and-defective"
                | "caller-exact-principal-parts"
                | "typed-defective-inventory"
        ) {
            return invalid(path, line, "irregular inventory has an unknown strategy");
        }
        if !matches!(
            row[4].as_str(),
            "implemented-bundled"
                | "implemented-by-metadata-contract"
                | "source-evidence-incomplete"
        ) {
            return invalid(
                path,
                line,
                "irregular inventory has an unknown implementation status",
            );
        }
        if row[5].trim().is_empty() || row[6].trim().is_empty() {
            return invalid(
                path,
                line,
                "irregular inventory requires evidence and a note",
            );
        }
    }
    if orders != expected_orders {
        let missing = expected_orders
            .difference(&orders)
            .copied()
            .collect::<Vec<_>>();
        let extra = orders
            .difference(&expected_orders)
            .copied()
            .collect::<Vec<_>>();
        return invalid(
            path,
            1,
            &format!(
                "irregular inventory must cover all 98 Alypy §104 verb entries; missing {missing:?}, extra {extra:?}"
            ),
        );
    }
    Ok(())
}

fn validate_morphology_references<const N: usize>(
    lexeme_path: &Path,
    lexemes: &Table,
    tables: [(&Path, &Table, usize); N],
) -> Result<()> {
    let ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    for (path, table, id_column) in tables {
        for (offset, row) in table.rows.iter().enumerate() {
            if !ids.contains(row[id_column].as_str()) {
                return invalid(
                    path,
                    offset + 2,
                    &format!(
                        "unknown lexeme ID {:?}; target registry is {}",
                        row[id_column],
                        lexeme_path.display()
                    ),
                );
            }
        }
    }
    Ok(())
}

fn validate_semantic_alignments(path: &Path, table: &Table) -> Result<()> {
    let mut mappings = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if row[1].is_empty() || row[2].is_empty() || row[4].is_empty() || row[5].is_empty() {
            return invalid(
                path,
                offset + 2,
                "semantic alignments require both sense IDs, evidence, and a review note",
            );
        }
        if !matches!(
            row[3].as_str(),
            "established" | "false-friend" | "uncertain"
        ) {
            return invalid(path, offset + 2, "unknown semantic alignment status");
        }
        if !mappings.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

fn validate_semantic_alignment_evidence(
    path: &Path,
    alignments: &Table,
    evidence: &Table,
) -> Result<()> {
    let decisions = evidence
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row[4].as_str()))
        .collect::<BTreeMap<_, _>>();
    for (offset, row) in alignments.rows.iter().enumerate() {
        let required_decision = if row[3] == "false-friend" {
            "rejected"
        } else {
            "reviewed"
        };
        if decisions.get(row[4].as_str()) != Some(&required_decision) {
            return invalid(
                path,
                offset + 2,
                &format!(
                    "semantic alignment status {:?} requires {required_decision} evidence",
                    row[3]
                ),
            );
        }
    }
    Ok(())
}

fn validate_alignment_references(
    alignment_path: &Path,
    alignments: &Table,
    lexemes: &Table,
    transformation_rules: &Table,
    conflict_path: &Path,
    conflicts: &Table,
) -> Result<()> {
    let target_ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    let transformations: BTreeSet<&str> = transformation_rules
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect();
    for (offset, row) in alignments.rows.iter().enumerate() {
        if !row[1].starts_with("ocs:") || !target_ids.contains(row[2].as_str()) {
            return invalid(
                alignment_path,
                offset + 2,
                "alignment requires an OCS source ID and an existing Synodal target ID",
            );
        }
        for transformation in row[9].split(',').filter(|value| !value.is_empty()) {
            if !transformations.contains(transformation) {
                return invalid(
                    alignment_path,
                    offset + 2,
                    &format!("unknown recension transformation {transformation:?}"),
                );
            }
        }
    }
    for (offset, row) in conflicts.rows.iter().enumerate() {
        if !target_ids.contains(row[2].as_str()) {
            return invalid(
                conflict_path,
                offset + 2,
                "conflict target is absent from the Synodal lexeme registry",
            );
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_dictionary_references(
    sense_path: &Path,
    senses: &Table,
    example_path: &Path,
    examples: &Table,
    semantic_path: &Path,
    semantic_alignments: &Table,
    lexemes: &Table,
    morphology_alignments: &Table,
) -> Result<()> {
    let lexeme_ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    let sense_ids: BTreeSet<&str> = senses.rows.iter().map(|row| row[1].as_str()).collect();
    let mapping_ids: BTreeSet<&str> = morphology_alignments
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect();
    for (offset, row) in senses.rows.iter().enumerate() {
        if !lexeme_ids.contains(row[0].as_str()) {
            return invalid(
                sense_path,
                offset + 2,
                "sense refers to an unknown Synodal lexeme",
            );
        }
    }
    for (offset, row) in examples.rows.iter().enumerate() {
        if !lexeme_ids.contains(row[1].as_str()) {
            return invalid(
                example_path,
                offset + 2,
                "example refers to an unknown Synodal lexeme",
            );
        }
    }
    for (offset, row) in semantic_alignments.rows.iter().enumerate() {
        if !mapping_ids.contains(row[0].as_str()) || !sense_ids.contains(row[2].as_str()) {
            return invalid(
                semantic_path,
                offset + 2,
                "semantic decision requires an existing mapping and target sense",
            );
        }
    }
    Ok(())
}

fn validate_senses(
    path: &Path,
    table: &Table,
    source_recensions: &BTreeMap<String, String>,
) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert((row[0].clone(), row[1].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if !row[0].starts_with("synodal:") || row[1].is_empty() || row[2].is_empty() {
            return invalid(
                path,
                offset + 2,
                "sense requires a Synodal lexeme ID, stable sense ID, and gloss",
            );
        }
        let registered_recension =
            source_recensions
                .get(&row[4])
                .ok_or_else(|| ExtractionError::InvalidRow {
                    file: path.to_owned(),
                    line: offset + 2,
                    reason: format!("sense source {:?} is not registered", row[4]),
                })?;
        if registered_recension != &row[5] {
            return invalid(
                path,
                offset + 2,
                "sense source recension disagrees with the source inventory",
            );
        }
        let valid_status = match row[5].as_str() {
            "mixed" => row[6] == "reviewed-with-synodal-corpus",
            "old-church-slavonic" => matches!(
                row[6].as_str(),
                "reviewed-ocs-inheritance" | "reviewed-with-synodal-corpus"
            ),
            "synodal-russian" => row[6] == "normative",
            _ => false,
        };
        if !valid_status {
            return invalid(
                path,
                offset + 2,
                "sense semantic status is incompatible with its source recension",
            );
        }
    }
    Ok(())
}

fn validate_examples(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        RenderedText::parse(&row[2]).map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line: offset + 2,
            reason: format!("invalid example text: {error}"),
        })?;
        validate_target(path, offset + 2, &row[7])?;
        if row[6] != TARGET {
            return invalid(
                path,
                offset + 2,
                "target dictionary examples require Synodal source recension",
            );
        }
        if row[8] != "source" && row[8] != "evaluation" {
            return invalid(path, offset + 2, "unknown source/evaluation partition");
        }
    }
    Ok(())
}

fn validate_target(path: &Path, line: usize, value: &str) -> Result<()> {
    if value == TARGET {
        Ok(())
    } else {
        invalid(path, line, "target_recension must be synodal-russian")
    }
}

fn validate_word(path: &Path, line: usize, value: &str, label: &str) -> Result<()> {
    SynodalWord::parse(value)
        .map(|_| ())
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: format!("invalid {label}: {error}"),
        })
}

fn reject_forbidden_authority(path: &Path, line: usize, value: &str) -> Result<()> {
    let lower = value.to_lowercase();
    if lower.contains("slovowiki") {
        invalid(path, line, "Slovowiki is a forbidden linguistic authority")
    } else {
        Ok(())
    }
}

fn invalid<T>(path: &Path, line: usize, reason: &str) -> Result<T> {
    Err(ExtractionError::InvalidRow {
        file: path.to_owned(),
        line,
        reason: reason.into(),
    })
}

struct RegistryTables {
    lexemes: Table,
    noun_restrictions: Table,
    principal_parts: Table,
    exact_forms: Table,
    alignments: Table,
    abbreviations: Table,
    abbreviation_families: Table,
    accents: Table,
    accent_paradigms: Table,
    positional_rules: Table,
    transformation_rules: Table,
    conflicts: Table,
    irregular_overrides: Table,
    defective_inventories: Table,
    irregular_inventory: Table,
    evidence_provenance: Table,
}

fn emit_registry(tables: RegistryTables) -> String {
    let RegistryTables {
        mut lexemes,
        mut noun_restrictions,
        mut principal_parts,
        mut exact_forms,
        mut alignments,
        mut abbreviations,
        mut abbreviation_families,
        mut accents,
        mut accent_paradigms,
        mut positional_rules,
        mut transformation_rules,
        mut conflicts,
        mut irregular_overrides,
        mut defective_inventories,
        mut irregular_inventory,
        evidence_provenance,
    } = tables;
    lexemes.rows.sort();
    noun_restrictions.rows.sort();
    principal_parts.rows.sort();
    exact_forms.rows.sort_by(|left, right| {
        let source_rank = |source: &str| match source {
            "normative-table" => 0,
            "synodal-attestation" => 1,
            "normative-variant" => 2,
            _ => 3,
        };
        left[0]
            .cmp(&right[0])
            .then_with(|| left[1].cmp(&right[1]))
            .then_with(|| source_rank(&left[5]).cmp(&source_rank(&right[5])))
            .then_with(|| left[2].cmp(&right[2]))
            .then_with(|| left.cmp(right))
    });
    alignments.rows.sort();
    abbreviations.rows.sort();
    abbreviation_families.rows.sort();
    accents.rows.sort();
    accent_paradigms.rows.sort();
    positional_rules.rows.sort();
    transformation_rules.rows.sort();
    conflicts.rows.sort();
    irregular_overrides.rows.sort();
    defective_inventories.rows.sort();
    irregular_inventory
        .rows
        .sort_by_key(|row| row[0].parse::<u8>().ok());

    let mut output = String::from(
        "// @generated by synodal-church-slavonic-extractor; do not edit.\n\
         // Source: data/synodal/*.tsv\n\n",
    );
    emit_rows(&mut output, "LEXEMES", "RawLexeme", &lexemes.rows);
    emit_rows(
        &mut output,
        "NOUN_RESTRICTIONS",
        "RawNounRestriction",
        &noun_restrictions.rows,
    );
    emit_rows(
        &mut output,
        "PRINCIPAL_PARTS",
        "RawPrincipalPart",
        &principal_parts.rows,
    );
    emit_rows(
        &mut output,
        "EXACT_FORMS",
        "RawExactForm",
        &exact_forms.rows,
    );
    emit_rows(&mut output, "ALIGNMENTS", "RawAlignment", &alignments.rows);
    emit_rows(
        &mut output,
        "ABBREVIATIONS",
        "RawAbbreviation",
        &abbreviations.rows,
    );
    emit_rows(
        &mut output,
        "ABBREVIATION_FAMILIES",
        "RawAbbreviationFamily",
        &abbreviation_families.rows,
    );
    emit_rows(&mut output, "ACCENTS", "RawAccent", &accents.rows);
    emit_rows(
        &mut output,
        "ACCENT_PARADIGMS",
        "RawAccentParadigm",
        &accent_paradigms.rows,
    );
    emit_rows(
        &mut output,
        "POSITIONAL_RULES",
        "RawPositionalRule",
        &positional_rules.rows,
    );
    emit_rows(
        &mut output,
        "TRANSFORMATION_RULES",
        "RawTransformationRule",
        &transformation_rules.rows,
    );
    emit_rows(&mut output, "CONFLICTS", "RawConflict", &conflicts.rows);
    emit_rows(
        &mut output,
        "IRREGULAR_OVERRIDES",
        "RawIrregularOverride",
        &irregular_overrides.rows,
    );
    emit_rows(
        &mut output,
        "DEFECTIVE_INVENTORIES",
        "RawDefectiveInventory",
        &defective_inventories.rows,
    );
    emit_rows(
        &mut output,
        "IRREGULAR_VERB_INVENTORY",
        "RawIrregularVerbInventory",
        &irregular_inventory.rows,
    );
    emit_rows(
        &mut output,
        "REVIEWED_EVIDENCE",
        "RawReviewedEvidence",
        &evidence_provenance.rows,
    );
    let _ = output.pop();
    output
}

fn emit_dictionary_registry(
    mut senses: Table,
    mut examples: Table,
    mut semantic_alignments: Table,
) -> String {
    senses.rows.sort();
    examples.rows.sort();
    semantic_alignments.rows.sort();
    let mut output = String::from(
        "// @generated by synodal-church-slavonic-extractor; do not edit.\n\
         // Source: data/synodal/senses.tsv and examples.tsv\n\n",
    );
    emit_rows(&mut output, "SENSES", "RawSense", &senses.rows);
    emit_rows(&mut output, "EXAMPLES", "RawExample", &examples.rows);
    emit_rows(
        &mut output,
        "SEMANTIC_ALIGNMENTS",
        "RawSemanticAlignment",
        &semantic_alignments.rows,
    );
    let _ = output.pop();
    output
}

fn emit_rows(output: &mut String, constant: &str, row_type: &str, rows: &[Vec<String>]) {
    output.push_str("pub(crate) const ");
    output.push_str(constant);
    output.push_str(": &[");
    output.push_str(row_type);
    output.push_str("] = &[\n");
    for row in rows {
        output.push_str("    ");
        output.push_str(row_type);
        output.push_str("([");
        for (index, value) in row.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            push_rust_string(output, value);
        }
        output.push_str("]),\n");
    }
    output.push_str("];\n\n");
}

fn push_rust_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output.push('"');
}

fn atomic_write(destination: &Path, bytes: &[u8]) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| ExtractionError::InvalidRow {
            file: destination.to_owned(),
            line: 0,
            reason: "destination must have a parent directory".into(),
        })?;
    fs::create_dir_all(parent)?;
    let temporary = destination.with_extension("rs.tmp");
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, destination)?;
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_string_emission_is_lossless() {
        let mut output = String::new();
        push_rust_string(&mut output, "слово\\\"\t");
        assert_eq!(output, "\"слово\\\\\\\"\\t\"");
    }

    #[test]
    fn rejects_forbidden_authority_names() {
        let error = reject_forbidden_authority(Path::new("source.tsv"), 2, "Slovowiki")
            .expect_err("forbidden authority");
        assert!(error.to_string().contains("forbidden"));
    }

    #[test]
    fn source_approval_fails_closed_for_neutral_ids() {
        assert!(!source_recension_is_approved(
            "unreviewed-neutral-source",
            "mixed"
        ));
        assert!(source_recension_is_approved(
            "ponomar-elizabeth-bible-2026-08-09",
            TARGET
        ));
    }

    #[test]
    fn lexical_attestations_require_a_direct_target_corpus() {
        let source_recensions = BTreeMap::from([
            ("neutral-comparative-source".into(), "mixed".into()),
            ("ponomar-elizabeth-bible-2026-08-09".into(), TARGET.into()),
        ]);
        assert!(
            require_direct_target_source("neutral-comparative-source", &source_recensions).is_err()
        );
        assert_eq!(
            require_direct_target_source("ponomar-elizabeth-bible-2026-08-09", &source_recensions)
                .expect("approved target source"),
            TARGET
        );
    }

    #[test]
    fn rejects_unreviewed_or_unproven_recension_mappings() {
        let path = Path::new("alignments.tsv");
        let unknown_status = Table {
            rows: vec![vec![
                "map:test".into(),
                "ocs:test".into(),
                "synodal:test".into(),
                "inherited-from".into(),
                "guessed".into(),
                "compatible".into(),
                "established".into(),
                "9000".into(),
                "evidence".into(),
                "identity-test".into(),
                "fixture".into(),
            ]],
        };
        assert!(validate_alignments(path, &unknown_status).is_err());

        let no_evidence = Table {
            rows: vec![vec![
                "map:test".into(),
                "ocs:test".into(),
                "synodal:test".into(),
                "inherited-from".into(),
                "reviewed".into(),
                "compatible".into(),
                "established".into(),
                "9000".into(),
                String::new(),
                "identity-test".into(),
                "fixture".into(),
            ]],
        };
        assert!(validate_alignments(path, &no_evidence).is_err());
    }

    #[test]
    fn rejects_other_recensions_and_unreviewed_runtime_evidence() {
        assert!(validate_target(Path::new("exact_forms.tsv"), 2, "old-church-slavonic").is_err());
        let reviewed = Table {
            rows: vec![vec![
                "known-evidence".into(),
                "synodal:candidate:known".into(),
                "source".into(),
                "citation".into(),
                "reviewed".into(),
                TARGET.into(),
                "review".into(),
            ]],
        };
        let runtime = Table {
            rows: vec![vec!["lexeme".into(), "missing-evidence".into()]],
        };
        let lexical_reviews = Table { rows: Vec::new() };
        assert!(
            validate_morphology_evidence(
                Path::new("data/synodal"),
                &reviewed,
                &lexical_reviews,
                [(&runtime, &[1_usize][..])],
            )
            .is_err()
        );

        let rejected = Table {
            rows: vec![vec![
                "rejected-evidence".into(),
                "synodal:candidate:rejected".into(),
                "source".into(),
                "citation".into(),
                "rejected".into(),
                TARGET.into(),
                "review".into(),
            ]],
        };
        let rejected_runtime = Table {
            rows: vec![vec!["lexeme".into(), "rejected-evidence".into()]],
        };
        assert!(
            validate_morphology_evidence(
                Path::new("data/synodal"),
                &rejected,
                &lexical_reviews,
                [(&rejected_runtime, &[1_usize][..])],
            )
            .is_err()
        );
    }

    #[test]
    fn exact_candidate_matching_accepts_canonical_unicode_equivalence() {
        let candidate = CandidateLink {
            source_id: "ponomar-elizabeth-bible-2026-08-09".into(),
            target_recension: Some(TARGET.into()),
            partition: Some("source".into()),
            passage: Some("Acts.1.10".into()),
            raw_spelling: "и҆ сѐ, мꙋ̑жа два̀".into(),
            normalized_spelling: String::new(),
        };
        assert!(candidate.contains_exact("сѐ"));
        assert!(
            !CandidateLink {
                raw_spelling: "вѣ́рꙋеши".into(),
                ..candidate
            }
            .contains_exact("вѣ́рꙋ")
        );
    }

    #[test]
    fn productive_lexical_upgrades_must_preserve_reviewed_identity() {
        let productive = vec![
            "synodal:noun:test".into(),
            "имѧ".into(),
            "noun".into(),
            "fourth-neuter-en".into(),
            "имен".into(),
            "neuter".into(),
            String::new(),
            "grammar".into(),
            TARGET.into(),
        ];
        let reviewed = vec![
            "synodal:noun:test".into(),
            "небо".into(),
            "noun".into(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            "attestation".into(),
            TARGET.into(),
        ];
        let mut lexemes = Table {
            rows: vec![productive.clone()],
        };
        let error =
            extend_missing_lexemes(Path::new("lexemes.tsv"), &mut lexemes, vec![reviewed], &[])
                .expect_err("identity mismatch must fail closed");
        assert!(
            error
                .to_string()
                .contains("must preserve the reviewed source or exact target citation")
        );

        let mut compatible = productive;
        compatible[3] = String::new();
        compatible[4] = String::new();
        compatible[5] = String::new();
        compatible[7] = "attestation".into();
        extend_missing_lexemes(
            Path::new("lexemes.tsv"),
            &mut lexemes,
            vec![compatible],
            &[],
        )
        .expect("matching reviewed identity");
        assert_eq!(lexemes.rows.len(), 1);
        assert_eq!(lexemes.rows[0][3], "fourth-neuter-en");

        let target = vec![
            "synodal:noun:stone".into(),
            "камень".into(),
            "noun".into(),
            "fourth-masculine-en-kamen".into(),
            "камен".into(),
            "masculine".into(),
            String::new(),
            "grammar".into(),
            TARGET.into(),
        ];
        let source = vec![
            "synodal:noun:stone".into(),
            "камꙑ".into(),
            "noun".into(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            "attestation".into(),
            TARGET.into(),
        ];
        let exact = vec![
            "synodal:noun:stone".into(),
            "lexical-form".into(),
            "камень".into(),
            "Ка́мень".into(),
            "review".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ];
        let alternate_exact = vec![
            "synodal:noun:stone".into(),
            "lexical-form".into(),
            "камы".into(),
            "Ка́мы".into(),
            "review".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ];
        let mut target_lexemes = Table { rows: vec![target] };
        extend_missing_lexemes(
            Path::new("lexemes.tsv"),
            &mut target_lexemes,
            vec![source],
            &[alternate_exact, exact],
        )
        .expect("reviewed exact target citation preserves the stable identity");
        assert_eq!(target_lexemes.rows.len(), 1);
    }

    #[test]
    fn lexeme_closed_codes_fail_before_registry_generation() {
        let valid = vec![
            "synodal:noun:test".into(),
            "камень".into(),
            "noun".into(),
            "fourth-masculine-en-kamen".into(),
            "камен".into(),
            "masculine".into(),
            String::new(),
            "grammar".into(),
            TARGET.into(),
        ];
        for (column, invalid_value) in [
            (2, "unknown-pos"),
            (3, "unknown-class"),
            (5, "unknown-gender"),
            (6, "unknown-aspect"),
        ] {
            let mut row = valid.clone();
            row[column] = invalid_value.into();
            assert!(
                validate_lexemes(Path::new("lexemes.tsv"), &Table { rows: vec![row] }).is_err(),
                "column {column} must be closed"
            );
        }
    }

    #[test]
    fn grammar_cell_rows_fail_with_source_context_before_emission() {
        let path = Path::new("exact_forms.tsv");
        let error = validate_grammar_cell(path, 17, "noun:ablative:singular:inanimate")
            .expect_err("unknown case must fail before registry emission");
        assert!(matches!(
            error,
            ExtractionError::InvalidRow {
                file,
                line: 17,
                ..
            } if file == path
        ));
        validate_grammar_cell(
            Path::new("abbreviations.tsv"),
            2,
            "pronoun:nominative:singular:any:any",
        )
        .expect("legacy wildcard cells remain accepted");
    }

    #[test]
    fn exact_and_abbreviation_cells_must_match_lexeme_part_of_speech() {
        let lexemes = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "градъ".into(),
                "noun".into(),
                "first-hard-m".into(),
                "град".into(),
                "masculine".into(),
                String::new(),
                "test-source".into(),
                TARGET.into(),
            ]],
        };
        let exact = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "indeclinable".into(),
                "градъ".into(),
                "гра́дъ".into(),
                "test-evidence".into(),
                "normative-table".into(),
                TARGET.into(),
            ]],
        };
        assert!(validate_exact_forms(Path::new("exact_forms.tsv"), &exact, &lexemes).is_err());

        let abbreviations = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "sense:test".into(),
                "indeclinable".into(),
                "градъ".into(),
                "гра́дъ".into(),
                "test-rule".into(),
                "test-evidence".into(),
                "true".into(),
                "titlo".into(),
                "unrestricted".into(),
                "unambiguous".into(),
                TARGET.into(),
                TARGET.into(),
            ]],
        };
        assert!(
            validate_abbreviations(Path::new("abbreviations.tsv"), &abbreviations, &lexemes,)
                .is_err()
        );
    }

    #[test]
    fn abbreviation_families_must_reproduce_every_reviewed_exact_shape() {
        let lexemes = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "градъ".into(),
                "noun".into(),
                "first-hard-m".into(),
                "град".into(),
                "masculine".into(),
                String::new(),
                "test-source".into(),
                TARGET.into(),
            ]],
        };
        let abbreviations = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "sense:test".into(),
                "noun:nominative:singular:inanimate".into(),
                "градъ".into(),
                "гр҃дъ".into(),
                "test-exact-rule".into(),
                "test-evidence".into(),
                "false".into(),
                "titlo".into(),
                "test identity".into(),
                "non-reversible".into(),
                TARGET.into(),
                TARGET.into(),
            ]],
        };
        let mut family_row = vec![
            "synodal:noun:test".into(),
            "sense:test".into(),
            "гра".into(),
            "гр҃".into(),
            "test-family-rule".into(),
            "test-evidence".into(),
            "false".into(),
            "titlo".into(),
            "test identity".into(),
            "non-reversible".into(),
            TARGET.into(),
            TARGET.into(),
        ];
        validate_abbreviation_families(
            Path::new("abbreviation_families.tsv"),
            &Table {
                rows: vec![family_row.clone()],
            },
            &abbreviations,
            &lexemes,
        )
        .expect("matching family skeleton");

        let mut unused = family_row.clone();
        unused[2] = "гро".into();
        assert!(
            validate_abbreviation_families(
                Path::new("abbreviation_families.tsv"),
                &Table {
                    rows: vec![family_row.clone(), unused],
                },
                &abbreviations,
                &lexemes,
            )
            .is_err()
        );

        family_row[3] = "гд҃".into();
        assert!(
            validate_abbreviation_families(
                Path::new("abbreviation_families.tsv"),
                &Table {
                    rows: vec![family_row],
                },
                &abbreviations,
                &lexemes,
            )
            .is_err()
        );
    }

    #[test]
    fn exact_runtime_tuples_must_be_unique() {
        let lexemes = Table {
            rows: vec![vec![
                "synodal:noun:test".into(),
                "градъ".into(),
                "noun".into(),
                "first-hard-m".into(),
                "град".into(),
                "masculine".into(),
                String::new(),
                "test-source".into(),
                TARGET.into(),
            ]],
        };
        let row = vec![
            "synodal:noun:test".into(),
            "noun:nominative:singular:inanimate".into(),
            "градъ".into(),
            "гра́дъ".into(),
            "test-evidence".into(),
            "normative-table".into(),
            TARGET.into(),
        ];
        let duplicate = Table {
            rows: vec![row.clone(), row],
        };
        assert!(validate_exact_forms(Path::new("exact_forms.tsv"), &duplicate, &lexemes).is_err());
    }

    #[test]
    fn one_target_token_cannot_confirm_incompatible_lexical_identities() {
        let lexical_row = |suffix: &str| {
            vec![
                format!("review:{suffix}"),
                format!("synodal:noun:{suffix}"),
                format!("sense:{suffix}"),
                "слово".into(),
                "noun".into(),
                "lexical-form".into(),
                "слово".into(),
                "сло́во".into(),
                format!("sense {suffix}"),
                "general".into(),
                "semantic-source".into(),
                format!("synodal:candidate:semantic:{suffix}"),
                "ponomar-elizabeth-bible-2026-08-09".into(),
                "synodal:candidate:shared-target".into(),
                "Passage.1".into(),
                "reviewed".into(),
                TARGET.into(),
                "contextually reviewed".into(),
            ]
        };
        let lexical = Table {
            rows: vec![lexical_row("one"), lexical_row("two")],
        };
        let ambiguities = Table { rows: vec![] };
        assert!(
            validate_lexical_reviews(Path::new("lexical_reviews.tsv"), &lexical, &ambiguities)
                .is_err()
        );

        let exact = Table {
            rows: vec![
                vec![
                    "synodal:noun:one".into(),
                    "noun:nominative:singular:inanimate".into(),
                    "слово".into(),
                    "сло́во".into(),
                    "shared-target".into(),
                    "synodal-attestation".into(),
                    TARGET.into(),
                ],
                vec![
                    "synodal:noun:two".into(),
                    "noun:nominative:singular:inanimate".into(),
                    "слово".into(),
                    "сло́во".into(),
                    "shared-target-alias".into(),
                    "synodal-attestation".into(),
                    TARGET.into(),
                ],
            ],
        };
        let provenance = Table {
            rows: vec![
                vec![
                    "shared-target".into(),
                    "target-source".into(),
                    TARGET.into(),
                    "Passage.1".into(),
                    "target-attestation".into(),
                    "contextually reviewed".into(),
                ],
                vec![
                    "shared-target-alias".into(),
                    "target-source".into(),
                    TARGET.into(),
                    "Passage.1".into(),
                    "target-attestation".into(),
                    "contextually reviewed".into(),
                ],
            ],
        };
        let reviewed_evidence = Table {
            rows: vec![
                vec![
                    "shared-target".into(),
                    "synodal:candidate:shared-target".into(),
                ],
                vec![
                    "shared-target-alias".into(),
                    "synodal:candidate:shared-target".into(),
                ],
            ],
        };
        let lexical_reviews = Table { rows: vec![] };
        assert!(
            validate_exact_form_attestation_evidence(
                Path::new("exact_forms.tsv"),
                &exact,
                &provenance,
                &reviewed_evidence,
                &lexical_reviews,
                &ambiguities,
            )
            .is_err()
        );

        let adjudicated = Table {
            rows: vec![vec![
                "v07-target-shared".into(),
                "synodal:candidate:shared-target".into(),
                "слово".into(),
                "сло́во".into(),
                "synodal:noun:one".into(),
                "noun:nominative:singular:inanimate".into(),
                "synodal:noun:two".into(),
                "noun:genitive:singular:inanimate".into(),
                "adjudicated".into(),
                "the two exact cells are contextually ambiguous".into(),
            ]],
        };
        let reviewed_ambiguity_evidence = Table {
            rows: vec![vec![
                "v07-target-shared".into(),
                "synodal:candidate:shared-target".into(),
                "ponomar-elizabeth-bible-2026-08-09".into(),
                "Passage.1".into(),
                "reviewed".into(),
                TARGET.into(),
                "contextually reviewed".into(),
            ]],
        };
        validate_target_identity_ambiguities(
            Path::new("ambiguities.tsv"),
            &adjudicated,
            &reviewed_ambiguity_evidence,
        )
        .expect("valid exact-cell adjudication");
        assert!(
            validate_exact_form_attestation_evidence(
                Path::new("exact_forms.tsv"),
                &exact,
                &provenance,
                &reviewed_evidence,
                &lexical_reviews,
                &adjudicated,
            )
            .is_err(),
            "an adjudication for a different cell must not authorize this analysis"
        );
        let mut exact_adjudicated = exact.clone();
        exact_adjudicated.rows[1][1] = "noun:genitive:singular:inanimate".into();
        assert!(
            validate_exact_form_attestation_evidence(
                Path::new("exact_forms.tsv"),
                &exact_adjudicated,
                &provenance,
                &reviewed_evidence,
                &lexical_reviews,
                &adjudicated,
            )
            .is_ok(),
            "only the explicitly adjudicated cell pair is permitted"
        );

        let wrong_owner = lexical_row("one");
        let exact = Table {
            rows: vec![vec![
                "synodal:noun:two".into(),
                "lexical-form".into(),
                "слово".into(),
                "сло́во".into(),
                "review:one".into(),
                "synodal-attestation".into(),
                TARGET.into(),
            ]],
        };
        let provenance = Table {
            rows: vec![vec![
                "review:one".into(),
                "target-source".into(),
                TARGET.into(),
                "Passage.1".into(),
                "reviewed-cell:lexical-form".into(),
                "contextually reviewed".into(),
            ]],
        };
        assert!(
            validate_exact_form_attestation_evidence(
                Path::new("exact_forms.tsv"),
                &exact,
                &provenance,
                &Table { rows: vec![] },
                &Table {
                    rows: vec![wrong_owner],
                },
                &ambiguities,
            )
            .is_err()
        );
    }

    #[test]
    fn reviewed_senses_preserve_registered_source_recension() {
        let reviewed_row = |review_id: &str, source_id: &str| {
            vec![
                review_id.into(),
                format!("synodal:noun:{review_id}"),
                format!("sense:{review_id}"),
                "слово".into(),
                "noun".into(),
                "lexical-form".into(),
                "слово".into(),
                "сло́во".into(),
                "word".into(),
                "general".into(),
                source_id.into(),
                format!("synodal:candidate:{review_id}:semantic"),
                "ponomar-elizabeth-bible-2026-08-09".into(),
                format!("synodal:candidate:{review_id}:attestation"),
                "Passage.1".into(),
                "reviewed".into(),
                TARGET.into(),
                "reviewed fixture".into(),
            ]
        };
        let reviews = Table {
            rows: vec![
                reviewed_row("ocs", "ocs-source"),
                reviewed_row("mixed", "mixed-source"),
                reviewed_row("synodal", "synodal-source"),
            ],
        };
        let source_recensions = BTreeMap::from([
            ("ocs-source".into(), "old-church-slavonic".into()),
            ("mixed-source".into(), "mixed".into()),
            ("synodal-source".into(), "synodal-russian".into()),
            ("ponomar-elizabeth-bible-2026-08-09".into(), TARGET.into()),
        ]);

        let (_, _, senses) = admitted_lexical_review_rows(&reviews, &source_recensions)
            .expect("registered semantic sources");
        assert_eq!(
            senses
                .iter()
                .map(|sense| (sense[5].as_str(), sense[6].as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("old-church-slavonic", "reviewed-ocs-inheritance"),
                ("mixed", "reviewed-with-synodal-corpus"),
                ("synodal-russian", "normative"),
            ]
        );
        assert!(
            admitted_lexical_review_rows(&reviews, &BTreeMap::new()).is_err(),
            "unregistered semantic sources must fail closed"
        );
    }

    #[test]
    fn noun_restrictions_require_a_matching_noun_recension() {
        let restrictions = Table {
            rows: vec![vec![
                "synodal:test".into(),
                "plural-only".into(),
                "evidence:test".into(),
                TARGET.into(),
            ]],
        };
        let lexeme = |part_of_speech: &str, target: &str| Table {
            rows: vec![vec![
                "synodal:test".into(),
                "тестъ".into(),
                part_of_speech.into(),
                "exact".into(),
                String::new(),
                String::new(),
                String::new(),
                "source:test".into(),
                target.into(),
            ]],
        };

        validate_noun_restriction_lexemes(
            Path::new("noun_restrictions.tsv"),
            &restrictions,
            &lexeme("noun", TARGET),
        )
        .expect("matching noun restriction");
        assert!(
            validate_noun_restriction_lexemes(
                Path::new("noun_restrictions.tsv"),
                &restrictions,
                &lexeme("verb", TARGET),
            )
            .is_err()
        );
        assert!(
            validate_noun_restriction_lexemes(
                Path::new("noun_restrictions.tsv"),
                &restrictions,
                &lexeme("noun", "old-church-slavonic"),
            )
            .is_err()
        );
    }

    #[test]
    fn finite_past_audit_is_exhaustive_locked_and_leaves_no_past_cells() {
        let historical = Table {
            rows: vec![vec![
                "v06-exact-03a1ca3817d4918e".into(),
                "admitted".into(),
                "source-typed-exact".into(),
                "family:test".into(),
                "synodal:verb:test".into(),
                "избити".into(),
                "verb".into(),
                "и҆збѝ".into(),
                "1".into(),
                "past:third:singular".into(),
                "semantic:test".into(),
                "morphology:test".into(),
                "target:test".into(),
                "candidate:test".into(),
                "source passage".into(),
                "evaluation passage".into(),
                "1".into(),
                "1".into(),
                String::new(),
                "historical review".into(),
            ]],
        };
        let reviews = Table {
            rows: vec![vec![
                "v06-exact-03a1ca3817d4918e".into(),
                "synodal:verb:test".into(),
                "избити".into(),
                "past:third:singular".into(),
                "и҆збѝ".into(),
                "reclassified-aorist".into(),
                "aorist:third:singular".into(),
                "source passage".into(),
                "evaluation passage".into(),
                "reviewed against the aorist grammar".into(),
            ]],
        };
        let exact = Table {
            rows: vec![vec![
                "synodal:verb:test".into(),
                "aorist:third:singular".into(),
                "изби".into(),
                "и҆збѝ".into(),
                "evidence:test".into(),
                "synodal-attestation".into(),
                TARGET.into(),
            ]],
        };
        let evaluation = Table {
            rows: vec![vec![
                "eval:test".into(),
                "synodal:verb:test".into(),
                "aorist:third:singular".into(),
                "strict".into(),
                "изби".into(),
                "и҆збѝ".into(),
                "source:test".into(),
                "evaluation passage".into(),
                "fixture".into(),
            ]],
        };
        let validate = |reviews: &Table, evaluation: &Table| {
            validate_past_classification_reviews(
                (Path::new("past_classification_reviews.tsv"), reviews),
                (Path::new("v06_exact_reviews.tsv"), &historical),
                (Path::new("exact_forms.tsv"), &exact),
                (Path::new("evaluation.tsv"), evaluation),
            )
        };

        validate(&reviews, &evaluation).expect("complete reclassification audit");
        assert!(validate(&Table { rows: Vec::new() }, &evaluation).is_err());

        let mut altered = reviews.clone();
        altered.rows[0][5] = "reclassified-imperfect".into();
        altered.rows[0][6] = "imperfect:third:singular".into();
        assert!(validate(&altered, &evaluation).is_err());

        let mut surviving_past = evaluation.clone();
        surviving_past.rows[0][2] = "past:third:singular".into();
        assert!(validate(&reviews, &surviving_past).is_err());
    }

    #[test]
    fn target_registry_rejects_the_historically_merged_supine_category() {
        let exact = Table { rows: Vec::new() };
        let evaluation = Table { rows: Vec::new() };
        let validate = |exact: &Table, evaluation: &Table| {
            validate_absent_target_cells(
                (Path::new("exact_forms.tsv"), exact),
                (Path::new("evaluation.tsv"), evaluation),
            )
        };
        validate(&exact, &evaluation).expect("empty target supine inventory");

        let exact_supine = Table {
            rows: vec![vec![
                "synodal:verb:test".into(),
                "supine".into(),
                "нестъ".into(),
                "не́стъ".into(),
                "evidence:test".into(),
                "synodal-attestation".into(),
                TARGET.into(),
            ]],
        };
        assert!(validate(&exact_supine, &evaluation).is_err());

        let evaluation_supine = Table {
            rows: vec![vec![
                "eval:test".into(),
                "synodal:verb:test".into(),
                "supine".into(),
                "strict".into(),
                "нестъ".into(),
                "не́стъ".into(),
                "source:test".into(),
                "passage".into(),
                "fixture".into(),
            ]],
        };
        assert!(validate(&exact, &evaluation_supine).is_err());
    }

    #[test]
    fn defective_inventories_are_closed_typed_and_verb_only() {
        let path = Path::new("verb_defectiveness.tsv");
        let verb = vec![
            "synodal:verb:test".into(),
            "подобати".into(),
            "verb".into(),
            "exact".into(),
            String::new(),
            String::new(),
            String::new(),
            "source".into(),
            TARGET.into(),
        ];
        let noun = vec![
            "synodal:noun:test".into(),
            "слово".into(),
            "noun".into(),
            "exact".into(),
            String::new(),
            "neuter".into(),
            String::new(),
            "source".into(),
            TARGET.into(),
        ];
        let lexemes = Table {
            rows: vec![verb, noun],
        };
        let valid = Table {
            rows: vec![vec![
                "synodal:verb:test".into(),
                "outside-inventory".into(),
                "infinitive,present:third:singular".into(),
                "historically-absent".into(),
                "irregular-override".into(),
                "closed impersonal inventory".into(),
                "evidence:test".into(),
                TARGET.into(),
            ]],
        };
        validate_defective_inventories(path, &valid, &lexemes)
            .expect("valid typed defect inventory");

        let mutate = |column: usize, value: &str| {
            let mut table = valid.clone();
            table.rows[0][column] = value.into();
            table
        };
        assert!(validate_defective_inventories(path, &mutate(1, "unknown"), &lexemes).is_err());
        assert!(
            validate_defective_inventories(path, &mutate(2, "present:fourth:singular"), &lexemes)
                .is_err()
        );
        assert!(validate_defective_inventories(path, &mutate(3, "unknown"), &lexemes).is_err());
        assert!(
            validate_defective_inventories(path, &mutate(4, "untyped-field"), &lexemes).is_err()
        );
        assert!(
            validate_defective_inventories(path, &mutate(0, "synodal:noun:test"), &lexemes)
                .is_err()
        );

        let prefix = Table {
            rows: vec![vec![
                "synodal:verb:test".into(),
                "cell-prefix".into(),
                "participle:present:passive:".into(),
                "historically-absent".into(),
                "participle-formation".into(),
                "explicitly absent system".into(),
                "evidence:test".into(),
                TARGET.into(),
            ]],
        };
        validate_defective_inventories(path, &prefix, &lexemes).expect("valid system-level defect");
    }

    #[test]
    fn irregular_inventory_requires_all_98_source_order_entries() {
        let row = |order: u8| {
            vec![
                order.to_string(),
                format!("headword-{order}"),
                "present".into(),
                "caller-exact-principal-parts".into(),
                "implemented-by-metadata-contract".into(),
                "evidence:test".into(),
                "reviewed source entry".into(),
                TARGET.into(),
            ]
        };
        let complete = Table {
            rows: (2_u8..=100).filter(|order| *order != 97).map(row).collect(),
        };
        let path = Path::new("irregular_verb_inventory.tsv");
        validate_irregular_verb_inventory(path, &complete).expect("complete §104 inventory");

        let mut missing = complete.clone();
        missing.rows.retain(|row| row[0] != "55");
        assert!(validate_irregular_verb_inventory(path, &missing).is_err());

        let mut unknown_system = complete.clone();
        unknown_system.rows[0][2] = "invented-system".into();
        assert!(validate_irregular_verb_inventory(path, &unknown_system).is_err());

        let mut unknown_strategy = complete;
        unknown_strategy.rows[0][3] = "guess".into();
        assert!(validate_irregular_verb_inventory(path, &unknown_strategy).is_err());
    }
}
