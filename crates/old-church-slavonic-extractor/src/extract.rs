use crate::emit::generated_rust;
use crate::normalize::{has_wiki_markup, lookup_key};
use crate::output::atomic_write_batch;
use crate::report::ExtractionReport;
use crate::schema::{AliasRow, Entry, FormRow, LexemeRow, Registry, SourceForm, VerbMetadataRow};
use crate::validate;
use crate::verb_metadata;
use old_church_slavonic_core::orthography::{Script, canonical_display, detect_script};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

const REGISTRY_SCHEMA: u32 = 2;
const MAX_PARSE_FAILURE_FRACTION: f64 = 0.001;

#[derive(Debug)]
struct PendingLexeme {
    lemma: String,
    page_word: String,
    pos: String,
    class: String,
    raw_class: String,
    gender: String,
    animacy: String,
    number_restriction: String,
    head_templates: String,
    aliases: BTreeMap<String, BTreeSet<String>>,
    forms: BTreeMap<String, Vec<PendingForm>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingForm {
    form: String,
    romanization: String,
    source_spelling: String,
    tags: String,
}

#[derive(Debug, Serialize)]
struct SourceMetadata {
    schema_version: u32,
    input_file: String,
    bytes: u64,
    sha256: String,
}

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("refresh") => {
            let mut dump = None;
            let mut root = workspace_root()?;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--dump" => dump = args.next().map(PathBuf::from),
                    "--root" => root = PathBuf::from(args.next().ok_or("--root needs a path")?),
                    other => return Err(format!("unknown argument: {other}").into()),
                }
            }
            let dump = dump.ok_or("refresh requires --dump <PATH>")?;
            refresh(&dump, &root)?;
        }
        Some("check") => {
            let root = args
                .next()
                .map_or(workspace_root(), |value| Ok(value.into()))?;
            check_registry(&root)?;
        }
        Some("report") => {
            let root = args
                .next()
                .map_or(workspace_root(), |value| Ok(value.into()))?;
            let registry = load_registry(&root.join("data/extracted"))?;
            println!(
                "registry: {} lexemes, {} aliases, {} forms",
                registry.lexemes.len(),
                registry.aliases.len(),
                registry.forms.len()
            );
        }
        Some("dictionary-refresh") => {
            let mut dump = None;
            let mut root = workspace_root()?;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--dump" => dump = args.next().map(PathBuf::from),
                    "--root" => root = PathBuf::from(args.next().ok_or("--root needs a path")?),
                    other => return Err(format!("unknown argument: {other}").into()),
                }
            }
            let dump = dump.ok_or("dictionary-refresh requires --dump <PATH>")?;
            crate::semantics::refresh_dictionary(&dump, &root)?;
        }
        Some("dictionary-check") => {
            let root = args
                .next()
                .map_or(workspace_root(), |value| Ok(value.into()))?;
            crate::semantics::check_dictionary(&root)?;
        }
        _ => {
            eprintln!(
                "usage: old-church-slavonic-extractor \
                 <refresh --dump PATH|check|report|dictionary-refresh --dump PATH|dictionary-check>"
            );
        }
    }
    Ok(())
}

pub fn refresh(dump: &Path, root: &Path) -> Result<(), Box<dyn Error>> {
    let (mut registry, mut report) = extract_dump(dump)?;
    registry.lexemes.sort();
    registry.aliases.sort();
    registry.aliases.dedup();
    registry.forms.sort();
    registry.verb_metadata = verb_metadata::derive(&registry, &BTreeSet::new())?;
    registry.verb_metadata.sort();
    validate::registry(&registry)?;
    let citation_exemptions = load_citation_exemptions(&root.join("data/citation-exemptions.tsv"))?;

    report.accepted_lexemes = registry.lexemes.len();
    report.accepted_forms = registry.forms.len();
    report.accepted_by_pos.clear();
    report.accepted_by_class.clear();
    report.accepted_by_feature.clear();
    report.scripts.clear();
    for lexeme in &registry.lexemes {
        bump(&mut report.accepted_by_pos, &lexeme.pos);
        bump(
            &mut report.accepted_by_class,
            if lexeme.class.is_empty() {
                "unclassified"
            } else {
                &lexeme.class
            },
        );
        bump(
            &mut report.scripts,
            match detect_script(&lexeme.lemma) {
                Script::Cyrillic => "cyrillic",
                Script::Glagolitic => "glagolitic",
                Script::Latin => "latin",
                Script::Mixed => "mixed",
                Script::Unknown => "unknown",
            },
        );
    }
    for form in &registry.forms {
        bump(
            &mut report.accepted_by_feature,
            feature_group(&form.feature),
        );
    }
    report.ambiguous_lemma_pos_pairs = ambiguous_lemma_pos_pairs(&registry);
    let extracted = root.join("data/extracted");
    if extracted.join("lexemes.tsv").exists() {
        let previous = load_registry(&extracted)?;
        let previous_ids = previous
            .lexemes
            .iter()
            .map(|lexeme| lexeme.id.as_str())
            .collect::<BTreeSet<_>>();
        let current_ids = registry
            .lexemes
            .iter()
            .map(|lexeme| lexeme.id.as_str())
            .collect::<BTreeSet<_>>();
        report.added_lexeme_ids = current_ids
            .difference(&previous_ids)
            .map(|id| (*id).to_string())
            .collect();
        report.removed_lexeme_ids = previous_ids
            .difference(&current_ids)
            .map(|id| (*id).to_string())
            .collect();
        report.lexeme_ids_added = report.added_lexeme_ids.len();
        report.lexeme_ids_removed = report.removed_lexeme_ids.len();
    }
    fs::create_dir_all(&extracted)?;
    fs::create_dir_all(root.join("reports"))?;
    fs::create_dir_all(root.join("crates/old-church-slavonic/generated"))?;
    let (lexemes_tsv, aliases_tsv, forms_tsv, verb_metadata_tsv) = registry_text(&registry);
    let generated_registry =
        registry_with_overrides(registry.clone(), &root.join("data/overrides.tsv"))?;
    validate::noun_citations(&generated_registry, &citation_exemptions)?;
    let generated = generated_rust(&generated_registry);
    let report_json = serde_json::to_vec_pretty(&report)?;
    let report_markdown = report.markdown();
    let source = source_metadata(dump)?;
    let source_json = serde_json::to_vec_pretty(&source)?;
    atomic_write_batch(&[
        (extracted.join("lexemes.tsv"), lexemes_tsv.as_bytes()),
        (extracted.join("aliases.tsv"), aliases_tsv.as_bytes()),
        (extracted.join("forms.tsv"), forms_tsv.as_bytes()),
        (
            extracted.join("verb_metadata.tsv"),
            verb_metadata_tsv.as_bytes(),
        ),
        (extracted.join("source.json"), &source_json),
        (
            root.join("crates/old-church-slavonic/generated/registry.rs"),
            generated.as_bytes(),
        ),
        (root.join("reports/extraction-coverage.json"), &report_json),
        (
            root.join("reports/extraction-coverage.md"),
            report_markdown.as_bytes(),
        ),
    ])?;
    println!(
        "refreshed {} lexemes / {} forms from {}",
        registry.lexemes.len(),
        registry.forms.len(),
        dump.display()
    );
    Ok(())
}

