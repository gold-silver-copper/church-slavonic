//! `cargo xtask import <source> --pos <pos> [--write]`: a pinned source's
//! entries become lexicon lines. The importer fits each entry to a class
//! and a stress paradigm, keeps as `overrides`/`variants` only what they
//! do not produce, and writes suspects to `quarantine.tsv` with a reason.
//! Without `--write` it prints the report and the diff summary only.

pub mod crosscheck;
pub mod fit;
pub mod ocs;
pub mod polyakov;

use church_slavonic::cell::Pos;
use church_slavonic::grammar::Recension;
use church_slavonic::lexicon::{self, Lexeme};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

/// A quarantined entry: the source's lemma with the reason it stays out.
#[derive(Debug, Clone)]
pub struct Quarantined {
    pub recension: Recension,
    pub pos: Pos,
    pub lemma: String,
    pub source: String,
    pub reason: &'static str,
    pub detail: String,
}

/// What an import produced.
#[derive(Default)]
pub struct Outcome {
    pub lexemes: Vec<Lexeme>,
    pub quarantine: Vec<Quarantined>,
    /// Counters by name, printed in the report.
    pub counts: BTreeMap<&'static str, u64>,
    /// Override cells by name, for the class-table review.
    pub override_cells: BTreeMap<String, u64>,
    /// Stress specs by their canonical string, for the inventory census.
    pub stress_specs: BTreeMap<String, u64>,
    /// Letter-level mismatches by (class, cell): the class table's residue.
    pub letter_misses: BTreeMap<(String, String), u64>,
    /// Which alternative the attested primaries matched, by (class, cell):
    /// the table's primary should be the majority.
    pub alt_preference: BTreeMap<(String, String), BTreeMap<usize, u64>>,
    /// (marked, unmarked) counts of attested primaries matching the
    /// table's primary alternative, by (class, cell): the number mark
    /// belongs where marked is the majority.
    pub mark_preference: BTreeMap<(String, String), (u64, u64)>,
    /// Per base paradigm (`a`/`b`) and cell: (stem, end) evidence counts
    /// across lexemes — where a cell disagrees with its base systematically,
    /// the named paradigm should say so.
    pub stress_cells: BTreeMap<(String, String), (u64, u64)>,
    /// Stress-only misses (letters right, stress wrong after the fit):
    /// (lemma, stress column, cell, attested, predicted), for the review.
    pub stress_miss_samples: Vec<(String, String, String, String, String)>,
    /// True exceptions (no alternative fits): (lemma, class, stress, cell,
    /// attested, predicted).
    pub exception_samples: Vec<(String, String, String, String, String, String)>,
}

impl Outcome {
    pub fn bump(&mut self, name: &'static str) {
        *self.counts.entry(name).or_default() += 1;
    }
}

pub fn lexicon_dir() -> PathBuf {
    crate::workspace_root().join("crates/church-slavonic/lexicon")
}

