//! The treebank: one `.sexp` file per book under `treebank/` (gitignored
//! — derived from the pinned text, regenerable; the pipeline and the
//! check are what is committed). Each entry is
//! `(verse <chapter> <verse> <tree>)`.
//!
//! `build` auto-lifts every verse from the pinned print through the
//! analyzer (nothing is carried over from an earlier build: a stored tree
//! holds no information the lifter does not recompute); `check`
//! re-renders every stored tree against the pinned print (the round-trip
//! invariant, enforced in bulk) and returns the coverage table. No silent
//! caps: a book that fails to parse or a verse that mismatches is an error
//! with its address, never a skip.

use crate::treebank::bible::{self, Bible};
use crate::treebank::lift::{Coverage, Lifter, RECENSION};
use crate::treebank::node::{Node, from_sexpr, render, to_sexpr};
use crate::treebank::sexpr::{self, Value};
use std::error::Error;
use std::path::{Path, PathBuf};

/// `treebank/` under the workspace root, or `$CS_TREEBANK`.
pub fn treebank_dir() -> PathBuf {
    std::env::var_os("CS_TREEBANK").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../treebank"))
}

pub(crate) fn book_file(dir: &Path, index: usize) -> PathBuf {
    dir.join(format!("b{index:02}.sexp"))
}

fn verse_entry(chapter: u32, verse: u32, tree: &Node) -> Value {
    Value::List(vec![Value::Atom("verse".to_string()), Value::Atom(chapter.to_string()), Value::Atom(verse.to_string()), to_sexpr(tree)])
}

pub(crate) fn read_entry(v: &Value) -> Result<(u32, u32, Node), Box<dyn Error>> {
    let Value::List(items) = v else {
        return Err("a treebank entry is a list".into());
    };
    match items.as_slice() {
        [Value::Atom(head), Value::Atom(ch), Value::Atom(vs), tree] if head == "verse" => Ok((ch.parse()?, vs.parse()?, from_sexpr(tree)?)),
        _ => Err(format!("malformed entry: {}", sexpr::print(v)).into()),
    }
}

/// One row of the coverage table.
pub struct BookReport {
    pub name: String,
    pub verses: usize,
    pub coverage: Coverage,
}

