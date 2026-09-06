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

use crate::treebank::sexpr::Value;
use church_slavonic::cell::CellSet;
use church_slavonic::cell::{
    AdjCell, AdvCell, Cell, FiniteTense, NounCell, PartTense, Pos, PronCell, VerbCell, case_name, degree_name, finite_name,
    gender_name, number_name, parse_case, parse_degree, parse_finite, parse_gender, parse_number, parse_person,
    parse_series, person_name,
};
use church_slavonic::grammar::{Degree, Series, Voice};
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
    /// `(f и҆)` — a function word of [`crate::treebank::closed::TABLE`],
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

pub use church_slavonic::grammar::{Case, Gender, Number, Person};

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

fn err<T>(message: impl Into<String>) -> Result<T, TreeError> {
    Err(TreeError(message.into()))
}

// ---------------------------------------------------------------------------
// Reading and writing
// ---------------------------------------------------------------------------

/// Read `:key value` pairs.
fn features(items: &[Value]) -> Result<Features, TreeError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < items.len() {
        let Value::Key(k) = &items[i] else {
            return err(format!("expected :feature, got {}", crate::treebank::sexpr::print(&items[i])));
        };
        let Some(Value::Atom(v)) = items.get(i + 1) else {
            return err(format!(":{k} needs an atom value"));
        };
        out.push((k.clone(), v.clone()));
        i += 2;
    }
    Ok(out)
}

fn take<'a>(fs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn require<'a>(fs: &'a [(String, String)], key: &str, head: &str) -> Result<&'a str, TreeError> {
    take(fs, key).ok_or_else(|| TreeError(format!("({head} …) requires :{key}")))
}

fn read<T>(value: &str, what: &str, parse: impl Fn(&str) -> Option<T>) -> Result<T, TreeError> {
    parse(value).ok_or_else(|| TreeError(format!("unknown {what}: {value}")))
}

/// A feature value with alternatives (`nom|acc|voc`): every member read.
fn read_set<T>(value: &str, what: &str, parse: impl Fn(&str) -> Option<T>) -> Result<Vec<T>, TreeError> {
    value.split('|').map(|v| read(v, what, &parse)).collect()
}

/// An optional feature with alternatives: `None` when absent.
fn read_opt_set<T>(fs: &[(String, String)], key: &str, what: &str, parse: impl Fn(&str) -> Option<T>) -> Result<Vec<Option<T>>, TreeError> {
    match take(fs, key) {
        None => Ok(vec![None]),
        Some(v) => Ok(read_set(v, what, parse)?.into_iter().map(Some).collect()),
    }
}

/// The set a leaf's product of feature values names; `:cell <name>` on
/// the leaf names the set outright (the non-product case).
fn cell_set(head: &str, fs: &[(String, String)], pos: Pos, product: Vec<Cell>) -> Result<CellSet, TreeError> {
    if let Some(text) = take(fs, "cell") {
        return CellSet::parse(pos, text).ok_or_else(|| TreeError(format!("({head} …): :cell {text} is not a set of {} cells", pos.tag())));
    }
    CellSet::new(product).ok_or_else(|| TreeError(format!("({head} …) names no cell")))
}

/// The disambiguator's notes a leaf may carry (`:by`, `:from`,
/// `:from-lexemes`), kept in order.
fn with_notes(node: Node, fs: &[(String, String)]) -> Node {
    match node {
        Node::Lex { id, cells, alt, .. } => Node::Lex {
            id,
            cells,
            alt,
            notes: fs.iter().filter(|(k, _)| matches!(k.as_str(), "by" | "from" | "from-lexemes" | "prob")).cloned().collect(),
        },
        other => other,
    }
}

fn read_alt(fs: &[(String, String)], head: &str) -> Result<usize, TreeError> {
    match take(fs, "alt") {
        None => Ok(0),
        Some(n) => n.parse().map_err(|_| TreeError(format!("({head} …): :alt {n} is not a number"))),
    }
}

type Features = Vec<(String, String)>;

fn leaf_id<'a>(rest: &'a [Value], head: &str) -> Result<(&'a str, Features), TreeError> {
    let Some(Value::Atom(id)) = rest.first() else {
        return err(format!("({head} …) starts with a lexeme id"));
    };
    if !is_lexeme_id(id) {
        return err(format!("({head} {id} …): not a lexeme id (землѧ.n, рещи.v, той.pron)"));
    }
    Ok((id, features(&rest[1..])?))
}

