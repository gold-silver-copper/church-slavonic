#![forbid(unsafe_code)]

mod corpus;

use old_church_slavonic::advanced::cells::{
    AdjectiveCell, AdjectiveForm, ClosedClassCell, FiniteVerbCell, ImperativeCell, LParticipleCell,
    NounCell, ParticipleCell,
};
use old_church_slavonic::advanced::rules::{
    AdjectiveClass, AoristFormation, ImperativeFormation, ImperfectFormation,
    ImperfectVariantPolicy, NounClass, NumberRestriction, PastActiveParticipleFormation,
    PastPassiveParticipleFormation, PresentActiveParticipleFormation,
    PresentPassiveParticipleFormation, VerbClass,
};
use old_church_slavonic::advanced::{by_id, metadata as api_metadata, raw_features};
use old_church_slavonic::{
    Animacy, Case, FiniteTense, Gender, InflectionError, Number, PartOfSpeech, ParticipleKind,
    Person,
};
use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::orthography;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_extractor::extract::{
    check_registry, load_registry, refresh, refresh_derived_registry, registry_with_overrides,
};
use old_church_slavonic_extractor::schema::{FormRow, LexemeRow, Registry};
use old_church_slavonic_extractor::semantics::{check_dictionary, refresh_dictionary};
use old_church_slavonic_extractor::verb_metadata;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("refresh-data") => {
            let dump = required_path_flag(&mut args, "--dump")?;
            refresh(&dump, &workspace_root()?)
        }
        Some("refresh-dictionary") => {
            let dump = required_path_flag(&mut args, "--dump")?;
            refresh_dictionary(&dump, &workspace_root()?)
        }
        Some("refresh-derived-registry") => refresh_derived_registry(&workspace_root()?),
        Some("check-registry") => check_registry(&workspace_root()?),
        Some("check-dictionary") => check_dictionary(&workspace_root()?),
        Some("extraction-report") => extraction_report(),
        Some("accuracy") => accuracy(&mut args),
        Some("accuracy-corpus") => corpus::run(&mut args, &workspace_root()?),
        Some("accuracy-ud") => accuracy_ud(&mut args),
        Some("dump-paradigms") => dump_paradigms(args.next()),
        Some("diff-paradigms") => {
            let before = args.next().ok_or("diff-paradigms needs BEFORE")?;
            let after = args.next().ok_or("diff-paradigms needs AFTER")?;
            diff_paradigms(Path::new(&before), Path::new(&after))
        }
        Some("examples") => examples(),
        Some("speed") => speed(),
        Some("guard-witnesses") => guard_witnesses(),
        Some("check-all") => check_all(),
        Some("help") | Some("-h") | Some("--help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command: {other}").into()),
    }
}

fn required_path_flag(
    args: &mut impl Iterator<Item = String>,
    expected: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let flag = args.next().ok_or(format!("expected {expected} PATH"))?;
    if flag != expected {
        return Err(format!("expected {expected}, found {flag}").into());
    }
    Ok(PathBuf::from(
        args.next()
            .ok_or(format!("expected a path after {expected}"))?,
    ))
}

fn extraction_report() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let markdown = fs::read_to_string(root.join("reports/extraction-coverage.md"))?;
    print!("{markdown}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct AccuracyReport {
    schema_version: u32,
    dictionary: DictionaryAccuracy,
    dictionary_metadata_e2e: MetadataE2eAccuracy,
    oov: OovAccuracy,
    extraction_exclusions: BTreeMap<String, usize>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct MetadataE2eAccuracy {
    source_verb_lexemes: usize,
    metadata_coverage_by_field: BTreeMap<String, usize>,
    development: MetadataFunnel,
    final_holdout: MetadataFunnel,
    development_by_system: BTreeMap<String, Slice>,
    final_by_system: BTreeMap<String, Slice>,
    development_by_cell: BTreeMap<String, Slice>,
    final_by_cell: BTreeMap<String, Slice>,
    development_by_generation_path: BTreeMap<String, Slice>,
    final_by_generation_path: BTreeMap<String, Slice>,
    development_by_present_class: BTreeMap<String, Slice>,
    final_by_present_class: BTreeMap<String, Slice>,
    development_by_formation: BTreeMap<String, Slice>,
    final_by_formation: BTreeMap<String, Slice>,
    development_by_source_policy: BTreeMap<String, Slice>,
    final_by_source_policy: BTreeMap<String, Slice>,
    development_by_analysis_kind: BTreeMap<String, Slice>,
    final_by_analysis_kind: BTreeMap<String, Slice>,
    development_by_lemma_frequency: BTreeMap<String, Slice>,
    final_by_lemma_frequency: BTreeMap<String, Slice>,
    skip_reasons: BTreeMap<String, usize>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct MetadataFunnel {
    compatible_target_cells: usize,
    unambiguous_target_cells: usize,
    metadata_records_found: usize,
    metadata_records_validated: usize,
    generation_attempts: usize,
    returned_forms: usize,
    diplomatic_top1_correct: usize,
    diplomatic_any_correct: usize,
    lookup_top1_correct: usize,
    lookup_any_correct: usize,
}

#[derive(Debug, Serialize)]
struct DictionaryAccuracy {
    lexemes: usize,
    cells: usize,
    variants: usize,
    reachable_variants: usize,
    exact_variant_order_cells: usize,
    primary_correct_cells: usize,
    ambiguous_bare_lemma_pos_pairs: usize,
    cells_by_source: BTreeMap<String, usize>,
    paradigm_cell_sets_correct: usize,
}

#[derive(Debug, Default, Serialize)]
struct OovAccuracy {
    development: BTreeMap<String, Slice>,
    test: BTreeMap<String, Slice>,
    development_by_cell: BTreeMap<String, Slice>,
    test_by_cell: BTreeMap<String, Slice>,
    skipped_cells: usize,
}

#[derive(Debug, Default, Clone, Serialize)]
struct Slice {
    correct: usize,
    normalized_correct: usize,
    total: usize,
}

fn accuracy(args: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let registry_path = accuracy_registry_path(args, &root)?;
    let report = evaluate_accuracy(&root, &registry_path)?;
    let json = serde_json::to_vec_pretty(&report)?;
    let markdown = accuracy_markdown(&report);
    fs::write(root.join("reports/accuracy.json"), json)?;
    fs::write(root.join("reports/accuracy.md"), markdown.as_bytes())?;
    print!("{markdown}");
    Ok(())
}

fn evaluate_accuracy(root: &Path, registry_path: &Path) -> Result<AccuracyReport, Box<dyn Error>> {
    let mut registry = load_registry(registry_path)?;
    if registry_path == root.join("data/extracted") {
        registry = registry_with_overrides(registry, &root.join("data/overrides.tsv"))?;
    }
    let dictionary = dictionary_accuracy(&registry)?;
    ensure_dictionary_integrity(&dictionary)?;
    let dictionary_metadata_e2e = dictionary_metadata_e2e_accuracy(&registry)?;
    ensure_metadata_e2e(&dictionary_metadata_e2e)?;
    let oov = oov_accuracy(&registry);
    let extraction: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("reports/extraction-coverage.json"))?)?;
    let extraction_exclusions = serde_json::from_value(
        extraction
            .get("dropped_by_reason")
            .cloned()
            .ok_or("extraction report has no dropped_by_reason")?,
    )?;
    Ok(AccuracyReport {
        schema_version: 4,
        dictionary,
        dictionary_metadata_e2e,
        oov,
        extraction_exclusions,
    })
}

fn ensure_metadata_e2e(report: &MetadataE2eAccuracy) -> Result<(), Box<dyn Error>> {
    for (name, funnel, minimum_availability) in [
        ("development", &report.development, 30_usize),
        ("final", &report.final_holdout, 35_usize),
    ] {
        if funnel.metadata_records_found > funnel.unambiguous_target_cells
            || funnel.metadata_records_validated > funnel.metadata_records_found
            || funnel.generation_attempts > funnel.metadata_records_validated
            || funnel.returned_forms > funnel.generation_attempts
            || funnel.diplomatic_top1_correct > funnel.returned_forms
            || funnel.diplomatic_any_correct > funnel.returned_forms
            || funnel.lookup_top1_correct > funnel.returned_forms
            || funnel.lookup_any_correct > funnel.returned_forms
        {
            return Err(
                format!("dictionary-metadata {name} funnel accounting is inconsistent").into(),
            );
        }
        if funnel.metadata_records_validated * 100
            < funnel.unambiguous_target_cells * minimum_availability
        {
            return Err(format!(
                "dictionary-metadata {name} availability fell below {minimum_availability}%"
            )
            .into());
        }
        if funnel.lookup_any_correct * 100 < funnel.returned_forms * 95 {
            return Err(format!(
                "dictionary-metadata {name} conditional lookup accuracy fell below 95%"
            )
            .into());
        }
    }
    Ok(())
}

