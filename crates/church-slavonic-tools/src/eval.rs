//! `cargo xtask eval`: the three numbers, each of which can go down —
//! held-out recall (UD PROIEL dev+test, Syntacticus), Bible coverage
//! through the analyzer, and guesser accuracy (leave-one-out over the
//! lexicon). Part 1 fills the guesser number; Part 2 the other two.
//! `--legacy` prints the 1.2 baselines by running the legacy harness.

use church_slavonic::cell::Pos;
use church_slavonic::grammar::Recension;
use church_slavonic::lexicon::Lexicon;
use std::error::Error;
use std::process::Command;

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    if args.iter().any(|a| a == "--legacy") {
        return legacy_baselines();
    }
    println!("held-out recall (UD PROIEL dev+test): n/a (Part 2)");
    println!("Bible coverage (analyzer):            n/a (Part 2)");
    let g = guesser(Lexicon::synodal(), Pos::Noun);
    println!(
        "guesser accuracy, Synodal nouns (leave-one-out over {} lexemes): class {:.2}%, cells {:.2}% ({}/{})",
        g.lexemes,
        100.0 * g.class_right as f64 / g.lexemes.max(1) as f64,
        100.0 * g.cells_right as f64 / g.cells.max(1) as f64,
        g.cells_right,
        g.cells
    );
    Ok(())
}

/// The guesser measured against the lexicon: for every lexeme, guess a
/// lexeme from its lemma alone and compare the class and every primary
/// form of the lexicon's paradigm.
pub struct GuessReport {
    pub lexemes: usize,
    pub class_right: usize,
    pub cells: usize,
    pub cells_right: usize,
}

pub fn guesser(lexicon: &Lexicon, pos: Pos) -> GuessReport {
    let mut r = GuessReport { lexemes: 0, class_right: 0, cells: 0, cells_right: 0 };
    for lexeme in lexicon.iter().filter(|l| l.pos == pos) {
        if lexeme.note.contains("pl-tantum") {
            continue;
        }
        r.lexemes += 1;
        let guessed = lexicon.guess(&lexeme.lemma, pos);
        if guessed.class == lexeme.class {
            r.class_right += 1;
        }
        for (cell, form) in lexeme.paradigm() {
            r.cells += 1;
            if guessed.inflect(cell).map(|f| f.print(lexicon.recension)) == Some(form.print(lexicon.recension)) {
                r.cells_right += 1;
            }
        }
    }
    r
}

/// The 1.2 numbers, from the legacy instruments themselves.
fn legacy_baselines() -> Result<(), Box<dyn Error>> {
    let root = crate::workspace_root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    for command in ["accuracy", "check-treebank"] {
        println!("== legacy {command}");
        let status = Command::new(&cargo)
            .current_dir(&root)
            .args(["run", "-p", "xtask-legacy", "--release", "--", command])
            .status()?;
        if !status.success() {
            return Err(format!("legacy {command} failed").into());
        }
    }
    let _ = Recension::Synodal;
    Ok(())
}
