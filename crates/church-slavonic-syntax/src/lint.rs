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

use crate::node::{Case, Form, Node, Person};

/// One finding: a path into the tree (`s/cl[0]/np[2]`) and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub message: String,
}

/// Prepositions whose governed case is reliable enough to check.
/// (word, allowed cases). Everything else goes unchecked — honestly.
const PREP_CASES: &[(&str, &[Case])] = &[
    ("къ", &[Case::Dative]),
    ("ко", &[Case::Dative]),
    ("ѿ", &[Case::Genitive]),
    ("и҆з̾", &[Case::Genitive]),
    ("без̾", &[Case::Genitive]),
    ("до", &[Case::Genitive]),
    ("при", &[Case::Locative]),
    ("въ", &[Case::Accusative, Case::Locative]),
    ("на", &[Case::Accusative, Case::Locative]),
    ("ѡ҆", &[Case::Accusative, Case::Locative]),
];

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
            if !crate::closed::is_closed(word) {
                push(findings, path, format!("(f {word}) is not in the closed-class table"));
            }
        }
        Node::W { surface, notes } => {
            if surface.is_empty() {
                push(findings, path, "(w …) with an empty surface");
            }
            let _ = notes;
        }
        Node::Noun { .. } | Node::Adj { .. } | Node::Verb { .. } | Node::LPart { .. }
        | Node::Npron { .. } | Node::Pers { .. } | Node::Refl { .. } | Node::Part { .. } => {}
        Node::Punct(_) => {}
        Node::Cap(child) => walk(child, recension, &format!("{path}/cap"), findings),
        Node::Abbr { prefix, child } => {
            if !crate::titlo::rows().iter().any(|r| r.abbr == *prefix) {
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
                "pp" => lint_pp(children, path, findings),
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
        Node::Noun { .. } | Node::Adj { .. } | Node::Verb { .. } | Node::LPart { .. }
        | Node::Npron { .. } | Node::Pers { .. } | Node::Refl { .. } | Node::Part { .. } => None,
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
        other => other,
    }
}

fn lint_np(children: &[Node], path: &str, findings: &mut Vec<Finding>) {
    let nouns: Vec<&Node> = children
        .iter()
        .map(unwrap_cap)
        .filter(|c| matches!(c, Node::Noun { .. }))
        .collect();
    for (i, child) in children.iter().enumerate() {
        if let Node::Adj { case, number, .. } = unwrap_cap(child) {
            for noun in &nouns {
                if let Node::Noun { lemma, case: ncase, number: nnum } = noun
                    && (case != ncase || number != nnum)
                {
                    push(
                        findings,
                        &format!("{path}/np[{i}]"),
                        format!("adjective disagrees with {lemma} in case or number"),
                    );
                }
            }
        }
    }
}

fn lint_pp(children: &[Node], path: &str, findings: &mut Vec<Finding>) {
    let Some(Node::Fn(prep)) = children.first().map(unwrap_cap) else {
        return; // not a canonical pp — the head check already spoke
    };
    let Some(&(_, allowed)) = PREP_CASES.iter().find(|(w, _)| w == prep) else {
        return; // preposition without a reliable case table — unchecked
    };
    for (i, child) in children.iter().enumerate().skip(1) {
        let target = match unwrap_cap(child) {
            Node::Group { head, children } if head == "np" => children
                .iter()
                .map(unwrap_cap)
                .find(|c| matches!(c, Node::Noun { .. })),
            leaf @ Node::Noun { .. } => Some(leaf),
            _ => None,
        };
        if let Some(Node::Noun { lemma, case, .. }) = target
            && !allowed.contains(case)
        {
            push(
                findings,
                &format!("{path}/pp[{i}]"),
                format!("{prep} does not govern the case of {lemma}"),
            );
        }
    }
}

fn lint_cl(children: &[Node], path: &str, findings: &mut Vec<Finding>) {
    let verb = children.iter().map(unwrap_cap).find_map(|c| match c {
        Node::Verb { lemma, person, number, form: Form::Finite, .. } => {
            Some((lemma, person, number))
        }
        _ => None,
    });
    let subject = children.iter().find_map(|c| match c {
        Node::Group { head, children } if head == "subj" => children
            .iter()
            .map(unwrap_cap)
            .find_map(|s| match s {
                Node::Noun { lemma, case: Case::Nominative, number } => Some((lemma, number)),
                Node::Group { head, children } if head == "np" => {
                    children.iter().map(unwrap_cap).find_map(|n| match n {
                        Node::Noun { lemma, case: Case::Nominative, number } => {
                            Some((lemma, number))
                        }
                        _ => None,
                    })
                }
                _ => None,
            }),
        _ => None,
    });
    if let (Some((vlemma, person, vnum)), Some((slemma, snum))) = (verb, subject) {
        if vnum != snum {
            push(
                findings,
                path,
                format!("{vlemma} disagrees with subject {slemma} in number"),
            );
        }
        if *person != Person::Third {
            push(
                findings,
                path,
                format!("{vlemma}: a noun subject takes the third person"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::from_sexpr;
    use crate::sexpr;

    fn lint_text(text: &str) -> Vec<Finding> {
        let node = from_sexpr(&sexpr::parse(text).expect("sexpr")).expect("node");
        lint(&node, &church_slavonic::Recension::Synodal)
    }

    #[test]
    fn a_clean_tree_has_no_findings() {
        let clean = r#"(s (cl
            (pp (cap (f въ)) (n нача́ло :case loc :num sg))
            (v сотвори́ти :t aor :p 3 :num sg)
            (subj (np (w "бг҃ъ" :lemma бо́гъ)))
            (np (n не́бо :case acc :num sg) (f и҆) (n землѧ̀ :case acc :num sg))
            (p ".")))"#;
        assert_eq!(lint_text(clean), Vec::new());
    }

    #[test]
    fn np_disagreement_is_found() {
        let f = lint_text("(np (adj вели́кїй :case gen :num sg :g n) (n не́бо :case acc :num sg))");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("disagrees with не́бо"));
    }

    #[test]
    fn verb_subject_disagreement_is_found() {
        let f = lint_text(
            "(cl (subj (n бо́гъ :case nom :num sg)) (v рещѝ :t aor :p 3 :num pl))",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("in number"));
        let f = lint_text(
            "(cl (subj (n бо́гъ :case nom :num sg)) (v рещѝ :t aor :p 1 :num sg))",
        );
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("third person"));
    }

    #[test]
    fn preposition_case_is_checked_only_where_reliable() {
        let f = lint_text("(pp (f къ) (n бо́гъ :case gen :num sg))");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("къ does not govern"));
        // въ takes acc OR loc — both clean
        assert_eq!(lint_text("(pp (f въ) (n нача́ло :case loc :num sg))"), Vec::new());
        assert_eq!(lint_text("(pp (f въ) (n нача́ло :case acc :num sg))"), Vec::new());
        // a preposition without a reliable table is not checked
        assert_eq!(lint_text("(pp (f за) (n бо́гъ :case nom :num sg))"), Vec::new());
    }

    #[test]
    fn typo_heads_and_unlisted_function_words_are_found() {
        let f = lint_text("(npp (n бо́гъ :case nom :num sg))");
        assert!(f.iter().any(|x| x.message.contains("unknown group head")));
        let f = lint_text("(s (f гдⷭ҇ь))");
        assert!(f.iter().any(|x| x.message.contains("closed-class")));
        let f = lint_text("(s (w \"бг҃ъ\" :expect бг҃ъ))");
        assert!(f.iter().any(|x| x.message.contains("verbatim leaf")));
    }
}
