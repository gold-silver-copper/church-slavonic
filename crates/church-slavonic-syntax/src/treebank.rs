//! The treebank: one `.sexp` file per book under `treebank/` (gitignored
//! — derived from the pinned text, regenerable; the pipeline and the
//! check are what is committed). Each entry is
//! `(verse <chapter> <verse> <tree>)`.
//!
//! `build` auto-lifts every verse; `check` re-renders every stored tree
//! against the pinned print (the round-trip invariant, enforced in bulk)
//! and returns the coverage table. No silent caps: a book that fails to
//! parse or a verse that mismatches is an error with its address, never
//! a skip.

use crate::bible::{self, Bible};
use crate::lift::{Coverage, Index, lift_verse};
use crate::node::{Node, from_sexpr, render, to_sexpr};
use crate::sexpr::{self, Value};
use church_slavonic::Recension;
use std::error::Error;
use std::path::{Path, PathBuf};

/// `treebank/` under the workspace root, or `$CS_TREEBANK`.
pub fn treebank_dir() -> PathBuf {
    std::env::var_os("CS_TREEBANK").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../treebank")
    })
}

fn book_file(dir: &Path, index: usize) -> PathBuf {
    dir.join(format!("b{index:02}.sexp"))
}

fn verse_entry(chapter: u32, verse: u32, tree: &Node) -> Value {
    Value::List(vec![
        Value::Atom("verse".to_string()),
        Value::Atom(chapter.to_string()),
        Value::Atom(verse.to_string()),
        to_sexpr(tree),
    ])
}

fn read_entry(v: &Value) -> Result<(u32, u32, Node), Box<dyn Error>> {
    let Value::List(items) = v else {
        return Err("a treebank entry is a list".into());
    };
    match items.as_slice() {
        [Value::Atom(head), Value::Atom(ch), Value::Atom(vs), tree] if head == "verse" => {
            Ok((ch.parse()?, vs.parse()?, from_sexpr(tree)?))
        }
        _ => Err(format!("malformed entry: {}", sexpr::print(v)).into()),
    }
}

/// One row of the coverage table.
pub struct BookReport {
    pub name: String,
    pub verses: usize,
    pub coverage: Coverage,
}

/// Auto-lift the whole pinned Bible into `dir`. Returns the per-book
/// coverage of what was WRITTEN (the check recomputes it from disk).
pub fn build(bible: &Bible, index: &Index, dir: &Path) -> Result<Vec<BookReport>, Box<dyn Error>> {
    std::fs::create_dir_all(dir)?;
    let recension = Recension::Synodal;
    let mut reports = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let mut out = String::new();
        let mut coverage = Coverage::default();
        let mut verses = 0;
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                let (tree, c) = lift_verse(verse.print(), index);
                // the invariant is checked at BUILD time too — a tree
                // that does not round-trip never reaches disk
                let rendered = render(&tree, &recension)?;
                if rendered != verse.print() {
                    return Err(format!(
                        "{} {}:{} does not round-trip at build",
                        book.name, chapter.chapter, verse.verse
                    )
                    .into());
                }
                out.push_str(&sexpr::print(&verse_entry(chapter.chapter, verse.verse, &tree)));
                out.push('\n');
                coverage.add(c);
                verses += 1;
            }
        }
        std::fs::write(book_file(dir, bi), out)?;
        reports.push(BookReport { name: book.name.clone(), verses, coverage });
    }
    Ok(reports)
}

/// Classify one stored tree's leaves for the coverage table.
fn tree_coverage(node: &Node, coverage: &mut Coverage) {
    match node {
        Node::W { surface, notes } => {
            if surface.contains('꙾') || surface.contains('[') {
                coverage.apparatus += 1;
            } else if notes.iter().any(|(k, _)| k == "amb") {
                coverage.ambiguous += 1;
            } else {
                coverage.verbatim += 1;
            }
        }
        Node::Fn(_) => coverage.closed += 1,
        Node::Noun { .. } | Node::Adj { .. } | Node::Verb { .. } | Node::LPart { .. } => {
            coverage.analyzed += 1;
        }
        Node::Punct(_) => {}
        Node::Cap(child) => tree_coverage(child, coverage),
        Node::Group { children, .. } => {
            for child in children {
                tree_coverage(child, coverage);
            }
        }
    }
}

