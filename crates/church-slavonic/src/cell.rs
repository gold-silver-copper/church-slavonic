//! Typed cells: WHICH form of a lexeme is asked for. Every cell has one
//! canonical name (the lexicon's `overrides`/`variants` columns, the
//! treebank leaves and the eval reports all use it) and parses back from
//! it, so `parse(name()) == Some(cell)` for every cell.
//!
//! Cell-name grammar:
//!
//! - noun: `nom|gen|dat|acc|ins|loc|voc . sg|du|pl` — `gen.pl`;
//! - adjective: `[short|long .] pos|comp|sup . m|f|n . sg|du|pl . case` —
//!   `pos.m.sg.nom`, `short.pos.f.pl.acc` (the series prefix only where
//!   the class has both series);
//! - verb: `pres|impf|aor|fut . 1|2|3 . sg|du|pl` — `aor.3.pl`; `impv.2.sg`;
//!   `inf`; `lpart . m|f|n . sg|du|pl`; `part . pres|past . act|pass .
//!   short|long . gender . number . case` — `part.pres.act.short.m.sg.nom`;
//! - pronoun: `[clit .] [1|2|3 .] [m|f|n .] [sg|du|pl .] case` — the
//!   personal pronoun's `1.sg.nom` / `3.m.sg.gen` / `clit.1.sg.dat`, a
//!   non-personal pronoun's `m.sg.gen`, the reflexive's `dat` / `clit.acc`.

use crate::grammar::*;
use std::fmt;

/// Parts of speech the lexicon inflects (plus the uninflected closed
/// classes, which the analyzer still resolves).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pos {
    Noun,
    Adjective,
    Verb,
    Pronoun,
    /// Adverbs, conjunctions, prepositions, particles, numerals: one form.
    Closed,
}

