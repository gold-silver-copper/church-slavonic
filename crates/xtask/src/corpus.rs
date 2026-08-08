use old_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, AoristFormation, Case, FiniteTense, FiniteVerbCell,
    FormSource, Gender, ImperfectFormation, Number, PartOfSpeech, ParticipleCell, ParticipleKind,
    Person, VerbClass,
};
use old_church_slavonic_core::verb::VerbLexeme;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MANIFEST_PATH: &str = "data/evaluation-sources.json";

#[derive(Debug, Deserialize)]
struct EvaluationManifest {
    schema_version: u32,
    thresholds: EvaluationThresholds,
    sources: Vec<EvaluationSource>,
}

#[derive(Debug, Deserialize)]
struct EvaluationThresholds {
    facade_attempt_coverage_basis_points: usize,
    facade_lookup_any_accuracy_basis_points: usize,
    native_attempt_coverage_basis_points: usize,
    native_lookup_any_accuracy_basis_points: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EvaluationSource {
    id: String,
    name: String,
    kind: String,
    version: String,
    commit: String,
    url: String,
    license: String,
    bundled: bool,
    files: Vec<EvaluationFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EvaluationFile {
    path: String,
    sha256: String,
    partition: String,
}

#[derive(Debug, Default, Clone, Serialize)]
struct MetricSlice {
    eligible: usize,
    attempted: usize,
    returned_forms: usize,
    raw_top1_correct: usize,
    raw_any_correct: usize,
    lookup_top1_correct: usize,
    lookup_any_correct: usize,
}

impl MetricSlice {
    fn observe_eligible(&mut self) {
        self.eligible += 1;
    }

    fn observe_attempt(
        &mut self,
        returned_forms: bool,
        raw_top1: bool,
        raw_any: bool,
        lookup_top1: bool,
        lookup_any: bool,
    ) {
        self.attempted += 1;
        self.returned_forms += usize::from(returned_forms);
        self.raw_top1_correct += usize::from(raw_top1);
        self.raw_any_correct += usize::from(raw_any);
        self.lookup_top1_correct += usize::from(lookup_top1);
        self.lookup_any_correct += usize::from(lookup_any);
    }
}

#[derive(Debug, Default, Serialize)]
struct CorpusCounts {
    all_tokens: usize,
    verb_or_aux_tokens: usize,
    compatible_bundles: usize,
    matched_lexemes: usize,
    sufficient_lexical_metadata: usize,
    generation_attempts: usize,
    tokens_with_returned_forms: usize,
    raw_top1_correct: usize,
    raw_any_correct: usize,
    lookup_top1_correct: usize,
    lookup_any_correct: usize,
    skipped_by_reason: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct CorpusReport {
    schema_version: u32,
    mapper_version: u32,
    sources: Vec<EvaluationSource>,
    normalization: NormalizationPolicy,
    partition_policy: PartitionPolicy,
    facade_attested_token_recall: EvaluationSection,
    core_generalization_oracle_metadata: Option<NativeCoreSection>,
    true_oov_oracle_metadata: Option<NativeOovSection>,
}

#[derive(Debug, Default, Serialize)]
struct EvaluationSection {
    counts: CorpusCounts,
    development: MetricSlice,
    final_holdout: MetricSlice,
    document_development: MetricSlice,
    document_holdout: MetricSlice,
    by_category: BTreeMap<String, MetricSlice>,
    by_feature: BTreeMap<String, MetricSlice>,
    by_document: BTreeMap<String, MetricSlice>,
    by_lemma_frequency: BTreeMap<String, MetricSlice>,
    by_generation_path: BTreeMap<String, MetricSlice>,
}

#[derive(Debug, Serialize)]
struct NativeCoreSection {
    metadata_policy: String,
    counts: CorpusCounts,
    aggregate: MetricSlice,
    by_category: BTreeMap<String, MetricSlice>,
    by_feature: BTreeMap<String, MetricSlice>,
    by_document: BTreeMap<String, MetricSlice>,
    by_formation: BTreeMap<String, MetricSlice>,
    metadata_sources_by_category: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct NativeOovSection {
    metadata_provenance: String,
    development: MetricSlice,
    final_holdout: MetricSlice,
    document_development: MetricSlice,
    document_holdout: MetricSlice,
    by_lemma_frequency: BTreeMap<String, MetricSlice>,
}

#[derive(Debug, Serialize)]
struct NormalizationPolicy {
    diplomatic_exact: String,
    project_lookup_exact: String,
    morphology_normalized_diagnostic: String,
}

#[derive(Debug, Serialize)]
struct PartitionPolicy {
    lemma_holdout: String,
    document_holdout: String,
    official_ud_partition: String,
}

#[derive(Debug)]
struct MappedCell {
    category: &'static str,
    feature_keys: Vec<String>,
}

#[derive(Debug)]
struct PendingOutcome {
    lemma_key: String,
    document: String,
    category: String,
    feature_label: String,
    attempted: bool,
    returned_forms: bool,
    formation: Option<String>,
    raw_top1_correct: bool,
    raw_any_correct: bool,
    lookup_top1_correct: bool,
    lookup_any_correct: bool,
}

#[derive(Debug)]
struct DetailRow<'a> {
    file: &'a str,
    sentence: &'a str,
    document: &'a str,
    token_id: &'a str,
    lemma: &'a str,
    surface: &'a str,
    features: &'a str,
    feature_label: &'a str,
    predictions: &'a [String],
    result: &'a str,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut ud_path = None;
    let mut syntacticus_path = None;
    let mut write_reports = false;
    let mut details_path = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--ud" => {
                ud_path = Some(PathBuf::from(
                    args.next().ok_or("expected a path after --ud")?,
                ));
            }
            "--syntacticus" => {
                syntacticus_path = Some(PathBuf::from(
                    args.next().ok_or("expected a path after --syntacticus")?,
                ));
            }
            "--write" => write_reports = true,
            "--details" => {
                details_path = Some(PathBuf::from(
                    args.next().ok_or("expected a path after --details")?,
                ));
            }
            _ => return Err(format!("unknown accuracy-corpus flag: {flag}").into()),
        }
    }
    if ud_path.is_none() && syntacticus_path.is_none() {
        return Err("accuracy-corpus requires --ud PATH, --syntacticus PATH, or both".into());
    }
    if write_reports && (ud_path.is_none() || syntacticus_path.is_none()) {
        return Err("accuracy-corpus --write requires both pinned corpus paths".into());
    }
    let manifest = load_manifest(root)?;
    let mut sources = Vec::new();
    let facade = if let Some(ud_path) = ud_path {
        let source = source_by_id(&manifest, "ud-ocs-proiel-r2.18")?;
        if source.kind != "ud-conllu" {
            return Err("pinned UD source has an unexpected kind".into());
        }
        verify_source(&ud_path, &source)?;
        sources.push(source.clone());
        evaluate_ud(&source, &ud_path, details_path.as_deref())?
    } else {
        EvaluationSection::default()
    };
    let (native_core, native_oov) = if let Some(syntacticus_path) = syntacticus_path {
        let source = source_by_id(&manifest, "syntacticus-ocs-20230428")?;
        if source.kind != "proiel-xml" {
            return Err("pinned Syntacticus source has an unexpected kind".into());
        }
        verify_source(&syntacticus_path, &source)?;
        sources.push(source.clone());
        let sections = evaluate_syntacticus(
            &source,
            &syntacticus_path,
            details_path.map(native_details_path).as_deref(),
        )?;
        (Some(sections.0), Some(sections.1))
    } else {
        (None, None)
    };
    let report = CorpusReport {
        schema_version: 2,
        mapper_version: 2,
        sources,
        normalization: normalization_policy(),
        partition_policy: partition_policy(),
        facade_attested_token_recall: facade,
        core_generalization_oracle_metadata: native_core,
        true_oov_oracle_metadata: native_oov,
    };
    check_thresholds(&report, &manifest.thresholds)?;
    let markdown = report_markdown(&report);
    let json = serde_json::to_vec_pretty(&report)?;
    if write_reports {
        fs::write(root.join("reports/corpus-accuracy.json"), json)?;
        fs::write(root.join("reports/corpus-accuracy.md"), &markdown)?;
    }
    print!("{markdown}");
    Ok(())
}

