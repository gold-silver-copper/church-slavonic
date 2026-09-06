//! `census forms [--write]`: what the pinned Bible prints for each
//! (lexeme, cell) — the treebank's leaves counted by the form they
//! render, so the importer can choose a cell's primary by attestation in
//! the print where a source's forms disagree (ѻ҆́вцꙋ fourteen to one over
//! ѻ҆вцꙋ̀). `--write` stores `data/treebank-forms.tsv` (lemma, pos, cell,
//! print, count, set_count); the report counts the cells whose commonest
//! print is not the lexicon's primary. `count` is from leaves resolved
//! to one cell by the analyzer or a constraint (a tagger's choice is a
//! guess and is not counted); `set_count` credits every cell of a set
//! leaf with the token, so the importer can tell a form the Bible never
//! prints from one that only ever hides in a set.

use crate::treebank::node::{Node, leaf_print};
use church_slavonic::{Lexicon, Recension};
use std::collections::BTreeMap;
use std::error::Error;

fn walk(node: &Node, out: &mut Vec<(String, String, usize, bool)>) {
    match node {
        Node::Lex { id, cells, alt, notes } if !crate::treebank::runner::tagged(notes) => out.push((id.clone(), cells.name(), *alt, cells.len() == 1)),
        Node::Lex { .. } => {}
        // a closed leaf printed as its primary (`(f id)`, 4.1: without
        // this the alts alone were tallied and the primary lost 1:0)
        Node::Fn(id) if church_slavonic::sentence::node::is_lexeme_id(id) => out.push((id.clone(), "word".to_string(), 0, true)),
        Node::Cap(inner) | Node::Abbr { child: inner, .. } => walk(inner, out),
        Node::Pw { host, .. } => walk(host, out),
        Node::Group { children, .. } => children.iter().for_each(|c| walk(c, out)),
        _ => {}
    }
}

pub fn run(write: bool) -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    // (lemma, pos, cell name, print) → count; and per (id, cell) the alt counts
    let mut counts: BTreeMap<(String, String, String, String), (u64, u64)> = BTreeMap::new();
    let mut alts: BTreeMap<(String, String), BTreeMap<usize, u64>> = BTreeMap::new();
    let mut leaves = 0u64;
    for (_, _, _, tree) in super::treebank_trees()? {
        let mut found = Vec::new();
        walk(&tree, &mut found);
        for (id, set, alt, one) in found {
            let Some(lexeme) = lexicon.get(&id) else { continue };
            let Ok(set) = church_slavonic::cell::CellSet::parse(lexeme.pos, &set) else { continue };
            let Ok(print) = leaf_print(&id, set.first(), alt, Recension::Synodal) else { continue };
            if one {
                leaves += 1;
            }
            for cell in set.iter() {
                let e = counts.entry((lexeme.lemma.clone(), lexeme.pos.tag().to_string(), cell.name(), print.clone())).or_default();
                if one {
                    e.0 += 1;
                    *alts.entry((id.clone(), cell.name())).or_default().entry(alt).or_default() += 1;
                } else {
                    e.1 += 1;
                }
            }
        }
    }
    let mut cells = 0usize;
    let mut disputed = 0usize;
    let mut primary_loses = 0usize;
    let mut samples: Vec<String> = Vec::new();
    // 3.0 Part 0.3: the outnumbered primaries by kind — the citation cell
    // (exempt: ids stable), a letter variant (not a stress twin), a stress
    // twin the Bible also prints inside a set leaf (exempt: the arbiter
    // counts only unambiguous prints), a stress twin never inside a set
    // (the arbiter's own material: should be none after the import)
    let mut kinds: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for ((id, cell), by_alt) in &alts {
        cells += 1;
        if by_alt.len() < 2 {
            continue;
        }
        disputed += 1;
        let (best, n) = by_alt.iter().max_by_key(|(_, n)| **n).map(|(a, n)| (*a, *n)).unwrap_or((0, 0));
        // strictly outnumbered: a tie keeps the primary
        if best != 0 && n > by_alt.get(&0).copied().unwrap_or(0) {
            primary_loses += 1;
            if samples.len() < 20 {
                samples.push(format!("{id} {cell}: alt {best} printed {n}, the primary {}", by_alt.get(&0).copied().unwrap_or(0)));
            }
            if let Some(lexeme) = lexicon.get(id) {
                let forms: Vec<String> = church_slavonic::cell::CellSet::parse(lexeme.pos, cell)
                    .map(|set| lexeme.forms(set.first()).iter().map(|f| f.print(Recension::Synodal)).collect())
                    .unwrap_or_default();
                let primary = forms.first().cloned().unwrap_or_default();
                let winner = forms.get(best).cloned().unwrap_or_default();
                let same_letters = church_slavonic::Form::from_print(&primary).key() == church_slavonic::Form::from_print(&winner).key();
                let in_a_set = |print: &str| counts.get(&(lexeme.lemma.clone(), lexeme.pos.tag().to_string(), cell.clone(), print.to_string())).is_some_and(|(_, sets)| *sets > 0);
                let kind = if primary == lexeme.lemma {
                    "the citation cell"
                } else if !same_letters {
                    "a letter variant"
                } else if in_a_set(&primary) || in_a_set(&winner) {
                    "a stress twin the Bible also prints inside a set"
                } else {
                    "a stress twin never inside a set"
                };
                kinds.entry(kind).or_default().push(format!("{id} {cell} {primary} < {winner} ({n}:{})", by_alt.get(&0).copied().unwrap_or(0)));
            }
        }
    }
    println!("census forms: {leaves} one-cell leaves, {cells} (lexeme, cell) pairs; {disputed} pairs printed in more than one form, the lexicon's primary outnumbered in {primary_loses}");
    for s in &samples {
        println!("  {s}");
    }
    println!("the outnumbered primaries by kind: {}", kinds.iter().map(|(k, v)| format!("{k} {}", v.len())).collect::<Vec<_>>().join("; "));
    for (k, v) in &kinds {
        println!("  {k}: {}", v.iter().take(6).cloned().collect::<Vec<_>>().join(", "));
    }
    if write {
        let path = crate::workspace_root().join("data/treebank-forms.tsv");
        let mut out = String::from("lemma\tpos\tcell\tprint\tcount\tset_count\n");
        for ((lemma, pos, cell, print), (n, sets)) in &counts {
            out.push_str(&format!("{lemma}\t{pos}\t{cell}\t{print}\t{n}\t{sets}\n"));
        }
        std::fs::write(&path, out)?;
        println!("wrote {} ({} rows)", path.display(), counts.len());
    }
    Ok(())
}
