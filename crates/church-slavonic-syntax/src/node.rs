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

use crate::sexpr::Value;
use std::fmt;

/// One node. Groups (`s`, `cl`, `np`, `pp`, …) carry an arbitrary head
/// atom — the linter knows some heads, the renderer treats them all as
/// ordered sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// `(w "гдⷭ҇ь")` — a witnessed surface form, rendered as-is. May
    /// carry belief annotations (`:lemma`, `:case`, …) that the renderer
    /// IGNORES; they exist for the linter and the lifting pipeline.
    W { surface: String, notes: Vec<(String, String)> },
    /// `(p ",")` — punctuation; glues to the neighbouring token (left by
    /// default, right for opening brackets/quotes).
    Punct(String),
    /// `(f и҆)` — a closed-class function word from [`crate::closed::TABLE`].
    Fn(String),
    /// `(n лемма :case nom :num sg)`
    Noun { lemma: String, case: Case, number: Number },
    /// `(adj лемма :case nom :num sg :g m)` — positive degree unless
    /// `:deg comp`.
    Adj { lemma: String, case: Case, number: Number, gender: Gender, degree: Degree },
    /// `(v лемма :t aor :p 3 :num sg)` — finite unless `:form imp`/`inf`.
    Verb { lemma: String, person: Person, number: Number, tense: Tense, form: Form },
    /// `(lp лемма :g m :num sg)` — the l-participle.
    LPart { lemma: String, gender: Gender, number: Number },
    /// `(pn лемма :case nom :num sg :g m)` — a non-personal pronoun.
    Npron { lemma: String, gender: Gender, number: Number, case: Case },
    /// `(pers :p 3 :num sg :case gen :g m)` — the personal pronoun; the
    /// first and second persons carry no gender (omit :g).
    Pers { person: Person, number: Number, gender: Option<Gender>, case: Case },
    /// `(part лемма :t pres :voice act :series long :case nom :num sg :g m)`
    Part {
        lemma: String,
        tense: Tense,
        voice: Voice,
        series: Series,
        case: Case,
        number: Number,
        gender: Gender,
    },
    /// `(abbr "гдⷭ҇" X)` — render the child in full, then abbreviate it
    /// under the matching row of `data/titlo.tsv` (the titlo layer).
    Abbr { prefix: String, child: Box<Node> },
    /// `(cap X)` — uppercase the first letter of the child's rendering
    /// (sentence-initial capitals; the tree stays lemma-true).
    Cap(Box<Node>),
    /// `(np …)`, `(cl …)`, `(s …)` — an ordered group.
    Group { head: String, children: Vec<Node> },
}

pub use church_slavonic::{Case, Degree, Form, Gender, Number, Person, Series, Tense, Voice};

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
// Feature vocabularies — one place, both directions
// ---------------------------------------------------------------------------

macro_rules! vocab {
    ($read:ident, $show:ident, $ty:ty, $feature:literal, $(($atom:literal, $variant:expr)),+ $(,)?) => {
        fn $read(atom: &str) -> Result<$ty, TreeError> {
            match atom {
                $($atom => Ok($variant),)+
                other => err(format!("unknown {}: {other}", $feature)),
            }
        }
        fn $show(v: &$ty) -> &'static str {
            match v {
                $(x if *x == $variant => $atom,)+
                _ => unreachable!(),
            }
        }
    };
}

vocab!(read_case, show_case, Case, "case",
    ("nom", Case::Nominative), ("gen", Case::Genitive), ("dat", Case::Dative),
    ("acc", Case::Accusative), ("ins", Case::Instrumental), ("loc", Case::Locative),
    ("voc", Case::Vocative));
vocab!(read_num, show_num, Number, "number",
    ("sg", Number::Singular), ("du", Number::Dual), ("pl", Number::Plural));
vocab!(read_gender, show_gender, Gender, "gender",
    ("m", Gender::Masculine), ("f", Gender::Feminine), ("n", Gender::Neuter));
vocab!(read_person, show_person, Person, "person",
    ("1", Person::First), ("2", Person::Second), ("3", Person::Third));
vocab!(read_tense, show_tense, Tense, "tense",
    ("pres", Tense::Present), ("impf", Tense::Imperfect), ("aor", Tense::Aorist));
vocab!(read_voice, show_voice, Voice, "voice",
    ("act", Voice::Active), ("pass", Voice::Passive));
