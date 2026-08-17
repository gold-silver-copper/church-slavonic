use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::report_io::write_if_changed_atomic;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use synodal_church_slavonic::{GenerationPolicy, Inflector, OrthographyProfile};
use synodal_church_slavonic_dictionary::coverage::{
    Analyzer, CheckTextOptions, CoveragePassage, CoverageReport, coverage_with_type_holdout,
};

const DEFAULT_SOURCES: [&str; 2] = [
    "ponomar-elizabeth-bible-2026-08-09",
    "wikisource-church-slavonic-bible-2026-08-09",
];
const LOCKED_PASSAGES: usize = 74_130;
const LOCKED_TOKENS: usize = 1_313_344;
const LOCKED_TYPES: usize = 57_476;
const LOCKED_INTERMEDIATE_SHA256: [(&str, &str); 2] = [
    (
        "ponomar-elizabeth-bible-2026-08-09",
        "ef0323df940c93c9b72a3cbb6f7adfb062ba38ffcdcf401eff5cf369c4869c26",
    ),
    (
        "wikisource-church-slavonic-bible-2026-08-09",
        "913d9781ef511988d8bcc5d19b1b8c63c7582cd5e476f62469eff199e7c2c08f",
    ),
];

#[derive(Clone, Debug, Deserialize)]
struct CandidatePassage {
    source_id: String,
    source_recension: String,
    target_recension: Option<String>,
    work: String,
    edition: String,
    passage: String,
    normalized_spelling: String,
    grammatical_cell: String,
    partition: String,
    parse_status: String,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut intermediate = root.join("data/intermediate/synodal");
    let mut sources = Vec::new();
    let mut policy = GenerationPolicy::Strict;
    let mut profile = OrthographyProfile::SynodalLiturgical;
    let mut maximum_passages = None;
    let mut check = false;
    let mut fixture = false;
    let mut offline = false;
    let mut require_complete = false;
    let mut reseal_floors = false;
    let mut canonical_inputs = true;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                canonical_inputs = false;
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--source" => {
                canonical_inputs = false;
                sources.push(args.next().ok_or("--source needs an ID")?);
            }
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            "--max-passages" => {
                canonical_inputs = false;
                maximum_passages = Some(
                    args.next()
                        .ok_or("--max-passages needs a number")?
                        .parse()?,
                );
            }
            "--check" => check = true,
            "--fixture" => fixture = true,
            "--offline" => offline = true,
            "--require-complete" => require_complete = true,
            "--reseal-floors" => reseal_floors = true,
            value => return Err(format!("unknown synodal-coverage argument {value:?}").into()),
        }
    }
    if !offline {
        return Err("synodal-coverage is intentionally offline; pass --offline".into());
    }
    if sources.is_empty() {
        sources.extend(DEFAULT_SOURCES.map(str::to_owned));
    }
    if require_complete {
        validate_canonical_request(fixture, canonical_inputs, policy, profile, &intermediate)?;
    }
    let passages = if fixture {
        load_fixture(&root.join("data/synodal/coverage_passages.tsv"))?
    } else {
        load_intermediate(&intermediate, &sources, maximum_passages)?
    };
    if passages.is_empty() {
        return Err("no eligible pinned Synodal passages were loaded".into());
    }
    let analyzer = Analyzer::new(
        Inflector::builder()
            .generation_policy(policy)
            .orthography(profile)
            .build(),
    )?;
    // The fixture is a ten-passage smoke corpus with no sealed holdout; the
    // canonical run measures the type-disjoint slice alongside the corpus.
    let held_out_types = if fixture {
        BTreeSet::new()
    } else {
        crate::synodal_type_holdout::load(&root.join(crate::synodal_type_holdout::HOLDOUT_PATH))?
    };
    let report = coverage_with_type_holdout(
        &analyzer,
        &passages,
        CheckTextOptions {
            generation_policy: policy,
            orthography_profile: profile,
        },
        &held_out_types,
    );
    let stem = if fixture {
        "synodal-coverage-fixture"
    } else {
        "synodal-coverage"
    };
    let json_path = root.join(format!("reports/{stem}.json"));
    let markdown_path = root.join(format!("reports/{stem}.md"));
    let queue_path = root.join(format!("reports/{stem}-review-queue.tsv"));
    let frontier_path = root.join(format!("reports/{stem}-frontier.tsv"));
    let json = format!("{}\n", serde_json::to_string_pretty(&report)?);
    let markdown = report.markdown();
    let queue = report.gaps_tsv();
    let frontier = report.uncovered_frontier_tsv();
    // The sealed floors are checked before anything is written, so a wave that
    // regresses a guarded measure cannot leave a report behind claiming it did
    // not. The fixture is a ten-passage smoke corpus and carries no floors.
    if !fixture {
        let floors_path = root.join("data/synodal/coverage_floors.tsv");
        let floors = load_floors(&floors_path)?;
        if reseal_floors {
            let resealed = reseal(&floors, &report);
            write_if_changed_atomic(&floors_path, &resealed)?;
            println!("synodal coverage: resealed {} floors", floors.len());
        } else {
            enforce_floors(&floors_path, &floors, &report)?;
        }
    }
    if check {
        check_contents(&json_path, &json)?;
        check_contents(&markdown_path, &markdown)?;
        check_contents(&queue_path, &queue)?;
        check_contents(&frontier_path, &frontier)?;
    } else {
        write_if_changed_atomic(&json_path, &json)?;
        write_if_changed_atomic(&markdown_path, &markdown)?;
        write_if_changed_atomic(&queue_path, &queue)?;
        write_if_changed_atomic(&frontier_path, &frontier)?;
    }
    if require_complete {
        validate_complete_report(&report)?;
    }
    println!(
        "Synodal coverage: {} passages, {} tokens, {} types, {} top-k, {} unresolved",
        report.passages,
        report.summary.total_tokens,
        report.token_types,
        report.summary.top_k_analyzed,
        report.summary.unresolved,
    );
    Ok(())
}

