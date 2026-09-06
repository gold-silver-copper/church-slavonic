//! The tree model and the renderer.
//!
//! **The round-trip invariant**: for every verse that has a tree,
//! [`render`] equals the pinned print byte-for-byte (the verse text
//! trimmed of the JSON arrangement's cosmetic leading space — the source
//! has no interior double spaces, verified over all 34,470 verses, so
//! "join tokens with single spaces" IS the print's own spacing).
//!
//! Children are ORDERED: Church Slavonic word order is free and
//! meaningful, so the tree records order and never derives it. Rendering
//! is a left-to-right walk plus the punctuation glue rule. Features on
//! analyzed leaves are explicit — agreement is checked by the linter,
//! never inferred by the renderer.
//!
//! An analyzed leaf is a lexeme id and a cell of the 2.0 lexicon, plus
//! `:alt n` for a non-primary form of the cell (the index into
//! `Lexeme::forms`). The head names the part of speech and the features
//! spell the cell:
//!
//! - `(n рабъ.n :case gen :num pl)`;
//! - `(adj мꙋдрый.a :case nom :num sg :g m [:series short|long] [:deg comp])`;
//! - `(v рещи.v :t aor :p 3 :num sg)`, `(v … :form imp :p 2 :num pl)`,
//!   `(v … :form inf)`, `(lp быти.v :g m :num sg)`,
//!   `(part творити.v :t pres :voice act :series long :case nom :num sg :g m)`;
//! - `(pn той.pron :case nom :num sg :g m)`, the personal
//!   `(pn азъ.pron :p 1 :num sg :case dat [:clit yes])`, the reflexive
//!   `(pn себе.pron :case dat [:clit yes])`;
//! - `(f и.x)` a closed-class lexeme, or `(f и҆)` a word of the hand
//!   table in `closed.rs`.
//!
//! Since 4.1 the tree type and its renderer live in the library
//! (`church_slavonic::sentence`); the s-expression reader and writer stay
//! in the tools' `treebank::node`.

use crate::cell::{Cell, CellSet};
use std::fmt;

/// One node. Groups (`s`, `cl`, `np`, `pp`, …) carry an arbitrary head
/// atom — the linter knows some heads, the renderer treats them all as
/// ordered sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// `(w "гдⷭ҇ь")` — a witnessed surface form, rendered as-is. May
    /// carry belief annotations (`:lemma`, `:amb`, …) that the renderer
    /// IGNORES; they exist for the linter and the lifting pipeline.
    W { surface: String, notes: Vec<(String, String)> },
    /// `(p ",")` — punctuation; glues to the neighbouring token (left by
    /// default, right for opening brackets/quotes).
    Punct(String),
    /// `(f и҆)` — a function word of [`crate::sentence::closed::TABLE`],
    /// or `(f и.x)` a closed-class lexeme of the lexicon.
    Fn(String),
    /// An analyzed leaf: a lexeme id, its cell — or the set of cells the
    /// paradigm does not tell apart in this token (`:case nom|acc|voc`;
    /// a set that is not a product of its features is written
    /// `:cell nom.pl|gen.sg|acc.pl`) — and which of the first cell's forms
    /// (`0` the primary). Rendering goes through the first cell; every
    /// member prints the same token.
    /// `notes` are the disambiguator's record (`:by prep-gov :from
    /// nom|acc|voc.sg`, `:from-lexemes 3`): what narrowed the leaf and
    /// from what; the renderer ignores them.
    Lex { id: String, cells: CellSet, alt: usize, notes: Vec<(String, String)> },
    /// `(abbr "гдⷭ҇" X)` — render the child in full, then abbreviate it
    /// under the matching row of `lexicon/titlo.tsv` (the titlo layer).
    /// A titlo-written token: the abbreviated prefix as printed, the row's
    /// full-prefix skeleton where the prefix has several rows for one lemma
    /// (гл҃ъ beside гл҃го́лъ, 3.3; `None` renders through the first row that
    /// names the child's lexeme), and the expansion.
    Abbr { prefix: String, full: Option<String>, child: Box<Node> },
    /// `(cap X)` — uppercase the first letter of the child's rendering
    /// (sentence-initial capitals; the tree stays lemma-true).
    Cap(Box<Node>),
    /// `(pw host (f же.x) …)` — a phonological word written solid (и҆̀хже:
    /// ихъ + же), `(pwa host (f же.x) …)` one written apart (Землѧ́ же):
    /// a host with the enclitics that lean on it, accented as one unit —
    /// the host's final stressed vowel takes the oxia because the unit
    /// goes on. The host is an analyzed leaf or a closed lexeme, each
    /// enclitic a closed lexeme with `pros=encl`.
    Pw { host: Box<Node>, enclitics: Vec<Node>, apart: bool },
    /// `(np …)`, `(cl …)`, `(s …)` — an ordered group.
    Group { head: String, children: Vec<Node> },
}

