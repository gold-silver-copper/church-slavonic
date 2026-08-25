//! The per-wave generalisation ledger.
//!
//! Corpus-wide top-k answers "how much of the locked corpus has an analysis";
//! it cannot tell a better engine from a longer lookup table. The ledger
//! records, for every sealed wave, the held-out measures that can — tokens of
//! never-seen types reached by rule versus by a row naming the type — next to
//! the corpus figure and the lexicon size that produced them, so a wave that
//! raises corpus coverage while `holdout_generalised` stays flat is visible as
//! memorising rather than assumed to be progress.
//!
//! The ledger is append-only and ratcheting: a new row may not lower
//! `holdout_generalised` or raise `holdout_memorised`, and `--check` verifies
//! that the last row still describes the live report.

use std::{collections::BTreeSet, error::Error, fs, path::Path};

use synodal_church_slavonic::PartOfSpeech;
use synodal_church_slavonic_dictionary::coverage::CoverageReport;

use crate::report_io::write_if_changed_atomic;

pub(crate) const LEDGER_PATH: &str = "reports/synodal-waves.tsv";

const HEADER: &str = "wave\tholdout_generalised\tholdout_memorised\tholdout_top_k\ttop_k_analyzed\tmorphology_free_analyzed\tlexemes\tverb_lexemes\tprincipal_parts\tproductive_evaluation_rows\tnote";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WaveMeasures {
    pub holdout_generalised: usize,
    pub holdout_memorised: usize,
    pub holdout_top_k: usize,
    pub top_k_analyzed: usize,
    pub morphology_free_analyzed: usize,
    pub lexemes: usize,
    pub verb_lexemes: usize,
    pub principal_parts: usize,
    pub productive_evaluation_rows: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WaveRow {
    wave: String,
    measures: WaveMeasures,
    note: String,
}

/// The measures of one wave: the report's held-out and corpus figures plus the
/// size of the reviewed lexicon that produced them.
pub(crate) fn measure(
    root: &Path,
    report: &CoverageReport,
) -> Result<WaveMeasures, Box<dyn Error>> {
    let lexemes = synodal_church_slavonic::lexemes()?;
    let verb_lexemes = lexemes
        .iter()
        .filter(|lexeme| lexeme.part_of_speech() == PartOfSpeech::Verb)
        .count();
    Ok(WaveMeasures {
        holdout_generalised: report.held_out_generalised(),
        holdout_memorised: report.held_out_memorised(),
        holdout_top_k: report.held_out_type_coverage.top_k_analyzed,
        top_k_analyzed: report.summary.top_k_analyzed,
        morphology_free_analyzed: report.integrity.morphology_free_analyzed,
        lexemes: lexemes.len(),
        verb_lexemes,
        principal_parts: data_rows(&root.join("data/synodal/principal_parts.tsv"), None)?,
        productive_evaluation_rows: data_rows(
            &root.join("data/synodal/evaluation.tsv"),
            Some(("policy", "productive")),
        )?,
    })
}

/// Counts data rows of a TSV, optionally only those whose named column equals
/// a value.
fn data_rows(path: &Path, filter: Option<(&str, &str)>) -> Result<usize, Box<dyn Error>> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = contents.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let column = match filter {
        Some((name, _)) => Some(
            header
                .iter()
                .position(|field| *field == name)
                .ok_or_else(|| format!("{} has no {name:?} column", path.display()))?,
        ),
        None => None,
    };
    Ok(lines
        .filter(|line| !line.is_empty())
        .filter(|line| match (column, filter) {
            (Some(index), Some((_, value))) => line.split('\t').nth(index) == Some(value),
            _ => true,
        })
        .count())
}

/// The label and measures of the most recently sealed wave, for delta
/// projections to compare against. `None` when the ledger has no rows.
pub(crate) struct LastWave {
    pub label: String,
    pub holdout_generalised: usize,
    pub holdout_memorised: usize,
    pub holdout_top_k: usize,
    pub top_k_analyzed: usize,
}

pub(crate) fn last_sealed_row(root: &Path) -> Result<Option<LastWave>, Box<dyn Error>> {
    let path = root.join(LEDGER_PATH);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(load(&path)?.last().map(|row| LastWave {
        label: row.wave.clone(),
        holdout_generalised: row.measures.holdout_generalised,
        holdout_memorised: row.measures.holdout_memorised,
        holdout_top_k: row.measures.holdout_top_k,
        top_k_analyzed: row.measures.top_k_analyzed,
    }))
}

fn load(path: &Path) -> Result<Vec<WaveRow>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = contents.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("invalid header in {}", path.display()).into());
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (offset, line) in lines.filter(|line| !line.is_empty()).enumerate() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [
            wave,
            generalised,
            memorised,
            holdout_top_k,
            top_k,
            free,
            lexemes,
            verbs,
            parts,
            productive,
            note,
        ] = columns.as_slice()
        else {
            return Err(format!("{}:{} needs eleven columns", path.display(), offset + 2).into());
        };
        if !seen.insert((*wave).to_owned()) {
            return Err(format!("{} repeats wave {wave:?}", path.display()).into());
        }
        rows.push(WaveRow {
            wave: (*wave).to_owned(),
            measures: WaveMeasures {
                holdout_generalised: generalised.parse()?,
                holdout_memorised: memorised.parse()?,
                holdout_top_k: holdout_top_k.parse()?,
                top_k_analyzed: top_k.parse()?,
                morphology_free_analyzed: free.parse()?,
                lexemes: lexemes.parse()?,
                verb_lexemes: verbs.parse()?,
                principal_parts: parts.parse()?,
                productive_evaluation_rows: productive.parse()?,
            },
            note: (*note).to_owned(),
        });
    }
    if rows.is_empty() {
        return Err(format!("{} has no sealed wave", path.display()).into());
    }
    Ok(rows)
}