fn validate_canonical_request(
    fixture: bool,
    canonical_inputs: bool,
    policy: GenerationPolicy,
    profile: OrthographyProfile,
    intermediate: &Path,
) -> Result<(), Box<dyn Error>> {
    if fixture || !canonical_inputs {
        return Err("--require-complete accepts only the canonical full default source set without --fixture, --source, --intermediate, or --max-passages".into());
    }
    if policy != GenerationPolicy::Strict || profile != OrthographyProfile::SynodalLiturgical {
        return Err(
            "--require-complete requires strict policy and synodal-liturgical profile".into(),
        );
    }
    for (source, expected) in LOCKED_INTERMEDIATE_SHA256 {
        let path = intermediate.join(format!("{source}.jsonl"));
        let actual = format!("{:x}", Sha256::digest(fs::read(&path)?));
        if actual != expected {
            return Err(format!(
                "canonical coverage input {} has SHA-256 {actual}, expected {expected}; audit any denominator change before updating the lock",
                path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn validate_complete_report(report: &CoverageReport) -> Result<(), Box<dyn Error>> {
    let mut failures = Vec::new();
    if report.target_recension != "synodal-russian"
        || report.generation_policy != GenerationPolicy::Strict
        || report.orthography_profile != OrthographyProfile::SynodalLiturgical
    {
        failures.push(format!(
            "resolver contract is target={:?}, policy={:?}, profile={:?}; expected synodal-russian/Strict/SynodalLiturgical",
            report.target_recension, report.generation_policy, report.orthography_profile
        ));
    }
    if report.passages != LOCKED_PASSAGES
        || report.summary.total_tokens != LOCKED_TOKENS
        || report.token_types != LOCKED_TYPES
    {
        failures.push(format!(
            "locked denominator is {LOCKED_PASSAGES} passages/{LOCKED_TOKENS} tokens/{LOCKED_TYPES} types, found {}/{}/{}",
            report.passages, report.summary.total_tokens, report.token_types
        ));
    }
    if report.summary.top_k_analyzed != report.summary.total_tokens {
        failures.push(format!(
            "top-k is {}/{}, leaving {} tokens outside strict top-k",
            report.summary.top_k_analyzed,
            report.summary.total_tokens,
            report
                .summary
                .total_tokens
                .saturating_sub(report.summary.top_k_analyzed)
        ));
    }
    if report.summary.unresolved != 0 {
        failures.push(format!(
            "{} tokens remain unresolved",
            report.summary.unresolved
        ));
    }
    let uncovered: usize = report.top_k_uncovered_frequency_by_surface.values().sum();
    if uncovered != 0 {
        failures.push(format!(
            "legacy uncovered-surface accounting still contains {uncovered} tokens"
        ));
    }
    let frontier_tokens: usize = report
        .uncovered_frontier
        .iter()
        .map(|item| item.token_frequency)
        .sum();
    if frontier_tokens != 0 || !report.uncovered_frontier.is_empty() {
        failures.push(format!(
            "complete frontier still contains {} rows and {frontier_tokens} tokens",
            report.uncovered_frontier.len()
        ));
    }
    for (dimension, actual, expected) in [
        ("corpus", report.by_corpus.len(), 2),
        ("source", report.by_source.len(), 2),
        ("partition", report.by_partition.len(), 2),
        ("source/partition", report.by_source_partition.len(), 4),
        ("policy", report.by_policy.len(), 1),
    ] {
        if actual != expected {
            failures.push(format!(
                "{dimension} matrix has {actual} slices, expected {expected}"
            ));
        }
    }
    for (dimension, slices) in [
        ("corpus", &report.by_corpus),
        ("source", &report.by_source),
        ("partition", &report.by_partition),
        ("source/partition", &report.by_source_partition),
        ("policy", &report.by_policy),
    ] {
        let matrix_tokens: usize = slices.values().map(|slice| slice.total_tokens).sum();
        if matrix_tokens != report.summary.total_tokens {
            failures.push(format!(
                "{dimension} matrix accounts for {matrix_tokens} tokens, expected {}",
                report.summary.total_tokens
            ));
        }
        for (name, slice) in slices {
            if slice.top_k_analyzed != slice.total_tokens || slice.unresolved != 0 {
                failures.push(format!(
                    "{dimension} {name:?} is {}/{} top-k with {} unresolved",
                    slice.top_k_analyzed, slice.total_tokens, slice.unresolved
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "canonical strict top-k completion gate failed:\n- {}",
            failures.join("\n- ")
        )
        .into())
    }
}

/// Loads the canonical default source set, as the coverage report does.
pub(crate) fn load_canonical_passages(
    intermediate: &Path,
) -> Result<Vec<CoveragePassage>, Box<dyn Error>> {
    let sources = DEFAULT_SOURCES.map(str::to_owned);
    load_intermediate(intermediate, &sources, None)
}

fn load_intermediate(
    intermediate: &Path,
    sources: &[String],
    maximum_passages: Option<usize>,
) -> Result<Vec<CoveragePassage>, Box<dyn Error>> {
    let mut passages = Vec::new();
    for source in sources {
        let path = intermediate.join(format!("{source}.jsonl"));
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "cannot read coverage source {}: {error}; run synodal-bootstrap first or use --fixture",
                path.display()
            )
        })?;
        for (index, line) in contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row: CandidatePassage = serde_json::from_str(line).map_err(|error| {
                format!(
                    "{}:{}: invalid candidate JSON: {error}",
                    path.display(),
                    index + 1
                )
            })?;
            if row.source_id != *source
                || row.source_recension != "synodal-russian"
                || row.target_recension.as_deref() != Some("synodal-russian")
                || row.parse_status != "parsed"
                || !matches!(row.grammatical_cell.as_str(), "verse" | "verse-or-paratext")
            {
                continue;
            }
            passages.push(CoveragePassage {
                corpus: row.work.clone(),
                source_id: row.source_id,
                work: row.work,
                edition: row.edition,
                passage: row.passage,
                partition: row.partition,
                source_recension: row.source_recension,
                text: row.normalized_spelling,
            });
            if maximum_passages.is_some_and(|maximum| passages.len() >= maximum) {
                return Ok(passages);
            }
        }
    }
    Ok(passages)
}

fn load_fixture(path: &Path) -> Result<Vec<CoveragePassage>, Box<dyn Error>> {
    const HEADER: &str =
        "corpus\tsource_id\twork\tedition\tpassage\tpartition\tsource_recension\ttext";
    let contents = fs::read_to_string(path)?;
    let mut lines = contents.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("{} has an invalid header", path.display()).into());
    }
    lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(index, line)| {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 8 {
                return Err(format!("{}:{}: expected 8 fields", path.display(), index + 2).into());
            }
            Ok(CoveragePassage {
                corpus: fields[0].into(),
                source_id: fields[1].into(),
                work: fields[2].into(),
                edition: fields[3].into(),
                passage: fields[4].into(),
                partition: fields[5].into(),
                source_recension: fields[6].into(),
                text: fields[7].replace("\\n", "\n"),
            })
        })
        .collect()
}

