//! The constraint layer (V2.2 Part 4): rules over a verse's readings that
//! ELIMINATE and never SELECT, each named, each recorded on the leaf it
//! narrowed (`:by prep-gov :from nom|acc|voc.sg`; a several-lexeme token
//! reduced to one lexeme carries `:from-lexemes n`). A rule that would
//! leave nothing leaves everything. The structure a rule reads is
//! adjacency in the flat auto-lifted tree: a preposition and the nominal
//! after it, an adjective and the noun beside it, a nominative noun and
//! the finite verb beside it. Scored against the hand overlay by
//! `cargo xtask score-disambiguation`; a rule that ever excludes a hand
//! cell is wrong and goes.

use crate::treebank::node::Node;
use church_slavonic::cell::{Cell, CellSet, VerbCell};
use church_slavonic::grammar::{Case, Gender, Number, Person, Prosody};
use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;

/// What each rule did over a tree (or a treebank).
#[derive(Debug, Default, Clone)]
pub struct Stats {
    /// rule → (leaves narrowed, several-lexeme tokens reduced to one)
    pub by_rule: BTreeMap<&'static str, (usize, usize)>,
}

impl Stats {
    pub fn add(&mut self, other: &Stats) {
        for (k, (a, b)) in &other.by_rule {
            let e = self.by_rule.entry(k).or_default();
            e.0 += a;
            e.1 += b;
        }
    }
    fn narrowed(&mut self, rule: &'static str) {
        self.by_rule.entry(rule).or_default().0 += 1;
    }
    fn reduced(&mut self, rule: &'static str) {
        self.by_rule.entry(rule).or_default().1 += 1;
    }
}

/// The analyzed leaf at a child position, through `cap`, `abbr` and a
/// phonological word's host.
pub(crate) fn leaf_mut(node: &mut Node) -> Option<&mut Node> {
    match node {
        Node::Lex { .. } => Some(node),
        Node::Cap(inner) | Node::Abbr { child: inner, .. } | Node::Pw { host: inner, .. } => leaf_mut(inner),
        _ => None,
    }
}

pub(crate) fn leaf(node: &Node) -> Option<&Node> {
    match node {
        Node::Lex { .. } => Some(node),
        Node::Cap(inner) | Node::Abbr { child: inner, .. } | Node::Pw { host: inner, .. } => leaf(inner),
        _ => None,
    }
}

pub(crate) fn fn_word(node: &Node) -> Option<&str> {
    match node {
        Node::Fn(w) => Some(w),
        // a closed lexeme printed by a variant (во): a word leaf (3.3)
        Node::Lex { id, cells, .. } if cells.iter().all(|c| c == Cell::Word) => Some(id),
        Node::Cap(inner) => fn_word(inner),
        Node::Pw { host, .. } => fn_word(host),
        _ => None,
    }
}

/// A verbatim token with several lexemes: its surface.
pub(crate) fn amb_surface(node: &Node) -> Option<&str> {
    match node {
        Node::W { surface, notes } if notes.iter().any(|(k, _)| k == "amb") => Some(surface),
        _ => None,
    }
}

fn is_nominal(cell: &Cell) -> bool {
    cell.case().is_some()
}

fn is_adjective_like(cell: &Cell) -> bool {
    matches!(cell, Cell::Adj(_) | Cell::Verb(VerbCell::Participle { .. })) || matches!(cell, Cell::Pron(p) if p.gender.is_some() && p.person.is_none())
}

fn is_finite(cell: &Cell) -> bool {
    matches!(cell, Cell::Verb(VerbCell::Finite { .. }))
}

fn is_imperative(cell: &Cell) -> bool {
    matches!(cell, Cell::Verb(VerbCell::Imperative { .. }))
}

