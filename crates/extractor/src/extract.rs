//! Source scanning and candidate policy — the editorial layer of the pipeline.
//!
//! Both sources funnel through one shape:
//!
//! 1. [`gather`] reads the filtered intermediates and turns them into
//!    per-`(recension, pos, lemma)` [`Observation`]s: for every cell of the
//!    schema row, the attested forms IN LISTING ORDER (order is editorial data:
//!    the first-listed form is the primary);
//! 2. [`finalize`] subtracts the rule engine cell by cell (a cell equal to the
//!    prediction — [`crate::cells::rule_matches`] — is blanked so the rule
//!    serves it), synthesises the k-th variant row slot-wise from the k-th
//!    alternates, drops the all-blank pattern (the rule engine serves it at
//!    runtime; its presence reserves the bare key), and numbers the survivors
//!    into deterministic `_<n>` keys via [`crate::assign`].
//!
//! Each refresh regenerates every table from the sources alone — there is no
//! lockfile, carry-forward state, or cross-source reconciliation. There is no
//! admission step either: an attested form the rules do not predict is in the
//! table, automatically.
//!
//! The policy decisions (each pinned by a fixture test at the bottom):
//! - a Kaikki entry is a lemma only when it is a proper single Cyrillic word and
//!   not merely a `form-of`/`alt-of` page; its unaccented, lowercased `word` is
//!   the key; each entry (sense) is one observation, so a homograph with two
//!   tables gets two candidates;
//! - a Kaikki adjective entry's tables are keyed by their OWN masculine
//!   nominative singular (the short and long paradigms are two lemmas, `новъ`
//!   and `новꙑи`), which is where the rule engine keys them too;
//! - a Kaikki finite verb block that wiktextract could not label
//!   (`error-unrecognized-form`) is read in printed order — persons 1, 2, 3 in a
//!   three-cell block, 1 then the shared 2/3 in a two-cell singular block; a
//!   block listing several paradigms back to back (aspect pairs) yields the
//!   later ones as alternates; the present active participle is the `present`
//!   participle table (feminine `-щи`), the past active the `-ши` one;
//! - an Alypy `Decline` table is a paradigm only when [`ALYPY_PARADIGMS`] says
//!   which part of speech it declines (the grammar also prints ending schemata,
//!   periphrastic tenses, numerals, participle declensions and word lists — all
//!   left out, with the reason in the list's comments); the lemma is the
//!   printed exemplar (`ра́б-ъ` -> `рабъ`), joined and unaccented, while every
//!   cell keeps its printed accents;
//! - the personal pronoun is one lemma-less row per recension (`personal`)
//!   merging the person entries (`азъ`, `тꙑ`, `и` in Kaikki; §47 in Alypy);
//!   only the first-listed alternative is reachable through the lemma-less API;
//! - a sense tagged `Old-East-Church-Slavonic` is soft: it sorts after standard
//!   siblings and never takes the bare key from one.

use crate::alypy::{self, Defaults, TenseWord};
use crate::assign::{Candidate, assign, forms_sig};
use crate::cells::{
    GENDERS, PRONOUN_KEY, Pos, adj_cell, noun_cell, pronoun_cell, recension_of_tag, rule_matches,
    tag, verb_cell,
};
use crate::kaikki::{self, Entry, has};
use church_slavonic_core::grammar::*;
use church_slavonic_core::orthography::strip_marks;
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

/// One part-of-speech's emitted table: `(key, cells)` rows, ready for the PHF
/// generator; the key already carries its recension prefix.
pub type Table = Vec<(String, Vec<String>)>;

#[derive(Debug, Default)]
pub struct Tables {
    pub noun: Table,
    pub adj: Table,
    pub verb: Table,
    pub pronoun: Table,
}

impl Tables {
    pub fn get(&self, pos: Pos) -> &Table {
        match pos {
            Pos::Noun => &self.noun,
            Pos::Adj => &self.adj,
            Pos::Verb => &self.verb,
            Pos::Pronoun => &self.pronoun,
        }
    }

    fn get_mut(&mut self, pos: Pos) -> &mut Table {
        match pos {
            Pos::Noun => &mut self.noun,
            Pos::Adj => &mut self.adj,
            Pos::Verb => &mut self.verb,
            Pos::Pronoun => &mut self.pronoun,
        }
    }
}

/// One attested paradigm: per schema cell, the forms listed for it, primary
/// first, deduplicated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observation {
    pub cells: Vec<Vec<String>>,
    pub soft: bool,
}

impl Observation {
    pub fn new(arity: usize) -> Self {
        Observation {
            cells: vec![Vec::new(); arity],
            soft: false,
        }
    }

