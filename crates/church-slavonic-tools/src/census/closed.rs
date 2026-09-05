//! `census closed`: the closed lines by note tag; the adverbs an
//! adjective already in the lexicon would produce (the neuter short
//! form in -о, printed -ѡ, or the locative in -ѣ), and how many print with
//! the wide ѡ; the prepositions' case frames counted from the auto-lifted
//! treebank (the nominal leaf after `(f …)`, its set by case).

use church_slavonic::cell::Cell;
use church_slavonic::form::Form;
use church_slavonic::orthography::comparison_key;
use church_slavonic::{Lexicon, Pos, Recension};
use std::collections::BTreeMap;
use std::error::Error;

fn tag(note: &str) -> &str {
    note.split([';', ' ']).next().unwrap_or("")
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    let closed: Vec<_> = lexicon.iter().filter(|l| l.pos == Pos::Closed).collect();
    let mut by_tag: BTreeMap<&str, usize> = BTreeMap::new();
    for l in &closed {
        *by_tag.entry(tag(&l.note)).or_default() += 1;
    }
    println!("== closed lines by tag ({} lines)", closed.len());
    for (t, n) in &by_tag {
        println!("{n:>6}  {t}");
    }
    // adverbs an adjective produces: key of the neuter short nominative
    // (-о) or the short locative (-ѣ) → (adjective id, cell, print)
    let mut produced: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
    for adj in lexicon.iter().filter(|l| l.pos == Pos::Adjective) {
        for name in ["short.pos.n.sg.nom", "short.pos.n.sg.loc", "short.pos.m.sg.loc"] {
            let Some(cell) = Cell::parse(Pos::Adjective, name) else { continue };
            for form in adj.forms(cell) {
                let print = form.print(Recension::Synodal);
                produced.entry(comparison_key(&print)).or_default().push((adj.id.clone(), name.to_string(), print));
            }
        }
    }
    let mut produced_same = 0;
    let mut produced_wide = 0;
    let mut produced_differs = 0;
    let mut none = 0;
    let mut examples: Vec<String> = Vec::new();
    for l in closed.iter().filter(|l| tag(&l.note) == "adv") {
        let key = comparison_key(&l.lemma);
        match produced.get(&key) {
            None => none += 1,
            Some(hits) => {
                // the adverb's wide ѡ folded to о for the comparison
                let adv_folded: String = l.lemma.chars().map(|c| if c == 'ѡ' { 'о' } else { c }).collect();
                let wide = l.lemma.ends_with('ѡ') || l.lemma.ends_with("ѡ\u{301}");
                if hits.iter().any(|(_, _, p)| *p == adv_folded || *p == l.lemma) {
                    produced_same += 1;
                    if wide {
                        produced_wide += 1;
                    }
                    if examples.len() < 12 {
                        examples.push(format!("{} ← {} {}", l.lemma, hits[0].0, hits[0].1));
                    }
                } else {
                    produced_differs += 1;
                }
            }
        }
    }
    println!("== adverbs: an adjective produces the letters and the accent {produced_same} (of which printed with the wide ѡ {produced_wide}); the letters only {produced_differs}; no adjective {none}");
    println!("   {}", examples.join("; "));
    // the prepositions' case frames from the treebank
    let preps: BTreeMap<String, String> = closed.iter().filter(|l| tag(&l.note) == "pr").map(|l| (l.id.clone(), l.lemma.clone())).collect();
    let mut frames: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut tokens: BTreeMap<String, usize> = BTreeMap::new();
    for (_, _, _, tree) in super::treebank_trees()? {
        walk(&tree, &preps, &mut frames, &mut tokens);
    }
    println!("== prepositions' case frames (auto-lifted treebank; a set counts under 'nom|acc'-style keys)");
    let mut rows: Vec<_> = tokens.iter().collect();
    rows.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for (prep, n) in rows {
        let f = &frames[prep];
        let mut cases: Vec<_> = f.iter().collect();
        cases.sort_by_key(|(_, k)| std::cmp::Reverse(**k));
        println!("{n:>7}  {prep:<14} {}", cases.iter().map(|(c, k)| format!("{c} {k}")).collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

fn unwrap(node: &crate::treebank::node::Node) -> &crate::treebank::node::Node {
    match node {
        crate::treebank::node::Node::Cap(inner) | crate::treebank::node::Node::Abbr { child: inner, .. } => unwrap(inner),
        other => other,
    }
}

fn walk(node: &crate::treebank::node::Node, preps: &BTreeMap<String, String>, frames: &mut BTreeMap<String, BTreeMap<String, usize>>, tokens: &mut BTreeMap<String, usize>) {
    use crate::treebank::node::Node;
    if let Node::Group { children, .. } = node {
        for (i, child) in children.iter().enumerate() {
            walk(child, preps, frames, tokens);
            let Node::Fn(word) = unwrap(child) else { continue };
            let prep = if let Some(lemma) = preps.get(word) {
                lemma.clone()
            } else if crate::treebank::closed::role(word) == Some("prep") || preps.values().any(|l| comparison_key(l) == comparison_key(word)) {
                Form::from_print(word).letters
            } else {
                continue;
            };
            let Some(next) = children.get(i + 1) else { continue };
            let Node::Lex { cells, .. } = unwrap(next) else { continue };
            let cases: Vec<String> = cells.iter().filter_map(|c| c.case()).map(church_slavonic::cell::case_name).map(str::to_string).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
            if cases.is_empty() {
                continue;
            }
            *tokens.entry(prep.clone()).or_default() += 1;
            *frames.entry(prep).or_default().entry(cases.join("|")).or_default() += 1;
        }
    }
}
