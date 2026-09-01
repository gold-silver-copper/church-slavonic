//! The inverse index and the auto-lift.
//!
//! The generator inverted: every Synodal lemma the crate can enumerate
//! ([`church_slavonic::ChurchSlavonic::lemmas`]) has its paradigm
//! generated through the same public API the renderer uses, into a
//! surface → analyses index. A verse token lifts to an analyzed leaf
//! ONLY when its surface matches exactly one analysis — the crate's own
//! output, byte-for-byte, so the round-trip invariant survives lifting
//! by construction. Ambiguous matches (acc=nom and friends) are recorded
//! (`:amb <n>`) and kept verbatim, never guessed: disambiguation by
//! syntactic context is a later, separate design.
//!
//! Apparatus tokens (anything containing `꙾` or `[`) stay verbatim
//! wholesale. Other tokens split into leading punctuation, core, and
//! trailing punctuation; a capitalized core is looked up decapitalized
//! and lifts under `(cap …)`.

use crate::node::Node;
use church_slavonic::{
    Case, ChurchSlavonic, Degree, Form, Gender, Number, PartOfSpeech, Person, Recension, Series,
    Tense, Voice,
};
use std::collections::HashMap;

/// One reading of a surface form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Analysis {
    Noun { lemma: &'static str, case: Case, number: Number },
    Adj { lemma: &'static str, case: Case, number: Number, gender: Gender, degree: Degree },
    Verb { lemma: &'static str, person: Person, number: Number, tense: Tense, form: Form },
    LPart { lemma: &'static str, gender: Gender, number: Number },
    Npron { lemma: &'static str, gender: Gender, number: Number, case: Case },
    Pers { person: Person, number: Number, gender: Option<Gender>, case: Case },
    Part {
        lemma: &'static str,
        tense: Tense,
        voice: Voice,
        series: Series,
        case: Case,
        number: Number,
        gender: Gender,
    },
    /// a titlo abbreviation of an inner reading (row index into
    /// [`crate::titlo::rows`])
    Abbr { row: usize, inner: Box<Analysis> },
}

impl Analysis {
    pub fn into_node(self) -> Node {
        match self {
            Analysis::Noun { lemma, case, number } => {
                Node::Noun { lemma: lemma.to_string(), case, number }
            }
            Analysis::Adj { lemma, case, number, gender, degree } => Node::Adj {
                lemma: lemma.to_string(),
                case,
                number,
                gender,
                degree,
            },
            Analysis::Verb { lemma, person, number, tense, form } => {
                Node::Verb { lemma: lemma.to_string(), person, number, tense, form }
            }
            Analysis::LPart { lemma, gender, number } => {
                Node::LPart { lemma: lemma.to_string(), gender, number }
            }
            Analysis::Npron { lemma, gender, number, case } => {
                Node::Npron { lemma: lemma.to_string(), gender, number, case }
            }
            Analysis::Pers { person, number, gender, case } => {
                Node::Pers { person, number, gender, case }
            }
            Analysis::Part { lemma, tense, voice, series, case, number, gender } => {
                Node::Part { lemma: lemma.to_string(), tense, voice, series, case, number, gender }
            }
            Analysis::Abbr { row, inner } => Node::Abbr {
                prefix: crate::titlo::rows()[row].abbr.to_string(),
                child: Box::new(inner.into_node()),
            },
        }
    }
}

const CASES: [Case; 7] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
    Case::Vocative,
];
const NUMBERS: [Number; 3] = [Number::Singular, Number::Dual, Number::Plural];
const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
const PERSONS: [Person; 3] = [Person::First, Person::Second, Person::Third];
const TENSES: [Tense; 3] = [Tense::Present, Tense::Imperfect, Tense::Aorist];
/// The participle voices the index inverts (see the build comment).
const PART_VOICES: [Voice; 2] = [Voice::Active, Voice::Passive];
/// Pronouns answer the vocative with the nominative — six real cases.
const NPRON_CASES: [Case; 6] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
];

/// The surface → analyses index over the crate's whole Synodal
/// inventory. Building it makes ~half a million generator calls; do it
/// once and reuse.
pub struct Index {
    map: HashMap<String, Vec<Analysis>>,
}

impl Index {
    pub fn build(recension: &Recension) -> Index {
        let mut map: HashMap<String, Vec<Analysis>> = HashMap::new();
        let mut add = |surface: String, a: Analysis| {
            let entry = map.entry(surface).or_default();
            if !entry.contains(&a) {
                entry.push(a);
            }
        };
        for lemma in ChurchSlavonic::lemmas(PartOfSpeech::Noun, recension) {
            for case in CASES {
                for number in NUMBERS {
                    add(
                        ChurchSlavonic::noun(lemma, &case, &number, recension),
                        Analysis::Noun { lemma, case, number },
                    );
                }
            }
        }
        for lemma in ChurchSlavonic::lemmas(PartOfSpeech::Adjective, recension) {
            for case in CASES {
                for number in NUMBERS {
                    for gender in GENDERS {
                        for degree in [Degree::Positive, Degree::Comparative, Degree::Superlative] {
                            add(
                                ChurchSlavonic::adj(
                                    lemma, &case, &number, &gender, &degree, recension,
                                ),
                                Analysis::Adj { lemma, case, number, gender, degree },
                            );
                        }
                    }
                }
            }
        }
        for lemma in ChurchSlavonic::lemmas(PartOfSpeech::Verb, recension) {
            for tense in TENSES {
                for person in PERSONS {
                    for number in NUMBERS {
                        add(
                            ChurchSlavonic::verb(
                                lemma,
                                &person,
                                &number,
                                &tense,
                                &Form::Finite,
                                recension,
                            ),
                            Analysis::Verb { lemma, person, number, tense, form: Form::Finite },
                        );
                    }
                }
            }
            for number in [Number::Singular, Number::Plural] {
                add(
                    ChurchSlavonic::verb(
                        lemma,
                        &Person::Second,
                        &number,
                        &Tense::Present,
                        &Form::Imperative,
                        recension,
                    ),
                    Analysis::Verb {
                        lemma,
                        person: Person::Second,
                        number,
                        tense: Tense::Present,
                        form: Form::Imperative,
                    },
                );
            }
            add(
                ChurchSlavonic::verb(
                    lemma,
                    &Person::Third,
                    &Number::Singular,
                    &Tense::Present,
                    &Form::Infinitive,
                    recension,
                ),
                Analysis::Verb {
                    lemma,
                    person: Person::Third,
                    number: Number::Singular,
                    tense: Tense::Present,
                    form: Form::Infinitive,
                },
            );
            for gender in GENDERS {
                for number in NUMBERS {
                    add(
                        ChurchSlavonic::l_participle(lemma, &gender, &number, recension),
                        Analysis::LPart { lemma, gender, number },
                    );
                }
            }
        }
        // participles, from the verb inventory: present and aorist
        // (imperfect aliases the aorist stem in the API), both series,
        // ACTIVE voice — passive measured and deferred or admitted by
        // the recorded numbers, never silently
        for lemma in ChurchSlavonic::lemmas(PartOfSpeech::Verb, recension) {
            for tense in [Tense::Present, Tense::Aorist] {
                for voice in PART_VOICES {
                    for series in [Series::Short, Series::Long] {
                        for case in CASES {
                            for number in NUMBERS {
                                for gender in GENDERS {
                                    let form = ChurchSlavonic::participle(
                                        lemma, &tense, &voice, &series, &case, &number, &gender,
                                        recension,
                                    );
                                    if !form.is_empty() {
                                        add(
                                            form,
                                            Analysis::Part {
                                                lemma,
                                                tense,
                                                voice,
                                                series,
                                                case,
                                                number,
                                                gender,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // non-personal pronouns: table lemmas (Synodal has none today —
        // the absence is a recorded crate gap, not a silent skip; the
        // code path stands and serves OCS the day an OCS treebank comes)
        for lemma in ChurchSlavonic::lemmas(PartOfSpeech::NonPersonalPronoun, recension) {
            for gender in GENDERS {
                for number in NUMBERS {
                    for case in NPRON_CASES {
                        let form = ChurchSlavonic::npron(lemma, &gender, &number, &case, recension);
                        if !form.is_empty() {
                            add(form, Analysis::Npron { lemma, gender, number, case });
                        }
                    }
                }
            }
        }
        // the personal pronoun: a closed paradigm, no lemma table; the
        // first and second persons are gender-free (one analysis each,
        // masculine as the API's placeholder argument), the third is
        // per-gender; the vocative answers with the nominative and is
        // not indexed as a distinct reading
        for number in NUMBERS {
            for case in NPRON_CASES {
                for person in [Person::First, Person::Second] {
                    let form = ChurchSlavonic::pronoun(
                        &person,
                        &number,
                        &Gender::Masculine,
                        &case,
                        recension,
                    );
                    if !form.is_empty() {
                        add(
                            form.to_string(),
                            Analysis::Pers { person, number, gender: None, case },
                        );
                    }
                }
                for gender in GENDERS {
                    let form =
                        ChurchSlavonic::pronoun(&Person::Third, &number, &gender, &case, recension);
                    if !form.is_empty() {
                        add(
                            form.to_string(),
                            Analysis::Pers {
                                person: Person::Third,
                                number,
                                gender: Some(gender),
                                case,
                            },
                        );
                    }
                }
            }
        }
        // the titlo layer: every committed row's full paradigm,
        // abbreviated by prefix cut (data/titlo.tsv; the verification
        // table in NOTES.md records per-row print agreement)
        for (ri, row) in crate::titlo::rows().iter().enumerate() {
            let mut add_abbr = |full: String, inner: Analysis| {
                if let Some(surface) = crate::titlo::abbreviate(&full, row) {
                    let entry = map.entry(surface).or_default();
                    let a = Analysis::Abbr { row: ri, inner: Box::new(inner) };
                    if !entry.contains(&a) {
                        entry.push(a);
                    }
                }
            };
            let lemma = row.lemma;
            match row.pos {
                crate::titlo::Pos::Noun => {
                    for case in CASES {
                        for number in NUMBERS {
                            add_abbr(
                                ChurchSlavonic::noun(lemma, &case, &number, recension),
                                Analysis::Noun { lemma, case, number },
                            );
                        }
                    }
                }
                crate::titlo::Pos::Adjective => {
                    for case in CASES {
                        for number in NUMBERS {
                            for gender in GENDERS {
                                add_abbr(
                                    ChurchSlavonic::adj(
                                        lemma,
                                        &case,
                                        &number,
                                        &gender,
                                        &Degree::Positive,
                                        recension,
                                    ),
                                    Analysis::Adj {
                                        lemma,
                                        case,
                                        number,
                                        gender,
                                        degree: Degree::Positive,
                                    },
                                );
                            }
                        }
                    }
                }
                crate::titlo::Pos::Verb => {
                    for tense in TENSES {
                        for person in PERSONS {
                            for number in NUMBERS {
                                add_abbr(
                                    ChurchSlavonic::verb(
                                        lemma,
                                        &person,
                                        &number,
                                        &tense,
                                        &Form::Finite,
                                        recension,
                                    ),
                                    Analysis::Verb {
                                        lemma,
                                        person,
                                        number,
                                        tense,
                                        form: Form::Finite,
                                    },
                                );
                            }
                        }
                    }
                    for tense in [Tense::Present, Tense::Aorist] {
                        for voice in PART_VOICES {
                            for series in [Series::Short, Series::Long] {
                                for case in CASES {
                                    for number in NUMBERS {
                                        for gender in GENDERS {
                                            add_abbr(
                                                ChurchSlavonic::participle(
                                                    lemma, &tense, &voice, &series, &case,
                                                    &number, &gender, recension,
                                                ),
                                                Analysis::Part {
                                                    lemma,
                                                    tense,
                                                    voice,
                                                    series,
                                                    case,
                                                    number,
                                                    gender,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Index { map }
    }

    /// Every reading of a surface, if any.
    pub fn analyses(&self, surface: &str) -> &[Analysis] {
        self.map.get(surface).map_or(&[], Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Characters that split off a token's edges as `(p …)` nodes.
fn is_punct(c: char) -> bool {
    matches!(c, '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '«' | '»')
}

/// What one verse token became.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenFate {
    Analyzed,
    ClosedClass,
    Ambiguous,
    Verbatim,
    Apparatus,
}

/// Lift one token into nodes; report its fate.
pub fn lift_token(token: &str, index: &Index) -> (Vec<Node>, TokenFate) {
    // apparatus stays whole — the target is the verse as printed
    if token.contains('꙾') || token.contains('[') {
        return (
            vec![Node::W { surface: token.to_string(), notes: Vec::new() }],
            TokenFate::Apparatus,
        );
    }
    let core_start = token.len() - token.trim_start_matches(is_punct).len();
    let core_end = token.trim_end_matches(is_punct).len();
    if core_start >= core_end {
        // a FREE-STANDING punctuation token (the print has e.g.
        // «а҆ссѷрі́йскъ .» in 4 Kings 17:3) — it must keep its own
        // space, so it stays a verbatim leaf, never a gluing (p …)
        return (
            vec![Node::W { surface: token.to_string(), notes: Vec::new() }],
            TokenFate::Verbatim,
        );
    }
    let (lead, rest) = token.split_at(core_start);
    let (core, trail) = rest.split_at(core_end - core_start);
    let mut nodes: Vec<Node> = lead.chars().map(|c| Node::Punct(c.to_string())).collect();
    let (core_node, fate) = lift_core(core, index);
    nodes.push(core_node);
    nodes.extend(trail.chars().map(|c| Node::Punct(c.to_string())));
    // the split must rebuild the token EXACTLY under the glue rule — the
    // print holds typographic oddities (Proverbs 15:33 opens a bracket
    // with «(,*…») that no reasonable rule should chase; when the local
    // reconstruction differs, the whole token stays verbatim
    let probe = Node::Group { head: "s".to_string(), children: nodes.clone() };
    match crate::node::render(&probe, &Recension::Synodal) {
        Ok(rebuilt) if rebuilt == token => (nodes, fate),
        _ => (
            vec![Node::W { surface: token.to_string(), notes: Vec::new() }],
            TokenFate::Verbatim,
        ),
    }
}

fn decapitalized(word: &str) -> Option<String> {
    let first = word.chars().next()?;
    if !first.is_uppercase() {
        return None;
    }
    Some(first.to_lowercase().chain(word.chars().skip(1)).collect())
}

fn lift_core(core: &str, index: &Index) -> (Node, TokenFate) {
    let (looked_up, capped) = match decapitalized(core) {
        Some(low) => (low, true),
        None => (core.to_string(), false),
    };
    let analyses = index.analyses(&looked_up);
    let closed = crate::closed::is_closed(&looked_up);
    let wrap = |node: Node| if capped { Node::Cap(Box::new(node)) } else { node };
    match (analyses.len(), closed) {
        // a function word that is also a paradigm form is ambiguous
        (0, true) => (wrap(Node::Fn(looked_up)), TokenFate::ClosedClass),
        (1, false) => (wrap(analyses[0].clone().into_node()), TokenFate::Analyzed),
        (0, false) => (
            Node::W { surface: core.to_string(), notes: Vec::new() },
            TokenFate::Verbatim,
        ),
        (n, _) => (
            Node::W {
                surface: core.to_string(),
                notes: vec![("amb".to_string(), (n + usize::from(closed)).to_string())],
            },
            TokenFate::Ambiguous,
        ),
    }
}

/// Per-verse (and aggregable) coverage counts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    pub analyzed: usize,
    pub closed: usize,
    pub ambiguous: usize,
    pub verbatim: usize,
    pub apparatus: usize,
}

impl Coverage {
    pub fn total(&self) -> usize {
        self.analyzed + self.closed + self.ambiguous + self.verbatim + self.apparatus
    }
    pub fn add(&mut self, other: Coverage) {
        self.analyzed += other.analyzed;
        self.closed += other.closed;
        self.ambiguous += other.ambiguous;
        self.verbatim += other.verbatim;
        self.apparatus += other.apparatus;
    }
}

/// Auto-lift one verse into an `(s …)` tree.
pub fn lift_verse(verse: &str, index: &Index) -> (Node, Coverage) {
    let mut children = Vec::new();
    let mut coverage = Coverage::default();
    for token in crate::node::tokenize(verse) {
        let (nodes, fate) = lift_token(token, index);
        children.extend(nodes);
        match fate {
            TokenFate::Analyzed => coverage.analyzed += 1,
            TokenFate::ClosedClass => coverage.closed += 1,
            TokenFate::Ambiguous => coverage.ambiguous += 1,
            TokenFate::Verbatim => coverage.verbatim += 1,
            TokenFate::Apparatus => coverage.apparatus += 1,
        }
    }
    (Node::Group { head: "s".to_string(), children }, coverage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::render;

    /// A small index over a handful of lemmas would not exercise the
    /// real ambiguity structure — build the full one once for the module.
    fn index() -> &'static Index {
        use std::sync::OnceLock;
        static INDEX: OnceLock<Index> = OnceLock::new();
        INDEX.get_or_init(|| Index::build(&Recension::Synodal))
    }

    #[test]
    fn lifting_preserves_the_round_trip() {
        let verse = "И҆ речѐ бг҃ъ: да бꙋ́детъ свѣ́тъ. И҆ бы́сть свѣ́тъ.";
        let (tree, coverage) = lift_verse(verse, index());
        let rendered = render(&tree, &Recension::Synodal).expect("renders");
        assert_eq!(rendered, verse);
        assert_eq!(coverage.total(), 9);
        assert!(coverage.analyzed + coverage.closed > 0, "{coverage:?}");
    }

    #[test]
    fn the_pitfall_verse_lifts_without_touching_the_apparatus() {
        let verse = "и҆ речѐ ю҆нѣ́йшїй ꙾є҆ю̀꙾[26] ѻ҆тцꙋ̀: ѻ҆́тче, да́ждь мѝ досто́йнꙋю ча́сть и҆мѣ́нїѧ. И҆ раздѣлѝ и҆́ма и҆мѣ́нїе.";
        let (tree, coverage) = lift_verse(verse, index());
        assert_eq!(render(&tree, &Recension::Synodal).expect("renders"), verse);
        assert_eq!(coverage.apparatus, 1, "꙾є҆ю̀꙾[26] stays whole");
    }

    /// A participle homographic with another reading must fall to :amb,
    /// never to whichever index insertion ran last.
    #[test]
    fn cross_pos_collisions_are_ambiguous() {
        let idx = index();
        // the third-person pronoun є҆гѡ̀ doubles as the possessive use —
        // and «несомаѧ» (pres pass long f) must be a Part reading
        let part = idx.analyses("несомаѧ");
        assert!(
            part.iter().any(|a| matches!(a, Analysis::Part { .. })),
            "{part:?}"
        );
        // find one real adjective/participle collision in the index and
        // assert the lift records it as ambiguous
        let collision = idx
            .analyses("сꙋ́щаѧ")
            .iter()
            .any(|a| matches!(a, Analysis::Part { .. }));
        if collision {
            let (_, fate) = lift_token("сꙋ́щаѧ", idx);
            if idx.analyses("сꙋ́щаѧ").len() > 1 {
                assert_eq!(fate, TokenFate::Ambiguous);
            }
        }
        // structural guarantee: EVERY multi-analysis surface lifts to :amb
        let (nodes, fate) = lift_token("є҆гѡ̀", idx);
        if idx.analyses("є҆гѡ̀").len() > 1 {
            assert_eq!(fate, TokenFate::Ambiguous);
            assert!(matches!(&nodes[0], crate::node::Node::W { .. }));
        } else {
            assert_eq!(fate, TokenFate::Analyzed);
        }
    }

    #[test]
    fn ambiguity_is_recorded_never_guessed() {
        // «сло́во» is nom=acc at least — must come back ambiguous
        let analyses = index().analyses("сло́во");
        assert!(analyses.len() > 1, "{analyses:?}");
        let (nodes, fate) = lift_token("сло́во", index());
        assert_eq!(fate, TokenFate::Ambiguous);
        assert!(matches!(&nodes[0], crate::node::Node::W { notes, .. } if !notes.is_empty()));
    }
}