    pub fn attest(&mut self, cell: usize, form: &str) {
        let slot = &mut self.cells[cell];
        if !slot.iter().any(|f| f == form) {
            slot.push(form.to_string());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_empty())
    }

    fn merge(&mut self, other: &Observation) {
        for (i, forms) in other.cells.iter().enumerate() {
            for f in forms {
                self.attest(i, f);
            }
        }
        self.soft = self.soft && other.soft;
    }
}

/// `(recension tag, pos, lemma)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexemeKey {
    pub tag: &'static str,
    pub pos: Pos,
    pub lemma: String,
}

pub type Lexemes = BTreeMap<LexemeKey, Vec<Observation>>;

/// The intermediate file names under `data/intermediate`.
pub const KAIKKI_INTERMEDIATE: &str = "kaikki.jsonl";
pub const ALYPY_INTERMEDIATE: &str = "alypy.jsonl";

/// Read every attested paradigm out of the filtered intermediates.
pub fn gather(intermediate_dir: &Path) -> Result<Lexemes, Box<dyn Error>> {
    let mut lexemes = Lexemes::new();
    let kaikki_path = intermediate_dir.join(KAIKKI_INTERMEDIATE);
    if kaikki_path.exists() {
        for entry in kaikki::read(&kaikki_path)? {
            gather_kaikki_entry(&entry, &mut lexemes);
        }
    }
    let alypy_path = intermediate_dir.join(ALYPY_INTERMEDIATE);
    if alypy_path.exists() {
        for table in alypy::read(&alypy_path)? {
            gather_alypy_table(&table, &mut lexemes)?;
        }
    }
    Ok(lexemes)
}

fn push_observation(lexemes: &mut Lexemes, key: LexemeKey, obs: Observation, merge: bool) {
    if obs.is_empty() {
        return;
    }
    let list = lexemes.entry(key).or_default();
    match list.first_mut() {
        Some(first) if merge => first.merge(&obs),
        _ => list.push(obs),
    }
}

// ---------------------------------------------------------------------------
// Kaikki (Old Church Slavonic)
// ---------------------------------------------------------------------------

const OCS: Recension = Recension::OldChurchSlavonic;

/// A lemma is a single Cyrillic word: no spaces, digits, underscores or
/// foreign letters (the dump's Glagolitic alternative-script pages fail this).
pub fn word_is_proper(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_cyrillic_letter)
}

fn is_cyrillic_letter(c: char) -> bool {
    matches!(c as u32, 0x0400..=0x052f | 0xa640..=0xa69f | 0x1c80..=0x1c8f)
}

/// A page whose every sense is a `form-of`/`alt-of` pointer is not a lemma.
fn entry_is_proper(entry: &Entry) -> bool {
    !entry.senses.is_empty()
        && entry
            .senses
            .iter()
            .any(|s| !s.tags.iter().any(|t| t == "form-of" || t == "alt-of"))
}

fn entry_is_soft(entry: &Entry) -> bool {
    !entry.senses.is_empty()
        && entry
            .senses
            .iter()
            .all(|s| s.tags.iter().any(|t| t == "Old-East-Church-Slavonic"))
}

/// A Kaikki surface: unaccented, lowercase, one word.
fn kaikki_form(form: &str) -> Option<String> {
    let f = strip_marks(form).to_lowercase();
    word_is_proper(&f).then_some(f)
}

