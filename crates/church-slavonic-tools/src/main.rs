//! `cargo xtask <command>`:
//!
//! - `eval [--legacy]` — the three numbers (held-out recall, Bible
//!   coverage, guesser accuracy); `--legacy` prints the 1.2 baselines by
//!   running the legacy harness;
//! - `build-treebank` / `check-treebank` — the Bible treebank.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("eval") => church_slavonic_tools::eval::run(args.collect()),
        Some("import") => church_slavonic_tools::import::run(args.collect()),
        Some("build-treebank") => church_slavonic_tools::treebank::runner::run(true),
        Some("check-treebank") => church_slavonic_tools::treebank::runner::run(false),
        Some("fix-hand-alts") => church_slavonic_tools::treebank::runner::fix_hand_alts(),
        Some("analyze") => {
            let lexicon = church_slavonic::Lexicon::synodal();
            for word in args {
                println!("{word}:");
                for a in lexicon.analyze(&word) {
                    println!("  {} {} alt {} exact {} print {}", a.lexeme.id, a.cell.name(), a.alt, a.exact, a.print);
                }
            }
            Ok(())
        }
        Some("-h") | Some("--help") | None => {
            eprintln!("cargo xtask <eval [--legacy] | import <source> --pos <pos> [--write] | build-treebank | check-treebank | fix-hand-alts | analyze <word>…>");
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command: {other}").into()),
    }
}