fn load_citation_exemptions(path: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let mut out = BTreeSet::new();
    for (line_index, line) in fs::read_to_string(path)?.lines().enumerate().skip(1) {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 3 || columns[1].trim().is_empty() || columns[2].trim().is_empty() {
            return Err(format!(
                "invalid citation-exemptions.tsv row {}: id, reason, and source are required",
                line_index + 1
            )
            .into());
        }
        if !out.insert(columns[0].to_string()) {
            return Err(format!("duplicate citation exemption: {}", columns[0]).into());
        }
    }
    Ok(out)
}

pub fn check_registry(root: &Path) -> Result<(), Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    validate::registry(&registry)?;
    let mut derived_metadata = verb_metadata::derive(&registry, &BTreeSet::new())?;
    derived_metadata.sort();
    if registry.verb_metadata != derived_metadata {
        return Err("committed verb metadata is stale relative to normalized source cells".into());
    }
    let citation_exemptions = load_citation_exemptions(&root.join("data/citation-exemptions.tsv"))?;
    validate::coverage(registry.lexemes.len(), registry.forms.len())?;
    let coverage_bytes = fs::read(root.join("reports/extraction-coverage.json"))?;
    let coverage_report: ExtractionReport = serde_json::from_slice(&coverage_bytes)?;
    if fs::read_to_string(root.join("reports/extraction-coverage.md"))?
        != coverage_report.markdown()
    {
        return Err("Markdown extraction report is stale relative to its JSON report".into());
    }
    let coverage: serde_json::Value = serde_json::from_slice(&coverage_bytes)?;
    let reported_lexemes = coverage["accepted_lexemes"].as_u64();
    let reported_forms = coverage["accepted_forms"].as_u64();
    if reported_lexemes != u64::try_from(registry.lexemes.len()).ok()
        || reported_forms != u64::try_from(registry.forms.len()).ok()
    {
        return Err("committed extraction report does not match the normalized registry".into());
    }
    let source_json = fs::read_to_string(root.join("data/extracted/source.json"))?;
    let source: serde_json::Value = serde_json::from_str(&source_json)?;
    let sources_toml = fs::read_to_string(root.join("data/SOURCES.toml"))?;
    let source_sha = source["sha256"]
        .as_str()
        .ok_or("source.json has no sha256")?;
    let source_bytes = source["bytes"]
        .as_u64()
        .ok_or("source.json has no byte count")?;
    if !sources_toml.contains(source_sha)
        || !sources_toml.contains(&format!("bytes = {source_bytes}"))
    {
        return Err("data/SOURCES.toml disagrees with data/extracted/source.json".into());
    }
    let generated_registry =
        registry_with_overrides(registry.clone(), &root.join("data/overrides.tsv"))?;
    validate::noun_citations(&generated_registry, &citation_exemptions)?;
    let expected = generated_rust(&generated_registry);
    let generated_path = root.join("crates/old-church-slavonic/generated/registry.rs");
    let committed = fs::read_to_string(&generated_path)?;
    if committed != expected {
        return Err(format!(
            "generated registry is stale: refresh {}",
            generated_path.display()
        )
        .into());
    }
    println!(
        "check-registry: OK ({} lexemes, {} forms)",
        registry.lexemes.len(),
        registry.forms.len()
    );
    Ok(())
}

/// Refresh only artifacts that are pure derivations of the already committed
/// normalized registry. This never substitutes for auditing a changed raw dump.
pub fn refresh_derived_registry(root: &Path) -> Result<(), Box<dyn Error>> {
    let mut registry = load_registry(&root.join("data/extracted"))?;
    registry.verb_metadata = verb_metadata::derive(&registry, &BTreeSet::new())?;
    registry.verb_metadata.sort();
    validate::registry(&registry)?;
    let (_, _, _, verb_metadata_tsv) = registry_text(&registry);
    let generated_registry = registry_with_overrides(registry, &root.join("data/overrides.tsv"))?;
    let citation_exemptions = load_citation_exemptions(&root.join("data/citation-exemptions.tsv"))?;
    validate::noun_citations(&generated_registry, &citation_exemptions)?;
    let generated = generated_rust(&generated_registry);
    atomic_write_batch(&[
        (
            root.join("data/extracted/verb_metadata.tsv"),
            verb_metadata_tsv.as_bytes(),
        ),
        (
            root.join("crates/old-church-slavonic/generated/registry.rs"),
            generated.as_bytes(),
        ),
    ])?;
    Ok(())
}

pub fn registry_with_overrides(
    mut registry: Registry,
    path: &Path,
) -> Result<Registry, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut seen = BTreeSet::new();
    for (line_index, line) in contents.lines().enumerate().skip(1) {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 7 {
            return Err(format!(
                "invalid overrides.tsv row {}: expected 7 columns",
                line_index + 1
            )
            .into());
        }
        let (lemma, pos, feature, variants, reason, source, review_status) = (
            columns[0], columns[1], columns[2], columns[3], columns[4], columns[5], columns[6],
        );
        if reason.trim().is_empty() || source.trim().is_empty() {
            return Err(format!(
                "override row {} needs a nonempty reason and source",
                line_index + 1
            )
            .into());
        }
        if review_status != "approved" {
            return Err(format!(
                "override row {} is not approved (status: {review_status})",
                line_index + 1
            )
            .into());
        }
        let key = lookup_key(lemma)?;
        let ids = registry
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.pos == pos)
            .filter(|lexeme| {
                registry
                    .aliases
                    .iter()
                    .any(|alias| alias.key == key && alias.lexeme_id == lexeme.id)
            })
            .map(|lexeme| lexeme.id.clone())
            .collect::<Vec<_>>();
        let [lexeme_id] = ids.as_slice() else {
            return Err(format!(
                "override row {} must resolve to exactly one lexeme, found {}",
                line_index + 1,
                ids.len()
            )
            .into());
        };
        if !seen.insert((lexeme_id.clone(), feature.to_string())) {
            return Err(format!(
                "duplicate override for {} {feature} on row {}",
                lexeme_id,
                line_index + 1
            )
            .into());
        }
        let parsed = variants
            .split(" || ")
            .map(str::trim)
            .filter(|variant| !variant.is_empty())
            .collect::<Vec<_>>();
        if parsed.is_empty() {
            return Err(format!("override row {} has no variants", line_index + 1).into());
        }
        for (rank, value) in parsed.into_iter().enumerate() {
            let (form, romanization) = value.split_once(" :: ").unwrap_or((value, ""));
            if form.is_empty() || matches!(form, "-" | "—" | "no-table-tags") {
                return Err(
                    format!("override row {} contains a sentinel form", line_index + 1).into(),
                );
            }
            registry.overrides.push(crate::schema::OverrideRow {
                lexeme_id: lexeme_id.clone(),
                feature: feature.to_string(),
                rank: u16::try_from(rank)?,
                form: form.to_string(),
                romanization: romanization.to_string(),
                reason: reason.to_string(),
                authority: source.to_string(),
            });
        }
    }
    registry.overrides.sort();
    validate::registry(&registry)?;
    Ok(registry)
}