fn ensure_dictionary_integrity(dictionary: &DictionaryAccuracy) -> Result<(), Box<dyn Error>> {
    if dictionary.reachable_variants != dictionary.variants {
        return Err("not every accepted dictionary variant reaches the public facade".into());
    }
    if dictionary.exact_variant_order_cells != dictionary.cells {
        return Err("source variant order changed in the public facade".into());
    }
    if dictionary.primary_correct_cells != dictionary.cells {
        return Err("source-order primary variants changed in the public facade".into());
    }
    if dictionary.paradigm_cell_sets_correct != dictionary.lexemes {
        return Err("dictionary paradigms and public cell getters disagree".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum MetadataTarget {
    Finite(FiniteVerbCell),
    Imperative(ImperativeCell),
    LParticiple(LParticipleCell),
    ParticipleCitation(ParticipleKind),
}

impl MetadataTarget {
    fn system(self) -> &'static str {
        match self {
            Self::Finite(cell) => match cell.tense {
                FiniteTense::Present => "present",
                FiniteTense::Imperfect => "imperfect",
                FiniteTense::Aorist => "aorist",
            },
            Self::Imperative(_) => "imperative",
            Self::LParticiple(_) => "l-participle",
            Self::ParticipleCitation(kind) => match kind {
                ParticipleKind::PresentActive => "present-active-participle",
                ParticipleKind::PresentPassive => "present-passive-participle",
                ParticipleKind::PastActive => "past-active-participle",
                ParticipleKind::PastPassive => "past-passive-participle",
            },
        }
    }
}

fn dictionary_metadata_e2e_accuracy(
    registry: &Registry,
) -> Result<MetadataE2eAccuracy, Box<dyn Error>> {
    let mut out = MetadataE2eAccuracy {
        source_verb_lexemes: registry
            .lexemes
            .iter()
            .filter(|row| row.pos == "verb")
            .count(),
        metadata_coverage_by_field: metadata_coverage(registry),
        ..MetadataE2eAccuracy::default()
    };
    let grouped = grouped_forms(registry);
    let mut forms_by_id: BTreeMap<&str, Vec<&FormRow>> = BTreeMap::new();
    for form in &registry.forms {
        forms_by_id
            .entry(form.lexeme_id.as_str())
            .or_default()
            .push(form);
    }
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut canonical_candidates: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for alias in &registry.aliases {
        if pos_by_id.get(alias.lexeme_id.as_str()) == Some(&"verb") {
            canonical_candidates
                .entry(alias.key.as_str())
                .or_default()
                .insert(alias.lexeme_id.as_str());
        }
    }

    for lexeme in registry.lexemes.iter().filter(|row| row.pos == "verb") {
        let held_final = fnv1a(lexeme.key.as_bytes()) % 5 == 0;
        let start = (lexeme.id.clone(), String::new());
        let end = (lexeme.id.clone(), "\u{10ffff}".to_string());
        for ((_id, feature), expected) in grouped.range(start..=end) {
            let Some(target) = parse_metadata_target(feature) else {
                continue;
            };
            let funnel = if held_final {
                &mut out.final_holdout
            } else {
                &mut out.development
            };
            funnel.compatible_target_cells += 1;
            if canonical_candidates
                .get(lexeme.key.as_str())
                .is_none_or(|ids| ids.len() != 1)
            {
                bump(&mut out.skip_reasons, "ambiguous-lemma");
                continue;
            }
            funnel.unambiguous_target_cells += 1;

            let target_spellings = expected
                .iter()
                .map(|row| row.form.as_str())
                .collect::<BTreeSet<_>>();
            let lexeme_forms = forms_by_id
                .get(lexeme.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or_default();
            let excluded_features = excluded_metadata_features(
                lexeme_forms.iter().copied(),
                feature,
                &target_spellings,
            );
            let mini = Registry {
                lexemes: vec![lexeme.clone()],
                aliases: Vec::new(),
                forms: lexeme_forms
                    .iter()
                    .filter(|row| !excluded_features.contains(row.feature.as_str()))
                    .map(|row| (*row).clone())
                    .collect(),
                verb_metadata: Vec::new(),
                overrides: Vec::new(),
            };
            let rows = verb_metadata::derive(&mini, &BTreeSet::new())?;
            if !rows.iter().any(|row| row.system == target.system()) {
                bump(
                    &mut out.skip_reasons,
                    "missing-principal-part-after-exclusion",
                );
                continue;
            }
            funnel.metadata_records_found += 1;
            let formation = metadata_formation_slice(&rows, target.system());
            let source_policy = metadata_source_policy_slice(&rows, target.system());
            let analysis_kind = if rows
                .iter()
                .filter(|row| row.system == target.system())
                .any(|row| row.analysis_rank > 0)
            {
                "regular-multiple-analyses"
            } else {
                "regular-single-analysis"
            };
            let fields = rows.into_iter().map(normalized_metadata_field);
            let metadata = match api_metadata::DictionaryVerbMetadata::from_normalized_fields(
                &lexeme.id,
                &lexeme.lemma,
                fields,
            ) {
                Ok(metadata) => metadata,
                Err(_) => {
                    bump(&mut out.skip_reasons, "invalid-filtered-metadata");
                    continue;
                }
            };
            funnel.metadata_records_validated += 1;
            funnel.generation_attempts += 1;
            let result = generate_metadata_target(&metadata, target);
            let forms = match result {
                Ok(forms) => forms,
                Err(error) => {
                    bump(&mut out.skip_reasons, metadata_error_reason(&error));
                    continue;
                }
            };
            funnel.returned_forms += 1;
            let expected_exact = expected
                .iter()
                .map(|row| row.form.as_str())
                .collect::<Vec<_>>();
            let expected_lookup = expected_exact
                .iter()
                .filter_map(|value| orthography::lookup_key(value).ok())
                .collect::<Vec<_>>();
            let returned_exact = forms
                .variants()
                .map(|variant| variant.text.as_str())
                .collect::<Vec<_>>();
            let returned_lookup = forms
                .variants()
                .filter_map(|variant| orthography::lookup_key(&variant.text).ok())
                .collect::<Vec<_>>();
            let top1 = returned_exact
                .first()
                .is_some_and(|value| expected_exact.contains(value));
            let any = returned_exact
                .iter()
                .any(|value| expected_exact.contains(value));
            let lookup_top1 = returned_lookup
                .first()
                .is_some_and(|value| expected_lookup.contains(value));
            let lookup_any = returned_lookup
                .iter()
                .any(|value| expected_lookup.contains(value));
            funnel.diplomatic_top1_correct += usize::from(top1);
            funnel.diplomatic_any_correct += usize::from(any);
            funnel.lookup_top1_correct += usize::from(lookup_top1);
            funnel.lookup_any_correct += usize::from(lookup_any);
            let (by_system, by_cell) = if held_final {
                (&mut out.final_by_system, &mut out.final_by_cell)
            } else {
                (&mut out.development_by_system, &mut out.development_by_cell)
            };
            score_metadata_slice(by_system, target.system(), any, lookup_any);
            score_metadata_slice(by_cell, feature, any, lookup_any);
            let generation_path = metadata_generation_path(&forms);
            let present_class = if lexeme.class.is_empty() {
                "unclassified"
            } else {
                lexeme.class.as_str()
            };
            let frequency = metadata_frequency_band(lexeme_forms.len());
            if held_final {
                score_metadata_slice(
                    &mut out.final_by_generation_path,
                    &generation_path,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.final_by_present_class,
                    present_class,
                    any,
                    lookup_any,
                );
                score_metadata_slice(&mut out.final_by_formation, &formation, any, lookup_any);
                score_metadata_slice(
                    &mut out.final_by_source_policy,
                    &source_policy,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.final_by_analysis_kind,
                    analysis_kind,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.final_by_lemma_frequency,
                    frequency,
                    any,
                    lookup_any,
                );
            } else {
                score_metadata_slice(
                    &mut out.development_by_generation_path,
                    &generation_path,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.development_by_present_class,
                    present_class,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.development_by_formation,
                    &formation,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.development_by_source_policy,
                    &source_policy,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.development_by_analysis_kind,
                    analysis_kind,
                    any,
                    lookup_any,
                );
                score_metadata_slice(
                    &mut out.development_by_lemma_frequency,
                    frequency,
                    any,
                    lookup_any,
                );
            }
        }
    }
    Ok(out)
}

fn metadata_formation_slice(
    rows: &[old_church_slavonic_extractor::schema::VerbMetadataRow],
    system: &str,
) -> String {
    let values = rows
        .iter()
        .filter(|row| row.system == system && matches!(row.field.as_str(), "formation" | "class"))
        .map(|row| row.value.as_str())
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        format!("{system}:no-formation")
    } else {
        format!(
            "{system}:{}",
            values.into_iter().collect::<Vec<_>>().join("+")
        )
    }
}

fn metadata_source_policy_slice(
    rows: &[old_church_slavonic_extractor::schema::VerbMetadataRow],
    system: &str,
) -> String {
    let features = rows
        .iter()
        .filter(|row| row.system == system)
        .map(|row| row.source_feature.as_str())
        .filter(|feature| !feature.is_empty())
        .collect::<BTreeSet<_>>();
    if features.is_empty() {
        "no-source-feature".to_string()
    } else {
        features.into_iter().collect::<Vec<_>>().join(" + ")
    }
}

fn metadata_generation_path(forms: &old_church_slavonic::FormSet) -> String {
    match forms.source() {
        old_church_slavonic::FormSource::DictionaryMetadataRule { rule_id } => {
            format!("dictionary-metadata-rule:{}", rule_id.code())
        }
        old_church_slavonic::FormSource::DictionaryMetadataAnalyses => {
            "dictionary-metadata-analyses".to_string()
        }
        old_church_slavonic::FormSource::ManualOverride => "manual-override".to_string(),
        old_church_slavonic::FormSource::DictionaryTable => "dictionary-table".to_string(),
        old_church_slavonic::FormSource::ExplicitMetadataRule { rule_id } => {
            format!("explicit-metadata-rule:{}", rule_id.code())
        }
        old_church_slavonic::FormSource::OovPrediction { rule_id } => {
            format!("oov-prediction:{}", rule_id.code())
        }
    }
}

fn metadata_frequency_band(frequency: usize) -> &'static str {
    match frequency {
        0 | 1 => "1",
        2..=10 => "2-10",
        11..=50 => "11-50",
        _ => "51+",
    }
}

fn excluded_metadata_features<'a>(
    forms: impl IntoIterator<Item = &'a FormRow>,
    target_feature: &str,
    target_spellings: &BTreeSet<&str>,
) -> BTreeSet<&'a str> {
    let equivalent_feature = match target_feature {
        "verb:finite:imperfect:2:sg" => Some("verb:finite:imperfect:3:sg"),
        "verb:finite:imperfect:3:sg" => Some("verb:finite:imperfect:2:sg"),
        "verb:finite:aorist:2:sg" => Some("verb:finite:aorist:3:sg"),
        "verb:finite:aorist:3:sg" => Some("verb:finite:aorist:2:sg"),
        "verb:imperative:2:sg" => Some("verb:imperative:3:sg"),
        "verb:imperative:3:sg" => Some("verb:imperative:2:sg"),
        _ => None,
    };
    forms
        .into_iter()
        .filter(|row| {
            row.feature == target_feature
                || equivalent_feature == Some(row.feature.as_str())
                || target_spellings.contains(row.form.as_str())
        })
        .map(|row| row.feature.as_str())
        .collect()
}

fn metadata_coverage(registry: &Registry) -> BTreeMap<String, usize> {
    let mut sets: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
    for key in [
        "aspect/aspect",
        "aspect/aspect=biaspectual",
        "aspect/aspect=imperfective",
        "aspect/aspect=perfective",
        "present/class",
        "present/class=IA1",
        "present/class=IA2",
        "present/class=II1",
        "present/class=II2",
        "present/class=II3",
        "present/stem",
        "present/first-singular-stem",
        "imperfect/stem",
        "imperfect/formation",
        "imperfect/formation=a",
        "imperfect/formation=yat-a",
        "imperfect/formation=palatalized-a",
        "imperfect/variant-policy",
        "imperfect/variant-policy=uncontracted-only",
        "aorist/stem",
        "aorist/formation",
        "aorist/formation=asigmatic",
        "aorist/formation=new",
        "aorist/formation=sigmatic-primary",
        "aorist/formation=sigmatic-secondary",
        "imperative/stem",
        "imperative/formation",
        "imperative/formation=i-series",
        "imperative/formation=yat-series",
        "l-participle/stem",
        "present-active-participle/stem",
        "present-active-participle/formation",
        "present-active-participle/formation=yusht-hard",
        "present-active-participle/formation=yusht-soft",
        "present-active-participle/formation=yesht-soft",
        "present-passive-participle/stem",
        "present-passive-participle/formation",
        "present-passive-participle/formation=im",
        "present-passive-participle/formation=em",
        "present-passive-participle/formation=om",
        "past-active-participle/stem",
        "past-active-participle/formation",
        "past-active-participle/formation=ush",
        "past-active-participle/formation=ish",
        "past-active-participle/formation=vush",
        "past-active-participle/formation=vush-after-j-deletion",
        "past-active-participle/formation=vush-after-ov-to-u",
        "past-passive-participle/stem",
        "past-passive-participle/formation",
        "past-passive-participle/formation=t",
        "past-passive-participle/formation=n",
        "past-passive-participle/formation=en",
    ] {
        sets.entry(key.to_string()).or_default();
    }
    for row in &registry.verb_metadata {
        sets.entry(format!("{}/{}", row.system, row.field))
            .or_default()
            .insert(row.lexeme_id.as_str());
        if matches!(
            row.field.as_str(),
            "formation" | "variant-policy" | "aspect" | "class"
        ) {
            sets.entry(format!("{}/{}={}", row.system, row.field, row.value))
                .or_default()
                .insert(row.lexeme_id.as_str());
        }
    }
    sets.into_iter()
        .map(|(field, ids)| (field, ids.len()))
        .collect()
}

fn normalized_metadata_field(
    row: old_church_slavonic_extractor::schema::VerbMetadataRow,
) -> api_metadata::NormalizedVerbMetadataField {
    api_metadata::NormalizedVerbMetadataField {
        system: row.system,
        analysis_rank: row.analysis_rank,
        field: row.field,
        value: row.value,
        provenance: row.provenance,
        source_feature: row.source_feature,
        source_form: row.source_form,
        crosscheck_features: row
            .crosscheck_features
            .split(" || ")
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect(),
        authority: row.authority,
    }
}

fn parse_metadata_target(feature: &str) -> Option<MetadataTarget> {
    if let Some(cell) = parse_finite_verb_cell(feature) {
        return Some(MetadataTarget::Finite(cell));
    }
    if let Some(cell) = parse_imperative_cell(feature) {
        return cell
            .is_supported()
            .then_some(MetadataTarget::Imperative(cell));
    }
    if let Some(cell) = parse_l_participle_cell(feature) {
        return Some(MetadataTarget::LParticiple(cell));
    }
    parse_participle_citation_kind(feature).map(MetadataTarget::ParticipleCitation)
}

