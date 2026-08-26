#![forbid(unsafe_code)]

mod alypy_oracle;
mod morphology_completeness;
mod ocs_lexical_union;
mod ocs_verb_metadata;
mod report_io;
mod rewrite_dictionary;
mod rewrite_pilot;
mod sources;
mod synodal;
mod synodal_accent_fit;
mod synodal_admit_check;
mod synodal_archive;
mod synodal_coverage;
mod synodal_evaluation_queue;
mod synodal_family_review;
mod synodal_gold_oracle;
mod synodal_lexical_review;
mod synodal_lexical_union;
mod synodal_marginal_recovery;
mod synodal_predict;
mod synodal_type_holdout;
mod synodal_wave_close;
mod synodal_waves;

use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell, Gender,
    ImperativeCell, LParticipleCell, NounCell, Number, ParticipleKind, Person,
};
use old_church_slavonic_extractor::extract::{check_registry, refresh, refresh_derived_registry};
use old_church_slavonic_extractor::semantics::{check_dictionary, refresh_dictionary};
use std::env;
use std::error::Error;
use std::fs;
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
        Some("synodal-regenerate") => synodal::regenerate(&workspace_root()?),
        Some("synodal-admit-check") => {
            let write = args.next().as_deref() == Some("--write-baseline");
            synodal_admit_check::admit_check(&workspace_root()?, write)
        }
        Some("synodal-wave-close") => synodal_wave_close::run(&mut args, &workspace_root()?),
        Some("synodal-check") => synodal::check(&workspace_root()?),
        Some("synodal-evaluate") => synodal::evaluate_and_write(&workspace_root()?),
        Some("synodal-guard-witnesses") => synodal::guard_witnesses(&workspace_root()?),
        Some("alypy-paradigm-oracle") => alypy_oracle::run(&mut args, &workspace_root()?),
        Some("synodal-gold-oracle") => synodal_gold_oracle::run(&mut args, &workspace_root()?),
        Some("synodal-sources") => sources::run(&mut args, &workspace_root()?),
        Some("synodal-bootstrap") => synodal::bootstrap(&mut args, &workspace_root()?),
        Some("synodal-fixture-bootstrap") => {
            synodal::fixture_bootstrap(&mut args, &workspace_root()?)
        }
        Some("synodal-coverage") => synodal_coverage::run(&mut args, &workspace_root()?),
        Some("synodal-archive") => synodal_archive::run(&mut args, &workspace_root()?),
        Some("synodal-predict") => synodal_predict::run(&mut args, &workspace_root()?),
        Some("synodal-coverage-floors") => {
            synodal_coverage::check_committed_floors(&workspace_root()?)
        }
        Some("synodal-accent-fit") => synodal_accent_fit::run(&mut args, &workspace_root()?),
        Some("synodal-type-holdout") => synodal_type_holdout::run(&mut args, &workspace_root()?),
        Some("synodal-evaluation-queue") => {
            synodal_evaluation_queue::run(&mut args, &workspace_root()?)
        }
        Some("synodal-family-review-queue") => {
            synodal_family_review::run(&mut args, &workspace_root()?)
        }
        Some("synodal-lexical-review-queue") => {
            synodal_lexical_review::run(&mut args, &workspace_root()?)
        }
        Some("synodal-lexical-union") => synodal_lexical_union::run(&mut args, &workspace_root()?),
        Some("synodal-marginal-recovery") => {
            synodal_marginal_recovery::run(&mut args, &workspace_root()?)
        }
        Some("morphology-completeness") => {
            morphology_completeness::run(&mut args, &workspace_root()?)
        }
        Some("ocs-lexical-union") => ocs_lexical_union::run(&mut args, &workspace_root()?),
        Some("rewrite-pilot-accuracy") => rewrite_pilot::accuracy(&mut args, &workspace_root()?),
        Some("rewrite-emit-residue") => rewrite_pilot::emit_residue(&workspace_root()?),
        Some("rewrite-dictionary") => rewrite_dictionary::emit(&workspace_root()?),
        Some("check-all") => check_all(),
        Some("check-structure") => check_structure(),
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

pub(crate) fn check_all() -> Result<(), Box<dyn Error>> {
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
    check_structure()
}

pub(crate) fn check_structure() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    check_registry(&root)?;
    check_dictionary(&root)?;
    check_runtime_boundaries(&root)?;
    check_attribution(&root)?;
    synodal::check(&root)?;
    morphology_completeness::check_progress_artifacts(&root)?;
    rewrite_pilot::accuracy(&mut std::iter::empty(), &root)?;
    rewrite_dictionary::check(&root)?;
    check_pilot_data_budget(&root)?;
    check_vendored_source_tables(&root)
}

