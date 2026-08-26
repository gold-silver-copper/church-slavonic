use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::*;

pub(crate) fn read_reviewed_evidence(data_directory: &Path) -> Result<Table> {
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

pub(crate) fn validate_morphology_evidence<const N: usize>(
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

pub(crate) fn evidence_provenance_rows(
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

pub(crate) fn validate_exact_form_attestation_evidence(
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

pub(crate) fn is_target_corpus_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "ponomar-elizabeth-bible-2026-08-09" | "wikisource-church-slavonic-bible-2026-08-09"
    )
}

pub(crate) fn require_direct_target_source<'a>(
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

pub(crate) fn runtime_evidence_ids(data_directory: &Path) -> Result<BTreeSet<String>> {
    let specifications: [(&str, &[usize]); 14] = [
        ("principal_parts.tsv", &[4]),
        ("exact_forms.tsv", &[4]),
        ("alignments.tsv", &[8]),
        ("abbreviations.tsv", &[6]),
        ("abbreviation_families.tsv", &[5]),
        ("accents.tsv", &[4]),
        ("accent_paradigms.tsv", &[6]),
        ("positional_paradigms.tsv", &[4]),
        ("noun_restrictions.tsv", &[3]),
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