fn generate_metadata_target(
    metadata: &api_metadata::DictionaryVerbMetadata,
    target: MetadataTarget,
) -> Result<old_church_slavonic::FormSet, InflectionError> {
    match target {
        MetadataTarget::Finite(cell) => {
            api_metadata::finite_verb_from_dictionary_metadata(metadata, cell)
        }
        MetadataTarget::Imperative(cell) => {
            api_metadata::imperative_from_dictionary_metadata(metadata, cell)
        }
        MetadataTarget::LParticiple(cell) => {
            api_metadata::l_participle_from_dictionary_metadata(metadata, cell)
        }
        MetadataTarget::ParticipleCitation(kind) => {
            api_metadata::participle_from_dictionary_metadata(
                metadata,
                ParticipleCell {
                    kind,
                    adjective: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                    },
                },
            )
        }
    }
}

fn metadata_error_reason(error: &InflectionError) -> &'static str {
    match error {
        InflectionError::MissingLexicalMetadata { .. } => "generation-missing-metadata",
        InflectionError::ContradictoryLexicalMetadata { .. } => "generation-contradictory-metadata",
        InflectionError::UnsupportedFormation { .. } => "represented-unsupported-formation",
        InflectionError::HistoricallyInvalidCell => "historically-invalid-cell",
        InflectionError::UnsupportedCell => "unsupported-cell",
        InflectionError::InvalidInput { .. } => "generation-invalid-metadata",
        InflectionError::UnknownLemma => "generation-unknown-lemma",
        InflectionError::AmbiguousLexeme { .. } => "generation-ambiguous-lemma",
    }
}

fn score_metadata_slice(
    destination: &mut BTreeMap<String, Slice>,
    key: &str,
    exact_any: bool,
    lookup_any: bool,
) {
    let slice = destination.entry(key.to_string()).or_default();
    slice.total += 1;
    slice.correct += usize::from(exact_any);
    slice.normalized_correct += usize::from(lookup_any);
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_default() += 1;
}

fn accuracy_registry_path(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let Some(flag) = args.next() else {
        return Ok(root.join("data/extracted"));
    };
    if flag != "--dump" {
        return Err(format!("accuracy expected --dump PATH, found {flag}").into());
    }
    let path = PathBuf::from(args.next().ok_or("accuracy --dump needs a path")?);
    if args.next().is_some() {
        return Err("accuracy received unexpected extra arguments".into());
    }
    if path.is_dir() {
        return Ok(path);
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("data/extracted/source.json"))?)?;
    let expected_bytes = metadata["bytes"].as_u64();
    if Some(fs::metadata(&path)?.len()) != expected_bytes {
        return Err(
            "raw dump does not match the committed source byte length; refresh first".into(),
        );
    }
    let expected_sha = metadata["sha256"]
        .as_str()
        .ok_or("committed source metadata has no sha256")?;
    let mut source = File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if format!("{:x}", hasher.finalize()) != expected_sha {
        return Err("raw dump SHA-256 does not match the committed source; refresh first".into());
    }
    Ok(root.join("data/extracted"))
}

fn dictionary_accuracy(registry: &Registry) -> Result<DictionaryAccuracy, Box<dyn Error>> {
    let grouped = grouped_forms(registry);
    let mut reachable = 0;
    let mut ordered = 0;
    let mut primary = 0;
    let mut cells_by_source = BTreeMap::new();
    for ((id, feature), expected) in &grouped {
        let actual = public_cell_by_id(id, feature)?;
        let expected_values = expected
            .iter()
            .map(|row| (row.form.as_str(), row.romanization.as_str()))
            .collect::<Vec<_>>();
        let actual_values = actual
            .variants()
            .map(|variant| {
                (
                    variant.text.as_str(),
                    variant.romanization.as_deref().unwrap_or(""),
                )
            })
            .collect::<Vec<_>>();
        reachable += expected_values
            .iter()
            .filter(|expected| actual_values.contains(expected))
            .count();
        ordered += usize::from(expected_values == actual_values);
        primary += usize::from(expected_values.first() == actual_values.first());
        let source = match actual.source() {
            old_church_slavonic::FormSource::DictionaryTable => "dictionary-table",
            old_church_slavonic::FormSource::ManualOverride => "manual-override",
            old_church_slavonic::FormSource::DictionaryMetadataRule { .. } => {
                "dictionary-metadata-rule"
            }
            old_church_slavonic::FormSource::DictionaryMetadataAnalyses => {
                "dictionary-metadata-analyses"
            }
            old_church_slavonic::FormSource::ExplicitMetadataRule { .. } => {
                "explicit-metadata-rule"
            }
            old_church_slavonic::FormSource::OovPrediction { .. } => "oov-prediction",
        };
        *cells_by_source.entry(source.to_string()).or_insert(0) += 1;
    }
    let mut alias_pos: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    for alias in &registry.aliases {
        if let Some(pos) = pos_by_id.get(alias.lexeme_id.as_str()) {
            alias_pos
                .entry((alias.key.as_str(), *pos))
                .or_default()
                .insert(alias.lexeme_id.as_str());
        }
    }
    let mut paradigm_cell_sets_correct = 0;
    for lexeme in &registry.lexemes {
        let paradigm = raw_features::dictionary_paradigm_by_id(&lexeme.id)?;
        let start = (lexeme.id.clone(), String::new());
        let end = (lexeme.id.clone(), "\u{10ffff}".to_string());
        let expected = grouped
            .range(start..=end)
            .map(|((_id, feature), _)| feature.as_str())
            .collect::<BTreeSet<_>>();
        let actual = paradigm
            .iter()
            .map(|(feature, _)| feature)
            .collect::<BTreeSet<_>>();
        paradigm_cell_sets_correct += usize::from(expected == actual);
    }
    Ok(DictionaryAccuracy {
        lexemes: registry.lexemes.len(),
        cells: grouped.len(),
        variants: registry.forms.len(),
        reachable_variants: reachable,
        exact_variant_order_cells: ordered,
        primary_correct_cells: primary,
        ambiguous_bare_lemma_pos_pairs: alias_pos.values().filter(|ids| ids.len() > 1).count(),
        cells_by_source,
        paradigm_cell_sets_correct,
    })
}

fn public_cell_by_id(
    id: &str,
    feature: &str,
) -> Result<old_church_slavonic::FormSet, Box<dyn Error>> {
    if let Some(cell) = parse_noun_cell(feature) {
        return Ok(by_id::noun_by_id(id, cell)?);
    }
    if let Some(cell) = parse_adjective_cell(feature) {
        return Ok(by_id::adjective_by_id(id, cell)?);
    }
    if feature == "adj:comparative:citation" {
        return Ok(by_id::adjective_comparatives_by_id(id)?);
    }
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] => Ok(by_id::finite_verb_by_id(
            id,
            FiniteVerbCell {
                tense: parse_tense(tense).ok_or("invalid finite tense")?,
                person: parse_person(person).ok_or("invalid finite person")?,
                number: parse_number(number).ok_or("invalid finite number")?,
            },
        )?),
        ["verb", "imperative", person, number] => Ok(by_id::imperative_by_id(
            id,
            ImperativeCell {
                person: parse_person(person).ok_or("invalid imperative person")?,
                number: parse_number(number).ok_or("invalid imperative number")?,
            },
        )?),
        ["verb", "l-participle", gender, number] => Ok(by_id::l_participle_by_id(
            id,
            LParticipleCell {
                gender: parse_gender_code(gender).ok_or("invalid l-participle gender")?,
                number: parse_number(number).ok_or("invalid l-participle number")?,
            },
        )?),
        ["verb", "participle", kind, "citation"] => Ok(by_id::participle_citation_by_id(
            id,
            parse_participle_kind(kind).ok_or("invalid participle kind")?,
        )?),
        ["verb", "infinitive"] => Ok(by_id::infinitive_by_id(id)?),
        ["verb", "supine"] => Ok(by_id::supine_by_id(id)?),
        ["verb", "verbal-noun"] => Ok(by_id::verbal_noun_by_id(id)?),
        ["decl", pos, case, number, rest @ ..] => {
            let part_of_speech = match *pos {
                "pron" => PartOfSpeech::Pronoun,
                "num" => PartOfSpeech::Numeral,
                "det" => PartOfSpeech::Determiner,
                _ => return Err("invalid closed-class part of speech".into()),
            };
            let mut gender = None;
            let mut person = None;
            for value in rest.iter().copied() {
                if let Some(value) = parse_gender_code(value) {
                    gender = Some(value);
                } else if let Some(value) = parse_person(value) {
                    person = Some(value);
                } else {
                    return Err(format!("invalid closed-class feature segment: {value}").into());
                }
            }
            Ok(raw_features::closed_class_by_id(
                id,
                part_of_speech,
                ClosedClassCell {
                    case: parse_case(case).ok_or("invalid closed-class case")?,
                    number: parse_number(number).ok_or("invalid closed-class number")?,
                    gender,
                    person,
                },
            )?)
        }
        _ => Err(format!("no typed public resolver for accepted feature: {feature}").into()),
    }
}

fn parse_tense(value: &str) -> Option<FiniteTense> {
    match value {
        "present" => Some(FiniteTense::Present),
        "imperfect" => Some(FiniteTense::Imperfect),
        "aorist" => Some(FiniteTense::Aorist),
        _ => None,
    }
}

fn parse_person(value: &str) -> Option<Person> {
    match value {
        "1" => Some(Person::First),
        "2" => Some(Person::Second),
        "3" => Some(Person::Third),
        _ => None,
    }
}

fn parse_gender_code(value: &str) -> Option<Gender> {
    match value {
        "m" => Some(Gender::Masculine),
        "f" => Some(Gender::Feminine),
        "n" => Some(Gender::Neuter),
        _ => None,
    }
}

fn parse_participle_kind(value: &str) -> Option<ParticipleKind> {
    match value {
        "present-active" => Some(ParticipleKind::PresentActive),
        "present-passive" => Some(ParticipleKind::PresentPassive),
        "past-active" => Some(ParticipleKind::PastActive),
        "past-passive" => Some(ParticipleKind::PastPassive),
        _ => None,
    }
}

