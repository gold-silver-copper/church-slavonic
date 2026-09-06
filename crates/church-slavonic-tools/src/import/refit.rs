//! `cargo xtask refit-stress --pos <pos>`: every lexeme's stress column
//! re-fitted to the accent-paradigm inventory from its own forms — the
//! cells it prints today are the evidence, and a new column is kept only
//! when every form of the paradigm prints byte-identically. For a file
//! Polyakov's import owns this is a no-op check; for the pronouns, whose
//! lines Alypy's tables made, it is the way the inventory reaches them.

use super::fit::{StressSample, stress_column, stress_sample};
use church_slavonic::lexicon::{self, Lexeme};
use church_slavonic::paradigm::Subject;
use church_slavonic::{Lexicon, Pos, Recension};
use std::collections::BTreeMap;
use std::error::Error;

/// The refitted column of a lexeme, or `None` when the fit would change a
/// form or the lexeme has no class.
pub fn refit(lexeme: &Lexeme) -> Option<String> {
    let class = lexeme.class()?;
    let lemma = lexeme.lemma_form();
    let subject = Subject { lemma: &lemma.letters, animate: lexeme.animate, stems: &lexeme.stems };
    let mut samples: BTreeMap<_, StressSample> = BTreeMap::new();
    let before: Vec<(String, Vec<String>)> = lexeme.all_forms().into_iter().map(|(c, f)| (c.name(), f.into_iter().map(|(_, p)| p).collect())).collect();
    for cell in lexeme.cells() {
        // the class's own primary (not an override) is the evidence
        if lexeme.overrides.iter().any(|(c, _)| *c == cell) {
            continue;
        }
        let Ok(form) = lexeme.inflect(cell) else { continue };
        if let Some(s) = stress_sample(class, &subject, cell, &form.print(lexeme.recension)) {
            samples.insert(cell, s);
        }
    }
    if samples.is_empty() {
        return None;
    }
    let column = stress_column(lexeme.pos, &BTreeMap::new(), &samples, lemma.stress);
    if column == lexeme.stress {
        return None;
    }
    let mut trial = lexeme.clone();
    trial.stress = column.clone();
    let after: Vec<(String, Vec<String>)> = trial.all_forms().into_iter().map(|(c, f)| (c.name(), f.into_iter().map(|(_, p)| p).collect())).collect();
    (after == before).then_some(column)
}

pub fn run(pos: Pos, write: bool) -> Result<(), Box<dyn Error>> {
    let recension = Recension::Synodal;
    let path = super::lexicon_dir().join("syn").join(super::lexicon_file(pos));
    let mut lexemes = lexicon::parse_in(&std::fs::read_to_string(&path)?, pos, recension)?;
    let _ = Lexicon::synodal();
    let mut changed = 0;
    let mut refused = 0;
    let mut samples: Vec<String> = Vec::new();
    for l in &mut lexemes {
        if l.is_hand_edited() {
            continue;
        }
        match refit(l) {
            Some(column) => {
                if samples.len() < 12 {
                    samples.push(format!("{} {} → {column}", l.id, l.stress));
                }
                l.stress = column;
                changed += 1;
            }
            None => {
                if l.stress.contains('{') {
                    refused += 1;
                }
            }
        }
    }
    println!("refit-stress {}: {} lexemes, {changed} columns refitted to the inventory, {refused} lists no paradigm fits without changing a form", pos.tag(), lexemes.len());
    for s in &samples {
        println!("  {s}");
    }
    if write {
        std::fs::write(&path, lexicon::format(&lexemes))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
