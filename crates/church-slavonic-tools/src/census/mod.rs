//! `cargo xtask census <what> …`: the censuses each design starts from —
//! a number on record before anything moves (V2.1 Part 0, V2.2 Part 0).
//!
//! - `stems --pos <pos> [--ocs]`: stored numbered stems by their relation
//!   to the lemma's stem.
//! - `verb-cells --ocs`: the OCS verb classes' aorist, imperfect and
//!   l-participle cells against what the Leskien type predicts, and the
//!   UD variants in those blocks.
//! - `closed [--write]`: the closed lines by subcategory, the adverbs an
//!   adjective produces, the prepositions' case frames counted from the
//!   treebank (`--write` stores them in `data/prep-frames.tsv`).
//! - `clitics`: the Bible tokens ending in an enclitic written solid.
//! - `homonymy`: the treebank's `:amb` tokens by shape, the sets by size.
//! - `stress`: the Synodal stress columns with an exception list by shape.
//! - `verbatim`: the treebank's verbatim leaves by why the lexicon does
//!   not print them (V3.3 Part 0).

pub mod clitics;
pub mod closed;
pub mod homonymy;
pub mod stems;
pub mod stress;
pub mod forms;
pub mod verb_cells;
pub mod verbatim;

use std::error::Error;

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match args.first().map(String::as_str).unwrap_or("") {
        "stems" => stems::run(&args),
        "verb-cells" => verb_cells::run(&args),
        "closed" => closed::run(args.iter().any(|a| a == "--write")),
        "clitics" => clitics::run(),
        "homonymy" => homonymy::run(),
        "stress" => stress::run(),
        "forms" => forms::run(args.iter().any(|a| a == "--write")),
        "verbatim" => verbatim::run(args.iter().any(|a| a == "--write")),
        _ => Err("census <stems --pos <pos> [--ocs] | verb-cells --ocs | closed | clitics | homonymy | stress | forms [--write]>".into()),
    }
}

/// One stored tree: (book index, chapter, verse, tree).
pub(crate) type StoredTree = (usize, u32, u32, crate::treebank::node::Node);

/// Every stored tree of the treebank.
pub(crate) fn treebank_trees() -> Result<Vec<StoredTree>, Box<dyn Error>> {
    let dir = crate::treebank::runner::treebank_dir();
    let mut out = Vec::new();
    for bi in 0..80 {
        let path = crate::treebank::runner::book_file(&dir, bi);
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for entry in crate::treebank::sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", path.display()))? {
            let (ch, vs, tree) = crate::treebank::runner::read_entry(&entry)?;
            out.push((bi, ch, vs, tree));
        }
    }
    if out.is_empty() {
        return Err("no treebank under treebank/ (run build-treebank)".into());
    }
    Ok(out)
}
