//! The pinned Bible: the phase-4 source, reused verbatim (found, validated
//! and sha256-pinned in vertograd's PHASE4-PROMPT.md — do not re-research):
//! repo `asdf-a11/ChurchSlavonicBibleInUtf8`, file `CSlElizabeth-CS.json`,
//! sha256 `de40ffb4457c2d61f1330eff631496091ad69046efa08781326cdf733e28dc1e`
//! (12,763,661 bytes), 77 books / 34,470 verses. Fetched by
//! `scripts/fetch-bible.sh` into the gitignored `data/bible-src/`
//! (public-domain text, unlicensed JSON arrangement — nothing vendored);
//! everything here degrades soft when the file is absent.
//!
//! The round-trip target for a verse is [`Verse::print`]: the `text`
//! field trimmed of the arrangement's cosmetic leading space. The source
//! holds NO interior double spaces, tabs or newlines (verified over all
//! 34,470 verses, 2026-09-01), so single-space token joining reproduces
//! the print exactly. Apparatus (`꙾…꙾` variant marks, `[26]` footnote
//! numbers) sits inside verse text and is CARRIED THROUGH — the target is
//! the verse as printed.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Bible {
    pub books: Vec<Book>,
}

#[derive(Deserialize)]
pub struct Book {
    pub name: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Deserialize)]
pub struct Chapter {
    pub chapter: u32,
    pub verses: Vec<Verse>,
}

#[derive(Deserialize)]
pub struct Verse {
    pub verse: u32,
    pub text: String,
}

impl Verse {
    /// The round-trip target: the verse as printed.
    pub fn print(&self) -> &str {
        self.text.trim()
    }
}

/// Where the fetched source lives: `$CS_BIBLE` overrides, else
/// `data/bible-src/CSlElizabeth-CS.json` under the workspace root.
pub fn source_path() -> PathBuf {
    std::env::var_os("CS_BIBLE").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/bible-src/CSlElizabeth-CS.json")
    })
}

/// Load the pinned Bible; `None` when the file is absent (offline) —
/// callers skip soft. A PRESENT file that fails to parse is an error,
/// never a skip.
pub fn load() -> Result<Option<Bible>, Box<dyn std::error::Error>> {
    // a selected corpus (4.1: --corpus ponomar) takes the Bible's place
    if crate::treebank::corpus::current().is_some() {
        return crate::treebank::corpus::load();
    }
    let path = source_path();
    if !path.exists() {
        return Ok(None);
    }
    let file = std::fs::File::open(&path)?;
    let bible: Bible = serde_json::from_reader(std::io::BufReader::new(file))?;
    if bible.books.len() != 77 {
        return Err(format!("pinned source promises 77 books, found {}", bible.books.len()).into());
    }
    Ok(Some(bible))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treebank::node::{render, verbatim_tree};
    use church_slavonic::Recension;

    /// Part 0's gate: tokenize pinned chapters, wrap every token
    /// verbatim, render, and match the print byte-for-byte — Genesis 1
    /// and the whole of Luke (its 15:12 carries the known apparatus
    /// pitfalls). Skips soft offline.
    #[test]
    fn verbatim_wrap_round_trips_pinned_chapters() {
        let Some(bible) = load().expect("present source must parse") else {
            eprintln!("bible source absent — round-trip test skipped");
            return;
        };
        let genesis = &bible.books[0];
        let luke = bible
            .books
            .iter()
            .find(|b| b.name == "Ѿ лꙋкѝ ст҃о́е бл҃говѣствова́нїе")
            .expect("Luke is in the canon");
        let mut checked = 0;
        for chapter in genesis.chapters.iter().take(1).chain(&luke.chapters) {
            for verse in &chapter.verses {
                let tree = verbatim_tree(verse.print());
                let rendered = render(&tree, &Recension::Synodal).expect("renders");
                assert_eq!(rendered, verse.print(), "{} {}", chapter.chapter, verse.verse);
                checked += 1;
            }
        }
        assert!(checked > 1100, "Gen 1 + all Luke should exceed 1100 verses, got {checked}");
        // the pitfall verse is really in the set, apparatus intact
        let v12 = &luke.chapters[14].verses[11];
        assert!(v12.print().contains("꙾є҆ю̀꙾[26]"));
    }
}