fn extract_dump(dump: &Path) -> Result<(Registry, ExtractionReport), Box<dyn Error>> {
    let reader = BufReader::new(File::open(dump)?);
    let mut report = ExtractionReport {
        schema_version: REGISTRY_SCHEMA,
        ..ExtractionReport::default()
    };
    let mut pending = Vec::new();
    for line in reader.lines() {
        report.input_lines += 1;
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => {
                report.parse_failures += 1;
                continue;
            }
        };
        if entry.lang_code != "cu" {
            continue;
        }
        report.ocs_entries += 1;
        let Some(pos) = normalize_pos(&entry.pos) else {
            bump(&mut report.dropped_by_reason, "unsupported-pos");
            continue;
        };
        match pending_lexeme(&entry, pos, &mut report) {
            Ok(Some(lexeme)) => pending.push(lexeme),
            Ok(None) => {}
            Err(reason) => bump(&mut report.dropped_by_reason, &reason),
        }
    }
    let failure_fraction = report.parse_failures as f64 / report.input_lines.max(1) as f64;
    if failure_fraction > MAX_PARSE_FAILURE_FRACTION {
        return Err(format!(
            "{} of {} input lines ({:.3}%) failed to parse; refusing likely schema drift",
            report.parse_failures,
            report.input_lines,
            failure_fraction * 100.0
        )
        .into());
    }
    if report.parse_failures > 0 {
        eprintln!(
            "warning: {} of {} input lines failed to parse",
            report.parse_failures, report.input_lines
        );
    }
    Ok((finalize(pending, &mut report)?, report))
}

fn pending_lexeme(
    entry: &Entry,
    pos: &str,
    report: &mut ExtractionReport,
) -> Result<Option<PendingLexeme>, String> {
    if !entry.senses.is_empty() && entry.senses.iter().all(|sense| has(&sense.tags, "form-of")) {
        bump(&mut report.dropped_by_reason, "form-of-entry-not-lexeme");
        return Ok(None);
    }
    if !is_safe_single_word(&entry.word) {
        bump(&mut report.dropped_by_reason, "unsafe-page-word");
        return Ok(None);
    }
    let lemma = canonical_lemma(entry, pos).to_string();
    let personal_reflexive_table = entry.forms.iter().any(|form| {
        form.source == "declension"
            && form.form == "l-self"
            && has(&form.tags, "inflection-template")
    });
    let reflexive_lemma = pos == "pron" && lookup_key(&lemma)? == lookup_key("сѧ")?;
    let lemma_key = lookup_key(&lemma)?;
    let page_key = lookup_key(&entry.word)?;
    let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    insert_alias(&mut aliases, lemma_key, "canonical", &lemma)?;
    insert_alias(&mut aliases, page_key, "page", &entry.word)?;
    for form in &entry.forms {
        if has_any(&form.tags, &["canonical", "alternative"]) {
            for spelling in public_source_spellings(&form.form).unwrap_or_default() {
                insert_alias(
                    &mut aliases,
                    lookup_key(spelling)?,
                    if has(&form.tags, "canonical") {
                        "source-canonical"
                    } else {
                        "alternative"
                    },
                    spelling,
                )?;
            }
        }
    }
    let gender = extract_gender(entry);
    let raw_class = entry
        .forms
        .iter()
        .find(|form| form.tags.iter().any(|tag| tag == "class"))
        .map_or("", |form| form.form.as_str());
    let class = if pos == "adj" {
        if lemma.ends_with('ъ') {
            "adj-hard".to_string()
        } else if lemma.ends_with(['ь', 'и']) {
            "adj-soft".to_string()
        } else {
            normalize_class(raw_class, &lemma, &gender)
        }
    } else {
        normalize_class(raw_class, &lemma, &gender)
    };
    let head_templates = serde_json::to_string(&entry.head_templates)
        .map_err(|error| format!("head-template serialization failed: {error}"))?;
    let animacy = extract_animacy(entry);
    let restriction = extract_restriction(entry);
    let mut forms: BTreeMap<String, Vec<PendingForm>> = BTreeMap::new();
    let mut adjective_table_block = 0_u8;
    for source in &entry.forms {
        if has_wiki_markup(&source.form) {
            bump(&mut report.dropped_by_reason, "template-markup-form");
            bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            continue;
        }
        let is_comparative = pos == "adj" && has(&source.tags, "comparative");
        if !is_comparative && source.source != "declension" && source.source != "conjugation" {
            continue;
        }
        if is_sentinel(source) {
            if pos == "adj" && has(&source.tags, "table-tags") {
                adjective_table_block = adjective_table_block.saturating_add(1);
            }
            bump(&mut report.dropped_by_reason, "table-metadata-or-sentinel");
            bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            continue;
        }
        let spellings = match public_source_spellings(&source.form) {
            Ok(spellings) => spellings,
            Err(reason) => {
                bump(&mut report.dropped_by_reason, reason);
                bump(&mut report.rejected_tag_signatures, &tag_signature(source));
                continue;
            }
        };
        if spellings.len() > 1 && !source.romanization.is_empty() {
            bump(
                &mut report.dropped_by_reason,
                "multi-variant-romanization-unsplittable",
            );
            bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            continue;
        }
        if is_comparative {
            let variants = forms
                .entry("adj:comparative:citation".to_string())
                .or_default();
            for spelling in spellings {
                let candidate = PendingForm {
                    form: spelling.to_string(),
                    romanization: source.romanization.clone(),
                    source_spelling: source.form.clone(),
                    tags: source.tags.join(","),
                };
                if !variants.iter().any(|existing| {
                    existing.form == candidate.form
                        && existing.romanization == candidate.romanization
                }) {
                    variants.push(candidate);
                    bump(&mut report.accepted_by_feature, "adj-comparative");
                    bump(&mut report.accepted_by_source, "headword-metadata");
                    bump(&mut report.accepted_tag_signatures, &tag_signature(source));
                }
            }
            continue;
        }
        if personal_reflexive_table
            && ((reflexive_lemma && !has(&source.tags, "reflexive"))
                || (!reflexive_lemma && has(&source.tags, "reflexive")))
        {
            bump(
                &mut report.dropped_by_reason,
                "personal-reflexive-table-other-lexeme",
            );
            bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            continue;
        }
        match feature_keys(pos, source, adjective_table_block) {
            Ok(keys) if !keys.is_empty() => {
                bump(&mut report.accepted_tag_signatures, &tag_signature(source));
                bump(
                    &mut report.accepted_by_source,
                    if source.source.is_empty() {
                        "unspecified"
                    } else {
                        &source.source
                    },
                );
                for key in keys {
                    let variants = forms.entry(key.clone()).or_default();
                    for spelling in &spellings {
                        let candidate = PendingForm {
                            form: (*spelling).to_string(),
                            romanization: source.romanization.clone(),
                            source_spelling: source.form.clone(),
                            tags: source.tags.join(","),
                        };
                        if !variants.iter().any(|existing| {
                            existing.form == candidate.form
                                && existing.romanization == candidate.romanization
                        }) {
                            variants.push(candidate);
                            bump(&mut report.accepted_by_feature, feature_group(&key));
                        }
                    }
                }
            }
            Ok(_) => {
                bump(&mut report.dropped_by_reason, "no-feature-key");
                bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            }
            Err(reason) => {
                bump(&mut report.dropped_by_reason, reason);
                bump(&mut report.rejected_tag_signatures, &tag_signature(source));
            }
        }
    }
    if forms.is_empty() {
        bump(&mut report.dropped_by_reason, "entry-without-safe-cells");
        return Ok(None);
    }
    bump(&mut report.accepted_by_pos, pos);
    bump(
        &mut report.accepted_by_class,
        if class.is_empty() {
            "unclassified"
        } else {
            &class
        },
    );
    bump(
        &mut report.scripts,
        match detect_script(&lemma) {
            Script::Cyrillic => "cyrillic",
            Script::Glagolitic => "glagolitic",
            Script::Latin => "latin",
            Script::Mixed => "mixed",
            Script::Unknown => "unknown",
        },
    );
    Ok(Some(PendingLexeme {
        lemma,
        page_word: entry.word.clone(),
        pos: pos.to_string(),
        class,
        raw_class: raw_class.to_string(),
        gender,
        animacy,
        number_restriction: restriction,
        head_templates,
        aliases,
        forms,
    }))
}