pub(crate) fn run_legacy(path: &Path, root: &Path) -> Result<(), Box<dyn Error>> {
    let manifest = load_manifest(root)?;
    let source = source_by_id(&manifest, "ud-ocs-proiel-r2.18")?;
    verify_source(path, &source)?;
    let report = evaluate_ud(&source, path, None)?;
    println!(
        "UD verb diagnostic (pinned {}, CC BY-NC-SA input, not bundled): {}/{} raw exact; {}/{} lookup exact; {} compatible bundles from {} verb/AUX tokens",
        source.version,
        report.counts.raw_any_correct,
        report.counts.generation_attempts,
        report.counts.lookup_any_correct,
        report.counts.generation_attempts,
        report.counts.compatible_bundles,
        report.counts.verb_or_aux_tokens,
    );
    Ok(())
}

fn evaluate_ud(
    source: &EvaluationSource,
    corpus_root: &Path,
    details_path: Option<&Path>,
) -> Result<EvaluationSection, Box<dyn Error>> {
    let mut counts = CorpusCounts::default();
    let mut outcomes = Vec::new();
    let mut details = details_path.map(|_| {
        String::from(
            "file\tsentence\tdocument\ttoken_id\tlemma\tsurface\tfeatures\tfeature\tpredictions\tresult\n",
        )
    });

    for file in &source.files {
        let file_path = corpus_root.join(&file.path);
        let input = fs::read_to_string(&file_path)?;
        let mut document = "unknown-document";
        let mut sentence = "unknown-sentence";
        for line in input.lines() {
            if let Some(value) = line.strip_prefix("# source = ") {
                document = value;
                continue;
            }
            if let Some(value) = line.strip_prefix("# sent_id = ") {
                sentence = value;
                continue;
            }
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let columns = line.split('\t').collect::<Vec<_>>();
            if columns.len() != 10 || columns[0].contains(['-', '.']) {
                continue;
            }
            counts.all_tokens += 1;
            let upos = columns[3];
            if !matches!(upos, "VERB" | "AUX") {
                continue;
            }
            counts.verb_or_aux_tokens += 1;
            let surface = columns[1];
            let lemma = columns[2];
            let features = columns[5];
            let mapped = match map_ud_verb(features) {
                Ok(mapped) => mapped,
                Err(reason) => {
                    bump(&mut counts.skipped_by_reason, reason);
                    if let Some(details) = details.as_mut() {
                        push_detail(
                            details,
                            DetailRow {
                                file: &file.path,
                                sentence,
                                document,
                                token_id: columns[0],
                                lemma,
                                surface,
                                features,
                                feature_label: "-",
                                predictions: &[],
                                result: reason,
                            },
                        );
                    }
                    continue;
                }
            };
            counts.compatible_bundles += 1;
            let lemma_key = match old_church_slavonic::orthography::lookup_key(lemma) {
                Ok(key) => key,
                Err(_) => {
                    bump(&mut counts.skipped_by_reason, "invalid-lemma");
                    continue;
                }
            };
            let candidates = old_church_slavonic::lookup(lemma, PartOfSpeech::Verb)?;
            if candidates.is_empty() {
                bump(&mut counts.skipped_by_reason, "unknown-lemma");
                if let Some(details) = details.as_mut() {
                    push_detail(
                        details,
                        DetailRow {
                            file: &file.path,
                            sentence,
                            document,
                            token_id: columns[0],
                            lemma,
                            surface,
                            features,
                            feature_label: &mapped.feature_keys.join("|"),
                            predictions: &[],
                            result: "unknown-lemma",
                        },
                    );
                }
                outcomes.push(PendingOutcome {
                    lemma_key,
                    document: document.to_string(),
                    category: mapped.category.to_string(),
                    feature_label: mapped.feature_keys.join("|"),
                    attempted: false,
                    returned_forms: false,
                    formation: None,
                    raw_top1_correct: false,
                    raw_any_correct: false,
                    lookup_top1_correct: false,
                    lookup_any_correct: false,
                });
                continue;
            }
            if candidates.len() != 1 {
                bump(&mut counts.skipped_by_reason, "ambiguous-lemma");
                outcomes.push(PendingOutcome {
                    lemma_key,
                    document: document.to_string(),
                    category: mapped.category.to_string(),
                    feature_label: mapped.feature_keys.join("|"),
                    attempted: false,
                    returned_forms: false,
                    formation: None,
                    raw_top1_correct: false,
                    raw_any_correct: false,
                    lookup_top1_correct: false,
                    lookup_any_correct: false,
                });
                continue;
            }
            counts.matched_lexemes += 1;
            counts.sufficient_lexical_metadata += 1;
            counts.generation_attempts += 1;
            let mut seen = BTreeSet::new();
            let mut generation_paths = BTreeSet::new();
            let mut predictions = Vec::new();
            for candidate in &candidates {
                for feature in &mapped.feature_keys {
                    if let Ok(forms) = old_church_slavonic::form_by_id(&candidate.id, feature) {
                        generation_paths.insert(form_source_label(&forms.source));
                        for form in forms.variants {
                            if seen.insert(form.text.clone()) {
                                predictions.push(form.text);
                            }
                        }
                    }
                }
            }
            let returned_forms = !predictions.is_empty();
            counts.tokens_with_returned_forms += usize::from(returned_forms);
            let raw_top1 = predictions.first().is_some_and(|form| form == surface);
            let raw_any = predictions.iter().any(|form| form == surface);
            let lookup_top1 = predictions
                .first()
                .is_some_and(|form| lookup_equal(form, surface));
            let lookup_any = predictions.iter().any(|form| lookup_equal(form, surface));
            counts.raw_top1_correct += usize::from(raw_top1);
            counts.raw_any_correct += usize::from(raw_any);
            counts.lookup_top1_correct += usize::from(lookup_top1);
            counts.lookup_any_correct += usize::from(lookup_any);
            if !returned_forms {
                bump(&mut counts.skipped_by_reason, "no-public-form");
            }
            if let Some(details) = details.as_mut() {
                let result = if raw_top1 {
                    "raw-top1-correct"
                } else if raw_any {
                    "raw-any-correct"
                } else if lookup_top1 {
                    "lookup-top1-correct"
                } else if lookup_any {
                    "lookup-any-correct"
                } else if returned_forms {
                    "mismatch"
                } else {
                    "no-public-form"
                };
                push_detail(
                    details,
                    DetailRow {
                        file: &file.path,
                        sentence,
                        document,
                        token_id: columns[0],
                        lemma,
                        surface,
                        features,
                        feature_label: &mapped.feature_keys.join("|"),
                        predictions: &predictions,
                        result,
                    },
                );
            }
            outcomes.push(PendingOutcome {
                lemma_key,
                document: document.to_string(),
                category: mapped.category.to_string(),
                feature_label: mapped.feature_keys.join("|"),
                attempted: true,
                returned_forms,
                formation: (!generation_paths.is_empty())
                    .then(|| generation_paths.into_iter().collect::<Vec<_>>().join("|")),
                raw_top1_correct: raw_top1,
                raw_any_correct: raw_any,
                lookup_top1_correct: lookup_top1,
                lookup_any_correct: lookup_any,
            });
        }
    }
    if let (Some(path), Some(details)) = (details_path, details) {
        fs::write(path, details)?;
    }

    let mut development = MetricSlice::default();
    let mut final_holdout = MetricSlice::default();
    let mut document_development = MetricSlice::default();
    let mut document_holdout = MetricSlice::default();
    let mut by_category = BTreeMap::new();
    let mut by_feature = BTreeMap::new();
    let mut by_document = BTreeMap::new();
    let frequencies = lemma_frequencies(&outcomes);
    let mut by_lemma_frequency = BTreeMap::new();
    let mut by_generation_path = BTreeMap::new();
    for outcome in &outcomes {
        let lemma_final = fnv1a(outcome.lemma_key.as_bytes()) % 5 == 0;
        let document_final = fnv1a(outcome.document.as_bytes()) % 5 == 0;
        observe(
            if lemma_final {
                &mut final_holdout
            } else {
                &mut development
            },
            outcome,
        );
        observe(
            if document_final {
                &mut document_holdout
            } else {
                &mut document_development
            },
            outcome,
        );
        observe(
            by_category.entry(outcome.category.clone()).or_default(),
            outcome,
        );
        observe(
            by_feature.entry(outcome.feature_label.clone()).or_default(),
            outcome,
        );
        observe(
            by_document.entry(outcome.document.clone()).or_default(),
            outcome,
        );
        let frequency = frequencies.get(&outcome.lemma_key).copied().unwrap_or(0);
        let band = match frequency {
            0 | 1 => "1",
            2..=5 => "2-5",
            6..=20 => "6-20",
            _ => "21+",
        };
        observe(
            by_lemma_frequency.entry(band.to_string()).or_default(),
            outcome,
        );
        if let Some(paths) = &outcome.formation {
            for path in paths.split('|') {
                observe(
                    by_generation_path.entry(path.to_string()).or_default(),
                    outcome,
                );
            }
        }
    }

    Ok(EvaluationSection {
        counts,
        development,
        final_holdout,
        document_development,
        document_holdout,
        by_category,
        by_feature,
        by_document,
        by_lemma_frequency,
        by_generation_path,
    })
}