pub use crate::grammar::{Case, Gender, Number, Person};

/// A tree-shape error (bad head, missing feature, unknown feature value),
/// with the offending form printed back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeError(pub String);

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tree error: {}", self.0)
    }
}
impl std::error::Error for TreeError {}

/// A [`TreeError`] from a message (the s-expression reader in the tools
/// uses it too).
pub fn err<T>(message: impl Into<String>) -> Result<T, TreeError> {
    Err(TreeError(message.into()))
}

// ---------------------------------------------------------------------------
// Reading and writing
// ---------------------------------------------------------------------------

/// Punctuation that glues to the FOLLOWING token (opening brackets and
/// quotes); everything else glues to the preceding one.
fn glues_right(p: &str) -> bool {
    matches!(p.chars().next(), Some('(' | '[' | '«' | '„' | '“'))
}

/// Render a tree: a left-to-right walk emitting one token per leaf,
/// single spaces between tokens, punctuation glued by the glue rule.
/// Analyzed leaves inflect through the lexicon of the given recension.
pub fn render(node: &Node, recension: &crate::Recension) -> Result<String, TreeError> {
    let mut out = String::new();
    let mut glue_next = false;
    walk(node, recension, &mut out, &mut glue_next)?;
    Ok(out)
}

fn emit(token: &str, glue_left: bool, out: &mut String, glue_next: &mut bool) {
    if !out.is_empty() && !glue_left && !*glue_next {
        out.push(' ');
    }
    out.push_str(token);
    *glue_next = false;
}

/// The form of one analyzed leaf (its layers, before the print).
pub fn leaf_form(id: &str, cell: Cell, alt: usize, recension: crate::Recension) -> Result<crate::Form, TreeError> {
    let lexicon = crate::Lexicon::of(recension);
    let Some(lexeme) = lexicon.get(id) else {
        return err(format!("{id}: no such lexeme in the lexicon"));
    };
    let mut forms = lexeme.forms(cell);
    if alt >= forms.len() {
        return err(format!("{id}: no form {alt} in cell {}", cell.name()));
    }
    Ok(forms.swap_remove(alt))
}

/// The print of one analyzed leaf.
pub fn leaf_print(id: &str, cell: Cell, alt: usize, recension: crate::Recension) -> Result<String, TreeError> {
    Ok(leaf_form(id, cell, alt, recension)?.print(recension))
}

/// The form of a phonological word's host: an analyzed leaf's, or a
/// closed lexeme's lemma.
fn host_form(host: &Node, recension: crate::Recension) -> Result<crate::Form, TreeError> {
    match host {
        Node::Lex { id, cells, alt, .. } => leaf_form(id, cells.first(), *alt, recension),
        Node::Fn(id) if is_lexeme_id(id) => match crate::Lexicon::of(recension).get(id) {
            Some(l) => Ok(crate::Form::from_print(&l.lemma)),
            None => err(format!("{id}: no such lexeme in the lexicon")),
        },
        _ => err("(pw …): the host is an analyzed leaf or a closed lexeme"),
    }
}

/// A print with its stress marks removed (the breathing stays).
fn unaccented(printed: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    printed.nfd().filter(|c| !matches!(*c, '\u{300}' | '\u{301}' | '\u{311}')).collect::<String>().nfc().collect()
}

/// The print of a phonological word: the host's form with its enclitics,
/// accented as one unit — written solid (и҆̀хже), or apart with the host
/// keeping the unit's oxia (Землѧ́ же).
pub fn unit_print(host: &Node, enclitics: &[Node], apart: bool, recension: crate::Recension) -> Result<String, TreeError> {
    let form = host_form(host, recension)?;
    let lexicon = crate::Lexicon::of(recension);
    let mut letters: Vec<String> = Vec::new();
    for e in enclitics {
        match e {
            Node::Fn(eid) => {
                let Some(lexeme) = lexicon.get(eid) else {
                    return err(format!("{eid}: no such lexeme in the lexicon"));
                };
                if lexeme.prosody() != crate::grammar::Prosody::Enclitic {
                    return err(format!("{eid} is not an enclitic (pros=encl)"));
                }
                letters.push(lexeme.lemma.clone());
            }
            // a pronoun's clitic cell leans on its host unaccented (3.3:
            // прельсти́ мѧ, да́ждь ми — the print's мѧ̀, мѝ lose the varia)
            Node::Lex { id, cells, alt, .. } => {
                if !matches!(cells.first(), Cell::Pron(pc) if pc.clitic) {
                    return err(format!("(pw …): {id} {} is not a clitic cell", cells.name()));
                }
                letters.push(unaccented(&leaf_print(id, cells.first(), *alt, recension)?));
            }
            _ => return err("(pw …): an enclitic is (f <id>) or a clitic pronoun leaf"),
        }
    }
    if apart {
        let mut out = form.print_hosting(recension);
        for l in &letters {
            out.push(' ');
            out.push_str(l);
        }
        return Ok(out);
    }
    let refs: Vec<&str> = letters.iter().map(String::as_str).collect();
    Ok(form.print_unit(recension, &refs))
}