fn gather_kaikki_entry(entry: &Entry, lexemes: &mut Lexemes) {
    if !entry_is_proper(entry) {
        return;
    }
    let Some(lemma) = kaikki_form(&entry.word) else {
        return;
    };
    let soft = entry_is_soft(entry);
    let key = |pos: Pos, lemma: String| LexemeKey {
        tag: tag(&OCS),
        pos,
        lemma,
    };
    match entry.pos.as_str() {
        "noun" => {
            let mut obs = Observation::new(Pos::Noun.arity());
            obs.soft = soft;
            for table in kaikki::tables(entry) {
                for f in table.forms {
                    let (Some(case), Some(number)) =
                        (kaikki::case(&f.tags), kaikki::number(&f.tags))
                    else {
                        continue;
                    };
                    if let Some(form) = kaikki_form(&f.form) {
                        obs.attest(noun_cell(&case, &number), &form);
                    }
                }
            }
            push_observation(lexemes, key(Pos::Noun, lemma), obs, false);
        }
        "adj" => {
            let mut by_lemma: BTreeMap<String, Observation> = BTreeMap::new();
            for table in kaikki::tables(entry) {
                let Some(table_lemma) = table.forms.iter().find_map(|f| {
                    let direct = kaikki::case(&f.tags) == Some(Case::Nominative)
                        && kaikki::number(&f.tags) == Some(Number::Singular)
                        && kaikki::genders(&f.tags).contains(&Gender::Masculine);
                    direct.then(|| kaikki_form(&f.form)).flatten()
                }) else {
                    continue;
                };
                let obs = by_lemma.entry(table_lemma).or_insert_with(|| {
                    let mut o = Observation::new(Pos::Adj.arity());
                    o.soft = soft;
                    o
                });
                for f in table.forms {
                    let (Some(case), Some(number)) =
                        (kaikki::case(&f.tags), kaikki::number(&f.tags))
                    else {
                        continue;
                    };
                    let Some(form) = kaikki_form(&f.form) else {
                        continue;
                    };
                    let genders = kaikki::genders(&f.tags);
                    let genders = if genders.is_empty() {
                        GENDERS.to_vec()
                    } else {
                        genders
                    };
                    for gender in genders {
                        if let Some(i) = adj_cell(&case, &number, &gender, &Degree::Positive) {
                            obs.attest(i, &form);
                        }
                    }
                }
            }
            for (table_lemma, obs) in by_lemma {
                push_observation(lexemes, key(Pos::Adj, table_lemma), obs, false);
            }
        }
        "verb" => {
            let mut obs = Observation::new(Pos::Verb.arity());
            obs.soft = soft;
            for table in kaikki::tables(entry) {
                if table.forms.iter().any(|f| has(&f.tags, "short-form")) {
                    gather_kaikki_participle(&table.forms, &mut obs);
                } else {
                    gather_kaikki_finite(&table.forms, &mut obs);
                }
            }
            push_observation(lexemes, key(Pos::Verb, lemma), obs, false);
        }
        "pron" => {
            let person = match lemma.as_str() {
                "азъ" => Person::First,
                "тꙑ" => Person::Second,
                "и" | "ѥ" | "ꙗ" => Person::Third,
                _ => return,
            };
            let mut obs = Observation::new(Pos::Pronoun.arity());
            for table in kaikki::tables(entry) {
                for f in table.forms {
                    let (Some(case), Some(number)) =
                        (kaikki::case(&f.tags), kaikki::number(&f.tags))
                    else {
                        continue;
                    };
                    if case == Case::Vocative {
                        continue;
                    }
                    let Some(form) = kaikki_form(&f.form) else {
                        continue;
                    };
                    let genders = kaikki::genders(&f.tags);
                    let genders = if genders.is_empty() || person != Person::Third {
                        GENDERS.to_vec()
                    } else {
                        genders
                    };
                    for gender in genders {
                        obs.attest(pronoun_cell(&person, &number, &gender, &case), &form);
                    }
                }
            }
            push_observation(
                lexemes,
                key(Pos::Pronoun, PRONOUN_KEY.to_string()),
                obs,
                true,
            );
        }
        _ => {}
    }
}

/// A participle table: its masculine nominative singular short form is the
/// citation cell; the feminine tells present (`-щи`) from past (`-ши`) active.
fn gather_kaikki_participle(forms: &[&kaikki::FormEntry], obs: &mut Observation) {
    let direct = |f: &kaikki::FormEntry, gender: Gender| {
        has(&f.tags, "short-form")
            && kaikki::case(&f.tags) == Some(Case::Nominative)
            && kaikki::number(&f.tags) == Some(Number::Singular)
            && kaikki::genders(&f.tags).contains(&gender)
    };
    let feminine = forms
        .iter()
        .find(|f| direct(f, Gender::Feminine))
        .and_then(|f| kaikki_form(&f.form));
    let cell = match feminine.as_deref() {
        Some(f) if f.ends_with("щи") => 36,
        Some(f) if f.ends_with("ши") => 37,
        _ => return,
    };
    for f in forms.iter().filter(|f| direct(f, Gender::Masculine)) {
        if let Some(form) = kaikki_form(&f.form) {
            obs.attest(cell, &form);
        }
    }
}

