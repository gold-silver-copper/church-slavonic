//! A second print (4.1 Part 2): the Ponomar library, fetched and pinned
//! by `scripts/fetch-ponomar.sh` under `data/corpus/ponomar/<book>/`
//! (the licence in `data/corpus/ponomar/LICENSE.md`). A book's HTML pages
//! become the treebank's shape — the book, its pages as chapters, its
//! paragraphs as verses — so every command over the Bible runs over a
//! book of the library unchanged: `--corpus ponomar` (every book, one
//! treebank under `treebank/ponomar/`) or `--corpus ponomar/<book>`.
//!
//! The unit is the paragraph as the HTML delimits it (`<p>…</p>`): a
//! troparion, a rubric, a heading, a psalm. Its print is the paragraph's
//! text with the markup removed — the rubrics (`<red>`) are text of the
//! print and stay, the page anchors go, a bracketed note (`[[Ѱ. є҃]]`)
//! is set off by spaces so it is a token of its own (the lifter's
//! apparatus rule takes it), whitespace is collapsed. That text is the
//! round-trip target; the pinned HTML is the source of record.

use crate::treebank::bible::{Bible, Book, Chapter, Verse};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The corpus a command runs over: the Bible by default, or a library
/// (`ponomar`) and optionally one of its books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub library: String,
    pub book: Option<String>,
}

static SELECTED: OnceLock<Selection> = OnceLock::new();

/// `--corpus ponomar` or `--corpus ponomar/<book>`; the Bible when never
/// called. Selected once per process, before any command runs.
pub fn select(spec: &str) -> Result<(), String> {
    let (library, book) = match spec.split_once('/') {
        Some((l, b)) => (l.to_string(), Some(b.to_string())),
        None => (spec.to_string(), None),
    };
    if library != "ponomar" {
        return Err(format!("unknown corpus {library} (ponomar)"));
    }
    if let Some(b) = &book
        && !root().join(b).is_dir()
    {
        return Err(format!("no book {b} under {} (scripts/fetch-ponomar.sh)", root().display()));
    }
    SELECTED.set(Selection { library, book }).map_err(|_| "corpus selected twice".to_string())
}

pub fn current() -> Option<&'static Selection> {
    SELECTED.get()
}

/// The path suffix of the selected corpus under `treebank/` and
/// `data/treebank-hand/` (`ponomar` or `ponomar/<book>`), empty for the
/// Bible.
pub fn dir_suffix() -> PathBuf {
    match current() {
        None => PathBuf::new(),
        Some(s) => match &s.book {
            Some(b) => PathBuf::from(&s.library).join(b),
            None => PathBuf::from(&s.library),
        },
    }
}

/// Where the pinned pages live.
pub fn root() -> PathBuf {
    crate::workspace_root().join("data/corpus/ponomar")
}

/// The books present, in name order.
pub fn books() -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(root())
        .map(|d| d.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).filter_map(|e| e.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

/// Load the selected library (all its books, or the one selected) in the
/// treebank's shape; `None` when nothing is fetched yet.
pub fn load() -> Result<Option<Bible>, Box<dyn std::error::Error>> {
    let Some(sel) = current() else { return Ok(None) };
    let names: Vec<String> = match &sel.book {
        Some(b) => vec![b.clone()],
        None => books(),
    };
    if names.is_empty() {
        return Ok(None);
    }
    let mut out = Vec::new();
    for name in names {
        out.push(load_book(&name)?);
    }
    Ok(Some(Bible { books: out }))
}

/// One book: its pages in file order as chapters, its paragraphs as
/// verses.
pub fn load_book(name: &str) -> Result<Book, Box<dyn std::error::Error>> {
    let dir = root().join(name);
    let mut pages: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| (f.ends_with(".html") || f.ends_with(".htm")) && f != "index.html")
        .collect();
    pages.sort();
    let mut chapters = Vec::new();
    for (pi, page) in pages.iter().enumerate() {
        let html = std::fs::read_to_string(dir.join(page))?;
        let units = paragraphs(&html);
        if units.is_empty() {
            continue;
        }
        let verses = units.into_iter().enumerate().map(|(vi, text)| Verse { verse: (vi + 1) as u32, text }).collect();
        chapters.push(Chapter { chapter: (pi + 1) as u32, verses });
    }
    Ok(Book { name: name.to_string(), chapters })
}

/// The paragraphs of a page, as print text.
pub fn paragraphs(html: &str) -> Vec<String> {
    // the text body: inside <cu> … </cu> where the page has it
    let body = match (html.find("<cu>"), html.rfind("</cu>")) {
        (Some(a), Some(b)) if a < b => &html[a + 4..b],
        _ => html,
    };
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<p") {
        let after = &rest[start + 2..];
        // `<p>` or `<p …>`, not `<pre`
        let Some(gt) = after.find('>') else { break };
        if !after[..gt].chars().next().is_none_or(|c| c == ' ' || c == '\n') {
            rest = &after[gt + 1..];
            continue;
        }
        let inner_start = gt + 1;
        let Some(end) = after[inner_start..].find("</p>") else { break };
        let inner = &after[inner_start..inner_start + end];
        let text = clean(inner);
        if !text.is_empty() {
            out.push(text);
        }
        rest = &after[inner_start + end + 4..];
    }
    out
}

/// A paragraph's markup to its print text.
fn clean(inner: &str) -> String {
    // `[[…]]` notes set off by spaces, tags removed, entities decoded,
    // whitespace collapsed
    let mut s = inner.replace("[[", " [[").replace("]]", " ]]");
    // tags
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.drain(..) {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    uk_digraph(&collapsed)
}

/// The uk written as the digraph — Unicode's narrow о (U+1C82) with у,
/// or the two letters оу at the head of a word — is the one letter ѹ the
/// lexicon prints (3.1): an encoding of the same glyph pair, normalised
/// so the print is the crate's; not a spelling change (the census
/// counts what it explains).
fn uk_digraph(s: &str) -> String {
    let s = s.replace("\u{1c82}у", "ѹ").replace("\u{1c82}У", "Ѹ");
    let mut out = String::with_capacity(s.len());
    let mut prev: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let at_head = prev.is_none_or(|p| !p.is_alphabetic());
        if at_head && (c == 'о' || c == 'О') && chars.peek() == Some(&'у') {
            chars.next();
            out.push(if c == 'о' { 'ѹ' } else { 'Ѹ' });
            prev = Some('ѹ');
            continue;
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_becomes_its_paragraphs() {
        let html = "<cu>\n<p><anchor label=\"73\" page=\"73\"></anchor></p>\n<p><red>Ча́съ пе́рвый.</red></p>\n<p><red>Г</red>лаго́лы[[Ѱ. є҃]] моѧ̑ внꙋшѝ\nгдⷭ҇и, <anchor label=\"74\" page=\"74\"></anchor> разꙋмѣ́й.</p></cu>";
        let ps = paragraphs(html);
        assert_eq!(ps, ["Ча́съ пе́рвый.", "Глаго́лы [[Ѱ. є҃ ]] моѧ̑ внꙋшѝ гдⷭ҇и, разꙋмѣ́й."]);
    }

    #[test]
    fn the_uk_is_one_letter() {
        assert_eq!(uk_digraph("\u{1c82}у҆слы́ши оу҆ста̀ доу́хъ"), "ѹ҆слы́ши ѹ҆ста̀ доу́хъ");
    }
}