fn walk(node: &Node, recension: &crate::Recension, out: &mut String, glue_next: &mut bool) -> Result<(), TreeError> {
    match node {
        Node::W { surface, .. } => emit(surface, false, out, glue_next),
        Node::Punct(p) => {
            if glues_right(p) {
                emit(p, false, out, glue_next);
                *glue_next = true;
            } else {
                emit(p, true, out, glue_next);
            }
        }
        Node::Fn(word) => {
            if is_lexeme_id(word) {
                let print = leaf_print(word, Cell::Word, 0, *recension)?;
                emit(&print, false, out, glue_next);
            } else {
                if !is_function_word(word, *recension) {
                    return err(format!("(f {word}) is neither a lexeme id, a closed-class lexeme's print, nor in the hand table"));
                }
                emit(word, false, out, glue_next);
            }
        }
        Node::Lex { id, cells, alt, .. } => {
            let print = leaf_print(id, cells.first(), *alt, *recension)?;
            emit(&print, false, out, glue_next);
        }
        Node::Abbr { prefix, full, child } => {
            let mut inner = String::new();
            let mut inner_glue = false;
            walk(child, recension, &mut inner, &mut inner_glue)?;
            // the rows of the prefix (and of the skeleton when the node
            // names one), those naming the child's lexeme first: гдⷭ҇ has
            // a strip row for госпо́дь and a keep row for господи́нъ
            let lemma_key = match crate::sentence::rules::leaf(child) {
                Some(Node::Lex { id, .. }) => crate::Lexicon::synodal().get(id).map(|l| crate::orthography::comparison_key(&l.lemma)),
                _ => None,
            };
            let mut rows: Vec<&crate::titlo::Row> = crate::titlo::rows()
                .iter()
                .filter(|row| row.abbr == prefix && full.as_ref().is_none_or(|f| row.full == *f))
                .collect();
            rows.sort_by_key(|row| lemma_key.as_ref().is_some_and(|k| crate::orthography::comparison_key(row.lemma) != *k));
            let abbreviated = rows.into_iter().find_map(|row| crate::titlo::abbreviate(&inner, row));
            match abbreviated {
                Some(form) => emit(&form, false, out, glue_next),
                None => return err(format!("(abbr \"{prefix}\" …): no titlo row abbreviates «{inner}»")),
            }
        }
        Node::Pw { host, enclitics, apart } => {
            let print = unit_print(host, enclitics, *apart, *recension)?;
            emit(&print, false, out, glue_next);
        }
        Node::Cap(child) => {
            let mut inner = String::new();
            let mut inner_glue = false;
            walk(child, recension, &mut inner, &mut inner_glue)?;
            let mut chars = inner.chars();
            let capped: String = match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => return err("(cap …) rendered nothing"),
            };
            emit(&capped, false, out, glue_next);
        }
        Node::Group { children, .. } => {
            for child in children {
                walk(child, recension, out, glue_next)?;
            }
        }
    }
    Ok(())
}

/// Is a word a function word: the print of a closed-class lexeme, or an
/// entry of the hand table in `closed.rs`?
pub fn is_function_word(word: &str, recension: crate::Recension) -> bool {
    crate::sentence::closed::is_closed(word)
        || crate::Lexicon::of(recension).analyze(word).iter().any(|a| a.exact && a.cell == Cell::Word)
}

/// Is an atom a 2.0 lexeme id (`землѧ.n`, `сꙑнъ.n.2`)?
pub fn is_lexeme_id(lemma: &str) -> bool {
    let mut parts = lemma.split('.');
    let _stem = parts.next();
    match (parts.next(), parts.next(), parts.next()) {
        (Some(tag), None, None) => crate::Pos::parse(tag).is_some(),
        (Some(tag), Some(n), None) => crate::Pos::parse(tag).is_some() && n.chars().all(|c| c.is_ascii_digit()),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tokenizing and the verbatim wrap
// ---------------------------------------------------------------------------

/// Tokenize a pinned verse: trim the JSON arrangement's cosmetic outer
/// whitespace, split on spaces. Punctuation and apparatus (`꙾…꙾`,
/// `[26]`) stay INSIDE their tokens — the round-trip target is the verse
/// as printed, and splitting is the lifting pipeline's business.
pub fn tokenize(verse: &str) -> Vec<&str> {
    verse.split_whitespace().collect()
}

/// Wrap a verse verbatim: every token a `(w …)` leaf under `(s …)`. The
/// starting point of every tree — round-trips by construction.
pub fn verbatim_tree(verse: &str) -> Node {
    Node::Group {
        head: "s".to_string(),
        children: tokenize(verse).into_iter().map(|t| Node::W { surface: t.to_string(), notes: Vec::new() }).collect(),
    }
}