fn form_source_label(source: &FormSource) -> String {
    match source {
        FormSource::DictionaryTable => "dictionary-table".to_string(),
        FormSource::DictionaryMetadataRule { rule_id } => {
            format!("dictionary-metadata-rule:{}", rule_id.code())
        }
        FormSource::DictionaryMetadataAnalyses => "dictionary-metadata-analyses".to_string(),
        FormSource::ExplicitMetadataRule { rule_id } => {
            format!("explicit-metadata-rule:{}", rule_id.code())
        }
        FormSource::OovPrediction { rule_id } => format!("oov-rule:{}", rule_id.code()),
        FormSource::ManualOverride => "manual-override".to_string(),
    }
}

fn load_manifest(root: &Path) -> Result<EvaluationManifest, Box<dyn Error>> {
    let manifest: EvaluationManifest =
        serde_json::from_slice(&fs::read(root.join(MANIFEST_PATH))?)?;
    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported evaluation manifest schema: {}",
            manifest.schema_version
        )
        .into());
    }
    Ok(manifest)
}

fn source_by_id(
    manifest: &EvaluationManifest,
    id: &str,
) -> Result<EvaluationSource, Box<dyn Error>> {
    manifest
        .sources
        .iter()
        .find(|source| source.id == id)
        .cloned()
        .ok_or_else(|| format!("evaluation manifest has no source `{id}`").into())
}

fn check_thresholds(
    report: &CorpusReport,
    thresholds: &EvaluationThresholds,
) -> Result<(), Box<dyn Error>> {
    let facade = &report.facade_attested_token_recall.counts;
    if facade.compatible_bundles > 0 {
        require_basis_points(
            "facade attempt coverage",
            facade.generation_attempts,
            facade.compatible_bundles,
            thresholds.facade_attempt_coverage_basis_points,
        )?;
        require_basis_points(
            "facade lookup-any conditional accuracy",
            facade.lookup_any_correct,
            facade.generation_attempts,
            thresholds.facade_lookup_any_accuracy_basis_points,
        )?;
    }
    if let Some(native) = &report.core_generalization_oracle_metadata {
        require_basis_points(
            "native oracle attempt coverage",
            native.counts.generation_attempts,
            native.counts.compatible_bundles,
            thresholds.native_attempt_coverage_basis_points,
        )?;
        require_basis_points(
            "native oracle lookup-any conditional accuracy",
            native.counts.lookup_any_correct,
            native.counts.generation_attempts,
            thresholds.native_lookup_any_accuracy_basis_points,
        )?;
    }
    Ok(())
}

fn require_basis_points(
    label: &str,
    numerator: usize,
    denominator: usize,
    minimum: usize,
) -> Result<(), Box<dyn Error>> {
    if denominator == 0 || numerator.saturating_mul(10_000) < minimum.saturating_mul(denominator) {
        return Err(format!(
            "corpus non-regression threshold failed for {label}: {numerator}/{denominator} is below {minimum} basis points"
        )
        .into());
    }
    Ok(())
}

fn verify_source(root: &Path, source: &EvaluationSource) -> Result<(), Box<dyn Error>> {
    if source.bundled {
        return Err("external evaluation source must not be marked bundled".into());
    }
    if source.license != "CC BY-NC-SA 4.0" {
        return Err(format!("unexpected evaluation license: {}", source.license).into());
    }
    for file in &source.files {
        let path = root.join(&file.path);
        if !path.is_file() {
            return Err(format!("pinned corpus file is missing: {}", path.display()).into());
        }
        let actual = file_sha256(&path)?;
        if actual != file.sha256 {
            return Err(format!(
                "pinned corpus hash mismatch for {}: expected {}, found {}",
                file.path, file.sha256, actual
            )
            .into());
        }
    }
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug, Clone)]
struct NativeToken {
    file: String,
    sentence: String,
    document: String,
    token_id: String,
    lemma: String,
    lemma_key: String,
    surface: String,
    morphology: String,
    cell: FiniteVerbCell,
}

#[derive(Debug, Clone)]
enum NativeFormation {
    Imperfect(ImperfectFormation),
    NewAorist,
}

#[derive(Debug, Clone)]
struct NativeMetadata {
    source_cell: FiniteVerbCell,
    stem: String,
    formation: NativeFormation,
    formation_label: String,
}