/// Lift the whole pinned Bible into `dir` through the lexicon. Returns
/// the per-book coverage of what was WRITTEN (the check recomputes it).
pub fn build(bible: &Bible, dir: &Path) -> Result<Vec<BookReport>, Box<dyn Error>> {
    std::fs::create_dir_all(dir)?;
    let lexicon = church_slavonic::Lexicon::synodal();
    let started = std::time::Instant::now();
    let lifter = Lifter::new(lexicon);
    println!("titlo index: {} surfaces in {:.2?}", lifter.titlo.len(), started.elapsed());
    let constrain = std::env::var_os("CS_NO_DISAMBIGUATE").is_none();
    let tagger = if constrain && crate::treebank::tag::enabled() { Some(church_slavonic_tagger::Tagger::bundled()) } else { None };
    let mut stats = crate::treebank::disambiguate::Stats::default();
    let mut tag_stats = crate::treebank::disambiguate::Stats::default();
    let mut reports = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let mut out = String::new();
        let mut coverage = Coverage::default();
        let mut verses = 0;
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                let (mut tree, mut c) = lifter.lift_verse(verse.print());
                if constrain {
                    let s = crate::treebank::disambiguate::disambiguate(&mut tree, lexicon);
                    stats.add(&s);
                    if let Some(t) = &tagger {
                        let s = crate::treebank::tag::tag(&mut tree, lexicon, t);
                        tag_stats.add(&s);
                    }
                    // the coverage after the constraints: recount the tree
                    c = Coverage::default();
                    tree_coverage(&tree, &mut c);
                    c.apparatus = 0;
                    for token in crate::treebank::node::tokenize(verse.print()) {
                        if token.contains('꙾') || token.contains('[') {
                            c.apparatus += 1;
                        }
                    }
                }
                // the invariant is checked at BUILD time too — a tree
                // that does not round-trip never reaches disk
                let rendered = render(&tree, &RECENSION)?;
                if rendered != verse.print() {
                    return Err(format!("{} {}:{} does not round-trip at build", book.name, chapter.chapter, verse.verse).into());
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
    if constrain {
        println!("constraints: {}", stats.by_rule.iter().map(|(r, (a, b))| format!("{r} narrowed {a} leaves, reduced {b} tokens")).collect::<Vec<_>>().join("; "));
        match tagger {
            Some(t) if !t.is_empty() => println!("tagger: {}", tag_stats.by_rule.iter().map(|(r, (a, b))| format!("{r} chose {a} leaves, reduced {b} tokens")).collect::<Vec<_>>().join("; ")),
            Some(_) => println!("tagger: no model (cargo xtask train-tagger)"),
            None => println!("tagger: off (CS_NO_TAGGER)"),
        }
    } else {
        println!("constraints: off (CS_NO_DISAMBIGUATE)");
    }
    Ok(reports)
}

/// Did the tagger choose this leaf's cell (`:by … tagger`)?
pub(crate) fn tagged(notes: &[(String, String)]) -> bool {
    notes.iter().any(|(k, v)| k == "by" && v.split('+').any(|r| r == "tagger"))
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
        Node::Punct(_) => {}
        Node::Fn(_) => coverage.closed += 1,
        Node::Lex { cells, notes, .. } => {
            if cells.len() > 1 {
                coverage.underspecified += 1;
            } else if tagged(notes) {
                coverage.tagged += 1;
            } else {
                coverage.analyzed += 1;
            }
        }
        Node::Abbr { child, .. } => match &**child {
            Node::Lex { cells, .. } if cells.len() > 1 => coverage.underspecified += 1,
            Node::Lex { notes, .. } if tagged(notes) => coverage.tagged += 1,
            _ => coverage.analyzed += 1,
        },
        Node::Cap(child) => tree_coverage(child, coverage),
        // a phonological word counts as its host; a solid enclitic is
        // the print's, not a token, one written apart is a closed token
        Node::Pw { host, enclitics, apart } => {
            tree_coverage(host, coverage);
            if *apart {
                coverage.closed += enclitics.len();
            }
        }
        Node::Group { children, .. } => {
            for c in children {
                tree_coverage(c, coverage);
            }
        }
    }
}

/// The hand-lifted overlay: `data/treebank-hand/bNN.sexp`, committed,
/// consulted by the check on top of the generated files.
pub fn hand_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/treebank-hand")
}

