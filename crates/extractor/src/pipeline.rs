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
use crate::extract::{ALYPY_INTERMEDIATE, KAIKKI_INTERMEDIATE, finalize, gather};
use crate::{alypy, kaikki};
use std::error::Error;
use std::fs;
use std::path::Path;

/// The two pinned sources, relative to `--sources`.
pub const KAIKKI_SOURCE: &str =
    "english-wiktionary-ocs/kaikki.org-dictionary-OldChurchSlavonic.jsonl";
pub const ALYPY_SOURCE: &str = "alypy-grammar";

/// Run one full data refresh. On success the four generated PHF tables are a
/// pure deterministic function of the (filtered) sources.
pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.generated_dir)?;
    fs::create_dir_all(&config.artifacts_dir)?;

    let kaikki_out = config.artifacts_dir.join(KAIKKI_INTERMEDIATE);
    let alypy_out = config.artifacts_dir.join(ALYPY_INTERMEDIATE);
    let kaikki_src = config.sources_dir.join(KAIKKI_SOURCE);
    let alypy_src = config.sources_dir.join(ALYPY_SOURCE);

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

    if config.checks_only {
        run_checks(&config.artifacts_dir, &config.artifacts_dir)?;
        println!("Checks-only run: generated tables untouched.");
        return Ok(());
    }

    let lexemes = gather(&config.artifacts_dir)?;
    let tables = finalize(&lexemes);
    generate_tables(&tables, &config.generated_dir)?;
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