impl Pos {
    /// The id suffix (`рабъ.n`) and the lexicon's `pos` column.
    pub fn tag(self) -> &'static str {
        match self {
            Pos::Noun => "n",
            Pos::Adjective => "a",
            Pos::Verb => "v",
            Pos::Pronoun => "pron",
            Pos::Closed => "x",
        }
    }
    pub fn parse(tag: &str) -> Option<Pos> {
        Some(match tag {
            "n" => Pos::Noun,
            "a" => Pos::Adjective,
            "v" => Pos::Verb,
            "pron" => Pos::Pronoun,
            "x" => Pos::Closed,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NounCell {
    pub case: Case,
    pub number: Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AdjCell {
    /// `None` where the class has one series only.
    pub series: Option<Series>,
    pub degree: Degree,
    pub gender: Gender,
    pub number: Number,
    pub case: Case,
}

/// The finite tenses a verb class may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiniteTense {
    Present,
    Imperfect,
    Aorist,
    /// The synthetic future of бы́ти (бꙋ́дꙋ); most classes do not declare it.
    Future,
}

/// A participle's tense: present or past.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PartTense {
    Present,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VerbCell {
    Finite { tense: FiniteTense, person: Person, number: Number },
    Imperative { person: Person, number: Number },
    Infinitive,
    /// The l-participle (resultative), nominative only.
    LPart { gender: Gender, number: Number },
    Participle {
        tense: PartTense,
        voice: Voice,
        series: Series,
        gender: Gender,
        number: Number,
        case: Case,
    },
}

/// A pronoun cell. The personal pronoun sets `person` (and `gender` in the
/// third person); a non-personal pronoun sets `gender`; the reflexive sets
/// neither person nor number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PronCell {
    pub clitic: bool,
    pub person: Option<Person>,
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub case: Case,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cell {
    Noun(NounCell),
    Adj(AdjCell),
    Verb(VerbCell),
    Pron(PronCell),
}

impl From<NounCell> for Cell {
    fn from(c: NounCell) -> Cell {
        Cell::Noun(c)
    }
}
impl From<AdjCell> for Cell {
    fn from(c: AdjCell) -> Cell {
        Cell::Adj(c)
    }
}
impl From<VerbCell> for Cell {
    fn from(c: VerbCell) -> Cell {
        Cell::Verb(c)
    }
}
impl From<PronCell> for Cell {
    fn from(c: PronCell) -> Cell {
        Cell::Pron(c)
    }
}

// ---------------------------------------------------------------------------
// Atom vocabularies
// ---------------------------------------------------------------------------

pub const CASES: [Case; 7] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
    Case::Vocative,
];
pub const NUMBERS: [Number; 3] = [Number::Singular, Number::Dual, Number::Plural];
pub const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
pub const PERSONS: [Person; 3] = [Person::First, Person::Second, Person::Third];

pub fn case_name(c: Case) -> &'static str {
    match c {
        Case::Nominative => "nom",
        Case::Genitive => "gen",
        Case::Dative => "dat",
        Case::Accusative => "acc",
        Case::Instrumental => "ins",
        Case::Locative => "loc",
        Case::Vocative => "voc",
    }
}
pub fn parse_case(s: &str) -> Option<Case> {
    CASES.into_iter().find(|c| case_name(*c) == s)
}
pub fn number_name(n: Number) -> &'static str {
    match n {
        Number::Singular => "sg",
        Number::Dual => "du",
        Number::Plural => "pl",
    }
}
pub fn parse_number(s: &str) -> Option<Number> {
    NUMBERS.into_iter().find(|n| number_name(*n) == s)
}
pub fn gender_name(g: Gender) -> &'static str {
    match g {
        Gender::Masculine => "m",
        Gender::Feminine => "f",
        Gender::Neuter => "n",
    }
}
pub fn parse_gender(s: &str) -> Option<Gender> {
    GENDERS.into_iter().find(|g| gender_name(*g) == s)
}
pub fn person_name(p: Person) -> &'static str {
    match p {
        Person::First => "1",
        Person::Second => "2",
        Person::Third => "3",
    }
}
pub fn parse_person(s: &str) -> Option<Person> {
    PERSONS.into_iter().find(|p| person_name(*p) == s)
}
fn degree_name(d: Degree) -> &'static str {
    match d {
        Degree::Positive => "pos",
        Degree::Comparative => "comp",
        Degree::Superlative => "sup",
    }
}
fn parse_degree(s: &str) -> Option<Degree> {
    Some(match s {
        "pos" => Degree::Positive,
        "comp" => Degree::Comparative,
        "sup" => Degree::Superlative,
        _ => return None,
    })
}
fn series_name(s: Series) -> &'static str {
    match s {
        Series::Short => "short",
        Series::Long => "long",
    }
}
fn parse_series(s: &str) -> Option<Series> {
    Some(match s {
        "short" => Series::Short,
        "long" => Series::Long,
        _ => return None,
    })
}
fn finite_name(t: FiniteTense) -> &'static str {
    match t {
        FiniteTense::Present => "pres",
        FiniteTense::Imperfect => "impf",
        FiniteTense::Aorist => "aor",
        FiniteTense::Future => "fut",
    }
}
fn parse_finite(s: &str) -> Option<FiniteTense> {
    Some(match s {
        "pres" => FiniteTense::Present,
        "impf" => FiniteTense::Imperfect,
        "aor" => FiniteTense::Aorist,
        "fut" => FiniteTense::Future,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------

impl NounCell {
    pub fn new(case: Case, number: Number) -> NounCell {
        NounCell { case, number }
    }
    pub fn name(&self) -> String {
        format!("{}.{}", case_name(self.case), number_name(self.number))
    }
    pub fn parse(s: &str) -> Option<NounCell> {
        let (c, n) = s.split_once('.')?;
        Some(NounCell { case: parse_case(c)?, number: parse_number(n)? })
    }
    /// Every noun cell, in schema order (number-major).
    pub fn all() -> impl Iterator<Item = NounCell> {
        NUMBERS
            .into_iter()
            .flat_map(|number| CASES.into_iter().map(move |case| NounCell { case, number }))
    }
}

impl AdjCell {
    pub fn name(&self) -> String {
        let mut out = String::new();
        if let Some(series) = self.series {
            out.push_str(series_name(series));
            out.push('.');
        }
        out.push_str(&format!(
            "{}.{}.{}.{}",
            degree_name(self.degree),
            gender_name(self.gender),
            number_name(self.number),
            case_name(self.case)
        ));
        out
    }
    pub fn parse(s: &str) -> Option<AdjCell> {
        let parts: Vec<&str> = s.split('.').collect();
        let (series, rest) = match parts.as_slice() {
            [series, rest @ ..] if parse_series(series).is_some() => {
                (parse_series(series), rest)
            }
            rest => (None, rest),
        };
        let [d, g, n, c] = rest else { return None };
        Some(AdjCell {
            series,
            degree: parse_degree(d)?,
            gender: parse_gender(g)?,
            number: parse_number(n)?,
            case: parse_case(c)?,
        })
    }
}

impl VerbCell {
    pub fn name(&self) -> String {
        match self {
            VerbCell::Finite { tense, person, number } => format!(
                "{}.{}.{}",
                finite_name(*tense),
                person_name(*person),
                number_name(*number)
            ),
            VerbCell::Imperative { person, number } => {
                format!("impv.{}.{}", person_name(*person), number_name(*number))
            }
            VerbCell::Infinitive => "inf".to_string(),
            VerbCell::LPart { gender, number } => {
                format!("lpart.{}.{}", gender_name(*gender), number_name(*number))
            }
            VerbCell::Participle { tense, voice, series, gender, number, case } => format!(
                "part.{}.{}.{}.{}.{}.{}",
                match tense {
                    PartTense::Present => "pres",
                    PartTense::Past => "past",
                },
                match voice {
                    Voice::Active => "act",
                    Voice::Passive => "pass",
                },
                series_name(*series),
                gender_name(*gender),
                number_name(*number),
                case_name(*case)
            ),
        }
    }
    pub fn parse(s: &str) -> Option<VerbCell> {
        let parts: Vec<&str> = s.split('.').collect();
        Some(match parts.as_slice() {
            ["inf"] => VerbCell::Infinitive,
            ["impv", p, n] => VerbCell::Imperative { person: parse_person(p)?, number: parse_number(n)? },
            ["lpart", g, n] => VerbCell::LPart { gender: parse_gender(g)?, number: parse_number(n)? },
            ["part", t, v, s, g, n, c] => VerbCell::Participle {
                tense: match *t {
                    "pres" => PartTense::Present,
                    "past" => PartTense::Past,
                    _ => return None,
                },
                voice: match *v {
                    "act" => Voice::Active,
                    "pass" => Voice::Passive,
                    _ => return None,
                },
                series: parse_series(s)?,
                gender: parse_gender(g)?,
                number: parse_number(n)?,
                case: parse_case(c)?,
            },
            [t, p, n] => VerbCell::Finite {
                tense: parse_finite(t)?,
                person: parse_person(p)?,
                number: parse_number(n)?,
            },
            _ => return None,
        })
    }
}

impl PronCell {
    pub fn name(&self) -> String {
        let mut out = String::new();
        if self.clitic {
            out.push_str("clit.");
        }
        if let Some(p) = self.person {
            out.push_str(person_name(p));
            out.push('.');
        }
        if let Some(g) = self.gender {
            out.push_str(gender_name(g));
            out.push('.');
        }
        if let Some(n) = self.number {
            out.push_str(number_name(n));
            out.push('.');
        }
        out.push_str(case_name(self.case));
        out
    }
    pub fn parse(s: &str) -> Option<PronCell> {
        let mut parts: Vec<&str> = s.split('.').collect();
        let case = parse_case(parts.pop()?)?;
        let mut cell = PronCell { clitic: false, person: None, gender: None, number: None, case };
        let mut it = parts.into_iter().peekable();
        if it.peek() == Some(&"clit") {
            cell.clitic = true;
            it.next();
        }
        if let Some(p) = it.peek().and_then(|s| parse_person(s)) {
            cell.person = Some(p);
            it.next();
        }
        if let Some(g) = it.peek().and_then(|s| parse_gender(s)) {
            cell.gender = Some(g);
            it.next();
        }
        if let Some(n) = it.peek().and_then(|s| parse_number(s)) {
            cell.number = Some(n);
            it.next();
        }
        if it.next().is_some() {
            return None;
        }
        Some(cell)
    }
}

impl Cell {
    pub fn name(&self) -> String {
        match self {
            Cell::Noun(c) => c.name(),
            Cell::Adj(c) => c.name(),
            Cell::Verb(c) => c.name(),
            Cell::Pron(c) => c.name(),
        }
    }
    /// Parse a cell name in the part of speech's grammar.
    pub fn parse(pos: Pos, s: &str) -> Option<Cell> {
        Some(match pos {
            Pos::Noun => Cell::Noun(NounCell::parse(s)?),
            Pos::Adjective => Cell::Adj(AdjCell::parse(s)?),
            Pos::Verb => Cell::Verb(VerbCell::parse(s)?),
            Pos::Pronoun => Cell::Pron(PronCell::parse(s)?),
            Pos::Closed => return None,
        })
    }
    /// The cell's number, where it has one.
    pub fn number(&self) -> Option<Number> {
        match self {
            Cell::Noun(c) => Some(c.number),
            Cell::Adj(c) => Some(c.number),
            Cell::Verb(VerbCell::Finite { number, .. })
            | Cell::Verb(VerbCell::Imperative { number, .. })
            | Cell::Verb(VerbCell::LPart { number, .. })
            | Cell::Verb(VerbCell::Participle { number, .. }) => Some(*number),
            Cell::Verb(VerbCell::Infinitive) => None,
            Cell::Pron(c) => c.number,
        }
    }

    pub fn pos(&self) -> Pos {
        match self {
            Cell::Noun(_) => Pos::Noun,
            Cell::Adj(_) => Pos::Adjective,
            Cell::Verb(_) => Pos::Verb,
            Cell::Pron(_) => Pos::Pronoun,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for cell in NounCell::all() {
            assert_eq!(NounCell::parse(&cell.name()), Some(cell));
        }
        let adj = AdjCell {
            series: Some(Series::Short),
            degree: Degree::Positive,
            gender: Gender::Feminine,
            number: Number::Plural,
            case: Case::Accusative,
        };
        assert_eq!(adj.name(), "short.pos.f.pl.acc");
        assert_eq!(AdjCell::parse("short.pos.f.pl.acc"), Some(adj));
        assert_eq!(AdjCell::parse("comp.n.sg.nom").map(|c| c.series), Some(None));
        for v in [
            VerbCell::Finite { tense: FiniteTense::Aorist, person: Person::Third, number: Number::Plural },
            VerbCell::Imperative { person: Person::Second, number: Number::Singular },
            VerbCell::Infinitive,
            VerbCell::LPart { gender: Gender::Neuter, number: Number::Dual },
            VerbCell::Participle {
                tense: PartTense::Past,
                voice: Voice::Passive,
                series: Series::Long,
                gender: Gender::Masculine,
                number: Number::Singular,
                case: Case::Genitive,
            },
        ] {
            assert_eq!(VerbCell::parse(&v.name()), Some(v), "{}", v.name());
        }
        assert_eq!(
            VerbCell::Finite { tense: FiniteTense::Aorist, person: Person::Third, number: Number::Plural }.name(),
            "aor.3.pl"
        );
        for p in [
            PronCell { clitic: false, person: Some(Person::First), gender: None, number: Some(Number::Singular), case: Case::Nominative },
            PronCell { clitic: true, person: Some(Person::Third), gender: Some(Gender::Feminine), number: Some(Number::Plural), case: Case::Accusative },
            PronCell { clitic: false, person: None, gender: Some(Gender::Masculine), number: Some(Number::Singular), case: Case::Genitive },
            PronCell { clitic: false, person: None, gender: None, number: None, case: Case::Dative },
            PronCell { clitic: true, person: None, gender: None, number: None, case: Case::Accusative },
        ] {
            assert_eq!(PronCell::parse(&p.name()), Some(p), "{}", p.name());
        }
        assert_eq!(PronCell::parse("3.m.sg.gen").map(|c| c.name()).as_deref(), Some("3.m.sg.gen"));
        assert_eq!(PronCell::parse("clit.dat").map(|c| c.clitic), Some(true));
        assert_eq!(PronCell::parse("sg.3.nom"), None, "order is fixed");
        assert_eq!(Cell::parse(Pos::Noun, "gen.pl").map(|c| c.name()).as_deref(), Some("gen.pl"));
        assert_eq!(Cell::parse(Pos::Noun, "gen"), None);
    }
}
