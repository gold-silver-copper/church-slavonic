//! Scores and applies the exploratory predictive tier (v0.12 phase 3).
//!
//! Two artifacts, both deterministic:
//!
//! - `reports/synodal-prediction-precision.md` — the tier's gate. Every
//!   reviewed verb with principal parts is masked in turn: its generated
//!   surfaces are re-derived by the corpus-free segmentation predictor and the
//!   predicted cell is scored against the engine's own cell. Precision is
//!   reported by confidence bucket, and only buckets at or above the stated
//!   floor may emit review candidates.
//! - `reports/synodal-prediction-candidates.tsv` — ranked admission
//!   candidates for the review queue: strict-uncovered surfaces grouped by
//!   hypothesised stem, kept only when at least two distinct cells of the
//!   same stem occur in the corpus (sibling support), ranked by token mass.
//!
//! Nothing here reaches the strict or productive resolvers, and no sealed
//! floor reads these reports.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use synodal_church_slavonic::{GenerationPolicy, GrammarCell, Inflector, OrthographyProfile};
use synodal_church_slavonic_core::normalize_lookup_accentless;
use synodal_church_slavonic_dictionary::coverage::CoverageReport;
use synodal_church_slavonic_dictionary::prediction::{SEGMENTATION_MODEL, predict};

use crate::report_io::write_if_changed_atomic;

/// A confidence bucket must reach this masked precision (top prediction, in
/// basis points) before its candidates are emitted.
const PRECISION_FLOOR_BP: usize = 6_000;

/// Surfaces below this corpus frequency are not proposed.
const MINIMUM_FREQUENCY: usize = 4;

const PRECISION_PATH: &str = "reports/synodal-prediction-precision.md";
const CANDIDATES_PATH: &str = "reports/synodal-prediction-candidates.tsv";

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args.by_ref() {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-predict argument {value:?}").into()),
        }
    }
    let (precision, passing_buckets) = masked_precision()?;
    let coverage: CoverageReport = serde_json::from_str(&fs::read_to_string(
        root.join("reports/synodal-coverage.json"),
    )?)?;
    let candidates = frontier_candidates(&coverage, &passing_buckets);
    let precision_path = root.join(PRECISION_PATH);
    let candidates_path = root.join(CANDIDATES_PATH);
    if check {
        for (path, contents) in [
            (&precision_path, &precision),
            (&candidates_path, &candidates),
        ] {
            let actual = fs::read_to_string(path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            if actual != *contents {
                return Err(format!(
                    "stale {}; rerun cargo xtask synodal-predict",
                    path.display()
                )
                .into());
            }
        }
        println!("synodal predict: current");
        return Ok(());
    }
    write_if_changed_atomic(&precision_path, &precision)?;
    write_if_changed_atomic(&candidates_path, &candidates)?;
    println!(
        "synodal predict: {} candidate stems written; buckets passing the {}bp floor: {:?}",
        candidates.lines().count().saturating_sub(1),
        PRECISION_FLOOR_BP,
        passing_buckets.iter().collect::<Vec<_>>(),
    );
    Ok(())
}

fn bucket(confidence_bp: u16) -> &'static str {
    match confidence_bp {
        0..=2399 => "0-2399",
        2400..=2999 => "2400-2999",
        3000..=3399 => "3000-3399",
        _ => "3400+",
    }
}

fn cell_system(cell: GrammarCell) -> &'static str {
    match cell {
        GrammarCell::FiniteVerb(inner) => match inner.tense {
            synodal_church_slavonic::FiniteTense::Aorist => "aorist",
            synodal_church_slavonic::FiniteTense::Imperfect => "imperfect",
            _ => "present-future",
        },
        GrammarCell::Imperative(_) => "imperative",
        GrammarCell::LParticiple(_) => "l-participle",
        GrammarCell::Infinitive => "infinitive",
        _ => "other",
    }
}