pub fn intermediate_dir() -> PathBuf {
    crate::workspace_root().join("data/intermediate")
}

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let source = args.first().ok_or("import <polyakov> --pos <noun> [--write] [--debug <lemma>]")?;
    if let Some(i) = args.iter().position(|a| a == "--debug") {
        let pos = match args.iter().position(|a| a == "--pos").and_then(|p| args.get(p + 1)).map(String::as_str) {
            Some("adj") => Pos::Adjective,
            Some("verb") => Pos::Verb,
            Some("pron") => Pos::Pronoun,
            Some("closed") => Pos::Closed,
            _ => Pos::Noun,
        };
        return polyakov::debug(pos, args.get(i + 1).ok_or("--debug <lemma>")?);
    }
    let mut pos = None;
    let mut write = false;
    let mut dump = false;
    let mut fix_marks = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pos" => {
                i += 1;
                pos = Some(match args.get(i).map(String::as_str) {
                    Some("noun") => Pos::Noun,
                    Some("adj") => Pos::Adjective,
                    Some("verb") => Pos::Verb,
                    Some("pron") => Pos::Pronoun,
                    Some("closed") => Pos::Closed,
                    other => return Err(format!("--pos {other:?}: noun|adj|verb|pron|closed").into()),
                });
            }
            "--write" => write = true,
            "--dump" => dump = true,
            "--fix-marks" => fix_marks = true,
            other => return Err(format!("unknown argument {other}").into()),
        }
        i += 1;
    }
    let pos = pos.ok_or("--pos is required")?;
    let outcome = match source.as_str() {
        "polyakov" => polyakov::import(pos)?,
        "alypy" | "ruwiktionary" => crosscheck::import(source, pos)?,
        "kaikki" => ocs::import_kaikki(pos)?,
        "ud" => ocs::import_ud(pos)?,
        s => return Err(format!("import {s}: unknown source").into()),
    };
    report(&outcome);
    if fix_marks {
        fix_table_marks(&outcome, pos)?;
    }
    if dump {
        print!("{}", lexicon::format(&outcome.lexemes));
    }
    if write {
        if source == "polyakov" {
            write_outcome(&outcome, Recension::Synodal, pos)?;
        } else if source == "kaikki" || source == "ud" {
            write_outcome(&outcome, Recension::OldChurchSlavonic, pos)?;
        } else {
            crosscheck::write(&outcome, pos)?;
        }
    } else {
        println!("(dry run — pass --write to update the lexicon)");
    }
    Ok(())
}

/// The lexicon file of a part of speech.
pub fn lexicon_file(pos: Pos) -> &'static str {
    match pos {
        Pos::Noun => "nouns.tsv",
        Pos::Adjective => "adjectives.tsv",
        Pos::Verb => "verbs.tsv",
        Pos::Pronoun => "pronouns.tsv",
        Pos::Closed => "closed.tsv",
    }
}

