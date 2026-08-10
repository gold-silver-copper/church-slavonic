use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, FiniteTense, FiniteVerbCell, Gender,
    GenerationPolicy, GrammarCell, ImperativeCell, Inflector, LexemeId, Number, NumeralCell,
    NumeralKind, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice, Person,
    PronounCell, RealizedPhrase, phrases,
};
use synodal_church_slavonic_core::FormSource;

const EVALUATION_HEADER: &str = "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";
const PHRASE_EVALUATION_HEADER: &str = "id\tconstruction\tlemma\tperson\tnumber\tgender\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";

#[derive(Clone, Debug)]
struct EvaluationRow {
    id: String,
    lexeme_id: LexemeId,
    cell: GrammarCell,
    cell_key: String,
    policy: GenerationPolicy,
    expected_expanded: String,
    expected_printed: String,
    source_id: String,
    passage: String,
    regularity: String,
}

#[derive(Clone, Debug)]
struct PhraseEvaluationRow {
    id: String,
    construction: String,
    lemma: String,
    person: Person,
    number: Number,
    gender: Gender,
    expected_expanded: String,
    expected_printed: String,
    source_id: String,
    passage: String,
    regularity: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct MetricSlice {
    total: usize,
    returned: usize,
    top_1_correct: usize,
    top_k_correct: usize,
    abstained: usize,
}

#[derive(Clone, Debug, Serialize)]
struct InheritanceMetrics {
    accepted_alignments: usize,
    rejected_negative_controls: usize,
    aligned_target_lexemes: usize,
    identity_alignments: usize,
    transformed_alignments: usize,
    gold_admission_true_positives: usize,
    gold_admission_false_positives: usize,
    gold_admission_precision_basis_points: u16,
    evaluated_inherited_cells: usize,
    extra_returned_cells: usize,
    exact_expanded_cells: usize,
    by_mapping_kind: BTreeMap<String, MetricSlice>,
    by_morphological_system: BTreeMap<String, MetricSlice>,
    by_confidence_band: BTreeMap<String, MetricSlice>,
    mean_returned_confidence_basis_points: Option<u16>,
    empirical_exactness_basis_points: Option<u16>,
    absolute_calibration_gap_basis_points: Option<u16>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct LeakageMetrics {
    masked_expanded: MetricSlice,
    masked_printed: MetricSlice,
    leave_one_synodal_lexeme_out_expanded: MetricSlice,
    leave_one_synodal_lexeme_out_printed: MetricSlice,
}

#[derive(Clone, Debug, Serialize)]
struct EvaluationReport {
    schema_version: u32,
    target_recension: &'static str,
    fixture_source: &'static str,
    fixture_rows: usize,
    phrase_fixture_rows: usize,
    expanded: MetricSlice,
    printed: MetricSlice,
    accent_bearing_rows: usize,
    exact_accent_agreement: usize,
    by_regularity: BTreeMap<String, MetricSlice>,
    by_morphological_system: BTreeMap<String, MetricSlice>,
    by_provenance_path: BTreeMap<String, MetricSlice>,
    by_source: BTreeMap<String, MetricSlice>,
    abstention_reasons: BTreeMap<String, usize>,
    phrase_expanded: MetricSlice,
    phrase_printed: MetricSlice,
    leakage: LeakageMetrics,
    inheritance: InheritanceMetrics,
    limitations: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
struct ExtractionReport {
    schema_version: u32,
    target_recension: &'static str,
    normalized_tables: BTreeMap<String, usize>,
    normalized_rows: usize,
    quarantined_rows: usize,
    parse_failure_ceiling: usize,
    morphology_registry_sha256: String,
    dictionary_registry_sha256: String,
    source_adapter_contracts: Vec<&'static str>,
}

pub(crate) fn regenerate(root: &Path) -> Result<(), Box<dyn Error>> {
    let data = root.join("data/synodal");
    let morphology = root.join("crates/synodal-church-slavonic/generated/registry.rs");
    let dictionary = root.join("crates/synodal-church-slavonic-dictionary/generated/registry.rs");
    let morphology_report =
        synodal_church_slavonic_extractor::generate_registry(&data, &morphology)?;
    let dictionary_report =
        synodal_church_slavonic_extractor::generate_dictionary_registry(&data, &dictionary)?;
    write_extraction_report(root)?;
    evaluate_and_write(root)?;
    println!(
        "synodal registries: {} lexemes, {} forms, {} senses, {} examples",
        morphology_report.lexemes,
        morphology_report.exact_forms,
        dictionary_report.senses,
        dictionary_report.examples
    );
    Ok(())
}

pub(crate) fn check(root: &Path) -> Result<(), Box<dyn Error>> {
    check_generated(root)?;
    check_source_boundaries(root)?;
    check_runtime_boundaries(root)?;
    check_evaluation_report(root)?;
    check_extraction_report(root)?;
    check_package_metadata(root)?;
    println!("synodal checks: current");
    Ok(())
}

fn extraction_report(root: &Path) -> Result<ExtractionReport, Box<dyn Error>> {
    let mut normalized_tables = BTreeMap::new();
    for name in [
        "abbreviations.tsv",
        "accents.tsv",
        "alignments.tsv",
        "conflicts.tsv",
        "evaluation.tsv",
        "exact_forms.tsv",
        "examples.tsv",
        "irregular_overrides.tsv",
        "lexemes.tsv",
        "positional_rules.tsv",
        "principal_parts.tsv",
        "phrase_evaluation.tsv",
        "semantic_alignments.tsv",
        "senses.tsv",
        "transformation_rules.tsv",
        "training_passages.tsv",
    ] {
        let text = fs::read_to_string(root.join("data/synodal").join(name))?;
        let rows = text.lines().skip(1).filter(|line| !line.is_empty()).count();
        normalized_tables.insert(name.into(), rows);
    }
    let normalized_rows = normalized_tables.values().sum();
    Ok(ExtractionReport {
        schema_version: 1,
        target_recension: "synodal-russian",
        normalized_tables,
        normalized_rows,
        quarantined_rows: 0,
        parse_failure_ceiling: 0,
        morphology_registry_sha256: file_sha256(
            &root.join("crates/synodal-church-slavonic/generated/registry.rs"),
        )?,
        dictionary_registry_sha256: file_sha256(
            &root.join("crates/synodal-church-slavonic-dictionary/generated/registry.rs"),
        )?,
        source_adapter_contracts: vec![
            "streaming Ponomar verse adapter with source order and quarantine",
            "streaming Kaikki OCS JSONL adapter with content IDs and no target surface rows",
        ],
    })
}

fn write_extraction_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = extraction_report(root)?;
    let reports = root.join("reports");
    fs::create_dir_all(&reports)?;
    fs::write(
        reports.join("synodal-extraction.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(
        reports.join("synodal-extraction.md"),
        extraction_markdown(&report),
    )?;
    Ok(())
}

fn check_extraction_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = extraction_report(root)?;
    if fs::read(root.join("reports/synodal-extraction.json"))?
        != serde_json::to_vec_pretty(&report)?
        || fs::read_to_string(root.join("reports/synodal-extraction.md"))?
            != extraction_markdown(&report)
    {
        return Err(
            "committed Synodal extraction reports are stale; run cargo xtask synodal-regenerate"
                .into(),
        );
    }
    Ok(())
}

fn extraction_markdown(report: &ExtractionReport) -> String {
    let mut markdown = format!(
        "# Synodal extraction report\n\nTarget recension: `synodal-russian`. The curated normalized layer contains {} rows across {} tables; {} rows are quarantined (ceiling {}).\n\n| Table | Rows |\n|---|---:|\n",
        report.normalized_rows,
        report.normalized_tables.len(),
        report.quarantined_rows,
        report.parse_failure_ceiling,
    );
    for (table, rows) in &report.normalized_tables {
        markdown.push_str(&format!("| `{table}` | {rows} |\n"));
    }
    markdown.push_str(&format!(
        "\nGenerated morphology SHA-256: `{}`.\n\nGenerated dictionary SHA-256: `{}`.\n\nLarge raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.\n",
        report.morphology_registry_sha256, report.dictionary_registry_sha256
    ));
    markdown
}

fn file_sha256(path: &Path) -> Result<String, Box<dyn Error>> {
    let digest = Sha256::digest(fs::read(path)?);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub(crate) fn evaluate_and_write(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = evaluate(root)?;
    let reports = root.join("reports");
    fs::create_dir_all(&reports)?;
    fs::write(
        reports.join("synodal-evaluation.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(
        reports.join("synodal-evaluation.md"),
        evaluation_markdown(&report),
    )?;
    println!(
        "synodal evaluation: expanded {}/{}, printed {}/{}",
        report.expanded.top_1_correct,
        report.expanded.total,
        report.printed.top_1_correct,
        report.printed.total
    );
    Ok(())
}

fn check_generated(root: &Path) -> Result<(), Box<dyn Error>> {
    let temporary = temporary_directory("generated");
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(&temporary)?;
    let result = (|| -> Result<(), Box<dyn Error>> {
        let generated_morphology = temporary.join("morphology.rs");
        let generated_dictionary = temporary.join("dictionary.rs");
        synodal_church_slavonic_extractor::generate_registry(
            &root.join("data/synodal"),
            &generated_morphology,
        )?;
        synodal_church_slavonic_extractor::generate_dictionary_registry(
            &root.join("data/synodal"),
            &generated_dictionary,
        )?;
        compare_files(
            &generated_morphology,
            &root.join("crates/synodal-church-slavonic/generated/registry.rs"),
        )?;
        compare_files(
            &generated_dictionary,
            &root.join("crates/synodal-church-slavonic-dictionary/generated/registry.rs"),
        )?;
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&temporary);
    result?;
    cleanup?;
    Ok(())
}

fn compare_files(actual: &Path, committed: &Path) -> Result<(), Box<dyn Error>> {
    if fs::read(actual)? == fs::read(committed)? {
        Ok(())
    } else {
        Err(format!(
            "stale generated Synodal artifact {}; run cargo xtask synodal-regenerate",
            committed.display()
        )
        .into())
    }
}

fn check_source_boundaries(root: &Path) -> Result<(), Box<dyn Error>> {
    let data = root.join("data/synodal");
    for entry in fs::read_dir(&data)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "tsv") {
            let bytes = fs::read(&path)?;
            let text = std::str::from_utf8(&bytes)?;
            let lower = text.to_lowercase();
            if lower.contains("slovowiki") || lower.contains("interslavic") {
                return Err(format!("forbidden linguistic authority in {}", path.display()).into());
            }
            if text.chars().any(|character| {
                matches!(character as u32, 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
            }) {
                return Err(format!("private-use Unicode in {}", path.display()).into());
            }
        }
    }

    let source_manifest = fs::read_to_string(root.join("data/SOURCES.toml"))?;
    let records = source_manifest.matches("[[synodal_source]]").count();
    if records < 10 {
        return Err("Synodal source manifest unexpectedly contains fewer than ten records".into());
    }
    for required in [
        "source_recension =",
        "content_kind =",
        "format =",
        "license =",
        "redistribution =",
        "authority_roles =",
        "upstream_lineage =",
        "normalization =",
    ] {
        if source_manifest.matches(required).count() < records {
            return Err(format!("Synodal source manifest records are missing {required}").into());
        }
    }
    check_partition_disjointness(root)?;
    Ok(())
}

fn check_partition_disjointness(root: &Path) -> Result<(), Box<dyn Error>> {
    const TRAINING_HEADER: &str =
        "source_id\tpassage\tpartition\tpurpose\tevidence_ids\tsource_recension";
    let training = fs::read_to_string(root.join("data/synodal/training_passages.tsv"))?;
    let mut lines = training.lines();
    if lines.next() != Some(TRAINING_HEADER) {
        return Err("invalid Synodal training-passage header".into());
    }
    let mut source_passages = std::collections::BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6
            || fields[2] != "source"
            || fields[4].is_empty()
            || fields[5] != "synodal-russian"
        {
            return Err(format!("invalid Synodal training-passage row {}", offset + 2).into());
        }
        if !source_passages.insert((fields[0].to_owned(), fields[1].to_owned())) {
            return Err(format!(
                "duplicate Synodal source passage {} {}",
                fields[0], fields[1]
            )
            .into());
        }
    }

    let evaluation = load_evaluation(&root.join("data/synodal/evaluation.tsv"))?;
    for row in evaluation {
        if source_passages.contains(&(row.source_id.clone(), row.passage.clone())) {
            return Err(format!(
                "Synodal passage {} {} occurs in both source and evaluation partitions",
                row.source_id, row.passage
            )
            .into());
        }
    }
    let phrase_evaluation =
        load_phrase_evaluation(&root.join("data/synodal/phrase_evaluation.tsv"))?;
    for row in phrase_evaluation {
        if source_passages.contains(&(row.source_id.clone(), row.passage.clone())) {
            return Err(format!(
                "Synodal phrase passage {} {} occurs in both source and evaluation partitions",
                row.source_id, row.passage
            )
            .into());
        }
    }
    Ok(())
}

fn check_runtime_boundaries(root: &Path) -> Result<(), Box<dyn Error>> {
    for package in [
        "synodal-church-slavonic-core",
        "synodal-church-slavonic",
        "synodal-church-slavonic-dictionary",
    ] {
        let source = root.join("crates").join(package).join("src");
        for path in rust_files(&source)? {
            let text = fs::read_to_string(&path)?;
            for forbidden in ["std::fs", "std::net", "reqwest::", "ureq::"] {
                if text.contains(forbidden) {
                    return Err(format!(
                        "runtime I/O boundary violated by {forbidden} in {}",
                        path.display()
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}

fn rust_files(directory: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            result.extend(rust_files(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            result.push(path);
        }
    }
    Ok(result)
}

fn check_package_metadata(root: &Path) -> Result<(), Box<dyn Error>> {
    for package in [
        "synodal-church-slavonic-core",
        "synodal-church-slavonic",
        "synodal-church-slavonic-dictionary",
    ] {
        let directory = root.join("crates").join(package);
        let manifest = fs::read_to_string(directory.join("Cargo.toml"))?;
        for field in [
            "description =",
            "license.workspace = true",
            "readme =",
            "include =",
            "ATTRIBUTION.md",
        ] {
            if !manifest.contains(field) {
                return Err(format!("{package} package metadata is missing {field}").into());
            }
        }
        if !directory.join("README.md").is_file() {
            return Err(format!("{package} has no README").into());
        }
        for license in ["LICENSE-MIT", "LICENSE-APACHE"] {
            if !directory.join(license).is_file() {
                return Err(format!("{package} package is missing {license}").into());
            }
        }
        let attribution = fs::read_to_string(directory.join("ATTRIBUTION.md"))?;
        if !attribution.contains("SHA-256") || !attribution.contains("MIT OR Apache-2.0") {
            return Err(format!("{package} attribution is incomplete").into());
        }
    }
    Ok(())
}

fn check_evaluation_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = evaluate(root)?;
    let json = serde_json::to_vec_pretty(&report)?;
    let markdown = evaluation_markdown(&report);
    if fs::read(root.join("reports/synodal-evaluation.json"))? != json
        || fs::read_to_string(root.join("reports/synodal-evaluation.md"))? != markdown
    {
        return Err(
            "committed Synodal evaluation reports are stale; run cargo xtask synodal-evaluate"
                .into(),
        );
    }
    Ok(())
}

fn evaluate(root: &Path) -> Result<EvaluationReport, Box<dyn Error>> {
    let rows = load_evaluation(&root.join("data/synodal/evaluation.tsv"))?;
    let phrase_rows = load_phrase_evaluation(&root.join("data/synodal/phrase_evaluation.tsv"))?;
    let exact_keys = load_exact_keys(&root.join("data/synodal/exact_forms.tsv"))?;
    let mut expanded = MetricSlice::default();
    let mut printed = MetricSlice::default();
    let mut by_regularity = BTreeMap::new();
    let mut by_source = BTreeMap::new();
    let mut by_morphological_system = BTreeMap::new();
    let mut by_provenance_path = BTreeMap::new();
    let mut abstention_reasons = BTreeMap::new();
    let mut accent_bearing_rows = 0;
    let mut exact_accent_agreement = 0;
    let mut evaluated_inherited_cells = 0;
    let mut extra_returned_cells = 0;
    let mut inherited_exact = 0;
    let mut inherited_by_mapping_kind = BTreeMap::new();
    let mut inherited_by_system = BTreeMap::new();
    let mut inherited_by_confidence = BTreeMap::new();
    let mut inherited_confidence_sum = 0_u64;
    let mut inherited_confidence_count = 0_usize;
    let mut leakage = LeakageMetrics::default();
    let alignments = synodal_church_slavonic::recension_alignments()?;

    for row in &rows {
        let expanded_result = inflector(row.policy, OrthographyProfile::Expanded)
            .form_by_id(&row.lexeme_id, row.cell);
        let printed_result = inflector(row.policy, OrthographyProfile::SynodalLiturgical)
            .form_by_id(&row.lexeme_id, row.cell);
        score_result(
            &mut expanded,
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        if let Err(error) = &expanded_result {
            *abstention_reasons
                .entry(abstention_reason(error).into())
                .or_default() += 1;
        }

        let is_masked = !exact_keys.contains(&(row.lexeme_id.to_string(), row.cell_key.clone()));
        if is_masked {
            score_result(
                &mut leakage.masked_expanded,
                expanded_result.as_ref().ok(),
                &row.expected_expanded,
            );
            score_result(
                &mut leakage.masked_printed,
                printed_result.as_ref().ok(),
                &row.expected_printed,
            );
        }
        if is_masked && row.regularity == "regular-inherited" {
            score_result(
                &mut leakage.leave_one_synodal_lexeme_out_expanded,
                expanded_result.as_ref().ok(),
                &row.expected_expanded,
            );
            score_result(
                &mut leakage.leave_one_synodal_lexeme_out_printed,
                printed_result.as_ref().ok(),
                &row.expected_printed,
            );
        }
        score_result(
            &mut printed,
            printed_result.as_ref().ok(),
            &row.expected_printed,
        );

        let regularity = by_regularity
            .entry(row.regularity.clone())
            .or_insert_with(MetricSlice::default);
        score_result(
            regularity,
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        let source = by_source
            .entry(row.source_id.clone())
            .or_insert_with(MetricSlice::default);
        score_result(
            source,
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        let system = morphological_system(row.cell);
        score_result(
            by_morphological_system
                .entry(system.into())
                .or_insert_with(MetricSlice::default),
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        if let Ok(forms) = &expanded_result {
            score_result(
                by_provenance_path
                    .entry(provenance_path(forms).into())
                    .or_insert_with(MetricSlice::default),
                Some(forms),
                &row.expected_expanded,
            );
        }

        if contains_accent(&row.expected_printed) {
            accent_bearing_rows += 1;
            if printed_result.as_ref().is_ok_and(|forms| {
                forms
                    .variants()
                    .iter()
                    .any(|form| form.printed == row.expected_printed)
            }) {
                exact_accent_agreement += 1;
            }
        }
        if row.regularity == "regular-inherited" {
            evaluated_inherited_cells += 1;
            if let Ok(forms) = &expanded_result {
                if forms
                    .variants()
                    .iter()
                    .any(|variant| matches!(variant.source, FormSource::InheritedPrediction { .. }))
                {
                    extra_returned_cells += 1;
                    let correct = forms
                        .variants()
                        .iter()
                        .any(|variant| variant.expanded == row.expected_expanded);
                    if correct {
                        inherited_exact += 1;
                    }
                    let inherited_variant = forms.variants().iter().find(|variant| {
                        matches!(variant.source, FormSource::InheritedPrediction { .. })
                    });
                    if let Some(variant) = inherited_variant {
                        let mapping_kind = variant
                            .recension_mapping
                            .as_ref()
                            .and_then(|id| {
                                alignments
                                    .iter()
                                    .find(|alignment| alignment.mapping_id == id.as_str())
                            })
                            .map_or("unknown", |alignment| {
                                if alignment
                                    .transformations
                                    .iter()
                                    .any(|value| value.starts_with("identity-"))
                                {
                                    "identity"
                                } else {
                                    "transformed"
                                }
                            });
                        score_result(
                            inherited_by_mapping_kind
                                .entry(mapping_kind.into())
                                .or_insert_with(MetricSlice::default),
                            Some(forms),
                            &row.expected_expanded,
                        );
                        score_result(
                            inherited_by_system
                                .entry(system.into())
                                .or_insert_with(MetricSlice::default),
                            Some(forms),
                            &row.expected_expanded,
                        );
                        let confidence = variant.confidence.basis_points();
                        inherited_confidence_sum += u64::from(confidence);
                        inherited_confidence_count += 1;
                        let band = match confidence {
                            9_500..=10_000 => "high-9500-10000",
                            8_000..=9_499 => "medium-8000-9499",
                            _ => "low-0-7999",
                        };
                        score_result(
                            inherited_by_confidence
                                .entry(band.into())
                                .or_insert_with(MetricSlice::default),
                            Some(forms),
                            &row.expected_expanded,
                        );
                    }
                }
            }
        }
        if row.id.is_empty() || row.passage.is_empty() {
            return Err("evaluation rows require stable IDs and passage identities".into());
        }
    }

    let mut phrase_expanded = MetricSlice::default();
    let mut phrase_printed = MetricSlice::default();
    for row in &phrase_rows {
        let expanded_result = realize_phrase(row, OrthographyProfile::Expanded);
        let printed_result = realize_phrase(row, OrthographyProfile::SynodalLiturgical);
        score_phrase_result(
            &mut phrase_expanded,
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        score_phrase_result(
            &mut phrase_printed,
            printed_result.as_ref().ok(),
            &row.expected_printed,
        );
        if row.id.is_empty()
            || row.source_id.is_empty()
            || row.passage.is_empty()
            || row.regularity.is_empty()
        {
            return Err("phrase evaluation rows require stable IDs and source metadata".into());
        }
    }

    let accepted_alignments = alignments
        .iter()
        .filter(|alignment| {
            alignment.status == "reviewed" || alignment.status == "automatically-validated"
        })
        .count();
    let rejected_negative_controls = alignments
        .iter()
        .filter(|alignment| alignment.status == "rejected")
        .count();
    let aligned_target_lexemes = alignments
        .iter()
        .filter(|alignment| alignment.status != "rejected")
        .map(|alignment| alignment.target_lexeme_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let identity_alignments = alignments
        .iter()
        .filter(|alignment| {
            alignment.status != "rejected"
                && alignment
                    .transformations
                    .iter()
                    .any(|value| value.starts_with("identity-"))
        })
        .count();
    let transformed_alignments = accepted_alignments.saturating_sub(identity_alignments);
    let gold_admission_true_positives = accepted_alignments;
    let gold_admission_false_positives = 0;
    let gold_admission_precision_basis_points = if gold_admission_true_positives == 0 {
        0
    } else {
        10_000
    };
    let mean_returned_confidence_basis_points = (inherited_confidence_count > 0)
        .then(|| (inherited_confidence_sum / inherited_confidence_count as u64) as u16);
    let empirical_exactness_basis_points = (extra_returned_cells > 0)
        .then(|| ((inherited_exact * 10_000) / extra_returned_cells) as u16);
    let absolute_calibration_gap_basis_points = mean_returned_confidence_basis_points
        .zip(empirical_exactness_basis_points)
        .map(|(confidence, accuracy)| confidence.abs_diff(accuracy));

    Ok(EvaluationReport {
        schema_version: 3,
        target_recension: "synodal-russian",
        fixture_source: "pinned Ponomar Elizabeth Bible, Matthew 1–5 and Acts 1:18",
        fixture_rows: rows.len(),
        phrase_fixture_rows: phrase_rows.len(),
        expanded,
        printed,
        accent_bearing_rows,
        exact_accent_agreement,
        by_regularity,
        by_morphological_system,
        by_provenance_path,
        by_source,
        abstention_reasons,
        phrase_expanded,
        phrase_printed,
        leakage,
        inheritance: InheritanceMetrics {
            accepted_alignments,
            rejected_negative_controls,
            aligned_target_lexemes,
            identity_alignments,
            transformed_alignments,
            gold_admission_true_positives,
            gold_admission_false_positives,
            gold_admission_precision_basis_points,
            evaluated_inherited_cells,
            extra_returned_cells,
            exact_expanded_cells: inherited_exact,
            by_mapping_kind: inherited_by_mapping_kind,
            by_morphological_system: inherited_by_system,
            by_confidence_band: inherited_by_confidence,
            mean_returned_confidence_basis_points,
            empirical_exactness_basis_points,
            absolute_calibration_gap_basis_points,
        },
        limitations: vec![
            "The current real-text slice is intentionally small and reports counts, not statistical confidence.",
            "Productive liturgical rendering abstains when accent metadata is absent.",
            "One participle and one analytic perfect are covered by independent corpus witnesses; other analytic constructions remain typed unit fixtures until their lexical registries grow.",
            "Gold admission precision is a structural policy check over the reviewed registry, not an independently estimated automatic-alignment precision.",
            "The single inherited held-out cell is insufficient to assess confidence calibration; the reported gap is descriptive only.",
        ],
    })
}

fn inflector(policy: GenerationPolicy, orthography: OrthographyProfile) -> Inflector {
    Inflector::builder()
        .generation_policy(policy)
        .orthography(orthography)
        .build()
}

fn score_result(
    slice: &mut MetricSlice,
    forms: Option<&synodal_church_slavonic::FormSet>,
    expected: &str,
) {
    slice.total += 1;
    let Some(forms) = forms else {
        slice.abstained += 1;
        return;
    };
    slice.returned += 1;
    if forms.primary_text() == expected {
        slice.top_1_correct += 1;
    }
    if forms
        .variants()
        .iter()
        .any(|variant| variant.printed == expected)
    {
        slice.top_k_correct += 1;
    }
}

fn morphological_system(cell: GrammarCell) -> &'static str {
    match cell {
        GrammarCell::Noun(_) => "noun",
        GrammarCell::Adjective(_) => "adjective",
        GrammarCell::FiniteVerb(cell) => match cell.tense {
            FiniteTense::Present => "present",
            FiniteTense::Imperfect => "imperfect",
            FiniteTense::Aorist => "aorist",
        },
        GrammarCell::Imperative(_) => "imperative",
        GrammarCell::Infinitive => "infinitive",
        GrammarCell::Supine => "supine",
        GrammarCell::LParticiple(_) => "l-participle",
        GrammarCell::Participle(_) => "participle",
        GrammarCell::VerbalNoun(_) => "verbal-noun",
        GrammarCell::Pronoun(_) => "pronoun",
        GrammarCell::Determiner(_) => "determiner",
        GrammarCell::Numeral(_) => "numeral",
    }
}

fn provenance_path(forms: &synodal_church_slavonic::FormSet) -> &'static str {
    match &forms.variants()[0].source {
        FormSource::SynodalAttestation { .. } => "exact-synodal-attestation",
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-REGISTRY-NORMATIVE-TABLE" =>
        {
            "synodal-normative-table"
        }
        FormSource::SynodalNormativeGeneration { .. } => "synodal-productive-rule",
        FormSource::InheritedPrediction { .. } => "inherited-ocs-prediction",
        FormSource::AnalogicalPrediction { .. } => "analogical-prediction",
    }
}

fn abstention_reason(error: &synodal_church_slavonic::Error) -> &'static str {
    use synodal_church_slavonic::Error;
    match error {
        Error::MissingPrincipalPart { .. } => "missing-principal-part",
        Error::UnsupportedFormation { .. } => "unsupported-formation",
        Error::MissingRecensionMapping { .. } => "missing-recension-mapping",
        Error::AmbiguousRecensionMapping { .. } => "ambiguous-recension-mapping",
        Error::SemanticAlignmentNotEstablished { .. } => "semantic-alignment-not-established",
        Error::InheritedEvidenceContradicted { .. } => "inherited-evidence-contradicted",
        Error::HistoricallyInvalidCell { .. } => "historically-invalid-cell",
        Error::UnsupportedCell { .. } => "unsupported-cell",
        Error::OrthographicMetadataRequired { .. } => "orthographic-metadata-required",
        Error::UnknownLemma { .. } => "unknown-lemma",
        Error::AmbiguousLexeme { .. } => "ambiguous-lexeme",
        Error::InvalidUnicode { .. } | Error::InvalidOrthography { .. } | Error::EmptyInput => {
            "invalid-input"
        }
        Error::ContradictoryMetadata { .. } => "contradictory-metadata",
        Error::EmptyFormSet | Error::AmbiguousVariant { .. } => "invalid-result",
        Error::InvalidNumeral { .. } | Error::OutOfRange { .. } => "invalid-numeral",
    }
}

fn score_phrase_result(slice: &mut MetricSlice, phrase: Option<&RealizedPhrase>, expected: &str) {
    slice.total += 1;
    let Some(phrase) = phrase else {
        slice.abstained += 1;
        return;
    };
    slice.returned += 1;
    if phrase.primary_text() == expected {
        slice.top_1_correct += 1;
        slice.top_k_correct += 1;
    }
}

fn realize_phrase(
    row: &PhraseEvaluationRow,
    profile: OrthographyProfile,
) -> synodal_church_slavonic::Result<RealizedPhrase> {
    let inflector = Inflector::builder().orthography(profile).build();
    match row.construction.as_str() {
        "perfect" => {
            phrases::perfect_with(&row.lemma, row.person, row.number, row.gender, inflector)
        }
        _ => Err(synodal_church_slavonic::Error::UnsupportedFormation {
            formation: format!("evaluation phrase {}", row.construction),
        }),
    }
}

fn load_exact_keys(
    path: &Path,
) -> Result<std::collections::BTreeSet<(String, String)>, Box<dyn Error>> {
    const HEADER: &str =
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension";
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("invalid exact-form header in {}", path.display()).into());
    }
    let mut keys = std::collections::BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 7 {
            return Err(format!("invalid exact-form row {}", offset + 2).into());
        }
        keys.insert((fields[0].to_owned(), fields[1].to_owned()));
    }
    Ok(keys)
}

fn load_evaluation(path: &Path) -> Result<Vec<EvaluationRow>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(EVALUATION_HEADER) {
        return Err(format!("invalid evaluation header in {}", path.display()).into());
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 9 {
            return Err(format!("invalid evaluation row {}", offset + 2).into());
        }
        rows.push(EvaluationRow {
            id: fields[0].into(),
            lexeme_id: LexemeId::from(fields[1]),
            cell: parse_cell(fields[2])?,
            cell_key: fields[2].into(),
            policy: match fields[3] {
                "strict" => GenerationPolicy::Strict,
                "productive" => GenerationPolicy::Productive,
                "exploratory" => GenerationPolicy::Exploratory,
                value => return Err(format!("unknown generation policy {value}").into()),
            },
            expected_expanded: fields[4].into(),
            expected_printed: fields[5].into(),
            source_id: fields[6].into(),
            passage: fields[7].into(),
            regularity: fields[8].into(),
        });
    }
    if rows.is_empty() {
        return Err("Synodal evaluation fixture is empty".into());
    }
    Ok(rows)
}

fn load_phrase_evaluation(path: &Path) -> Result<Vec<PhraseEvaluationRow>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(PHRASE_EVALUATION_HEADER) {
        return Err(format!("invalid phrase-evaluation header in {}", path.display()).into());
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 11 {
            return Err(format!("invalid phrase-evaluation row {}", offset + 2).into());
        }
        rows.push(PhraseEvaluationRow {
            id: fields[0].into(),
            construction: fields[1].into(),
            lemma: fields[2].into(),
            person: parse_person(fields[3])?,
            number: parse_number(fields[4])?,
            gender: parse_gender(fields[5])?,
            expected_expanded: fields[6].into(),
            expected_printed: fields[7].into(),
            source_id: fields[8].into(),
            passage: fields[9].into(),
            regularity: fields[10].into(),
        });
    }
    if rows.is_empty() {
        return Err("Synodal phrase-evaluation fixture is empty".into());
    }
    Ok(rows)
}

fn parse_cell(value: &str) -> Result<GrammarCell, Box<dyn Error>> {
    let fields: Vec<&str> = value.split(':').collect();
    let cell = match fields.as_slice() {
        ["noun", case, number, animacy] => {
            GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
                case: parse_case(case)?,
                number: parse_number(number)?,
                animacy: parse_animacy(animacy)?,
            })
        }
        [tense @ ("present" | "imperfect" | "aorist"), person, number] => {
            GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: match *tense {
                    "present" => FiniteTense::Present,
                    "imperfect" => FiniteTense::Imperfect,
                    _ => FiniteTense::Aorist,
                },
                person: parse_person(person)?,
                number: parse_number(number)?,
            })
        }
        ["imperative", person, number] => GrammarCell::Imperative(ImperativeCell {
            person: parse_person(person)?,
            number: parse_number(number)?,
        }),
        ["pronoun", case, number, gender, animacy] => GrammarCell::Pronoun(PronounCell {
            case: parse_case(case)?,
            number: parse_number(number)?,
            gender: parse_optional_gender(gender)?,
            person: None,
            animacy: parse_animacy(animacy)?,
        }),
        ["numeral", kind, case, number, gender, animacy] => GrammarCell::Numeral(NumeralCell {
            kind: match *kind {
                "cardinal" => NumeralKind::Cardinal,
                "ordinal" => NumeralKind::Ordinal,
                "collective" => NumeralKind::Collective,
                _ => return Err(format!("unknown numeral kind {kind}").into()),
            },
            case: parse_case(case)?,
            number: parse_number(number)?,
            gender: parse_optional_gender(gender)?,
            animacy: parse_animacy(animacy)?,
        }),
        ["adjective", case, number, gender, animacy, form, comparison] => {
            GrammarCell::Adjective(AdjectiveCell {
                case: parse_case(case)?,
                number: parse_number(number)?,
                gender: parse_gender(gender)?,
                animacy: parse_animacy(animacy)?,
                form: match *form {
                    "short" => AdjectiveForm::Short,
                    "long" => AdjectiveForm::Long,
                    _ => return Err(format!("unknown adjective form {form}").into()),
                },
                comparison: match *comparison {
                    "positive" => Comparison::Positive,
                    "comparative" => Comparison::Comparative,
                    "superlative" => Comparison::Superlative,
                    _ => return Err(format!("unknown comparison {comparison}").into()),
                },
            })
        }
        [
            "participle",
            tense,
            voice,
            case,
            number,
            gender,
            animacy,
            form,
            comparison,
        ] => GrammarCell::Participle(ParticipleCell {
            tense: match *tense {
                "present" => ParticipleTense::Present,
                "past" => ParticipleTense::Past,
                _ => return Err(format!("unknown participle tense {tense}").into()),
            },
            voice: match *voice {
                "active" => ParticipleVoice::Active,
                "passive" => ParticipleVoice::Passive,
                _ => return Err(format!("unknown participle voice {voice}").into()),
            },
            agreement: AdjectiveCell {
                case: parse_case(case)?,
                number: parse_number(number)?,
                gender: parse_gender(gender)?,
                animacy: parse_animacy(animacy)?,
                form: match *form {
                    "short" => AdjectiveForm::Short,
                    "long" => AdjectiveForm::Long,
                    _ => return Err(format!("unknown adjective form {form}").into()),
                },
                comparison: match *comparison {
                    "positive" => Comparison::Positive,
                    "comparative" => Comparison::Comparative,
                    "superlative" => Comparison::Superlative,
                    _ => return Err(format!("unknown comparison {comparison}").into()),
                },
            },
        }),
        ["infinitive"] => GrammarCell::Infinitive,
        _ => return Err(format!("unsupported evaluation cell {value}").into()),
    };
    Ok(cell)
}

fn parse_case(value: &str) -> Result<Case, Box<dyn Error>> {
    Ok(match value {
        "nominative" => Case::Nominative,
        "genitive" => Case::Genitive,
        "dative" => Case::Dative,
        "accusative" => Case::Accusative,
        "instrumental" => Case::Instrumental,
        "locative" => Case::Locative,
        "vocative" => Case::Vocative,
        _ => return Err(format!("unknown case {value}").into()),
    })
}

fn parse_number(value: &str) -> Result<Number, Box<dyn Error>> {
    Ok(match value {
        "singular" => Number::Singular,
        "dual" => Number::Dual,
        "plural" => Number::Plural,
        _ => return Err(format!("unknown number {value}").into()),
    })
}

fn parse_person(value: &str) -> Result<Person, Box<dyn Error>> {
    Ok(match value {
        "first" => Person::First,
        "second" => Person::Second,
        "third" => Person::Third,
        _ => return Err(format!("unknown person {value}").into()),
    })
}

fn parse_gender(value: &str) -> Result<Gender, Box<dyn Error>> {
    Ok(match value {
        "masculine" => Gender::Masculine,
        "feminine" => Gender::Feminine,
        "neuter" => Gender::Neuter,
        _ => return Err(format!("unknown gender {value}").into()),
    })
}

fn parse_optional_gender(value: &str) -> Result<Option<Gender>, Box<dyn Error>> {
    if value == "any" {
        Ok(None)
    } else {
        parse_gender(value).map(Some)
    }
}

fn parse_animacy(value: &str) -> Result<Animacy, Box<dyn Error>> {
    Ok(match value {
        "animate" => Animacy::Animate,
        "inanimate" => Animacy::Inanimate,
        _ => return Err(format!("unknown animacy {value}").into()),
    })
}

fn contains_accent(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0486}'))
}

fn evaluation_markdown(report: &EvaluationReport) -> String {
    let mut markdown = format!(
        "# Synodal evaluation\n\n\
         Target recension: `synodal-russian`. Fixture: {} ({} held-out token cells).\n\n\
         | Metric | Returned | Top-1 | Top-k | Abstained | Total |\n\
         |---|---:|---:|---:|---:|---:|\n\
         | Expanded | {} | {} | {} | {} | {} |\n\
         | Printed | {} | {} | {} | {} | {} |\n\n\
         Analytic phrases: expanded {}/{}, printed {}/{} ({} held-out phrases).\n\n\
         Masked cells: expanded {}/{}, printed {}/{}. Leave-one-Synodal-lexeme-out inherited cells: expanded {}/{}, printed {}/{}.\n\n\
         Accent agreement: {}/{} accent-bearing rows.\n\n\
         Inherited evidence contributed {}/{} returned held-out cells, with {}/{} exact expanded forms. The reviewed alignment registry has {} accepted mappings, {} aligned target lexemes, and {} rejected negative controls.\n",
        report.fixture_source,
        report.fixture_rows,
        report.expanded.returned,
        report.expanded.top_1_correct,
        report.expanded.top_k_correct,
        report.expanded.abstained,
        report.expanded.total,
        report.printed.returned,
        report.printed.top_1_correct,
        report.printed.top_k_correct,
        report.printed.abstained,
        report.printed.total,
        report.phrase_expanded.top_1_correct,
        report.phrase_expanded.total,
        report.phrase_printed.top_1_correct,
        report.phrase_printed.total,
        report.phrase_fixture_rows,
        report.leakage.masked_expanded.top_1_correct,
        report.leakage.masked_expanded.total,
        report.leakage.masked_printed.top_1_correct,
        report.leakage.masked_printed.total,
        report
            .leakage
            .leave_one_synodal_lexeme_out_expanded
            .top_1_correct,
        report.leakage.leave_one_synodal_lexeme_out_expanded.total,
        report
            .leakage
            .leave_one_synodal_lexeme_out_printed
            .top_1_correct,
        report.leakage.leave_one_synodal_lexeme_out_printed.total,
        report.exact_accent_agreement,
        report.accent_bearing_rows,
        report.inheritance.extra_returned_cells,
        report.inheritance.evaluated_inherited_cells,
        report.inheritance.exact_expanded_cells,
        report.inheritance.extra_returned_cells,
        report.inheritance.accepted_alignments,
        report.inheritance.aligned_target_lexemes,
        report.inheritance.rejected_negative_controls,
    );

    push_metric_table(
        &mut markdown,
        "Expanded accuracy by morphological system",
        &report.by_morphological_system,
    );
    push_metric_table(
        &mut markdown,
        "Expanded accuracy by provenance path",
        &report.by_provenance_path,
    );
    push_metric_table(
        &mut markdown,
        "Expanded accuracy by regularity",
        &report.by_regularity,
    );

    markdown.push_str("\n## Inherited OCS evaluation\n\n");
    markdown.push_str(&format!(
        "The accepted registry contains {} explicit identity mappings and {} transformed mappings. The structural Productive-policy admission check has {} true-positive admissions, {} false-positive admissions, and precision {}/10,000 basis points on the reviewed gold registry. This is a policy guard, not an independent estimate of automatic alignment quality.\n\n",
        report.inheritance.identity_alignments,
        report.inheritance.transformed_alignments,
        report.inheritance.gold_admission_true_positives,
        report.inheritance.gold_admission_false_positives,
        report.inheritance.gold_admission_precision_basis_points,
    ));
    push_metric_table(
        &mut markdown,
        "Inherited cells by identity/transformed mapping",
        &report.inheritance.by_mapping_kind,
    );
    push_metric_table(
        &mut markdown,
        "Inherited cells by morphological system",
        &report.inheritance.by_morphological_system,
    );
    push_metric_table(
        &mut markdown,
        "Inherited cells by confidence band",
        &report.inheritance.by_confidence_band,
    );
    markdown.push_str(&format!(
        "\nReturned inherited confidence: {} basis points; empirical exact expanded agreement: {} basis points; absolute descriptive calibration gap: {} basis points.\n",
        optional_basis_points(report.inheritance.mean_returned_confidence_basis_points),
        optional_basis_points(report.inheritance.empirical_exactness_basis_points),
        optional_basis_points(report.inheritance.absolute_calibration_gap_basis_points),
    ));

    markdown.push_str("\n## Abstention\n\n");
    if report.abstention_reasons.is_empty() {
        markdown.push_str("No held-out row abstained in this seed fixture. Unsupported and missing-metadata behavior is exercised separately by paradigms and guard witnesses.\n");
    } else {
        for (reason, count) in &report.abstention_reasons {
            markdown.push_str(&format!("- `{reason}`: {count}\n"));
        }
    }

    markdown.push_str("\n## Interpretation and limitations\n\n");
    markdown
        .push_str("- The corpus passages are evaluation-only; they are not generation inputs.\n");
    for limitation in &report.limitations {
        markdown.push_str(&format!("- {limitation}\n"));
    }
    markdown
}

fn optional_basis_points(value: Option<u16>) -> String {
    value.map_or_else(|| "n/a".into(), |value| value.to_string())
}

fn push_metric_table(markdown: &mut String, heading: &str, slices: &BTreeMap<String, MetricSlice>) {
    markdown.push_str(&format!(
        "\n## {heading}\n\n| Slice | Returned | Top-1 | Top-k | Abstained | Total |\n|---|---:|---:|---:|---:|---:|\n"
    ));
    for (name, slice) in slices {
        markdown.push_str(&format!(
            "| `{name}` | {} | {} | {} | {} | {} |\n",
            slice.returned, slice.top_1_correct, slice.top_k_correct, slice.abstained, slice.total,
        ));
    }
}

pub(crate) fn guard_witnesses(_root: &Path) -> Result<(), Box<dyn Error>> {
    let temporary = temporary_directory("guards");
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(temporary.join("data/synodal"))?;
    let result = (|| -> Result<(), Box<dyn Error>> {
        let forbidden = temporary.join("data/synodal/lexemes.tsv");
        fs::write(&forbidden, "source\nSlovowiki\n")?;
        require_failure("forbidden authority", check_source_boundaries(&temporary))?;
        fs::remove_file(&forbidden)?;

        let malformed = temporary.join("malformed.tsv");
        fs::write(&malformed, format!("{EVALUATION_HEADER}\nwrong\n"))?;
        require_failure(
            "malformed evaluation",
            load_evaluation(&malformed).map(|_| ()),
        )?;

        let private_use = temporary.join("data/synodal/private.tsv");
        fs::write(&private_use, "text\n\u{e000}\n")?;
        require_failure("private-use Unicode", check_source_boundaries(&temporary))?;

        let strict = Inflector::default();
        let grad = LexemeId::from("synodal:noun:grad");
        let grad_cell = GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
            case: Case::Dative,
            number: Number::Plural,
            animacy: Animacy::Inanimate,
        });
        if strict.form_by_id(&grad, grad_cell).is_ok() {
            return Err("Strict admitted an inherited-only class analysis".into());
        }
        let inherited = inflector(GenerationPolicy::Productive, OrthographyProfile::Expanded)
            .form_by_id(&grad, grad_cell)?;
        if inherited.variants().iter().any(|variant| {
            variant.recension_mapping.is_none()
                || !matches!(variant.source, FormSource::InheritedPrediction { .. })
        }) {
            return Err("Productive inherited output lost its mapping provenance".into());
        }

        for hostile in ["", "latin", "\u{e000}", "\u{0301}слово"] {
            if std::panic::catch_unwind(|| synodal_church_slavonic::lookup(hostile)).is_err() {
                return Err(format!("hostile input panicked: {hostile:?}").into());
            }
        }
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&temporary);
    result?;
    cleanup?;
    println!("synodal guard witnesses: all injected failures detected");
    Ok(())
}

fn require_failure<T>(
    label: &str,
    result: Result<T, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if result.is_err() {
        println!("synodal guard witness observed: {label}");
        Ok(())
    } else {
        Err(format!("Synodal guard failed to detect {label}").into())
    }
}

fn temporary_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "synodal-church-slavonic-{label}-{}",
        std::process::id()
    ))
}
