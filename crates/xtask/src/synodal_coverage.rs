use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::report_io::write_if_changed_atomic;
use serde::Deserialize;
use synodal_church_slavonic::{GenerationPolicy, Inflector, OrthographyProfile};
use synodal_church_slavonic_dictionary::coverage::{
    Analyzer, CheckTextOptions, CoveragePassage, coverage,
};

const DEFAULT_SOURCES: [&str; 2] = [
    "ponomar-elizabeth-bible-2026-08-09",
    "wikisource-church-slavonic-bible-2026-08-09",
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
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--source" => sources.push(args.next().ok_or("--source needs an ID")?),
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            "--max-passages" => {
                maximum_passages = Some(
                    args.next()
                        .ok_or("--max-passages needs a number")?
                        .parse()?,
                );
            }
            "--check" => check = true,
            "--fixture" => fixture = true,
            "--offline" => offline = true,
            value => return Err(format!("unknown synodal-coverage argument {value:?}").into()),
        }
    }
    if !offline {
        return Err("synodal-coverage is intentionally offline; pass --offline".into());
    }
    if sources.is_empty() {
        sources.extend(DEFAULT_SOURCES.map(str::to_owned));
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
    let report = coverage(
        &analyzer,
        &passages,
        CheckTextOptions {
            generation_policy: policy,
            orthography_profile: profile,
        },
    );
    let stem = if fixture {
        "synodal-coverage-fixture"
    } else {
        "synodal-coverage"
    };
    let json_path = root.join(format!("reports/{stem}.json"));
    let markdown_path = root.join(format!("reports/{stem}.md"));
    let queue_path = root.join(format!("reports/{stem}-review-queue.tsv"));
    let json = format!("{}\n", serde_json::to_string_pretty(&report)?);
    let markdown = report.markdown();
    let queue = report.gaps_tsv();
    if check {
        check_contents(&json_path, &json)?;
        check_contents(&markdown_path, &markdown)?;
        check_contents(&queue_path, &queue)?;
    } else {
        write_if_changed_atomic(&json_path, &json)?;
        write_if_changed_atomic(&markdown_path, &markdown)?;
        write_if_changed_atomic(&queue_path, &queue)?;
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
    use sha2::{Digest, Sha256};

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
}