pub(crate) fn canonical_lemma<'a>(entry: &'a Entry, pos: &str) -> &'a str {
    if let Some(form) = entry
        .forms
        .iter()
        .find(|form| has(&form.tags, "canonical") && is_safe_single_word(&form.form))
    {
        return &form.form;
    }
    let table_citation = entry.forms.iter().find(|form| match pos {
        "noun" => {
            form.source == "declension"
                && has(&form.tags, "nominative")
                && has(&form.tags, "singular")
                && is_safe_single_word(&form.form)
        }
        "adj" => {
            form.source == "declension"
                && has(&form.tags, "nominative")
                && has(&form.tags, "singular")
                && has(&form.tags, "masculine")
                && is_safe_single_word(&form.form)
        }
        "verb" => {
            form.source == "conjugation"
                && has(&form.tags, "infinitive")
                && is_safe_single_word(&form.form)
        }
        _ => false,
    });
    table_citation.map_or(entry.word.as_str(), |form| form.form.as_str())
}

fn finalize(
    pending: Vec<PendingLexeme>,
    report: &mut ExtractionReport,
) -> Result<Registry, Box<dyn Error>> {
    let mut registry = Registry::default();
    let mut by_id: BTreeMap<String, PendingLexeme> = BTreeMap::new();
    for lexeme in pending {
        let signature = signature(&lexeme);
        let id = format!(
            "{}|{}|{:016x}",
            lookup_key(&lexeme.lemma)?,
            lexeme.pos,
            fnv1a(signature.as_bytes())
        );
        match by_id.entry(id) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(lexeme);
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let existing = slot.get_mut();
                for (key, spellings) in lexeme.aliases {
                    existing.aliases.entry(key).or_default().extend(spellings);
                }
                bump(&mut report.dropped_by_reason, "duplicate-identical-lexeme");
            }
        }
    }
    for (id, lexeme) in by_id {
        let key = lookup_key(&lexeme.lemma)?;
        let signature = format!("{:016x}", fnv1a(signature(&lexeme).as_bytes()));
        registry.lexemes.push(LexemeRow {
            id: id.clone(),
            lemma: lexeme.lemma,
            page_word: lexeme.page_word,
            key,
            pos: lexeme.pos,
            class: lexeme.class,
            raw_class: lexeme.raw_class,
            gender: lexeme.gender,
            animacy: lexeme.animacy,
            number_restriction: lexeme.number_restriction,
            head_templates: lexeme.head_templates,
            signature,
        });
        for (alias, source_spellings) in lexeme.aliases {
            registry.aliases.push(AliasRow {
                key: alias,
                lexeme_id: id.clone(),
                source_spellings: source_spellings
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(" || "),
            });
        }
        for (feature, variants) in lexeme.forms {
            for (rank, variant) in variants.into_iter().enumerate() {
                let rank = u16::try_from(rank).map_err(|_| "too many variants in one cell")?;
                registry.forms.push(FormRow {
                    lexeme_id: id.clone(),
                    feature: feature.clone(),
                    rank,
                    form: variant.form,
                    romanization: variant.romanization,
                    source_spelling: variant.source_spelling,
                    source_tags: variant.tags,
                });
            }
        }
    }
    Ok(registry)
}

fn feature_keys<'a>(
    pos: &str,
    form: &'a SourceForm,
    adjective_table_block: u8,
) -> Result<Vec<String>, &'a str> {
    match pos {
        "noun" => noun_features(form),
        "adj" => adjective_features(form, adjective_table_block),
        "verb" => verb_features(form),
        "pron" | "num" | "det" => nominal_closed_features(pos, form),
        _ => Err("unsupported-pos"),
    }
}

fn noun_features(form: &SourceForm) -> Result<Vec<String>, &'static str> {
    let case = one_case(&form.tags).ok_or("noun-missing-or-ambiguous-case")?;
    let number = one_number(&form.tags).ok_or("noun-missing-or-ambiguous-number")?;
    Ok(vec![format!("noun:{case}:{number}")])
}

fn adjective_features(form: &SourceForm, table_block: u8) -> Result<Vec<String>, &'static str> {
    let case = one_case(&form.tags).ok_or("adjective-missing-or-ambiguous-case")?;
    let number = one_number(&form.tags).ok_or("adjective-missing-or-ambiguous-number")?;
    let genders = genders(&form.tags);
    if genders.is_empty() {
        return Err("adjective-missing-gender");
    }
    let adjective_form = match (has(&form.tags, "long-form"), table_block) {
        (true, _) | (false, 2) => "long",
        (false, 1) => "short",
        (false, _) => return Err("adjective-unknown-table-block"),
    };
    let mut keys = Vec::new();
    for gender in genders {
        let animacies: &[&str] = if has(&form.tags, "animate") {
            &["an"]
        } else if has(&form.tags, "inanimate") {
            &["in"]
        } else {
            &["an", "in"]
        };
        for animacy in animacies {
            keys.push(format!(
                "adj:{adjective_form}:{case}:{number}:{gender}:{animacy}"
            ));
        }
    }
    Ok(keys)
}

