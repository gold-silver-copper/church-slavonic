//! `census stress`: every Synodal stress column with an exception list
//! (`a{gen.pl=E;…}`), the lists normalised to their shape (base paradigm
//! plus the sorted cell → place pairs), the shapes ranked, and how many
//! lines the twelve commonest shapes would absorb as named paradigms.

use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;
use std::error::Error;

/// The shape of a stress column: the base name and its exception pairs
/// sorted, so two lexemes with the same rule read the same.
pub fn shape(column: &str) -> Option<String> {
    let (base, rest) = column.split_once('{')?;
    let inner = rest.strip_suffix('}')?;
    let mut pairs: Vec<&str> = inner.split(';').map(str::trim).filter(|p| !p.is_empty()).collect();
    pairs.sort();
    pairs.dedup();
    Some(format!("{base}{{{}}}", pairs.join(";")))
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    for pos in [Pos::Noun, Pos::Adjective, Pos::Verb, Pos::Pronoun] {
        let mut shapes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut plain: BTreeMap<String, usize> = BTreeMap::new();
        let mut with_list = 0;
        let total = lexicon.iter().filter(|l| l.pos == pos).count();
        for l in lexicon.iter().filter(|l| l.pos == pos) {
            match shape(&l.stress) {
                Some(s) => {
                    with_list += 1;
                    shapes.entry(s).or_default().push(l.lemma.clone());
                }
                None => *plain.entry(l.stress.clone()).or_default() += 1,
            }
        }
        let mut ranked: Vec<_> = shapes.iter().collect();
        ranked.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
        let twelve: usize = ranked.iter().take(12).map(|(_, v)| v.len()).sum();
        println!("== {} stress: {total} lines; plain columns {}; with an exception list {with_list} in {} shapes; the twelve commonest shapes absorb {twelve}", pos.tag(), plain.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), shapes.len());
        for (s, v) in ranked.iter().take(12) {
            println!("{:>6}  {s}  {}", v.len(), v.iter().take(4).cloned().collect::<Vec<_>>().join(", "));
        }
    }
    Ok(())
}