/// Read a tree from a parsed sexpr value.
pub fn from_sexpr(v: &Value) -> Result<Node, TreeError> {
    let Value::List(items) = v else {
        return err(format!("expected a list, got {}", crate::treebank::sexpr::print(v)));
    };
    let Some(Value::Atom(head)) = items.first() else {
        return err("a node starts with an atom head");
    };
    let rest = &items[1..];
    let lex = |id: &str, cells: CellSet, alt: usize| Ok(Node::Lex { id: id.to_string(), cells, alt, notes: Vec::new() });
    match head.as_str() {
        "w" => {
            let Some(Value::Str(surface)) = rest.first() else {
                return err("(w …) takes a quoted surface string");
            };
            let notes = features(&rest[1..])?;
            Ok(Node::W { surface: surface.clone(), notes })
        }
        "p" => {
            let Some(Value::Str(s)) = rest.first() else {
                return err("(p …) takes a quoted punctuation string");
            };
            if rest.len() != 1 || s.is_empty() {
                return err("(p …) takes exactly one non-empty string");
            }
            Ok(Node::Punct(s.clone()))
        }
        "f" => {
            let Some(Value::Atom(word)) = rest.first() else {
                return err("(f …) takes one atom");
            };
            if rest.len() == 1 {
                return Ok(Node::Fn(word.clone()));
            }
            // `(f въ.x.2 :alt 1)`: a closed lexeme printed by one of its
            // variants (во, ко, со — 3.3): a word leaf with the alternative
            let fs = features(&rest[1..])?;
            let alt = read_alt(&fs, "f")?;
            if !is_lexeme_id(word) {
                return err("(f …) with :alt takes a lexeme id");
            }
            lex(word, CellSet::one(Cell::Word), alt).map(|n| with_notes(n, &fs))
        }
        "n" => {
            let (id, fs) = leaf_id(rest, "n")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                for case in read_set(require(&fs, "case", "n")?, "case", parse_case)? {
                    for number in read_set(require(&fs, "num", "n")?, "number", parse_number)? {
                        product.push(Cell::Noun(NounCell::new(case, number)));
                    }
                }
            }
            lex(id, cell_set("n", &fs, Pos::Noun, product)?, read_alt(&fs, "n")?).map(|n| with_notes(n, &fs))
        }
        "adj" => {
            let (id, fs) = leaf_id(rest, "adj")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                let series = read_opt_set(&fs, "series", "series", parse_series)?;
                let degrees = match take(&fs, "deg") {
                    None => vec![Degree::Positive],
                    Some(d) => read_set(d, "degree", parse_degree)?,
                };
                for series in &series {
                    for degree in &degrees {
                        for gender in read_set(require(&fs, "g", "adj")?, "gender", parse_gender)? {
                            for number in read_set(require(&fs, "num", "adj")?, "number", parse_number)? {
                                for case in read_set(require(&fs, "case", "adj")?, "case", parse_case)? {
                                    product.push(Cell::Adj(AdjCell { series: *series, degree: *degree, gender, number, case }));
                                }
                            }
                        }
                    }
                }
            }
            lex(id, cell_set("adj", &fs, Pos::Adjective, product)?, read_alt(&fs, "adj")?).map(|n| with_notes(n, &fs))
        }
        "adv" => {
            let (id, fs) = leaf_id(rest, "adv")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                let degrees = match take(&fs, "deg") {
                    None => vec![Degree::Positive],
                    Some(d) => read_set(d, "degree", parse_degree)?,
                };
                for degree in degrees {
                    product.push(Cell::Adv(AdvCell { degree }));
                }
            }
            lex(id, cell_set("adv", &fs, Pos::Adjective, product)?, read_alt(&fs, "adv")?).map(|n| with_notes(n, &fs))
        }
        "v" => {
            let (id, fs) = leaf_id(rest, "v")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                match take(&fs, "form") {
                    None | Some("fin") => {
                        for tense in read_set(require(&fs, "t", "v")?, "tense", parse_finite)? {
                            for person in read_set(require(&fs, "p", "v")?, "person", parse_person)? {
                                for number in read_set(require(&fs, "num", "v")?, "number", parse_number)? {
                                    product.push(Cell::Verb(VerbCell::Finite { tense, person, number }));
                                }
                            }
                        }
                    }
                    Some("imp") => {
                        for person in read_set(require(&fs, "p", "v")?, "person", parse_person)? {
                            for number in read_set(require(&fs, "num", "v")?, "number", parse_number)? {
                                product.push(Cell::Verb(VerbCell::Imperative { person, number }));
                            }
                        }
                    }
                    Some("inf") => product.push(Cell::Verb(VerbCell::Infinitive)),
                    Some(other) => return err(format!("unknown form: {other}")),
                }
            }
            lex(id, cell_set("v", &fs, Pos::Verb, product)?, read_alt(&fs, "v")?).map(|n| with_notes(n, &fs))
        }
        "lp" => {
            let (id, fs) = leaf_id(rest, "lp")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                for gender in read_set(require(&fs, "g", "lp")?, "gender", parse_gender)? {
                    for number in read_set(require(&fs, "num", "lp")?, "number", parse_number)? {
                        product.push(Cell::Verb(VerbCell::LPart { gender, number }));
                    }
                }
            }
            lex(id, cell_set("lp", &fs, Pos::Verb, product)?, read_alt(&fs, "lp")?).map(|n| with_notes(n, &fs))
        }
        "part" => {
            let (id, fs) = leaf_id(rest, "part")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                let tenses = read_set(require(&fs, "t", "part")?, "participle tense", |t| match t { "pres" => Some(PartTense::Present), "past" => Some(PartTense::Past), _ => None })?;
                let voices = read_set(require(&fs, "voice", "part")?, "voice", |v| match v { "act" => Some(Voice::Active), "pass" => Some(Voice::Passive), _ => None })?;
                let series = read_set(require(&fs, "series", "part")?, "series", parse_series)?;
                for tense in &tenses {
                    for voice in &voices {
                        for series in &series {
                            for gender in read_set(require(&fs, "g", "part")?, "gender", parse_gender)? {
                                for number in read_set(require(&fs, "num", "part")?, "number", parse_number)? {
                                    for case in read_set(require(&fs, "case", "part")?, "case", parse_case)? {
                                        product.push(Cell::Verb(VerbCell::Participle { tense: *tense, voice: *voice, series: *series, gender, number, case }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            lex(id, cell_set("part", &fs, Pos::Verb, product)?, read_alt(&fs, "part")?).map(|n| with_notes(n, &fs))
        }
        "pn" => {
            let (id, fs) = leaf_id(rest, "pn")?;
            let mut product = Vec::new();
            if take(&fs, "cell").is_none() {
                let clitic = match take(&fs, "clit") {
                    None | Some("no") => false,
                    Some("yes") => true,
                    Some(other) => return err(format!("(pn …): :clit {other} is not yes/no")),
                };
                for person in read_opt_set(&fs, "p", "person", parse_person)? {
                    for gender in read_opt_set(&fs, "g", "gender", parse_gender)? {
                        for number in read_opt_set(&fs, "num", "number", parse_number)? {
                            for case in read_set(require(&fs, "case", "pn")?, "case", parse_case)? {
                                product.push(Cell::Pron(PronCell { clitic, person, gender, number, case }));
                            }
                        }
                    }
                }
            }
            lex(id, cell_set("pn", &fs, Pos::Pronoun, product)?, read_alt(&fs, "pn")?).map(|n| with_notes(n, &fs))
        }
        "cap" => {
            if rest.len() != 1 {
                return err("(cap …) takes exactly one child");
            }
            Ok(Node::Cap(Box::new(from_sexpr(&rest[0])?)))
        }
        "pw" | "pwa" => {
            if rest.len() < 2 {
                return err("(pw host encl…) takes a host and at least one enclitic");
            }
            let host = from_sexpr(&rest[0])?;
            if !matches!(host, Node::Lex { .. } | Node::Fn(_)) {
                return err("(pw …): the host is an analyzed leaf or a closed lexeme");
            }
            let enclitics = rest[1..].iter().map(from_sexpr).collect::<Result<Vec<_>, _>>()?;
            if !enclitics.iter().all(|e| matches!(e, Node::Fn(id) if is_lexeme_id(id))) {
                return err("(pw …): every enclitic is a closed lexeme by id ((f же.x))");
            }
            Ok(Node::Pw { host: Box::new(host), enclitics, apart: head == "pwa" })
        }
        "abbr" => {
            // (abbr "гл҃" child) or (abbr "гл҃" "гла" child): the second
            // string is the row's full-prefix skeleton
            let (prefix, full, child) = match (rest.first(), rest.get(1), rest.get(2), rest.get(3)) {
                (Some(Value::Str(prefix)), Some(Value::Str(full)), Some(child), None) => (prefix, Some(full.clone()), child),
                (Some(Value::Str(prefix)), Some(child), None, None) => (prefix, None, child),
                _ => return err("(abbr …) takes a quoted prefix, an optional quoted skeleton and one child"),
            };
            Ok(Node::Abbr { prefix: prefix.clone(), full, child: Box::new(from_sexpr(child)?) })
        }
        _ => {
            let children = rest.iter().map(from_sexpr).collect::<Result<Vec<_>, _>>()?;
            Ok(Node::Group { head: head.clone(), children })
        }
    }
}

fn atom(s: &str) -> Value {
    Value::Atom(s.to_string())
}
fn key(s: &str) -> Value {
    Value::Key(s.to_string())
}

/// The distinct values of one feature over a set, joined by `|`.
fn joined<T: PartialEq, S: AsRef<str>>(cells: &[Cell], pick: impl Fn(&Cell) -> Option<T>, name: impl Fn(&T) -> S) -> (usize, String) {
    let mut values: Vec<T> = Vec::new();
    for c in cells {
        if let Some(v) = pick(c)
            && !values.contains(&v)
        {
            values.push(v);
        }
    }
    let n = values.len().max(1);
    (n, values.iter().map(|v| name(v).as_ref().to_string()).collect::<Vec<_>>().join("|"))
}

/// The leaf syntax of (id, cells, alt): one cell as its features; a
/// product set as its features with `|`; any other set as `:cell`.
pub fn leaf_sexpr(id: &str, cells: &CellSet, alt: usize) -> Value {
    let head = |cell: &Cell| match cell {
        Cell::Noun(_) => "n",
        Cell::Adj(_) => "adj",
        Cell::Adv(_) => "adv",
        Cell::Verb(VerbCell::LPart { .. }) => "lp",
        Cell::Verb(VerbCell::Participle { .. }) => "part",
        Cell::Verb(_) => "v",
        Cell::Pron(_) => "pn",
        Cell::Word => "f",
    };
    let first = cells.first();
    let mut items = if cells.len() == 1 {
        leaf_items(id, &first)
    } else {
        let all = cells.cells();
        let same_shape = all.iter().all(|c| head(c) == head(&first) && std::mem::discriminant(c) == std::mem::discriminant(&first))
            && match first {
                Cell::Verb(v) => all.iter().all(|c| matches!(c, Cell::Verb(w) if std::mem::discriminant(w) == std::mem::discriminant(&v))),
                _ => true,
            };
        let product = same_shape.then(|| product_items(id, all)).flatten();
        match product {
            Some(items) => items,
            None => vec![atom(head(&first)), atom(id), key("cell"), atom(&cells.name())],
        }
    };
    if alt > 0 {
        items.push(key("alt"));
        items.push(atom(&alt.to_string()));
    }
    Value::List(items)
}

/// A product set's features with `|`, or `None` when the set is not the
/// product of its feature values.
fn product_items(id: &str, all: &[Cell]) -> Option<Vec<Value>> {
    let first = all[0];
    let series_name = |s: &Series| match s { Series::Short => "short", Series::Long => "long" };
    let mut feats: Vec<(&str, (usize, String))> = Vec::new();
    let head = match first {
        Cell::Noun(_) => {
            feats.push(("case", joined(all, |c| c.case(), |c| case_name(*c))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            "n"
        }
        Cell::Adj(_) => {
            feats.push(("case", joined(all, |c| c.case(), |c| case_name(*c))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            feats.push(("g", joined(all, |c| c.gender(), |g| gender_name(*g))));
            let series = joined(all, |c| if let Cell::Adj(a) = c { a.series } else { None }, series_name);
            let with_series = all.iter().filter(|c| matches!(c, Cell::Adj(a) if a.series.is_some())).count();
            if with_series != 0 && with_series != all.len() {
                return None;
            }
            if with_series != 0 {
                feats.push(("series", series));
            }
            let degree = joined(all, |c| if let Cell::Adj(a) = c { Some(a.degree) } else { None }, |d| degree_name(*d));
            if degree.1 != "pos" {
                feats.push(("deg", degree));
            }
            "adj"
        }
        Cell::Adv(_) => {
            feats.push(("deg", joined(all, |c| if let Cell::Adv(a) = c { Some(a.degree) } else { None }, |d| degree_name(*d))));
            "adv"
        }
        Cell::Verb(VerbCell::Finite { .. }) => {
            feats.push(("t", joined(all, |c| if let Cell::Verb(VerbCell::Finite { tense, .. }) = c { Some(*tense) } else { None }, |t| finite_name(*t))));
            feats.push(("p", joined(all, |c| c.person(), |p| person_name(*p))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            "v"
        }
        Cell::Verb(VerbCell::Imperative { .. }) => {
            feats.push(("form", (1, "imp".to_string())));
            feats.push(("p", joined(all, |c| c.person(), |p| person_name(*p))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            "v"
        }
        Cell::Verb(VerbCell::Infinitive) => return None,
        Cell::Verb(VerbCell::LPart { .. }) => {
            feats.push(("g", joined(all, |c| c.gender(), |g| gender_name(*g))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            "lp"
        }
        Cell::Verb(VerbCell::Participle { .. }) => {
            feats.push(("t", joined(all, |c| if let Cell::Verb(VerbCell::Participle { tense, .. }) = c { Some(*tense) } else { None }, |t| match t { PartTense::Present => "pres", PartTense::Past => "past" })));
            feats.push(("voice", joined(all, |c| if let Cell::Verb(VerbCell::Participle { voice, .. }) = c { Some(*voice) } else { None }, |v| match v { Voice::Active => "act", Voice::Passive => "pass" })));
            feats.push(("series", joined(all, |c| if let Cell::Verb(VerbCell::Participle { series, .. }) = c { Some(*series) } else { None }, series_name)));
            feats.push(("case", joined(all, |c| c.case(), |c| case_name(*c))));
            feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            feats.push(("g", joined(all, |c| c.gender(), |g| gender_name(*g))));
            "part"
        }
        Cell::Pron(_) => {
            // an optional feature must be present on every member or on none
            let count = |f: fn(&PronCell) -> bool| all.iter().filter(|c| matches!(c, Cell::Pron(p) if f(p))).count();
            let persons = count(|p| p.person.is_some());
            let numbers = count(|p| p.number.is_some());
            let genders = count(|p| p.gender.is_some());
            let clitics = count(|p| p.clitic);
            for k in [persons, numbers, genders, clitics] {
                if k != 0 && k != all.len() {
                    return None;
                }
            }
            if persons != 0 {
                feats.push(("p", joined(all, |c| c.person(), |p| person_name(*p))));
            }
            if numbers != 0 {
                feats.push(("num", joined(all, |c| c.number(), |n| number_name(*n))));
            }
            feats.push(("case", joined(all, |c| c.case(), |c| case_name(*c))));
            if genders != 0 {
                feats.push(("g", joined(all, |c| c.gender(), |g| gender_name(*g))));
            }
            if clitics != 0 {
                feats.push(("clit", (1, "yes".to_string())));
            }
            "pn"
        }
        Cell::Word => return None,
    };
    let n: usize = feats.iter().map(|(_, (count, _))| *count).product();
    if n != all.len() {
        return None;
    }
    let mut items = vec![atom(head), atom(id)];
    for (k, (_, value)) in feats {
        items.push(key(k));
        items.push(atom(&value));
    }
    Some(items)
}

/// The features of one cell.
fn leaf_items(id: &str, cell: &Cell) -> Vec<Value> {
    let case = |c: &Case| atom(case_name(*c));
    let num = |n: &Number| atom(number_name(*n));
    let gender = |g: &Gender| atom(gender_name(*g));
    let person = |p: &Person| atom(person_name(*p));
    let series = |s: &Series| atom(match s { Series::Short => "short", Series::Long => "long" });
    match cell {
        Cell::Noun(NounCell { case: c, number: n }) => vec![atom("n"), atom(id), key("case"), case(c), key("num"), num(n)],
        Cell::Adj(AdjCell { series: s, degree, gender: g, number: n, case: c }) => {
            let mut v = vec![atom("adj"), atom(id), key("case"), case(c), key("num"), num(n), key("g"), gender(g)];
            if let Some(s) = s {
                v.push(key("series"));
                v.push(series(s));
            }
            if *degree != Degree::Positive {
                v.push(key("deg"));
                v.push(atom(if *degree == Degree::Comparative { "comp" } else { "sup" }));
            }
            v
        }
        Cell::Adv(AdvCell { degree }) => {
            let mut v = vec![atom("adv"), atom(id)];
            if *degree != Degree::Positive {
                v.push(key("deg"));
                v.push(atom(degree_name(*degree)));
            }
            v
        }
        Cell::Verb(VerbCell::Finite { tense, person: p, number: n }) => vec![
            atom("v"), atom(id),
            key("t"), atom(match tense { FiniteTense::Present => "pres", FiniteTense::Imperfect => "impf", FiniteTense::Aorist => "aor", FiniteTense::Future => "fut" }),
            key("p"), person(p), key("num"), num(n),
        ],
        Cell::Verb(VerbCell::Imperative { person: p, number: n }) => vec![atom("v"), atom(id), key("form"), atom("imp"), key("p"), person(p), key("num"), num(n)],
        Cell::Verb(VerbCell::Infinitive) => vec![atom("v"), atom(id), key("form"), atom("inf")],
        Cell::Verb(VerbCell::LPart { gender: g, number: n }) => vec![atom("lp"), atom(id), key("g"), gender(g), key("num"), num(n)],
        Cell::Verb(VerbCell::Participle { tense, voice, series: s, gender: g, number: n, case: c }) => vec![
            atom("part"), atom(id),
            key("t"), atom(match tense { PartTense::Present => "pres", PartTense::Past => "past" }),
            key("voice"), atom(match voice { Voice::Active => "act", Voice::Passive => "pass" }),
            key("series"), series(s), key("case"), case(c), key("num"), num(n), key("g"), gender(g),
        ],
        Cell::Pron(PronCell { clitic, person: p, gender: g, number: n, case: c }) => {
            let mut v = vec![atom("pn"), atom(id)];
            if let Some(p) = p {
                v.push(key("p"));
                v.push(person(p));
            }
            if let Some(n) = n {
                v.push(key("num"));
                v.push(num(n));
            }
            v.push(key("case"));
            v.push(case(c));
            if let Some(g) = g {
                v.push(key("g"));
                v.push(gender(g));
            }
            if *clitic {
                v.push(key("clit"));
                v.push(atom("yes"));
            }
            v
        }
        Cell::Word => vec![atom("f"), atom(id)],
    }
}

/// Write a tree back to a sexpr value (`from_sexpr(to_sexpr(n)) == n`).
pub fn to_sexpr(node: &Node) -> Value {
    match node {
        Node::W { surface, notes } => {
            let mut items = vec![atom("w"), Value::Str(surface.clone())];
            for (k, v) in notes {
                items.push(key(k));
                items.push(atom(v));
            }
            Value::List(items)
        }
        Node::Punct(s) => Value::List(vec![atom("p"), Value::Str(s.clone())]),
        Node::Fn(word) => Value::List(vec![atom("f"), atom(word)]),
        Node::Lex { id, cells, alt, notes } => {
            let mut v = leaf_sexpr(id, cells, *alt);
            if let Value::List(items) = &mut v {
                for (k, val) in notes {
                    items.push(key(k));
                    items.push(atom(val));
                }
            }
            v
        }
        Node::Cap(child) => Value::List(vec![atom("cap"), to_sexpr(child)]),
        Node::Pw { host, enclitics, apart } => {
            let mut items = vec![atom(if *apart { "pwa" } else { "pw" }), to_sexpr(host)];
            items.extend(enclitics.iter().map(to_sexpr));
            Value::List(items)
        }
        Node::Abbr { prefix, full, child } => {
            let mut items = vec![atom("abbr"), Value::Str(prefix.clone())];
            if let Some(full) = full {
                items.push(Value::Str(full.clone()));
            }
            items.push(to_sexpr(child));
            Value::List(items)
        }
        Node::Group { head, children } => {
            let mut items = vec![atom(head)];
            items.extend(children.iter().map(to_sexpr));
            Value::List(items)
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Punctuation that glues to the FOLLOWING token (opening brackets and
/// quotes); everything else glues to the preceding one.
fn glues_right(p: &str) -> bool {
    matches!(p.chars().next(), Some('(' | '[' | '«' | '„' | '“'))
}

/// Render a tree: a left-to-right walk emitting one token per leaf,
/// single spaces between tokens, punctuation glued by [`glues_right`].
/// Analyzed leaves inflect through the lexicon of the given recension.
pub fn render(node: &Node, recension: &church_slavonic::Recension) -> Result<String, TreeError> {
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
pub fn leaf_form(id: &str, cell: Cell, alt: usize, recension: church_slavonic::Recension) -> Result<church_slavonic::Form, TreeError> {
    let lexicon = church_slavonic::Lexicon::of(recension);
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
pub fn leaf_print(id: &str, cell: Cell, alt: usize, recension: church_slavonic::Recension) -> Result<String, TreeError> {
    Ok(leaf_form(id, cell, alt, recension)?.print(recension))
}

/// The form of a phonological word's host: an analyzed leaf's, or a
/// closed lexeme's lemma.
fn host_form(host: &Node, recension: church_slavonic::Recension) -> Result<church_slavonic::Form, TreeError> {
    match host {
        Node::Lex { id, cells, alt, .. } => leaf_form(id, cells.first(), *alt, recension),
        Node::Fn(id) if is_lexeme_id(id) => match church_slavonic::Lexicon::of(recension).get(id) {
            Some(l) => Ok(church_slavonic::Form::from_print(&l.lemma)),
            None => err(format!("{id}: no such lexeme in the lexicon")),
        },
        _ => err("(pw …): the host is an analyzed leaf or a closed lexeme"),
    }
}

/// The print of a phonological word: the host's form with its enclitics,
/// accented as one unit — written solid (и҆̀хже), or apart with the host
/// keeping the unit's oxia (Землѧ́ же).
pub fn unit_print(host: &Node, enclitics: &[Node], apart: bool, recension: church_slavonic::Recension) -> Result<String, TreeError> {
    let form = host_form(host, recension)?;
    let lexicon = church_slavonic::Lexicon::of(recension);
    let mut letters: Vec<String> = Vec::new();
    for e in enclitics {
        let Node::Fn(eid) = e else { return err("(pw …): an enclitic is (f <id>)") };
        let Some(lexeme) = lexicon.get(eid) else {
            return err(format!("{eid}: no such lexeme in the lexicon"));
        };
        if lexeme.prosody() != church_slavonic::grammar::Prosody::Enclitic {
            return err(format!("{eid} is not an enclitic (pros=encl)"));
        }
        letters.push(lexeme.lemma.clone());
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

fn walk(node: &Node, recension: &church_slavonic::Recension, out: &mut String, glue_next: &mut bool) -> Result<(), TreeError> {
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
            let lemma_key = match crate::treebank::disambiguate::leaf(child) {
                Some(Node::Lex { id, .. }) => church_slavonic::Lexicon::synodal().get(id).map(|l| church_slavonic::orthography::comparison_key(&l.lemma)),
                _ => None,
            };
            let mut rows: Vec<&crate::treebank::titlo::Row> = crate::treebank::titlo::rows()
                .iter()
                .filter(|row| row.abbr == prefix && full.as_ref().is_none_or(|f| row.full == *f))
                .collect();
            rows.sort_by_key(|row| lemma_key.as_ref().is_some_and(|k| church_slavonic::orthography::comparison_key(row.lemma) != *k));
            let abbreviated = rows.into_iter().find_map(|row| crate::treebank::titlo::abbreviate(&inner, row));
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
pub fn is_function_word(word: &str, recension: church_slavonic::Recension) -> bool {
    crate::treebank::closed::is_closed(word)
        || church_slavonic::Lexicon::of(recension).analyze(word).iter().any(|a| a.exact && a.cell == Cell::Word)
}

/// Is an atom a 2.0 lexeme id (`землѧ.n`, `сꙑнъ.n.2`)?
pub fn is_lexeme_id(lemma: &str) -> bool {
    let mut parts = lemma.split('.');
    let _stem = parts.next();
    match (parts.next(), parts.next(), parts.next()) {
        (Some(tag), None, None) => church_slavonic::Pos::parse(tag).is_some(),
        (Some(tag), Some(n), None) => church_slavonic::Pos::parse(tag).is_some() && n.chars().all(|c| c.is_ascii_digit()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treebank::sexpr;

    fn read(text: &str) -> Node {
        from_sexpr(&sexpr::parse(text).expect("sexpr")).expect("node")
    }

    #[test]
    fn node_sexpr_round_trip() {
        let text = r#"(s (cl (f и҆) (v рещи.v :t aor :p 3 :num sg)
            (np (adj великій.a :case nom :num sg :g m :series long) (n богъ.n :case nom :num sg :alt 1))
            (pn азъ.pron :p 1 :num sg :case dat :clit yes) (pn себе.pron :case dat)
            (part творити.v :t pres :voice act :series long :case nom :num sg :g m)
            (lp быти.v :g m :num sg) (v рещи.v :form inf) (v рещи.v :form imp :p 2 :num pl)
            (p ":") (w "гдⷭ҇ь" :lemma госпо́дь) (abbr "бг҃" (cap (n богъ.n :case nom :num sg)))))"#;
        let node = read(text);
        let back = to_sexpr(&node);
        assert_eq!(from_sexpr(&back).expect("round-trip"), node);
        assert_eq!(sexpr::print(&to_sexpr(&from_sexpr(&back).unwrap())), sexpr::print(&back));
    }

    #[test]
    fn underspecified_leaves_round_trip() {
        // a product set as disjunctive features, a non-product set as :cell
        for text in [
            "(n свѣтъ.n :case nom|acc|voc :num sg)",
            "(adj мꙋдрый.a :case gen :num sg :g m|n :series long)",
            "(v рещи.v :t aor :p 2|3 :num sg)",
            "(v сотворити.v :cell aor|impv.2|3.sg)",
            "(n жена.n :cell nom.pl|gen.sg|acc.pl)",
            "(pn себе.pron :case dat|loc)",
            "(part творити.v :t pres :voice act :series long :case nom|acc :num sg :g m)",
            "(abbr \"бг҃\" (n богъ.n :case nom|acc :num sg :alt 1))",
        ] {
            let node = read(text);
            assert_eq!(sexpr::print(&to_sexpr(&node)), text);
            if let Node::Lex { cells, .. } = &node {
                assert!(cells.len() > 1, "{text}");
            }
        }
        // a set across two verb forms is written as the factored :cell
        // name, and reads back from the listed form too
        assert_eq!(read("(v сотворити.v :cell aor.2.sg|aor.3.sg|impv.2.sg|impv.3.sg)"), read("(v сотворити.v :cell aor|impv.2|3.sg)"));
        // the reader expands the product; the set is sorted and deduplicated
        let Node::Lex { cells, .. } = read("(n свѣтъ.n :case voc|nom|acc|nom :num sg)") else { panic!() };
        assert_eq!(cells.name(), "nom|acc|voc.sg");
        assert!(from_sexpr(&sexpr::parse("(n свѣтъ.n :case nom|bogus :num sg)").unwrap()).is_err());
        assert!(from_sexpr(&sexpr::parse("(n свѣтъ.n :cell nom.sg|word)").unwrap()).is_err());
    }

    #[test]
    fn shape_errors_are_loud() {
        for bad in [
            "(n богъ.n :case nom)",          // missing :num
            "(n богъ.n :case weird :num sg)", // unknown value
            "(n бо́гъ :case nom :num sg)",    // a lemma, not an id
            "(v рещи.v :p 3 :num sg)",        // missing :t on a finite verb
            "(p \"\")",                       // empty punctuation
            "(f)",                            // missing function word
            "()",                             // headless
        ] {
            let v = sexpr::parse(bad).expect("parses as sexpr");
            assert!(from_sexpr(&v).is_err(), "{bad}");
        }
    }

    #[test]
    fn verbatim_wrap_round_trips_the_pitfall_verse() {
        // Luke 15:12 as pinned — apparatus marks and footnote included
        let verse = " и҆ речѐ ю҆нѣ́йшїй ꙾є҆ю̀꙾[26] ѻ҆тцꙋ̀: ѻ҆́тче, да́ждь мѝ досто́йнꙋю ча́сть и҆мѣ́нїѧ. И҆ раздѣлѝ и҆́ма и҆мѣ́нїе.";
        let tree = verbatim_tree(verse);
        let rendered = render(&tree, &church_slavonic::Recension::Synodal).expect("renders");
        assert_eq!(rendered, verse.trim());
    }

    /// Exact-output tests: every expected string is the print's own.
    #[test]
    fn analyzed_leaves_render_through_the_lexicon() {
        let syn = church_slavonic::Recension::Synodal;
        for (tree, expect) in [
            ("(n начало.n :case loc :num sg)", "нача́лѣ"),
            ("(n землѧ.n :case acc :num sg)", "зе́млю"),
            ("(v сотворити.v :t aor :p 3 :num sg)", "сотворѝ"),
            ("(v быти.v :t aor :p 3 :num sg)", "бы́сть"),
            ("(v быти.v :t aor :p 3 :num sg :alt 1)", "бѣ̀"),
            ("(v быти.v :t fut :p 3 :num sg)", "бꙋ́детъ"),
            ("(v быти.v :t pres :p 3 :num sg)", "є҆́сть"),
            ("(pn азъ.pron :p 1 :num sg :case dat :clit yes)", "мѝ"),
            ("(pn онъ.pron :p 3 :num pl :case gen :g m)", "и҆́хъ"),
            ("(cap (f и҆))", "И҆"),
            ("(abbr \"бг҃\" (n богъ.n :case nom :num sg))", "бг҃ъ"),
            ("(s (f и҆) (v рещи.v :t aor :p 3 :num sg) (p \":\"))", "и҆ речѐ:"),
        ] {
            let rendered = render(&read(tree), &syn).unwrap_or_else(|e| panic!("{tree}: {e}"));
            assert_eq!(rendered, expect, "{tree}");
        }
        assert!(render(&read("(n нѣтъ.n :case nom :num sg)"), &syn).is_err());
        assert!(render(&read("(f нѣтъ)"), &syn).is_err());
    }

    #[test]
    fn punctuation_glue() {
        let syn = church_slavonic::Recension::Synodal;
        let tree = read(r#"(s (w "а") (p ",") (p "(") (w "б") (p ")") (p "."))"#);
        assert_eq!(render(&tree, &syn).unwrap(), "а, (б).");
    }
}
