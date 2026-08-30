//! Table emission — a sorted static slice per part of speech, looked up by
//! binary search. BYTE STABILITY IS LOAD-BEARING here: the
//! `tables_round_trip_committed_output` test parses the committed tables and
//! re-emits them, requiring a byte-identical result — any formatting change
//! (whitespace, ordering, headers) must be committed together with regenerated
//! tables (`cargo xtask refresh-data`). Entries are sorted by key before writing
//! so output is independent of input order.
//!
//! A row is written sparsely — `(cell index, form)` pairs in cell order, the
//! blank cells (the rule's) left out — so the source the compiler sees is
//! proportional to the attested exceptions, not to the row's arity.

use crate::cells::Pos;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// The static table and getter names of each part of speech.
pub fn names(pos: Pos) -> (&'static str, &'static str) {
    match pos {
        Pos::Noun => ("NOUN_TABLE", "get_noun"),
        Pos::Adj => ("ADJ_TABLE", "get_adj"),
        Pos::Verb => ("VERB_TABLE", "get_verb"),
        Pos::Pronoun => ("PRONOUN_TABLE", "get_pronoun"),
    }
}

/// The cell-order documentation printed above each map (see `crate::cells`).
fn doc(pos: Pos) -> &'static str {
    match pos {
        Pos::Noun => {
            "number * 7 + case; numbers Singular, Dual, Plural; cases Nominative, Genitive, Dative, Accusative, Instrumental, Locative, Vocative"
        }
        Pos::Adj => {
            "((degree * 3 + gender) * 3 + number) * 7 + case; degrees Positive, Comparative; genders Masculine, Feminine, Neuter; then as nouns"
        }
        Pos::Verb => {
            "finite blocks Present, Imperfect, Aorist, Imperative at block * 9 + number * 3 + person; 36 present active participle; 37 past active participle"
        }
        Pos::Pronoun => {
            "first person number * 6 + case, second person 18 + the same, third person 36 + (gender * 3 + number) * 6 + case; six cases, no vocative"
        }
    }
}

pub fn write_phf(
    pos: Pos,
    mut rows: Vec<(String, Vec<String>)>,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let (map, getter) = names(pos);
    let arity = pos.arity();
    let mut out = File::create(path)?;
    writeln!(
        out,
        "/// `\"<recension>:<key>\"` -> the attested `(cell, form)` pairs of a {arity}-cell row; a cell not listed falls back to the rule."
    )?;
    writeln!(out, "/// Cell order: {}.", doc(pos))?;
    writeln!(out, "/// Sorted by key; looked up by binary search.")?;
    writeln!(out, "pub static {map}: &[(&str, &[(u16, &str)])] = &[")?;
    for (key, cells) in &rows {
        for text in std::iter::once(key).chain(cells.iter()) {
            if text.contains('"') || text.contains('\\') || text.contains('\n') {
                return Err(io::Error::other(format!(
                    "refusing to emit {key}: a cell contains a quote, backslash or newline"
                )));
            }
        }
        if cells.len() != arity {
            return Err(io::Error::other(format!(
                "refusing to emit {key}: {} cells, expected {arity}",
                cells.len()
            )));
        }
        let cells: Vec<String> = cells
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_empty())
            .map(|(i, c)| format!("({i}, \"{c}\")"))
            .collect();
        writeln!(out, "    (\"{key}\", &[{}]),", cells.join(", "))?;
    }
    writeln!(out, "];")?;
    writeln!(out)?;
    writeln!(
        out,
        "pub fn {getter}(key: &str) -> Option<&'static [(u16, &'static str)]> {{"
    )?;
    writeln!(out, "    {map}.binary_search_by_key(&key, |(k, _)| *k)")?;
    writeln!(out, "        .ok()")?;
    writeln!(out, "        .map(|i| {map}[i].1)")?;
    writeln!(out, "}}")?;
    Ok(())
}