/// Narrow a leaf to the cells `keep` admits; records the rule and the set
/// it narrowed from (the first narrowing's). Returns whether it narrowed.
pub(crate) fn narrow(node: &mut Node, lexicon: &Lexicon, rule: &'static str, keep: impl Fn(&Cell) -> bool, stats: &mut Stats) -> bool {
    let Some(Node::Lex { id, cells, notes, alt, .. }) = leaf_mut(node) else { return false };
    let kept: Vec<Cell> = cells.iter().filter(|c| keep(c)).collect();
    if kept.is_empty() || kept.len() == cells.len() {
        return false;
    }
    let Some(set) = CellSet::new(kept) else { return false };
    // the alternative index belongs to the old first cell: the new first
    // cell prints the same token from some alternative of its own — find
    // it, or leave the leaf alone
    if set.first() != cells.first() {
        let Some(lexeme) = lexicon.get(id) else { return false };
        let token = lexeme.forms(cells.first()).get(*alt).map(|f| f.print(lexicon.recension));
        let Some(token) = token else { return false };
        let Some(k) = lexeme.forms(set.first()).iter().position(|f| f.print(lexicon.recension) == token) else { return false };
        *alt = k;
    }
    if !notes.iter().any(|(k, _)| k == "from") {
        notes.push(("from".to_string(), cells.name()));
    }
    *cells = set;
    match notes.iter_mut().find(|(k, _)| k == "by") {
        Some((_, v)) => {
            if !v.split('+').any(|r| r == rule) {
                v.push('+');
                v.push_str(rule);
            }
        }
        None => notes.push(("by".to_string(), rule.to_string())),
    }
    stats.narrowed(rule);
    true
}

