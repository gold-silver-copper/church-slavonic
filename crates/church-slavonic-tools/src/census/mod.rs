//! `cargo xtask census <what> …`: the censuses each design starts from —
//! a number on record before anything moves (V2.1 Part 0, V2.2 Part 0).
//!
//! - `stems --pos <pos> [--ocs]`: stored numbered stems by their relation
//!   to the lemma's stem.
//! - `verb-cells --ocs`: the OCS verb classes' aorist, imperfect and
//!   l-participle cells against what the Leskien type predicts, and the
//!   UD variants in those blocks.
//! - `closed`: the closed lines by subcategory, the adverbs an adjective
//!   produces, the prepositions' case frames counted from the treebank.
//! - `clitics`: the Bible tokens ending in an enclitic written solid.
//! - `homonymy`: the treebank's `:amb` tokens by shape, the sets by size.
//! - `stress`: the Synodal stress columns with an exception list by shape.

pub mod clitics;
pub mod closed;
pub mod homonymy;
pub mod stems;
pub mod stress;
pub mod verb_cells;

use std::error::Error;

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match args.first().map(String::as_str).unwrap_or("") {
        "stems" => stems::run(&args),
        "verb-cells" => verb_cells::run(&args),
        "closed" => closed::run(),
        "clitics" => clitics::run(),
        "homonymy" => homonymy::run(),
        "stress" => stress::run(),
        _ => Err("census <stems --pos <pos> [--ocs] | verb-cells --ocs | closed | clitics | homonymy | stress>".into()),
    }
}

/// Every stored tree of the treebank, (book index, chapter, verse, tree).
pub(crate) fn treebank_trees() -> Result<Vec<(usize, u32, u32, crate::treebank::node::Node)>, Box<dyn Error>> {
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