fn render(rows: &[WaveRow]) -> String {
    let mut output = String::from(HEADER);
    output.push('\n');
    for row in rows {
        let m = &row.measures;
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.wave,
            m.holdout_generalised,
            m.holdout_memorised,
            m.holdout_top_k,
            m.top_k_analyzed,
            m.morphology_free_analyzed,
            m.lexemes,
            m.verb_lexemes,
            m.principal_parts,
            m.productive_evaluation_rows,
            row.note,
        ));
    }
    output
}

/// Every row must ratchet: generalisation may not fall and memorisation may
/// not rise between consecutive sealed waves.
fn check_ratchet(path: &Path, rows: &[WaveRow]) -> Result<(), Box<dyn Error>> {
    for pair in rows.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        if after.measures.holdout_generalised < before.measures.holdout_generalised {
            return Err(format!(
                "{}: wave {:?} lowers holdout_generalised from {} to {}",
                path.display(),
                after.wave,
                before.measures.holdout_generalised,
                after.measures.holdout_generalised
            )
            .into());
        }
        if after.measures.holdout_memorised > before.measures.holdout_memorised {
            return Err(format!(
                "{}: wave {:?} raises holdout_memorised from {} to {}",
                path.display(),
                after.wave,
                before.measures.holdout_memorised,
                after.measures.holdout_memorised
            )
            .into());
        }
    }
    Ok(())
}

/// Fails unless the last sealed wave still describes the given report and the
/// current lexicon.
pub(crate) fn check(root: &Path, report: &CoverageReport) -> Result<(), Box<dyn Error>> {
    let path = root.join(LEDGER_PATH);
    let rows = load(&path)?;
    check_ratchet(&path, &rows)?;
    let live = measure(root, report)?;
    let last = rows.last().ok_or("empty ledger")?;
    if last.measures != live {
        return Err(format!(
            "{} is stale: last sealed wave {:?} recorded {:?} but the live report and lexicon give {:?}; seal a new wave with `cargo xtask synodal-coverage --offline --seal-wave <label>`",
            path.display(),
            last.wave,
            last.measures,
            live
        )
        .into());
    }
    Ok(())
}

/// Appends one wave. Refuses a repeated label, a row identical to the last
/// one, and any row that would break the ratchet.
pub(crate) fn seal(
    root: &Path,
    report: &CoverageReport,
    wave: &str,
    note: &str,
) -> Result<(), Box<dyn Error>> {
    if wave.is_empty() || wave.contains('\t') || note.contains('\t') {
        return Err("wave label and note must be non-empty and tab-free".into());
    }
    let path = root.join(LEDGER_PATH);
    let mut rows = if path.is_file() {
        load(&path)?
    } else {
        Vec::new()
    };
    if rows.iter().any(|row| row.wave == wave) {
        return Err(format!("{} already seals wave {wave:?}", path.display()).into());
    }
    let measures = measure(root, report)?;
    if rows.last().is_some_and(|last| last.measures == measures) {
        return Err(format!(
            "wave {wave:?} would be identical to the last sealed wave; nothing to seal"
        )
        .into());
    }
    rows.push(WaveRow {
        wave: wave.to_owned(),
        measures,
        note: note.to_owned(),
    });
    check_ratchet(&path, &rows)?;
    write_if_changed_atomic(&path, &render(&rows))?;
    let last = rows.last().ok_or("empty ledger")?;
    println!(
        "synodal wave {wave:?} sealed: holdout generalised {}, memorised {}, top-k {}",
        last.measures.holdout_generalised,
        last.measures.holdout_memorised,
        last.measures.top_k_analyzed
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(wave: &str, generalised: usize, memorised: usize) -> WaveRow {
        WaveRow {
            wave: wave.into(),
            measures: WaveMeasures {
                holdout_generalised: generalised,
                holdout_memorised: memorised,
                holdout_top_k: 0,
                top_k_analyzed: 0,
                morphology_free_analyzed: 0,
                lexemes: 0,
                verb_lexemes: 0,
                principal_parts: 0,
                productive_evaluation_rows: 0,
            },
            note: String::new(),
        }
    }

    #[test]
    fn ratchet_rejects_falling_generalisation_and_rising_memorisation() {
        let path = Path::new("ledger.tsv");
        assert!(check_ratchet(path, &[row("a", 10, 5), row("b", 10, 5), row("c", 12, 4)]).is_ok());
        assert!(check_ratchet(path, &[row("a", 10, 5), row("b", 9, 5)]).is_err());
        assert!(check_ratchet(path, &[row("a", 10, 5), row("b", 11, 6)]).is_err());
    }

    #[test]
    fn render_round_trips_through_load() -> Result<(), Box<dyn Error>> {
        let rows = vec![row("a", 1, 2), row("b", 3, 1)];
        let directory = std::env::temp_dir().join(format!("synodal-waves-{}", std::process::id()));
        fs::create_dir_all(&directory)?;
        let path = directory.join("ledger.tsv");
        fs::write(&path, render(&rows))?;
        assert_eq!(load(&path)?, rows);
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