/// The cases a function word governs, by id or by print.
fn frame(lexicon: &Lexicon, word: &str) -> Vec<Case> {
    let lexemes: Vec<&church_slavonic::Lexeme> = if crate::treebank::node::is_lexeme_id(word) {
        lexicon.get(word).into_iter().collect()
    } else {
        lexicon.find(word, Pos::Closed)
    };
    let mut out = Vec::new();
    for l in lexemes {
        for c in l.government() {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

fn is_interjection(lexicon: &Lexicon, word: &str) -> bool {
    let lexemes: Vec<&church_slavonic::Lexeme> = if crate::treebank::node::is_lexeme_id(word) {
        lexicon.get(word).into_iter().collect()
    } else {
        lexicon.find(word, Pos::Closed)
    };
    lexemes.iter().any(|l| l.subcategory() == Some("intj"))
}

/// Is the child at `i` a boundary no rule reads across (punctuation)?
pub(crate) fn boundary(children: &[Node], i: usize) -> bool {
    matches!(children.get(i), Some(Node::Punct(_)) | None)
}

/// A several-lexeme token reduced to the readings `keep` admits: when one
/// lexeme is left, a leaf (capitalised as the surface is) that renders
/// the surface back; else unchanged.
pub(crate) fn reduce(node: &mut Node, lexicon: &Lexicon, rule: &'static str, keep: impl Fn(&Cell) -> bool, stats: &mut Stats) -> bool {
    let Some(surface) = amb_surface(node).map(str::to_string) else { return false };
    let looked_up = crate::treebank::lift::decapitalized(&surface).unwrap_or_else(|| surface.clone());
    let readings: Vec<church_slavonic::Reading<'_>> = lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).collect();
    let total = readings.len();
    let kept: Vec<(&church_slavonic::Reading<'_>, Vec<(Cell, usize)>)> = readings
        .iter()
        .filter_map(|r| {
            let cells: Vec<(Cell, usize)> = r.cells.iter().filter(|(c, _)| keep(c)).copied().collect();
            (!cells.is_empty()).then_some((r, cells))
        })
        .collect();
    if kept.len() != 1 || total < 2 {
        return false;
    }
    let (r, cells) = &kept[0];
    if cells.iter().all(|(c, _)| *c == Cell::Word) {
        return false;
    }
    let Some(set) = CellSet::new(cells.iter().map(|(c, _)| *c).collect()) else { return false };
    let alt = cells.iter().find(|(c, _)| *c == set.first()).map(|(_, a)| *a).unwrap_or(0);
    let from: Option<CellSet> = r.cell_set();
    let mut leaf = Node::Lex {
        id: r.lexeme.id.clone(),
        cells: set,
        alt,
        notes: vec![("by".to_string(), rule.to_string()), ("from-lexemes".to_string(), total.to_string())],
    };
    if let (Node::Lex { notes, cells, .. }, Some(from)) = (&mut leaf, from)
        && from.len() > cells.len()
    {
        notes.push(("from".to_string(), from.name()));
    }
    let capped = crate::treebank::lift::decapitalized(&surface).is_some();
    let candidate = if capped { Node::Cap(Box::new(leaf)) } else { leaf };
    match crate::treebank::node::render(&candidate, &lexicon.recension) {
        Ok(rendered) if rendered == surface => {
            *node = candidate;
            stats.reduced(rule);
            true
        }
        _ => false,
    }
}

/// Apply the rules to one auto-lifted verse tree, in place.
pub fn disambiguate(tree: &mut Node, lexicon: &Lexicon) -> Stats {
    let mut stats = Stats::default();
    let Node::Group { children, .. } = tree else { return stats };
    let n = children.len();
    // 1. prep-gov: a preposition's frame narrows the nominal after it
    //    (and a second nominal when the first is an adjective: въ
    //    нача́лѣ, на всѧ́кой землѝ)
    for i in 0..n {
        let Some(word) = fn_word(&children[i]).map(str::to_string) else { continue };
        let cases = frame(lexicon, &word);
        if cases.is_empty() {
            continue;
        }
        let keep = |c: &Cell| c.case().is_some_and(|k| cases.contains(&k));
        // the first nominal after the preposition; a second one only
        // when the first is an adjective or a participle (въ нача́лѣ, на
        // всѧ́кой землѝ) — never after a pronoun: въ не́мже льстѝ нѣ́сть has
        // the genitive of negation, ѡ҆ не́мже а҆́зъ рѣ́хъ a new subject
        let mut j = i + 1;
        let mut targets = 0;
        while j < n && targets < 2 && !boundary(children, j) {
            let adjective_first = leaf(&children[j]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().all(|c| matches!(c, Cell::Adj(_) | Cell::Verb(VerbCell::Participle { .. })))));
            let nominal = leaf(&children[j]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().any(|c| is_nominal(&c)))) || amb_surface(&children[j]).is_some();
            if !nominal {
                break;
            }
            if !narrow(&mut children[j], lexicon, "prep-gov", keep, &mut stats) && targets == 0 {
                reduce(&mut children[j], lexicon, "prep-gov", keep, &mut stats);
            }
            targets += 1;
            if !adjective_first {
                break;
            }
            j += 1;
        }
    }
    // 2. np-agree: an adjective-like leaf beside a noun leaf, each kept
    //    to the cells that agree with some cell of the other
    for i in 0..n.saturating_sub(1) {
        if boundary(children, i + 1) {
            continue;
        }
        let pair = [(i, i + 1), (i + 1, i)];
        for (a, b) in pair {
            let (Some(Node::Lex { id: aid, cells: ac, .. }), Some(Node::Lex { id: bid, cells: bc, .. })) = (leaf(&children[a]), leaf(&children[b])) else { continue };
            if !ac.iter().all(|c| is_adjective_like(&c)) || !bc.iter().all(|c| matches!(c, Cell::Noun(_))) {
                continue;
            }
            // the relative pronoun after a noun opens a clause, it does not
            // modify the noun (на ѻ҆гнѝ ꙗ҆̀же на ѻ҆лтарѝ — 3.1, Leviticus 1:8)
            if aid == "иже.pron" {
                continue;
            }
            // a short present active participle before a noun is a converb
            // with that noun as its object, not its attribute (разверза́ѧ
            // ложесна̀ — 3.1, Luke 2:23); after the noun it may modify it
            // (мѣ́дь звенѧ́щи)
            let converb = a < b
                && ac.iter().all(|c| matches!(c, Cell::Verb(VerbCell::Participle { tense: church_slavonic::cell::PartTense::Present, voice: church_slavonic::grammar::Voice::Active, series: church_slavonic::grammar::Series::Short, .. })));
            if converb {
                continue;
            }
            let noun_gender: Option<Gender> = lexicon.get(bid).and_then(|l| l.gender);
            let agree = |x: &Cell, y: &Cell| -> bool {
                x.case() == y.case() && x.number() == y.number() && noun_gender.is_none_or(|g| x.gender().is_none_or(|xg| xg == g))
            };
            let adj_cells: Vec<Cell> = ac.iter().collect();
            let noun_cells: Vec<Cell> = bc.iter().collect();
            if !adj_cells.iter().any(|x| noun_cells.iter().any(|y| agree(x, y))) {
                continue;
            }
            let keep_adj = |c: &Cell| noun_cells.iter().any(|y| agree(c, y));
            let keep_noun = |c: &Cell| adj_cells.iter().any(|x| agree(x, c));
            narrow(&mut children[a], lexicon, "np-agree", keep_adj, &mut stats);
            narrow(&mut children[b], lexicon, "np-agree", keep_noun, &mut stats);
        }
    }
    // 3. subj-verb: a noun whose every reading is nominative beside a
    //    finite verb: the verb is third person and agrees in number
    for i in 0..n {
        let Some(Node::Lex { cells, .. }) = leaf(&children[i]) else { continue };
        if !cells.iter().all(|c| matches!(c, Cell::Noun(_)) && c.case() == Some(Case::Nominative)) {
            continue;
        }
        let numbers: Vec<Number> = cells.iter().filter_map(|c| c.number()).collect();
        for j in [i.checked_sub(1), i.checked_add(1)].into_iter().flatten() {
            if j >= n || boundary(children, j) {
                continue;
            }
            let is_verb = leaf(&children[j]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().all(|c| is_finite(&c)) && cells.iter().any(|c| c.person() == Some(Person::Third))));
            if !is_verb {
                continue;
            }
            let numbers = numbers.clone();
            narrow(&mut children[j], lexicon, "subj-verb", move |c| c.person() == Some(Person::Third) && c.number().is_some_and(|k| numbers.contains(&k)), &mut stats);
        }
    }
    // 4. voc-drop: the vocative goes from a set with other members unless
    //    an imperative or an interjection stands beside the token
    for i in 0..n {
        let Some(Node::Lex { cells, .. }) = leaf(&children[i]) else { continue };
        if !cells.iter().any(|c| c.case() == Some(Case::Vocative)) || cells.iter().all(|c| c.case() == Some(Case::Vocative)) {
            continue;
        }
        let beside = |j: Option<usize>| -> bool {
            let Some(j) = j else { return false };
            if j >= n {
                return false;
            }
            let imperative = leaf(&children[j]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().any(|c| is_imperative(&c))));
            let interjection = fn_word(&children[j]).is_some_and(|w| is_interjection(lexicon, w));
            imperative || interjection
        };
        if beside(i.checked_sub(1)) || beside(i.checked_add(1)) {
            continue;
        }
        narrow(&mut children[i], lexicon, "voc-drop", |c| c.case() != Some(Case::Vocative), &mut stats);
    }
    let _ = Prosody::Tonic;
    // 5. one-subject (3.2): the clause has one finite transitive verb; a
    //    noun that can only be nominative and agrees with it in number is
    //    the subject, so every other noun of the clause that reads
    //    nominative or accusative is not — it drops the nominative
    //    (ви́дѣ бг҃ъ свѣ́тъ). With a first- or second-person verb no noun
    //    is the subject at all (ви́дѣхъ свѣ́тъ). A clause is the span
    //    between punctuation, conjunctions and the relative pronoun; a
    //    noun after a preposition is that preposition's and is left
    //    alone; a copula or an intransitive verb (быти, ꙗвитисѧ) takes a
    //    predicate nominative and no rule fires.
    let mut start = 0;
    while start < n {
        let mut end = start;
        while end < n && !clause_boundary(children, end, lexicon) {
            end += 1;
        }
        one_subject(children, start, end, lexicon, &mut stats);
        start = end + 1;
    }

    stats
}