fn evaluate_syntacticus(
    source: &EvaluationSource,
    corpus_root: &Path,
    details_path: Option<&Path>,
) -> Result<(NativeCoreSection, NativeOovSection), Box<dyn Error>> {
    let mut counts = CorpusCounts::default();
    let mut groups: BTreeMap<(String, FiniteTense), Vec<NativeToken>> = BTreeMap::new();
    for file in &source.files {
        let input = fs::read_to_string(corpus_root.join(&file.path))?;
        if !input.contains("<field tag=\"tense\">")
            || !input.contains("<value tag=\"i\" summary=\"imperfect\"/>")
            || !input.contains("<value tag=\"a\" summary=\"aorist\"/>")
        {
            return Err(format!(
                "native PROIEL morphology schema marker mismatch in {}",
                file.path
            )
            .into());
        }
        let mut sentence = "unknown-sentence".to_string();
        for line in input.lines() {
            if line.contains("<sentence ") {
                if let Some(id) = xml_attribute(line, "id") {
                    sentence = id;
                }
                continue;
            }
            if !line.contains("<token ") || xml_attribute(line, "form").is_none() {
                continue;
            }
            counts.all_tokens += 1;
            if xml_attribute(line, "part-of-speech").as_deref() != Some("V-") {
                continue;
            }
            counts.verb_or_aux_tokens += 1;
            let Some(morphology) = xml_attribute(line, "morphology") else {
                bump(&mut counts.skipped_by_reason, "native-missing-morphology");
                continue;
            };
            let cell = match map_native_finite(&morphology) {
                Ok(cell) => cell,
                Err(reason) => {
                    bump(&mut counts.skipped_by_reason, reason);
                    continue;
                }
            };
            counts.compatible_bundles += 1;
            let Some(lemma) = xml_attribute(line, "lemma") else {
                bump(&mut counts.skipped_by_reason, "native-missing-lemma");
                continue;
            };
            let lemma_key = match old_church_slavonic::orthography::lookup_key(&lemma) {
                Ok(key) => key,
                Err(_) => {
                    bump(&mut counts.skipped_by_reason, "native-invalid-lemma");
                    continue;
                }
            };
            counts.matched_lexemes += 1;
            let surface = xml_attribute(line, "form").expect("form checked above");
            let token = NativeToken {
                file: file.path.clone(),
                sentence: sentence.clone(),
                document: file.partition.clone(),
                token_id: xml_attribute(line, "id").unwrap_or_else(|| "unknown-token".to_string()),
                lemma,
                lemma_key: lemma_key.clone(),
                surface,
                morphology,
                cell,
            };
            groups
                .entry((lemma_key, cell.tense))
                .or_default()
                .push(token);
        }
    }

    let mut details = details_path.map(|_| {
        String::from(
            "file\tsentence\tdocument\ttoken_id\tlemma\tsurface\tfeatures\tfeature\tpredictions\tresult\n",
        )
    });
    let mut outcomes = Vec::new();
    let mut metadata_sources_by_category = BTreeMap::new();
    for ((_lemma_key, tense), tokens) in groups {
        let metadata = choose_native_metadata(&tokens);
        if metadata.is_some() {
            bump(
                &mut metadata_sources_by_category,
                match tense {
                    FiniteTense::Imperfect => "imperfect",
                    FiniteTense::Aorist => "aorist-new",
                    FiniteTense::Present => unreachable!("native mapper only returns past cells"),
                },
            );
        }
        for token in tokens {
            let category = match tense {
                FiniteTense::Imperfect => "imperfect",
                FiniteTense::Aorist => "aorist-new",
                FiniteTense::Present => unreachable!("native mapper only returns past cells"),
            };
            let feature_label = token.cell.key();
            let Some(metadata) = metadata.as_ref() else {
                bump(
                    &mut counts.skipped_by_reason,
                    "native-missing-safe-oracle-metadata",
                );
                outcomes.push(unattempted_native_outcome(
                    &token,
                    category,
                    &feature_label,
                    None,
                ));
                continue;
            };
            counts.sufficient_lexical_metadata += 1;
            if token.cell == metadata.source_cell {
                bump(
                    &mut counts.skipped_by_reason,
                    "native-metadata-source-cell-excluded",
                );
                outcomes.push(unattempted_native_outcome(
                    &token,
                    category,
                    &feature_label,
                    Some(metadata.formation_label.clone()),
                ));
                continue;
            }

            let mut lexeme = VerbLexeme::new(token.lemma.clone(), VerbClass::Root);
            match metadata.formation {
                NativeFormation::Imperfect(formation) => {
                    lexeme.stems.imperfect = Some(metadata.stem.clone());
                    lexeme.formations.imperfect = Some(formation);
                    lexeme.formations.imperfect_variant_policy =
                        Some(old_church_slavonic::ImperfectVariantPolicy::UncontractedOnly);
                }
                NativeFormation::NewAorist => {
                    lexeme.stems.aorist = Some(metadata.stem.clone());
                    lexeme.formations.aorist = Some(AoristFormation::New);
                }
            }
            counts.generation_attempts += 1;
            let prediction = match old_church_slavonic_core::verb::finite(&lexeme, token.cell) {
                Ok(form) => form.text,
                Err(_) => {
                    bump(
                        &mut counts.skipped_by_reason,
                        "native-core-generation-error",
                    );
                    outcomes.push(failed_native_outcome(
                        &token,
                        category,
                        &feature_label,
                        Some(metadata.formation_label.clone()),
                    ));
                    continue;
                }
            };
            counts.tokens_with_returned_forms += 1;
            let raw = prediction == token.surface;
            let lookup = lookup_equal(&prediction, &token.surface);
            counts.raw_top1_correct += usize::from(raw);
            counts.raw_any_correct += usize::from(raw);
            counts.lookup_top1_correct += usize::from(lookup);
            counts.lookup_any_correct += usize::from(lookup);
            if let Some(details) = details.as_mut() {
                push_detail(
                    details,
                    DetailRow {
                        file: &token.file,
                        sentence: &token.sentence,
                        document: &token.document,
                        token_id: &token.token_id,
                        lemma: &token.lemma,
                        surface: &token.surface,
                        features: &token.morphology,
                        feature_label: &feature_label,
                        predictions: std::slice::from_ref(&prediction),
                        result: if raw {
                            "raw-top1-correct"
                        } else if lookup {
                            "lookup-top1-correct"
                        } else {
                            "mismatch"
                        },
                    },
                );
            }
            outcomes.push(PendingOutcome {
                lemma_key: token.lemma_key,
                document: token.document,
                category: category.to_string(),
                feature_label,
                attempted: true,
                returned_forms: true,
                formation: Some(metadata.formation_label.clone()),
                raw_top1_correct: raw,
                raw_any_correct: raw,
                lookup_top1_correct: lookup,
                lookup_any_correct: lookup,
            });
        }
    }
    if let (Some(path), Some(details)) = (details_path, details) {
        fs::write(path, details)?;
    }

    let mut aggregate = MetricSlice::default();
    let mut by_category = BTreeMap::new();
    let mut by_feature = BTreeMap::new();
    let mut by_document = BTreeMap::new();
    let mut by_formation = BTreeMap::new();
    let mut development = MetricSlice::default();
    let mut final_holdout = MetricSlice::default();
    let mut document_development = MetricSlice::default();
    let mut document_holdout = MetricSlice::default();
    let frequencies = lemma_frequencies(&outcomes);
    let mut by_lemma_frequency = BTreeMap::new();
    for outcome in &outcomes {
        observe(&mut aggregate, outcome);
        observe(
            by_category.entry(outcome.category.clone()).or_default(),
            outcome,
        );
        observe(
            by_feature.entry(outcome.feature_label.clone()).or_default(),
            outcome,
        );
        observe(
            by_document.entry(outcome.document.clone()).or_default(),
            outcome,
        );
        observe(
            by_formation
                .entry(
                    outcome
                        .formation
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                )
                .or_default(),
            outcome,
        );
        observe(
            if is_final_lemma(&outcome.lemma_key) {
                &mut final_holdout
            } else {
                &mut development
            },
            outcome,
        );
        observe(
            if is_final_document(&outcome.document) {
                &mut document_holdout
            } else {
                &mut document_development
            },
            outcome,
        );
        let band = frequency_band(frequencies.get(&outcome.lemma_key).copied().unwrap_or(0));
        observe(
            by_lemma_frequency.entry(band.to_string()).or_default(),
            outcome,
        );
    }

    Ok((
        NativeCoreSection {
            metadata_policy: "oracle principal part derived from one morphologically diagnostic token of the same lemma; every token in the source person-number cell is excluded; no target surface is consulted during generation".to_string(),
            counts,
            aggregate,
            by_category,
            by_feature,
            by_document,
            by_formation,
            metadata_sources_by_category,
        },
        NativeOovSection {
            metadata_provenance: "same native-corpus oracle principal-part policy as core generalization; lemmas are assigned wholly to FNV-1a development or final holdout".to_string(),
            development,
            final_holdout,
            document_development,
            document_holdout,
            by_lemma_frequency,
        },
    ))
}