vocab!(read_series, show_series, Series, "series",
    ("short", Series::Short), ("long", Series::Long));

// ---------------------------------------------------------------------------
// sexpr <-> Node
// ---------------------------------------------------------------------------

fn features(items: &[Value]) -> Result<Vec<(String, String)>, TreeError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < items.len() {
        match (&items[i], items.get(i + 1)) {
            (Value::Key(k), Some(Value::Atom(v))) => {
                out.push((k.clone(), v.clone()));
                i += 2;
            }
            (Value::Key(k), Some(Value::Str(v))) => {
                out.push((k.clone(), v.clone()));
                i += 2;
            }
            (Value::Key(k), _) => return err(format!(":{k} lacks a value")),
            (other, _) => return err(format!("expected :feature, got {}", crate::sexpr::print(other))),
        }
    }
    Ok(out)
}

fn take<'a>(fs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn require<'a>(fs: &'a [(String, String)], key: &str, head: &str) -> Result<&'a str, TreeError> {
    take(fs, key).ok_or_else(|| TreeError(format!("({head} …) requires :{key}")))
}

/// Read a tree from a parsed sexpr value.
pub fn from_sexpr(v: &Value) -> Result<Node, TreeError> {
    let Value::List(items) = v else {
        return err(format!("expected a list, got {}", crate::sexpr::print(v)));
    };
    let Some(Value::Atom(head)) = items.first() else {
        return err("a node starts with an atom head");
    };
    let rest = &items[1..];
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
            if rest.len() != 1 {
                return err("(f …) takes exactly one atom");
            }
            Ok(Node::Fn(word.clone()))
        }
        "n" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(n …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            Ok(Node::Noun {
                lemma: lemma.clone(),
                case: read_case(require(&fs, "case", "n")?)?,
                number: read_num(require(&fs, "num", "n")?)?,
            })
        }
        "adj" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(adj …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            Ok(Node::Adj {
                lemma: lemma.clone(),
                case: read_case(require(&fs, "case", "adj")?)?,
                number: read_num(require(&fs, "num", "adj")?)?,
                gender: read_gender(require(&fs, "g", "adj")?)?,
                degree: match take(&fs, "deg") {
                    None | Some("pos") => Degree::Positive,
                    Some("comp") => Degree::Comparative,
                    Some("sup") => Degree::Superlative,
                    Some(other) => return err(format!("unknown degree: {other}")),
                },
            })
        }
        "v" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(v …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            let form = match take(&fs, "form") {
                None | Some("fin") => Form::Finite,
                Some("imp") => Form::Imperative,
                Some("inf") => Form::Infinitive,
                Some(other) => return err(format!("unknown form: {other}")),
            };
            Ok(Node::Verb {
                lemma: lemma.clone(),
                person: read_person(require(&fs, "p", "v")?)?,
                number: read_num(require(&fs, "num", "v")?)?,
                tense: match take(&fs, "t") {
                    Some(t) => read_tense(t)?,
                    None if form == Form::Infinitive => Tense::Present,
                    None => return err("(v …) requires :t"),
                },
                form,
            })
        }
        "pn" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(pn …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            Ok(Node::Npron {
                lemma: lemma.clone(),
                case: read_case(require(&fs, "case", "pn")?)?,
                number: read_num(require(&fs, "num", "pn")?)?,
                gender: read_gender(require(&fs, "g", "pn")?)?,
            })
        }
        "pers" => {
            let fs = features(rest)?;
            Ok(Node::Pers {
                person: read_person(require(&fs, "p", "pers")?)?,
                number: read_num(require(&fs, "num", "pers")?)?,
                case: read_case(require(&fs, "case", "pers")?)?,
                gender: match take(&fs, "g") {
                    Some(g) => Some(read_gender(g)?),
                    None => None,
                },
            })
        }
        "part" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(part …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            Ok(Node::Part {
                lemma: lemma.clone(),
                tense: read_tense(require(&fs, "t", "part")?)?,
                voice: read_voice(require(&fs, "voice", "part")?)?,
                series: read_series(require(&fs, "series", "part")?)?,
                case: read_case(require(&fs, "case", "part")?)?,
                number: read_num(require(&fs, "num", "part")?)?,
                gender: read_gender(require(&fs, "g", "part")?)?,
            })
        }
        "cap" => {
            if rest.len() != 1 {
                return err("(cap …) takes exactly one child");
            }
            Ok(Node::Cap(Box::new(from_sexpr(&rest[0])?)))
        }
        "abbr" => {
            let (Some(Value::Str(prefix)), Some(child), None) =
                (rest.first(), rest.get(1), rest.get(2))
            else {
                return err("(abbr …) takes a quoted prefix and one child");
            };
            Ok(Node::Abbr { prefix: prefix.clone(), child: Box::new(from_sexpr(child)?) })
        }
        "lp" => {
            let Some(Value::Atom(lemma)) = rest.first() else {
                return err("(lp …) starts with a lemma atom");
            };
            let fs = features(&rest[1..])?;
            Ok(Node::LPart {
                lemma: lemma.clone(),
                gender: read_gender(require(&fs, "g", "lp")?)?,
                number: read_num(require(&fs, "num", "lp")?)?,
            })
        }
        _ => {
            let children = rest.iter().map(from_sexpr).collect::<Result<Vec<_>, _>>()?;
            Ok(Node::Group { head: head.clone(), children })
        }
    }
}

