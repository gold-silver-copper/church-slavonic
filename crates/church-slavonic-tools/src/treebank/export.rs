//! `cargo xtask export [--corpus ponomar[/<book>]]` (4.1 Part 2): the
//! stored treebank as one tab-separated file per book with the
//! provenance of every token — the corpus a reader takes. Columns: the
//! chapter, the unit, the token's index, the token as printed, the
//! lexeme id, the cell (or the set), the provenance: `lexicon` (one
//! lexeme, one cell, no rule needed), `rule:<name>` (the eliminations
//! that narrowed it, joined by `+`), `tagger:<p>` (a choice, never a
//! fact), `set` (one lexeme, several cells left), `function` (a closed
//! word), `amb` (several lexemes, none chosen), `verbatim` (no reading),
//! `apparatus`. A manifest beside the files carries each book's coverage
//! table. Nothing here is training material for the tagger; the rules'
//! column is the one exception Part 3 measures.

use crate::treebank::node::Node;
use crate::treebank::runner::{book_file, treebank_dir};
use crate::treebank::{bible, sexpr};
use std::error::Error;
use std::fmt::Write as _;
use std::path::PathBuf;

/// Where the export goes: `export/` under the workspace root, then the
/// corpus's suffix (`export/ponomar/<book>.tsv`; the Bible under
/// `export/bible/`).
pub fn export_dir() -> PathBuf {
    let base = crate::workspace_root().join("export");
    match crate::treebank::corpus::current() {
        None => base.join("bible"),
        Some(_) => base.join(crate::treebank::corpus::dir_suffix()),
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let Some(corpus) = bible::load()? else {
        return Err("nothing to export: the corpus is not fetched".into());
    };
    let dir = treebank_dir();
    let out_dir = export_dir();
    std::fs::create_dir_all(&out_dir)?;
    let lexicon = church_slavonic::Lexicon::synodal();
    let mut manifest = String::from("book\tunits\ttokens\tlexicon\trule\ttagger\tset\tfunction\tamb\tverbatim\tapparatus\n");
    let mut total = [0usize; 9];
    for (bi, book) in corpus.books.iter().enumerate() {
        let path = book_file(&dir, bi);
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let mut out = String::from("chapter\tunit\ttoken\tform\tlexeme\tcell\tprovenance\n");
        let mut counts = [0usize; 9];
        let mut units = 0;
        for entry in sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", path.display()))? {
            let (ch, vs, tree) = crate::treebank::runner::read_entry(&entry)?;
            units += 1;
            let mut k = 0;
            walk(&tree, lexicon, &mut |surface, lexeme, cell, prov| {
                k += 1;
                let _ = writeln!(out, "{ch}\t{vs}\t{k}\t{surface}\t{}\t{}\t{prov}", lexeme.unwrap_or("_"), cell.as_deref().unwrap_or("_"));
                counts[0] += 1;
                counts[slot(prov)] += 1;
            });
        }
        let name = safe_name(&book.name);
        std::fs::write(out_dir.join(format!("{name}.tsv")), out)?;
        let _ = writeln!(manifest, "{}\t{units}\t{}", book.name, counts.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\t"));
        for (t, c) in total.iter_mut().zip(counts.iter()) {
            *t += c;
        }
    }
    let _ = writeln!(manifest, "all\t\t{}", total.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\t"));
    std::fs::write(out_dir.join("MANIFEST.tsv"), &manifest)?;
    println!("{manifest}");
    println!("export: {}", out_dir.display());
    Ok(())
}

/// The manifest's column of a provenance (after `tokens`).
fn slot(prov: &str) -> usize {
    match prov.split(':').next().unwrap_or("") {
        "lexicon" => 1,
        "rule" => 2,
        "tagger" => 3,
        "set" => 4,
        "function" => 5,
        "amb" => 6,
        "verbatim" => 7,
        _ => 8,
    }
}

fn safe_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
}

/// Every token of a tree in order, with its provenance.
fn walk(node: &Node, lexicon: &church_slavonic::Lexicon, emit: &mut dyn FnMut(&str, Option<&str>, Option<String>, &str)) {
    match node {
        Node::Group { children, .. } => {
            for c in children {
                walk(c, lexicon, emit);
            }
        }
        Node::Punct(_) => {}
        Node::Pw { host, enclitics, .. } => {
            walk(host, lexicon, emit);
            for e in enclitics {
                walk(e, lexicon, emit);
            }
        }
        Node::Cap(inner) | Node::Abbr { child: inner, .. } => {
            let surface = church_slavonic::sentence::node::render(node, &lexicon.recension).unwrap_or_default();
            leaf_line(inner, &surface, lexicon, emit);
        }
        other => {
            let surface = church_slavonic::sentence::node::render(other, &lexicon.recension).unwrap_or_default();
            leaf_line(other, &surface, lexicon, emit);
        }
    }
}

fn leaf_line(node: &Node, surface: &str, lexicon: &church_slavonic::Lexicon, emit: &mut dyn FnMut(&str, Option<&str>, Option<String>, &str)) {
    match node {
        Node::Lex { id, cells, notes, .. } => {
            let by = notes.iter().find(|(k, _)| k == "by").map(|(_, v)| v.as_str());
            let prob = notes.iter().find(|(k, _)| k == "prob").map(|(_, v)| v.as_str());
            let prov = match (by, prob, cells.len()) {
                (Some(b), Some(p), _) if b.split('+').any(|r| r == "tagger") => format!("tagger:{p}"),
                (Some(b), _, _) => format!("rule:{b}"),
                (None, _, 1) => "lexicon".to_string(),
                (None, _, _) => "set".to_string(),
            };
            emit(surface, Some(id), Some(cells.name()), &prov);
        }
        Node::Fn(id) => emit(surface, Some(id), None, "function"),
        Node::W { surface: s, notes } => {
            let prov = if church_slavonic::sentence::lift::is_apparatus(s) {
                "apparatus"
            } else if notes.iter().any(|(k, _)| k == "amb") {
                "amb"
            } else {
                "verbatim"
            };
            emit(surface, None, None, prov);
        }
        Node::Cap(inner) | Node::Abbr { child: inner, .. } => leaf_line(inner, surface, lexicon, emit),
        Node::Pw { .. } | Node::Group { .. } | Node::Punct(_) => {}
    }
}