/// Whether a predicted cell counts as matching the engine's cell. Present and
/// simple-future cells share their surfaces for perfective verbs, so tense is
/// compared only up to that pair.
fn cells_agree(predicted: GrammarCell, actual: GrammarCell) -> bool {
    use synodal_church_slavonic::FiniteTense as T;
    match (predicted, actual) {
        (GrammarCell::FiniteVerb(a), GrammarCell::FiniteVerb(b)) => {
            let tense_ok = a.tense == b.tense
                || (matches!(a.tense, T::Present | T::Future)
                    && matches!(b.tense, T::Present | T::Future));
            tense_ok && a.person == b.person && a.number == b.number
        }
        (GrammarCell::Imperative(a), GrammarCell::Imperative(b)) => a == b,
        (GrammarCell::LParticiple(a), GrammarCell::LParticiple(b)) => a == b,
        (GrammarCell::Infinitive, GrammarCell::Infinitive) => true,
        _ => false,
    }
}

#[derive(Default)]
struct Tally {
    surfaces: usize,
    top_matches: usize,
    any_matches: usize,
}

fn masked_precision() -> Result<(String, BTreeSet<&'static str>), Box<dyn Error>> {
    let inflector = Inflector::builder()
        .generation_policy(GenerationPolicy::Productive)
        .orthography(OrthographyProfile::Expanded)
        .build();
    let mut by_bucket: BTreeMap<&'static str, Tally> = BTreeMap::new();
    let mut by_system: BTreeMap<&'static str, Tally> = BTreeMap::new();
    let mut masked_lexemes = 0usize;
    let cells = verb_cells();
    for lexeme in synodal_church_slavonic::lexemes()? {
        if lexeme.part_of_speech() != synodal_church_slavonic::PartOfSpeech::Verb {
            continue;
        }
        let mut any = false;
        for cell in &cells {
            let Ok(forms) = inflector.form_by_id(lexeme.id(), *cell) else {
                continue;
            };
            any = true;
            for variant in forms.variants() {
                let surface = normalize_lookup_accentless(&variant.expanded);
                let predictions = predict(&surface);
                let Some(top) = predictions.first() else {
                    // An unpredicted generated surface still counts against
                    // the bucketless total via the system slice.
                    by_system.entry(cell_system(*cell)).or_default().surfaces += 1;
                    continue;
                };
                let top_hit = cells_agree(top.cell, *cell);
                let any_hit = predictions
                    .iter()
                    .any(|prediction| cells_agree(prediction.cell, *cell));
                let bucket_tally = by_bucket.entry(bucket(top.confidence_bp)).or_default();
                bucket_tally.surfaces += 1;
                bucket_tally.top_matches += usize::from(top_hit);
                bucket_tally.any_matches += usize::from(any_hit);
                let system_tally = by_system.entry(cell_system(*cell)).or_default();
                system_tally.surfaces += 1;
                system_tally.top_matches += usize::from(top_hit);
                system_tally.any_matches += usize::from(any_hit);
            }
        }
        masked_lexemes += usize::from(any);
    }
    let mut passing = BTreeSet::new();
    let mut output = String::from(
        "# Synodal prediction precision\n\nThe gate for the exploratory segmentation tier \
         (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its \
         generated surfaces are re-derived by the corpus-free predictor and the predicted cell \
         is scored against the engine's own cell (present and simple future count as one pair). \
         Only confidence buckets at or above the floor emit review candidates.\n\n",
    );
    output.push_str(&format!(
        "- Masked verb lexemes: {masked_lexemes}\n- Precision floor: {PRECISION_FLOOR_BP} bp (top prediction)\n- Model: `{SEGMENTATION_MODEL}`\n\n## Precision by confidence bucket\n\n| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |\n|---|---:|---:|---:|---|\n",
    ));
    for (name, tally) in &by_bucket {
        let top_bp = tally.top_matches.saturating_mul(10_000) / tally.surfaces.max(1);
        let any_bp = tally.any_matches.saturating_mul(10_000) / tally.surfaces.max(1);
        let passes = top_bp >= PRECISION_FLOOR_BP;
        if passes {
            passing.insert(*name);
        }
        output.push_str(&format!(
            "| {name} | {} | {top_bp} bp | {any_bp} bp | {} |\n",
            tally.surfaces,
            if passes { "yes" } else { "no" }
        ));
    }
    output.push_str("\n## Precision by system\n\n| System | Surfaces | Top-1 precision | Any-prediction precision |\n|---|---:|---:|---:|\n");
    for (name, tally) in &by_system {
        output.push_str(&format!(
            "| {name} | {} | {} bp | {} bp |\n",
            tally.surfaces,
            tally.top_matches.saturating_mul(10_000) / tally.surfaces.max(1),
            tally.any_matches.saturating_mul(10_000) / tally.surfaces.max(1),
        ));
    }
    Ok((output, passing))
}