fn unattempted_native_outcome(
    token: &NativeToken,
    category: &str,
    feature_label: &str,
    formation: Option<String>,
) -> PendingOutcome {
    PendingOutcome {
        lemma_key: token.lemma_key.clone(),
        document: token.document.clone(),
        category: category.to_string(),
        feature_label: feature_label.to_string(),
        attempted: false,
        returned_forms: false,
        formation,
        raw_top1_correct: false,
        raw_any_correct: false,
        lookup_top1_correct: false,
        lookup_any_correct: false,
    }
}

fn failed_native_outcome(
    token: &NativeToken,
    category: &str,
    feature_label: &str,
    formation: Option<String>,
) -> PendingOutcome {
    let mut outcome = unattempted_native_outcome(token, category, feature_label, formation);
    outcome.attempted = true;
    outcome
}

fn choose_native_metadata(tokens: &[NativeToken]) -> Option<NativeMetadata> {
    tokens
        .iter()
        .filter_map(|token| {
            let metadata = derive_native_metadata(token)?;
            let priority = native_source_cell_priority(token.cell);
            let editorial_penalty = usize::from(
                token
                    .surface
                    .chars()
                    .any(|character| !character.is_alphabetic()),
            );
            Some((
                priority,
                editorial_penalty,
                &token.file,
                &token.token_id,
                metadata,
            ))
        })
        .min_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.cmp(&right.1))
                .then(left.2.cmp(right.2))
                .then(left.3.cmp(right.3))
        })
        .map(|tuple| tuple.4)
}

fn derive_native_metadata(token: &NativeToken) -> Option<NativeMetadata> {
    match token.cell.tense {
        FiniteTense::Imperfect => {
            let personal = imperfect_personal_ending(token.cell)?;
            let base = token.surface.strip_suffix(personal)?;
            let (stem, formation, label) = if let Some(stem) = base.strip_suffix("ѣа") {
                (stem, ImperfectFormation::YatA, "imperfect-yat-a")
            } else if let Some(stem) = base.strip_suffix('а') {
                (stem, ImperfectFormation::A, "imperfect-a-explicit-base")
            } else {
                return None;
            };
            if stem.is_empty() {
                return None;
            }
            Some(NativeMetadata {
                source_cell: token.cell,
                stem: stem.to_string(),
                formation: NativeFormation::Imperfect(formation),
                formation_label: label.to_string(),
            })
        }
        FiniteTense::Aorist => {
            let ending = new_aorist_diagnostic_ending(token.cell)?;
            let stem = token.surface.strip_suffix(ending)?;
            if stem.is_empty() {
                return None;
            }
            Some(NativeMetadata {
                source_cell: token.cell,
                stem: stem.to_string(),
                formation: NativeFormation::NewAorist,
                formation_label: "aorist-new-ox".to_string(),
            })
        }
        FiniteTense::Present => None,
    }
}

fn imperfect_personal_ending(cell: FiniteVerbCell) -> Option<&'static str> {
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => Some("хъ"),
        (Person::Second | Person::Third, Number::Singular) => Some("ше"),
        (Person::First, Number::Dual) => Some("ховѣ"),
        (Person::Second, Number::Dual) => Some("шета"),
        (Person::Third, Number::Dual) => Some("шете"),
        (Person::First, Number::Plural) => Some("хомъ"),
        (Person::Second, Number::Plural) => Some("шете"),
        (Person::Third, Number::Plural) => Some("хѫ"),
    }
}

fn new_aorist_diagnostic_ending(cell: FiniteVerbCell) -> Option<&'static str> {
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => Some("охъ"),
        (Person::First, Number::Dual) => Some("оховѣ"),
        (Person::Second, Number::Dual) => Some("оста"),
        (Person::Third, Number::Dual) => Some("осте"),
        (Person::First, Number::Plural) => Some("охомъ"),
        (Person::Second, Number::Plural) => Some("осте"),
        (Person::Third, Number::Plural) => Some("ошѧ"),
        (Person::Second | Person::Third, Number::Singular) => None,
    }
}

fn native_source_cell_priority(cell: FiniteVerbCell) -> u8 {
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => 0,
        (Person::First, Number::Dual) => 1,
        (Person::First, Number::Plural) => 2,
        (Person::Third, Number::Plural) => 3,
        (Person::Third, Number::Dual) => 4,
        (Person::Second, Number::Plural) => 5,
        (Person::Second, Number::Dual) => 6,
        (Person::Second, Number::Singular) => 7,
        (Person::Third, Number::Singular) => 8,
    }
}

fn map_native_finite(morphology: &str) -> Result<FiniteVerbCell, &'static str> {
    let chars = morphology.chars().collect::<Vec<_>>();
    if chars.len() != 10 {
        return Err("native-invalid-morphology-width");
    }
    let person = match chars[0] {
        '1' => Person::First,
        '2' => Person::Second,
        '3' => Person::Third,
        _ => return Err("native-incomplete-person"),
    };
    let number = match chars[1] {
        's' => Number::Singular,
        'd' => Number::Dual,
        'p' => Number::Plural,
        _ => return Err("native-incomplete-number"),
    };
    let tense = match chars[2] {
        'i' => FiniteTense::Imperfect,
        'a' => FiniteTense::Aorist,
        _ => return Err("native-not-imperfect-or-aorist"),
    };
    if chars[3] != 'i' {
        return Err("native-not-indicative");
    }
    if chars[4] != 'a' {
        return Err("native-not-active");
    }
    Ok(FiniteVerbCell {
        tense,
        person,
        number,
    })
}

fn xml_attribute(line: &str, name: &str) -> Option<String> {
    let marker = format!(" {name}=\"");
    let start = line.find(&marker)? + marker.len();
    let value = line.get(start..)?.split_once('"')?.0;
    Some(
        value
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&"),
    )
}

fn native_details_path(path: PathBuf) -> PathBuf {
    let mut name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("corpus-details")
        .to_string();
    name.push_str("-native.tsv");
    path.with_file_name(name)
}

fn normalization_policy() -> NormalizationPolicy {
    NormalizationPolicy {
        diplomatic_exact: "byte-for-byte corpus surface equality".to_string(),
        project_lookup_exact: "the shared project NFC plus Unicode lowercase lookup key"
            .to_string(),
        morphology_normalized_diagnostic:
            "disabled: no independently audited lossless morphology fold exists".to_string(),
    }
}

fn partition_policy() -> PartitionPolicy {
    PartitionPolicy {
        lemma_holdout: "shared normalized lemma key; FNV-1a modulo 5; residue 0 is final holdout"
            .to_string(),
        document_holdout:
            "document/manuscript label; FNV-1a modulo 5; residue 0 is document holdout".to_string(),
        official_ud_partition: "source file partition retained in the verified file manifest"
            .to_string(),
    }
}

