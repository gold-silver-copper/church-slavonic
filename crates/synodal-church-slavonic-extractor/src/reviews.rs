use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use synodal_church_slavonic_core::GrammarCell;

use super::*;

pub(crate) fn read_lexical_reviews(data_directory: &Path) -> Result<Table> {
    read_table(
        &data_directory.join("lexical_reviews.tsv"),
        LEXICAL_REVIEW_HEADER,
        18,
    )
}

pub(crate) fn expected_past_classification(review_id: &str) -> &'static str {
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

pub(crate) fn validate_past_classification_reviews(
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

pub(crate) fn validate_absent_target_cells(
    exact: (&Path, &Table),
    held_out: (&Path, &Table),
) -> Result<()> {
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

pub(crate) fn load_source_recensions(data_directory: &Path) -> Result<BTreeMap<String, String>> {
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

pub(crate) fn target_identity_is_adjudicated(
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

pub(crate) fn validate_target_identity_ambiguities(
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

pub(crate) fn validate_lexical_reviews(
    path: &Path,
    table: &Table,
    ambiguities: &Table,
) -> Result<()> {
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
        let cell = row[5]
            .parse::<GrammarCell>()
            .map_err(|error| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line,
                reason: error.to_string(),
            })?;
        let compatible = if closed {
            cell == GrammarCell::Indeclinable
        } else if inflectable {
            grammar_cell_matches_part_of_speech(cell, &row[4])
        } else {
            false
        };
        if !compatible {
            return invalid(
                path,
                line,
                "part of speech must use a matching exact-only or explicitly typed grammar cell",
            );
        }
    }
    Ok(())
}

pub(crate) type AdmittedLexicalReviewRows = (Vec<Vec<String>>, Vec<Vec<String>>, Vec<Vec<String>>);

pub(crate) fn extend_missing_lexemes(
    path: &Path,
    lexemes: &mut Table,
    reviewed: Vec<Vec<String>>,
    reviewed_exact_forms: &[Vec<String>],
    target_exact_forms: &[Vec<String>],
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
                || reviewed_exact_forms
                    .iter()
                    .chain(target_exact_forms)
                    .any(|form| {
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

pub(crate) fn extend_reviewed_exact_forms(
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

pub(crate) fn admitted_lexical_review_rows(
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