fn nominal_closed_features(pos: &str, form: &SourceForm) -> Result<Vec<String>, &'static str> {
    let case = one_case(&form.tags).ok_or("closed-class-missing-or-ambiguous-case")?;
    let numbers = if let Some(number) = one_number(&form.tags) {
        vec![number]
    } else if pos == "pron" && has(&form.tags, "reflexive") {
        vec!["sg", "du", "pl"]
    } else {
        return Err("closed-class-missing-or-ambiguous-number");
    };
    let genders = genders(&form.tags);
    let person = one_person(&form.tags);
    let genders = if genders.is_empty() {
        vec![None]
    } else {
        genders.into_iter().map(Some).collect()
    };
    let mut keys = Vec::new();
    for number in numbers {
        for gender in &genders {
            let mut key = format!("decl:{pos}:{case}:{number}");
            if let Some(gender) = gender {
                key.push(':');
                key.push_str(gender);
            }
            if let Some(person) = person {
                key.push(':');
                key.push_str(person);
            }
            keys.push(key);
        }
    }
    Ok(keys)
}

fn verb_features(form: &SourceForm) -> Result<Vec<String>, &'static str> {
    if has(&form.tags, "error-unrecognized-form") {
        return Err("unsafe-verb-error-unrecognized-form");
    }
    if has(&form.tags, "infinitive") {
        return Ok(vec!["verb:infinitive".to_string()]);
    }
    if has(&form.tags, "supine") {
        return Ok(vec!["verb:supine".to_string()]);
    }
    if has(&form.tags, "noun-from-verb") {
        return Ok(vec!["verb:verbal-noun".to_string()]);
    }
    let person = one_person(&form.tags);
    let number = one_number(&form.tags);
    if has(&form.tags, "imperative") {
        return match (person, number) {
            (Some(person), Some(number)) => Ok(vec![format!("verb:imperative:{person}:{number}")]),
            _ => Err("unsafe-verb-imperative-missing-person-or-number"),
        };
    }
    let active = has(&form.tags, "active");
    let passive = has(&form.tags, "passive");
    if active && passive {
        return Err("participle-contradictory-voice");
    }
    if active || passive {
        if one_case(&form.tags).is_some() {
            return Err("declined-participle-not-safely-attributed");
        }
        if person.is_none() && number.is_none() {
            let present = has(&form.tags, "present");
            let past = has(&form.tags, "past");
            let tense = match (present, past) {
                (true, false) => "present",
                (false, true) => "past",
                (true, true) => return Err("participle-contradictory-tense"),
                (false, false) => return Err("participle-missing-tense"),
            };
            let voice = if active { "active" } else { "passive" };
            return Ok(vec![format!("verb:participle:{tense}-{voice}:citation")]);
        }
    }
    for (tag, tense) in [
        ("present", "present"),
        ("imperfect", "imperfect"),
        ("aorist", "aorist"),
    ] {
        if has(&form.tags, tag) {
            return match (person, number) {
                (Some(person), Some(number)) => {
                    Ok(vec![format!("verb:finite:{tense}:{person}:{number}")])
                }
                (Some(_), None) => Err("unsafe-verb-finite-missing-number"),
                (None, _) => Err("unsafe-verb-finite-missing-person"),
            };
        }
    }
    if has(&form.tags, "l-participle")
        && !has_any(
            &form.tags,
            &["present", "imperfect", "aorist", "imperative"],
        )
    {
        let number = number.ok_or("l-participle-missing-number")?;
        let genders = genders(&form.tags);
        if genders.is_empty() {
            return Err("l-participle-missing-gender");
        }
        return Ok(genders
            .into_iter()
            .map(|gender| format!("verb:l-participle:{gender}:{number}"))
            .collect());
    }
    if one_case(&form.tags).is_some() && !genders(&form.tags).is_empty() {
        return Err("declined-participle-not-safely-attributed");
    }
    Err("unsafe-or-unknown-verb-shape")
}

fn normalize_pos(pos: &str) -> Option<&'static str> {
    match pos {
        "noun" | "name" => Some("noun"),
        "adj" => Some("adj"),
        "verb" => Some("verb"),
        "pron" => Some("pron"),
        "num" => Some("num"),
        "det" => Some("det"),
        _ => None,
    }
}

fn extract_gender(entry: &Entry) -> String {
    for form in &entry.forms {
        if has(&form.tags, "canonical") {
            let found = genders(&form.tags);
            if found.len() == 1 {
                return found[0].to_string();
            }
        }
    }
    for template in &entry.head_templates {
        for key in ["g", "1"] {
            if let Some(value) = template.args.get(key) {
                if matches!(value.as_str(), "m" | "f" | "n") {
                    return value.clone();
                }
            }
        }
    }
    String::new()
}

fn extract_animacy(entry: &Entry) -> String {
    let tags = entry
        .forms
        .iter()
        .flat_map(|form| form.tags.iter())
        .chain(entry.senses.iter().flat_map(|sense| sense.tags.iter()));
    let mut animate = false;
    let mut inanimate = false;
    for tag in tags {
        animate |= tag == "animate";
        inanimate |= tag == "inanimate";
    }
    match (animate, inanimate) {
        (true, false) => "an".to_string(),
        (false, true) => "in".to_string(),
        _ => String::new(),
    }
}

fn extract_restriction(entry: &Entry) -> String {
    for template in &entry.head_templates {
        if let Some(value) = template.args.get("n") {
            return match value.as_str() {
                "sg" => "sg",
                "dl" | "du" => "du",
                "pl" => "pl",
                _ => "",
            }
            .to_string();
        }
    }
    String::new()
}

fn normalize_class(raw: &str, lemma: &str, gender: &str) -> String {
    match (raw, gender) {
        ("o-stem", "m") if lemma.ends_with('ъ') => "o-m-hard".to_string(),
        ("o-stem", "m") => "jo-m-soft".to_string(),
        ("o-stem", "n") if lemma.ends_with('о') => "o-n-hard".to_string(),
        ("o-stem", "n") => "jo-n-soft".to_string(),
        ("a-stem", "f") if lemma.ends_with('а') => "a-hard".to_string(),
        ("a-stem", "f") => "ja-soft".to_string(),
        ("i-stem", "m") => "i-m".to_string(),
        ("i-stem", "f") => "i-f".to_string(),
        ("u-stem", _) => "u-m".to_string(),
        ("n-stem", "m") => "n-m".to_string(),
        ("n-stem", "n") => "n-n".to_string(),
        ("nt-stem", _) => "nt-n".to_string(),
        ("r-stem", _) => "r-n".to_string(),
        ("s-stem", _) => "s-n".to_string(),
        ("v-stem", _) => "v-f".to_string(),
        ("IA1" | "IA2" | "II1" | "II2" | "II3", _) => raw.to_string(),
        ("", _) => String::new(),
        _ => format!("raw:{raw}"),
    }
}