fn map_ud_verb(features: &str) -> Result<MappedCell, &'static str> {
    let mut map = BTreeMap::new();
    for feature in features.split('|') {
        let Some((name, value)) = feature.split_once('=') else {
            if feature == "_" {
                continue;
            }
            return Err("invalid-ud-feature");
        };
        if map.insert(name, value).is_some() {
            return Err("contradictory-or-duplicate-ud-feature");
        }
    }
    if map.get("Polarity").copied() == Some("Neg") {
        return Err("incompatible-negative-form");
    }
    match map.get("VerbForm").copied() {
        Some("Inf") => Ok(MappedCell {
            category: "infinitive",
            feature_keys: vec!["verb:infinitive".to_string()],
        }),
        Some("Sup") => Ok(MappedCell {
            category: "supine",
            feature_keys: vec!["verb:supine".to_string()],
        }),
        Some("PartRes") => map_resultative(&map),
        Some("Part") => map_participle(&map),
        Some("Fin") => map_finite(&map),
        Some(_) => Err("incompatible-verb-form"),
        None => Err("missing-verb-form"),
    }
}

fn map_finite(map: &BTreeMap<&str, &str>) -> Result<MappedCell, &'static str> {
    let person = map_person(map.get("Person").copied())?;
    let number = map_number(map.get("Number").copied())?;
    match map.get("Voice").copied() {
        Some("Act") => {}
        Some(_) => return Err("incompatible-finite-voice"),
        None => return Err("missing-finite-voice"),
    }
    match map.get("Mood").copied() {
        Some("Imp") => {
            match map.get("Tense").copied() {
                Some("Pres") => {}
                Some(_) => return Err("incompatible-imperative-tense"),
                None => return Err("missing-imperative-tense"),
            }
            let cell = old_church_slavonic::ImperativeCell {
                person: map_person_enum(person),
                number: map_number_code(number),
            };
            if !cell.is_supported() {
                return Err("unsupported-imperative-cell");
            }
            Ok(MappedCell {
                category: "imperative",
                feature_keys: vec![cell.key()],
            })
        }
        Some("Ind") => match map.get("Tense").copied() {
            Some("Pres") => Ok(MappedCell {
                category: "present",
                feature_keys: vec![format!("verb:finite:present:{person}:{number}")],
            }),
            Some("Past") => Err("incompatible-past-subtype"),
            Some(_) => Err("incompatible-finite-tense"),
            None => Err("missing-finite-tense"),
        },
        Some(_) => Err("incompatible-finite-mood"),
        None => Err("missing-finite-mood"),
    }
}

fn map_resultative(map: &BTreeMap<&str, &str>) -> Result<MappedCell, &'static str> {
    match map.get("Case").copied() {
        Some("Nom") => {}
        Some(_) => return Err("incompatible-resultative-case"),
        None => return Err("missing-resultative-case"),
    }
    match map.get("Tense").copied() {
        Some("Past") => {}
        Some(_) => return Err("incompatible-resultative-tense"),
        None => return Err("missing-resultative-tense"),
    }
    match map.get("Variant").copied() {
        Some("Short") => {}
        Some(_) => return Err("incompatible-resultative-variant"),
        None => return Err("missing-resultative-variant"),
    }
    match map.get("Voice").copied() {
        Some("Act") => {}
        Some(_) => return Err("incompatible-resultative-voice"),
        None => return Err("missing-resultative-voice"),
    }
    let gender = map_gender(map.get("Gender").copied())?;
    let number = map_number(map.get("Number").copied())?;
    Ok(MappedCell {
        category: "l-participle",
        feature_keys: vec![format!("verb:l-participle:{gender}:{number}")],
    })
}

fn map_person_enum(person: &str) -> Person {
    match person {
        "1" => Person::First,
        "2" => Person::Second,
        "3" => Person::Third,
        _ => unreachable!("map_person returned an invalid code"),
    }
}

fn map_number_code(number: &str) -> Number {
    match number {
        "sg" => Number::Singular,
        "du" => Number::Dual,
        "pl" => Number::Plural,
        _ => unreachable!("map_number returned an invalid code"),
    }
}

fn map_participle(map: &BTreeMap<&str, &str>) -> Result<MappedCell, &'static str> {
    let kind = match (map.get("Tense").copied(), map.get("Voice").copied()) {
        (Some("Pres"), Some("Act")) => ParticipleKind::PresentActive,
        (Some("Pres"), Some("Pass")) => ParticipleKind::PresentPassive,
        (Some("Past"), Some("Act")) => ParticipleKind::PastActive,
        (Some("Past"), Some("Pass")) => ParticipleKind::PastPassive,
        (None, _) => return Err("missing-participle-tense"),
        (_, None) => return Err("missing-participle-voice"),
        _ => return Err("incompatible-participle-kind"),
    };
    let form = match map.get("Variant").copied() {
        Some("Short") => AdjectiveForm::Short,
        Some("Long") => AdjectiveForm::Long,
        Some(_) => return Err("incompatible-participle-variant"),
        None => return Err("missing-participle-variant"),
    };
    let case = map_case(map.get("Case").copied())?;
    let number = map_number_enum(map.get("Number").copied())?;
    let gender = map_gender_enum(map.get("Gender").copied())?;
    let mut keys = Vec::new();
    for animacy in [Animacy::Inanimate, Animacy::Animate] {
        let cell = ParticipleCell {
            kind,
            adjective: AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
            },
        };
        keys.push(cell.key());
        if !(case == Case::Accusative && gender == Gender::Masculine) {
            break;
        }
    }
    Ok(MappedCell {
        category: match kind {
            ParticipleKind::PresentActive => "present-active-participle",
            ParticipleKind::PresentPassive => "present-passive-participle",
            ParticipleKind::PastActive => "past-active-participle",
            ParticipleKind::PastPassive => "past-passive-participle",
        },
        feature_keys: keys,
    })
}

fn map_case(value: Option<&str>) -> Result<Case, &'static str> {
    match value {
        Some("Nom") => Ok(Case::Nominative),
        Some("Gen") => Ok(Case::Genitive),
        Some("Dat") => Ok(Case::Dative),
        Some("Acc") => Ok(Case::Accusative),
        Some("Ins") => Ok(Case::Instrumental),
        Some("Loc") => Ok(Case::Locative),
        Some("Voc") => Ok(Case::Vocative),
        Some(_) => Err("incompatible-participle-case"),
        None => Err("missing-participle-case"),
    }
}

fn map_number(value: Option<&str>) -> Result<&'static str, &'static str> {
    match value {
        Some("Sing") => Ok("sg"),
        Some("Dual") => Ok("du"),
        Some("Plur") => Ok("pl"),
        Some(_) => Err("incompatible-number"),
        None => Err("missing-number"),
    }
}

fn map_number_enum(value: Option<&str>) -> Result<Number, &'static str> {
    match value {
        Some("Sing") => Ok(Number::Singular),
        Some("Dual") => Ok(Number::Dual),
        Some("Plur") => Ok(Number::Plural),
        Some(_) => Err("incompatible-participle-number"),
        None => Err("missing-participle-number"),
    }
}

fn map_gender(value: Option<&str>) -> Result<&'static str, &'static str> {
    match value {
        Some("Masc") => Ok("m"),
        Some("Fem") => Ok("f"),
        Some("Neut") => Ok("n"),
        Some(_) => Err("incompatible-gender"),
        None => Err("missing-gender"),
    }
}