fn oov_accuracy(registry: &Registry) -> OovAccuracy {
    let grouped = grouped_forms(registry);
    let mut out = OovAccuracy::default();
    for lexeme in &registry.lexemes {
        let test = fnv1a(lexeme.key.as_bytes()) % 5 == 0;
        let (destination, by_cell) = if test {
            (&mut out.test, &mut out.test_by_cell)
        } else {
            (&mut out.development, &mut out.development_by_cell)
        };
        match lexeme.pos.as_str() {
            "noun" => evaluate_oov_noun(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            "adj" => evaluate_oov_adjective(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            "verb" => evaluate_oov_verb(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            _ => {}
        }
    }
    out
}

fn evaluate_oov_noun(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = match row.class.as_str() {
        "o-m-hard" => NounClass::OMasculineHard,
        "o-n-hard" => NounClass::ONeuterHard,
        "a-hard" => NounClass::AHard,
        "jo-m-soft" => NounClass::JoMasculineSoft,
        "jo-n-soft" => NounClass::JoNeuterSoft,
        "ja-soft" => NounClass::JaSoft,
        "i-f" => NounClass::IFeminine,
        "i-m" => NounClass::IMasculine,
        "u-m" => NounClass::UMasculine,
        "n-m" => NounClass::NMasculine,
        "n-n" => NounClass::NNeuter,
        "nt-n" => NounClass::NtNeuter,
        "r-n" => NounClass::RStem,
        "s-n" => NounClass::SNeuter,
        "v-f" => NounClass::VFeminine,
        _ => {
            *skipped +=
                count_lexeme_features(row, grouped, |feature| parse_noun_cell(feature).is_some());
            return;
        }
    };
    let gender = match row.gender.as_str() {
        "m" => Gender::Masculine,
        "f" => Gender::Feminine,
        "n" => Gender::Neuter,
        _ => {
            *skipped +=
                count_lexeme_features(row, grouped, |feature| parse_noun_cell(feature).is_some());
            return;
        }
    };
    let animacy = match row.animacy.as_str() {
        "an" => Some(Animacy::Animate),
        "in" => Some(Animacy::Inanimate),
        _ => None,
    };
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        let Some(cell) = parse_noun_cell(feature) else {
            continue;
        };
        if gender == Gender::Masculine && cell.case == Case::Accusative && animacy.is_none() {
            *skipped += 1;
            continue;
        }
        let lexeme = NounLexeme {
            lemma: row.lemma.clone(),
            class,
            gender,
            animacy: animacy.unwrap_or(Animacy::Inanimate),
            number_restriction: parse_restriction(&row.number_restriction),
        };
        let Ok(predicted) = old_church_slavonic_core::noun::decline(&lexeme, cell) else {
            *skipped += 1;
            continue;
        };
        score_prediction(
            destination,
            &row.class,
            by_cell,
            &format!("noun/{}/{feature}", row.class),
            expected,
            &predicted.text,
        );
    }
}

fn evaluate_oov_adjective(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = match row.class.as_str() {
        "adj-hard" => AdjectiveClass::Hard,
        "adj-soft" => AdjectiveClass::Soft,
        _ => {
            *skipped += count_lexeme_features(row, grouped, |feature| {
                parse_adjective_cell(feature).is_some()
            });
            return;
        }
    };
    let lexeme = AdjectiveLexeme {
        lemma: row.lemma.clone(),
        class,
    };
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        let Some(cell) = parse_adjective_cell(feature) else {
            continue;
        };
        let Ok(predicted) = old_church_slavonic_core::adjective::decline(&lexeme, cell) else {
            *skipped += 1;
            continue;
        };
        let rule_slice = format!(
            "adj-{}-{}",
            if class == AdjectiveClass::Hard {
                "hard"
            } else {
                "soft"
            },
            cell.form.code()
        );
        score_prediction(
            destination,
            &rule_slice,
            by_cell,
            &format!("adj/{rule_slice}/{feature}"),
            expected,
            &predicted.text,
        );
    }
}

fn evaluate_oov_verb(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = parse_productive_verb_class(&row.class);
    let present_stem = class.and_then(|class| {
        grouped
            .get(&(row.id.clone(), "verb:finite:present:2:sg".to_string()))
            .and_then(|forms| {
                forms
                    .iter()
                    .find_map(|form| derive_present_stem(class, &form.form))
            })
    });
    let aorist_stem = grouped
        .get(&(row.id.clone(), "verb:l-participle:m:sg".to_string()))
        .and_then(|forms| {
            forms
                .iter()
                .find_map(|form| derive_l_participle_stem(&form.form))
        });
    let imperfect_metadata = grouped
        .get(&(row.id.clone(), "verb:finite:imperfect:1:sg".to_string()))
        .and_then(|forms| {
            forms
                .iter()
                .find_map(|form| derive_imperfect_metadata(&form.form))
        });
    let new_aorist_stem = grouped
        .get(&(row.id.clone(), "verb:finite:aorist:1:sg".to_string()))
        .and_then(|forms| {
            forms
                .iter()
                .find_map(|form| derive_new_aorist_stem(&form.form))
        });
    let imperative_stem = grouped
        .get(&(row.id.clone(), "verb:imperative:2:sg".to_string()))
        .and_then(|forms| {
            forms
                .iter()
                .find_map(|form| derive_imperative_stem(&form.form))
        });
    let mut lexeme = VerbLexeme::new(row.lemma.clone(), class.unwrap_or(VerbClass::Irregular));
    lexeme.stems.present = present_stem;
    lexeme.stems.aorist = new_aorist_stem.clone().or(aorist_stem);
    if let Some((stem, formation)) = imperfect_metadata {
        lexeme.stems.imperfect = Some(stem);
        lexeme.formations.imperfect = Some(formation);
        lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
    }
    if new_aorist_stem.is_some() {
        lexeme.formations.aorist = Some(AoristFormation::New);
    }
    if let Some(stem) = imperative_stem {
        lexeme.stems.imperative = Some(stem);
        lexeme.formations.imperative = class.map(|class| {
            if matches!(class, VerbClass::II1 | VerbClass::II2 | VerbClass::II3) {
                ImperativeFormation::ISeries
            } else {
                ImperativeFormation::YatSeries
            }
        });
    }
    if let (Some(class), Some(stem)) = (class, lexeme.stems.present.clone()) {
        lexeme.stems.present_active_participle = Some(stem.clone());
        lexeme.formations.present_active_participle =
            Some(if matches!(class, VerbClass::IA1 | VerbClass::IA2) {
                PresentActiveParticipleFormation::YushtHard
            } else {
                PresentActiveParticipleFormation::YeshtSoft
            });
        lexeme.stems.present_passive_participle = Some(stem);
        lexeme.formations.present_passive_participle =
            Some(if matches!(class, VerbClass::IA1 | VerbClass::IA2) {
                PresentPassiveParticipleFormation::Om
            } else {
                PresentPassiveParticipleFormation::Im
            });
    }
    if let Some(stem) = lexeme.stems.aorist.clone() {
        lexeme.stems.past_active_participle = Some(stem.clone());
        lexeme.formations.past_active_participle = Some(if stem.ends_with(['а', 'ѣ', 'и']) {
            PastActiveParticipleFormation::Vush
        } else {
            PastActiveParticipleFormation::Ush
        });
        if matches!(class, Some(VerbClass::IA1 | VerbClass::IA2)) {
            lexeme.stems.past_passive_participle = Some(stem);
            lexeme.formations.past_passive_participle = Some(PastPassiveParticipleFormation::En);
        }
    }
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        if let Some(cell) = parse_finite_verb_cell(feature) {
            let metadata_cell = match cell.tense {
                FiniteTense::Present => {
                    cell.person == Person::Second && cell.number == Number::Singular
                }
                FiniteTense::Imperfect | FiniteTense::Aorist => {
                    cell.person == Person::First && cell.number == Number::Singular
                }
            };
            if metadata_cell {
                continue;
            }
            if cell.tense == FiniteTense::Present && class.is_none() {
                *skipped += 1;
                continue;
            }
            let Ok(predicted) = old_church_slavonic_core::verb::finite(&lexeme, cell) else {
                *skipped += 1;
                continue;
            };
            let rule_slice = match cell.tense {
                FiniteTense::Present => {
                    let Some(class) = class else {
                        *skipped += 1;
                        continue;
                    };
                    format!("verb-{}-present", class.code())
                }
                FiniteTense::Imperfect => "verb-imperfect".to_string(),
                FiniteTense::Aorist => "verb-aorist-new".to_string(),
            };
            score_prediction(
                destination,
                &rule_slice,
                by_cell,
                &format!("verb/{rule_slice}/{feature}"),
                expected,
                &predicted.text,
            );
            continue;
        }
        if let Some(cell) = parse_imperative_cell(feature) {
            // The 2sg supplies the imperative stem and the 3sg is the same
            // morphological form; neither is scored against its own metadata.
            if cell.number == Number::Singular {
                continue;
            }
            match old_church_slavonic_core::verb::imperative(&lexeme, cell) {
                Ok(predicted) => score_prediction(
                    destination,
                    "verb-imperative",
                    by_cell,
                    &format!("verb/verb-imperative/{feature}"),
                    expected,
                    &predicted.text,
                ),
                Err(_) => *skipped += 1,
            }
            continue;
        }
        if let Some(kind) = parse_participle_citation_kind(feature) {
            let cell = ParticipleCell {
                kind,
                adjective: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                },
            };
            match old_church_slavonic_core::verb::participle(&lexeme, cell) {
                Ok(predicted) => {
                    let rule_slice = participle_rule_slice(&lexeme, kind);
                    score_prediction(
                        destination,
                        rule_slice,
                        by_cell,
                        &format!("verb/{rule_slice}/{feature}"),
                        expected,
                        &predicted.text,
                    );
                }
                Err(_) => *skipped += 1,
            }
            continue;
        }
        if feature == "verb:infinitive" {
            match old_church_slavonic_core::verb::infinitive(&lexeme) {
                Ok(predicted) => score_prediction(
                    destination,
                    "verb-infinitive",
                    by_cell,
                    "verb/verb-infinitive/verb:infinitive",
                    expected,
                    &predicted.text,
                ),
                Err(_) => *skipped += 1,
            }
            continue;
        }
        if feature == "verb:supine" {
            match old_church_slavonic_core::verb::supine(&lexeme) {
                Ok(predicted) => score_prediction(
                    destination,
                    "verb-supine",
                    by_cell,
                    "verb/verb-supine/verb:supine",
                    expected,
                    &predicted.text,
                ),
                Err(_) => *skipped += 1,
            }
            continue;
        }
        let Some(cell) = parse_l_participle_cell(feature) else {
            continue;
        };
        if cell.gender == Gender::Masculine && cell.number == Number::Singular {
            continue;
        }
        match old_church_slavonic_core::verb::l_participle(&lexeme, cell) {
            Ok(predicted) => score_prediction(
                destination,
                "verb-l-participle",
                by_cell,
                &format!("verb/verb-l-participle/{feature}"),
                expected,
                &predicted.text,
            ),
            Err(_) => *skipped += 1,
        }
    }
}

fn parse_productive_verb_class(value: &str) -> Option<VerbClass> {
    match value {
        "IA1" => Some(VerbClass::IA1),
        "IA2" => Some(VerbClass::IA2),
        "II1" => Some(VerbClass::II1),
        "II2" => Some(VerbClass::II2),
        "II3" => Some(VerbClass::II3),
        _ => None,
    }
}

fn derive_present_stem(class: VerbClass, second_singular: &str) -> Option<String> {
    let ending = match class {
        VerbClass::IA1 | VerbClass::IA2 => "еши",
        VerbClass::II1 | VerbClass::II2 | VerbClass::II3 => "иши",
        _ => return None,
    };
    second_singular
        .strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn derive_l_participle_stem(masculine_singular: &str) -> Option<String> {
    masculine_singular
        .strip_suffix("лъ")
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn derive_imperfect_metadata(form: &str) -> Option<(String, ImperfectFormation)> {
    if let Some(stem) = form.strip_suffix("ѣахъ").filter(|stem| !stem.is_empty()) {
        return Some((stem.to_string(), ImperfectFormation::YatA));
    }
    form.strip_suffix("ахъ")
        .filter(|stem| !stem.is_empty())
        .map(|stem| (stem.to_string(), ImperfectFormation::A))
}

fn derive_new_aorist_stem(form: &str) -> Option<String> {
    form.strip_suffix("охъ")
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn derive_imperative_stem(form: &str) -> Option<String> {
    form.strip_suffix('и')
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn parse_finite_verb_cell(feature: &str) -> Option<FiniteVerbCell> {
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] => Some(FiniteVerbCell {
            tense: parse_tense(tense)?,
            person: parse_person(person)?,
            number: parse_number(number)?,
        }),
        _ => None,
    }
}

fn parse_l_participle_cell(feature: &str) -> Option<LParticipleCell> {
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "l-participle", gender, number] => Some(LParticipleCell {
            gender: parse_gender_code(gender)?,
            number: parse_number(number)?,
        }),
        _ => None,
    }
}

fn parse_imperative_cell(feature: &str) -> Option<ImperativeCell> {
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "imperative", person, number] => Some(ImperativeCell {
            person: parse_person(person)?,
            number: parse_number(number)?,
        }),
        _ => None,
    }
}

fn parse_participle_citation_kind(feature: &str) -> Option<ParticipleKind> {
    match feature {
        "verb:participle:present-active:citation" => Some(ParticipleKind::PresentActive),
        "verb:participle:present-passive:citation" => Some(ParticipleKind::PresentPassive),
        "verb:participle:past-active:citation" => Some(ParticipleKind::PastActive),
        "verb:participle:past-passive:citation" => Some(ParticipleKind::PastPassive),
        _ => None,
    }
}

