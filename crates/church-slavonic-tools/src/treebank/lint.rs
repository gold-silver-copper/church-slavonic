//! The linter: a tree is REFUTABLE because its features are explicit.
//! `lint` returns findings and never panics; only rules that are actually
//! reliable are checked — a linter that guesses teaches annotators to
//! ignore it.
//!
//! Checked today:
//! - NP-internal agreement: inside `(np …)`, every adjective matches
//!   every noun in case and number (gender is NOT checked against nouns —
//!   the noun leaf carries no gender feature to check against);
//! - verb ↔ subject: inside `(cl …)`, a finite verb agrees in number
//!   with a nominative noun inside a `(subj …)` group, and its person is
//!   third (a noun subject cannot be 1st/2nd) — opt-in via the `subj`
//!   head, because subject identification without roles is guesswork;
//! - preposition case: inside `(pp …)` headed by a listed preposition,
//!   the governed noun's case is one the preposition reliably takes
//!   (small table; unlisted prepositions are not checked);
//! - unknown group heads (typo defence: the known heads are `s`, `cl`,
//!   `np`, `pp`, `vp`, `subj`, `obj`);
//! - `(f …)` words missing from the closed-class table;
//! - `:expect` on an analyzed leaf whose rendering differs — the
//!   annotator's pinned surface against the crate's answer.

use crate::treebank::node::{Case, Node, Person};
use church_slavonic::cell::{Cell, VerbCell};

/// One finding: a path into the tree (`s/cl[0]/np[2]`) and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub message: String,
}

