//! Workspace task runner (`cargo xtask <command>`).
//!
//! - `refresh-data` — full source-driven regeneration of the PHF tables
//!   (delegates to the extractor binary in release mode);
//! - `check-registry` — the source-free CI gate: the committed tables are
//!   well-formed, have unique keys and correct arity, and preserve rule/table
//!   layering (no cell merely duplicates the rule engine);
//! - `accuracy` — source-driven measurement of rules + tables together; prints
//!   the README's two tables.
//!
//! Sense-numbered `_<n>` keys are DETERMINISTIC but NOT immutable: `refresh-data`
//! regenerates every table from the sources alone (see `extractor::assign`), so
//! an upstream data change can renumber a lemma's keys. There is no lockfile,
//! override file, human-review flow, or cross-version immutability gate — the
//! committed generated tables are the whole artifact.

use extractor::bootstrap::audit_tables;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::{self, Command};

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("refresh-data") => run_extractor(args.collect(), false),
        Some("check-registry") => check_registry(),
        Some("check-witnesses") => check_witnesses(),
        Some("accuracy") => run_extractor(args.collect(), true),
        Some("-h") | Some("--help") | None => {
            print_usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

// --------------------------------------------------------------------------
// check-registry: source-free structural + layering gate over the committed
// tables. It CANNOT verify a row's attested VALUES are correct — `cargo xtask
// accuracy` (with the sources) is the authoritative value check.
// --------------------------------------------------------------------------
/// check-witnesses: every row of data/witnesses.tsv must quote a line
/// findable VERBATIM in its named file under the vertograd checkout
/// (`VERTOGRAD_DIR`, default `../vertograd`). Offline-soft: absent files
/// warn and skip; a present file whose quote is missing FAILS.
fn check_witnesses() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let witnesses = root.join("data/witnesses.tsv");
    let vertograd = std::env::var_os("VERTOGRAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("../vertograd"));
    let text = std::fs::read_to_string(&witnesses)?;
    let mut checked = 0;
    let mut skipped = 0;
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 7 {
            return Err(format!("witnesses.tsv: malformed line: {line}").into());
        }
        let (file, quote) = (cols[5], cols[6]);
        let path = vertograd.join(file);
        let Ok(content) = std::fs::read_to_string(&path) else {
            eprintln!("check-witnesses: {} absent — skipped (offline)", path.display());
            skipped += 1;
            continue;
        };
        if !content.contains(quote) {
            return Err(format!(
                "check-witnesses: quote not found in {}: {quote}",
                path.display()
            )
            .into());
        }
        checked += 1;
    }
    println!("check-witnesses: OK — {checked} citation(s) verified, {skipped} skipped.");
    Ok(())
}

fn check_registry() -> Result<(), Box<dyn Error>> {
    let generated_dir = workspace_root()?.join("crates/church-slavonic/generated");
    let violations = audit_tables(&generated_dir)?;
    if violations.is_empty() {
        println!(
            "check-registry: OK — tables are well-formed, keys are unique, and rule/table layering \
             holds. (Structural + layering only; run `cargo xtask accuracy` to verify the forms \
             against the sources.)"
        );
        Ok(())
    } else {
        eprintln!(
            "check-registry: FAILED — {} violation(s):",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        process::exit(1);
    }
}

// --------------------------------------------------------------------------
// refresh-data / accuracy: run the extractor over the sources. `accuracy`
// links the facade (feature `checks`) and measures the CURRENT committed
// tables + rule engine; it reuses the cached filtered sources in
// `data/intermediate` when the downloads are absent.
// --------------------------------------------------------------------------
fn run_extractor(args: Vec<String>, checks_only: bool) -> Result<(), Box<dyn Error>> {
    let mut sources: Option<PathBuf> = None;
    let mut artifacts_dir: Option<PathBuf> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--sources" => sources = Some(PathBuf::from(req(&mut iter, "--sources")?)),
            "--artifacts-dir" => {
                artifacts_dir = Some(PathBuf::from(req(&mut iter, "--artifacts-dir")?))
            }
            "-h" | "--help" => {
                eprintln!(
                    "Usage: cargo xtask {} [--sources DIR] [--artifacts-dir DIR]",
                    if checks_only {
                        "accuracy"
                    } else {
                        "refresh-data"
                    }
                );
                eprintln!(
                    "Sources default to references/downloads (see the README's source table)."
                );
                return Ok(());
            }
            other => return Err(format!("unknown flag: {other}").into()),
        }
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command
        .current_dir(workspace_root()?)
        .args(["run", "-p", "extractor", "--release"]);
    if checks_only {
        command.args(["--features", "checks"]);
    }
    command.arg("--");
    if let Some(sources) = sources {
        command.arg("--sources").arg(sources);
    }
    if let Some(dir) = artifacts_dir {
        command.arg("--artifacts-dir").arg(dir);
    }
    if checks_only {
        command.arg("--checks-only");
    }
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        process::exit(status.code().unwrap_or(1));
    }
}

fn req(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Box<dyn Error>> {
    iter.next()
        .ok_or_else(|| format!("expected a value after `{flag}`").into())
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  refresh-data     Regenerate the PHF tables from the pinned sources");
    eprintln!(
        "  check-registry   Source-free gate: tables well-formed, keys unique, layering holds"
    );
    eprintln!("  accuracy         Measure % of attested source slots the library reproduces");
}