fn participle_rule_slice(lexeme: &VerbLexeme, kind: ParticipleKind) -> &'static str {
    match kind {
        ParticipleKind::PresentActive => match lexeme.formations.present_active_participle {
            Some(PresentActiveParticipleFormation::YushtHard) => {
                "verb-present-active-participle-yusht-hard"
            }
            Some(PresentActiveParticipleFormation::YushtSoft) => {
                "verb-present-active-participle-yusht-soft"
            }
            Some(PresentActiveParticipleFormation::YeshtSoft) => {
                "verb-present-active-participle-yesht-soft"
            }
            None => "verb-present-active-participle-missing-formation",
        },
        ParticipleKind::PresentPassive => match lexeme.formations.present_passive_participle {
            Some(PresentPassiveParticipleFormation::Im) => "verb-present-passive-participle-im",
            Some(PresentPassiveParticipleFormation::Em) => "verb-present-passive-participle-em",
            Some(PresentPassiveParticipleFormation::Om) => "verb-present-passive-participle-om",
            None => "verb-present-passive-participle-missing-formation",
        },
        ParticipleKind::PastActive => match lexeme.formations.past_active_participle {
            Some(PastActiveParticipleFormation::Ush) => "verb-past-active-participle-ush",
            Some(PastActiveParticipleFormation::Ish) => "verb-past-active-participle-ish",
            Some(PastActiveParticipleFormation::VushAfterJDeletion) => {
                "verb-past-active-participle-vush-j-deletion"
            }
            Some(PastActiveParticipleFormation::VushAfterOvToU) => {
                "verb-past-active-participle-vush-ov-to-u"
            }
            Some(PastActiveParticipleFormation::Vush) => "verb-past-active-participle-vush",
            None => "verb-past-active-participle-missing-formation",
        },
        ParticipleKind::PastPassive => match lexeme.formations.past_passive_participle {
            Some(PastPassiveParticipleFormation::T) => "verb-past-passive-participle-t",
            Some(PastPassiveParticipleFormation::N) => "verb-past-passive-participle-n",
            Some(PastPassiveParticipleFormation::En) => "verb-past-passive-participle-en",
            None => "verb-past-passive-participle-missing-formation",
        },
    }
}

fn count_lexeme_features(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    predicate: impl Fn(&str) -> bool,
) -> usize {
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    grouped
        .range(start..=end)
        .filter(|((_id, feature), _)| predicate(feature))
        .count()
}

fn score_prediction(
    destination: &mut BTreeMap<String, Slice>,
    rule_slice: &str,
    by_cell: &mut BTreeMap<String, Slice>,
    cell_slice: &str,
    expected: &[&FormRow],
    predicted: &str,
) {
    let exact = expected.iter().any(|form| form.form == predicted);
    let normalized = expected
        .iter()
        .any(|form| normalized_equal(&form.form, predicted));
    for slice in [
        destination.entry(rule_slice.to_string()).or_default(),
        by_cell.entry(cell_slice.to_string()).or_default(),
    ] {
        slice.total += 1;
        slice.correct += usize::from(exact);
        slice.normalized_correct += usize::from(normalized);
    }
}

fn normalized_equal(left: &str, right: &str) -> bool {
    orthography::lookup_key(left).ok() == orthography::lookup_key(right).ok()
}

fn grouped_forms(registry: &Registry) -> BTreeMap<(String, String), Vec<&FormRow>> {
    let mut grouped: BTreeMap<(String, String), Vec<&FormRow>> = BTreeMap::new();
    for row in &registry.forms {
        grouped
            .entry((row.lexeme_id.clone(), row.feature.clone()))
            .or_default()
            .push(row);
    }
    for rows in grouped.values_mut() {
        rows.sort_by_key(|row| row.rank);
    }
    grouped
}

fn parse_noun_cell(feature: &str) -> Option<NounCell> {
    let mut parts = feature.split(':');
    if parts.next()? != "noun" {
        return None;
    }
    let case = parse_case(parts.next()?)?;
    let number = parse_number(parts.next()?)?;
    parts.next().is_none().then_some(NounCell { case, number })
}

fn parse_adjective_cell(feature: &str) -> Option<AdjectiveCell> {
    let mut parts = feature.split(':');
    if parts.next()? != "adj" {
        return None;
    }
    let form = match parts.next()? {
        "short" => AdjectiveForm::Short,
        "long" => AdjectiveForm::Long,
        _ => return None,
    };
    let case = parse_case(parts.next()?)?;
    let number = parse_number(parts.next()?)?;
    let gender = match parts.next()? {
        "m" => Gender::Masculine,
        "f" => Gender::Feminine,
        "n" => Gender::Neuter,
        _ => return None,
    };
    let animacy = match parts.next()? {
        "an" => Animacy::Animate,
        "in" => Animacy::Inanimate,
        _ => return None,
    };
    parts.next().is_none().then_some(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form,
    })
}

fn parse_case(value: &str) -> Option<Case> {
    match value {
        "nom" => Some(Case::Nominative),
        "gen" => Some(Case::Genitive),
        "dat" => Some(Case::Dative),
        "acc" => Some(Case::Accusative),
        "ins" => Some(Case::Instrumental),
        "loc" => Some(Case::Locative),
        "voc" => Some(Case::Vocative),
        _ => None,
    }
}

fn parse_number(value: &str) -> Option<Number> {
    match value {
        "sg" => Some(Number::Singular),
        "du" => Some(Number::Dual),
        "pl" => Some(Number::Plural),
        _ => None,
    }
}

fn parse_restriction(value: &str) -> NumberRestriction {
    match value {
        "sg" => NumberRestriction::SingularOnly,
        "du" => NumberRestriction::DualOnly,
        "pl" => NumberRestriction::PluralOnly,
        _ => NumberRestriction::All,
    }
}

fn accuracy_markdown(report: &AccuracyReport) -> String {
    let dictionary = &report.dictionary;
    let mut out = String::from("# Accuracy\n\n");
    out.push_str("Dictionary round-trip and OOV prediction are separate measurements.\n\n");
    out.push_str(
        "The OOV split is lemma-level: 64-bit FNV-1a of the shared normalized lemma key, \
modulo 5. Residue 0 is the fixed held-out final-evaluation partition; residues 1-4 \
are development. Homographs and parts of speech sharing a lemma key therefore cannot \
cross partitions. The held-out partition is deterministic, not cryptographically \
sealed, and must not be used for rule tuning.\n\n",
    );
    out.push_str("## Dictionary registry round-trip\n\n");
    out.push_str("| Metric | Value |\n|---|---:|\n");
    out.push_str(&format!("| lexemes | {} |\n", dictionary.lexemes));
    out.push_str(&format!("| cells | {} |\n", dictionary.cells));
    out.push_str(&format!("| variants | {} |\n", dictionary.variants));
    out.push_str(&format!(
        "| reachable variants | {} / {} |\n",
        dictionary.reachable_variants, dictionary.variants
    ));
    out.push_str(&format!(
        "| exact variant-order cells | {} / {} |\n",
        dictionary.exact_variant_order_cells, dictionary.cells
    ));
    out.push_str(&format!(
        "| primary-correct cells | {} / {} |\n",
        dictionary.primary_correct_cells, dictionary.cells
    ));
    out.push_str(&format!(
        "| ambiguous bare lemma/POS pairs | {} |\n",
        dictionary.ambiguous_bare_lemma_pos_pairs
    ));
    out.push_str(&format!(
        "| complete dictionary paradigm key sets | {} / {} |\n\n",
        dictionary.paradigm_cell_sets_correct, dictionary.lexemes
    ));
    out.push_str("Cells by public provenance:\n\n");
    for (source, count) in &dictionary.cells_by_source {
        out.push_str(&format!("- `{source}`: {count}\n"));
    }
    out.push('\n');
    let e2e = &report.dictionary_metadata_e2e;
    out.push_str("## Leakage-controlled dictionary-metadata generation\n\n");
    out.push_str(
        "This primary fallback score removes the target feature, an equivalent 2sg/3sg \
finite or imperative feature, and every same-spelling dictionary feature before rebuilding metadata. It then calls the public \
dictionary-metadata resolver; exact table lookup is unavailable to this path. \
Development and final lemmas use the same frozen modulo-five partition as OOV.\n\n",
    );
    out.push_str(&format!(
        "Source dictionary verb lexemes: {}.\n\n",
        e2e.source_verb_lexemes
    ));
    out.push_str(
        "### Metadata coverage by field\n\n| Field or declared value | Lexemes |\n|---|---:|\n",
    );
    for (field, count) in &e2e.metadata_coverage_by_field {
        out.push_str(&format!("| `{field}` | {count} |\n"));
    }
    out.push_str("\n### Held-cell stage funnel\n\n");
    out.push_str("| Stage | Development | Final holdout |\n|---|---:|---:|\n");
    for (label, value) in metadata_funnel_rows(&e2e.development, &e2e.final_holdout) {
        out.push_str(&format!("| {label} | {} | {} |\n", value.0, value.1));
    }
    out.push_str("\nThe slice tables below report diplomatic-any in `Exact` and shared NFC/lowercase-any in `NFC/lowercase`; top-1 remains separate in the funnel.\n\n");
    for (title, slices) in [
        ("Development by system", &e2e.development_by_system),
        ("Final holdout by system", &e2e.final_by_system),
        ("Development by complete cell", &e2e.development_by_cell),
        ("Final holdout by complete cell", &e2e.final_by_cell),
        (
            "Development by generation path",
            &e2e.development_by_generation_path,
        ),
        (
            "Final holdout by generation path",
            &e2e.final_by_generation_path,
        ),
        (
            "Development by present class",
            &e2e.development_by_present_class,
        ),
        (
            "Final holdout by present class",
            &e2e.final_by_present_class,
        ),
        ("Development by formation", &e2e.development_by_formation),
        ("Final holdout by formation", &e2e.final_by_formation),
        (
            "Development by metadata source-cell policy",
            &e2e.development_by_source_policy,
        ),
        (
            "Final holdout by metadata source-cell policy",
            &e2e.final_by_source_policy,
        ),
        (
            "Development by regular/analysis kind",
            &e2e.development_by_analysis_kind,
        ),
        (
            "Final holdout by regular/analysis kind",
            &e2e.final_by_analysis_kind,
        ),
        (
            "Development by lemma dictionary frequency",
            &e2e.development_by_lemma_frequency,
        ),
        (
            "Final holdout by lemma dictionary frequency",
            &e2e.final_by_lemma_frequency,
        ),
    ] {
        out.push_str(&format!(
            "#### {title}\n\n| Slice | Exact | NFC/lowercase | Returned |\n|---|---:|---:|---:|\n"
        ));
        for (key, slice) in slices {
            out.push_str(&format!(
                "| `{key}` | {} | {} | {} |\n",
                slice.correct, slice.normalized_correct, slice.total
            ));
        }
        out.push('\n');
    }
    out.push_str("Skip and failure reasons:\n\n");
    for (reason, count) in &e2e.skip_reasons {
        out.push_str(&format!("- `{reason}`: {count}\n"));
    }
    out.push('\n');
    out.push_str(
        "The legacy oracle/core OOV diagnostic below may use the 2nd-singular present, masculine-singular \
l-participle, 1st-singular imperfect/new aorist, or 2nd-singular imperative. Every \
metadata source cell and equivalent duplicate target is excluded. Participle citation \
targets use only those independently held principal parts plus declared class/formation \
policies; they are never used to derive themselves.\n\n",
    );
    for (title, slices, by_cell) in [
        (
            "Development OOV",
            &report.oov.development,
            &report.oov.development_by_cell,
        ),
        ("Held-out OOV", &report.oov.test, &report.oov.test_by_cell),
    ] {
        out.push_str(&format!(
            "## {title}\n\n| Rule slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |\n|---|---:|---:|---:|---:|---:|\n"
        ));
        let mut macro_exact = 0.0;
        let mut macro_normalized = 0.0;
        for (class, slice) in slices {
            let rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.correct as f64 / slice.total as f64
            };
            let normalized_rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.normalized_correct as f64 / slice.total as f64
            };
            macro_exact += rate;
            macro_normalized += normalized_rate;
            out.push_str(&format!(
                "| `{class}` | {} | {} | {} | {rate:.2}% | {normalized_rate:.2}% |\n",
                slice.correct, slice.normalized_correct, slice.total
            ));
        }
        let classes = slices.len().max(1) as f64;
        out.push_str(&format!(
            "\nMacro average across reported rule slices: {:.2}% exact, {:.2}% normalized.\n",
            macro_exact / classes,
            macro_normalized / classes
        ));
        out.push_str(
            "\n### POS, class, and cell detail\n\n| Cell slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (cell, slice) in by_cell {
            let rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.correct as f64 / slice.total as f64
            };
            let normalized_rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.normalized_correct as f64 / slice.total as f64
            };
            out.push_str(&format!(
                "| `{cell}` | {} | {} | {} | {rate:.2}% | {normalized_rate:.2}% |\n",
                slice.correct, slice.normalized_correct, slice.total
            ));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "Skipped OOV cells requiring unavailable lexical metadata: {}.\n",
        report.oov.skipped_cells
    ));
    out.push_str("\n## Extraction exclusions\n\n");
    for (reason, count) in &report.extraction_exclusions {
        out.push_str(&format!("- `{reason}`: {count}\n"));
    }
    out
}