fn map_gender_enum(value: Option<&str>) -> Result<Gender, &'static str> {
    match value {
        Some("Masc") => Ok(Gender::Masculine),
        Some("Fem") => Ok(Gender::Feminine),
        Some("Neut") => Ok(Gender::Neuter),
        Some(_) => Err("incompatible-participle-gender"),
        None => Err("missing-participle-gender"),
    }
}

fn map_person(value: Option<&str>) -> Result<&'static str, &'static str> {
    match value {
        Some("1") => Ok("1"),
        Some("2") => Ok("2"),
        Some("3") => Ok("3"),
        Some(_) => Err("incompatible-person"),
        None => Err("missing-person"),
    }
}

fn observe(slice: &mut MetricSlice, outcome: &PendingOutcome) {
    slice.observe_eligible();
    if outcome.attempted {
        slice.observe_attempt(
            outcome.returned_forms,
            outcome.raw_top1_correct,
            outcome.raw_any_correct,
            outcome.lookup_top1_correct,
            outcome.lookup_any_correct,
        );
    }
}

fn lemma_frequencies(outcomes: &[PendingOutcome]) -> BTreeMap<String, usize> {
    let mut frequencies = BTreeMap::new();
    for outcome in outcomes {
        *frequencies.entry(outcome.lemma_key.clone()).or_default() += 1;
    }
    frequencies
}

fn lookup_equal(left: &str, right: &str) -> bool {
    match (
        old_church_slavonic::orthography::lookup_key(left),
        old_church_slavonic::orthography::lookup_key(right),
    ) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_default() += 1;
}

fn push_detail(out: &mut String, row: DetailRow<'_>) {
    let predictions = row.predictions.join("|");
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        tsv(row.file),
        tsv(row.sentence),
        tsv(row.document),
        tsv(row.token_id),
        tsv(row.lemma),
        tsv(row.surface),
        tsv(row.features),
        tsv(row.feature_label),
        tsv(&predictions),
        tsv(row.result),
    ));
}

fn tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn is_final_lemma(lemma_key: &str) -> bool {
    fnv1a(lemma_key.as_bytes()) % 5 == 0
}

fn is_final_document(document: &str) -> bool {
    fnv1a(document.as_bytes()) % 5 == 0
}

fn frequency_band(frequency: usize) -> &'static str {
    match frequency {
        0 | 1 => "1",
        2..=5 => "2-5",
        6..=20 => "6-20",
        _ => "21+",
    }
}

fn report_markdown(report: &CorpusReport) -> String {
    let mut out = String::new();
    out.push_str("# Attested verb corpus accuracy\n\n");
    out.push_str("All listed inputs are external and every pinned file hash was verified before evaluation:\n\n");
    for source in &report.sources {
        out.push_str(&format!(
            "- {} `{}` at `{}` ({}).\n",
            source.name, source.version, source.commit, source.license
        ));
    }
    out.push('\n');
    out.push_str("UD finite `Tense=Past` is deliberately excluded because it does not distinguish aorist from imperfect. `Aspect` is never used as a substitute.\n\n");
    out.push_str("## 1. Facade attested-token recall (UD)\n\n");
    out.push_str("This asks whether the public table-first facade can expose an attested token for an unambiguous known dictionary lexeme. The generation-path slice separates exact dictionary cells from source-backed dictionary-metadata rules. Because the target token is not held out from dictionary principal-part extraction, this is real-text recall, not the leakage-controlled dictionary held-cell score.\n\n");
    push_counts(&mut out, &report.facade_attested_token_recall.counts);

    out.push_str("\n### Fixed holdouts\n\n");
    push_slice_header(&mut out, "Partition");
    push_slice_row(
        &mut out,
        "lemma development",
        &report.facade_attested_token_recall.development,
    );
    push_slice_row(
        &mut out,
        "lemma final holdout",
        &report.facade_attested_token_recall.final_holdout,
    );
    push_slice_row(
        &mut out,
        "document development",
        &report.facade_attested_token_recall.document_development,
    );
    push_slice_row(
        &mut out,
        "document holdout",
        &report.facade_attested_token_recall.document_holdout,
    );

    out.push_str("\n### By verb category\n\n");
    push_slice_header(&mut out, "Category");
    for (name, slice) in &report.facade_attested_token_recall.by_category {
        push_slice_row(&mut out, name, slice);
    }

    out.push_str("\n### By public generation path\n\n");
    push_slice_header(&mut out, "Path");
    for (name, slice) in &report.facade_attested_token_recall.by_generation_path {
        push_slice_row(&mut out, name, slice);
    }

    out.push_str("\n### Facade skip and incompatibility reasons\n\n");
    for (reason, count) in &report.facade_attested_token_recall.counts.skipped_by_reason {
        out.push_str(&format!("- `{reason}`: {count}\n"));
    }

    if let Some(core) = &report.core_generalization_oracle_metadata {
        out.push_str(
            "\n## 2. Core generalization with declared principal parts (native PROIEL/TOROT)\n\n",
        );
        out.push_str(&core.metadata_policy);
        out.push_str(". This is explicitly an oracle-metadata result, not end-to-end lemmatization or class induction.\n\n");
        push_counts(&mut out, &core.counts);
        out.push_str("\n### Aggregate and category results\n\n");
        push_slice_header(&mut out, "Slice");
        push_slice_row(&mut out, "all native oracle cells", &core.aggregate);
        for (name, slice) in &core.by_category {
            push_slice_row(&mut out, name, slice);
        }
        out.push_str("\n### By independently declared formation\n\n");
        push_slice_header(&mut out, "Formation");
        for (name, slice) in &core.by_formation {
            push_slice_row(&mut out, name, slice);
        }
        out.push_str("\n### Native skip and incompatibility reasons\n\n");
        for (reason, count) in &core.counts.skipped_by_reason {
            out.push_str(&format!("- `{reason}`: {count}\n"));
        }
    }

    if let Some(oov) = &report.true_oov_oracle_metadata {
        out.push_str("\n## 3. Lemma-disjoint OOV view (native oracle metadata)\n\n");
        out.push_str(&oov.metadata_provenance);
        out.push_str(
            ". The final partition was frozen by the shared hash rule before rule tuning.\n\n",
        );
        push_slice_header(&mut out, "Partition");
        push_slice_row(&mut out, "lemma development", &oov.development);
        push_slice_row(&mut out, "lemma final holdout", &oov.final_holdout);
        push_slice_row(&mut out, "document development", &oov.document_development);
        push_slice_row(&mut out, "document holdout", &oov.document_holdout);
    }

    out.push_str("\nThe morphology-normalized diagnostic is disabled: no independently audited lossless fold exists. Detailed token mismatches are emitted only with `--details PATH` (the native file receives a `-native` suffix) and must not be committed for these CC BY-NC-SA sources.\n");
    out
}

fn push_counts(out: &mut String, counts: &CorpusCounts) {
    out.push_str("| Stage | Tokens |\n|---|---:|\n");
    out.push_str(&format!("| all corpus tokens | {} |\n", counts.all_tokens));
    out.push_str(&format!(
        "| verb/AUX tokens | {} |\n",
        counts.verb_or_aux_tokens
    ));
    out.push_str(&format!(
        "| losslessly compatible bundles | {} |\n",
        counts.compatible_bundles
    ));
    out.push_str(&format!(
        "| unambiguous/valid lemma matches | {} |\n",
        counts.matched_lexemes
    ));
    out.push_str(&format!(
        "| sufficient lexical metadata | {} |\n",
        counts.sufficient_lexical_metadata
    ));
    out.push_str(&format!(
        "| generation attempts | {} |\n",
        counts.generation_attempts
    ));
    out.push_str(&format!(
        "| attempts returning forms | {} |\n",
        counts.tokens_with_returned_forms
    ));
    out.push_str(&format!(
        "| diplomatic top-1 / any | {} / {} |\n",
        counts.raw_top1_correct, counts.raw_any_correct
    ));
    out.push_str(&format!(
        "| project-lookup top-1 / any | {} / {} |\n",
        counts.lookup_top1_correct, counts.lookup_any_correct
    ));
}

