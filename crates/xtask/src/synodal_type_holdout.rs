//! Seals a type-disjoint evaluation holdout for the locked Synodal corpus.
//!
//! The corpus partition split is passage-disjoint. Most frontier surfaces occur
//! on both sides of it, so an exact row sourced from a `source` passage closes
//! its own held-out twin, and only a fraction of a percent of the corpus can
//! ever test whether the engine generalises rather than memorises.
//!
//! Holding out normalized *types* fixes that. The selector is a content hash of
//! the type itself, so membership cannot be tuned to avoid hard types, dodge a
//! weak area, or protect a percentage: change the corpus and the same rule
//! reproduces the same set, and there is no free parameter to move.
//!
//! This command only seals the set. Measurement happens in the coverage report,
//! which reports held-out coverage together with the resolver statuses that
//! produced it, so memorised coverage (an exact row citing the held-out type)
//! stays distinguishable from generalised coverage (a productive rule or
//! normative table reaching it).

use std::{
    collections::BTreeMap,
    error::Error,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use synodal_church_slavonic::{GenerationPolicy, Inflector, OrthographyProfile};
use synodal_church_slavonic_dictionary::coverage::{
    Analyzer, CheckTextOptions, CoveragePassage, check_text,
};

use crate::report_io::write_if_changed_atomic;

pub(crate) const HOLDOUT_PATH: &str = "data/synodal/held_out_types.tsv";

const HEADER: &str = "normalized_type\tcorpus_frequency\tselector";

/// One type in every `SAMPLE_DIVISOR` is held out. Five percent is large enough
/// that the measured slice is not dominated by sampling noise and small enough
/// that the holdout does not distort the primary corpus figures.
const SAMPLE_DIVISOR: u64 = 20;

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut intermediate = root.join("data/intermediate/synodal");
    let mut check = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--check" => check = true,
            value => {
                return Err(format!("unknown synodal-type-holdout argument {value:?}").into());
            }
        }
    }

    let passages = crate::synodal_coverage::load_canonical_passages(&intermediate)?;
    let frequencies = type_frequencies(&passages)?;
    let selected = select(&frequencies);
    let rendered = render(&selected);
    let path = root.join(HOLDOUT_PATH);
    if check {
        let actual = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if actual != rendered {
            return Err(format!("stale {}; rerun synodal-type-holdout", path.display()).into());
        }
        println!("synodal type holdout: current");
        return Ok(());
    }
    write_if_changed_atomic(&path, &rendered)?;
    let tokens: usize = selected.values().sum();
    println!(
        "synodal type holdout: {} of {} types held out ({tokens} tokens)",
        selected.len(),
        frequencies.len()
    );
    Ok(())
}

/// Counts every normalized type in the corpus through the same tokenizer the
/// coverage report uses, so the sealed keys match the report's keys exactly.
fn type_frequencies(
    passages: &[CoveragePassage],
) -> Result<BTreeMap<String, usize>, Box<dyn Error>> {
    let analyzer = Analyzer::new(
        Inflector::builder()
            .generation_policy(GenerationPolicy::Strict)
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build(),
    )?;
    let options = CheckTextOptions {
        generation_policy: GenerationPolicy::Strict,
        orthography_profile: OrthographyProfile::SynodalLiturgical,
    };
    let mut frequencies: BTreeMap<String, usize> = BTreeMap::new();
    for passage in passages {
        let report = check_text(&analyzer, &passage.text, options.clone());
        for analysis in report.tokens {
            *frequencies.entry(analysis.token.normalized).or_default() += 1;
        }
    }
    Ok(frequencies)
}

/// The content-derived selector. Deterministic, corpus-independent, and free of
/// any tunable parameter that could be used to shape the sample.
fn selector(normalized: &str) -> u64 {
    let digest = Sha256::digest(normalized.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes)
}

fn is_held_out(normalized: &str) -> bool {
    selector(normalized) % SAMPLE_DIVISOR == 0
}

fn select(frequencies: &BTreeMap<String, usize>) -> BTreeMap<String, usize> {
    frequencies
        .iter()
        .filter(|(normalized, _)| is_held_out(normalized))
        .map(|(normalized, frequency)| (normalized.clone(), *frequency))
        .collect()
}

fn render(selected: &BTreeMap<String, usize>) -> String {
    let mut output = String::from(HEADER);
    output.push('\n');
    for (normalized, frequency) in selected {
        output.push_str(&format!(
            "{normalized}\t{frequency}\tsha256-mod-{SAMPLE_DIVISOR}\n"
        ));
    }
    output
}

/// Reads the sealed holdout for the coverage run.
pub(crate) fn load(path: &Path) -> Result<std::collections::BTreeSet<String>, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = contents.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("invalid header in {}", path.display()).into());
    }
    let mut held_out = std::collections::BTreeSet::new();
    for (offset, line) in lines.filter(|line| !line.is_empty()).enumerate() {
        let Some(normalized) = line.split('\t').next() else {
            return Err(format!("{}:{} is empty", path.display(), offset + 2).into());
        };
        // A sealed row that the selector would not choose means the file was
        // hand-edited to shape the sample, which is exactly what the content
        // hash exists to prevent.
        if !is_held_out(normalized) {
            return Err(format!(
                "{}:{} holds out {normalized:?}, which the content selector does not choose",
                path.display(),
                offset + 2
            )
            .into());
        }
        held_out.insert(normalized.to_owned());
    }
    Ok(held_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_selector_is_content_derived_and_stable() {
        let first = selector("бра́тїѧ");
        assert_eq!(
            first,
            selector("бра́тїѧ"),
            "the same type always hashes alike"
        );
        assert_ne!(first, selector("бра́тїю"), "distinct types hash apart");
    }

    #[test]
    fn selection_samples_close_to_the_intended_share() {
        // A biased selector would defeat the purpose, so assert the realised
        // share rather than trusting the construction.
        let corpus: BTreeMap<String, usize> = (0..20_000)
            .map(|index| (format!("тест{index}"), 1))
            .collect();
        let held_out = select(&corpus).len();
        let expected = corpus.len() / SAMPLE_DIVISOR as usize;
        let tolerance = expected / 5;
        assert!(
            held_out.abs_diff(expected) < tolerance,
            "held out {held_out} of {}, expected about {expected}",
            corpus.len()
        );
    }

    #[test]
    fn a_hand_added_row_is_rejected() {
        let directory =
            std::env::temp_dir().join(format!("synodal-holdout-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("held_out_types.tsv");
        let smuggled = (0..10_000)
            .map(|index| format!("тест{index}"))
            .find(|candidate| !is_held_out(candidate))
            .expect("an unselected type");
        std::fs::write(
            &path,
            format!("{HEADER}\n{smuggled}\t1\tsha256-mod-{SAMPLE_DIVISOR}\n"),
        )
        .expect("write");
        let error = load(&path)
            .expect_err("a type the selector does not choose must be rejected")
            .to_string();
        assert!(error.contains("content selector"), "{error}");
        std::fs::remove_file(&path).expect("remove");
        std::fs::remove_dir(&directory).expect("remove directory");
    }
}