/// Re-render every stored tree against the pinned print; return the
/// coverage table. A mismatch is an error naming book/chapter/verse.
pub fn check(bible: &Bible, dir: &Path) -> Result<Vec<BookReport>, Box<dyn Error>> {
    let recension = Recension::Synodal;
    let mut reports = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let path = book_file(dir, bi);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{}: {e} (run build-treebank first)", path.display()))?;
        let entries = sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut by_address = std::collections::HashMap::new();
        for entry in &entries {
            let (ch, vs, tree) = read_entry(entry)?;
            by_address.insert((ch, vs), tree);
        }
        let mut coverage = Coverage::default();
        let mut verses = 0;
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                let Some(tree) = by_address.get(&(chapter.chapter, verse.verse)) else {
                    return Err(format!(
                        "{} {}:{} has no tree — a missing verse is a defect, not a skip",
                        book.name, chapter.chapter, verse.verse
                    )
                    .into());
                };
                let rendered = render(tree, &recension)?;
                if rendered != verse.print() {
                    return Err(format!(
                        "ROUND-TRIP BROKEN at {} {}:{}\n  print:    {}\n  rendered: {}",
                        book.name, chapter.chapter, verse.verse, verse.print(), rendered
                    )
                    .into());
                }
                tree_coverage(tree, &mut coverage);
                verses += 1;
            }
        }
        reports.push(BookReport { name: book.name.clone(), verses, coverage });
    }
    Ok(reports)
}

/// Render the coverage table (Markdown, README-ready).
pub fn coverage_table(reports: &[BookReport]) -> String {
    let mut out = String::from(
        "| Book | Verses | Tokens | Analyzed | Closed | Ambiguous | Verbatim | Apparatus |\n|---|---|---|---|---|---|---|---|\n",
    );
    let mut total = Coverage::default();
    let mut total_verses = 0;
    let pct = |part: usize, whole: usize| {
        if whole == 0 { 0.0 } else { 100.0 * part as f64 / whole as f64 }
    };
    for r in reports {
        let c = r.coverage;
        out.push_str(&format!(
            "| {} | {} | {} | {} ({:.1}%) | {} | {} | {} | {} |\n",
            r.name,
            r.verses,
            c.total(),
            c.analyzed,
            pct(c.analyzed, c.total()),
            c.closed,
            c.ambiguous,
            c.verbatim,
            c.apparatus,
        ));
        total.add(c);
        total_verses += r.verses;
    }
    out.push_str(&format!(
        "| **All** | **{}** | **{}** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{}** |\n",
        total_verses,
        total.total(),
        total.analyzed,
        pct(total.analyzed, total.total()),
        total.closed,
        pct(total.closed, total.total()),
        total.ambiguous,
        pct(total.ambiguous, total.total()),
        total.verbatim,
        pct(total.verbatim, total.total()),
        total.apparatus,
    ));
    out
}

/// The whole pipeline for xtask: build (when asked) then check, print
/// the table. Offline-soft ONLY on the absent source.
pub fn run(build_first: bool) -> Result<(), Box<dyn Error>> {
    let Some(bible) = bible::load()? else {
        println!(
            "treebank: pinned Bible source absent (run scripts/fetch-bible.sh) — skipped"
        );
        return Ok(());
    };
    let dir = treebank_dir();
    if build_first {
        let index = Index::build(&Recension::Synodal);
        println!("inverse index: {} distinct surfaces", index.len());
        build(&bible, &index, &dir)?;
    }
    let reports = check(&bible, &dir)?;
    println!("{}", coverage_table(&reports));
    println!("round-trip: every stored tree matches the pinned print byte-for-byte");
    Ok(())
}