/// The packaged old-church-slavonic-core crate vendors the Polivanova source
/// tables (a published crate cannot include files outside its root); the
/// canonical copies stay in data/ocs and must remain byte-identical.
fn check_vendored_source_tables(root: &Path) -> Result<(), Box<dyn Error>> {
    for name in [
        "polivanova_regular_nouns.tsv",
        "polivanova_regular_verbs.tsv",
    ] {
        let canonical = fs::read(root.join("data/ocs").join(name))?;
        let vendored = fs::read(root.join("crates/old-church-slavonic-core/data").join(name))?;
        if canonical != vendored {
            return Err(format!(
                "vendored {name} diverges from data/ocs; re-copy the canonical file"
            )
            .into());
        }
    }
    Ok(())
}

/// The rewrite plan caps the pilot facade's bundled generated data at 2 MB
/// (docs/REWRITE_PLAN.md, phase 5 gates).
fn check_pilot_data_budget(root: &Path) -> Result<(), Box<dyn Error>> {
    const BUDGET_BYTES: u64 = 2 * 1024 * 1024;
    let generated = root.join("crates/church-slavonic/generated");
    let mut total = 0u64;
    for entry in fs::read_dir(&generated)? {
        total += entry?.metadata()?.len();
    }
    if total > BUDGET_BYTES {
        return Err(format!(
            "pilot facade generated data is {total} bytes, over the {BUDGET_BYTES}-byte budget"
        )
        .into());
    }
    println!("pilot facade generated data: {total} bytes (budget {BUDGET_BYTES})");
    Ok(())
}

fn check_runtime_boundaries(root: &Path) -> Result<(), Box<dyn Error>> {
    for relative in [
        "crates/old-church-slavonic-core/src",
        "crates/church-slavonic-core/src",
        "crates/church-slavonic/src",
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
        "crates/church-slavonic-core/Cargo.toml",
        "crates/church-slavonic/Cargo.toml",
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
    let package = root.join("crates/church-slavonic");
    let attribution = fs::read_to_string(package.join("ATTRIBUTION.md"))?;
    if !attribution.contains("English Wiktionary")
        || !attribution.contains("CC BY-SA 4.0")
        || !attribution.contains("creativecommons.org/licenses/by-sa/4.0/legalcode")
    {
        return Err("published attribution is missing source identity or license".into());
    }
    let manifest = fs::read_to_string(package.join("Cargo.toml"))?;
    if !manifest.contains("CC-BY-SA-4.0") {
        return Err("published manifest omits the bundled data license".into());
    }
    println!("package attribution: current");
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
    eprintln!("  synodal-regenerate");
    eprintln!("  synodal-admit-check [--write-baseline]");
    eprintln!("  synodal-wave-close [--check|--fix]");
    eprintln!("  synodal-check");
    eprintln!("  synodal-evaluate");
    eprintln!("  synodal-guard-witnesses");
    eprintln!("  alypy-paradigm-oracle [--check]");
    eprintln!("  synodal-gold-oracle [--check]");
    eprintln!("  synodal-sources <list|status|fetch|verify|refresh> [OPTIONS]");
    eprintln!(
        "  synodal-bootstrap [--cache PATH] [--offline] [--source ID] [--skip-fetch] [--keep-intermediate]"
    );
    eprintln!("  synodal-fixture-bootstrap");
    eprintln!(
        "  synodal-coverage --offline [--fixture] [--source ID] [--policy POLICY] [--profile PROFILE] [--check] [--require-complete]"
    );
    eprintln!("  synodal-evaluation-queue [--limit N] [--check]");
    eprintln!("  synodal-family-review-queue [--limit N] [--check]");
    eprintln!("  synodal-lexical-review-queue [--limit N] [--check]");
    eprintln!("  synodal-lexical-union [--check]");
    eprintln!("  synodal-marginal-recovery [--check] [--require-source-inputs]");
    eprintln!("  morphology-completeness [--check]");
    eprintln!("  ocs-lexical-union [--check | --kaikki PATH --osd-jsonl PATH]");
    eprintln!("  rewrite-pilot-accuracy");
    eprintln!("  rewrite-emit-residue");
    eprintln!("  rewrite-dictionary");
    eprintln!("  check-all");
    eprintln!("  check-structure");
}
