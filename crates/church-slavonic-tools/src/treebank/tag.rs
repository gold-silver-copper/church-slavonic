//! The statistical layer of homonymy (V2.2 Part 5), applied AFTER the
//! constraint layer and only where it left several readings: a leaf
//! whose set still has several cells is narrowed to the tagger's choice
//! (`:by … +tagger :prob 0.87`; `:p` is a verb leaf's person), a
//! several-lexeme token whose readings the
//! tagger separates by cell becomes a leaf (`:by tagger :from-lexemes n
//! :prob 0.74`). The choice is recorded, never counted as analysed: the
//! coverage table reports it in its own column, and `CS_NO_TAGGER=1`
//! rebuilds without it.

use crate::treebank::disambiguate::{amb_surface, boundary, fn_word, leaf, leaf_mut, narrow, reduce, Stats};
use crate::treebank::node::Node;
use church_slavonic::cell::Cell;
use church_slavonic::{Lexicon, Pos};
use church_slavonic_tagger::{Candidate, Context, Tagger};

/// Is the tagger on for this run? (`CS_NO_TAGGER=1` turns it off.)
pub fn enabled() -> bool {
    std::env::var_os("CS_NO_TAGGER").is_none()
}

/// The surface a child renders (for the tagger's context), or nothing
/// for a child that fails to render.
pub(crate) fn surface_of(node: &Node, lexicon: &Lexicon) -> Option<String> {
    match node {
        Node::Punct(p) => Some(p.clone()),
        Node::W { surface, .. } => Some(surface.clone()),
        other => crate::treebank::node::render(other, &lexicon.recension).ok(),
    }
}

/// The reading a child settles on, for the next token's context: a leaf
/// with one cell, or a closed-class word.
pub(crate) fn choice_of(node: &Node, lexicon: &Lexicon) -> Option<Candidate> {
    if let Some(Node::Lex { id, cells, .. }) = leaf(node) {
        let pos = lexicon.get(id)?.pos;
        return Some(Candidate { pos, cell: cells.first() });
    }
    if fn_word(node).is_some() {
        return Some(Candidate { pos: Pos::Closed, cell: Cell::Word });
    }
    None
}

/// The lemma a child settles on: an analyzed leaf's lexeme, a closed
/// word's lexeme (or the word itself).
pub(crate) fn lemma_of(node: &Node, lexicon: &Lexicon) -> Option<String> {
    if let Some(Node::Lex { id, .. }) = leaf(node) {
        return lexicon.get(id).map(|l| l.lemma.clone());
    }
    if let Some(w) = fn_word(node) {
        if crate::treebank::node::is_lexeme_id(w) {
            return lexicon.get(w).map(|l| l.lemma.clone());
        }
        return Some(w.to_string());
    }
    None
}

fn note_p(node: &mut Node, p: f32) {
    if let Some(Node::Lex { notes, .. }) = leaf_mut(node) {
        notes.retain(|(k, _)| k != "prob");
        notes.push(("prob".to_string(), format!("{p:.2}")));
    }
}