fn parse_policy(value: &str) -> Result<GenerationPolicy, Box<dyn Error>> {
    match value {
        "strict" => Ok(GenerationPolicy::Strict),
        "productive" => Ok(GenerationPolicy::Productive),
        "exploratory" => Ok(GenerationPolicy::Exploratory),
        _ => Err(format!("unknown generation policy {value:?}").into()),
    }
}

fn parse_profile(value: &str) -> Result<OrthographyProfile, Box<dyn Error>> {
    match value {
        "expanded" => Ok(OrthographyProfile::Expanded),
        "expanded-accentless" | "accentless" => Ok(OrthographyProfile::ExpandedAccentless),
        "synodal-liturgical" | "liturgical" | "printed" => {
            Ok(OrthographyProfile::SynodalLiturgical)
        }
        _ => Err(format!("unknown orthography profile {value:?}").into()),
    }
}

fn check_contents(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = fs::read_to_string(path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("stale {}; rerun synodal-coverage", path.display()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synodal_church_slavonic_dictionary::coverage::coverage;

    fn floor(measure: &str, at_least: bool, value: usize) -> Floor {
        Floor {
            measure: measure.to_owned(),
            at_least,
            value,
            sealed_at: "test".into(),
            justification: "test".into(),
        }
    }

    /// Builds a report whose guarded measures are known, by covering one
    /// Cyrillic numeral and leaving everything else empty.
    fn probe_report() -> CoverageReport {
        let analyzer = Analyzer::new(Inflector::default()).expect("analyzer");
        coverage(
            &analyzer,
            &[CoveragePassage {
                corpus: "fixture".into(),
                source_id: "fixture".into(),
                work: "fixture".into(),
                edition: "fixture".into(),
                passage: "1".into(),
                partition: "source".into(),
                source_recension: "synodal-russian".into(),
                text: "҂а҃".into(),
            }],
            CheckTextOptions::default(),
        )
    }

    #[test]
    fn sealed_floors_catch_a_regressed_measure() {
        let report = probe_report();
        let path = Path::new("coverage_floors.tsv");
        let mut floors: Vec<Floor> = guarded_measures(&report)
            .iter()
            .map(|(measure, value)| floor(measure, true, *value))
            .collect();
        enforce_floors(path, &floors, &report).expect("sealed at the achieved values");

        let target = floors
            .iter_mut()
            .find(|floor| floor.measure == "summary:top_k_analyzed")
            .expect("guarded");
        target.value += 1;
        let error = enforce_floors(path, &floors, &report)
            .expect_err("a top-k regression must fail the gate")
            .to_string();
        assert!(error.contains("summary:top_k_analyzed"), "{error}");
        assert!(error.contains("at least"), "{error}");
    }

    #[test]
    fn sealed_floors_catch_a_breached_morphology_free_ceiling() {
        let report = probe_report();
        let mut floors: Vec<Floor> = guarded_measures(&report)
            .iter()
            .map(|(measure, value)| floor(measure, true, *value))
            .collect();
        // The ceiling is the one bound whose direction is reversed: a rise in
        // morphology-free coverage is the cheap-recall failure mode.
        for entry in &mut floors {
            if entry.measure == "integrity:morphology_free_analyzed" {
                entry.at_least = false;
                entry.value = 0;
            }
        }
        let mut breached = report.clone();
        breached.integrity.morphology_free_analyzed = 1;
        let error = enforce_floors(Path::new("coverage_floors.tsv"), &floors, &breached)
            .expect_err("morphology-free growth must fail the gate")
            .to_string();
        assert!(
            error.contains("integrity:morphology_free_analyzed"),
            "{error}"
        );
        assert!(error.contains("at most"), "{error}");
    }

    #[test]
    fn sealed_floors_reject_an_unsealed_or_vanished_measure() {
        let report = probe_report();
        let path = Path::new("coverage_floors.tsv");
        let complete: Vec<Floor> = guarded_measures(&report)
            .iter()
            .map(|(measure, value)| floor(measure, true, *value))
            .collect();

        let missing: Vec<Floor> = complete
            .iter()
            .filter(|entry| entry.measure != "summary:top_1_analyzed")
            .cloned()
            .collect();
        let error = enforce_floors(path, &missing, &report)
            .expect_err("an unsealed guarded measure must fail")
            .to_string();
        assert!(error.contains("has no sealed floor"), "{error}");

        let mut stale = complete;
        stale.push(floor("system:retired-system", true, 0));
        let error = enforce_floors(path, &stale, &report)
            .expect_err("a floor naming a vanished measure must fail")
            .to_string();
        assert!(error.contains("no longer produces it"), "{error}");
    }

    #[test]
    fn resealing_only_ever_tightens_a_bound() {
        let report = probe_report();
        let covered = report.summary.top_k_analyzed;
        let floors = vec![
            floor("summary:top_k_analyzed", true, 0),
            floor("integrity:morphology_free_analyzed", false, 9_999),
        ];
        let resealed = reseal(&floors, &report);
        assert!(
            resealed.contains(&format!("summary:top_k_analyzed\tat-least\t{covered}\t")),
            "an at-least floor rises to the achieved value: {resealed}"
        );
        assert!(
            resealed.contains("integrity:morphology_free_analyzed\tat-most\t0\t"),
            "an at-most ceiling falls to the achieved value: {resealed}"
        );

        // Resealing must never relax a bound the current report cannot meet.
        let strict = vec![floor("summary:top_k_analyzed", true, covered + 5)];
        let kept = reseal(&strict, &report);
        assert!(
            kept.contains(&format!(
                "summary:top_k_analyzed\tat-least\t{}\t",
                covered + 5
            )),
            "a stricter existing floor survives resealing: {kept}"
        );
    }

    #[test]
    fn the_committed_floors_parse_and_cover_every_guarded_measure() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/synodal/coverage_floors.tsv");
        let floors = load_floors(&path).expect("committed floors parse");
        assert!(
            floors.iter().any(
                |entry| entry.measure == "integrity:morphology_free_analyzed" && !entry.at_least
            ),
            "the morphology-free ceiling must be sealed as an upper bound"
        );
        assert!(
            floors
                .iter()
                .all(|entry| !entry.justification.trim().is_empty()),
            "every sealed bound states why it exists"
        );
    }

    #[test]
    fn fixture_parser_preserves_passage_identity() {
        let root =
            std::env::temp_dir().join(format!("synodal-coverage-fixture-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temporary directory");
        let path = root.join("fixture.tsv");
        fs::write(
            &path,
            "corpus\tsource_id\twork\tedition\tpassage\tpartition\tsource_recension\ttext\ncorpus\tsource\twork\tedition\tpassage\tevaluation\tsynodal-russian\tє҆́смь\n",
        )
        .expect("fixture");
        let passages = load_fixture(&path).expect("load");
        assert_eq!(passages[0].corpus, "corpus");
        assert_eq!(passages[0].passage, "passage");
        assert_eq!(passages[0].text, "є҆́смь");
        fs::remove_file(path).expect("remove fixture");
        fs::remove_dir(root).expect("remove temporary directory");
    }

    #[test]
    fn committed_passage_fixture_hash_is_stable() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/synodal/coverage_passages.tsv");
        let digest = format!("{:x}", Sha256::digest(fs::read(path).expect("fixture")));
        assert_eq!(
            digest,
            "86c85fa10a8b1b954a72754fa41aa16dd379fc6ae0e41bab87432f93612d5d1f"
        );
    }

    #[test]
    fn completion_gate_rejects_noncanonical_denominator_and_uncovered_tokens() {
        let analyzer = Analyzer::new(Inflector::default()).expect("analyzer");
        let mut report = coverage(
            &analyzer,
            &[CoveragePassage {
                corpus: "fixture".into(),
                source_id: "fixture".into(),
                work: "fixture".into(),
                edition: "fixture".into(),
                passage: "1".into(),
                partition: "source".into(),
                source_recension: "synodal-russian".into(),
                text: "҂а҃".into(),
            }],
            CheckTextOptions::default(),
        );
        let error = validate_complete_report(&report)
            .expect_err("fixture denominator must not satisfy canonical completion")
            .to_string();
        assert!(error.contains("locked denominator"));
        report
            .top_k_uncovered_frequency_by_surface
            .insert("gap".into(), 2);
        let error = validate_complete_report(&report)
            .expect_err("nonempty uncovered map must fail")
            .to_string();
        assert!(error.contains("contains 2 tokens"));
    }

    #[test]
    fn completion_gate_accepts_only_complete_partition_accounting() {
        use synodal_church_slavonic_dictionary::coverage::CoverageSlice;

        fn complete_slice(total_tokens: usize) -> CoverageSlice {
            CoverageSlice {
                total_tokens,
                top_1_analyzed: total_tokens,
                top_k_analyzed: total_tokens,
                ambiguous: 0,
                unresolved: 0,
            }
        }

        let analyzer = Analyzer::new(Inflector::default()).expect("analyzer");
        let mut report = coverage(
            &analyzer,
            &[CoveragePassage {
                corpus: "fixture".into(),
                source_id: "fixture".into(),
                work: "fixture".into(),
                edition: "fixture".into(),
                passage: "1".into(),
                partition: "source".into(),
                source_recension: "synodal-russian".into(),
                text: "҂а҃".into(),
            }],
            CheckTextOptions::default(),
        );
        report.passages = LOCKED_PASSAGES;
        report.token_types = LOCKED_TYPES;
        report.orthography_profile = OrthographyProfile::SynodalLiturgical;
        report.summary = complete_slice(LOCKED_TOKENS);
        report.by_corpus = [
            ("corpus-a".into(), complete_slice(LOCKED_TOKENS / 2)),
            ("corpus-b".into(), complete_slice(LOCKED_TOKENS / 2)),
        ]
        .into_iter()
        .collect();
        report.by_source = [
            ("source-a".into(), complete_slice(LOCKED_TOKENS / 2)),
            ("source-b".into(), complete_slice(LOCKED_TOKENS / 2)),
        ]
        .into_iter()
        .collect();
        report.by_partition = [
            ("evaluation".into(), complete_slice(LOCKED_TOKENS / 2)),
            ("source".into(), complete_slice(LOCKED_TOKENS / 2)),
        ]
        .into_iter()
        .collect();
        report.by_source_partition = [
            (
                "source-a:evaluation".into(),
                complete_slice(LOCKED_TOKENS / 4),
            ),
            ("source-a:source".into(), complete_slice(LOCKED_TOKENS / 4)),
            (
                "source-b:evaluation".into(),
                complete_slice(LOCKED_TOKENS / 4),
            ),
            ("source-b:source".into(), complete_slice(LOCKED_TOKENS / 4)),
        ]
        .into_iter()
        .collect();
        report.by_policy = [("strict".into(), complete_slice(LOCKED_TOKENS))]
            .into_iter()
            .collect();

        validate_complete_report(&report).expect("complete canonical accounting");
        report
            .by_source_partition
            .get_mut("source-a:evaluation")
            .expect("slice")
            .top_k_analyzed -= 1;
        let error = validate_complete_report(&report)
            .expect_err("one incomplete partition must fail")
            .to_string();
        assert!(error.contains("source/partition \"source-a:evaluation\""));
    }
}

/// One sealed bound on a coverage measure.
///
/// The file is a reviewed contract, not a generated artifact: weakening a bound
/// means editing the row and stating why, which surfaces in review as a diff.
#[derive(Clone, Debug)]
struct Floor {
    measure: String,
    at_least: bool,
    value: usize,
    sealed_at: String,
    justification: String,
}

const FLOOR_HEADER: &str = "measure\tdirection\tvalue\tsealed_at\tjustification";

/// Enforces the sealed floors against the *committed* coverage report.
///
/// The canonical coverage run needs `data/intermediate/`, which is gitignored,
/// so CI's fresh checkout cannot execute it and the floors would otherwise be
/// enforced on no automatic path at all. This reads the committed report
/// instead, which the full-bootstrap workflow keeps current, so a wave that
/// regresses a guarded measure fails on every pull request rather than only on
/// a manual dispatch.
pub(crate) fn check_committed_floors(root: &Path) -> Result<(), Box<dyn Error>> {
    let report_path = root.join("reports/synodal-coverage.json");
    let report: CoverageReport = serde_json::from_str(&fs::read_to_string(&report_path)?)?;
    let floors_path = root.join("data/synodal/coverage_floors.tsv");
    let floors = load_floors(&floors_path)?;
    enforce_floors(&floors_path, &floors, &report)?;
    println!(
        "synodal coverage floors: {} sealed bounds hold",
        floors.len()
    );
    Ok(())
}

fn load_floors(path: &Path) -> Result<Vec<Floor>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = contents.lines();
    if lines.next() != Some(FLOOR_HEADER) {
        return Err(format!("invalid header in {}", path.display()).into());
    }
    let mut floors = Vec::new();
    let mut seen = BTreeSet::new();
    for (offset, line) in lines.filter(|line| !line.is_empty()).enumerate() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [measure, direction, value, sealed_at, justification] = columns.as_slice() else {
            return Err(format!("{}:{} needs five columns", path.display(), offset + 2).into());
        };
        let at_least = match *direction {
            "at-least" => true,
            "at-most" => false,
            other => {
                return Err(format!(
                    "{}:{} has unknown direction {other:?}",
                    path.display(),
                    offset + 2
                )
                .into());
            }
        };
        if justification.trim().is_empty() || sealed_at.trim().is_empty() {
            return Err(format!(
                "{}:{} must state when it was sealed and why",
                path.display(),
                offset + 2
            )
            .into());
        }
        if !seen.insert((*measure).to_owned()) {
            return Err(format!("{} repeats {measure:?}", path.display()).into());
        }
        floors.push(Floor {
            measure: (*measure).to_owned(),
            at_least,
            value: value.parse()?,
            sealed_at: (*sealed_at).to_owned(),
            justification: (*justification).to_owned(),
        });
    }
    Ok(floors)
}

