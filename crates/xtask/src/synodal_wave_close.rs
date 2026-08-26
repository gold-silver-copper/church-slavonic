//! One-command wave close: the entire closing suite in the canonical order,
//! with one pass/fail table and per-step timings.
//!
//! The ordering of the closing suite is load-bearing (`synodal-lexical-union`
//! reads lexemes, reviews, and family reviews, so it must be regenerated
//! last; `synodal-marginal-recovery` reads family decisions), and running the
//! sixteen commands by hand mis-ordered them four separate times during
//! v0.12, each time breaking CI on a stale derived artifact. Centralizing the
//! order here — and pointing the CI structural job at `--check` — makes local
//! and CI runs order-identical by construction.
//!
//! Modes:
//! - `--check` (CI-safe): read-only gates only, no artifact regenerated.
//! - default (local): `--check` plus `cargo fmt --check`, clippy, and the
//!   workspace test suite.
//! - `--fix` (local): regenerate every derived artifact first, in canonical
//!   order, then run the default gates. On an incomplete family-review
//!   top-200, the undecided proposals are printed as ready-to-review stubs.

use std::{error::Error, path::Path, process::Command, time::Instant};

use crate::report_io::read_tsv;

struct StepResult {
    name: &'static str,
    seconds: f64,
    error: Option<String>,
}

fn step(
    results: &mut Vec<StepResult>,
    name: &'static str,
    action: impl FnOnce() -> Result<(), Box<dyn Error>>,
) {
    let start = Instant::now();
    let error = action().err().map(|error| error.to_string());
    results.push(StepResult {
        name,
        seconds: start.elapsed().as_secs_f64(),
        error,
    });
}

fn with_args(
    arguments: &[&str],
    root: &Path,
    run: impl FnOnce(&mut std::vec::IntoIter<String>, &Path) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut iterator = arguments
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect::<Vec<_>>()
        .into_iter();
    run(&mut iterator, root)
}

fn cargo(arguments: &[&str], root: &Path) -> Result<(), Box<dyn Error>> {
    let status = Command::new("cargo")
        .args(arguments)
        .current_dir(root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", arguments.join(" ")).into())
    }
}

fn print_undecided_family_stubs(root: &Path) {
    let queue_path = root.join("reports/synodal-family-review-queue.tsv");
    let decisions_path = root.join("data/synodal/family_reviews.tsv");
    let (Ok(queue), Ok(decisions)) = (read_tsv(&queue_path), read_tsv(&decisions_path)) else {
        return;
    };
    let decided: std::collections::BTreeSet<&str> =
        decisions.rows.iter().map(|row| row[0].as_str()).collect();
    let (Ok(rank), Ok(candidate), Ok(lemma), Ok(surfaces)) = (
        queue.index("rank"),
        queue.index("candidate_id"),
        queue.index("proposed_lemma"),
        queue.index("surfaces"),
    ) else {
        return;
    };
    eprintln!("undecided top-200 family proposals (append a decision row to family_reviews.tsv):");
    for row in &queue.rows {
        if row[rank].parse::<usize>().is_ok_and(|value| value <= 200)
            && !decided.contains(row[candidate].as_str())
        {
            eprintln!(
                "  rank {}\t{}\t{}\t{}",
                row[rank], row[candidate], row[lemma], row[surfaces]
            );
        }
    }
}