/// Write a tree back to a sexpr value (`from_sexpr(to_sexpr(n)) == n`).
pub fn to_sexpr(node: &Node) -> Value {
    let atom = |s: &str| Value::Atom(s.to_string());
    let key = |s: &str| Value::Key(s.to_string());
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
        Node::Noun { lemma, case, number } => Value::List(vec![
            atom("n"), atom(lemma),
            key("case"), atom(show_case(case)),
            key("num"), atom(show_num(number)),
        ]),
        Node::Adj { lemma, case, number, gender, degree } => {
            let mut items = vec![
                atom("adj"), atom(lemma),
                key("case"), atom(show_case(case)),
                key("num"), atom(show_num(number)),
                key("g"), atom(show_gender(gender)),
            ];
            match degree {
                Degree::Positive => {}
                Degree::Comparative => {
                    items.push(key("deg"));
                    items.push(atom("comp"));
                }
                Degree::Superlative => {
                    items.push(key("deg"));
                    items.push(atom("sup"));
                }
            }
            Value::List(items)
        }
        Node::Verb { lemma, person, number, tense, form } => {
            let mut items = vec![
                atom("v"), atom(lemma),
                key("t"), atom(show_tense(tense)),
                key("p"), atom(show_person(person)),
                key("num"), atom(show_num(number)),
            ];
            match form {
                Form::Finite => {}
                Form::Imperative => {
                    items.push(key("form"));
                    items.push(atom("imp"));
                }
                Form::Infinitive => {
                    items.push(key("form"));
                    items.push(atom("inf"));
                }
                Form::Participle => {
                    items.push(key("form"));
                    items.push(atom("part"));
                }
            }
            Value::List(items)
        }
        Node::LPart { lemma, gender, number } => Value::List(vec![
            atom("lp"), atom(lemma),
            key("g"), atom(show_gender(gender)),
            key("num"), atom(show_num(number)),
        ]),
        Node::Npron { lemma, gender, number, case } => Value::List(vec![
            atom("pn"), atom(lemma),
            key("case"), atom(show_case(case)),
            key("num"), atom(show_num(number)),
            key("g"), atom(show_gender(gender)),
        ]),
        Node::Pers { person, number, gender, case } => {
            let mut items = vec![
                atom("pers"),
                key("p"), atom(show_person(person)),
                key("num"), atom(show_num(number)),
                key("case"), atom(show_case(case)),
            ];
            if let Some(g) = gender {
                items.push(key("g"));
                items.push(atom(show_gender(g)));
            }
            Value::List(items)
        }
        Node::Part { lemma, tense, voice, series, case, number, gender } => Value::List(vec![
            atom("part"), atom(lemma),
            key("t"), atom(show_tense(tense)),
            key("voice"), atom(show_voice(voice)),
            key("series"), atom(show_series(series)),
            key("case"), atom(show_case(case)),
            key("num"), atom(show_num(number)),
            key("g"), atom(show_gender(gender)),
        ]),
        Node::Cap(child) => Value::List(vec![atom("cap"), to_sexpr(child)]),
        Node::Abbr { prefix, child } => {
            Value::List(vec![atom("abbr"), Value::Str(prefix.clone()), to_sexpr(child)])
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
/// Analyzed leaves inflect through the `church-slavonic` public API in
/// the given recension.
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

fn walk(
    node: &Node,
    recension: &church_slavonic::Recension,
    out: &mut String,
    glue_next: &mut bool,
) -> Result<(), TreeError> {
    use church_slavonic::ChurchSlavonic;
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
            if !crate::closed::is_closed(word) {
                return err(format!("(f {word}) is not in the closed-class table"));
            }
            emit(word, false, out, glue_next);
        }
        Node::Noun { lemma, case, number } => {
            emit(&ChurchSlavonic::noun(lemma, case, number, recension), false, out, glue_next);
        }
        Node::Adj { lemma, case, number, gender, degree } => {
            emit(
                &ChurchSlavonic::adj(lemma, case, number, gender, degree, recension),
                false,
                out,
                glue_next,
            );
        }
        Node::Verb { lemma, person, number, tense, form } => {
            emit(
                &ChurchSlavonic::verb(lemma, person, number, tense, form, recension),
                false,
                out,
                glue_next,
            );
        }
        Node::LPart { lemma, gender, number } => {
            emit(
                &ChurchSlavonic::l_participle(lemma, gender, number, recension),
                false,
                out,
                glue_next,
            );
        }
        Node::Npron { lemma, gender, number, case } => {
            let form = ChurchSlavonic::npron(lemma, gender, number, case, recension);
            if form.is_empty() {
                return err(format!("(pn {lemma}) renders empty in this recension"));
            }
            emit(&form, false, out, glue_next);
        }
        Node::Pers { person, number, gender, case } => {
            // the first and second persons carry no gender; masculine is
            // the API's placeholder argument for them
            let g = gender.unwrap_or(Gender::Masculine);
            let form = ChurchSlavonic::pronoun(person, number, &g, case, recension);
            if form.is_empty() {
                return err("(pers …) renders empty".to_string());
            }
            emit(form, false, out, glue_next);
        }
        Node::Part { lemma, tense, voice, series, case, number, gender } => {
            emit(
                &ChurchSlavonic::participle(
                    lemma, tense, voice, series, case, number, gender, recension,
                ),
                false,
                out,
                glue_next,
            );
        }
        Node::Abbr { prefix, child } => {
            let mut inner = String::new();
            let mut inner_glue = false;
            walk(child, recension, &mut inner, &mut inner_glue)?;
            let abbreviated = crate::titlo::rows()
                .iter()
                .filter(|row| row.abbr == prefix)
                .find_map(|row| crate::titlo::abbreviate(&inner, row));
            match abbreviated {
                Some(form) => emit(&form, false, out, glue_next),
                None => {
                    return err(format!(
                        "(abbr \"{prefix}\" …): no titlo row abbreviates «{inner}»"
                    ));
                }
            }
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
        children: tokenize(verse)
            .into_iter()
            .map(|t| Node::W { surface: t.to_string(), notes: Vec::new() })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexpr;

    #[test]
    fn node_sexpr_round_trip() {
        let text = r#"(s (cl (f и҆) (v рещѝ :t aor :p 3 :num sg)
            (np (n бо́гъ :case nom :num sg))
            (p ":") (w "гдⷭ҇ь" :lemma госпо́дь)))"#;
        let v = sexpr::parse(text).expect("parses");
        let node = from_sexpr(&v).expect("reads");
        let back = to_sexpr(&node);
        assert_eq!(from_sexpr(&back).expect("round-trip"), node);
    }

    #[test]
    fn shape_errors_are_loud() {
        for bad in [
            "(n бо́гъ :case nom)",          // missing :num
            "(n бо́гъ :case weird :num sg)", // unknown value
            "(v рещѝ :p 3 :num sg)",        // missing :t on a finite verb
            "(p \"\")",                      // empty punctuation
            "(f) ",                          // missing function word
            "()",                            // headless
        ] {
            let v = sexpr::parse(bad.trim()).expect("parses as sexpr");
            assert!(from_sexpr(&v).is_err(), "{bad}");
        }
    }

    #[test]
    fn verbatim_wrap_round_trips_the_pitfall_verse() {
        // Luke 15:12 as pinned — apparatus marks and footnote included
        let verse = " и҆ речѐ ю҆нѣ́йшїй ꙾є҆ю̀꙾[26] ѻ҆тцꙋ̀: ѻ҆́тче, да́ждь мѝ досто́йнꙋю ча́сть и҆мѣ́нїѧ. И҆ раздѣлѝ и҆́ма и҆мѣ́нїе.";
        let tree = verbatim_tree(verse);
        let rendered =
            render(&tree, &church_slavonic::Recension::Synodal).expect("renders");
        assert_eq!(rendered, verse.trim());
    }

    /// Part 1's exact-output tests: every expected string on the right is
    /// PASTED crate output (or the pinned print itself) — never retyped.
    #[test]
    fn analyzed_leaves_render_through_the_crate() {
        let syn = church_slavonic::Recension::Synodal;
        let read = |text: &str| {
            from_sexpr(&sexpr::parse(text).expect("sexpr")).expect("node")
        };
        for (tree, expect) in [
            ("(n нача́ло :case loc :num sg)", "нача́лѣ"),
            ("(n землѧ̀ :case acc :num sg)", "зе́млю"),
            ("(n не́бо :case acc :num sg)", "не́бо"),
            ("(v сотвори́ти :t aor :p 3 :num sg)", "сотворѝ"),
            ("(v рещѝ :t aor :p 3 :num sg)", "речѐ"),
            ("(v бы́ти :t aor :p 3 :num sg)", "бы́сть"),
            ("(v бы́ти :t pres :p 3 :num sg)", "є҆́сть"),
            ("(adj вели́кїй :case acc :num sg :g n)", "вели́кое"),
            ("(cap (f въ))", "Въ"),
            ("(f и҆)", "и҆"),
            // wave 2: the pronoun and participle leaves (pasted output)
            ("(pers :p 3 :num sg :case gen :g m)", "є҆гѡ̀"),
            ("(pers :p 3 :num sg :case gen :g f)", "є҆ѧ̀"),
            ("(pers :p 1 :num sg :case gen)", "менє̀"),
            ("(part нестѝ :t pres :voice act :series short :case nom :num sg :g m)", "несы́й"),
            ("(part нестѝ :t pres :voice pass :series long :case nom :num sg :g f)", "несомаѧ"),
            ("(adj вели́кїй :case nom :num sg :g m :deg sup)", "превели́кий"),
        ] {
            assert_eq!(render(&read(tree), &syn).expect("renders"), expect, "{tree}");
        }
        // an unlisted function word refuses to render
        assert!(render(&Node::Fn("гдⷭ҇ь".into()), &syn).is_err());
        // v1.2: the Synodal non-personal pronouns render (pasted output);
        // a lemma outside the tables and the rule still refuses loudly
        assert_eq!(
            render(&read("(pn сво́й :case gen :num sg :g f)"), &syn).expect("renders"),
            "своеѧ̀"
        );
        assert_eq!(
            render(&read("(pn и҆́же :case nom :num pl :g f)"), &syn).expect("renders"),
            "ꙗ҆̀же"
        );
        assert!(render(&read("(pn нѣ́тъ :case gen :num sg :g f)"), &syn).is_err());
    }

    /// The first real lift: Genesis 1:1, byte-checked against the print.
    /// «бг҃ъ» stays verbatim-with-reason (titlo abbreviation — the crate
    /// tables spell the full «бо́гъ»); everything else is analyzed or
    /// closed-class.
    #[test]
    fn genesis_1_1_lifts_and_round_trips() {
        let text = r#"(s (cl
            (pp (cap (f въ)) (n нача́ло :case loc :num sg))
            (v сотвори́ти :t aor :p 3 :num sg)
            (np (w "бг҃ъ" :lemma бо́гъ :case nom :num sg))
            (np (n не́бо :case acc :num sg)
                (f и҆)
                (n землѧ̀ :case acc :num sg))
            (p ".")))"#;
        let node = from_sexpr(&sexpr::parse(text).expect("sexpr")).expect("node");
        assert_eq!(
            render(&node, &church_slavonic::Recension::Synodal).expect("renders"),
            "Въ нача́лѣ сотворѝ бг҃ъ не́бо и҆ зе́млю."
        );
    }

    #[test]
    fn punctuation_glue() {
        let tree = Node::Group {
            head: "s".to_string(),
            children: vec![
                Node::W { surface: "а҆́зъ".into(), notes: vec![] },
                Node::Punct(",".into()),
                Node::W { surface: "ты̀".into(), notes: vec![] },
                Node::Punct(".".into()),
            ],
        };
        assert_eq!(
            render(&tree, &church_slavonic::Recension::Synodal).expect("renders"),
            "а҆́зъ, ты̀."
        );
    }
}