/// Re-render every stored tree against the pinned print; return the
/// coverage table (the last row aggregates the hand-lifted entries — the
/// annotation ceiling). A mismatch is an error naming book/chapter/verse.
pub fn check(bible: &Bible, dir: &Path) -> Result<Vec<BookReport>, Box<dyn Error>> {
    let mut reports = Vec::new();
    let mut hand_coverage = Coverage::default();
    let mut hand_verses = 0;
    let lexicon = church_slavonic::Lexicon::synodal();
    let lifter = Lifter::new(lexicon);
    let mut leaves_checked = 0usize;
    let mut incomplete: Vec<String> = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let path = book_file(dir, bi);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e} (run build-treebank first)", path.display()))?;
        let entries = sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut by_address = std::collections::HashMap::new();
        for entry in &entries {
            let (ch, vs, tree) = read_entry(entry)?;
            by_address.insert((ch, vs), tree);
        }
        let mut hand_addresses = std::collections::HashSet::new();
        let hand_path = book_file(&hand_dir(), bi);
        if let Ok(hand_text) = std::fs::read_to_string(&hand_path) {
            let hand_entries = sexpr::parse_many(&hand_text).map_err(|e| format!("{}: {e}", hand_path.display()))?;
            for entry in &hand_entries {
                let (ch, vs, tree) = read_entry(entry)?;
                // hand trees claim structure — their claims get linted;
                // auto-lifted trees are flat and uninteresting to lint
                let findings = crate::treebank::lint::lint(&tree, &RECENSION);
                if !findings.is_empty() {
                    return Err(format!("hand tree {} {ch}:{vs} has lint findings: {:?}", book.name, findings).into());
                }
                hand_addresses.insert((ch, vs));
                by_address.insert((ch, vs), tree);
            }
        }
        let mut coverage = Coverage::default();
        let mut verses = 0;
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                let Some(tree) = by_address.get(&(chapter.chapter, verse.verse)) else {
                    return Err(format!("{} {}:{} has no tree — a missing verse is a defect, not a skip", book.name, chapter.chapter, verse.verse).into());
                };
                let rendered = render(tree, &RECENSION)?;
                if rendered != verse.print() {
                    return Err(format!(
                        "ROUND-TRIP BROKEN at {} {}:{}\n  print:    {}\n  rendered: {}",
                        book.name, chapter.chapter, verse.verse, verse.print(), rendered
                    )
                    .into());
                }
                tree_coverage(tree, &mut coverage);
                if hand_addresses.contains(&(chapter.chapter, verse.verse)) {
                    tree_coverage(tree, &mut hand_coverage);
                    hand_verses += 1;
                } else {
                    // the census: every auto-lifted leaf names every cell of
                    // its lexeme that prints the token — no more, no less
                    for leaf in leaf_prints(tree)? {
                        leaves_checked += 1;
                        let expected = lexeme_cells(&lifter, &leaf);
                        let ok = match (&leaf.from, &leaf.from_lexemes) {
                            // narrowed by a rule: the set it narrowed from is the
                            // lexicon's, the cells a subset of it
                            (Some(from), _) => expected.as_ref() == Some(from) && leaf.cells.iter().all(|c| from.contains(c)),
                            // a several-lexeme token reduced to this lexeme
                            (None, Some(_)) => expected.as_ref().is_some_and(|e| leaf.cells.iter().all(|c| e.contains(c))),
                            (None, None) => expected.as_ref() == Some(&leaf.cells),
                        };
                        if !ok {
                            incomplete.push(format!("{} {}:{} {} «{}»: leaf {} vs lexicon {}", book.name, chapter.chapter, verse.verse, leaf.id, leaf.token, leaf.cells.name(), expected.map(|e| e.name()).unwrap_or_default()));
                        }
                    }
                }
                verses += 1;
            }
        }
        reports.push(BookReport { name: book.name.clone(), verses, coverage });
    }
    if !incomplete.is_empty() {
        return Err(format!("{} of {leaves_checked} leaves do not name every cell that prints their token:\n{}", incomplete.len(), incomplete.iter().take(20).cloned().collect::<Vec<_>>().join("\n")).into());
    }
    println!("leaves: {leaves_checked} auto-lifted leaves name every cell of their lexeme that prints the token");
    if hand_verses > 0 {
        reports.push(BookReport {
            // the ceiling row re-reports verses already counted in their
            // book row — coverage_table keeps it OUT of the totals
            name: HAND_ROW.to_string(),
            verses: hand_verses,
            coverage: hand_coverage,
        });
    }
    Ok(reports)
}

/// One analyzed leaf of a tree with the token it renders (the
/// abbreviated token under an `(abbr …)`, with the row's prefix).
struct LeafPrint {
    id: String,
    cells: church_slavonic::cell::CellSet,
    token: String,
    abbr: Option<String>,
    /// The disambiguator's `:from` set, when the leaf was narrowed.
    from: Option<church_slavonic::cell::CellSet>,
    /// The disambiguator's `:from-lexemes` count, when it reduced a token.
    from_lexemes: Option<usize>,
}

fn leaf_prints(tree: &Node) -> Result<Vec<LeafPrint>, Box<dyn Error>> {
    let mut out = Vec::new();
    collect_prints(tree, None, &mut out)?;
    Ok(out)
}