fn metadata_funnel_rows(
    development: &MetadataFunnel,
    final_holdout: &MetadataFunnel,
) -> Vec<(&'static str, (usize, usize))> {
    vec![
        (
            "compatible requested cells",
            (
                development.compatible_target_cells,
                final_holdout.compatible_target_cells,
            ),
        ),
        (
            "unambiguous lexeme cells",
            (
                development.unambiguous_target_cells,
                final_holdout.unambiguous_target_cells,
            ),
        ),
        (
            "metadata records found",
            (
                development.metadata_records_found,
                final_holdout.metadata_records_found,
            ),
        ),
        (
            "metadata records validated",
            (
                development.metadata_records_validated,
                final_holdout.metadata_records_validated,
            ),
        ),
        (
            "generation attempts",
            (
                development.generation_attempts,
                final_holdout.generation_attempts,
            ),
        ),
        (
            "returned forms",
            (development.returned_forms, final_holdout.returned_forms),
        ),
        (
            "diplomatic top-1 correct",
            (
                development.diplomatic_top1_correct,
                final_holdout.diplomatic_top1_correct,
            ),
        ),
        (
            "diplomatic any correct",
            (
                development.diplomatic_any_correct,
                final_holdout.diplomatic_any_correct,
            ),
        ),
        (
            "project-lookup top-1 correct",
            (
                development.lookup_top1_correct,
                final_holdout.lookup_top1_correct,
            ),
        ),
        (
            "project-lookup any correct",
            (
                development.lookup_any_correct,
                final_holdout.lookup_any_correct,
            ),
        ),
    ]
}

fn accuracy_ud(args: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let path = required_path_flag(args, "--path")?;
    if !path.exists() {
        return Err(format!("UD path does not exist: {}", path.display()).into());
    }
    corpus::run_legacy(&path, &workspace_root()?)
}

fn dump_paradigms(name: Option<String>) -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let registry = load_registry(&root.join("data/extracted"))?;
    let directory = root.join("target/paradigm-fingerprint");
    fs::create_dir_all(&directory)?;
    let path = directory.join(format!(
        "{}.tsv",
        name.unwrap_or_else(|| "dump".to_string())
    ));
    let mut output = String::from("lexeme_id\tpos\tfeature\trank\tform\tromanization\n");
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    for row in &registry.forms {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.lexeme_id,
            pos_by_id
                .get(row.lexeme_id.as_str())
                .copied()
                .unwrap_or("?"),
            row.feature,
            row.rank,
            row.form,
            row.romanization
        ));
    }
    fs::write(&path, output.as_bytes())?;
    println!(
        "wrote {} ({} form variants)",
        path.display(),
        registry.forms.len()
    );
    Ok(())
}

fn diff_paradigms(before: &Path, after: &Path) -> Result<(), Box<dyn Error>> {
    let load = |path: &Path| -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        Ok(fs::read_to_string(path)?
            .lines()
            .skip(1)
            .filter_map(|line| {
                let columns = line.split('\t').collect::<Vec<_>>();
                (columns.len() == 6).then(|| {
                    (
                        format!(
                            "{}\t{}\t{}\t{}",
                            columns[0], columns[1], columns[2], columns[3]
                        ),
                        format!("{}\t{}", columns[4], columns[5]),
                    )
                })
            })
            .collect())
    };
    let before_rows = load(before)?;
    let after_rows = load(after)?;
    let mut changes = 0usize;
    for (key, old) in &before_rows {
        match after_rows.get(key) {
            Some(new) if new != old => {
                println!("changed\t{key}\t{old}\t->\t{new}");
                changes += 1;
            }
            None => {
                println!("removed\t{key}\t{old}");
                changes += 1;
            }
            _ => {}
        }
    }
    for (key, new) in &after_rows {
        if !before_rows.contains_key(key) {
            println!("added\t{key}\t{new}");
            changes += 1;
        }
    }
    eprintln!("{changes} changed variants");
    Ok(())
}

fn examples() -> Result<(), Box<dyn Error>> {
    run_cargo(&["run", "-p", "old-church-slavonic", "--example", "basic"])?;
    run_cargo(&["run", "-p", "old-church-slavonic", "--example", "tour"])
}

fn speed() -> Result<(), Box<dyn Error>> {
    run_cargo(&[
        "run",
        "-p",
        "old-church-slavonic",
        "--example",
        "speedmark",
        "--release",
    ])
}

fn check_all() -> Result<(), Box<dyn Error>> {
    run_cargo(&["fmt", "--all", "--", "--check"])?;
    run_cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;
    run_cargo(&["test", "--workspace", "--all-features"])?;
    run_cargo(&["test", "--workspace", "--doc"])?;
    let root = workspace_root()?;
    check_registry(&root)?;
    check_dictionary(&root)?;
    check_accuracy_report(&root)?;
    check_public_api_structure(&root)?;
    check_runtime_boundaries(&root)?;
    check_attribution(&root)?;
    examples()
}

fn check_accuracy_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = evaluate_accuracy(root, &root.join("data/extracted"))?;
    let expected_json = serde_json::to_vec_pretty(&report)?;
    let expected_markdown = accuracy_markdown(&report);
    if fs::read(root.join("reports/accuracy.json"))? != expected_json
        || fs::read_to_string(root.join("reports/accuracy.md"))? != expected_markdown
    {
        return Err("committed accuracy reports are stale; run cargo xtask accuracy".into());
    }
    println!("accuracy reports: current");
    Ok(())
}

fn check_public_api_structure(root: &Path) -> Result<(), Box<dyn Error>> {
    let facade = fs::read_to_string(root.join("crates/old-church-slavonic/src/lib.rs"))?;
    let result = fs::read_to_string(root.join("crates/old-church-slavonic-core/src/result.rs"))?;
    let resolver = fs::read_to_string(root.join("crates/old-church-slavonic/src/resolver.rs"))?;

    if facade.contains("pub use old_church_slavonic_core::*") {
        return Err("facade root restores a blanket core re-export".into());
    }

    for (name, dimensions) in [
        ("noun", &["lemma: &str", "case: Case", "number: Number"][..]),
        (
            "adjective",
            &[
                "lemma: &str",
                "case: Case",
                "number: Number",
                "gender: Gender",
                "animacy: Animacy",
            ][..],
        ),
        (
            "short_adjective",
            &[
                "lemma: &str",
                "case: Case",
                "number: Number",
                "gender: Gender",
                "animacy: Animacy",
            ][..],
        ),
        ("verb", &["lemma: &str", "person: Person", "number: Number"]),
        (
            "imperfect",
            &["lemma: &str", "person: Person", "number: Number"],
        ),
        (
            "aorist",
            &["lemma: &str", "person: Person", "number: Number"],
        ),
        (
            "finite_verb",
            &[
                "lemma: &str",
                "tense: FiniteTense",
                "person: Person",
                "number: Number",
            ],
        ),
        (
            "imperative",
            &["lemma: &str", "person: Person", "number: Number"],
        ),
        (
            "l_participle",
            &["lemma: &str", "gender: Gender", "number: Number"],
        ),
        ("infinitive", &["lemma: &str"]),
        ("supine", &["lemma: &str"]),
        ("verbal_noun", &["lemma: &str"]),
        ("comparative", &["lemma: &str"]),
    ] {
        let header = source_item(&facade, &format!("pub fn {name}("))?;
        if header[..=header.find('{').ok_or("public function has no body")?].contains("Cell") {
            return Err(format!("ordinary root function {name} requires a cell struct").into());
        }
        for dimension in dimensions {
            if !header.contains(dimension) {
                return Err(format!(
                    "ordinary root function {name} is missing direct dimension `{dimension}`"
                )
                .into());
            }
        }
    }

    for (name, delegation) in [
        ("noun", "resolver::noun("),
        ("adjective", "adjective_form("),
        ("short_adjective", "adjective_form("),
        ("verb", "finite_verb("),
        ("imperfect", "finite_verb("),
        ("aorist", "finite_verb("),
        ("finite_verb", "resolver::finite_verb("),
        ("imperative", "resolver::imperative("),
        ("l_participle", "resolver::l_participle("),
        ("infinitive", "resolver::infinitive("),
        ("supine", "resolver::supine("),
        ("verbal_noun", "resolver::verbal_noun("),
        ("comparative", "resolver::adjective_comparatives("),
        ("present_active_participle", "Verb::new("),
        ("present_passive_participle", "Verb::new("),
        ("past_active_participle", "Verb::new("),
        ("past_passive_participle", "Verb::new("),
        ("noun_paradigm", "Noun::new("),
        ("adjective_paradigm", "Adjective::new("),
        ("verb_paradigm", "Verb::new("),
        ("finite_verb_paradigm", "Verb::new("),
        ("imperative_paradigm", "Verb::new("),
        ("l_participle_paradigm", "Verb::new("),
        ("participle_paradigm", "Verb::new("),
    ] {
        let item = source_item(&facade, &format!("pub fn {name}("))?;
        if !item.contains(delegation) {
            return Err(format!(
                "root convenience function {name} bypasses canonical delegation `{delegation}`"
            )
            .into());
        }
    }
    if !source_item(&facade, "fn adjective_form(")?.contains("resolver::adjective(") {
        return Err("ordinary adjective functions bypass the canonical resolver".into());
    }

    let form_set = source_item(&result, "pub struct FormSet {")?;
    let constructor = source_item(&result, "pub fn new(")?;
    if form_set.contains("pub variants:")
        || !form_set.contains("variants: Vec<FormVariant>")
        || !constructor.contains("primary: FormVariant")
        || !constructor.contains("variants.push(primary)")
        || result.matches("\n        Self {\n").count() != 1
        || result.contains("impl Default for FormSet")
    {
        return Err("successful FormSet construction is not structurally nonempty".into());
    }

    for (builder, resolver_call) in [
        ("build_noun_paradigm", "result: noun_by_id(id, cell)"),
        (
            "build_adjective_paradigm",
            "result: adjective_by_id(id, cell)",
        ),
        (
            "build_finite_verb_paradigm",
            "result: finite_verb_by_id(id, cell)",
        ),
        ("build_verb_paradigm", "result: finite_verb_by_id(id, cell)"),
        (
            "build_imperative_paradigm",
            "result: imperative_by_id(id, cell)",
        ),
        (
            "build_l_participle_paradigm",
            "result: l_participle_by_id(id, cell)",
        ),
        (
            "build_participle_paradigm",
            "result: participle_by_id(id, cell)",
        ),
    ] {
        let item = source_item(&resolver, &format!("fn {builder}("))?;
        if !item.contains("cells.push(CellOutcome") || !item.contains(resolver_call) {
            return Err(format!("{builder} does not retain every canonical cell outcome").into());
        }
        for dropping in [
            "if let Ok",
            ".is_ok()",
            ".filter(",
            ".filter_map(",
            ".retain(",
        ] {
            if item.contains(dropping) {
                return Err(format!("{builder} can drop failed cells through `{dropping}`").into());
            }
        }
    }

    let mut remaining = facade.as_str();
    let mut offset = 0;
    while let Some(relative) = remaining.find("pub fn ") {
        let start = offset + relative;
        let name_start = start + "pub fn ".len();
        let name_end = facade[name_start..]
            .find('(')
            .map(|position| name_start + position)
            .ok_or("public root function has no argument list")?;
        let name = &facade[name_start..name_end];
        let docs = immediately_preceding_docs(&facade, start);
        if !docs.contains("```") {
            return Err(format!("root function {name} lacks a rustdoc example").into());
        }
        offset = name_end;
        remaining = &facade[offset..];
    }

    println!("public API structure: curated, delegated, nonempty, and documented");
    Ok(())
}