fn verb_cells() -> Vec<GrammarCell> {
    use synodal_church_slavonic::FiniteTense as T;
    use synodal_church_slavonic_core::{Gender, Number, Person};
    let mut cells = Vec::new();
    for tense in [T::Present, T::Future, T::Aorist, T::Imperfect] {
        for person in [Person::First, Person::Second, Person::Third] {
            for number in [Number::Singular, Number::Plural] {
                cells.push(GrammarCell::FiniteVerb(
                    synodal_church_slavonic::FiniteVerbCell {
                        tense,
                        person,
                        number,
                    },
                ));
            }
        }
    }
    for number in [Number::Singular, Number::Plural] {
        cells.push(GrammarCell::Imperative(
            synodal_church_slavonic::ImperativeCell {
                person: Person::Second,
                number,
            },
        ));
        for gender in [Gender::Masculine, Gender::Feminine] {
            cells.push(GrammarCell::LParticiple(
                synodal_church_slavonic::LParticipleCell { gender, number },
            ));
        }
    }
    cells.push(GrammarCell::Infinitive);
    cells
}

fn frontier_candidates(coverage: &CoverageReport, passing: &BTreeSet<&'static str>) -> String {
    #[derive(Default)]
    struct Group {
        tokens: usize,
        cells: BTreeSet<String>,
        surfaces: BTreeSet<String>,
        classes: BTreeSet<&'static str>,
        reflexive: bool,
    }
    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for (surface, frequency) in &coverage.top_k_uncovered_frequency_by_surface {
        if *frequency < MINIMUM_FREQUENCY {
            continue;
        }
        for prediction in predict(surface) {
            if !passing.contains(bucket(prediction.confidence_bp)) {
                continue;
            }
            let group = groups.entry(prediction.stem.clone()).or_default();
            group.tokens += *frequency;
            group.cells.insert(prediction.cell.key());
            group.surfaces.insert(surface.clone());
            group.classes.insert(prediction.class);
            group.reflexive |= prediction.reflexive;
        }
    }
    let mut rows: Vec<(String, Group)> = groups
        .into_iter()
        .filter(|(_, group)| group.cells.len() >= 2)
        .collect();
    rows.sort_by(|left, right| {
        right
            .1
            .tokens
            .cmp(&left.1.tokens)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut output =
        String::from("stem\ttokens\tdistinct_cells\treflexive\tclasses\tsurfaces\tmodel\n");
    for (stem, group) in rows.into_iter().take(400) {
        output.push_str(&format!(
            "{stem}\t{}\t{}\t{}\t{}\t{}\t{SEGMENTATION_MODEL}\n",
            group.tokens,
            group.cells.len(),
            group.reflexive,
            group.classes.into_iter().collect::<Vec<_>>().join(","),
            group.surfaces.into_iter().collect::<Vec<_>>().join(" "),
        ));
    }
    output
}