fn report(o: &Outcome) {
    println!("== import report");
    for (k, v) in &o.counts {
        println!("{v:>8}  {k}");
    }
    println!("{:>8}  lexemes kept", o.lexemes.len());
    println!("{:>8}  quarantined", o.quarantine.len());
    // CS_QUARANTINE_SAMPLE=<n> lists the first n quarantined entries
    if let Some(n) = std::env::var("CS_QUARANTINE_SAMPLE").ok().and_then(|v| v.parse::<usize>().ok()) {
        for q in o.quarantine.iter().take(n) {
            println!("          {} [{}] {}: {}", q.lemma, q.source, q.reason, q.detail);
        }
    }
    let mut reasons: BTreeMap<&str, u64> = BTreeMap::new();
    for q in &o.quarantine {
        *reasons.entry(q.reason).or_default() += 1;
    }
    for (r, n) in reasons {
        println!("{n:>8}    {r}");
    }
    let with_overrides = o.lexemes.iter().filter(|l| !l.overrides.is_empty()).count();
    println!(
        "{:>8}  lexemes with overrides ({:.2}%)",
        with_overrides,
        100.0 * with_overrides as f64 / o.lexemes.len().max(1) as f64
    );
    println!("== override cells (top 25)");
    let mut cells: Vec<_> = o.override_cells.iter().collect();
    cells.sort_by(|a, b| b.1.cmp(a.1));
    for (c, n) in cells.iter().take(25) {
        println!("{n:>8}  {c}");
    }
    println!("== letter misses by class and cell (top 30)");
    let mut misses: Vec<_> = o.letter_misses.iter().collect();
    misses.sort_by(|a, b| b.1.cmp(a.1));
    for ((class, cell), n) in misses.iter().take(30) {
        println!("{n:>8}  {class:8} {cell}");
    }
    println!("== alternative preference where the primary is not the majority");
    for ((class, cell), counts) in &o.alt_preference {
        let total: u64 = counts.values().sum();
        let first = counts.get(&0).copied().unwrap_or(0);
        if let Some((best, n)) = counts.iter().max_by_key(|(_, n)| **n)
            && *best != 0
            && total >= 5
        {
            println!("{class:8} {cell:8} alt {best} wins {n}/{total} (alt 0: {first})");
        }
    }
    println!("== number-mark disagreements (class cell: marked/unmarked vs the table)");
    let pos = o.lexemes.first().map(|l| l.pos).unwrap_or(Pos::Noun);
    let table = church_slavonic::paradigm::table(pos);
    let mut disagreements = 0;
    for ((class, cell), (marked, unmarked)) in &o.mark_preference {
        let Some(c) = table.get(class) else { continue };
        let Some(cellv) = church_slavonic::cell::Cell::parse(pos, cell) else { continue };
        let Some(alts) = c.cells.get(&cellv) else { continue };
        let table_mark = match alts.first().map(|a| &a.shape) {
            Some(church_slavonic::paradigm::Shape::Ending { mark, .. }) => *mark,
            _ => continue,
        };
        let majority = marked > unmarked;
        if majority != table_mark && marked + unmarked >= 3 {
            disagreements += 1;
            if disagreements <= 40 {
                println!("{class:8} {cell:8} marked {marked} unmarked {unmarked} (table: {})", if table_mark { "^" } else { "plain" });
            }
        }
    }
    println!("{disagreements} disagreements in all");
    println!("== number-mark split cells (both readings attested on 5+ lexemes)");
    for ((class, cell), (marked, unmarked)) in &o.mark_preference {
        if *marked >= 5 && *unmarked >= 5 {
            println!("{class:8} {cell:8} marked {marked} unmarked {unmarked}");
        }
    }
    println!("== stress evidence by base and cell (stem/end; cells disagreeing with their base)");
    for ((base, cell), (stem, end)) in &o.stress_cells {
        let disagree = if base == "a" { *end } else { *stem };
        let total = stem + end;
        if total >= 20 && disagree * 10 >= total {
            println!("{base} {cell:8} stem {stem:5} end {end:5}");
        }
    }
    println!("== true exceptions (a sample of {})", o.exception_samples.len());
    // CS_SAMPLE_CELL=<cell name> narrows the sample to one cell
    let wanted = std::env::var("CS_SAMPLE_CELL").ok();
    let pool: Vec<_> = o
        .exception_samples
        .iter()
        .filter(|(_, _, _, cell, _, _)| wanted.as_ref().is_none_or(|w| cell == w))
        .collect();
    let step = (pool.len() / 40).max(1);
    for (lemma, class, spec, cell, attested, predicted) in pool.iter().step_by(step).take(40) {
        let spec: String = spec.chars().take(40).collect();
        println!("  {lemma:20} {class:5} {spec:40} {cell:8} attested {attested:22} predicted {predicted}");
    }
    println!("== stress-only misses (a sample of {})", o.stress_miss_samples.len());
    let step = (o.stress_miss_samples.len() / 25).max(1);
    for (lemma, spec, cell, attested, predicted) in o.stress_miss_samples.iter().step_by(step).take(25) {
        println!("  {lemma:20} {spec:24} {cell:8} attested {attested:20} predicted {predicted}");
    }
    println!("== stress specs (top 40 of {})", o.stress_specs.len());
    let mut specs: Vec<_> = o.stress_specs.iter().collect();
    specs.sort_by(|a, b| b.1.cmp(a.1));
    for (s, n) in specs.iter().take(40) {
        println!("{n:>8}  {s}");
    }
}