/// A child no clause reads across: punctuation, a conjunction, the
/// relative pronoun.
fn clause_boundary(children: &[Node], i: usize, lexicon: &Lexicon) -> bool {
    match children.get(i) {
        None | Some(Node::Punct(_)) => true,
        Some(node) => {
            if let Some(word) = fn_word(node) {
                let lexemes: Vec<&church_slavonic::Lexeme> = if crate::treebank::node::is_lexeme_id(word) { lexicon.get(word).into_iter().collect() } else { lexicon.find(word, Pos::Closed) };
                return lexemes.iter().any(|l| l.subcategory() == Some("conj"));
            }
            matches!(leaf(node), Some(Node::Lex { id, .. }) if id == "иже.pron")
        }
    }
}

/// Is the child at `i` the nominal a preposition governs (the child
/// before it is a preposition with a frame)?
fn in_prepositional_phrase(children: &[Node], i: usize, lexicon: &Lexicon) -> bool {
    i > 0 && fn_word(&children[i - 1]).is_some_and(|w| !frame(lexicon, w).is_empty())
}

/// A leaf whose every cell is a noun cell.
fn noun_leaf(children: &[Node], i: usize) -> bool {
    leaf(&children[i]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().all(|c| matches!(c, Cell::Noun(_)))))
}