/// The guarded measures of one report, keyed exactly as the floors file names
/// them.
fn guarded_measures(report: &CoverageReport) -> BTreeMap<String, usize> {
    let mut measures = BTreeMap::new();
    measures.insert(
        "summary:top_k_analyzed".to_owned(),
        report.summary.top_k_analyzed,
    );
    measures.insert(
        "summary:top_1_analyzed".to_owned(),
        report.summary.top_1_analyzed,
    );
    measures.insert(
        "integrity:lemma_unique_analyzed".to_owned(),
        report.integrity.lemma_unique_analyzed,
    );
    measures.insert(
        "integrity:morphologically_typed_analyzed".to_owned(),
        report.integrity.morphologically_typed_analyzed,
    );
    measures.insert(
        "integrity:morphology_free_analyzed".to_owned(),
        report.integrity.morphology_free_analyzed,
    );
    // Sealed after review: a duplicated identity raises this while making the
    // analyzer worse, and the lemma-unique floor only catches one whose
    // collisions happen to exceed its gains.
    measures.insert(
        "integrity:cross_lexeme_ambiguous".to_owned(),
        report.integrity.cross_lexeme_ambiguous,
    );
    for (system, slice) in &report.by_morphological_system {
        measures.insert(format!("system:{system}"), slice.top_k_analyzed);
    }
    // The type-disjoint holdout is where generalisation is actually visible.
    // Coverage there must not fall, coverage that arrives by memorising the
    // held-out type itself must not rise, and coverage that arrives by rule
    // must not fall. Together these force new work onto the generalising side.
    let status = |label: &str| {
        report
            .held_out_type_status
            .get(label)
            .copied()
            .unwrap_or_default()
    };
    measures.insert(
        "holdout:top_k_analyzed".to_owned(),
        report.held_out_type_coverage.top_k_analyzed,
    );
    measures.insert(
        "holdout:memorised_analyzed".to_owned(),
        status("exact-synodal-attestation"),
    );
    measures.insert(
        "holdout:generalised_analyzed".to_owned(),
        status("synodal-normative-table")
            + status("synodal-productive-rule")
            + status("synodal-irregular-override"),
    );
    measures
}