/// Rewrite the class table so each cell's primary alternative carries the
/// number mark exactly where the attested primaries mark it (majority, at
/// least three observations). A table edit, printed as it is applied.
fn fix_table_marks(o: &Outcome, pos: Pos) -> Result<(), Box<dyn Error>> {
    let path = lexicon_dir().join("classes").join(match pos {
        Pos::Noun => "noun.tsv",
        Pos::Adjective => "adj.tsv",
        Pos::Verb => "verb.tsv",
        Pos::Pronoun => "pronoun.tsv",
        Pos::Closed => return Ok(()),
    });
    let text = std::fs::read_to_string(&path)?;
    let mut header: Vec<String> = Vec::new();
    let mut out = Vec::new();
    let mut changed = 0;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            out.push(line.to_string());
            continue;
        }
        let mut cols: Vec<String> = line.split('\t').map(str::to_string).collect();
        if cols[0] == "class" {
            header = cols.clone();
            out.push(line.to_string());
            continue;
        }
        for (i, name) in header.iter().enumerate().skip(4) {
            let Some((marked, unmarked)) = o.mark_preference.get(&(cols[0].clone(), name.clone())) else { continue };
            if marked + unmarked < 3 {
                continue;
            }
            let want = marked > unmarked;
            let mut alts: Vec<String> = cols[i].split('|').map(str::to_string).collect();
            let first = alts[0].clone();
            if first.starts_with('@') || first.contains(':') {
                continue;
            }
            let has = first.ends_with('^');
            if has != want {
                alts[0] = if want { format!("{first}^") } else { first.trim_end_matches('^').to_string() };
                println!("mark fix: {} {name}: {} -> {}", cols[0], first, alts[0]);
                cols[i] = alts.join("|");
                changed += 1;
            }
        }
        out.push(cols.join("\t"));
    }
    std::fs::write(&path, out.join("\n") + "\n")?;
    println!("{changed} marks changed in {}", path.display());
    Ok(())
}

fn write_outcome(o: &Outcome, recension: Recension, pos: Pos) -> Result<(), Box<dyn Error>> {
    let dir = lexicon_dir();
    let rec = match recension {
        Recension::Synodal => "syn",
        Recension::OldChurchSlavonic => "ocs",
    };
    let path = dir.join(rec).join(lexicon_file(pos));
    // merge: existing hand-edited entries survive; everything else is
    // replaced by the import (matching by id), keeping the variants and
    // provenance the cross-checking sources added (A:/R:/W:)
    let existing = lexicon::parse_in(&std::fs::read_to_string(&path)?, pos, recension)?;
    let mut merged: BTreeMap<String, Lexeme> = BTreeMap::new();
    for l in o.lexemes.iter().cloned() {
        merged.insert(l.id.clone(), l);
    }
    let mut kept_hand = 0;
    for l in existing {
        if l.is_hand_edited() {
            merged.insert(l.id.clone(), l);
            kept_hand += 1;
        } else if let Some(new) = merged.get_mut(&l.id) {
            for token in l.src.iter().filter(|s| !s.starts_with("P:")) {
                if !new.src.contains(token) {
                    new.src.push(token.clone());
                }
            }
            for (cell, variants) in &l.variants {
                let produced: Vec<String> = new.forms(*cell).iter().map(|f| f.print(recension)).collect();
                for v in variants {
                    if !produced.contains(v) {
                        match new.variants.iter_mut().find(|(c, _)| c == cell) {
                            Some((_, vs)) => {
                                if !vs.contains(v) {
                                    vs.push(v.clone());
                                }
                            }
                            None => new.variants.push((*cell, vec![v.clone()])),
                        }
                    }
                }
            }
        }
    }
    let lexemes: Vec<Lexeme> = merged.into_values().collect();
    std::fs::write(&path, lexicon::format(&lexemes))?;
    println!("wrote {} lexemes to {} ({kept_hand} hand-edited kept)", lexemes.len(), path.display());
    let qpath = dir.join("quarantine.tsv");
    let mut text = String::from("# Source entries judged noise, with the reason. Columns: recension pos lemma source reason detail\n");
    let mut lines: Vec<String> = o
        .quarantine
        .iter()
        .map(|q| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                match q.recension {
                    Recension::Synodal => "syn",
                    Recension::OldChurchSlavonic => "ocs",
                },
                q.pos.tag(),
                q.lemma,
                q.source,
                q.reason,
                if q.detail.is_empty() { "-" } else { &q.detail }
            )
        })
        .collect();
    // keep other recensions'/parts' lines
    if let Ok(old) = std::fs::read_to_string(&qpath) {
        for line in old.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let (r, p) = (cols.next().unwrap_or(""), cols.next().unwrap_or(""));
            if r != rec || p != pos.tag() {
                lines.push(line.to_string());
            }
        }
    }
    lines.sort();
    lines.dedup();
    for l in lines {
        text.push_str(&l);
        text.push('\n');
    }
    std::fs::write(&qpath, text)?;
    Ok(())
}