/// Rule 5 over one clause span `[start, end)`.
fn one_subject(children: &mut [Node], start: usize, end: usize, lexicon: &Lexicon, stats: &mut Stats) {
    // exactly one finite verb
    let verbs: Vec<usize> = (start..end)
        .filter(|&i| leaf(&children[i]).is_some_and(|l| matches!(l, Node::Lex { cells, .. } if cells.iter().all(|c| is_finite(&c)))))
        .collect();
    let [v] = verbs[..] else { return };
    let (vid, persons, numbers): (String, Vec<Person>, Vec<Number>) = match leaf(&children[v]) {
        Some(Node::Lex { id, cells, .. }) => (id.clone(), cells.iter().filter_map(|c| c.person()).collect(), cells.iter().filter_map(|c| c.number()).collect()),
        _ => return,
    };
    let Some(verb) = lexicon.get(&vid) else { return };
    let transitive = verb.note.split("; ").any(|t| t == "tran");
    if !transitive || persons.is_empty() {
        return;
    }
    // an aorist reads second or third person alike (ви́дѣ): with a unique
    // nominative subject in the clause it is third; without one nothing
    // fires. A verb that cannot be third person has no noun subject.
    let third = persons.contains(&Person::Third);
    let not_third = !third;
    if third {
        let _ = not_third;
        // the one noun or pronoun that can only be nominative, in the
        // verb's number
        let subjects: Vec<usize> = (start..end)
            .filter(|&i| i != v && !in_prepositional_phrase(children, i, lexicon))
            .filter(|&i| {
                // a noun or pronoun whose every cell IN THE VERB'S NUMBER is
                // nominative (an abbreviation hides other numbers' cells:
                // бг҃ъ is nom.sg beside gen.pl and acc.pl, and a singular
                // verb reads the singular)
                leaf(&children[i]).is_some_and(|l| matches!(l, Node::Lex { cells, .. }
                    if cells.iter().all(|c| matches!(c, Cell::Noun(_) | Cell::Pron(_)))
                    && {
                        let same: Vec<Cell> = cells.iter().filter(|c| c.number().is_none_or(|k| numbers.contains(&k))).collect();
                        !same.is_empty() && same.iter().all(|c| c.case() == Some(Case::Nominative))
                    }))
            })
            .collect();
        let [subject] = subjects[..] else { return };
        for i in start..end {
            if i == v || i == subject || !noun_leaf(children, i) || in_prepositional_phrase(children, i, lexicon) {
                continue;
            }
            narrow(&mut children[i], lexicon, "one-subject", |c| c.case() != Some(Case::Nominative), stats);
        }
    } else {
        for i in start..end {
            if i == v || !noun_leaf(children, i) || in_prepositional_phrase(children, i, lexicon) {
                continue;
            }
            narrow(&mut children[i], lexicon, "one-subject", |c| c.case() != Some(Case::Nominative), stats);
        }
    }
}
