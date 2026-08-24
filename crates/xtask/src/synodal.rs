use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    thread,
};
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, CompoundFutureAuxiliary,
    ConditionalFormation, Gender, GenerationPolicy, GrammarCell, Inflector, LexemeId, Number,
    OptativeFiniteSystem, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice,
    PassiveFormation, PerfectFormation, PeriphrasticTenseFormation, Person, PhraseOrder,
    RealizedPhrase, abbreviation, phrases,
};
use synodal_church_slavonic_core::FormSource;

const EVALUATION_HEADER: &str = "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";
const PHRASE_EVALUATION_HEADER: &str = "id\tconstruction\tlemma\tperson\tnumber\tgender\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";
const ABBREVIATION_EVALUATION_HEADER: &str = "id\tlexeme_id\tsense_id\tcell\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";
const EXACT_CELL_CORRECTION_HEADER: &str = "correction_id\tlexeme_id\tcell\tobsolete_expanded\thistorical_review_id\tobsolete_evaluation_id\treplacement_rule_id\tdecision\treview_note";

#[derive(Debug, Deserialize)]
struct AuthoritativeSourceManifest {
    #[serde(default)]
    source: Vec<AuthoritativeSource>,
}

#[derive(Debug, Deserialize)]
struct AuthoritativeSource {
    id: String,
    source_recension: String,
    content_kind: String,
    format: String,
    license: String,
    redistribution: String,
    authority_roles: Vec<String>,
    upstream_lineage: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SourceMirrorManifest {
    #[serde(default)]
    synodal_source: Vec<SourceMirror>,
}

#[derive(Debug, Deserialize)]
struct SourceMirror {
    id: String,
    source_recension: String,
    content_kind: String,
    format: String,
    license: String,
    redistribution: String,
    authority_roles: Vec<String>,
    upstream_lineage: Vec<String>,
    normalization: String,
}

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

#[derive(Clone, Debug)]
struct AbbreviationEvaluationRow {
    id: String,
    lexeme_id: LexemeId,
    sense_id: String,
    cell: GrammarCell,
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
    retracted_fixture_rows: Vec<String>,
    phrase_fixture_rows: usize,
    abbreviation_fixture_rows: usize,
    expanded: MetricSlice,
    printed: MetricSlice,
    expanded_disagreements: Vec<EvaluationDisagreement>,
    printed_disagreements: Vec<EvaluationDisagreement>,
    exact_registry_expanded_round_trip: MetricSlice,
    exact_registry_printed_round_trip: MetricSlice,
    accent_bearing_rows: usize,
    exact_accent_agreement: usize,
    by_regularity: BTreeMap<String, MetricSlice>,
    by_policy: BTreeMap<String, MetricSlice>,
    by_attestation_status: BTreeMap<String, MetricSlice>,
    by_morphological_system: BTreeMap<String, MetricSlice>,
    by_provenance_path: BTreeMap<String, MetricSlice>,
    by_source: BTreeMap<String, MetricSlice>,
    abstention_reasons: BTreeMap<String, usize>,
    phrase_expanded: MetricSlice,
    phrase_printed: MetricSlice,
    abbreviation_expansion: MetricSlice,
    leakage: LeakageMetrics,
    inheritance: InheritanceMetrics,
    limitations: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
struct EvaluationDisagreement {
    id: String,
    cell: String,
    expected: String,
    returned_top_1: Option<String>,
    returned_top_k: Vec<String>,
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

#[derive(Debug, Serialize)]
struct BootstrapReport {
    schema_version: u8,
    target_recension: &'static str,
    source_filter: Option<String>,
    source_verification: &'static str,
    candidate_pipeline: synodal_church_slavonic_extractor::pipeline::PipelineReport,
    reviewed_overlay: &'static str,
    registry_generation: &'static str,
    evaluation: &'static str,
    freshness_and_boundaries: &'static str,
}

#[derive(Debug, Serialize)]
struct FixtureBootstrapReport {
    schema_version: u8,
    target_recension: &'static str,
    fixture_source: &'static str,
    fixture_candidates_sha256: String,
    fixture_runs_byte_identical: bool,
    morphology_registry_sha256: String,
    dictionary_registry_sha256: String,
    generated_runs_byte_identical: bool,
    committed_outputs_current: bool,
    source_locks_unchanged: bool,
    evaluation_rows: usize,
    evaluation_top_1_correct: usize,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
struct VerseDisagreementReport {
    schema_version: u8,
    target_recension: String,
    comparison_basis: String,
    passages_by_source: BTreeMap<String, usize>,
    pairwise: Vec<VersePairComparison>,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
struct VersePairComparison {
    left_source: String,
    right_source: String,
    overlapping_passages: usize,
    exact_text_agreements: usize,
    text_disagreements: usize,
    disagreement_samples: Vec<VerseDisagreementSample>,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
struct VerseDisagreementSample {
    passage: String,
    left_sha256: String,
    right_sha256: String,
}

pub(crate) fn bootstrap(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut cache = root.join("references/downloads");
    let mut offline = false;
    let mut source = None::<String>;
    let mut skip_fetch = false;
    let mut keep_intermediate = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--cache" => cache = PathBuf::from(args.next().ok_or("--cache requires a path")?),
            "--offline" => offline = true,
            "--source" => source = Some(args.next().ok_or("--source requires an ID")?),
            "--skip-fetch" => skip_fetch = true,
            "--keep-intermediate" => keep_intermediate = true,
            value => return Err(format!("unknown synodal-bootstrap argument {value:?}").into()),
        }
    }

    if !offline && !skip_fetch {
        let mut source_arguments = vec!["fetch".into(), "--cache".into()];
        source_arguments.push(cache.display().to_string());
        if let Some(source) = &source {
            source_arguments.extend(["--source".into(), source.clone()]);
        }
        crate::sources::run(&mut source_arguments.into_iter(), root)?;
    }
    let mut verify_arguments = vec![
        "verify".into(),
        "--offline".into(),
        "--cache".into(),
        cache.display().to_string(),
    ];
    if let Some(source) = &source {
        verify_arguments.extend(["--source".into(), source.clone()]);
    }
    crate::sources::run(&mut verify_arguments.into_iter(), root)?;

    let candidate_pipeline = synodal_church_slavonic_extractor::pipeline::run_pipeline(
        &synodal_church_slavonic_extractor::pipeline::PipelineOptions {
            workspace_root: root.to_owned(),
            cache,
            intermediate: root.join("data/intermediate/synodal"),
            quarantine: root.join("data/quarantine/synodal"),
            source: source.clone(),
            failure_ceiling: 10_000,
            keep_work: keep_intermediate,
        },
    )?;
    if source.is_none() {
        synodal_church_slavonic_extractor::validate_candidate_links(
            &root.join("data/synodal"),
            &root.join("data/intermediate/synodal"),
        )?;
        write_verse_disagreement_report(root)?;
    }

