use church_slavonic::Recension;
use church_slavonic_syntax::{bible, lift, node};
use std::collections::BTreeMap;

fn main() {
    let bible = bible::load().expect("parse").expect("present");
    let index = lift::Index::build(&Recension::Synodal);
    let gen1 = &bible.books[0].chapters[0];
    let mut amb: BTreeMap<String, usize> = BTreeMap::new();
    let mut verb: BTreeMap<String, usize> = BTreeMap::new();
    for v in &gen1.verses {
        for token in node::tokenize(v.print()) {
            let (nodes, fate) = lift::lift_token(token, &index);
            let core = nodes.iter().find_map(|n| match n {
                node::Node::W { surface, .. } => Some(surface.clone()),
                _ => None,
            });
            match fate {
                lift::TokenFate::Ambiguous => {
                    let s = core.unwrap_or_default();
                    let n = index.analyses(&s).len();
                    *amb.entry(format!("{s} ({n}: {:?})", index.analyses(&s).iter().take(4).collect::<Vec<_>>())).or_default() += 1;
                }
                lift::TokenFate::Verbatim => *verb.entry(core.unwrap_or(token.to_string())).or_default() += 1,
                _ => {}
            }
        }
    }
    println!("=== AMBIGUOUS ({}) ===", amb.values().sum::<usize>());
    for (k, c) in &amb { println!("{c:3} {k}"); }
    println!("=== VERBATIM ({}) ===", verb.values().sum::<usize>());
    for (k, c) in &verb { println!("{c:3} {k}"); }
}