/// Apply the tagger to one auto-lifted, constrained verse tree, in place.
pub fn tag(tree: &mut Node, lexicon: &Lexicon, tagger: &Tagger) -> Stats {
    let mut stats = Stats::default();
    if tagger.is_empty() {
        return stats;
    }
    let Node::Group { children, .. } = tree else { return stats };
    let n = children.len();
    let surfaces: Vec<Option<String>> = children.iter().map(|c| surface_of(c, lexicon)).collect();
    let mut prev_choice: Option<Candidate> = None;
    for i in 0..n {
        let before = i.checked_sub(1).filter(|j| !boundary(children, *j));
        let after = (i + 1 < n && !boundary(children, i + 1)).then_some(i + 1);
        let ctx = Context {
            surface: surfaces[i].clone().unwrap_or_default(),
            prev: before.and_then(|j| surfaces[j].clone()),
            next: after.and_then(|j| surfaces[j].clone()),
            prev_lemma: before.and_then(|j| lemma_of(&children[j], lexicon)),
            next_lemma: after.and_then(|j| lemma_of(&children[j], lexicon)),
            prev_choice: if before.is_none() { None } else { prev_choice },
        };
        if surfaces[i].is_none() {
            prev_choice = None;
            continue;
        }
        // a leaf the constraints left with several cells
        if let Some(Node::Lex { id, cells, .. }) = leaf(&children[i])
            && cells.len() > 1
            && let Some(pos) = lexicon.get(id).map(|l| l.pos)
        {
            let candidates: Vec<Candidate> = cells.iter().map(|cell| Candidate { pos, cell }).collect();
            if let Some((k, p)) = tagger.choose(&ctx, &candidates) {
                let chosen = candidates[k].cell;
                if narrow(&mut children[i], lexicon, "tagger", move |c| *c == chosen, &mut stats) {
                    note_p(&mut children[i], p);
                }
            }
        } else if let Some(surface) = amb_surface(&children[i]).map(str::to_string) {
            // a several-lexeme token: the readings' (pos, cell) pairs
            let looked_up = crate::treebank::lift::decapitalized(&surface).unwrap_or_else(|| surface.clone());
            let mut candidates: Vec<Candidate> = Vec::new();
            for r in lexicon.readings(&looked_up).into_iter().filter(|r| r.exact) {
                for (cell, _) in &r.cells {
                    let c = Candidate { pos: r.lexeme.pos, cell: *cell };
                    if !candidates.contains(&c) {
                        candidates.push(c);
                    }
                }
            }
            if candidates.len() > 1
                && let Some((k, p)) = tagger.choose(&ctx, &candidates)
            {
                let chosen = candidates[k].cell;
                if reduce(&mut children[i], lexicon, "tagger", move |c| *c == chosen, &mut stats) {
                    note_p(&mut children[i], p);
                }
            }
        }
        prev_choice = choice_of(&children[i], lexicon);
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treebank::lift::Lifter;
    use crate::treebank::node::{render, to_sexpr};
    use crate::treebank::sexpr;

    /// A hand-built model that prefers the accusative to the nominative.
    fn accusative_model() -> Tagger {
        let mut t = Tagger::default();
        t.set("c=acc.sg", 2.0);
        t.set("c=nom.sg", -2.0);
        t
    }

    #[test]
    fn a_tagger_choice_is_always_marked_and_round_trips() {
        let lexicon = church_slavonic::Lexicon::synodal();
        let lifter = Lifter::new(lexicon);
        let verse = "И҆ сотворѝ бг҃ъ свѣ́тъ.";
        let (mut tree, _) = lifter.lift_verse(verse);
        crate::treebank::disambiguate::disambiguate(&mut tree, lexicon);
        // an empty model changes nothing
        let before = sexpr::print(&to_sexpr(&tree));
        let stats = tag(&mut tree, lexicon, &Tagger::default());
        assert!(stats.by_rule.is_empty());
        assert_eq!(sexpr::print(&to_sexpr(&tree)), before);
        // the model picks the accusative for свѣ́тъ, records itself and its
        // confidence, and the verse still renders back
        let stats = tag(&mut tree, lexicon, &accusative_model());
        assert_eq!(stats.by_rule.get("tagger").map(|s| s.0), Some(1), "{stats:?}");
        let text = sexpr::print(&to_sexpr(&tree));
        assert!(text.contains("(n свѣтъ.n :case acc :num sg :from nom|acc.sg :by tagger :prob 0.98)"), "{text}");
        // the verb, where the model has no preference, is left alone
        assert!(text.contains(":cell aor|impv.2|3.sg)"), "{text}");
        assert_eq!(render(&tree, &lexicon.recension).expect("renders"), verse);
        // every leaf the tagger touched says so
        let Node::Group { children, .. } = &tree else { panic!() };
        for c in children {
            if let Some(Node::Lex { cells, notes, .. }) = leaf(c)
                && cells.len() == 1
                && notes.iter().any(|(k, _)| k == "prob")
            {
                assert!(crate::treebank::runner::tagged(notes));
            }
        }
    }
}
