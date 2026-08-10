//! Offline, deterministic source adapters for Synodal Russian Church Slavonic.
#![forbid(unsafe_code)]

pub mod adapters;

use std::{
    collections::BTreeSet,
    error, fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use synodal_church_slavonic_core::{RenderedText, SynodalWord};

/// Schema version for normalized Synodal registries.
pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

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

/// Validates reviewable TSV and atomically writes the generated Rust registry.
pub fn generate_registry(data_directory: &Path, destination: &Path) -> Result<GenerationReport> {
    let lexeme_path = data_directory.join("lexemes.tsv");
    let principal_path = data_directory.join("principal_parts.tsv");
    let exact_path = data_directory.join("exact_forms.tsv");
    let alignment_path = data_directory.join("alignments.tsv");
    let abbreviation_path = data_directory.join("abbreviations.tsv");
    let accent_path = data_directory.join("accents.tsv");
    let positional_path = data_directory.join("positional_rules.tsv");
    let transformation_path = data_directory.join("transformation_rules.tsv");
    let conflict_path = data_directory.join("conflicts.tsv");
    let irregular_path = data_directory.join("irregular_overrides.tsv");

    let lexemes = read_table(
        &lexeme_path,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
    )?;
    let principal_parts = read_table(
        &principal_path,
        "lexeme_id\tsystem\tvalue\tformation\tevidence_id\ttarget_recension",
        6,
    )?;
    let exact_forms = read_table(
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
        "lexeme_id\tsense_id\texpanded\tprinted\trule_id\tevidence_id\treversible\ttarget_recension",
        8,
    )?;
    let accents = read_table(
        &accent_path,
        "lexeme_id\tcell\texpanded\taccented\tevidence_id\tsource_id\tsource_recension\ttarget_recension",
        8,
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

    validate_lexemes(&lexeme_path, &lexemes)?;
    validate_principal_parts(&principal_path, &principal_parts)?;
    validate_exact_forms(&exact_path, &exact_forms)?;
    validate_alignments(&alignment_path, &alignments)?;
    validate_abbreviations(&abbreviation_path, &abbreviations)?;
    validate_accents(&accent_path, &accents)?;
    validate_positional_rules(&positional_path, &positional_rules)?;
    validate_transformation_rules(&transformation_path, &transformation_rules)?;
    validate_conflicts(&conflict_path, &conflicts)?;
    validate_irregular_overrides(&irregular_path, &irregular_overrides)?;
    validate_morphology_references(
        &lexeme_path,
        &lexemes,
        [
            (&principal_path, &principal_parts, 0),
            (&exact_path, &exact_forms, 0),
            (&abbreviation_path, &abbreviations, 0),
            (&accent_path, &accents, 0),
            (&irregular_path, &irregular_overrides, 0),
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
        principal_parts: principal_parts.clone(),
        exact_forms: exact_forms.clone(),
        alignments: alignments.clone(),
        abbreviations: abbreviations.clone(),
        accents: accents.clone(),
        positional_rules: positional_rules.clone(),
        transformation_rules: transformation_rules.clone(),
        conflicts: conflicts.clone(),
        irregular_overrides: irregular_overrides.clone(),
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
    let senses = read_table(
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
    validate_senses(&sense_path, &senses)?;
    validate_examples(&example_path, &examples)?;
    validate_semantic_alignments(&semantic_alignment_path, &semantic_alignments)?;
    let lexemes = read_table(
        &data_directory.join("lexemes.tsv"),
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
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
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        validate_target(path, offset + 2, &row[8])?;
        validate_word(path, offset + 2, &row[1], "lemma")?;
        if !row[4].is_empty() {
            validate_word(path, offset + 2, &row[4], "stem")?;
        }
        if !row[0].starts_with("synodal:") {
            return invalid(
                path,
                offset + 2,
                "target lexeme IDs must use the synodal namespace",
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
    }
    Ok(())
}

fn validate_exact_forms(path: &Path, table: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[6])?;
        validate_word(path, offset + 2, &row[2], "expanded form")?;
        validate_word(path, offset + 2, &row[3], "printed form")?;
        if !matches!(row[5].as_str(), "normative-table" | "synodal-attestation") {
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

fn validate_abbreviations(path: &Path, table: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[7])?;
        validate_word(path, offset + 2, &row[2], "expanded abbreviation")?;
        validate_word(path, offset + 2, &row[3], "printed abbreviation")?;
        if row[1].is_empty() {
            return invalid(
                path,
                offset + 2,
                "abbreviation rows require a semantic sense ID",
            );
        }
    }
    Ok(())
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

fn validate_senses(path: &Path, table: &Table) -> Result<()> {
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
        if row[5] == "mixed" && row[6] != "reviewed-with-synodal-corpus" {
            return invalid(
                path,
                offset + 2,
                "mixed-recension meanings require explicit target-corpus review",
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
    if lower.contains("slovowiki") || lower.contains("interslavic") {
        invalid(
            path,
            line,
            "Interslavic and Slovowiki are forbidden linguistic authorities",
        )
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
    principal_parts: Table,
    exact_forms: Table,
    alignments: Table,
    abbreviations: Table,
    accents: Table,
    positional_rules: Table,
    transformation_rules: Table,
    conflicts: Table,
    irregular_overrides: Table,
}

fn emit_registry(tables: RegistryTables) -> String {
    let RegistryTables {
        mut lexemes,
        mut principal_parts,
        mut exact_forms,
        mut alignments,
        mut abbreviations,
        mut accents,
        mut positional_rules,
        mut transformation_rules,
        mut conflicts,
        mut irregular_overrides,
    } = tables;
    lexemes.rows.sort();
    principal_parts.rows.sort();
    exact_forms.rows.sort();
    alignments.rows.sort();
    abbreviations.rows.sort();
    accents.rows.sort();
    positional_rules.rows.sort();
    transformation_rules.rows.sort();
    conflicts.rows.sort();
    irregular_overrides.rows.sort();

    let mut output = String::from(
        "// @generated by synodal-church-slavonic-extractor; do not edit.\n\
         // Source: data/synodal/*.tsv\n\n",
    );
    emit_rows(&mut output, "LEXEMES", "RawLexeme", &lexemes.rows);
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
    emit_rows(&mut output, "ACCENTS", "RawAccent", &accents.rows);
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
}