fn source_item<'a>(source: &'a str, needle: &str) -> Result<&'a str, Box<dyn Error>> {
    let start = source
        .find(needle)
        .ok_or_else(|| format!("source item `{needle}` is missing"))?;
    let open = source[start..]
        .find('{')
        .map(|position| start + position)
        .ok_or_else(|| format!("source item `{needle}` has no body"))?;
    let mut depth = 0_usize;
    for (relative, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| format!("source item `{needle}` has unbalanced braces"))?;
                if depth == 0 {
                    return Ok(&source[start..open + relative + character.len_utf8()]);
                }
            }
            _ => {}
        }
    }
    Err(format!("source item `{needle}` has no closing brace").into())
}

fn immediately_preceding_docs(source: &str, item_start: usize) -> String {
    let mut lines = source[..item_start].lines().rev();
    let mut docs = Vec::new();
    for line in lines.by_ref() {
        if line.trim_start().starts_with("///") {
            docs.push(line);
        } else {
            break;
        }
    }
    docs.reverse();
    docs.join("\n")
}

fn check_runtime_boundaries(root: &Path) -> Result<(), Box<dyn Error>> {
    for relative in [
        "crates/old-church-slavonic-core/src",
        "crates/old-church-slavonic/src",
    ] {
        let mut stack = vec![root.join(relative)];
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    stack.push(entry?.path());
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path)?;
                for forbidden in [
                    "std::fs",
                    "std::io",
                    "std::net",
                    "TcpStream",
                    "UdpSocket",
                    "reqwest",
                    "ureq",
                    "serde_json",
                    "quick_xml",
                    "roxmltree",
                    "csv::",
                    "mlua",
                    "rlua",
                ] {
                    if source.contains(forbidden) {
                        return Err(format!(
                            "runtime I/O/network boundary violation in {}: {forbidden}",
                            path.display()
                        )
                        .into());
                    }
                }
            }
        }
    }
    for relative in [
        "crates/old-church-slavonic-core/Cargo.toml",
        "crates/old-church-slavonic/Cargo.toml",
    ] {
        let manifest = fs::read_to_string(root.join(relative))?;
        for forbidden in [
            "reqwest",
            "ureq",
            "serde_json",
            "quick-xml",
            "roxmltree",
            "csv",
            "mlua",
            "rlua",
        ] {
            if manifest.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(forbidden)
                    .is_some_and(|suffix| suffix.trim_start().starts_with(['=', '.']))
            }) {
                return Err(format!(
                    "runtime data/network dependency violation in {relative}: {forbidden}"
                )
                .into());
            }
        }
    }
    println!("runtime boundary: no file, network, JSON, TSV, XML, or Lua access");
    Ok(())
}

