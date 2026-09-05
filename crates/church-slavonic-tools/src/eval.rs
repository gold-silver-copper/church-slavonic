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
    println!("held-out recall (UD PROIEL dev+test): n/a until the OCS lexicon (Part 4)");
    match bible_coverage()? {
        Some(c) => println!(
            "Bible coverage, Synodal nouns (analyzer over {} tokens): one reading {} ({:.2}%), several {} ({:.2}%), none {} ({:.2}%); index {} entries in {:.2?}",
            c.tokens,
            c.one,
            100.0 * c.one as f64 / c.tokens.max(1) as f64,
            c.many,
            100.0 * c.many as f64 / c.tokens.max(1) as f64,
            c.none,
            100.0 * c.none as f64 / c.tokens.max(1) as f64,
            c.index_entries,
            c.index_time
        ),
        None => println!("Bible coverage (analyzer):            pinned Bible absent (scripts/fetch-bible.sh)"),
    }
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

pub struct BibleCoverage {
    pub tokens: usize,
    pub one: usize,
    pub many: usize,
    pub none: usize,
    pub index_entries: usize,
    pub index_time: std::time::Duration,
}

/// Every word token of the pinned Bible (punctuation split off,
/// apparatus tokens skipped) through the Synodal analyzer, EXACT readings
/// only: one, several, none.
pub fn bible_coverage() -> Result<Option<BibleCoverage>, Box<dyn Error>> {
    let Some(bible) = crate::treebank::bible::load()? else {
        return Ok(None);
    };
    let lexicon = Lexicon::synodal();
    let started = std::time::Instant::now();
    let index_entries = lexicon.index().len();
    let index_time = started.elapsed();
    let mut c = BibleCoverage { tokens: 0, one: 0, many: 0, none: 0, index_entries, index_time };
    for book in &bible.books {
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                for token in crate::treebank::node::tokenize(verse.print()) {
                    let Some(core) = crate::treebank::lift::token_core(token) else { continue };
                    c.tokens += 1;
                    let looked_up = crate::treebank::lift::decapitalized(core).unwrap_or_else(|| core.to_string());
                    let n = lexicon.analyze(&looked_up).into_iter().filter(|a| a.exact).count();
                    match n {
                        0 => c.none += 1,
                        1 => c.one += 1,
                        _ => c.many += 1,
                    }
                }
            }
        }
    }
    Ok(Some(c))
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
