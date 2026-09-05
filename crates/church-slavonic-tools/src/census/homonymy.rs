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
        Node::Cap(child) | Node::Abbr { child, .. } => walk(child, lexicon, shapes, surfaces, sizes, set_names, amb),
        Node::Group { children, .. } => {
            for c in children {
                walk(c, lexicon, shapes, surfaces, sizes, set_names, amb);
            }
        }
        _ => {}
    }
}