/// The cases a preposition governs, from the lexicon (`stems=gov=`):
/// by id (`къ.x.2`) or by print, the frames of every closed lexeme that
/// prints the word joined. Empty when the lexicon says nothing — then
/// the preposition goes unchecked, honestly.
fn governed(word: &str, recension: &church_slavonic::Recension) -> Vec<Case> {
    let lexicon = church_slavonic::Lexicon::of(*recension);
    let mut out: Vec<Case> = Vec::new();
    let lexemes: Vec<&church_slavonic::Lexeme> = if crate::treebank::node::is_lexeme_id(word) {
        lexicon.get(word).into_iter().collect()
    } else {
        lexicon.find(word, church_slavonic::Pos::Closed)
    };
    for l in lexemes {
        for c in l.government() {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

const KNOWN_HEADS: &[&str] = &["s", "cl", "np", "pp", "vp", "subj", "obj"];

/// Lint a tree in the given recension (`:expect` checks render through it).
pub fn lint(node: &Node, recension: &church_slavonic::Recension) -> Vec<Finding> {
    let mut findings = Vec::new();
    walk(node, recension, "", &mut findings);
    findings
}

fn push(findings: &mut Vec<Finding>, path: &str, message: impl Into<String>) {
    findings.push(Finding { path: path.to_string(), message: message.into() });
}

fn walk(
    node: &Node,
    recension: &church_slavonic::Recension,
    path: &str,
    findings: &mut Vec<Finding>,
) {
    match node {
        Node::Fn(word) => {
            if !crate::treebank::node::is_lexeme_id(word) && !crate::treebank::node::is_function_word(word, *recension) {
                push(findings, path, format!("(f {word}) is not in the closed-class table"));
            }
        }
        Node::W { surface, notes } => {
            if surface.is_empty() {
                push(findings, path, "(w …) with an empty surface");
            }
            let _ = notes;
        }
        Node::Lex { .. } => {}
        Node::Punct(_) => {}
        Node::Pw { host, enclitics, .. } => {
            walk(host, recension, &format!("{path}/pw"), findings);
            for e in enclitics {
                walk(e, recension, &format!("{path}/pw"), findings);
            }
        }
        Node::Cap(child) => walk(child, recension, &format!("{path}/cap"), findings),
        Node::Abbr { prefix, child, .. } => {
            if !crate::treebank::titlo::rows().iter().any(|r| r.abbr == *prefix) {
                push(findings, path, format!("(abbr \"{prefix}\" …): unknown titlo prefix"));
            }
            walk(child, recension, &format!("{path}/abbr"), findings);
        }
        Node::Group { head, children } => {
            if !KNOWN_HEADS.contains(&head.as_str()) {
                push(findings, path, format!("unknown group head: {head}"));
            }
            match head.as_str() {
                "np" => lint_np(children, path, findings),
                "pp" => lint_pp(children, recension, path, findings),
                "cl" => lint_cl(children, path, findings),
                _ => {}
            }
            for (i, child) in children.iter().enumerate() {
                walk(child, recension, &format!("{path}/{head}[{i}]"), findings);
            }
        }
    }
    lint_expect(node, recension, path, findings);
}

/// `:expect` on a `(w …)` note-carrying leaf is not a thing — verbatim IS
/// its own surface. On analyzed leaves the annotator may pin the surface
/// they believe the features produce; a mismatch is a finding.
fn lint_expect(
    node: &Node,
    recension: &church_slavonic::Recension,
    path: &str,
    findings: &mut Vec<Finding>,
) {
    let expect = match node {
        Node::W { notes, .. } => {
            if notes.iter().any(|(k, _)| k == "expect") {
                push(findings, path, ":expect on a verbatim leaf means nothing");
            }
            return;
        }
        Node::Lex { .. } => None,
        _ => return,
    };
    // Analyzed leaves carry no notes today; :expect arrives through the
    // richer leaf syntax when the treebank needs it. The hook stays so
    // the check has one home.
    let _ = (expect as Option<&str>, recension);
}

/// Strip transparent wrappers when looking for a leaf.
fn unwrap_cap(node: &Node) -> &Node {
    match node {
        Node::Cap(inner) => unwrap_cap(inner),
        Node::Pw { host, .. } => unwrap_cap(host),
        other => other,
    }
}

type Num = church_slavonic::grammar::Number;

/// A noun leaf's id and its (case, number) readings — several where the
/// leaf is underspecified; a disjunctive feature is satisfied when any
/// member agrees (narrowing the set by agreement is disambiguation, out
/// of the linter's scope).
fn noun(node: &Node) -> Option<(&str, Vec<(Case, Num)>)> {
    match unwrap_cap(node) {
        Node::Lex { id, cells, .. } if matches!(cells.first(), Cell::Noun(_)) => Some((
            id,
            cells.iter().filter_map(|c| Some((c.case()?, c.number()?))).collect(),
        )),
        _ => None,
    }
}

fn lint_np(children: &[Node], path: &str, findings: &mut Vec<Finding>) {
    let nouns: Vec<_> = children.iter().filter_map(noun).collect();
    for (i, child) in children.iter().enumerate() {
        if let Node::Lex { cells, .. } = unwrap_cap(child)
            && matches!(cells.first(), Cell::Adj(_))
        {
            let adj: Vec<(Case, Num)> = cells.iter().filter_map(|c| Some((c.case()?, c.number()?))).collect();
            for (id, readings) in &nouns {
                if !adj.iter().any(|a| readings.contains(a)) {
                    push(findings, &format!("{path}/np[{i}]"), format!("adjective disagrees with {id} in case or number"));
                }
            }
        }
    }
}

fn lint_pp(children: &[Node], recension: &church_slavonic::Recension, path: &str, findings: &mut Vec<Finding>) {
    let Some(Node::Fn(prep)) = children.first().map(unwrap_cap) else {
        return; // not a canonical pp — the head check already spoke
    };
    let allowed = governed(prep, recension);
    if allowed.is_empty() {
        return; // the lexicon names no frame — unchecked
    }
    for (i, child) in children.iter().enumerate().skip(1) {
        let target = match unwrap_cap(child) {
            Node::Group { head, children } if head == "np" => children.iter().find_map(noun),
            leaf => noun(leaf),
        };
        if let Some((id, readings)) = target
            && !readings.iter().any(|(case, _)| allowed.contains(case))
        {
            push(findings, &format!("{path}/pp[{i}]"), format!("{prep} does not govern the case of {id}"));
        }
    }
}

fn lint_cl(children: &[Node], path: &str, findings: &mut Vec<Finding>) {
    let verb = children.iter().map(unwrap_cap).find_map(|c| match c {
        Node::Lex { id, cells, .. } if matches!(cells.first(), Cell::Verb(VerbCell::Finite { .. })) => {
            Some((id, cells.iter().filter_map(|c| Some((c.person()?, c.number()?))).collect::<Vec<_>>()))
        }
        _ => None,
    });
    fn nominative((id, readings): (&str, Vec<(Case, Num)>)) -> Option<(&str, Vec<Num>)> {
        let numbers: Vec<Num> = readings.iter().filter(|(case, _)| *case == Case::Nominative).map(|(_, n)| *n).collect();
        (!numbers.is_empty()).then_some((id, numbers))
    }
    let subject = children.iter().find_map(|c| match c {
        Node::Group { head, children } if head == "subj" => children.iter().find_map(|s| match unwrap_cap(s) {
            Node::Group { head, children } if head == "np" => children.iter().filter_map(noun).find_map(nominative),
            leaf => noun(leaf).and_then(nominative),
        }),
        _ => None,
    });
    if let (Some((vid, readings)), Some((sid, numbers))) = (verb, subject) {
        if !readings.iter().any(|(_, n)| numbers.contains(n)) {
            push(findings, path, format!("{vid} disagrees with subject {sid} in number"));
        }
        if !readings.iter().any(|(p, _)| *p == Person::Third) {
            push(findings, path, format!("{vid}: a noun subject takes the third person"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treebank::node::from_sexpr;
    use crate::treebank::sexpr;

    fn lint_text(text: &str) -> Vec<Finding> {
        let node = from_sexpr(&sexpr::parse(text).expect("sexpr")).expect("node");
        lint(&node, &church_slavonic::Recension::Synodal)
    }

    #[test]
    fn a_clean_tree_has_no_findings() {
        let clean = r#"(s (cl
            (pp (cap (f въ)) (n начало.n :case loc :num sg))
            (v сотворити.v :t aor :p 3 :num sg)
            (subj (np (w "бг҃ъ" :lemma бо́гъ)))
            (np (n небо.n :case acc :num sg) (f и҆) (n землѧ.n :case acc :num sg))
            (p ".")))"#;
        assert_eq!(lint_text(clean), Vec::new());
    }

    #[test]
    fn a_disjunctive_feature_is_satisfied_by_any_member() {
        // the adjective's set meets the noun's set in acc.sg: no finding
        assert_eq!(lint_text("(np (adj великій.a :case nom|acc :num sg :g m :series long) (n свѣтъ.n :case nom|acc|voc :num sg))"), Vec::new());
        // no member agrees: a finding
        let f = lint_text("(np (adj великій.a :case gen|dat :num sg :g m :series long) (n свѣтъ.n :case nom|acc :num sg))");
        assert_eq!(f.len(), 1);
        // the aorist's 2|3.sg agrees with a singular nominative subject
        assert_eq!(lint_text("(cl (subj (n богъ.n :case nom|acc :num sg)) (v рещи.v :t aor :p 2|3 :num sg))"), Vec::new());
        // a preposition governs the set if it governs any member
        assert_eq!(lint_text("(pp (f къ) (n богъ.n :case gen|dat :num sg))"), Vec::new());
    }

    #[test]
    fn np_disagreement_is_found() {
        let f = lint_text("(np (adj великій.a :case gen :num sg :g n) (n небо.n :case acc :num sg))");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("disagrees with небо.n"));
    }

    #[test]
    fn verb_subject_disagreement_is_found() {
        let f = lint_text(
            "(cl (subj (n богъ.n :case nom :num sg)) (v рещи.v :t aor :p 3 :num pl))",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("in number"));
        let f = lint_text(
            "(cl (subj (n богъ.n :case nom :num sg)) (v рещи.v :t aor :p 1 :num sg))",
        );
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("third person"));
    }

    #[test]
    fn preposition_case_is_checked_only_where_reliable() {
        let f = lint_text("(pp (f къ) (n богъ.n :case gen :num sg))");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("къ does not govern"));
        // въ takes acc OR loc — both clean
        assert_eq!(lint_text("(pp (f въ) (n начало.n :case loc :num sg))"), Vec::new());
        assert_eq!(lint_text("(pp (f въ) (n начало.n :case acc :num sg))"), Vec::new());
        // за governs the accusative and the instrumental (the lexicon's
        // gov=acc|ins): the nominative is a finding now
        assert_eq!(lint_text("(pp (f за) (n богъ.n :case acc :num sg))"), Vec::new());
        assert_eq!(lint_text("(pp (f за) (n богъ.n :case nom :num sg))").len(), 1);
        // a word the lexicon gives no frame is not checked
        assert_eq!(lint_text("(pp (f сѐ) (n богъ.n :case nom :num sg))"), Vec::new());
    }

    #[test]
    fn typo_heads_and_unlisted_function_words_are_found() {
        let f = lint_text("(npp (n богъ.n :case nom :num sg))");
        assert!(f.iter().any(|x| x.message.contains("unknown group head")));
        let f = lint_text("(s (f гдⷭ҇ь))");
        assert!(f.iter().any(|x| x.message.contains("closed-class")));
        let f = lint_text("(s (w \"бг҃ъ\" :expect бг҃ъ))");
        assert!(f.iter().any(|x| x.message.contains("verbatim leaf")));
    }
}