fn check_attribution(root: &Path) -> Result<(), Box<dyn Error>> {
    let package = root.join("crates/old-church-slavonic");
    let attribution = fs::read_to_string(package.join("ATTRIBUTION.md"))?;
    let source: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("data/extracted/source.json"))?)?;
    let sha = source["sha256"]
        .as_str()
        .ok_or("source metadata has no SHA-256")?;
    if !attribution.contains(sha)
        || !attribution.contains("English Wiktionary")
        || !attribution.contains("CC BY-SA 4.0")
        || !attribution.contains("creativecommons.org/licenses/by-sa/4.0/legalcode")
        || !attribution.contains("source was modified")
    {
        return Err("published attribution is missing source identity or license".into());
    }
    let manifest = fs::read_to_string(package.join("Cargo.toml"))?;
    if !manifest.contains("CC-BY-SA-4.0") {
        return Err("published manifest omits the bundled data license".into());
    }
    for required in [
        "ATTRIBUTION.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "generated/**",
    ] {
        if !manifest.contains(required) {
            return Err(format!("published manifest omits required artifact: {required}").into());
        }
    }
    if !fs::read_to_string(package.join("LICENSE-MIT"))?.contains("MIT License")
        || !fs::read_to_string(package.join("LICENSE-APACHE"))?.contains("Apache License")
    {
        return Err("published code license texts are incomplete".into());
    }
    println!("package attribution: current");
    Ok(())
}

fn guard_witnesses() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let witness_root = std::env::temp_dir().join(format!(
        "old-church-slavonic-guard-witnesses-{}",
        std::process::id()
    ));
    if witness_root.exists() {
        fs::remove_dir_all(&witness_root)?;
    }
    let result = (|| -> Result<(), Box<dyn Error>> {
        copy_guard_fixture(&root, &witness_root)?;

        let facade_lib = "crates/old-church-slavonic/src/lib.rs";
        let mut changed = fs::read_to_string(witness_root.join(facade_lib))?;
        changed.push_str("\npub use old_church_slavonic_core::*;\n");
        fs::write(witness_root.join(facade_lib), changed)?;
        require_guard_failure(
            "curated facade root",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, facade_lib)?;

        let mut changed = fs::read_to_string(witness_root.join(facade_lib))?;
        changed = changed.replacen(
            "pub fn noun(lemma: &str, case: Case, number: Number)",
            "pub fn noun(lemma: &str, cell: old_church_slavonic_core::NounCell)",
            1,
        );
        fs::write(witness_root.join(facade_lib), changed)?;
        require_guard_failure(
            "direct ordinary dimensions",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, facade_lib)?;

        let result_source = "crates/old-church-slavonic-core/src/result.rs";
        let mut changed = fs::read_to_string(witness_root.join(result_source))?;
        changed = changed.replacen(
            "    variants: Vec<FormVariant>,",
            "    pub variants: Vec<FormVariant>,",
            1,
        );
        fs::write(witness_root.join(result_source), changed)?;
        require_guard_failure(
            "nonempty successful FormSet",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, result_source)?;

        let mut changed = fs::read_to_string(witness_root.join(facade_lib))?;
        changed = changed.replacen(
            "resolver::noun(lemma,",
            "old_church_slavonic_core::noun::decline(lemma,",
            1,
        );
        fs::write(witness_root.join(facade_lib), changed)?;
        require_guard_failure(
            "canonical convenience delegation",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, facade_lib)?;

        let resolver_source = "crates/old-church-slavonic/src/resolver.rs";
        let mut changed = fs::read_to_string(witness_root.join(resolver_source))?;
        changed = changed.replacen(
            "result: noun_by_id(id, cell),",
            "result: if noun_by_id(id, cell).is_ok() { noun_by_id(id, cell) } else { continue },",
            1,
        );
        fs::write(witness_root.join(resolver_source), changed)?;
        require_guard_failure(
            "failed paradigm cells remain visible",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, resolver_source)?;

        let mut changed = fs::read_to_string(witness_root.join(facade_lib))?;
        let noun_docs_start = changed
            .find("/// Decline one dictionary noun cell.")
            .ok_or("noun rustdoc witness target is missing")?;
        let noun_item_start = changed[noun_docs_start..]
            .find("pub fn noun(")
            .map(|position| noun_docs_start + position)
            .ok_or("noun function witness target is missing")?;
        let without_example = changed[noun_docs_start..noun_item_start].replace("```", "~~~");
        changed.replace_range(noun_docs_start..noun_item_start, &without_example);
        fs::write(witness_root.join(facade_lib), changed)?;
        require_guard_failure(
            "root rustdoc examples",
            check_public_api_structure(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, facade_lib)?;

        let generated = "crates/old-church-slavonic/generated/registry.rs";
        let mut changed = fs::read_to_string(witness_root.join(generated))?;
        changed.push_str("\n// stale generated witness\n");
        fs::write(witness_root.join(generated), changed)?;
        require_guard_failure(
            "generated registry freshness",
            check_registry(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, generated)?;

        let forms = "data/extracted/forms.tsv";
        let mut changed = fs::read_to_string(witness_root.join(forms))?;
        let duplicate = changed
            .lines()
            .nth(1)
            .ok_or("forms fixture has no data row")?
            .to_string();
        changed.push_str(&duplicate);
        changed.push('\n');
        fs::write(witness_root.join(forms), changed)?;
        require_guard_failure("duplicate cell/rank", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        let metadata = "data/extracted/verb_metadata.tsv";
        rewrite_metadata_row(
            &witness_root.join(metadata),
            |columns| columns.get(3).is_some_and(|field| field == "formation"),
            |columns| columns[4] = "unknown-formation".to_string(),
        )?;
        require_guard_failure("unknown metadata formation", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        let changed = fs::read_to_string(witness_root.join(metadata))?;
        let mut removed_policy = false;
        let changed = changed
            .lines()
            .filter(|line| {
                let remove = !removed_policy
                    && line
                        .split('\t')
                        .nth(3)
                        .is_some_and(|field| field == "variant-policy");
                removed_policy |= remove;
                !remove
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        if !removed_policy {
            return Err("metadata fixture has no variant-policy row".into());
        }
        fs::write(witness_root.join(metadata), changed)?;
        require_guard_failure(
            "incomplete metadata analysis",
            check_registry(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, metadata)?;

        let mut changed = fs::read_to_string(witness_root.join(metadata))?;
        let duplicate = changed
            .lines()
            .nth(1)
            .ok_or("metadata fixture has no data row")?
            .to_string();
        changed.push_str(&duplicate);
        changed.push('\n');
        fs::write(witness_root.join(metadata), changed)?;
        require_guard_failure("duplicate metadata field", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        rewrite_metadata_row(
            &witness_root.join(metadata),
            |_| true,
            |columns| columns[0] = "missing|verb|orphan".to_string(),
        )?;
        require_guard_failure("orphan metadata lexeme", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        rewrite_metadata_row(
            &witness_root.join(metadata),
            |columns| columns.get(3).is_some_and(|field| field == "stem"),
            |columns| columns[4].clear(),
        )?;
        require_guard_failure("empty metadata stem", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        rewrite_metadata_row(
            &witness_root.join(metadata),
            |columns| columns.get(3).is_some_and(|field| field == "stem"),
            |columns| columns[4] = "И\u{306}".to_string(),
        )?;
        require_guard_failure("non-NFC metadata stem", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        rewrite_metadata_row(
            &witness_root.join(metadata),
            |columns| columns.get(3).is_some_and(|field| field == "stem"),
            |columns| columns[4] = "latin".to_string(),
        )?;
        require_guard_failure("non-Cyrillic metadata stem", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, metadata)?;

        let overrides = "data/overrides.tsv";
        rewrite_metadata_row(
            &witness_root.join(overrides),
            |columns| columns.get(1).is_some_and(|pos| pos == "verb"),
            |columns| columns[2] = "verb:finite:future:1:sg".to_string(),
        )?;
        require_guard_failure("invalid override feature", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, overrides)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns.get(3).is_some_and(|form| !form.is_empty()),
            |columns| columns[3] = "—".to_string(),
        )?;
        require_guard_failure("sentinel public form", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns.get(3).is_some_and(|form| !form.is_empty()),
            |columns| columns[3] = "сло{{{2}}}во".to_string(),
        )?;
        require_guard_failure("MediaWiki markup form", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns[0].starts_with("обѣдъ|noun|") && columns[1] == "noun:nom:sg",
            |columns| columns[3] = "несъвпадение".to_string(),
        )?;
        require_guard_failure("canonical noun citation", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        swap_first_variant_pair(&witness_root.join(forms))?;
        require_guard_failure("source variant order", check_accuracy_report(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        require_guard_failure(
            "coverage floor",
            old_church_slavonic_extractor::validate::coverage(
                old_church_slavonic_extractor::validate::MIN_ACCEPTED_LEXEMES - 1,
                old_church_slavonic_extractor::validate::MIN_ACCEPTED_FORMS,
            ),
        )?;

        let runtime = "crates/old-church-slavonic-core/src/lib.rs";
        let mut changed = fs::read_to_string(witness_root.join(runtime))?;
        changed.push_str("\nuse std::fs;\n");
        fs::write(witness_root.join(runtime), changed)?;
        require_guard_failure(
            "runtime I/O boundary",
            check_runtime_boundaries(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, runtime)?;

        let attribution = "crates/old-church-slavonic/ATTRIBUTION.md";
        let mut changed = fs::read_to_string(witness_root.join(attribution))?;
        changed = changed.replace(
            "5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c",
            "missing-source-hash",
        );
        fs::write(witness_root.join(attribution), changed)?;
        require_guard_failure("published attribution", check_attribution(&witness_root))?;
        restore_guard_file(&root, &witness_root, attribution)?;

        let extraction_report = "reports/extraction-coverage.json";
        let mut changed: serde_json::Value =
            serde_json::from_slice(&fs::read(witness_root.join(extraction_report))?)?;
        changed["accepted_forms"] = serde_json::Value::from(1_u64);
        fs::write(
            witness_root.join(extraction_report),
            serde_json::to_vec_pretty(&changed)?,
        )?;
        require_guard_failure("extraction report freshness", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, extraction_report)?;

        let accuracy_report = "reports/accuracy.md";
        let mut changed = fs::read_to_string(witness_root.join(accuracy_report))?;
        changed.push_str("\nstale accuracy witness\n");
        fs::write(witness_root.join(accuracy_report), changed)?;
        require_guard_failure(
            "accuracy report freshness",
            check_accuracy_report(&witness_root),
        )?;

        let registry = load_registry(&root.join("data/extracted"))?;
        let mut integrity = dictionary_accuracy(&registry)?;
        integrity.paradigm_cell_sets_correct = integrity
            .paradigm_cell_sets_correct
            .checked_sub(1)
            .ok_or("dictionary fixture has no lexemes")?;
        require_guard_failure(
            "paradigm/cell agreement",
            ensure_dictionary_integrity(&integrity),
        )?;

        let healthy_funnel = MetadataFunnel {
            compatible_target_cells: 100,
            unambiguous_target_cells: 100,
            metadata_records_found: 100,
            metadata_records_validated: 100,
            generation_attempts: 100,
            returned_forms: 100,
            diplomatic_top1_correct: 100,
            diplomatic_any_correct: 100,
            lookup_top1_correct: 100,
            lookup_any_correct: 100,
        };
        let mut metadata_integrity = MetadataE2eAccuracy {
            development: healthy_funnel.clone(),
            final_holdout: healthy_funnel,
            ..MetadataE2eAccuracy::default()
        };
        metadata_integrity.final_holdout.metadata_records_found = 34;
        metadata_integrity.final_holdout.metadata_records_validated = 34;
        metadata_integrity.final_holdout.generation_attempts = 34;
        metadata_integrity.final_holdout.returned_forms = 34;
        metadata_integrity.final_holdout.diplomatic_top1_correct = 34;
        metadata_integrity.final_holdout.diplomatic_any_correct = 34;
        metadata_integrity.final_holdout.lookup_top1_correct = 34;
        metadata_integrity.final_holdout.lookup_any_correct = 34;
        require_guard_failure(
            "metadata availability floor",
            ensure_metadata_e2e(&metadata_integrity),
        )?;
        metadata_integrity.final_holdout = metadata_integrity.development.clone();
        metadata_integrity.final_holdout.lookup_any_correct = 94;
        require_guard_failure(
            "metadata conditional-accuracy floor",
            ensure_metadata_e2e(&metadata_integrity),
        )?;

        for hostile in ["", "two words", "\0", &"x".repeat(4_097)] {
            if std::panic::catch_unwind(|| {
                let _ = old_church_slavonic::lookup(hostile, PartOfSpeech::Noun);
            })
            .is_err()
            {
                return Err(format!("hostile-input guard observed a panic for {hostile:?}").into());
            }
        }
        println!("guard witness observed: hostile input remains panic-free");
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&witness_root);
    result?;
    cleanup?;
    println!("guard-witnesses: all injected failures were detected and reverted");
    Ok(())
}

fn copy_guard_fixture(root: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    for relative in [
        "data/extracted",
        "crates/old-church-slavonic-core/src",
        "crates/old-church-slavonic/src",
        "crates/old-church-slavonic/generated",
    ] {
        copy_tree(&root.join(relative), &destination.join(relative))?;
    }
    for relative in [
        "data/overrides.tsv",
        "data/citation-exemptions.tsv",
        "data/SOURCES.toml",
        "reports/extraction-coverage.json",
        "reports/extraction-coverage.md",
        "reports/accuracy.json",
        "reports/accuracy.md",
        "crates/old-church-slavonic/ATTRIBUTION.md",
        "crates/old-church-slavonic/Cargo.toml",
        "crates/old-church-slavonic/LICENSE-MIT",
        "crates/old-church-slavonic/LICENSE-APACHE",
    ] {
        restore_guard_file(root, destination, relative)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn restore_guard_file(
    root: &Path,
    destination: &Path,
    relative: &str,
) -> Result<(), Box<dyn Error>> {
    let target = destination.join(relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(root.join(relative), target)?;
    Ok(())
}

fn require_guard_failure<E: std::fmt::Display>(
    name: &str,
    result: Result<(), E>,
) -> Result<(), Box<dyn Error>> {
    match result {
        Ok(()) => Err(format!("guard witness did not fail: {name}").into()),
        Err(error) => {
            println!("guard witness observed: {name}: {error}");
            Ok(())
        }
    }
}

fn rewrite_form_row(
    path: &Path,
    predicate: impl Fn(&[String]) -> bool,
    mutation: impl Fn(&mut [String]),
) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut changed = false;
    let mut output = String::new();
    for (line_index, line) in contents.lines().enumerate() {
        let mut columns = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        if line_index > 0 && !changed && predicate(&columns) {
            mutation(&mut columns);
            changed = true;
        }
        output.push_str(&columns.join("\t"));
        output.push('\n');
    }
    if !changed {
        return Err("guard witness could not find its target form row".into());
    }
    fs::write(path, output)?;
    Ok(())
}

fn rewrite_metadata_row(
    path: &Path,
    predicate: impl Fn(&[String]) -> bool,
    mutation: impl Fn(&mut [String]),
) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut changed = false;
    let mut output = String::new();
    for (line_index, line) in contents.lines().enumerate() {
        let mut columns = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        if line_index > 0 && !changed && predicate(&columns) {
            mutation(&mut columns);
            changed = true;
        }
        output.push_str(&columns.join("\t"));
        output.push('\n');
    }
    if !changed {
        return Err("guard witness could not find its target metadata row".into());
    }
    fs::write(path, output)?;
    Ok(())
}

fn swap_first_variant_pair(path: &Path) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut lines = contents
        .lines()
        .map(|line| line.split('\t').map(str::to_string).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let pair = (1..lines.len().saturating_sub(1)).find(|index| {
        lines[*index][0] == lines[*index + 1][0]
            && lines[*index][1] == lines[*index + 1][1]
            && lines[*index][2] == "0"
            && lines[*index + 1][2] == "1"
    });
    let index = pair.ok_or("guard witness found no multi-variant cell")?;
    let (left, right) = lines.split_at_mut(index + 1);
    for column in 3..=4 {
        std::mem::swap(&mut left[index][column], &mut right[0][column]);
    }
    let mut output = lines
        .into_iter()
        .map(|columns| columns.join("\t"))
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    fs::write(path, output)?;
    Ok(())
}

fn run_cargo(args: &[&str]) -> Result<(), Box<dyn Error>> {
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .current_dir(workspace_root()?)
        .args(args)
        .status()?;
    require_success(status, args)
}

fn require_success(status: ExitStatus, args: &[&str]) -> Result<(), Box<dyn Error>> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed with {status}", args.join(" ")).into())
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn print_help() {
    eprintln!("cargo xtask <command>");
    eprintln!("  refresh-data --dump PATH");
    eprintln!("  refresh-dictionary --dump PATH");
    eprintln!("  refresh-derived-registry");
    eprintln!("  check-registry");
    eprintln!("  check-dictionary");
    eprintln!("  extraction-report");
    eprintln!("  accuracy");
    eprintln!("  accuracy-ud --path UD_DIRECTORY");
    eprintln!(
        "  accuracy-corpus [--ud UD_DIRECTORY] [--syntacticus TREEBANK_DIRECTORY] [--details PATH] [--write]"
    );
    eprintln!("  dump-paradigms [NAME]");
    eprintln!("  diff-paradigms BEFORE AFTER");
    eprintln!("  examples");
    eprintln!("  speed");
    eprintln!("  guard-witnesses");
    eprintln!("  check-all");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(feature: &str, value: &str) -> FormRow {
        FormRow {
            lexeme_id: "нести|verb|fixture".to_string(),
            feature: feature.to_string(),
            rank: 0,
            form: value.to_string(),
            romanization: String::new(),
            source_spelling: value.to_string(),
            source_tags: "fixture".to_string(),
        }
    }

    #[test]
    fn held_principal_parts_yield_explicit_verb_stems() {
        assert_eq!(
            derive_present_stem(VerbClass::IA1, "несеши").as_deref(),
            Some("нес")
        );
        assert_eq!(
            derive_present_stem(VerbClass::II1, "правиши").as_deref(),
            Some("прав")
        );
        assert_eq!(
            derive_l_participle_stem("правилъ").as_deref(),
            Some("прави")
        );
        assert_eq!(derive_present_stem(VerbClass::Root, "еси"), None);
        assert_eq!(derive_l_participle_stem("лъ"), None);
    }

    #[test]
    fn leakage_filter_removes_target_and_same_spelling_features() {
        let forms = [
            form("verb:finite:imperfect:1:sg", "несѣахъ"),
            form("verb:finite:imperfect:3:sg", "несѣахъ"),
            form("verb:finite:imperfect:2:sg", "несѣаше"),
        ];
        let spellings = BTreeSet::from(["несѣахъ"]);
        let excluded =
            excluded_metadata_features(forms.iter(), "verb:finite:imperfect:1:sg", &spellings);
        assert_eq!(
            excluded,
            BTreeSet::from(["verb:finite:imperfect:1:sg", "verb:finite:imperfect:3:sg"])
        );
        assert!(!excluded.contains("verb:finite:imperfect:2:sg"));
    }

    #[test]
    fn leakage_filter_removes_equivalent_person_cells_even_when_spelling_differs() {
        let forms = [
            form("verb:finite:imperfect:2:sg", "несѣаше"),
            form("verb:finite:imperfect:3:sg", "несѣа҅ше"),
            form("verb:finite:imperfect:1:sg", "несѣахъ"),
        ];
        let spellings = BTreeSet::from(["несѣаше"]);
        let excluded =
            excluded_metadata_features(forms.iter(), "verb:finite:imperfect:2:sg", &spellings);
        assert_eq!(
            excluded,
            BTreeSet::from(["verb:finite:imperfect:2:sg", "verb:finite:imperfect:3:sg"])
        );
    }

    #[test]
    fn frozen_dictionary_partition_witnesses_do_not_drift() {
        assert_eq!(fnv1a("нести".as_bytes()), 9_211_201_522_989_420_120);
        assert_eq!(fnv1a("нести".as_bytes()) % 5, 0);
        assert_eq!(fnv1a("бꙑти".as_bytes()) % 5, 1);
        assert_eq!(fnv1a("благословити".as_bytes()) % 5, 4);
    }
}