/// Fails when any sealed bound is violated, when a guarded measure has no
/// floor, or when a floor names a measure the report no longer produces.
///
/// An unsealed measure is an error rather than a warning: a newly appearing
/// morphological system has to be admitted deliberately, and a disappearing one
/// must not slip past because its floor stopped being evaluated.
fn enforce_floors(
    path: &Path,
    floors: &[Floor],
    report: &CoverageReport,
) -> Result<(), Box<dyn Error>> {
    let measures = guarded_measures(report);
    let sealed: BTreeSet<&str> = floors.iter().map(|floor| floor.measure.as_str()).collect();
    let mut failures = Vec::new();
    for measure in measures.keys() {
        if !sealed.contains(measure.as_str()) {
            failures.push(format!(
                "  {measure} has no sealed floor; add a reviewed row to {}",
                path.display()
            ));
        }
    }
    for floor in floors {
        let Some(actual) = measures.get(&floor.measure).copied() else {
            failures.push(format!(
                "  {} is sealed but the report no longer produces it",
                floor.measure
            ));
            continue;
        };
        let ok = if floor.at_least {
            actual >= floor.value
        } else {
            actual <= floor.value
        };
        if !ok {
            let direction = if floor.at_least {
                "at least"
            } else {
                "at most"
            };
            failures.push(format!(
                "  {} is {actual} but was sealed {direction} {} at {}: {}",
                floor.measure, floor.value, floor.sealed_at, floor.justification
            ));
        }
    }
    if failures.is_empty() {
        return Ok(());
    }
    Err(format!(
        "sealed coverage floors violated:\n{}\n\nRaise coverage, or edit {} with a stated justification if the change is intended.",
        failures.join("\n"),
        path.display()
    )
    .into())
}

/// Ratchets every bound toward the current report, never away from it.
///
/// Resealing can only tighten: an `at-least` floor rises to the achieved value
/// and an `at-most` ceiling falls to it. Weakening a bound stays a hand edit so
/// that it is reviewed rather than absorbed by a tool run.
fn reseal(floors: &[Floor], report: &CoverageReport) -> String {
    let measures = guarded_measures(report);
    let mut output = String::from(FLOOR_HEADER);
    output.push('\n');
    for floor in floors {
        let value = measures
            .get(&floor.measure)
            .copied()
            .map_or(floor.value, |actual| {
                if floor.at_least {
                    actual.max(floor.value)
                } else {
                    actual.min(floor.value)
                }
            });
        output.push_str(&format!(
            "{}\t{}\t{value}\t{}\t{}\n",
            floor.measure,
            if floor.at_least {
                "at-least"
            } else {
                "at-most"
            },
            floor.sealed_at,
            floor.justification,
        ));
    }
    output
}