fn collect_prints(node: &Node, abbr: Option<&Node>, out: &mut Vec<LeafPrint>) -> Result<(), Box<dyn Error>> {
    match node {
        Node::Lex { id, cells, alt, notes } => {
            let token = match abbr {
                Some(a) => render(a, &RECENSION)?,
                None => crate::treebank::node::leaf_print(id, cells.first(), *alt, RECENSION)?,
            };
            let prefix = match abbr {
                Some(Node::Abbr { prefix, .. }) => Some(prefix.clone()),
                _ => None,
            };
            let pos = cells.pos();
            let from = notes.iter().find(|(k, _)| k == "from").and_then(|(_, v)| church_slavonic::cell::CellSet::parse(pos, v));
            let from_lexemes = notes.iter().find(|(k, _)| k == "from-lexemes").and_then(|(_, v)| v.parse().ok());
            out.push(LeafPrint { id: id.clone(), cells: cells.clone(), token, abbr: prefix, from, from_lexemes });
        }
        Node::Abbr { child, .. } => collect_prints(child, Some(node), out)?,
        Node::Cap(child) => collect_prints(child, abbr, out)?,
        // the host's own print names the same cells the unit does
        Node::Pw { host, .. } => collect_prints(host, None, out)?,
        Node::Group { children, .. } => {
            for c in children {
                collect_prints(c, None, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// The cells of the leaf's lexeme that print its token exactly: through
/// the analyzer, or through the titlo index for an abbreviated token.
fn lexeme_cells(lifter: &Lifter<'_>, leaf: &LeafPrint) -> Option<church_slavonic::cell::CellSet> {
    let token = crate::treebank::lift::decapitalized(&leaf.token).unwrap_or_else(|| leaf.token.clone());
    match &leaf.abbr {
        Some(prefix) => lifter.titlo.cells(&token, prefix, &leaf.id),
        None => lifter.lexicon.readings(&token).into_iter().filter(|r| r.exact && r.lexeme.id == leaf.id).find_map(|r| r.cell_set()),
    }
}

/// `cargo xtask narrow-hand`: for every leaf of the hand overlay, is the
/// hand's cell among the cells the lexicon prints the token from? A hand
/// cell outside that set is a finding (the tree claims a cell the lexeme
/// does not print); a set larger than the hand's cell is what the hand
/// disambiguated, reported as a count.
pub fn narrow_hand() -> Result<(), Box<dyn Error>> {
    let Some(bible) = bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lifter = Lifter::new(church_slavonic::Lexicon::synodal());
    let mut leaves = 0;
    let mut narrowed = 0;
    let mut findings = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let hand_path = book_file(&hand_dir(), bi);
        let Ok(text) = std::fs::read_to_string(&hand_path) else { continue };
        let entries = sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", hand_path.display()))?;
        for entry in &entries {
            let (ch, vs, tree) = read_entry(entry)?;
            for leaf in leaf_prints(&tree)? {
                leaves += 1;
                let (id, cells, print) = (&leaf.id, &leaf.cells, &leaf.token);
                match lexeme_cells(&lifter, &leaf) {
                    Some(set) if cells.iter().all(|c| set.contains(c)) => {
                        if set.len() > cells.len() {
                            narrowed += 1;
                            println!("{} {ch}:{vs} {id} «{print}»: hand {} narrows {}", book.name, cells.name(), set.name());
                        }
                    }
                    Some(set) => findings.push(format!("{} {ch}:{vs} {id} «{print}»: hand {} is outside the lexicon's {}", book.name, cells.name(), set.name())),
                    None => findings.push(format!("{} {ch}:{vs} {id} «{print}»: hand {}, the lexicon prints it from no cell", book.name, cells.name())),
                }
            }
        }
    }
    println!("narrow-hand: {leaves} hand leaves, {narrowed} narrow a larger set, {} findings", findings.len());
    for f in &findings {
        println!("  {f}");
    }
    Ok(())
}

/// Repair the `:alt` of every analyzed leaf in the hand overlay: for each
/// verse whose tree does not round-trip, try each leaf's other forms in
/// turn (greedily, left to right) until the verse renders as printed.
/// Reports what it could not repair.
pub fn fix_hand_alts() -> Result<(), Box<dyn Error>> {
    let Some(bible) = bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lexicon = church_slavonic::Lexicon::synodal();
    for (bi, book) in bible.books.iter().enumerate() {
        let hand_path = book_file(&hand_dir(), bi);
        let Ok(text) = std::fs::read_to_string(&hand_path) else { continue };
        let entries = sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", hand_path.display()))?;
        let mut out = String::new();
        let mut fixed = 0;
        for entry in &entries {
            let (ch, vs, mut tree) = read_entry(entry)?;
            let target = book
                .chapters
                .iter()
                .find(|c| c.chapter == ch)
                .and_then(|c| c.verses.iter().find(|v| v.verse == vs))
                .map(|v| v.print().to_string())
                .ok_or_else(|| format!("{} {ch}:{vs} not in the Bible", book.name))?;
            // an alternative index past the cell's forms (a hand cell
            // changed under an old :alt) starts again from the primary
            for leaf in lex_leaves(&mut tree) {
                if let Node::Lex { id, cells, alt, .. } = leaf
                    && lexicon.get(id).is_some_and(|l| l.forms(cells.first()).len() <= *alt)
                {
                    *alt = 0;
                }
            }
            let mut guard = 0;
            let mut changed = true;
            while changed && render(&tree, &RECENSION)? != target && guard < 16 {
                changed = false;
                guard += 1;
                let count = lex_leaves(&mut tree).len();
                for i in 0..count {
                    let (n, current) = {
                        let leaves = lex_leaves(&mut tree);
                        let Node::Lex { id, cells, alt, .. } = &*leaves[i] else { continue };
                        (lexicon.get(id).map(|l| l.forms(cells.first()).len()).unwrap_or(0), *alt)
                    };
                    let before = render(&tree, &RECENSION)?;
                    for k in 0..n {
                        if k == current {
                            continue;
                        }
                        set_alt(&mut tree, i, k);
                        let after = render(&tree, &RECENSION)?;
                        if closer(&after, &target) > closer(&before, &target) {
                            changed = true;
                            fixed += 1;
                            break;
                        }
                        set_alt(&mut tree, i, current);
                    }
                }
            }
            if render(&tree, &RECENSION)? != target {
                println!("{} {ch}:{vs}: still not round-tripping", book.name);
            }
            out.push_str(&sexpr::print(&verse_entry(ch, vs, &tree)));
            out.push('\n');
        }
        // keep the file's leading comment lines
        let comments: String = text.lines().take_while(|l| l.starts_with(';')).map(|l| format!("{l}\n")).collect();
        std::fs::write(&hand_path, format!("{comments}{out}"))?;
        println!("{}: {fixed} :alt repairs", hand_path.display());
    }
    Ok(())
}

fn set_alt(tree: &mut Node, i: usize, k: usize) {
    let mut leaves = lex_leaves(tree);
    if let Node::Lex { alt, .. } = &mut *leaves[i] {
        *alt = k;
    }
}

/// How many leading bytes two renderings share (the greedy repair's score).
fn closer(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

fn lex_leaves(node: &mut Node) -> Vec<&mut Node> {
    let mut out = Vec::new();
    collect_lex(node, &mut out);
    out
}

fn collect_lex<'a>(node: &'a mut Node, out: &mut Vec<&'a mut Node>) {
    match node {
        Node::Lex { .. } => out.push(node),
        Node::Cap(child) | Node::Abbr { child, .. } | Node::Pw { host: child, .. } => collect_lex(child, out),
        Node::Group { children, .. } => {
            for c in children {
                collect_lex(c, out);
            }
        }
        _ => {}
    }
}

/// The ceiling row's name — re-reported verses, excluded from totals.
pub const HAND_ROW: &str = "hand-lifted (the annotation ceiling)";

/// Render the coverage table (Markdown, README-ready).
pub fn coverage_table(reports: &[BookReport]) -> String {
    let mut out = String::from("| Book | Verses | Tokens | Analyzed (one cell) | One lexeme, several cells | Tagger | Closed | Several lexemes | Verbatim | Apparatus |\n|---|---|---|---|---|---|---|---|---|---|\n");
    let mut total = Coverage::default();
    let mut total_verses = 0;
    let pct = |part: usize, whole: usize| if whole == 0 { 0.0 } else { 100.0 * part as f64 / whole as f64 };
    for r in reports {
        let c = r.coverage;
        out.push_str(&format!(
            "| {} | {} | {} | {} ({:.1}%) | {} ({:.1}%) | {} ({:.1}%) | {} | {} | {} | {} |\n",
            r.name,
            r.verses,
            c.total(),
            c.analyzed,
            pct(c.analyzed, c.total()),
            c.underspecified,
            pct(c.underspecified, c.total()),
            c.tagged,
            pct(c.tagged, c.total()),
            c.closed,
            c.ambiguous,
            c.verbatim,
            c.apparatus,
        ));
        if r.name != HAND_ROW {
            total.add(c);
            total_verses += r.verses;
        }
    }
    out.push_str(&format!(
        "| **All** | **{}** | **{}** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{} ({:.1}%)** | **{}** |\n",
        total_verses,
        total.total(),
        total.analyzed,
        pct(total.analyzed, total.total()),
        total.underspecified,
        pct(total.underspecified, total.total()),
        total.tagged,
        pct(total.tagged, total.total()),
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
        println!("treebank: pinned Bible source absent (run scripts/fetch-bible.sh) — skipped");
        return Ok(());
    };
    let dir = treebank_dir();
    if build_first {
        let started = std::time::Instant::now();
        let lexicon = church_slavonic::Lexicon::synodal();
        println!("index: {} entries in {:.2?}", lexicon.index().len(), started.elapsed());
        build(&bible, &dir)?;
        println!("build-treebank: {:.2?}", started.elapsed());
    }
    let reports = check(&bible, &dir)?;
    println!("{}", coverage_table(&reports));
    println!("round-trip: every stored tree matches the pinned print byte-for-byte");
    Ok(())
}

/// The word nodes of a tree in order (punctuation dropped; a
/// phonological word is its host, an abbreviation its child): what two
/// trees of the same verse align on.
fn word_nodes(node: &Node) -> Vec<&Node> {
    fn go<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
        match node {
            Node::Punct(_) => {}
            Node::Group { children, .. } => {
                for c in children {
                    go(c, out);
                }
            }
            Node::Cap(inner) | Node::Abbr { child: inner, .. } => go(inner, out),
            Node::Pw { host, enclitics, apart } => {
                go(host, out);
                if *apart {
                    for e in enclitics {
                        go(e, out);
                    }
                }
            }
            leaf => out.push(leaf),
        }
    }
    let mut out = Vec::new();
    go(node, &mut out);
    out
}

/// The kind of a tagger's wrong cell against the hand's: what a one-token
/// window does not see (the subject against the object of an inanimate,
/// the antecedent's gender) apart from the rest.
fn error_kind(hand: church_slavonic::cell::Cell, auto: church_slavonic::cell::Cell) -> &'static str {
    use church_slavonic::grammar::Case;
    let case_only = hand.gender() == auto.gender() && hand.number() == auto.number() && hand.person() == auto.person();
    match (hand.case(), auto.case()) {
        (Some(h), Some(a)) if h != a && case_only => match (h, a) {
            (Case::Nominative, Case::Accusative) | (Case::Accusative, Case::Nominative) => "nominative against accusative",
            (Case::Locative, Case::Dative) | (Case::Dative, Case::Locative) => "dative against locative (по)",
            (Case::Genitive, Case::Accusative) | (Case::Accusative, Case::Genitive) => "genitive against accusative",
            (Case::Vocative, _) | (_, Case::Vocative) => "the vocative",
            _ => "another case",
        },
        _ if hand.case() == auto.case() && hand.gender() != auto.gender() && hand.number() == auto.number() => "gender",
        _ if hand.case() == auto.case() && hand.number() != auto.number() => "number",
        _ if hand.case() == auto.case() && hand.gender() == auto.gender() && hand.number() == auto.number() => "another feature (person, tense, series)",
        _ => "several features",
    }
}

/// `cargo xtask score-disambiguation`: the constraint layer against the
/// hand overlay. Every hand verse is auto-lifted and constrained; each
/// hand leaf is aligned with the auto word at its position. Precision:
/// the auto leaf's set contains the hand's cell (a rule that excludes a
/// hand cell is wrong). Resolution: the auto set equals the hand's cell.
/// A hand leaf the auto lift left `:amb` or verbatim is out of the
/// constraint layer's reach and counted apart.
pub fn score_disambiguation() -> Result<(), Box<dyn Error>> {
    let Some(bible) = bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lexicon = church_slavonic::Lexicon::synodal();
    let lifter = Lifter::new(lexicon);
    let tagger = crate::treebank::tag::enabled().then(church_slavonic_tagger::Tagger::bundled).filter(|t| !t.is_empty());
    // the tagger's own score: (leaves it chose, right, wrong cell, wrong lexeme)
    let mut tagger_score = (0usize, 0usize, 0usize, 0usize);
    let mut tagger_wrong: Vec<String> = Vec::new();
    // 3.0 Part 0.5: the tagger's errors by kind
    let mut tagger_kinds: std::collections::BTreeMap<&'static str, usize> = std::collections::BTreeMap::new();
    // by confidence: p bucket (tenths) → (chosen, right)
    let mut tagger_buckets: std::collections::BTreeMap<u8, (usize, usize)> = std::collections::BTreeMap::new();
    let mut hand_leaves = 0;
    let mut contained = 0;
    let mut resolved = 0;
    let mut wrong: Vec<String> = Vec::new();
    let mut out_of_reach = 0;
    let mut other_lexeme: Vec<String> = Vec::new();
    let mut misaligned = 0;
    let mut by_rule: std::collections::BTreeMap<String, (usize, usize, usize)> = std::collections::BTreeMap::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let hand_path = book_file(&hand_dir(), bi);
        let Ok(text) = std::fs::read_to_string(&hand_path) else { continue };
        let entries = sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", hand_path.display()))?;
        for entry in &entries {
            let (ch, vs, hand) = read_entry(entry)?;
            let Some(print) = book.chapters.iter().find(|c| c.chapter == ch).and_then(|c| c.verses.iter().find(|v| v.verse == vs)).map(|v| v.print().to_string()) else { continue };
            let (mut auto, _) = lifter.lift_verse(&print);
            crate::treebank::disambiguate::disambiguate(&mut auto, lexicon);
            if let Some(t) = &tagger {
                crate::treebank::tag::tag(&mut auto, lexicon, t);
            }
            let h = word_nodes(&hand);
            let a = word_nodes(&auto);
            if h.len() != a.len() {
                misaligned += 1;
                continue;
            }
            for (hn, an) in h.iter().zip(a.iter()) {
                let Node::Lex { id: hid, cells: hc, .. } = hn else { continue };
                hand_leaves += 1;
                match an {
                    Node::Lex { id: aid, cells: ac, notes, .. } => {
                        let rule = notes.iter().find(|(k, _)| k == "by").map(|(_, v)| v.clone()).unwrap_or_default();
                        if tagged(notes) {
                            tagger_score.0 += 1;
                            let p = notes.iter().find(|(k, _)| k == "prob").map(|(_, v)| v.as_str()).unwrap_or("?");
                            let bucket = p.parse::<f32>().map(|x| ((x * 10.0).floor() as u8).min(9)).unwrap_or(0);
                            let b = tagger_buckets.entry(bucket).or_default();
                            b.0 += 1;
                            if aid == hid && ac.first() == hc.first() {
                                b.1 += 1;
                            }
                            if aid != hid {
                                tagger_score.3 += 1;
                                tagger_wrong.push(format!("{} {ch}:{vs} hand {hid} {} / tagger {aid} {} (p {p})", book.name, hc.name(), ac.name()));
                                let same_pos = lexicon.get(hid).map(|l| l.pos) == lexicon.get(aid).map(|l| l.pos);
                                *tagger_kinds.entry(if same_pos { "another lexeme of the same part of speech" } else { "another part of speech" }).or_default() += 1;
                            } else if ac.first() == hc.first() {
                                tagger_score.1 += 1;
                            } else {
                                tagger_score.2 += 1;
                                tagger_wrong.push(format!("{} {ch}:{vs} {hid}: hand {} / tagger {} (p {p})", book.name, hc.name(), ac.name()));
                                *tagger_kinds.entry(error_kind(hc.first(), ac.first())).or_default() += 1;
                            }
                        }
                        if aid != hid {
                            other_lexeme.push(format!("{} {ch}:{vs} hand {hid} {} / auto {aid} {} {}", book.name, hc.name(), ac.name(), if rule.is_empty() { String::new() } else { format!("(:by {rule})") }));
                            if !rule.is_empty() {
                                by_rule.entry(rule).or_default().2 += 1;
                            }
                            continue;
                        }
                        let hand_cell = hc.first();
                        if ac.contains(hand_cell) {
                            contained += 1;
                            if ac.len() == 1 {
                                resolved += 1;
                            }
                            if !rule.is_empty() {
                                let e = by_rule.entry(rule).or_default();
                                e.0 += 1;
                                if ac.len() == 1 {
                                    e.1 += 1;
                                }
                            }
                        } else {
                            wrong.push(format!("{} {ch}:{vs} {hid}: hand {} outside auto {} (:by {rule})", book.name, hc.name(), ac.name()));
                            by_rule.entry(rule).or_default().2 += 1;
                        }
                    }
                    _ => out_of_reach += 1,
                }
            }
        }
    }
    println!("score-disambiguation: {hand_leaves} hand leaves; auto contains the hand cell {contained} ({:.2}%), resolves it {resolved} ({:.2}%); hand cell outside the auto set {} (precision failures); another lexeme {}; out of reach (auto :amb or verbatim) {out_of_reach}; misaligned verses {misaligned}",
        100.0 * contained as f64 / hand_leaves.max(1) as f64,
        100.0 * resolved as f64 / hand_leaves.max(1) as f64,
        wrong.len(),
        other_lexeme.len());
    for (rule, (ok, res, bad)) in &by_rule {
        println!("  rule {rule:<16} hand cell inside {ok}, resolved {res}, excluded {bad}");
    }
    match &tagger {
        Some(_) => {
            let (chose, right, wrong, lexeme) = tagger_score;
            println!("tagger: chose {chose} hand leaves; right {right} ({:.2}%), wrong cell {wrong}, wrong lexeme {lexeme}", 100.0 * right as f64 / chose.max(1) as f64);
            let mut above_n = 0;
            let mut above_r = 0;
            for (bucket, (n, r)) in tagger_buckets.iter().rev() {
                above_n += n;
                above_r += r;
                println!("  p ≥ 0.{bucket}: chose {above_n}, right {above_r} ({:.2}%); this tenth {n} chosen, {r} right", 100.0 * above_r as f64 / above_n.max(1) as f64);
            }
            println!("  errors by kind: {}", tagger_kinds.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join("; "));
            for w in tagger_wrong.iter().take(if std::env::var_os("CS_ALL").is_some() { usize::MAX } else { 60 }) {
                println!("  TAGGER {w}");
            }
        }
        None => println!("tagger: off (CS_NO_TAGGER or no model)"),
    }
    for w in &wrong {
        println!("  WRONG {w}");
    }
    for o in other_lexeme.iter().take(40) {
        println!("  lexeme {o}");
    }
    Ok(())
}

/// `cargo xtask hand-draft <book> <chapter>`: the auto-lifted trees of a
/// chapter before the constraint layer, one per line, every
/// underspecified set and several-lexeme token listed with its readings
/// — the draft a hand annotator decides from, never a decision.
pub fn hand_draft(book_index: usize, chapter: u32) -> Result<(), Box<dyn Error>> {
    let Some(bible) = bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lexicon = church_slavonic::Lexicon::synodal();
    let lifter = Lifter::new(lexicon);
    let book = bible.books.get(book_index).ok_or("no such book")?;
    let ch = book.chapters.iter().find(|c| c.chapter == chapter).ok_or("no such chapter")?;
    println!("; {} {chapter}", book.name);
    for verse in &ch.verses {
        let (tree, _) = lifter.lift_verse(verse.print());
        println!("; {chapter}:{} {}", verse.verse, verse.print());
        for w in word_nodes(&tree) {
            if let Node::W { surface, notes } = w
                && notes.iter().any(|(k, _)| k == "amb")
            {
                let looked_up = crate::treebank::lift::decapitalized(surface).unwrap_or_else(|| surface.clone());
                let readings: Vec<String> = lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).map(|r| format!("{} {}", r.lexeme.id, r.cell_set().map(|c| c.name()).unwrap_or_else(|| "word".to_string()))).collect();
                println!(";   {surface}: {}", readings.join(" | "));
            }
        }
        println!("{}", sexpr::print(&verse_entry(chapter, verse.verse, &tree)));
    }
    Ok(())
}