fn is_sentinel(form: &SourceForm) -> bool {
    form.form.is_empty()
        || matches!(form.form.as_str(), "-" | "—" | "no-table-tags")
        || has_any(&form.tags, &["table-tags", "class", "inflection-template"])
}

fn public_source_spellings(value: &str) -> Result<Vec<&str>, &'static str> {
    let spellings = if value.contains(", ") {
        value.split(", ").collect::<Vec<_>>()
    } else {
        vec![value]
    };
    for spelling in &spellings {
        if has_wiki_markup(spelling) {
            return Err("template-markup-form");
        }
        if spelling.contains([',', '/', ';']) {
            return Err("contextual-or-unsplit-source-form");
        }
        let normalized =
            canonical_display(spelling).map_err(|_| "invalid-word-level-source-form")?;
        if normalized != *spelling {
            return Err("non-nfc-source-form");
        }
        if !matches!(
            detect_script(spelling),
            Script::Cyrillic | Script::Glagolitic
        ) {
            return Err("non-ocs-script-form");
        }
    }
    Ok(spellings)
}

fn is_safe_single_word(value: &str) -> bool {
    public_source_spellings(value).is_ok_and(|spellings| spellings.len() == 1)
}

fn one_case(tags: &[String]) -> Option<&'static str> {
    one_tag(
        tags,
        &[
            ("nominative", "nom"),
            ("genitive", "gen"),
            ("dative", "dat"),
            ("accusative", "acc"),
            ("instrumental", "ins"),
            ("locative", "loc"),
            ("vocative", "voc"),
        ],
    )
}

fn one_number(tags: &[String]) -> Option<&'static str> {
    one_tag(
        tags,
        &[("singular", "sg"), ("dual", "du"), ("plural", "pl")],
    )
}

fn one_person(tags: &[String]) -> Option<&'static str> {
    one_tag(
        tags,
        &[
            ("first-person", "1"),
            ("second-person", "2"),
            ("third-person", "3"),
        ],
    )
}

fn genders(tags: &[String]) -> Vec<&'static str> {
    [("masculine", "m"), ("feminine", "f"), ("neuter", "n")]
        .into_iter()
        .filter_map(|(tag, code)| has(tags, tag).then_some(code))
        .collect()
}

fn one_tag(tags: &[String], choices: &[(&str, &'static str)]) -> Option<&'static str> {
    let mut found = choices
        .iter()
        .filter_map(|(tag, code)| has(tags, tag).then_some(*code));
    let first = found.next()?;
    found.next().is_none().then_some(first)
}

fn has(tags: &[String], wanted: &str) -> bool {
    tags.iter().any(|tag| tag == wanted)
}

fn has_any(tags: &[String], wanted: &[&str]) -> bool {
    wanted.iter().any(|tag| has(tags, tag))
}

fn insert_alias(
    aliases: &mut BTreeMap<String, BTreeSet<String>>,
    key: String,
    relation: &str,
    spelling: &str,
) -> Result<(), String> {
    let spelling = serde_json::to_string(spelling)
        .map_err(|error| format!("alias spelling serialization failed: {error}"))?;
    aliases
        .entry(key)
        .or_default()
        .insert(format!("{relation}:{spelling}"));
    Ok(())
}

fn feature_group(feature: &str) -> &str {
    if feature.starts_with("adj:comparative") {
        "adj-comparative"
    } else if feature.starts_with("adj:long") {
        "adj-long"
    } else if feature.starts_with("adj:short") {
        "adj-short"
    } else if feature.starts_with("verb:finite") {
        "verb-finite"
    } else if feature.starts_with("verb:participle") {
        "verb-participle"
    } else if feature.starts_with("verb:l-participle") {
        "verb-l-participle"
    } else if feature.starts_with("verb:") {
        "verb-other"
    } else if feature.starts_with("noun:") {
        "noun"
    } else {
        "closed-class"
    }
}

fn tag_signature(form: &SourceForm) -> String {
    let mut tags = form.tags.iter().map(String::as_str).collect::<Vec<_>>();
    tags.sort_unstable();
    tags.dedup();
    if tags.is_empty() {
        "(none)".to_string()
    } else {
        tags.join("+")
    }
}

fn ambiguous_lemma_pos_pairs(registry: &Registry) -> usize {
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|lexeme| (lexeme.id.as_str(), lexeme.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut groups: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    for alias in &registry.aliases {
        if let Some(pos) = pos_by_id.get(alias.lexeme_id.as_str()) {
            groups
                .entry((alias.key.as_str(), *pos))
                .or_default()
                .insert(alias.lexeme_id.as_str());
        }
    }
    groups.values().filter(|ids| ids.len() > 1).count()
}

fn signature(lexeme: &PendingLexeme) -> String {
    let mut out = format!(
        "{}\0{}\0{}\0{}\0{}\0{}\0{}\0",
        lexeme.lemma,
        lexeme.page_word,
        lexeme.pos,
        lexeme.class,
        lexeme.gender,
        lexeme.animacy,
        lexeme.number_restriction
    );
    for (feature, variants) in &lexeme.forms {
        for variant in variants {
            out.push_str(feature);
            out.push('\0');
            out.push_str(&variant.form);
            out.push('\0');
            out.push_str(&variant.romanization);
            out.push('\0');
        }
    }
    out
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_default() += 1;
}

fn source_metadata(path: &Path) -> Result<SourceMetadata, Box<dyn Error>> {
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
    Ok(SourceMetadata {
        schema_version: REGISTRY_SCHEMA,
        input_file: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source.jsonl")
            .to_string(),
        bytes: fs::metadata(path)?.len(),
        sha256: format!("{:x}", hasher.finalize()),
    })
}

fn registry_text(registry: &Registry) -> (String, String, String, String) {
    let mut lexemes = String::from(
        "id\tlemma\tpage_word\tkey\tpos\tclass\traw_class\tgender\tanimacy\tnumber_restriction\thead_templates\tsignature\n",
    );
    for row in &registry.lexemes {
        lexemes.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.id,
            row.lemma,
            row.page_word,
            row.key,
            row.pos,
            row.class,
            row.raw_class,
            row.gender,
            row.animacy,
            row.number_restriction,
            row.head_templates,
            row.signature
        ));
    }
    let mut aliases = String::from("key\tlexeme_id\tsource_spellings\n");
    for row in &registry.aliases {
        aliases.push_str(&format!(
            "{}\t{}\t{}\n",
            row.key, row.lexeme_id, row.source_spellings
        ));
    }
    let mut forms = String::from(
        "lexeme_id\tfeature\trank\tform\tromanization\tsource_spelling\tsource_tags\n",
    );
    for row in &registry.forms {
        forms.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.lexeme_id,
            row.feature,
            row.rank,
            row.form,
            row.romanization,
            row.source_spelling,
            row.source_tags
        ));
    }
    let mut verb_metadata = String::from(
        "lexeme_id\tsystem\tanalysis_rank\tfield\tvalue\tprovenance\tsource_feature\tsource_form\tcrosscheck_features\tauthority\n",
    );
    for row in &registry.verb_metadata {
        verb_metadata.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.lexeme_id,
            row.system,
            row.analysis_rank,
            row.field,
            row.value,
            row.provenance,
            row.source_feature,
            row.source_form,
            row.crosscheck_features,
            row.authority,
        ));
    }
    (lexemes, aliases, forms, verb_metadata)
}

