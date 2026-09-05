//! `cargo xtask <command>`:
//!
//! - `eval` — the three numbers (held-out recall, Bible coverage, guesser
//!   accuracy);
//! - `build-treebank` / `check-treebank` — the Bible treebank.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("eval") => church_slavonic_tools::eval::run(args.collect()),
        Some("import") => church_slavonic_tools::import::run(args.collect()),
        Some("census") => church_slavonic_tools::census::run(args.collect()),
        Some("refit-stress") => {
            let args: Vec<String> = args.collect();
            let pos = match args.iter().position(|a| a == "--pos").and_then(|p| args.get(p + 1)).map(String::as_str) {
                Some("noun") => church_slavonic::Pos::Noun,
                Some("adj") => church_slavonic::Pos::Adjective,
                Some("verb") => church_slavonic::Pos::Verb,
                Some("pron") => church_slavonic::Pos::Pronoun,
                _ => return Err("refit-stress --pos <noun|adj|verb|pron> [--write]".into()),
            };
            church_slavonic_tools::import::refit::run(pos, args.iter().any(|a| a == "--write"))
        }
        Some("train-tagger") => church_slavonic_tools::tagger::train(&args.collect::<Vec<_>>()),
        Some("build-treebank") => church_slavonic_tools::treebank::runner::run(true),
        Some("check-treebank") => church_slavonic_tools::treebank::runner::run(false),
        Some("fix-hand-alts") => church_slavonic_tools::treebank::runner::fix_hand_alts(),
        Some("narrow-hand") => church_slavonic_tools::treebank::runner::narrow_hand(),
        Some("score-disambiguation") => church_slavonic_tools::treebank::runner::score_disambiguation(),
        Some("hand-draft") => {
            let args: Vec<String> = args.collect();
            let book: usize = args.first().and_then(|a| a.parse().ok()).ok_or("hand-draft <book index> <chapter>")?;
            let chapter: u32 = args.get(1).and_then(|a| a.parse().ok()).ok_or("hand-draft <book index> <chapter>")?;
            church_slavonic_tools::treebank::runner::hand_draft(book, chapter)
        }
        Some("filter-ud") => {
            let root = church_slavonic_tools::workspace_root();
            church_slavonic_tools::sources::ud::filter_train(&root.join("references/downloads"), &root.join("target/sources"), &root.join("data/intermediate/ud_proiel.jsonl"))
        }
        Some("analyze") => {
            let mut args: Vec<String> = args.collect();
            let ocs = args.iter().position(|a| a == "--ocs").map(|i| args.remove(i)).is_some();
            let lexicon = if ocs { church_slavonic::Lexicon::ocs() } else { church_slavonic::Lexicon::synodal() };
            for word in args {
                let form = church_slavonic::Form::from_print(&word);
                println!("{word}: letters {:?} print {}", form.letters, form.print(lexicon.recension));
                for a in lexicon.analyze(&word) {
                    println!("  {} {} alt {} exact {} print {}", a.lexeme.id, a.cell.name(), a.alt, a.exact, a.print);
                }
            }
            Ok(())
        }
        Some("-h") | Some("--help") | None => {
            eprintln!("cargo xtask <eval [--guess verbs [--ocs]] | census <stems --pos <pos> [--ocs] | verb-cells --ocs | closed | clitics | homonymy | stress> | import <source> --pos <pos> [--write] | build-treebank | check-treebank | fix-hand-alts | narrow-hand | score-disambiguation | hand-draft <book> <chapter> | analyze <word>…>");
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command: {other}").into()),
    }
}
