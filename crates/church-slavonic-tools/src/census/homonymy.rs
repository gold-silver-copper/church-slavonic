//! `census homonymy`: the treebank's `:amb` tokens by shape — several
//! lexemes of one part of speech, several parts of speech, a closed-class
//! word beside an inflected form — with the commonest surfaces; and the
//! underspecified sets by size.

use church_slavonic::cell::Cell;
use church_slavonic::Lexicon;
use std::collections::BTreeMap;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    let mut shapes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut surfaces: BTreeMap<String, usize> = BTreeMap::new();
    let mut sizes: BTreeMap<usize, usize> = BTreeMap::new();
    let mut set_names: BTreeMap<String, usize> = BTreeMap::new();
    let mut amb = 0;
    for (_, _, _, tree) in super::treebank_trees()? {
        walk(&tree, lexicon, &mut shapes, &mut surfaces, &mut sizes, &mut set_names, &mut amb);
    }
    twins(lexicon, &surfaces);
    println!("== several-lexeme tokens (:amb): {amb}");
    for (s, n) in &shapes {
        println!("{n:>7}  {s}");
    }
    let mut top: Vec<_> = surfaces.iter().collect();
    top.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    println!("commonest: {}", top.iter().take(20).map(|(s, n)| format!("{s} {n}")).collect::<Vec<_>>().join(", "));
    println!("== underspecified leaves by set size");
    for (k, n) in &sizes {
        println!("{n:>7}  {k} cells");
    }
    let mut names: Vec<_> = set_names.iter().collect();
    names.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    println!("commonest sets: {}", names.iter().take(20).map(|(s, n)| format!("{s} {n}")).collect::<Vec<_>>().join(", "));
    Ok(())
}

/// 3.0 Part 0.4: the lexicon's twins. For every `:amb` surface, the
/// lexemes of one part of speech that share it, in pairs; a pair whose
/// lines print identical forms for every cell both declare is a twin (one
/// lexeme held twice), one whose forms are a subset of the other's is the
/// same lexeme with fewer attestations, the rest differ (a genuine
/// homonym pair). Counted by token and by pair.
fn twins(lexicon: &Lexicon, surfaces: &BTreeMap<String, usize>) {
    use std::collections::{BTreeSet, HashMap};
    let mut cache: HashMap<String, BTreeMap<String, BTreeSet<String>>> = HashMap::new();
    let mut paradigm = |id: &str| -> BTreeMap<String, BTreeSet<String>> {
        if let Some(p) = cache.get(id) {
            return p.clone();
        }
        let p: BTreeMap<String, BTreeSet<String>> = lexicon
            .get(id)
            .map(|l| l.all_forms().into_iter().map(|(c, f)| (c.name(), f.into_iter().map(|(_, p)| p).collect())).collect())
            .unwrap_or_default();
        cache.insert(id.to_string(), p.clone());
        p
    };
    let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut top: Vec<(&String, &usize)> = surfaces.iter().collect();
    top.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    println!("== the commonest several-lexeme surfaces with their lexemes");
    for (surface, n) in top.iter().take(20) {
        let ids: Vec<String> = lexicon.readings(surface).into_iter().filter(|r| r.exact).map(|r| format!("{} {}", r.lexeme.id, r.cell_set().map(|c| c.name()).unwrap_or_default())).collect();
        println!("{n:>7}  {surface}  {}", ids.join(" | "));
    }
    for (surface, n) in surfaces {
        let readings: Vec<_> = lexicon.readings(surface).into_iter().filter(|r| r.exact && r.lexeme.pos != church_slavonic::Pos::Closed).collect();
        for (i, a) in readings.iter().enumerate() {
            for b in readings.iter().skip(i + 1) {
                if a.lexeme.pos != b.lexeme.pos || a.lexeme.id == b.lexeme.id {
                    continue;
                }
                let key = if a.lexeme.id < b.lexeme.id { (a.lexeme.id.clone(), b.lexeme.id.clone()) } else { (b.lexeme.id.clone(), a.lexeme.id.clone()) };
                *pairs.entry(key).or_default() += n;
            }
        }
    }
    let mut kinds: BTreeMap<&'static str, (usize, usize, Vec<String>)> = BTreeMap::new();
    for ((a, b), n) in &pairs {
        let pa = paradigm(a);
        let pb = paradigm(b);
        let shared: Vec<&String> = pa.keys().filter(|k| pb.contains_key(*k)).collect();
        let equal = !shared.is_empty() && shared.iter().all(|k| pa[*k] == pb[*k]);
        let sub = |x: &BTreeMap<String, BTreeSet<String>>, y: &BTreeMap<String, BTreeSet<String>>| x.iter().all(|(k, v)| y.get(k).is_some_and(|w| v.is_subset(w)));
        let kind = if equal && pa.len() == pb.len() {
            "twin: identical forms in every cell"
        } else if sub(&pa, &pb) || sub(&pb, &pa) {
            "subset: one line's forms inside the other's"
        } else if equal {
            "same forms where both declare the cell, more cells on one side"
        } else {
            "differ: a genuine pair"
        };
        let e = kinds.entry(kind).or_default();
        e.0 += 1;
        e.1 += n;
        e.2.push(format!("{a}+{b} {n}"));
    }
    println!("== lexeme pairs of one part of speech sharing a surface, by kind (pairs, tokens)");
    for (k, (p, t, ex)) in kinds.iter_mut() {
        ex.sort_by_key(|s| std::cmp::Reverse(s.rsplit(' ').next().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0)));
        println!("{p:>6} pairs {t:>7} tokens  {k}: {}", ex.iter().take(12).cloned().collect::<Vec<_>>().join(", "));
    }
}

#[allow(clippy::too_many_arguments)]
fn walk(node: &crate::treebank::node::Node, lexicon: &Lexicon, shapes: &mut BTreeMap<&'static str, usize>, surfaces: &mut BTreeMap<String, usize>, sizes: &mut BTreeMap<usize, usize>, set_names: &mut BTreeMap<String, usize>, amb: &mut usize) {
    use crate::treebank::node::Node;
    match node {
        Node::W { surface, notes } if notes.iter().any(|(k, _)| k == "amb") => {
            *amb += 1;
            let looked_up = crate::treebank::lift::decapitalized(surface).unwrap_or_else(|| surface.clone());
            let readings: Vec<_> = lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).collect();
            let closed = crate::treebank::closed::is_closed(&looked_up) || readings.iter().any(|r| r.cells.iter().all(|(c, _)| *c == Cell::Word));
            let pos: std::collections::BTreeSet<_> = readings.iter().filter(|r| !r.cells.iter().all(|(c, _)| *c == Cell::Word)).map(|r| r.lexeme.pos).collect();
            let shape = if closed && !pos.is_empty() {
                "a closed-class word beside an inflected form"
            } else if pos.len() > 1 {
                "several parts of speech"
            } else {
                "several lexemes of one part of speech"
            };
            *shapes.entry(shape).or_default() += 1;
            *surfaces.entry(looked_up).or_default() += 1;
        }
        Node::Lex { cells, .. } => {
            *sizes.entry(cells.len()).or_default() += 1;
            if cells.len() > 1 {
                *set_names.entry(cells.name()).or_default() += 1;
            }
        }
        Node::Cap(child) | Node::Abbr { child, .. } | Node::Pw { host: child, .. } => walk(child, lexicon, shapes, surfaces, sizes, set_names, amb),
        Node::Group { children, .. } => {
            for c in children {
                walk(c, lexicon, shapes, surfaces, sizes, set_names, amb);
            }
        }
        _ => {}
    }
}