pub fn load_registry(dir: &Path) -> Result<Registry, Box<dyn Error>> {
    let mut registry = Registry::default();
    for (line_number, line) in fs::read_to_string(dir.join("lexemes.tsv"))?
        .lines()
        .enumerate()
        .skip(1)
    {
        let columns: Vec<_> = line.split('\t').collect();
        if !matches!(columns.len(), 9 | 12) {
            return Err(format!("invalid lexemes.tsv row {}", line_number + 1).into());
        }
        registry.lexemes.push(if columns.len() == 12 {
            LexemeRow {
                id: columns[0].to_string(),
                lemma: columns[1].to_string(),
                page_word: columns[2].to_string(),
                key: columns[3].to_string(),
                pos: columns[4].to_string(),
                class: columns[5].to_string(),
                raw_class: columns[6].to_string(),
                gender: columns[7].to_string(),
                animacy: columns[8].to_string(),
                number_restriction: columns[9].to_string(),
                head_templates: columns[10].to_string(),
                signature: columns[11].to_string(),
            }
        } else {
            LexemeRow {
                id: columns[0].to_string(),
                lemma: columns[1].to_string(),
                page_word: columns[1].to_string(),
                key: columns[2].to_string(),
                pos: columns[3].to_string(),
                class: columns[4].to_string(),
                raw_class: String::new(),
                gender: columns[5].to_string(),
                animacy: columns[6].to_string(),
                number_restriction: columns[7].to_string(),
                head_templates: "[]".to_string(),
                signature: columns[8].to_string(),
            }
        });
    }
    for (line_number, line) in fs::read_to_string(dir.join("aliases.tsv"))?
        .lines()
        .enumerate()
        .skip(1)
    {
        let columns: Vec<_> = line.split('\t').collect();
        if !matches!(columns.len(), 2 | 3) {
            return Err(format!("invalid aliases.tsv row {}", line_number + 1).into());
        }
        registry.aliases.push(AliasRow {
            key: columns[0].to_string(),
            lexeme_id: columns[1].to_string(),
            source_spellings: columns.get(2).map_or_else(
                || format!("legacy-key:{}", columns[0]),
                |value| (*value).to_string(),
            ),
        });
    }
    for (line_number, line) in fs::read_to_string(dir.join("forms.tsv"))?
        .lines()
        .enumerate()
        .skip(1)
    {
        let columns: Vec<_> = line.split('\t').collect();
        if !matches!(columns.len(), 6 | 7) {
            return Err(format!("invalid forms.tsv row {}", line_number + 1).into());
        }
        registry.forms.push(FormRow {
            lexeme_id: columns[0].to_string(),
            feature: columns[1].to_string(),
            rank: columns[2].parse()?,
            form: columns[3].to_string(),
            romanization: columns[4].to_string(),
            source_spelling: if columns.len() == 7 {
                columns[5].to_string()
            } else {
                columns[3].to_string()
            },
            source_tags: columns[columns.len() - 1].to_string(),
        });
    }
    let metadata_path = dir.join("verb_metadata.tsv");
    if metadata_path.exists() {
        for (line_number, line) in fs::read_to_string(metadata_path)?
            .lines()
            .enumerate()
            .skip(1)
        {
            let columns = line.split('\t').collect::<Vec<_>>();
            if columns.len() != 10 {
                return Err(format!("invalid verb_metadata.tsv row {}", line_number + 1).into());
            }
            registry.verb_metadata.push(VerbMetadataRow {
                lexeme_id: columns[0].to_string(),
                system: columns[1].to_string(),
                analysis_rank: columns[2].parse()?,
                field: columns[3].to_string(),
                value: columns[4].to_string(),
                provenance: columns[5].to_string(),
                source_feature: columns[6].to_string(),
                source_form: columns[7].to_string(),
                crosscheck_features: columns[8].to_string(),
                authority: columns[9].to_string(),
            });
        }
    }
    Ok(registry)
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(tags: &[&str]) -> SourceForm {
        SourceForm {
            form: "форма".to_string(),
            tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
            raw_tags: Vec::new(),
            source: "declension".to_string(),
            romanization: String::new(),
        }
    }

    #[test]
    fn noun_feature_is_complete_and_order_independent() {
        assert_eq!(
            noun_features(&form(&["dual", "genitive"])).expect("safe"),
            ["noun:gen:du"]
        );
    }

    #[test]
    fn malformed_finite_verb_is_rejected() {
        assert_eq!(
            verb_features(&form(&["error-unrecognized-form", "present", "singular"])),
            Err("unsafe-verb-error-unrecognized-form")
        );
    }

    #[test]
    fn spurious_l_participle_is_ignored_only_for_complete_finite_cell() {
        let keys = verb_features(&form(&[
            "first-person",
            "l-participle",
            "present",
            "singular",
        ]))
        .expect("complete finite signature");
        assert_eq!(keys, ["verb:finite:present:1:sg"]);
    }

    #[test]
    fn present_participle_citations_precede_finite_tense_mapping() {
        assert_eq!(
            verb_features(&form(&["present", "active"])).expect("ordered citation"),
            ["verb:participle:present-active:citation"]
        );
        assert_eq!(
            verb_features(&form(&["passive", "present", "present"]))
                .expect("reordered duplicate tags are deterministic"),
            ["verb:participle:present-passive:citation"]
        );
        assert_eq!(
            verb_features(&form(&[
                "present",
                "active",
                "genitive",
                "singular",
                "masculine",
            ])),
            Err("declined-participle-not-safely-attributed")
        );
        assert_eq!(
            verb_features(&form(&["present", "past", "active"])),
            Err("participle-contradictory-tense")
        );
        assert_eq!(
            verb_features(&form(&["present", "active", "passive"])),
            Err("participle-contradictory-voice")
        );
    }

    #[test]
    fn adjective_sentinel_blocks_separate_short_and_long_cells() {
        let source = form(&["nominative", "singular", "masculine"]);
        assert_eq!(
            adjective_features(&source, 1).expect("short block"),
            ["adj:short:nom:sg:m:an", "adj:short:nom:sg:m:in"]
        );
        assert_eq!(
            adjective_features(&source, 2).expect("long block"),
            ["adj:long:nom:sg:m:an", "adj:long:nom:sg:m:in"]
        );
        assert_eq!(
            adjective_features(&source, 3),
            Err("adjective-unknown-table-block")
        );
    }

    #[test]
    fn reflexive_pronoun_without_number_expands_but_personal_rows_stay_separate() {
        let reflexive = SourceForm {
            form: "сѧ".to_string(),
            tags: ["accusative", "personal", "pronoun", "reflexive"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            raw_tags: Vec::new(),
            source: "declension".to_string(),
            romanization: "sę".to_string(),
        };
        assert_eq!(
            nominal_closed_features("pron", &reflexive).expect("number-invariant reflexive"),
            [
                "decl:pron:acc:sg".to_string(),
                "decl:pron:acc:du".to_string(),
                "decl:pron:acc:pl".to_string(),
            ]
        );

        let entry: Entry = serde_json::from_str(
            r#"{
                "word":"сѧ","lang_code":"cu","pos":"pron",
                "head_templates":[{"name":"cu-pron","args":{}}],
                "forms":[
                  {"form":"l-self","source":"declension","tags":["inflection-template"]},
                  {"form":"азъ","source":"declension","tags":["nominative","singular","first-person","personal","pronoun"]},
                  {"form":"сѧ","source":"declension","tags":["accusative","personal","pronoun","reflexive"]}
                ]
            }"#,
        )
        .expect("fixture JSON");
        let mut report = ExtractionReport::default();
        let pending = pending_lexeme(&entry, "pron", &mut report)
            .expect("safe entry")
            .expect("reflexive table retained");
        assert_eq!(pending.lemma, "сѧ");
        assert_eq!(pending.forms.len(), 3);
        assert!(
            pending
                .forms
                .values()
                .flatten()
                .all(|form| form.form == "сѧ")
        );
    }

    #[test]
    fn form_of_pages_do_not_become_duplicate_lexemes() {
        let entry: Entry = serde_json::from_str(
            r#"{
                "word":"себе","lang_code":"cu","pos":"pron",
                "senses":[{"tags":["form-of"],"form_of":[{"word":"сѧ"}]}]
            }"#,
        )
        .expect("fixture JSON");
        let mut report = ExtractionReport::default();
        assert!(
            pending_lexeme(&entry, "pron", &mut report)
                .expect("safe skip")
                .is_none()
        );
        assert_eq!(report.dropped_by_reason["form-of-entry-not-lexeme"], 1);
    }

    #[test]
    fn mediawiki_placeholders_never_become_lemmas_or_forms() {
        let entry: Entry = serde_json::from_str(
            r#"{
                "word":"ломъ","lang_code":"cu","pos":"noun",
                "forms":[
                  {"form":"ло{{{2}}}мъ","source":"declension","tags":["canonical","nominative","singular"]},
                  {"form":"ло{{{2}}}ма","source":"declension","tags":["genitive","singular"]}
                ]
            }"#,
        )
        .expect("fixture JSON");
        let mut report = ExtractionReport::default();
        assert!(
            pending_lexeme(&entry, "noun", &mut report)
                .expect("safe rejection")
                .is_none()
        );
        assert_eq!(report.dropped_by_reason["template-markup-form"], 2);
        assert_eq!(report.dropped_by_reason["entry-without-safe-cells"], 1);
    }

    #[test]
    fn source_alternative_lists_split_but_phrases_fail_closed() {
        assert_eq!(
            public_source_spellings("чьсо, чесого, чьсого").expect("word alternatives"),
            ["чьсо", "чесого", "чьсого"]
        );
        assert_eq!(
            public_source_spellings("не сѫтъ"),
            Err("invalid-word-level-source-form")
        );
        assert_eq!(
            public_source_spellings("ни/о/при чесомьже"),
            Err("contextual-or-unsplit-source-form")
        );
    }

    #[test]
    fn atomic_batch_preparation_failure_preserves_existing_targets() {
        let root = std::env::temp_dir().join(format!(
            "old-church-slavonic-atomic-failure-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale test directory");
        }
        fs::create_dir_all(&root).expect("create test directory");
        let existing = root.join("existing.txt");
        fs::write(&existing, b"original").expect("write original fixture");
        let unavailable = root.join("missing-parent/new.txt");

        let result = atomic_write_batch(&[
            (existing.clone(), b"replacement"),
            (unavailable.clone(), b"new"),
        ]);

        assert!(result.is_err());
        assert_eq!(
            fs::read(&existing).expect("read preserved target"),
            b"original"
        );
        assert!(!unavailable.exists());
        assert!(
            fs::read_dir(&root)
                .expect("read test directory")
                .all(|entry| !entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .contains(".refresh-"))
        );

        let directory_target = root.join("directory-target");
        fs::create_dir(&directory_target).expect("create non-file target");
        let result = atomic_write_batch(&[(directory_target.clone(), b"replacement")]);
        assert!(result.is_err());
        assert!(directory_target.is_dir());
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn atomic_batch_replaces_all_targets_together() {
        let root = std::env::temp_dir().join(format!(
            "old-church-slavonic-atomic-success-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale test directory");
        }
        fs::create_dir_all(&root).expect("create test directory");
        let first = root.join("first.txt");
        let second = root.join("second.txt");
        fs::write(&first, b"old-first").expect("write first fixture");
        fs::write(&second, b"old-second").expect("write second fixture");

        atomic_write_batch(&[
            (first.clone(), b"new-first"),
            (second.clone(), b"new-second"),
        ])
        .expect("replace batch");

        assert_eq!(fs::read(&first).expect("read first target"), b"new-first");
        assert_eq!(
            fs::read(&second).expect("read second target"),
            b"new-second"
        );
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn content_change_rekeys_lexeme_and_rewrites_every_alias_reference() {
        let entry = |genitive: &str| {
            serde_json::from_str::<Entry>(&format!(
                r#"{{
                    "word":"домъ","lang_code":"cu","pos":"noun",
                    "forms":[
                      {{"form":"домъ","source":"declension","tags":["nominative","singular"]}},
                      {{"form":"{genitive}","source":"declension","tags":["genitive","singular"]}}
                    ]
                }}"#
            ))
            .expect("fixture JSON")
        };
        let build = |entry: Entry| {
            let mut report = ExtractionReport::default();
            let pending = pending_lexeme(&entry, "noun", &mut report)
                .expect("safe entry")
                .expect("fixture lexeme");
            finalize(vec![pending], &mut report).expect("finalized registry")
        };

        let before = build(entry("дома"));
        let after = build(entry("домоу"));
        assert_ne!(before.lexemes[0].id, after.lexemes[0].id);
        assert!(
            before
                .aliases
                .iter()
                .all(|alias| alias.lexeme_id == before.lexemes[0].id)
        );
        assert!(
            after
                .aliases
                .iter()
                .all(|alias| alias.lexeme_id == after.lexemes[0].id)
        );
        assert!(
            after
                .forms
                .iter()
                .all(|form| form.lexeme_id == after.lexemes[0].id)
        );
    }
}