    // The committed data/synodal TSVs are the reviewed overlay. Candidate
    // extraction cannot write there; generation reads it only after adapters
    // have completed successfully.
    regenerate(root)?;
    if source.is_none() {
        write_bootstrap_report(root, candidate_pipeline)?;
    }
    check(root)?;
    println!("Synodal bootstrap completed with verified, review-separated data");
    Ok(())
}

fn bootstrap_report(
    candidate_pipeline: synodal_church_slavonic_extractor::pipeline::PipelineReport,
) -> BootstrapReport {
    BootstrapReport {
        schema_version: 1,
        target_recension: "synodal-russian",
        source_filter: None,
        source_verification: "all selected artifacts matched immutable SHA-256 locks",
        candidate_pipeline,
        reviewed_overlay: "committed data/synodal review boundary applied read-only",
        registry_generation: "deterministic runtime registries regenerated",
        evaluation: "passage and lemma partition evaluation completed",
        freshness_and_boundaries: "freshness, recension, package, and runtime-I/O guards passed",
    }
}

fn write_bootstrap_report(
    root: &Path,
    candidate_pipeline: synodal_church_slavonic_extractor::pipeline::PipelineReport,
) -> Result<(), Box<dyn Error>> {
    fs::write(
        root.join("reports/synodal-bootstrap.json"),
        serde_json::to_vec_pretty(&bootstrap_report(candidate_pipeline))?,
    )?;
    Ok(())
}

/// Runs the small, network-free acceptance bootstrap used by default CI.
/// The fixture cache and both output trees are reconstructed under a temporary
/// directory; the real cache is never read or modified.
pub(crate) fn fixture_bootstrap(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(argument) = args.next() {
        return Err(format!("unknown synodal-fixture-bootstrap argument {argument:?}").into());
    }
    let temporary = temporary_directory("fixture-bootstrap");
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(&temporary)?;
    let source_lock_before = fs::read(root.join("references/SOURCE_LOCK.tsv"))?;
    let checksums_before = fs::read(root.join("references/SHA256SUMS"))?;

    let result = (|| -> Result<FixtureBootstrapReport, Box<dyn Error>> {
        let html = b"<html><h3>\xc2\xa734</h3><span class=\"DSText\">\xd1\x80\xd0\xb0\xcc\x81\xd0\xb1-\xd1\x8a</span></html>";
        let run_adapter = |name: &str| {
            let (fixture_root, cache) = prepare_fixture_cache(&temporary, name, html)?;
            let intermediate = temporary.join(format!("{name}/intermediate"));
            let quarantine = temporary.join(format!("{name}/quarantine"));
            let report = synodal_church_slavonic_extractor::pipeline::run_pipeline(
                &synodal_church_slavonic_extractor::pipeline::PipelineOptions {
                    workspace_root: fixture_root,
                    cache,
                    intermediate: intermediate.clone(),
                    quarantine: quarantine.clone(),
                    source: Some("alypy-gamanovich-grammar-web-2023".into()),
                    failure_ceiling: 0,
                    keep_work: false,
                },
            )?;
            Ok::<_, Box<dyn Error>>((
                report,
                fs::read(intermediate.join("alypy-gamanovich-grammar-web-2023.jsonl"))?,
                fs::read(intermediate.join("adapter-reports.json"))?,
                fs::read(quarantine.join("alypy-gamanovich-grammar-web-2023.jsonl"))?,
            ))
        };
        let first_adapter = run_adapter("first")?;
        let second_adapter = run_adapter("second")?;
        if first_adapter != second_adapter {
            return Err("fixture adapter outputs differ across independent directories".into());
        }

        let generated_one = temporary.join("generated-one");
        let generated_two = temporary.join("generated-two");
        fs::create_dir_all(&generated_one)?;
        fs::create_dir_all(&generated_two)?;
        for destination in [&generated_one, &generated_two] {
            synodal_church_slavonic_extractor::generate_registry(
                &root.join("data/synodal"),
                &destination.join("morphology.rs"),
            )?;
            synodal_church_slavonic_extractor::generate_dictionary_registry(
                &root.join("data/synodal"),
                &destination.join("dictionary.rs"),
            )?;
        }
        let morphology_one = fs::read(generated_one.join("morphology.rs"))?;
        let dictionary_one = fs::read(generated_one.join("dictionary.rs"))?;
        let generated_runs_byte_identical = morphology_one
            == fs::read(generated_two.join("morphology.rs"))?
            && dictionary_one == fs::read(generated_two.join("dictionary.rs"))?;
        if !generated_runs_byte_identical {
            return Err("generated registries differ across independent directories".into());
        }
        let committed_outputs_current = morphology_one
            == fs::read(root.join("crates/synodal-church-slavonic/generated/registry.rs"))?
            && dictionary_one
                == fs::read(
                    root.join("crates/synodal-church-slavonic-dictionary/generated/registry.rs"),
                )?;
        if !committed_outputs_current {
            return Err("fixture bootstrap reconstructed stale committed registries".into());
        }
        let evaluation = evaluate(root)?;
        let source_locks_unchanged = source_lock_before
            == fs::read(root.join("references/SOURCE_LOCK.tsv"))?
            && checksums_before == fs::read(root.join("references/SHA256SUMS"))?;
        if !source_locks_unchanged {
            return Err("fixture bootstrap mutated committed source locks".into());
        }
        let source = first_adapter
            .0
            .source_reports
            .get("alypy-gamanovich-grammar-web-2023")
            .ok_or("fixture adapter omitted its source report")?;
        Ok(FixtureBootstrapReport {
            schema_version: 1,
            target_recension: "synodal-russian",
            fixture_source: "miniature locked Alypy HTML page",
            fixture_candidates_sha256: source.output_sha256.clone(),
            fixture_runs_byte_identical: true,
            morphology_registry_sha256: file_sha256(&generated_one.join("morphology.rs"))?,
            dictionary_registry_sha256: file_sha256(&generated_one.join("dictionary.rs"))?,
            generated_runs_byte_identical,
            committed_outputs_current,
            source_locks_unchanged,
            evaluation_rows: evaluation.expanded.total,
            evaluation_top_1_correct: evaluation.expanded.top_1_correct,
        })
    })();
    let cleanup = fs::remove_dir_all(&temporary);
    let report = result?;
    cleanup?;
    fs::write(
        root.join("reports/synodal-fixture-bootstrap.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "Synodal fixture bootstrap: {} byte-identical candidates; {}/{} evaluation cells",
        report.fixture_candidates_sha256, report.evaluation_top_1_correct, report.evaluation_rows
    );
    Ok(())
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
    crate::sources::check_lock(root)?;
    check_reviewed_candidate_links(root)?;
    check_generated(root)?;
    check_source_boundaries(root)?;
    check_runtime_boundaries(root)?;
    check_evaluation_report(root)?;
    check_extraction_report(root)?;
    check_verse_disagreement_report(root)?;
    check_bootstrap_report(root)?;
    check_fixture_bootstrap_report(root, &evaluate(root)?)?;
    check_package_metadata(root)?;
    println!("synodal checks: current");
    Ok(())
}

fn check_reviewed_candidate_links(root: &Path) -> Result<(), Box<dyn Error>> {
    let intermediate = root.join("data/intermediate/synodal");
    let adapter_report = intermediate.join("adapter-reports.json");
    if !adapter_report.is_file() {
        return Ok(());
    }
    let pipeline: synodal_church_slavonic_extractor::pipeline::PipelineReport =
        serde_json::from_slice(&fs::read(adapter_report)?)?;
    if pipeline.source_reports.len() == 13 {
        synodal_church_slavonic_extractor::validate_candidate_links(
            &root.join("data/synodal"),
            &intermediate,
        )?;
    }
    Ok(())
}

fn check_bootstrap_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report_path = root.join("reports/synodal-bootstrap.json");
    let committed = fs::read(&report_path)?;
    let value: serde_json::Value = serde_json::from_slice(&committed)?;
    if value
        .get("target_recension")
        .and_then(serde_json::Value::as_str)
        != Some("synodal-russian")
        || !value
            .get("source_filter")
            .is_some_and(serde_json::Value::is_null)
        || value.get("offline").is_some()
    {
        return Err(format!(
            "{} is not a deterministic full-source Synodal bootstrap report",
            report_path.display()
        )
        .into());
    }
    let reported_sources = value
        .pointer("/candidate_pipeline/source_reports")
        .and_then(serde_json::Value::as_object)
        .map_or(0, serde_json::Map::len);
    if reported_sources != 13 {
        return Err(format!(
            "{} records {reported_sources} adapters, expected 13",
            report_path.display()
        )
        .into());
    }

    let adapter_report = root.join("data/intermediate/synodal/adapter-reports.json");
    if adapter_report.is_file() {
        let pipeline: synodal_church_slavonic_extractor::pipeline::PipelineReport =
            serde_json::from_slice(&fs::read(&adapter_report)?)?;
        if pipeline.source_reports.len() == 13 {
            let expected = serde_json::to_vec_pretty(&bootstrap_report(pipeline))?;
            if committed != expected {
                return Err(format!(
                    "stale {}; rerun cargo xtask synodal-bootstrap",
                    report_path.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

fn verse_disagreement_report(root: &Path) -> Result<VerseDisagreementReport, Box<dyn Error>> {
    const SOURCES: [&str; 3] = [
        "ponomar-elizabeth-bible-2026-08-09",
        "wikisource-church-slavonic-bible-2026-08-09",
        "crosswire-csl-elizabeth-1.5.2",
    ];
    let intermediate = root.join("data/intermediate/synodal");
    let mut corpora = BTreeMap::<String, BTreeMap<String, String>>::new();
    for source in SOURCES {
        let path = intermediate.join(format!("{source}.jsonl"));
        let mut passages = BTreeMap::<String, String>::new();
        for (offset, line) in fs::read_to_string(&path)?.lines().enumerate() {
            let value: serde_json::Value = serde_json::from_str(line).map_err(|error| {
                format!(
                    "invalid candidate JSON in {}:{}: {error}",
                    path.display(),
                    offset + 1
                )
            })?;
            let passage = value
                .get("passage")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("candidate in {} has no passage", path.display()))?;
            let text = value
                .get("normalized_spelling")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    format!("candidate in {} has no normalized spelling", path.display())
                })?;
            let accumulated = passages.entry(passage.to_owned()).or_default();
            if !accumulated.is_empty() {
                accumulated.push(' ');
            }
            accumulated.push_str(text);
        }
        corpora.insert(source.into(), passages);
    }

    let passages_by_source = corpora
        .iter()
        .map(|(source, passages)| (source.clone(), passages.len()))
        .collect();
    let mut pairwise = Vec::new();
    for (left, right) in [
        (SOURCES[0], SOURCES[1]),
        (SOURCES[0], SOURCES[2]),
        (SOURCES[1], SOURCES[2]),
    ] {
        let left_passages = corpora.get(left).ok_or("missing left corpus")?;
        let right_passages = corpora.get(right).ok_or("missing right corpus")?;
        let mut overlapping_passages = 0;
        let mut exact_text_agreements = 0;
        let mut disagreement_samples = Vec::new();
        for (passage, left_text) in left_passages {
            let Some(right_text) = right_passages.get(passage) else {
                continue;
            };
            overlapping_passages += 1;
            if left_text == right_text {
                exact_text_agreements += 1;
            } else if disagreement_samples.len() < 25 {
                disagreement_samples.push(VerseDisagreementSample {
                    passage: passage.clone(),
                    left_sha256: sha256_bytes(left_text.as_bytes()),
                    right_sha256: sha256_bytes(right_text.as_bytes()),
                });
            }
        }
        pairwise.push(VersePairComparison {
            left_source: left.into(),
            right_source: right.into(),
            overlapping_passages,
            exact_text_agreements,
            text_disagreements: overlapping_passages.saturating_sub(exact_text_agreements),
            disagreement_samples,
        });
    }
    Ok(VerseDisagreementReport {
        schema_version: 1,
        target_recension: "synodal-russian".into(),
        comparison_basis: "exact normalized spelling after source-specific deterministic markup removal; CrossWire remains explicitly modernized".into(),
        passages_by_source,
        pairwise,
    })
}

fn write_verse_disagreement_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = verse_disagreement_report(root)?;
    fs::write(
        root.join("reports/synodal-verse-disagreement.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// The fixture-bootstrap report records the sha256 of the registries it
/// reconstructed and asserts they matched the committed outputs, so its hashes
/// must equal the committed registries' hashes. `check_generated` already proves
/// the committed registries are current, which makes this a cheap tripwire for a
/// registry change that was committed without rerunning
/// `cargo xtask synodal-fixture-bootstrap`.
fn check_fixture_bootstrap_report(
    root: &Path,
    evaluation: &EvaluationReport,
) -> Result<(), Box<dyn Error>> {
    let report_path = root.join("reports/synodal-fixture-bootstrap.json");
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&report_path)?)?;
    let field = |name: &str| -> Result<&serde_json::Value, Box<dyn Error>> {
        value
            .get(name)
            .ok_or_else(|| format!("{} lacks field {name:?}", report_path.display()).into())
    };
    let stale = |what: &str| -> Box<dyn Error> {
        format!(
            "{} is stale ({what}); run cargo xtask synodal-fixture-bootstrap",
            report_path.display()
        )
        .into()
    };
    for (name, committed) in [
        (
            "morphology_registry_sha256",
            "crates/synodal-church-slavonic/generated/registry.rs",
        ),
        (
            "dictionary_registry_sha256",
            "crates/synodal-church-slavonic-dictionary/generated/registry.rs",
        ),
    ] {
        if field(name)?.as_str() != Some(file_sha256(&root.join(committed))?.as_str()) {
            return Err(stale(name));
        }
    }
    if field("evaluation_rows")?.as_u64() != Some(evaluation.expanded.total as u64) {
        return Err(stale("evaluation_rows"));
    }
    if field("evaluation_top_1_correct")?.as_u64() != Some(evaluation.expanded.top_1_correct as u64)
    {
        return Err(stale("evaluation_top_1_correct"));
    }
    for name in [
        "fixture_runs_byte_identical",
        "generated_runs_byte_identical",
        "committed_outputs_current",
        "source_locks_unchanged",
    ] {
        if field(name)?.as_bool() != Some(true) {
            return Err(format!("{} reports {name} = false", report_path.display()).into());
        }
    }
    Ok(())
}

fn check_verse_disagreement_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let path = root.join("reports/synodal-verse-disagreement.json");
    let committed: VerseDisagreementReport = serde_json::from_slice(&fs::read(&path)?)?;
    if committed.target_recension != "synodal-russian"
        || committed.pairwise.len() != 3
        || committed
            .pairwise
            .iter()
            .any(|pair| pair.overlapping_passages == 0)
    {
        return Err("committed Synodal verse-disagreement report is incomplete".into());
    }
    let intermediate = root.join("data/intermediate/synodal");
    let complete_intermediate = [
        "ponomar-elizabeth-bible-2026-08-09",
        "wikisource-church-slavonic-bible-2026-08-09",
        "crosswire-csl-elizabeth-1.5.2",
    ]
    .iter()
    .all(|source| intermediate.join(format!("{source}.jsonl")).is_file());
    if complete_intermediate && committed != verse_disagreement_report(root)? {
        return Err(
            "committed Synodal verse-disagreement report is stale; run a full synodal-bootstrap"
                .into(),
        );
    }
    Ok(())
}

fn extraction_report(root: &Path) -> Result<ExtractionReport, Box<dyn Error>> {
    let mut normalized_tables = BTreeMap::new();
    for name in [
        "abbreviation_families.tsv",
        "abbreviation_inventory.tsv",
        "abbreviations.tsv",
        "abbreviation_evaluation.tsv",
        "accent_paradigms.tsv",
        "accents.tsv",
        "alignments.tsv",
        "conflicts.tsv",
        "evaluation.tsv",
        "engine_capabilities.tsv",
        "exact_forms.tsv",
        "examples.tsv",
        "irregular_overrides.tsv",
        "irregular_verb_inventory.tsv",
        "linguistic_evaluation.tsv",
        "lexical_reviews.tsv",
        "lexemes.tsv",
        "noun_restrictions.tsv",
        "past_classification_reviews.tsv",
        "positional_rules.tsv",
        "principal_parts.tsv",
        "reviewed_evidence.tsv",
        "phrase_evaluation.tsv",
        "semantic_alignments.tsv",
        "senses.tsv",
        "transformation_rules.tsv",
        "training_passages.tsv",
        "v10_exact_cell_corrections.tsv",
        "verb_defectiveness.tsv",
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
            "complete Alypy HTML inventory with section and DSText witnesses",
            "pinned D'yachenko DjVu OCR with page, line, box, confidence, and uncorrected status",
            "exact-revision Wikisource MediaWiki export with template lineage",
            "CrossWire CSlElizabeth SWORD export with module version and modernized-spelling label",
            "Polivanova OSD spreadsheet and TEI adapters with common-lineage labels",
            "UD PROIEL CoNLL-U and Syntacticus native CoNLL with shared-lineage labels",
            "CCMH text/XML historical-comparison adapter",
            "DIACU JSON recension-classification adapter",
            "streaming Kaikki OCS JSONL adapter with content IDs and no target surface rows",
            "Ponomar modern Church Slavonic structured frequency-list and dictionary adapter",
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

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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
            if lower.contains("slovowiki") {
                return Err(format!("forbidden linguistic authority in {}", path.display()).into());
            }
            if text.chars().any(|character| {
                matches!(character as u32, 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
            }) {
                return Err(format!("private-use Unicode in {}", path.display()).into());
            }
        }
    }

    check_source_manifests(root)?;
    check_partition_disjointness(root)?;
    Ok(())
}

fn check_source_manifests(root: &Path) -> Result<(), Box<dyn Error>> {
    let authoritative: AuthoritativeSourceManifest =
        toml::from_str(&fs::read_to_string(root.join("references/SOURCES.toml"))?)?;
    let mirror: SourceMirrorManifest =
        toml::from_str(&fs::read_to_string(root.join("data/SOURCES.toml"))?)?;
    if authoritative.source.len() < 10 || mirror.synodal_source.len() < 10 {
        return Err("Synodal source manifests unexpectedly contain fewer than ten records".into());
    }
    let mut authoritative_pairs = BTreeMap::new();
    for source in &authoritative.source {
        validate_source_record(
            &source.id,
            &source.source_recension,
            &source.content_kind,
            &source.format,
            &source.license,
            &source.redistribution,
            &source.authority_roles,
        )?;
        let _ = &source.upstream_lineage;
        if authoritative_pairs
            .insert(source.id.as_str(), source.source_recension.as_str())
            .is_some()
        {
            return Err(format!("duplicate authoritative source ID {}", source.id).into());
        }
    }
    let mut mirror_pairs = BTreeMap::new();
    for source in &mirror.synodal_source {
        validate_source_record(
            &source.id,
            &source.source_recension,
            &source.content_kind,
            &source.format,
            &source.license,
            &source.redistribution,
            &source.authority_roles,
        )?;
        if source.normalization.trim().is_empty() {
            return Err(format!("source mirror {} has empty normalization", source.id).into());
        }
        let _ = &source.upstream_lineage;
        if mirror_pairs
            .insert(source.id.as_str(), source.source_recension.as_str())
            .is_some()
        {
            return Err(format!("duplicate source-mirror ID {}", source.id).into());
        }
    }
    if authoritative_pairs != mirror_pairs {
        return Err(
            "references/SOURCES.toml and data/SOURCES.toml disagree on source IDs or recensions"
                .into(),
        );
    }
    if authoritative_pairs.len()
        != synodal_church_slavonic_extractor::APPROVED_SOURCE_RECENSIONS.len()
    {
        return Err("Synodal source manifests do not contain the complete approved set".into());
    }
    Ok(())
}

fn validate_source_record(
    id: &str,
    source_recension: &str,
    content_kind: &str,
    format: &str,
    license: &str,
    redistribution: &str,
    authority_roles: &[String],
) -> Result<(), Box<dyn Error>> {
    if [id, content_kind, format, license, redistribution]
        .into_iter()
        .any(str::is_empty)
        || authority_roles.is_empty()
    {
        return Err(format!("source manifest record {id:?} has empty required provenance").into());
    }
    if !matches!(
        source_recension,
        "old-church-slavonic" | "synodal-russian" | "mixed"
    ) {
        return Err(format!("source {id} has unknown recension {source_recension}").into());
    }
    if !synodal_church_slavonic_extractor::source_recension_is_approved(id, source_recension) {
        return Err(format!(
            "source {id} with recension {source_recension} is not explicitly approved"
        )
        .into());
    }
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

    // The v0.3 lexical-review overlay is also generation evidence. It is kept
    // separate from the older training manifest because many lexemes share a
    // passage, but its source-partition passages must obey the same disjoint
    // evaluation boundary.
    let lexical_reviews = fs::read_to_string(root.join("data/synodal/lexical_reviews.tsv"))?;
    let mut lines = lexical_reviews.lines();
    let expected_header = "review_id\tlexeme_id\tsense_id\tlemma\tpart_of_speech\tcell\texpanded\tprinted\tgloss\tdomains\tsemantic_source_id\tsemantic_candidate_id\tattestation_source_id\tattestation_candidate_id\tcitation\tdecision\ttarget_recension\treview_note";
    if lines.next() != Some(expected_header) {
        return Err("invalid Synodal lexical-review header".into());
    }
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 18 {
            return Err(format!("invalid Synodal lexical-review row {}", offset + 2).into());
        }
        if fields[15] == "reviewed" {
            source_passages.insert((fields[12].to_owned(), fields[14].to_owned()));
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
    let abbreviation_evaluation =
        load_abbreviation_evaluation(&root.join("data/synodal/abbreviation_evaluation.tsv"))?;
    for row in abbreviation_evaluation {
        if source_passages.contains(&(row.source_id.clone(), row.passage.clone())) {
            return Err(format!(
                "Synodal abbreviation passage {} {} occurs in both source and evaluation partitions",
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
            if path
                .components()
                .any(|component| component.as_os_str() == "bin")
            {
                continue;
            }
            let text = fs::read_to_string(&path)?;
            for forbidden in [
                "std::fs",
                "std::io",
                "std::net",
                "reqwest::",
                "ureq::",
                "serde_json::",
                "quick_xml::",
                "calamine::",
                "csv::",
                "rusqlite::",
            ] {
                if text.contains(forbidden) {
                    return Err(format!(
                        "runtime I/O boundary violated by {forbidden} in {}",
                        path.display()
                    )
                    .into());
                }
            }
        }
        let manifest = fs::read_to_string(root.join("crates").join(package).join("Cargo.toml"))?;
        for forbidden in [
            "reqwest",
            "ureq",
            "serde_json",
            "quick-xml",
            "calamine",
            "csv",
            "rusqlite",
        ] {
            if package == "synodal-church-slavonic-dictionary" && forbidden == "serde_json" {
                continue;
            }
            if manifest.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(forbidden)
                    .is_some_and(|suffix| suffix.trim_start().starts_with(['=', '.']))
            }) {
                return Err(format!(
                    "runtime data/network dependency boundary violated by {forbidden} in {package}"
                )
                .into());
            }
        }
        if package == "synodal-church-slavonic-dictionary"
            && (!manifest.contains("serde_json = { workspace = true, optional = true }")
                || !manifest.contains("cli = [\"dep:serde_json\"]")
                || !manifest.contains("required-features = [\"cli\"]"))
        {
            return Err(
                "synodal-dict serialization must remain optional and CLI-feature-gated".into(),
            );
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

/// The generated registries are embedded in the published crates, so their
/// source size is what eventually meets the crates.io per-crate limit.
///
/// Measured on this tree: `cargo package -p synodal-church-slavonic` produced
/// 3.4 MiB of files as a 379 KiB `.crate`, a ratio near 9:1, against a 10 MiB
/// compressed limit. That is roughly 90 MiB of source before publishing fails,
/// so at 2.8 MiB today there is ample headroom and moving the payload out of
/// generated Rust would be premature.
///
/// This budget exists so the ceiling is met as a failing check with a known
/// remedy rather than as a surprise at publish time. Tripping it is the signal
/// to move the bulk rows into a compact embedded representation, not to raise
/// the number.
const GENERATED_REGISTRY_BUDGET_BYTES: u64 = 40 * 1024 * 1024;

fn check_generated_registry_budget(root: &Path) -> Result<(), Box<dyn Error>> {
    for generated in [
        "crates/synodal-church-slavonic/generated/registry.rs",
        "crates/synodal-church-slavonic-dictionary/generated/registry.rs",
    ] {
        let path = root.join(generated);
        let bytes = fs::metadata(&path)?.len();
        if bytes > GENERATED_REGISTRY_BUDGET_BYTES {
            return Err(format!(
                "{generated} is {bytes} bytes, over the {GENERATED_REGISTRY_BUDGET_BYTES}-byte budget. \
                 The published crate embeds this file and crates.io caps a package at 10 MiB compressed, \
                 so the payload now needs a compact embedded representation rather than one Rust literal per row."
            )
            .into());
        }
    }
    Ok(())
}

fn check_package_metadata(root: &Path) -> Result<(), Box<dyn Error>> {
    check_generated_registry_budget(root)?;
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

        let output = Command::new("cargo")
            .args(["package", "-p", package, "--list", "--allow-dirty"])
            .current_dir(root)
            .output()?;
        if !output.status.success() {
            return Err(format!(
                "cargo package --list failed for {package}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        let packaged = String::from_utf8(output.stdout)?;
        for path in packaged.lines() {
            if path.starts_with("data/")
                || path.starts_with("references/")
                || path.starts_with("reports/")
                || path.ends_with(".jsonl")
                || path.ends_with(".tsv")
                || path.ends_with(".xml")
            {
                return Err(format!(
                    "{package} package leaks non-runtime or evaluation data: {path}"
                )
                .into());
            }
        }
        if package != "synodal-church-slavonic-core"
            && !packaged.lines().any(|path| path == "generated/registry.rs")
        {
            return Err(format!("{package} package omits generated/registry.rs").into());
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
    let mut rows = load_evaluation(&root.join("data/synodal/evaluation.tsv"))?;
    let retracted_evaluation_ids =
        load_retracted_evaluation_ids(&root.join("data/synodal/v10_exact_cell_corrections.tsv"))?;
    for id in &retracted_evaluation_ids {
        if !rows.iter().any(|row| &row.id == id) {
            return Err(format!("retracted Synodal evaluation row {id:?} does not exist").into());
        }
    }
    rows.retain(|row| !retracted_evaluation_ids.contains(&row.id));
    let phrase_rows = load_phrase_evaluation(&root.join("data/synodal/phrase_evaluation.tsv"))?;
    let abbreviation_rows =
        load_abbreviation_evaluation(&root.join("data/synodal/abbreviation_evaluation.tsv"))?;
    let exact_keys = load_exact_keys(&root.join("data/synodal/exact_forms.tsv"))?;
    let mut expanded = MetricSlice::default();
    let mut printed = MetricSlice::default();
    let mut by_regularity = BTreeMap::new();
    let mut by_policy = BTreeMap::new();
    let mut by_attestation_status = BTreeMap::from([
        ("attested".to_owned(), MetricSlice::default()),
        ("predicted".to_owned(), MetricSlice::default()),
        (
            "expected-form-not-returned".to_owned(),
            MetricSlice::default(),
        ),
    ]);
    let mut by_source = BTreeMap::new();
    let mut by_morphological_system = BTreeMap::new();
    let mut by_provenance_path = BTreeMap::new();
    let mut abstention_reasons = BTreeMap::new();
    let mut expanded_disagreements = Vec::new();
    let mut printed_disagreements = Vec::new();
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
        if expanded_result
            .as_ref()
            .ok()
            .and_then(|forms| forms.variants().first())
            .map(|form| form.expanded.as_str())
            != Some(row.expected_expanded.as_str())
        {
            expanded_disagreements.push(EvaluationDisagreement {
                id: row.id.clone(),
                cell: row.cell_key.clone(),
                expected: row.expected_expanded.clone(),
                returned_top_1: expanded_result
                    .as_ref()
                    .ok()
                    .and_then(|forms| forms.variants().first())
                    .map(|form| form.expanded.clone()),
                returned_top_k: expanded_result
                    .as_ref()
                    .ok()
                    .map_or_else(Vec::new, |forms| {
                        forms
                            .variants()
                            .iter()
                            .map(|form| form.expanded.clone())
                            .collect()
                    }),
            });
        }
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
        if printed_result
            .as_ref()
            .ok()
            .and_then(|forms| forms.variants().first())
            .map(|form| form.printed.as_str())
            != Some(row.expected_printed.as_str())
        {
            printed_disagreements.push(EvaluationDisagreement {
                id: row.id.clone(),
                cell: row.cell_key.clone(),
                expected: row.expected_printed.clone(),
                returned_top_1: printed_result
                    .as_ref()
                    .ok()
                    .and_then(|forms| forms.variants().first())
                    .map(|form| form.printed.clone()),
                returned_top_k: printed_result.as_ref().ok().map_or_else(Vec::new, |forms| {
                    forms
                        .variants()
                        .iter()
                        .map(|form| form.printed.clone())
                        .collect()
                }),
            });
        }

        let regularity = by_regularity
            .entry(row.regularity.clone())
            .or_insert_with(MetricSlice::default);
        score_result(
            regularity,
            expanded_result.as_ref().ok(),
            &row.expected_expanded,
        );
        // The row's policy remains the primary evaluation contract, but every
        // held-out cell is also scored under all three policies. This keeps the
        // policy slices comparable and makes Strict abstention visible instead
        // of omitting cells assigned to Productive or Exploratory.
        for (policy, label) in [
            (GenerationPolicy::Strict, "strict"),
            (GenerationPolicy::Productive, "productive"),
            (GenerationPolicy::Exploratory, "exploratory"),
        ] {
            let policy_result = inflector(policy, OrthographyProfile::Expanded)
                .form_by_id(&row.lexeme_id, row.cell);
            score_result(
                by_policy
                    .entry(label.into())
                    .or_insert_with(MetricSlice::default),
                policy_result.as_ref().ok(),
                &row.expected_expanded,
            );
        }
        let attestation_status = expanded_result
            .as_ref()
            .ok()
            .and_then(|forms| {
                forms
                    .variants()
                    .iter()
                    .find(|variant| variant.expanded == row.expected_expanded)
            })
            .map_or("expected-form-not-returned", |variant| {
                if variant.is_attested() {
                    "attested"
                } else {
                    "predicted"
                }
            });
        score_result(
            by_attestation_status
                .get_mut(attestation_status)
                .ok_or("unknown attestation-status metric slice")?,
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

    let mut abbreviation_expansion = MetricSlice::default();
    for row in &abbreviation_rows {
        abbreviation_expansion.total += 1;
        match abbreviation::expand(&row.expected_printed) {
            Ok(candidates) => {
                abbreviation_expansion.returned += 1;
                let matches = |candidate: &synodal_church_slavonic::abbreviation::Abbreviation| {
                    candidate.lexeme_id == row.lexeme_id
                        && candidate.sense_id == row.sense_id
                        && candidate.matches_cell(row.cell)
                        && candidate.expanded == row.expected_expanded
                        && candidate.printed == row.expected_printed
                };
                if candidates.first().is_some_and(matches) {
                    abbreviation_expansion.top_1_correct += 1;
                }
                if candidates.iter().any(matches) {
                    abbreviation_expansion.top_k_correct += 1;
                }
            }
            Err(_) => abbreviation_expansion.abstained += 1,
        }
        let reverse = abbreviation::contractions_by_id(&row.lexeme_id, &row.sense_id)?;
        if !reverse.iter().any(|candidate| {
            candidate.matches_cell(row.cell)
                && candidate.expanded == row.expected_expanded
                && candidate.printed == row.expected_printed
        }) {
            return Err(format!(
                "abbreviation evaluation row {} is not reversible through the typed registry lookup",
                row.id
            )
            .into());
        }
        if row.source_id.is_empty() || row.passage.is_empty() || row.regularity.is_empty() {
            return Err("abbreviation evaluation rows require source metadata".into());
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
    let (exact_registry_expanded_round_trip, exact_registry_printed_round_trip) =
        exact_registry_round_trip(root)?;

    Ok(EvaluationReport {
        schema_version: 5,
        target_recension: "synodal-russian",
        fixture_source: "pinned passage-held-out Ponomar Elizabeth Bible rows across Matthew, Acts, Daniel, Apocalypse, Amos, and Deuteronomy",
        fixture_rows: rows.len(),
        retracted_fixture_rows: retracted_evaluation_ids.into_iter().collect(),
        phrase_fixture_rows: phrase_rows.len(),
        abbreviation_fixture_rows: abbreviation_rows.len(),
        expanded,
        printed,
        expanded_disagreements,
        printed_disagreements,
        exact_registry_expanded_round_trip,
        exact_registry_printed_round_trip,
        accent_bearing_rows,
        exact_accent_agreement,
        by_regularity,
        by_policy,
        by_attestation_status,
        by_morphological_system,
        by_provenance_path,
        by_source,
        abstention_reasons,
        phrase_expanded,
        phrase_printed,
        abbreviation_expansion,
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
            "No legally cleared, machine-readable non-biblical Synodal liturgical corpus is currently pinned; catalog-only and unresolved-rights editions are intentionally excluded from held-out scoring.",
            "Productive liturgical rendering abstains when accent metadata is absent.",
            "One participle and one analytic perfect are covered by independent corpus witnesses; other analytic constructions remain typed unit fixtures until their lexical registries grow.",
            "Abbreviation, numeral, malformed-mark, and hostile-Unicode regressions are deterministic utility fixtures, not corpus-accuracy rows.",
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
        GrammarCell::LexicalForm => "lexical-form",
        GrammarCell::Indeclinable => "indeclinable",
        GrammarCell::Noun(_) => "noun",
        GrammarCell::Adjective(_) => "adjective",
        GrammarCell::FiniteVerb(cell) => cell.tense.code(),
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
        FormSource::SynodalIrregularOverride { .. } => "synodal-irregular-override",
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-REGISTRY-NORMATIVE-TABLE" =>
        {
            "synodal-normative-table"
        }
        FormSource::SynodalNormativeGeneration { .. } => "synodal-productive-rule",
        FormSource::CallerSpecifiedPrediction { .. } => "caller-specified-prediction",
        FormSource::InheritedPrediction { .. } => "inherited-ocs-prediction",
        FormSource::AnalogicalPrediction { .. } => "analogical-prediction",
    }
}

fn abstention_reason(error: &synodal_church_slavonic::Error) -> &'static str {
    use synodal_church_slavonic::Error;
    match error {
        Error::MissingPrincipalPart { .. } => "missing-principal-part",
        Error::MissingMetadata { .. } => "missing-metadata",
        Error::UnsupportedFormation { .. } => "unsupported-formation",
        Error::MissingRecensionMapping { .. } => "missing-recension-mapping",
        Error::AmbiguousRecensionMapping { .. } => "ambiguous-recension-mapping",
        Error::SemanticAlignmentNotEstablished { .. } => "semantic-alignment-not-established",
        Error::InheritedEvidenceContradicted { .. } => "inherited-evidence-contradicted",
        Error::HistoricallyInvalidCell { .. } => "historically-invalid-cell",
        Error::EvidenceIncompleteCell { .. } => "evidence-incomplete-cell",
        Error::UnsupportedCell { .. } => "unsupported-cell",
        Error::OrthographicMetadataRequired { .. } => "orthographic-metadata-required",
        Error::UnknownLemma { .. } => "unknown-lemma",
        Error::AmbiguousLexeme { .. } => "ambiguous-lexeme",
        Error::ProviderConflict { .. } => "provider-conflict",
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
        "compound-future" => {
            phrases::compound_future_with(&row.lemma, row.person, row.number, inflector)
        }
        "compound-future-byti" => phrases::compound_future_with_auxiliary(
            &row.lemma,
            CompoundFutureAuxiliary::Byti,
            row.person,
            row.number,
            PhraseOrder::AuxiliaryFirst,
            inflector,
        ),
        "compound-future-khoteti" => phrases::compound_future_with_auxiliary(
            &row.lemma,
            CompoundFutureAuxiliary::Khoteti,
            row.person,
            row.number,
            PhraseOrder::AuxiliaryFirst,
            inflector,
        ),
        "compound-future-nachati" => phrases::compound_future_with_auxiliary(
            &row.lemma,
            CompoundFutureAuxiliary::Nachati,
            row.person,
            row.number,
            PhraseOrder::AuxiliaryFirst,
            inflector,
        ),
        "perfect" => {
            phrases::perfect_with(&row.lemma, row.person, row.number, row.gender, inflector)
        }
        "perfect-elliptical" => phrases::perfect_with_formation(
            &row.lemma,
            row.person,
            row.number,
            row.gender,
            PerfectFormation::OmittedThirdSingularCopula,
            PhraseOrder::PredicateFirst,
            inflector,
        ),
        "pluperfect" => {
            phrases::pluperfect_with(&row.lemma, row.person, row.number, row.gender, inflector)
        }
        "conditional" => {
            phrases::conditional_with(&row.lemma, row.person, row.number, row.gender, inflector)
        }
        "conditional-invariant" => phrases::conditional_with_formation(
            &row.lemma,
            row.person,
            row.number,
            row.gender,
            ConditionalFormation::InvariantBy,
            PhraseOrder::PredicateFirst,
            inflector,
        ),
        "future-anterior" => phrases::future_anterior(
            &row.lemma,
            row.person,
            row.number,
            row.gender,
            PhraseOrder::AuxiliaryFirst,
            inflector,
        ),
        "optative-present" => phrases::optative(
            &row.lemma,
            OptativeFiniteSystem::Present,
            row.person,
            row.number,
            inflector,
        ),
        "periphrastic-future" => phrases::periphrastic_tense(
            &row.lemma,
            evaluation_participle(
                row,
                ParticipleTense::Present,
                ParticipleVoice::Active,
                Animacy::Animate,
            ),
            PeriphrasticTenseFormation::Future,
            row.person,
            row.number,
            PhraseOrder::PredicateFirst,
            inflector,
        ),
        "analytic-passive-infinitive" => phrases::analytic_passive_formation(
            &row.lemma,
            evaluation_participle(
                row,
                ParticipleTense::Past,
                ParticipleVoice::Passive,
                Animacy::Inanimate,
            ),
            PassiveFormation::PastParticipleInfinitive,
            row.person,
            row.number,
            PhraseOrder::PredicateFirst,
            inflector,
        ),
        "analytic-passive" => phrases::analytic_passive_with(
            &row.lemma,
            synodal_church_slavonic::ParticipleCell {
                tense: synodal_church_slavonic::ParticipleTense::Past,
                voice: synodal_church_slavonic::ParticipleVoice::Passive,
                agreement: synodal_church_slavonic::AdjectiveCell {
                    case: synodal_church_slavonic::Case::Nominative,
                    number: row.number,
                    gender: row.gender,
                    animacy: synodal_church_slavonic::Animacy::Inanimate,
                    form: synodal_church_slavonic::AdjectiveForm::Short,
                    comparison: synodal_church_slavonic::Comparison::Positive,
                },
            },
            row.person,
            row.number,
            inflector,
        ),
        _ => Err(synodal_church_slavonic::Error::UnsupportedFormation {
            formation: format!("evaluation phrase {}", row.construction),
        }),
    }
}

fn evaluation_participle(
    row: &PhraseEvaluationRow,
    tense: ParticipleTense,
    voice: ParticipleVoice,
    animacy: Animacy,
) -> ParticipleCell {
    ParticipleCell {
        tense,
        voice,
        agreement: AdjectiveCell {
            case: Case::Nominative,
            number: row.number,
            gender: row.gender,
            animacy,
            form: AdjectiveForm::Short,
            comparison: Comparison::Positive,
        },
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

fn load_retracted_evaluation_ids(path: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(EXACT_CELL_CORRECTION_HEADER) {
        return Err(format!("invalid exact-cell correction header in {}", path.display()).into());
    }
    let mut ids = BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 9
            || fields[5].is_empty()
            || fields[7] != "retracted"
            || fields[8].trim().is_empty()
        {
            return Err(format!("invalid exact-cell correction row {}", offset + 2).into());
        }
        if !ids.insert(fields[5].to_owned()) {
            return Err(format!("duplicate retracted evaluation ID {:?}", fields[5]).into());
        }
    }
    Ok(ids)
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

fn exact_registry_round_trip(root: &Path) -> Result<(MetricSlice, MetricSlice), Box<dyn Error>> {
    const HEADER: &str =
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension";
    let path = root.join("data/synodal/exact_forms.tsv");
    let text = fs::read_to_string(&path)?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("invalid exact-form header in {}", path.display()).into());
    }
    let expanded_inflector = inflector(GenerationPolicy::Strict, OrthographyProfile::Expanded);
    let printed_inflector = inflector(
        GenerationPolicy::Strict,
        OrthographyProfile::SynodalLiturgical,
    );
    let mut expanded = MetricSlice::default();
    let mut printed = MetricSlice::default();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 7 || fields[6] != "synodal-russian" {
            return Err(format!("invalid exact-form row {}", offset + 2).into());
        }
        let id = LexemeId::from(fields[0]);
        let cell = parse_cell(fields[1])?;
        let expanded_result = expanded_inflector.form_by_id(&id, cell);
        let printed_result = printed_inflector.form_by_id(&id, cell);
        score_result(&mut expanded, expanded_result.as_ref().ok(), fields[2]);
        score_result(&mut printed, printed_result.as_ref().ok(), fields[3]);
    }
    if expanded.total == 0 {
        return Err("exact-form registry is empty".into());
    }
    Ok((expanded, printed))
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
            person: parse_closed_code("person", fields[3], Person::from_code)?,
            number: parse_closed_code("number", fields[4], Number::from_code)?,
            gender: parse_closed_code("gender", fields[5], Gender::from_code)?,
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

fn load_abbreviation_evaluation(
    path: &Path,
) -> Result<Vec<AbbreviationEvaluationRow>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(ABBREVIATION_EVALUATION_HEADER) {
        return Err(format!(
            "invalid abbreviation-evaluation header in {}",
            path.display()
        )
        .into());
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 9 {
            return Err(format!("invalid abbreviation-evaluation row {}", offset + 2).into());
        }
        rows.push(AbbreviationEvaluationRow {
            id: fields[0].into(),
            lexeme_id: LexemeId::from(fields[1]),
            sense_id: fields[2].into(),
            cell: parse_cell(fields[3])?,
            expected_expanded: fields[4].into(),
            expected_printed: fields[5].into(),
            source_id: fields[6].into(),
            passage: fields[7].into(),
            regularity: fields[8].into(),
        });
    }
    if rows.is_empty() {
        return Err("Synodal abbreviation-evaluation fixture is empty".into());
    }
    Ok(rows)
}

fn parse_cell(value: &str) -> Result<GrammarCell, Box<dyn Error>> {
    value.parse().map_err(Into::into)
}

fn parse_closed_code<T>(
    kind: &str,
    value: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, Box<dyn Error>> {
    parse(value).ok_or_else(|| format!("unknown {kind} {value}").into())
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
         The correction ledger excludes {} historically preserved but grammatically retracted evaluation rows from scoring.\n\n\
         | Metric | Returned | Top-1 | Top-k | Abstained | Total |\n\
         |---|---:|---:|---:|---:|---:|\n\
         | Expanded | {} | {} | {} | {} | {} |\n\
         | Printed | {} | {} | {} | {} | {} |\n\n\
         Analytic phrases: expanded {}/{}, printed {}/{} ({} held-out phrases).\n\n\
         Typed abbreviations: top-1 {}/{}, top-k {}/{} ({} held-out contractions; reverse lookup also required).\n\n\
         Exact registry round trips (top-k, including reviewed variants): expanded {}/{}, printed {}/{}.\n\n\
         Masked cells: expanded {}/{}, printed {}/{}. Leave-one-Synodal-lexeme-out inherited cells: expanded {}/{}, printed {}/{}.\n\n\
         Accent agreement: {}/{} accent-bearing rows.\n\n\
         Inherited evidence contributed {}/{} returned held-out cells, with {}/{} exact expanded forms. The reviewed alignment registry has {} accepted mappings, {} aligned target lexemes, and {} rejected negative controls.\n",
        report.fixture_source,
        report.fixture_rows,
        report.retracted_fixture_rows.len(),
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
        report.abbreviation_expansion.top_1_correct,
        report.abbreviation_expansion.total,
        report.abbreviation_expansion.top_k_correct,
        report.abbreviation_expansion.total,
        report.abbreviation_fixture_rows,
        report.exact_registry_expanded_round_trip.top_k_correct,
        report.exact_registry_expanded_round_trip.total,
        report.exact_registry_printed_round_trip.top_k_correct,
        report.exact_registry_printed_round_trip.total,
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
        "Expanded accuracy by generation policy",
        &report.by_policy,
    );
    push_metric_table(
        &mut markdown,
        "Expanded accuracy by attestation status",
        &report.by_attestation_status,
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

    markdown.push_str("\n## Top-1 disagreements\n\n");
    if report.expanded_disagreements.is_empty() && report.printed_disagreements.is_empty() {
        markdown.push_str("No top-1 disagreements.\n");
    } else {
        for disagreement in &report.expanded_disagreements {
            markdown.push_str(&format!(
                "- Expanded `{}` (`{}`): expected `{}`, top-1 `{}`.\n",
                disagreement.id,
                disagreement.cell,
                disagreement.expected,
                disagreement
                    .returned_top_1
                    .as_deref()
                    .unwrap_or("abstained")
            ));
        }
        for disagreement in &report.printed_disagreements {
            markdown.push_str(&format!(
                "- Printed `{}` (`{}`): expected `{}`, top-1 `{}`.\n",
                disagreement.id,
                disagreement.cell,
                disagreement.expected,
                disagreement
                    .returned_top_1
                    .as_deref()
                    .unwrap_or("abstained")
            ));
        }
    }

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
        markdown.push_str("No held-out row abstained in this reviewed fixture. Unsupported and missing-metadata behavior is exercised separately by paradigms and guard witnesses.\n");
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

pub(crate) fn guard_witnesses(root: &Path) -> Result<(), Box<dyn Error>> {
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
        fs::remove_file(&private_use)?;

        fs::create_dir_all(temporary.join("references"))?;
        fs::copy(
            root.join("references/SOURCES.toml"),
            temporary.join("references/SOURCES.toml"),
        )?;
        let mirror_path = temporary.join("data/SOURCES.toml");
        let source_mirror = fs::read_to_string(root.join("data/SOURCES.toml"))?;
        fs::write(
            &mirror_path,
            source_mirror.replacen(
                "source_recension = \"mixed\"",
                "source_recension = \"synodal-russian\"",
                1,
            ),
        )?;
        require_failure(
            "source provenance mirror drift",
            check_source_manifests(&temporary),
        )?;

        let review = temporary.join("data/synodal/reviewed_evidence.tsv");
        fs::write(
            &review,
            "evidence_id\tcandidate_id\tsource_id\tcitation\tdecision\ttarget_recension\treview_note\n\
             guard-orphan\tsynodal:candidate:missing\tguard-source\tfixture\treviewed\tsynodal-russian\tinjected orphan\n",
        )?;
        let intermediate = temporary.join("data/intermediate/synodal");
        fs::create_dir_all(&intermediate)?;
        fs::write(
            intermediate.join("guard.jsonl"),
            "{\"candidate_id\":\"synodal:candidate:different\"}\n",
        )?;
        require_failure(
            "orphaned reviewed overlay",
            synodal_church_slavonic_extractor::validate_candidate_links(
                &temporary.join("data/synodal"),
                &intermediate,
            )
            .map_err(|error| Box::new(error) as Box<dyn Error>),
        )?;

        for package in [
            "synodal-church-slavonic-core",
            "synodal-church-slavonic",
            "synodal-church-slavonic-dictionary",
        ] {
            let package_root = temporary.join("crates").join(package);
            fs::create_dir_all(package_root.join("src"))?;
            fs::write(
                package_root.join("Cargo.toml"),
                "[package]\nname = \"guard\"\n",
            )?;
            fs::write(package_root.join("src/lib.rs"), "")?;
        }
        fs::write(
            temporary.join("crates/synodal-church-slavonic-core/src/lib.rs"),
            "use std::fs;\n",
        )?;
        require_failure(
            "runtime filesystem boundary",
            check_runtime_boundaries(&temporary),
        )?;

        let strict = Inflector::default();
        let grad = LexemeId::from("synodal:noun:grad");
        let grad_cell = GrammarCell::Noun(synodal_church_slavonic::core::NounCell {
            case: synodal_church_slavonic::Case::Dative,
            number: Number::Dual,
            animacy: synodal_church_slavonic::Animacy::Inanimate,
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

        let fixture_report = root.join("reports/synodal-fixture-bootstrap.json");
        let mut stale_report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture_report)?)?;
        stale_report["morphology_registry_sha256"] = serde_json::Value::String("0".repeat(64));
        fs::create_dir_all(temporary.join("reports"))?;
        fs::write(
            temporary.join("reports/synodal-fixture-bootstrap.json"),
            serde_json::to_vec_pretty(&stale_report)?,
        )?;
        for path in [
            "crates/synodal-church-slavonic/generated/registry.rs",
            "crates/synodal-church-slavonic-dictionary/generated/registry.rs",
        ] {
            if let Some(parent) = temporary.join(path).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(root.join(path), temporary.join(path))?;
        }
        let current_evaluation = evaluate(root)?;
        require_failure(
            "stale fixture-bootstrap registry checksum",
            check_fixture_bootstrap_report(&temporary, &current_evaluation),
        )?;
        check_fixture_bootstrap_report(root, &current_evaluation)?;

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

fn prepare_fixture_cache(
    temporary: &Path,
    label: &str,
    bytes: &'static [u8],
) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let fixture_root = temporary.join(label).join("workspace");
    let references = fixture_root.join("references");
    let cache = temporary.join(label).join("empty-cache");
    fs::create_dir_all(&references)?;
    if cache.exists() {
        return Err(format!("fixture cache was not empty: {}", cache.display()).into());
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || -> Result<(), String> {
        let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
        let mut request = [0_u8; 8192];
        let count = stream
            .read(&mut request)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("fixture server received an empty request".into());
        }
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            bytes.len()
        )
        .map_err(|error| error.to_string())?;
        stream.write_all(bytes).map_err(|error| error.to_string())?;
        Ok(())
    });

    let sha = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let locked_path = "downloads/alypy-grammar/p034.htm";
    fs::write(
        references.join("SOURCE_LOCK.tsv"),
        format!(
            "source_id\tartifact_id\ttransport\turl\tpath\tsha256\tsize_bytes\tformat\tsignature\tcontent_types\n\
             alypy-gamanovich-grammar-web-2023\tfixture-page\tdirect\thttp://{address}/p034.htm\t{locked_path}\t{sha}\t{}\thtml\thtml\ttext/html\n",
            bytes.len()
        ),
    )?;
    fs::write(
        references.join("SHA256SUMS"),
        format!("{sha}  {locked_path}\n"),
    )?;
    fs::write(
        references.join("SOURCES.toml"),
        "[[source]]\nid = \"alypy-gamanovich-grammar-web-2023\"\nname = \"Fixture\"\n",
    )?;

    let mut fetch = vec![
        "fetch".into(),
        "--cache".into(),
        cache.display().to_string(),
    ]
    .into_iter();
    crate::sources::run(&mut fetch, &fixture_root)?;
    server
        .join()
        .map_err(|_| "fixture HTTP server panicked")?
        .map_err(|error| format!("fixture HTTP server failed: {error}"))?;
    let mut verify = vec![
        "verify".into(),
        "--offline".into(),
        "--cache".into(),
        cache.display().to_string(),
    ]
    .into_iter();
    crate::sources::run(&mut verify, &fixture_root)?;
    Ok((fixture_root, cache))
}