/// The finite table. Explicit person tags map directly; the unlabelled
/// blocks are read in printed order (see the module docs).
fn gather_kaikki_finite(forms: &[&kaikki::FormEntry], obs: &mut Observation) {
    let mut blocks: Vec<((Tense, Form, Number), Vec<String>)> = Vec::new();
    for f in forms {
        let Some(number) = kaikki::number(&f.tags) else {
            continue;
        };
        let (tense, form) = if has(&f.tags, "imperative") {
            (Tense::Present, Form::Imperative)
        } else if let Some(t) = kaikki::tense(&f.tags) {
            (t, Form::Finite)
        } else {
            continue;
        };
        if let Some(person) = kaikki::person(&f.tags) {
            if let (Some(surface), Some(i)) = (
                kaikki_form(&f.form),
                verb_cell(&person, &number, &tense, &form),
            ) {
                obs.attest(i, &surface);
            }
            continue;
        }
        if !has(&f.tags, "error-unrecognized-form") {
            continue;
        }
        let slot = (tense, form, number);
        match blocks.last_mut() {
            Some((last, list)) if *last == slot => list.push(f.form.clone()),
            _ => blocks.push((slot, vec![f.form.clone()])),
        }
    }
    for ((tense, form, number), list) in blocks {
        // The print merges the 2nd and 3rd singular of the pasts and the
        // imperative into one cell; every other block prints three persons.
        let merged_singular =
            number == Number::Singular && (form == Form::Imperative || tense != Tense::Present);
        let width = if merged_singular { 2 } else { 3 };
        for chunk in list.chunks(width) {
            if chunk.len() != width {
                continue;
            }
            for (j, printed) in chunk.iter().enumerate() {
                let Some(surface) = kaikki_form(printed) else {
                    continue;
                };
                let persons: &[Person] = match (merged_singular, j) {
                    (true, 0) => &[Person::First],
                    (true, _) => &[Person::Second, Person::Third],
                    (false, 0) => &[Person::First],
                    (false, 1) => &[Person::Second],
                    (false, _) => &[Person::Third],
                };
                for person in persons {
                    if let Some(i) = verb_cell(person, &number, &tense, &form) {
                        obs.attest(i, &surface);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Alypy (Synodal)
// ---------------------------------------------------------------------------

const SYN: Recension = Recension::Synodal;

#[derive(Debug, Clone, Copy)]
enum Block {
    /// A nominal declension keyed by the column exemplar.
    Declension,
    /// §60: the short comparative declined; its cells are the positive
    /// lemma's `Comparative` degree row.
    Comparative,
    /// Finite tense columns. `future_as_present` reads a `будущее` column as
    /// the (perfective) present of the lemma.
    Finite {
        future_as_present: bool,
    },
    Imperative,
}

/// One `Decline` table the grammar prints as a paradigm, and how to read it.
struct Paradigm {
    artifact: &'static str,
    index: usize,
    pos: Pos,
    /// The lemma when the table prints no exemplar (`бы́ти`).
    lemma: Option<&'static str>,
    /// Per-column lemmas when the table prints only forms (§103).
    column_lemmas: &'static [&'static str],
    defaults: Defaults,
    block: Block,
}

const fn declension(artifact: &'static str, index: usize, pos: Pos) -> Paradigm {
    Paradigm {
        artifact,
        index,
        pos,
        lemma: None,
        column_lemmas: &[],
        defaults: Defaults {
            number: None,
            tense: None,
        },
        block: Block::Declension,
    }
}

const fn verb(
    artifact: &'static str,
    index: usize,
    tense: Option<TenseWord>,
    block: Block,
) -> Paradigm {
    Paradigm {
        artifact,
        index,
        pos: Pos::Verb,
        lemma: None,
        column_lemmas: &[],
        defaults: Defaults {
            number: None,
            tense,
        },
        block,
    }
}

/// The paradigm tables of the grammar, keyed by (artifact, index among that
/// artifact's `Decline` tables). Every other `Decline` table is deliberately
/// not a source: §37 (a collective with no number dimension), §48 (the
/// interrogatives, outside the personal matrix), §56 (a two-word phrase),
/// §58/§80/§86.0/§87.0/§93.0 (ending schemata), §62–§69 (numerals),
/// §74–§77 (aspect illustration), §81.2–3/§84/§88/§89/§91/§102 (periphrastic
/// tenses and moods), §95–§98 (participle formation and declension),
/// §97 (the l-participle) and §103.2 (a participle inventory).
const ALYPY_PARADIGMS: &[Paradigm] = &[
    declension("p034.htm", 0, Pos::Noun),
    declension("p034.htm", 1, Pos::Noun),
    declension("p039.htm", 0, Pos::Noun),
    declension("p041.htm", 0, Pos::Noun),
    declension("p043.htm", 0, Pos::Noun),
    declension("p043.htm", 1, Pos::Noun),
    declension("p043.htm", 2, Pos::Noun),
    // §44 dual-only fragments (ѻ҆́чи, ᲂу҆́ши): the prose states the dual.
    Paradigm {
        defaults: Defaults {
            number: Some(Number::Dual),
            tense: None,
        },
        ..declension("p044.htm", 0, Pos::Noun)
    },
    declension("p047.htm", 0, Pos::Pronoun),
    declension("p047.htm", 1, Pos::Pronoun),
    declension("p053.htm", 0, Pos::Adj),
    declension("p053.htm", 1, Pos::Adj),
    declension("p057.htm", 0, Pos::Adj),
    declension("p057.htm", 1, Pos::Adj),
    declension("p057.htm", 2, Pos::Adj),
    declension("p057.htm", 3, Pos::Adj),
    Paradigm {
        block: Block::Comparative,
        ..declension("p060.htm", 0, Pos::Adj)
    },
    // §81 бы́ти: the present (its `бꙋ́дꙋ` future is a distinct stem, and the
    // compound future is periphrastic), then the two aorists and the imperfect.
    Paradigm {
        lemma: Some("быти"),
        ..verb(
            "p081.htm",
            0,
            None,
            Block::Finite {
                future_as_present: false,
            },
        )
    },
    Paradigm {
        lemma: Some("быти"),
        ..verb(
            "p081.htm",
            1,
            None,
            Block::Finite {
                future_as_present: false,
            },
        )
    },
    verb(
        "p082.htm",
        0,
        Some(TenseWord::Present),
        Block::Finite {
            future_as_present: false,
        },
    ),
    verb(
        "p086.htm",
        1,
        Some(TenseWord::Aorist),
        Block::Finite {
            future_as_present: false,
        },
    ),
    verb(
        "p087.htm",
        1,
        Some(TenseWord::Imperfect),
        Block::Finite {
            future_as_present: false,
        },
    ),
    verb(
        "p087.htm",
        2,
        Some(TenseWord::Imperfect),
        Block::Finite {
            future_as_present: false,
        },
    ),
    verb("p093.htm", 1, None, Block::Imperative),
    verb("p093.htm", 2, None, Block::Imperative),
    // §103 archaic athematic verbs print no infinitive row; the columns are
    // да́ти (a perfective, whose present the grammar labels future), ꙗ҆́сти,
    // вѣ́дѣти, и҆мѣ́ти.
    Paradigm {
        column_lemmas: &["дати", "ꙗсти", "вѣдѣти", "имѣти"],
        ..verb(
            "p103.htm",
            0,
            None,
            Block::Finite {
                future_as_present: true,
            },
        )
    },
    Paradigm {
        column_lemmas: &["дати", "ꙗсти", "вѣдѣти", "имѣти"],
        ..verb("p103.htm", 1, None, Block::Imperative)
    },
];

fn gather_alypy_table(table: &alypy::Table, lexemes: &mut Lexemes) -> Result<(), Box<dyn Error>> {
    let Some(paradigm) = ALYPY_PARADIGMS
        .iter()
        .find(|p| p.artifact == table.artifact && p.index == table.index)
    else {
        return Ok(());
    };
    let rows = alypy::rows(table, paradigm.defaults)?;
    let mut columns: Vec<usize> = rows.iter().map(|r| r.column).collect();
    columns.sort_unstable();
    columns.dedup();

    // The lemma of the table's masculine nominative singular (adjectives).
    let masculine_lemma = rows
        .iter()
        .find(|r| {
            r.cases.contains(&Case::Nominative)
                && r.number == Some(Number::Singular)
                && r.genders.contains(&Gender::Masculine)
        })
        .and_then(|r| alypy::alternatives(&r.surface).into_iter().next())
        .and_then(|s| alypy::lemma_key(&s));

    let mut observations: BTreeMap<String, Observation> = BTreeMap::new();
    for row in &rows {
        let lemma = match paradigm.pos {
            Pos::Pronoun => PRONOUN_KEY.to_string(),
            Pos::Adj => match (paradigm.block, &masculine_lemma) {
                (Block::Comparative, Some(l)) => match l.strip_suffix("ѣй") {
                    Some(stem) => format!("{stem}ъ"),
                    None => continue,
                },
                (_, Some(l)) => l.clone(),
                (_, None) => continue,
            },
            _ => {
                if let Some(l) = paradigm.lemma {
                    l.to_string()
                } else if !paradigm.column_lemmas.is_empty() {
                    let rank = columns.iter().position(|c| *c == row.column).unwrap_or(0);
                    match paradigm.column_lemmas.get(rank) {
                        Some(l) => l.to_string(),
                        None => continue,
                    }
                } else {
                    match alypy::lemma_key(&row.headword) {
                        Some(l) => l,
                        None => continue,
                    }
                }
            }
        };
        let forms: Vec<String> = alypy::alternatives(&row.surface)
            .into_iter()
            .filter(|f| !f.contains(' '))
            .collect();
        if forms.is_empty() {
            continue;
        }
        let obs = observations
            .entry(lemma)
            .or_insert_with(|| Observation::new(paradigm.pos.arity()));
        let mut cells: Vec<usize> = Vec::new();
        match (paradigm.pos, paradigm.block) {
            (Pos::Noun, _) => {
                if let Some(number) = row.number {
                    for case in &row.cases {
                        cells.push(noun_cell(case, &number));
                    }
                }
            }
            (Pos::Adj, block) => {
                let degree = match block {
                    Block::Comparative => Degree::Comparative,
                    _ => Degree::Positive,
                };
                let genders = if row.genders.is_empty() {
                    GENDERS.to_vec()
                } else {
                    row.genders.clone()
                };
                if let Some(number) = row.number {
                    for case in &row.cases {
                        for gender in &genders {
                            cells.extend(adj_cell(case, &number, gender, &degree));
                        }
                    }
                }
            }
            (Pos::Verb, Block::Finite { future_as_present }) => {
                let tense = match row.tense {
                    Some(TenseWord::Present) => Tense::Present,
                    Some(TenseWord::Imperfect) => Tense::Imperfect,
                    Some(TenseWord::Aorist) => Tense::Aorist,
                    Some(TenseWord::Future) if future_as_present => Tense::Present,
                    _ => continue,
                };
                if let Some(number) = row.number {
                    for person in &row.persons {
                        cells.extend(verb_cell(person, &number, &tense, &Form::Finite));
                    }
                }
            }
            (Pos::Verb, _) => {
                if let Some(number) = row.number {
                    for person in &row.persons {
                        cells.extend(verb_cell(
                            person,
                            &number,
                            &Tense::Present,
                            &Form::Imperative,
                        ));
                    }
                }
            }
            (Pos::Pronoun, _) => {
                let person = if paradigm.index == 1 {
                    Person::Third
                } else {
                    match alypy::lemma_key(&row.headword).as_deref() {
                        Some("азъ") => Person::First,
                        Some("ты") => Person::Second,
                        _ => continue,
                    }
                };
                let genders = if row.genders.is_empty() {
                    GENDERS.to_vec()
                } else {
                    row.genders.clone()
                };
                if let Some(number) = row.number {
                    for case in &row.cases {
                        if *case == Case::Vocative {
                            continue;
                        }
                        for gender in &genders {
                            cells.push(pronoun_cell(&person, &number, gender, case));
                        }
                    }
                }
            }
        }
        for cell in cells {
            for form in &forms {
                obs.attest(cell, form);
            }
        }
    }
    for (lemma, obs) in observations {
        let key = LexemeKey {
            tag: tag(&SYN),
            pos: paradigm.pos,
            lemma,
        };
        // The grammar has one lexeme per lemma: its tables merge.
        push_observation(lexemes, key, obs, true);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Finalize: subtract the rules, number the survivors.
// ---------------------------------------------------------------------------

/// Per-lemma accumulator: distinct patterns keyed by their form signature so
/// duplicates merge, plus whether a rule-equal pattern was seen (and dropped
/// as a runtime fall-through).
#[derive(Default)]
struct LemmaAcc {
    by_sig: BTreeMap<String, Candidate>,
    had_regular: bool,
}

impl LemmaAcc {
    fn observe(&mut self, forms: Vec<String>, soft: bool) {
        let sig = forms_sig(&forms);
        let is_new = !self.by_sig.contains_key(&sig);
        let c = self
            .by_sig
            .entry(sig)
            .or_insert_with(|| Candidate::new(forms));
        c.soft_sense = if is_new { soft } else { c.soft_sense && soft };
    }

    /// Drop the pattern with nothing left after subtracting the rules: the
    /// rule engine produces it at runtime, so a table row would add nothing,
    /// and its presence reserves the bare key.
    fn drop_regular(&mut self, arity: usize) {
        let sig = forms_sig(&vec![String::new(); arity]);
        if self.by_sig.remove(&sig).is_some() {
            self.had_regular = true;
        }
    }
}

/// Subtract the rule engine from every observation and number the survivors.
pub fn finalize(lexemes: &Lexemes) -> Tables {
    let mut tables = Tables::default();
    for (key, observations) in lexemes {
        let Some(recension) = recension_of_tag(key.tag) else {
            continue;
        };
        let arity = key.pos.arity();
        let predicted = key.pos.predict(&key.lemma, &recension);
        let mut acc = LemmaAcc::default();
        for obs in observations {
            let variants = if key.pos == Pos::Pronoun {
                1
            } else {
                obs.cells.iter().map(Vec::len).max().unwrap_or(0)
            };
            for k in 0..variants {
                let forms: Vec<String> = (0..arity)
                    .map(|i| {
                        let alts = &obs.cells[i];
                        match alts.get(k).or(alts.first()) {
                            Some(f) if !rule_matches(&recension, f, &predicted[i]) => f.clone(),
                            _ => String::new(),
                        }
                    })
                    .collect();
                acc.observe(forms, obs.soft);
            }
        }
        acc.drop_regular(arity);
        let had_regular = acc.had_regular;
        let candidates: Vec<Candidate> = acc.by_sig.into_values().collect();
        for a in assign(&key.lemma, candidates, had_regular) {
            tables
                .get_mut(key.pos)
                .push((format!("{}:{}", key.tag, a.key), a.forms));
        }
    }
    tables
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::CASES;

    fn entry(json: &str) -> Entry {
        serde_json::from_str(json).expect("fixture parses")
    }

    fn form(f: &str, tags: &[&str]) -> String {
        format!(
            r#"{{"form":"{f}","tags":[{}],"source":"conjugation"}}"#,
            tags.iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn only(lexemes: &Lexemes) -> (&LexemeKey, &Vec<Observation>) {
        assert_eq!(lexemes.len(), 1, "{lexemes:?}");
        lexemes.iter().next().expect("one lexeme")
    }

    #[test]
    fn unlabelled_finite_blocks_are_read_in_printed_order() {
        let forms = [
            form("no-table-tags", &["table-tags"]),
            form("аа", &["error-unrecognized-form", "present", "singular"]),
            form("аб", &["error-unrecognized-form", "present", "singular"]),
            form("ав", &["error-unrecognized-form", "present", "singular"]),
            form("ба", &["aorist", "error-unrecognized-form", "singular"]),
            form("бв", &["aorist", "error-unrecognized-form", "singular"]),
            form("-", &["error-unrecognized-form", "imperative", "singular"]),
            form("вв", &["error-unrecognized-form", "imperative", "singular"]),
            form("га", &["dual", "error-unrecognized-form", "imperative"]),
            form("гб", &["dual", "error-unrecognized-form", "imperative"]),
            form("-", &["dual", "error-unrecognized-form", "imperative"]),
        ];
        let e = entry(&format!(
            r#"{{"word":"нести","pos":"verb","senses":[{{"tags":[]}}],"forms":[{}]}}"#,
            forms.join(",")
        ));
        let mut lexemes = Lexemes::new();
        gather_kaikki_entry(&e, &mut lexemes);
        let (key, obs) = only(&lexemes);
        assert_eq!(key.lemma, "нести");
        let cell = |p: Person, n: Number, t: Tense, f: Form| {
            obs[0].cells[verb_cell(&p, &n, &t, &f).expect("cell")].clone()
        };
        use Form::*;
        use Number::*;
        use Person::*;
        use Tense::*;
        assert_eq!(cell(First, Singular, Present, Finite), ["аа"]);
        assert_eq!(cell(Third, Singular, Present, Finite), ["ав"]);
        assert_eq!(cell(Second, Singular, Aorist, Finite), ["бв"]);
        assert_eq!(cell(Third, Singular, Aorist, Finite), ["бв"]);
        assert!(cell(First, Singular, Present, Imperative).is_empty());
        assert_eq!(cell(Third, Singular, Present, Imperative), ["вв"]);
        assert_eq!(cell(Second, Dual, Present, Imperative), ["гб"]);
        assert!(cell(Third, Dual, Present, Imperative).is_empty());
    }

    #[test]
    fn adjective_tables_are_keyed_by_their_own_masculine_nominative() {
        let e = entry(
            r#"{"word":"новъ","pos":"adj","senses":[{"tags":[]}],"forms":[
              {"form":"no-table-tags","tags":["table-tags"],"source":"declension"},
              {"form":"новъ","tags":["masculine","nominative","singular"],"source":"declension"},
              {"form":"нова","tags":["genitive","masculine","neuter","singular"],"source":"declension"},
              {"form":"no-table-tags","tags":["table-tags"],"source":"declension"},
              {"form":"новꙑи","tags":["masculine","nominative","singular"],"source":"declension"},
              {"form":"новаѥго","tags":["genitive","masculine","neuter","singular"],"source":"declension"}
            ]}"#,
        );
        let mut lexemes = Lexemes::new();
        gather_kaikki_entry(&e, &mut lexemes);
        let lemmas: Vec<&str> = lexemes.keys().map(|k| k.lemma.as_str()).collect();
        assert_eq!(lemmas, ["новъ", "новꙑи"]);
        let genitive = adj_cell(
            &Case::Genitive,
            &Number::Singular,
            &Gender::Neuter,
            &Degree::Positive,
        )
        .expect("cell");
        let long = &lexemes.values().nth(1).expect("long")[0];
        assert_eq!(long.cells[genitive], ["новаѥго"]);
    }

    #[test]
    fn form_of_pages_and_improper_words_are_not_lemmas() {
        let mut lexemes = Lexemes::new();
        gather_kaikki_entry(
            &entry(
                r#"{"word":"града","pos":"noun","senses":[{"tags":["form-of"]}],"forms":[
                  {"form":"града","tags":["genitive","singular"],"source":"declension"}]}"#,
            ),
            &mut lexemes,
        );
        gather_kaikki_entry(
            &entry(
                r#"{"word":"ⰳⱃⰰⰴⱏ","pos":"noun","senses":[{"tags":[]}],"forms":[
                  {"form":"ⰳⱃⰰⰴⰰ","tags":["genitive","singular"],"source":"declension"}]}"#,
            ),
            &mut lexemes,
        );
        assert!(lexemes.is_empty());
        assert!(word_is_proper("градъ"));
        assert!(!word_is_proper("градъ_2"));
        assert!(!word_is_proper("два града"));
    }

    fn lexeme(tag: &'static str, pos: Pos, lemma: &str, cells: &[(usize, &[&str])]) -> Lexemes {
        let mut obs = Observation::new(pos.arity());
        for (i, forms) in cells {
            for f in *forms {
                obs.attest(*i, f);
            }
        }
        let mut l = Lexemes::new();
        l.insert(
            LexemeKey {
                tag,
                pos,
                lemma: lemma.to_string(),
            },
            vec![obs],
        );
        l
    }

    #[test]
    fn rule_equal_cells_are_blanked_and_a_fully_regular_lemma_has_no_row() {
        let genitive = noun_cell(&Case::Genitive, &Number::Singular);
        let dat = noun_cell(&Case::Dative, &Number::Singular);
        // рабъ: genitive раба and dative рабоу are exactly the rule.
        let t = finalize(&lexeme(
            "ocs",
            Pos::Noun,
            "рабъ",
            &[(genitive, &["раба"]), (dat, &["рабоу"])],
        ));
        assert!(t.noun.is_empty());
        // A u-stem dative on a word the rule declines as an o-stem differs;
        // the regular genitive is blanked.
        let t = finalize(&lexeme(
            "ocs",
            Pos::Noun,
            "рабъ",
            &[(genitive, &["раба"]), (dat, &["рабови"])],
        ));
        assert_eq!(t.noun.len(), 1);
        assert_eq!(t.noun[0].0, "ocs:рабъ");
        assert_eq!(t.noun[0].1[genitive], "");
        assert_eq!(t.noun[0].1[dat], "рабови");
    }

    #[test]
    fn alternates_become_variant_rows_and_a_regular_alternate_reserves_the_bare_key() {
        let dat = noun_cell(&Case::Dative, &Number::Singular);
        // The second-listed dative is the rule's: the irregular primary lands at
        // `_2` and the bare key falls through to the rule.
        let t = finalize(&lexeme(
            "ocs",
            Pos::Noun,
            "рабъ",
            &[(dat, &["рабови", "рабоу"])],
        ));
        assert_eq!(t.noun.len(), 1);
        assert_eq!(t.noun[0].0, "ocs:рабъ_2");
        // Two irregular alternates: both rows, sorted by signature.
        let t = finalize(&lexeme(
            "ocs",
            Pos::Noun,
            "рабъ",
            &[(dat, &["рабови", "рабъви"])],
        ));
        let keys: Vec<&str> = t.noun.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["ocs:рабъ", "ocs:рабъ_2"]);
        assert_eq!(t.noun[0].1[dat], "рабови");
        assert_eq!(t.noun[1].1[dat], "рабъви");
    }

    #[test]
    fn synodal_surfaces_are_compared_exactly_so_accents_are_tabled() {
        let t = finalize(&lexeme(
            "syn",
            Pos::Noun,
            "рабъ",
            &[
                (noun_cell(&Case::Genitive, &Number::Singular), &["раба̀"]),
                (noun_cell(&Case::Dative, &Number::Singular), &["рабꙋ"]),
            ],
        ));
        assert_eq!(t.noun.len(), 1);
        assert_eq!(t.noun[0].0, "syn:рабъ");
        let row = &t.noun[0].1;
        assert_eq!(row[noun_cell(&Case::Genitive, &Number::Singular)], "раба̀");
        assert_eq!(row[noun_cell(&Case::Dative, &Number::Singular)], "");
        assert_eq!(row.len(), CASES.len() * 3);
    }
}
