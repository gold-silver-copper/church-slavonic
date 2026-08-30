//! Refresh orchestration: filter -> extract -> emit.
//!
//! There is no persisted state to reconcile: each refresh regenerates the
//! tables from the sources alone. The only guards are the parse-failure
//! threshold inside the Kaikki reader (a wiktextract schema break) and the
//! duplicate-key check inside [`generate_tables`]. `--checks-only` measures
//! accuracy without touching the generated tables.

use crate::args::Config;
use crate::bootstrap::generate_tables;
use crate::checks::run_checks;
use crate::extract::{Source, disagreements, finalize, gather, gather_sources};
use crate::{alypy, kaikki, polyakov, ruwiktionary, treebank};
use std::error::Error;
use std::fs;
use std::path::Path;

/// The pinned table sources, relative to `--sources` (the fifth, the UD
/// PROIEL train split, lives at `treebank::UD_PROIEL_SOURCE`).
pub const KAIKKI_SOURCE: &str =
    "english-wiktionary-ocs/kaikki.org-dictionary-OldChurchSlavonic.jsonl";
pub const ALYPY_SOURCE: &str = "alypy-grammar";
pub const POLYAKOV_SOURCE: &str = "polyakov";
pub const RUWIKTIONARY_SOURCE: &str =
    "ruwiktionary-cu/kaikki.org-dictionary-Церковнославянский.jsonl";

/// Run one full data refresh. On success the four generated PHF tables are a
/// pure deterministic function of the (filtered) sources.
pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.generated_dir)?;
    fs::create_dir_all(&config.artifacts_dir)?;

    let kaikki_out = config.artifacts_dir.join(Source::Kaikki.intermediate());
    let alypy_out = config.artifacts_dir.join(Source::Alypy.intermediate());
    let polyakov_out = config.artifacts_dir.join(Source::Polyakov.intermediate());
    let ruwiktionary_out = config
        .artifacts_dir
        .join(Source::RuWiktionary.intermediate());
    let kaikki_src = config.sources_dir.join(KAIKKI_SOURCE);
    let alypy_src = config.sources_dir.join(ALYPY_SOURCE);
    let polyakov_src = config.sources_dir.join(POLYAKOV_SOURCE);
    let ruwiktionary_src = config.sources_dir.join(RUWIKTIONARY_SOURCE);
    let ud_proiel_out = config.artifacts_dir.join(Source::UdProiel.intermediate());
    let ud_proiel_src = config.sources_dir.join(treebank::UD_PROIEL_SOURCE);

    if kaikki_src.is_file() {
        kaikki::filter(&kaikki_src, &kaikki_out)?;
    } else {
        reuse_or_fail(&kaikki_out, &kaikki_src)?;
    }
    if alypy_src.is_dir() {
        alypy::filter(&alypy_src, &alypy_out)?;
    } else {
        reuse_or_fail(&alypy_out, &alypy_src)?;
    }
    if polyakov_src.is_dir() {
        polyakov::filter(&polyakov_src, &polyakov_out)?;
    } else {
        reuse_or_fail(&polyakov_out, &polyakov_src)?;
    }
    if ruwiktionary_src.is_file() {
        ruwiktionary::filter(&ruwiktionary_src, &ruwiktionary_out)?;
    } else {
        reuse_or_fail(&ruwiktionary_out, &ruwiktionary_src)?;
    }
    if ud_proiel_src.is_dir() {
        treebank::filter_train(&config.sources_dir, &config.artifacts_dir, &ud_proiel_out)?;
    } else {
        reuse_or_fail(&ud_proiel_out, &ud_proiel_src)?;
    }

    if config.checks_only {
        run_checks(
            &config.artifacts_dir,
            &config.artifacts_dir,
            &config.sources_dir,
        )?;
        println!("Checks-only run: generated tables untouched.");
        return Ok(());
    }

    let lexemes = gather(&config.artifacts_dir)?;
    let polyakov = gather_sources(&config.artifacts_dir, &[Source::Polyakov])?;
    for (source, name) in [
        (Source::Alypy, "Alypy"),
        (Source::RuWiktionary, "ru.wiktionary"),
    ] {
        let (exact, beyond) = disagreements(
            &gather_sources(&config.artifacts_dir, &[source])?,
            &polyakov,
        );
        println!(
            "{name}/Polyakov slots attested by both with a different primary: {exact} ({beyond} beyond accent and letter conventions) — variants by the sort."
        );
    }
    let tables = finalize(&lexemes);
    generate_tables(&tables, &config.generated_dir)?;
    accent_class_agreement(&tables, &config.artifacts_dir);
    println!(
        "Refresh complete: {} noun, {} adjective, {} verb, {} pronoun rows regenerated in {}.",
        tables.noun.len(),
        tables.adj.len(),
        tables.verb.len(),
        tables.pronoun.len(),
        config.generated_dir.display()
    );
    Ok(())
}

/// The accent-pattern cross-check: how uniform are the adopted tokens
/// within each Polyakov accent class? Reported, not enforced — a low rate
/// means the inference is reading noise.
fn accent_class_agreement(tables: &crate::extract::Tables, artifacts_dir: &Path) {
    use church_slavonic_core::schema as sch;
    use std::collections::BTreeMap;
    let path = artifacts_dir.join(Source::Polyakov.intermediate());
    let Ok(entries) = polyakov::read(&path) else {
        return;
    };
    let class_of: BTreeMap<String, String> = entries
        .into_iter()
        .filter(|e| !e.class.is_empty())
        .map(|e| (e.lemma, e.class))
        .collect();
    let mut by_class: BTreeMap<&str, BTreeMap<&str, u64>> = BTreeMap::new();
    for (rows, accent_cell) in [
        (&tables.noun, sch::NOUN_ACCENT_CELL),
        (&tables.adj, sch::ADJ_ACCENT_CELL),
        (&tables.verb, sch::VERB_ACCENT_CELL),
    ] {
        for (key, cells) in rows {
            let Some(lemma) = key.strip_prefix("syn:") else {
                continue;
            };
            let token = &cells[accent_cell];
            if token.is_empty() {
                continue;
            }
            if let Some(class) = class_of.get(lemma) {
                *by_class
                    .entry(class.as_str())
                    .or_default()
                    .entry(token.as_str())
                    .or_default() += 1;
            }
        }
    }
    let (mut majority, mut total) = (0u64, 0u64);
    for tokens in by_class.values() {
        let sum: u64 = tokens.values().sum();
        majority += tokens.values().max().copied().unwrap_or(0);
        total += sum;
    }
    if total > 0 {
        println!(
            "Accent-pattern/Polyakov-class agreement: {majority}/{total} adopted tokens share \
             their class's majority token across {} classes.",
            by_class.len()
        );
    }
}

fn reuse_or_fail(intermediate: &Path, source: &Path) -> Result<(), Box<dyn Error>> {
    if intermediate.exists() {
        println!("Reusing filtered dataset at {}", intermediate.display());
        Ok(())
    } else {
        Err(format!(
            "no source at {} and no filtered dataset at {} — download the pinned source (see the README) or pass `--sources DIR`.",
            source.display(),
            intermediate.display()
        )
        .into())
    }
}