pub(crate) fn run(
    arguments: &mut dyn Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mode = arguments.next();
    let (fix, check_only) = match mode.as_deref() {
        Some("--fix") => (true, false),
        Some("--check") => (false, true),
        None => (false, false),
        Some(other) => return Err(format!("unknown synodal-wave-close mode {other:?}").into()),
    };
    let mut results = Vec::new();

    if fix {
        // Canonical regeneration order. The union ledger is regenerated LAST
        // because it reads lexemes, lexical reviews, and family reviews.
        step(&mut results, "regenerate family-review-queue", || {
            with_args(&[], root, crate::synodal_family_review::run)
        });
        step(&mut results, "regenerate accent-fit report", || {
            with_args(&[], root, crate::synodal_accent_fit::run)
        });
        step(&mut results, "regenerate prediction feed", || {
            with_args(&[], root, crate::synodal_predict::run)
        });
        step(&mut results, "regenerate marginal recovery", || {
            with_args(&[], root, crate::synodal_marginal_recovery::run)
        });
        step(&mut results, "regenerate lexical union (last)", || {
            with_args(&[], root, crate::synodal_lexical_union::run)
        });
        step(&mut results, "regenerate coverage fixture", || {
            with_args(
                &["--offline", "--fixture"],
                root,
                crate::synodal_coverage::run,
            )
        });
        step(&mut results, "cargo fmt", || cargo(&["fmt", "--all"], root));
    }

    step(&mut results, "fixture bootstrap", || {
        with_args(&[], root, crate::synodal::fixture_bootstrap)
    });
    step(&mut results, "coverage fixture --check", || {
        with_args(
            &["--offline", "--fixture", "--check"],
            root,
            crate::synodal_coverage::run,
        )
    });
    step(&mut results, "coverage floors", || {
        crate::synodal_coverage::check_committed_floors(root)
    });
    step(&mut results, "predict --check", || {
        with_args(&["--check"], root, crate::synodal_predict::run)
    });
    step(&mut results, "check-structure", crate::check_structure);
    step(&mut results, "synodal-check", || {
        crate::synodal::check(root)
    });
    step(&mut results, "synodal-guard-witnesses", || {
        crate::synodal::guard_witnesses(root)
    });
    // These two gates recompute from the gitignored intermediate corpus, so
    // they can only run where it exists (locally, or after a bootstrap). CI
    // enforces their committed artifacts through synodal-check and the
    // generated-tree-stays-clean step instead.
    let has_intermediates = root
        .join("data/intermediate/synodal/adapter-reports.json")
        .is_file();
    if has_intermediates {
        step(&mut results, "accent-fit --check", || {
            with_args(&["--check"], root, crate::synodal_accent_fit::run)
        });
        let family_gate_index = results.len();
        step(&mut results, "family-review-queue --check", || {
            with_args(&["--check"], root, crate::synodal_family_review::run)
        });
        if results[family_gate_index].error.is_some() {
            print_undecided_family_stubs(root);
        }
    } else {
        println!("skipping accent-fit and family-review-queue gates: no intermediate corpus");
    }
    step(&mut results, "marginal-recovery --check", || {
        with_args(&["--check"], root, crate::synodal_marginal_recovery::run)
    });
    step(&mut results, "lexical-union --check", || {
        with_args(&["--check"], root, crate::synodal_lexical_union::run)
    });
    step(&mut results, "archive --check", || {
        with_args(&["--check"], root, crate::synodal_archive::run)
    });

    if !check_only {
        step(&mut results, "cargo fmt --check", || {
            cargo(&["fmt", "--all", "--check"], root)
        });
        step(&mut results, "cargo clippy", || {
            cargo(
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                root,
            )
        });
        step(&mut results, "cargo test", || {
            cargo(
                &["test", "--workspace", "--all-targets", "--all-features"],
                root,
            )
        });
        step(&mut results, "cargo test --doc", || {
            cargo(&["test", "--workspace", "--doc"], root)
        });
    }

    let mut failures = 0usize;
    println!(
        "\nsynodal wave close ({}):",
        if fix {
            "fix"
        } else if check_only {
            "check"
        } else {
            "local"
        }
    );
    for result in &results {
        let verdict = match &result.error {
            None => "ok".to_owned(),
            Some(error) => {
                failures += 1;
                format!("FAIL — {error}")
            }
        };
        println!("  {:<36} {:>7.1}s  {verdict}", result.name, result.seconds);
    }
    if failures == 0 {
        println!("synodal wave close: all {} steps green", results.len());
        Ok(())
    } else {
        Err(format!("synodal wave close: {failures} steps failed").into())
    }
}
