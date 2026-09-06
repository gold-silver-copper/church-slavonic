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

use crate::error::CellError;
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

/// The adverb an adjective derives (мꙋ́дрѡ, до́брѣ; the comparative
/// мꙋдрѣ́е): a cell of the adjective's paradigm with no case, number or
/// gender — the print tells it from the neuter short form by the wide ѡ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AdvCell {
    pub degree: Degree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cell {
    Noun(NounCell),
    Adj(AdjCell),
    /// An adjective's adverb (`adv`, `comp.adv`).
    Adv(AdvCell),
    Verb(VerbCell),
    Pron(PronCell),
    /// The one form of an uninflected word (the closed classes).
    Word,
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
pub fn degree_name(d: Degree) -> &'static str {
    match d {
        Degree::Positive => "pos",
        Degree::Comparative => "comp",
        Degree::Superlative => "sup",
    }
}
pub fn parse_degree(s: &str) -> Option<Degree> {
    Some(match s {
        "pos" => Degree::Positive,
        "comp" => Degree::Comparative,
        "sup" => Degree::Superlative,
        _ => return None,
    })
}
pub fn series_name(s: Series) -> &'static str {
    match s {
        Series::Short => "short",
        Series::Long => "long",
    }
}
pub fn parse_series(s: &str) -> Option<Series> {
    Some(match s {
        "short" => Series::Short,
        "long" => Series::Long,
        _ => return None,
    })
}
pub fn finite_name(t: FiniteTense) -> &'static str {
    match t {
        FiniteTense::Present => "pres",
        FiniteTense::Imperfect => "impf",
        FiniteTense::Aorist => "aor",
        FiniteTense::Future => "fut",
    }
}
pub fn parse_finite(s: &str) -> Option<FiniteTense> {
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
    /// Every adjective cell with both series, in schema order.
    pub fn all() -> impl Iterator<Item = AdjCell> {
        [Series::Short, Series::Long].into_iter().flat_map(|series| {
            [Degree::Positive, Degree::Comparative].into_iter().flat_map(move |degree| {
                GENDERS.into_iter().flat_map(move |gender| {
                    NUMBERS.into_iter().flat_map(move |number| {
                        CASES.into_iter().map(move |case| AdjCell { series: Some(series), degree, gender, number, case })
                    })
                })
            })
        })
    }

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
    /// Every participle cell, in schema order.
    pub fn participles() -> impl Iterator<Item = VerbCell> {
        [PartTense::Present, PartTense::Past].into_iter().flat_map(|tense| {
            [Voice::Active, Voice::Passive].into_iter().flat_map(move |voice| {
                [Series::Short, Series::Long].into_iter().flat_map(move |series| {
                    GENDERS.into_iter().flat_map(move |gender| {
                        NUMBERS.into_iter().flat_map(move |number| {
                            CASES.into_iter().map(move |case| VerbCell::Participle { tense, voice, series, gender, number, case })
                        })
                    })
                })
            })
        })
    }

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

impl AdvCell {
    pub fn name(&self) -> String {
        match self.degree {
            Degree::Positive => "adv".to_string(),
            d => format!("{}.adv", degree_name(d)),
        }
    }
    pub fn parse(s: &str) -> Option<AdvCell> {
        match s {
            "adv" => Some(AdvCell { degree: Degree::Positive }),
            _ => {
                let (d, rest) = s.split_once('.')?;
                (rest == "adv").then(|| parse_degree(d).map(|degree| AdvCell { degree }))?
            }
        }
    }
}

impl Cell {
    pub fn name(&self) -> String {
        match self {
            Cell::Noun(c) => c.name(),
            Cell::Adj(c) => c.name(),
            Cell::Adv(c) => c.name(),
            Cell::Verb(c) => c.name(),
            Cell::Pron(c) => c.name(),
            Cell::Word => "word".to_string(),
        }
    }
    /// Parse a cell name in the part of speech's grammar — the notation of
    /// the class tables, the lexicon's `overrides`/`variants` columns and
    /// the treebank's leaves (`gen.pl`, `long.pos.m.sg.acc`, `aor.3.sg`,
    /// `part.pres.act.short.m.sg.nom`, `3.m.sg.gen`, `clit.dat`, `word`).
    ///
    /// ```
    /// use church_slavonic::{Cell, Pos, Case, Number};
    /// assert_eq!(Cell::parse(Pos::Noun, "gen.pl"), Ok(Cell::noun(Case::Genitive, Number::Plural)));
    /// assert!(Cell::parse(Pos::Noun, "gen").is_err());
    /// ```
    pub fn parse(pos: Pos, s: &str) -> Result<Cell, CellError> {
        let parsed = match pos {
            Pos::Noun => NounCell::parse(s).map(Cell::Noun),
            Pos::Adjective => match AdvCell::parse(s) {
                Some(a) => Some(Cell::Adv(a)),
                None => AdjCell::parse(s).map(Cell::Adj),
            },
            Pos::Verb => VerbCell::parse(s).map(Cell::Verb),
            Pos::Pronoun => PronCell::parse(s).map(Cell::Pron),
            Pos::Closed => (s == "word").then_some(Cell::Word),
        };
        parsed.ok_or_else(|| CellError { pos, text: s.to_string() })
    }

    // ---- typed constructors (4.0): the cells by their features, beside
    // the name parser ----

    /// A noun's cell.
    pub fn noun(case: Case, number: Number) -> Cell {
        Cell::Noun(NounCell { case, number })
    }
    /// An adjective's cell; `series` is `None` for a class with one series.
    pub fn adj(series: Option<Series>, degree: Degree, gender: Gender, number: Number, case: Case) -> Cell {
        Cell::Adj(AdjCell { series, degree, gender, number, case })
    }
    /// The adverb an adjective derives (мꙋ́дрѡ; the comparative мꙋдрѣ́е).
    pub fn adv(degree: Degree) -> Cell {
        Cell::Adv(AdvCell { degree })
    }
    /// A finite verb form.
    pub fn finite(tense: FiniteTense, person: Person, number: Number) -> Cell {
        Cell::Verb(VerbCell::Finite { tense, person, number })
    }
    /// An imperative.
    pub fn imperative(person: Person, number: Number) -> Cell {
        Cell::Verb(VerbCell::Imperative { person, number })
    }
    /// The infinitive.
    pub fn infinitive() -> Cell {
        Cell::Verb(VerbCell::Infinitive)
    }
    /// The l-participle (nominative only).
    pub fn lpart(gender: Gender, number: Number) -> Cell {
        Cell::Verb(VerbCell::LPart { gender, number })
    }
    /// A participle's cell.
    pub fn participle(tense: PartTense, voice: Voice, series: Series, gender: Gender, number: Number, case: Case) -> Cell {
        Cell::Verb(VerbCell::Participle { tense, voice, series, gender, number, case })
    }
    /// A pronoun's cell: a personal pronoun sets `person` (and `gender` in
    /// the third person), a non-personal one `gender`, the reflexive
    /// neither person nor number; `clitic` is the enclitic twin (мѧ̀, мѝ).
    pub fn pron(clitic: bool, person: Option<Person>, gender: Option<Gender>, number: Option<Number>, case: Case) -> Cell {
        Cell::Pron(PronCell { clitic, person, gender, number, case })
    }
    /// A closed lexeme's one cell.
    pub fn word() -> Cell {
        Cell::Word
    }
    /// The block a class table may address with one column instead of a
    /// cell each: an adjective's `<series>.<degree>` (`short.comp`), a
    /// participle's `part.<tense>.<voice>.<series>`; `None` elsewhere.
    pub fn block(&self) -> Option<String> {
        match self {
            Cell::Adj(c) => Some(format!(
                "{}.{}",
                c.series.map(series_name).unwrap_or("long"),
                degree_name(c.degree)
            )),
            Cell::Verb(VerbCell::Participle { tense, voice, series, .. }) => Some(format!(
                "part.{}.{}.{}",
                match tense {
                    PartTense::Present => "pres",
                    PartTense::Past => "past",
                },
                match voice {
                    Voice::Active => "act",
                    Voice::Passive => "pass",
                },
                series_name(*series)
            )),
            _ => None,
        }
    }

    /// The adjective cell a participle or comparative cell declines as:
    /// the same series, gender, number and case in the positive degree.
    pub fn as_adjective(&self) -> Option<AdjCell> {
        match self {
            Cell::Adj(c) => Some(AdjCell { degree: Degree::Positive, ..*c }),
            Cell::Verb(VerbCell::Participle { series, gender, number, case, .. }) => Some(AdjCell {
                series: Some(*series),
                degree: Degree::Positive,
                gender: *gender,
                number: *number,
                case: *case,
            }),
            _ => None,
        }
    }

    /// The cell's number, where it has one.
    /// The case of a nominal cell (a noun, adjective, participle or
    /// pronoun cell); `None` elsewhere.
    pub fn case(&self) -> Option<Case> {
        match self {
            Cell::Noun(c) => Some(c.case),
            Cell::Adj(c) => Some(c.case),
            Cell::Verb(VerbCell::Participle { case, .. }) => Some(*case),
            Cell::Pron(c) => Some(c.case),
            _ => None,
        }
    }

    /// The gender of a cell that has one.
    pub fn gender(&self) -> Option<Gender> {
        match self {
            Cell::Adj(c) => Some(c.gender),
            Cell::Verb(VerbCell::LPart { gender, .. }) | Cell::Verb(VerbCell::Participle { gender, .. }) => Some(*gender),
            Cell::Pron(c) => c.gender,
            _ => None,
        }
    }

    /// The person of a finite, imperative or personal-pronoun cell.
    pub fn person(&self) -> Option<Person> {
        match self {
            Cell::Verb(VerbCell::Finite { person, .. }) | Cell::Verb(VerbCell::Imperative { person, .. }) => Some(*person),
            Cell::Pron(c) => c.person,
            _ => None,
        }
    }

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
            Cell::Adv(_) | Cell::Word => None,
        }
    }

    pub fn pos(&self) -> Pos {
        match self {
            Cell::Noun(_) => Pos::Noun,
            Cell::Adj(_) | Cell::Adv(_) => Pos::Adjective,
            Cell::Verb(_) => Pos::Verb,
            Cell::Pron(_) => Pos::Pronoun,
            Cell::Word => Pos::Closed,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

/// An underspecified cell: the set of cells of one part of speech a
/// paradigm does not tell apart in a form (the nominative, accusative and
/// vocative of a masculine inanimate; the masculine and neuter genitive
/// of a long adjective; the second and third person of an aorist). Sorted
/// and deduplicated; [`CellSet::first`] is the cell a consumer renders
/// through. The name factors the shared components and writes the
/// disjunction where they differ (`nom|acc|voc.sg`, `long.pos.m|n.sg.gen`,
/// `aor.2|3.sg`); a set that is not such a product lists its cells in
/// cell order (`nom.pl|gen.sg|acc.pl`). [`CellSet::parse`] is the inverse of
/// [`CellSet::name`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellSet {
    cells: Vec<Cell>,
}

impl CellSet {
    /// A set from cells of one part of speech; `None` when empty or mixed.
    pub fn new(mut cells: Vec<Cell>) -> Option<CellSet> {
        let pos = cells.first()?.pos();
        if cells.iter().any(|c| c.pos() != pos) {
            return None;
        }
        cells.sort();
        cells.dedup();
        Some(CellSet { cells })
    }

    pub fn one(cell: Cell) -> CellSet {
        CellSet { cells: vec![cell] }
    }

    pub fn first(&self) -> Cell {
        self.cells[0]
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn iter(&self) -> impl Iterator<Item = Cell> + '_ {
        self.cells.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn contains(&self, cell: Cell) -> bool {
        self.cells.binary_search(&cell).is_ok()
    }

    pub fn pos(&self) -> Pos {
        self.cells[0].pos()
    }

    /// The per-component values when the set is a Cartesian product of
    /// them (every cell has the same number of name components).
    fn columns(&self) -> Option<Vec<Vec<String>>> {
        let names: Vec<Vec<String>> = self.cells.iter().map(|c| c.name().split('.').map(str::to_string).collect()).collect();
        let width = names[0].len();
        if names.iter().any(|n| n.len() != width) {
            return None;
        }
        let mut columns: Vec<Vec<String>> = vec![Vec::new(); width];
        for name in &names {
            for (i, part) in name.iter().enumerate() {
                if !columns[i].contains(part) {
                    columns[i].push(part.clone());
                }
            }
        }
        let product: usize = columns.iter().map(Vec::len).product();
        (product == self.cells.len()).then_some(columns)
    }

    /// The factored name where the set is a product and the factored
    /// form reads back as the same set; the listed cells otherwise.
    pub fn name(&self) -> String {
        if self.cells.len() == 1 {
            return self.cells[0].name();
        }
        if let Some(columns) = self.columns() {
            let factored: String = columns.iter().map(|c| c.join("|")).collect::<Vec<_>>().join(".");
            if CellSet::parse_opt(self.pos(), &factored).as_ref() == Some(self) {
                return factored;
            }
        }
        self.cells.iter().map(Cell::name).collect::<Vec<_>>().join("|")
    }

    /// The inverse of [`CellSet::name`]: a list of whole cell names, or
    /// a factored name with `|` inside its components.
    ///
    /// ```
    /// use church_slavonic::{CellSet, Pos};
    /// let set = CellSet::parse(Pos::Noun, "nom|acc.sg").unwrap();
    /// assert_eq!(set.len(), 2);
    /// assert_eq!(set.name(), "nom|acc.sg");
    /// ```
    pub fn parse(pos: Pos, text: &str) -> Result<CellSet, CellError> {
        CellSet::parse_opt(pos, text).ok_or_else(|| CellError { pos, text: text.to_string() })
    }

    fn parse_opt(pos: Pos, text: &str) -> Option<CellSet> {
        let listed: Option<Vec<Cell>> = text.split('|').map(|piece| Cell::parse(pos, piece).ok()).collect();
        if let Some(cells) = listed {
            return CellSet::new(cells);
        }
        let columns: Vec<Vec<&str>> = text.split('.').map(|c| c.split('|').collect()).collect();
        let mut names: Vec<String> = vec![String::new()];
        for (i, column) in columns.iter().enumerate() {
            let mut next = Vec::new();
            for prefix in &names {
                for value in column {
                    next.push(if i == 0 { (*value).to_string() } else { format!("{prefix}.{value}") });
                }
            }
            names = next;
        }
        let cells: Option<Vec<Cell>> = names.iter().map(|n| Cell::parse(pos, n).ok()).collect();
        CellSet::new(cells?)
    }
}

impl fmt::Display for CellSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_sets_factor_and_round_trip() {
        let set = |pos: Pos, names: &[&str]| CellSet::new(names.iter().map(|n| Cell::parse(pos, n).unwrap()).collect()).unwrap();
        let s = set(Pos::Noun, &["voc.sg", "nom.sg", "acc.sg"]);
        assert_eq!(s.name(), "nom|acc|voc.sg");
        assert_eq!(s.first().name(), "nom.sg");
        assert_eq!(CellSet::parse(Pos::Noun, "nom|acc|voc.sg"), Ok(s.clone()));
        // not a product: listed
        let t = set(Pos::Noun, &["gen.sg", "nom.pl", "acc.pl"]);
        assert_eq!(t.name(), "nom.pl|gen.sg|acc.pl"); // listed in cell order (case-major)
        assert_eq!(CellSet::parse(Pos::Noun, &t.name()), Ok(t));
        let a = set(Pos::Adjective, &["long.pos.m.sg.gen", "long.pos.n.sg.gen"]);
        assert_eq!(a.name(), "long.pos.m|n.sg.gen");
        assert_eq!(CellSet::parse(Pos::Adjective, "long.pos.m|n.sg.gen"), Ok(a));
        let v = set(Pos::Verb, &["aor.2.sg", "aor.3.sg"]);
        assert_eq!(v.name(), "aor.2|3.sg");
        assert_eq!(CellSet::parse(Pos::Verb, "aor.2|3.sg"), Ok(v));
        // a pronoun's factored form that would read as a list of whole
        // cells (dat is the reflexive's cell) is written listed instead
        let p = set(Pos::Pronoun, &["3.m.sg.gen", "3.m.sg.dat"]);
        assert_eq!(p.name(), "3.m.sg.gen|3.m.sg.dat");
        assert_eq!(CellSet::parse(Pos::Pronoun, &p.name()), Ok(p));
        assert!(CellSet::parse(Pos::Noun, "nom|bogus.sg").is_err());
        assert_eq!(CellSet::new(vec![]), None);
        assert_eq!(CellSet::new(vec![Cell::parse(Pos::Noun, "nom.sg").unwrap(), Cell::Word]), None);
    }

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
        assert_eq!(Cell::parse(Pos::Noun, "gen.pl").map(|c| c.name()).as_deref(), Ok("gen.pl"));
        assert!(Cell::parse(Pos::Noun, "gen").is_err());
    }
}