fn push_slice_header(out: &mut String, label: &str) {
    out.push_str(&format!(
        "| {label} | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |\n|---|---:|---:|---:|---:|---:|---:|---:|\n"
    ));
}

fn push_slice_row(out: &mut String, name: &str, slice: &MetricSlice) {
    out.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
        name,
        slice.eligible,
        slice.attempted,
        slice.returned_forms,
        ratio(slice.raw_top1_correct, slice.attempted),
        ratio(slice.raw_any_correct, slice.attempted),
        ratio(slice.lookup_top1_correct, slice.attempted),
        ratio(slice.lookup_any_correct, slice.attempted),
    ));
}

fn ratio(correct: usize, total: usize) -> String {
    if total == 0 {
        return "-".to_string();
    }
    format!(
        "{correct}/{total} ({:.2}%)",
        correct as f64 * 100.0 / total as f64
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_rejects_undifferentiated_past_and_maps_supported_verbs() {
        assert_eq!(
            map_ud_verb(
                "Aspect=Perf|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin|Voice=Act"
            )
            .expect_err("UD past must not be guessed"),
            "incompatible-past-subtype"
        );
        assert_eq!(
            map_ud_verb("Mood=Imp|Number=Dual|Person=2|Tense=Pres|VerbForm=Fin|Voice=Act")
                .expect("imperative is compatible")
                .feature_keys,
            ["verb:imperative:2:du"]
        );
        assert_eq!(
            map_ud_verb("Number=Sing|Person=1|Tense=Pres|Mood=Ind|VerbForm=Fin|Voice=Act")
                .expect("present is compatible")
                .feature_keys,
            ["verb:finite:present:1:sg"]
        );
        assert_eq!(
            map_ud_verb("Number=Sing|Person=1|Person=2|Tense=Pres|Mood=Ind|VerbForm=Fin")
                .expect_err("contradictory person must fail closed"),
            "contradictory-or-duplicate-ud-feature"
        );
        assert_eq!(
            map_ud_verb("Mood=Imp|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin|Voice=Act")
                .expect_err("untyped imperative cell must fail closed"),
            "unsupported-imperative-cell"
        );
        assert_eq!(
            map_ud_verb(
                "Mood=Ind|Number=Sing|Person=3|Polarity=Neg|Tense=Pres|VerbForm=Fin|Voice=Act"
            )
            .expect_err("fused negative forms are not positive finite cells"),
            "incompatible-negative-form"
        );
        assert_eq!(
            map_ud_verb("Mood=Imp|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin|Voice=Pass")
                .expect_err("passive finite form is incompatible"),
            "incompatible-finite-voice"
        );
    }

    #[test]
    fn declined_participle_requires_every_lossless_dimension() {
        let mapped = map_ud_verb(
            "Case=Gen|Gender=Masc|Number=Sing|Tense=Past|Variant=Short|VerbForm=Part|Voice=Act",
        )
        .expect("complete participle should map");
        assert_eq!(mapped.category, "past-active-participle");
        assert_eq!(
            mapped.feature_keys,
            ["verb:participle:past-active:adj:short:gen:sg:m:in"]
        );
        assert_eq!(
            map_ud_verb("Gender=Masc|Number=Sing|Tense=Past|Variant=Short|VerbForm=Part|Voice=Act")
                .expect_err("case is required"),
            "missing-participle-case"
        );
        assert_eq!(
            map_ud_verb("Case=Nom|Gender=Masc|Number=Sing|Tense=Past|VerbForm=PartRes|Voice=Act")
                .expect_err("resultative variant must be explicit"),
            "missing-resultative-variant"
        );
    }

    #[test]
    fn native_mapper_preserves_past_subtype_and_rejects_partial_bundles() {
        assert_eq!(
            map_native_finite("3siia----i").expect("native imperfect"),
            FiniteVerbCell {
                tense: FiniteTense::Imperfect,
                person: Person::Third,
                number: Number::Singular,
            }
        );
        assert_eq!(
            map_native_finite("1saia----i").expect("native aorist"),
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            }
        );
        assert_eq!(
            map_native_finite("-siia----i").expect_err("person is required"),
            "native-incomplete-person"
        );
        assert_eq!(
            map_native_finite("3spia----i").expect_err("present is out of native past scope"),
            "native-not-imperfect-or-aorist"
        );
        assert_eq!(
            map_native_finite("3sipa----i").expect_err("participle is not finite indicative"),
            "native-not-indicative"
        );
    }

    #[test]
    fn native_oracle_derivation_uses_only_diagnostic_source_cells() {
        let imperfect = native_token(
            "нести",
            "несѣаше",
            FiniteVerbCell {
                tense: FiniteTense::Imperfect,
                person: Person::Third,
                number: Number::Singular,
            },
        );
        let metadata = derive_native_metadata(&imperfect).expect("diagnostic imperfect");
        assert_eq!(metadata.stem, "нес");
        assert!(matches!(
            metadata.formation,
            NativeFormation::Imperfect(ImperfectFormation::YatA)
        ));

        let aorist = native_token(
            "мощи",
            "могохъ",
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            },
        );
        let metadata = derive_native_metadata(&aorist).expect("diagnostic new aorist");
        assert_eq!(metadata.stem, "мог");
        assert!(matches!(metadata.formation, NativeFormation::NewAorist));

        let nondiagnostic = native_token(
            "мощи",
            "може",
            FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::Third,
                number: Number::Singular,
            },
        );
        assert!(derive_native_metadata(&nondiagnostic).is_none());
    }

    #[test]
    fn frozen_corpus_partition_witnesses_do_not_drift() {
        assert!(is_final_lemma("нести"));
        assert!(!is_final_lemma("бꙑти"));
        assert_eq!(fnv1a("нести".as_bytes()), 9_211_201_522_989_420_120);
        assert_eq!(frequency_band(1), "1");
        assert_eq!(frequency_band(2), "2-5");
        assert_eq!(frequency_band(21), "21+");
    }

    #[test]
    fn constrained_xml_attribute_parser_decodes_entities() {
        let line = r#"<token id="42" form="а&amp;б" lemma="а&lt;б"/>"#;
        assert_eq!(xml_attribute(line, "id").as_deref(), Some("42"));
        assert_eq!(xml_attribute(line, "form").as_deref(), Some("а&б"));
        assert_eq!(xml_attribute(line, "lemma").as_deref(), Some("а<б"));
        assert_eq!(xml_attribute(line, "missing"), None);
    }

    fn native_token(lemma: &str, surface: &str, cell: FiniteVerbCell) -> NativeToken {
        NativeToken {
            file: "fixture.xml".to_string(),
            sentence: "s1".to_string(),
            document: "fixture".to_string(),
            token_id: "t1".to_string(),
            lemma: lemma.to_string(),
            lemma_key: lemma.to_string(),
            surface: surface.to_string(),
            morphology: "fixture".to_string(),
            cell,
        }
    }
}
