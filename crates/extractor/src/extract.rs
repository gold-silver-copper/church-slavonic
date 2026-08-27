//! Source scanning and candidate policy — the editorial layer of the pipeline.
//!
//! All three sources funnel through one shape:
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
//! lockfile, carry-forward state, or adjudication between sources. There is no
//! admission step either: an attested form the rules do not predict is in the
//! table, automatically. The two Synodal sources are read Polyakov first, then
//! Alypy: a lemma both attest is one observation (the union of their slots,
//! Polyakov's corpus-frequency primary first), so a slot they spell
//! differently becomes two variant rows by the sort — [`disagreements`]
//! counts those slots for the README.
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
//! - a Polyakov entry is one observation of its headword (marks stripped for
//!   the key, as the facade folds its input; the printed letters are kept, so
//!   its `у` never merges with Alypy's `ꙋ`), with each cell's forms ordered by
//!   corpus frequency (an enclitic pronoun after the full forms); a part of
//!   speech outside the four tables, the
//!   infinitive, the perfect (l-participle), the passive and long participles
//!   and the participle declension, an imperfective's `fut` (the periphrastic
//!   future) and a pronoun outside the personal matrix are skipped and counted
//!   ([`Skips`]); a perfective's `fut` is its present block; the short and long
//!   adjective series are two lemmas keyed by their own masculine nominative
//!   (as in Kaikki) — attested, or spelled by the paradigm class's legend
//!   ([`legend_nominative`]); a series with neither is skipped and counted
//!   (the starred fleeting-vowel classes); a `comp` form belongs to its series' positive lemma
//!   at `Comparative`; an `A,comp` headword (`бо́льшій`) is its own lemma;
//! - the personal pronoun is one lemma-less row per recension (`personal`)
//!   merging the person entries (`азъ`, `тꙑ`, `и` in Kaikki; §47 in Alypy;
//!   `а́зъ`, `ты́`, `мы́`, `вы́`, `и́` in Polyakov);
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
use crate::polyakov::{self, Features, Mood, Series, TenseTag, Voice};
use church_slavonic_core::grammar::*;
use church_slavonic_core::orthography::{comparison_key, strip_marks};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

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

/// The three labelled full-form sources, each with its filtered intermediate
/// under `data/intermediate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Kaikki,
    Alypy,
    Polyakov,
}

impl Source {
    /// Reading order: Polyakov before Alypy, so Alypy's paradigm merges into
    /// the corpus observation of the same lemma (see the module docs).
    pub const ALL: [Source; 3] = [Source::Kaikki, Source::Polyakov, Source::Alypy];

    pub fn intermediate(self) -> &'static str {
        match self {
            Source::Kaikki => "kaikki.jsonl",
            Source::Alypy => "alypy.jsonl",
            Source::Polyakov => "polyakov.jsonl",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Source::Kaikki => "kaikki",
            Source::Alypy => "alypy",
            Source::Polyakov => "polyakov",
        }
    }

    /// The README's "Recension" column: the recension, and the source where
    /// a recension has more than one.
    pub fn recension_label(self) -> &'static str {
        match self {
            Source::Kaikki => "OCS",
            Source::Alypy => "Synodal (Alypy)",
            Source::Polyakov => "Synodal (Polyakov)",
        }
    }
}

/// Read every attested paradigm out of the filtered intermediates, reporting
/// the Polyakov mapping coverage.
pub fn gather(intermediate_dir: &Path) -> Result<Lexemes, Box<dyn Error>> {
    let mut skips = Skips::default();
    let lexemes = gather_with(intermediate_dir, &Source::ALL, &mut skips)?;
    println!("Polyakov mapping: {skips}");
    Ok(lexemes)
}

/// [`gather`] restricted to `sources` (the accuracy harness scores each
/// source on its own). The reading order is always [`Source::ALL`]'s.
pub fn gather_sources(
    intermediate_dir: &Path,
    sources: &[Source],
) -> Result<Lexemes, Box<dyn Error>> {
    gather_with(intermediate_dir, sources, &mut Skips::default())
}

fn gather_with(
    intermediate_dir: &Path,
    sources: &[Source],
    skips: &mut Skips,
) -> Result<Lexemes, Box<dyn Error>> {
    let mut lexemes = Lexemes::new();
    for source in Source::ALL {
        if !sources.contains(&source) {
            continue;
        }
        let path = intermediate_dir.join(source.intermediate());
        if !path.exists() {
            continue;
        }
        match source {
            Source::Kaikki => {
                for entry in kaikki::read(&path)? {
                    gather_kaikki_entry(&entry, &mut lexemes);
                }
            }
            Source::Alypy => {
                for table in alypy::read(&path)? {
                    gather_alypy_table(&table, &mut lexemes)?;
                }
            }
            Source::Polyakov => {
                for entry in polyakov::read(&path)? {
                    gather_polyakov_entry(&entry, &mut lexemes, skips);
                }
            }
        }
    }
    Ok(lexemes)
}

/// Slots two gatherings attest with a different primary: `(exact, beyond
/// spelling)` — the second ignores accents, breathings, titla and the
/// recensions' letter conventions ([`comparison_key`]), so it counts the
/// disagreements that are not merely a convention of print.
pub fn disagreements(a: &Lexemes, b: &Lexemes) -> (u64, u64) {
    let (mut exact, mut beyond) = (0, 0);
    for (key, obs_a) in a {
        let (Some(first_a), Some(first_b)) = (obs_a.first(), b.get(key).and_then(|o| o.first()))
        else {
            continue;
        };
        for (cell_a, cell_b) in first_a.cells.iter().zip(&first_b.cells) {
            if let (Some(fa), Some(fb)) = (cell_a.first(), cell_b.first())
                && fa != fb
            {
                exact += 1;
                if comparison_key(fa) != comparison_key(fb) {
                    beyond += 1;
                }
            }
        }
    }
    (exact, beyond)
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
// Polyakov (Synodal)
// ---------------------------------------------------------------------------

/// What the Polyakov reading left out, by reason, plus what it mapped. One
/// unit is one expanded analysis of one form (`sg,gen/acc` is two).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Skips {
    pub entries: u64,
    pub forms: u64,
    pub mapped: u64,
    pub by_reason: BTreeMap<&'static str, u64>,
}

impl Skips {
    fn skip(&mut self, reason: &'static str) {
        *self.by_reason.entry(reason).or_default() += 1;
    }
}

impl fmt::Display for Skips {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} entries, {} forms, {} analyses mapped onto cells; skipped:",
            self.entries, self.forms, self.mapped
        )?;
        for (reason, n) in &self.by_reason {
            write!(f, " {reason}={n};")?;
        }
        Ok(())
    }
}

/// A Polyakov headword or surface as a table key: the facade's fold (NFC,
/// marks stripped, lowercase), and a proper single word.
fn polyakov_key(printed: &str) -> Option<String> {
    let key = strip_marks(printed).to_lowercase();
    word_is_proper(&key).then_some(key)
}

/// A printed surface, kept as printed (NFC) when it is a proper word under
/// its marks.
fn polyakov_surface(printed: &str) -> Option<String> {
    polyakov_key(printed).map(|_| printed.nfc().collect())
}

/// The cells one analysis attests, with the lemma that owns them.
type Attestation = (String, Vec<usize>);

/// Per `(lemma, cell)`: the attesting forms as `(clitic, frequency, print
/// order, surface)`.
type Attested = BTreeMap<(String, usize), Vec<(bool, u64, usize, String)>>;

fn gather_polyakov_entry(entry: &polyakov::Entry, lexemes: &mut Lexemes, skips: &mut Skips) {
    let pos = match entry.tags.first().map(String::as_str) {
        Some("S" | "N") => Pos::Noun,
        Some("A") => Pos::Adj,
        Some("V") => Pos::Verb,
        Some("SPRO") => Pos::Pronoun,
        _ => {
            skips.skip("entry: part of speech outside the four tables");
            return;
        }
    };
    let Some(lemma) = polyakov_key(&entry.lemma) else {
        skips.skip("entry: headword is not one word");
        return;
    };
    skips.entries += 1;
    let perfective = entry.tags.iter().any(|t| t == "pf" || t == "pf/ipf");
    let pronoun_person = match lemma.as_str() {
        "азъ" | "мы" => Some(Person::First),
        "ты" | "вы" => Some(Person::Second),
        "и" => Some(Person::Third),
        _ => None,
    };
    let series_lemmas = if pos == Pos::Adj {
        adjective_series_lemmas(entry, &lemma)
    } else {
        BTreeMap::new()
    };

    let mut attested = Attested::new();
    for (order, form) in entry.forms.iter().enumerate() {
        if form.cells.is_empty() {
            skips.skip("form: unanalysed");
            continue;
        }
        let Some(surface) = polyakov_surface(&form.form) else {
            skips.skip("form: not one word");
            continue;
        };
        skips.forms += 1;
        for set in &form.cells {
            let f = polyakov::features(set);
            let result = match pos {
                Pos::Noun => polyakov_noun_cells(&f).map(|c| (lemma.clone(), c)),
                Pos::Adj => polyakov_adj_cells(&f, &series_lemmas),
                Pos::Verb => polyakov_verb_cells(&f, perfective).map(|c| (lemma.clone(), c)),
                Pos::Pronoun => match pronoun_person {
                    Some(person) => {
                        polyakov_pronoun_cells(&f, person).map(|c| (PRONOUN_KEY.to_string(), c))
                    }
                    None => Err("pronoun: outside the personal matrix"),
                },
            };
            match result {
                Ok((owner, cells)) => {
                    skips.mapped += 1;
                    for cell in cells {
                        attested.entry((owner.clone(), cell)).or_default().push((
                            f.clitic,
                            form.count,
                            order,
                            surface.clone(),
                        ));
                    }
                }
                Err(reason) => skips.skip(reason),
            }
        }
    }

    let mut observations: BTreeMap<String, Observation> = BTreeMap::new();
    for ((owner, cell), mut forms) in attested {
        // Corpus frequency decides the primary and print order breaks ties;
        // an enclitic (`мя`) never outranks the full form (`мене́`), as in
        // the grammar, where the clitics are the alternatives.
        forms.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        let obs = observations
            .entry(owner)
            .or_insert_with(|| Observation::new(pos.arity()));
        for (_, _, _, surface) in forms {
            obs.attest(cell, &surface);
        }
    }
    for (owner, obs) in observations {
        let key = LexemeKey {
            tag: tag(&SYN),
            pos,
            lemma: owner,
        };
        // Each entry is one lexeme (a homograph gets its own observation);
        // the personal pronoun is the one shared row.
        push_observation(lexemes, key, obs, pos == Pos::Pronoun);
    }
}

/// The lemma of each adjective series in an entry: the headword for its own
/// series; for the other, the most frequent attested masculine nominative
/// singular, else the nominative the paradigm class's legend gives it
/// ([`legend_nominative`]). A series with neither has no lemma and its forms
/// are skipped.
fn adjective_series_lemmas(entry: &polyakov::Entry, lemma: &str) -> BTreeMap<Series, String> {
    let headword: String = entry.lemma.nfc().collect();
    let headword_series = entry
        .forms
        .iter()
        .filter(|f| f.form.nfc().collect::<String>() == headword)
        .flat_map(|f| &f.cells)
        .map(|set| polyakov::features(set))
        .filter(|f| !f.comparative)
        .find_map(|f| f.series)
        .unwrap_or(if has_long_ending(lemma) {
            Series::Long
        } else {
            Series::Short
        });
    let mut out = BTreeMap::new();
    out.insert(headword_series, lemma.to_string());
    let other = match headword_series {
        Series::Short => Series::Long,
        Series::Long => Series::Short,
    };
    let nominative = entry
        .forms
        .iter()
        .filter(|f| {
            f.cells.iter().map(|set| polyakov::features(set)).any(|f| {
                f.series == Some(other)
                    && !f.comparative
                    && f.number == Some(Number::Singular)
                    && f.gender == Some(Gender::Masculine)
                    && f.cases.contains(&Case::Nominative)
            })
        })
        .max_by_key(|f| f.count)
        .and_then(|f| polyakov_key(&f.form))
        .or_else(|| legend_nominative(&entry.class, lemma, other));
    if let Some(l) = nominative {
        out.insert(other, l);
    }
    out
}

fn has_long_ending(lemma: &str) -> bool {
    lemma.ends_with("ый") || lemma.ends_with("ій") || lemma.ends_with("ой")
}

/// The other series' masculine nominative singular as `flexslav.htm` defines
/// it for the entry's paradigm class: `A1t`/`A1k`/`A1g`/`A1a`/`A1n` (and the
/// possessive `A2t`) pair `-ъ` with `-ый` (`-ій` after a velar), `A1j`/`A1s`/
/// `A2j`/`A2s` pair `-ь` with `-ій`, `A1i`/`A2i` (`божій`) share one
/// nominative. A starred class (`A1t*`: `умный` ~ `уменъ`) has a fleeting
/// vowel the legend does not spell out, so it derives nothing. The key is
/// unaccented, so the series' accent difference is immaterial.
fn legend_nominative(class: &str, lemma: &str, wanted: Series) -> Option<String> {
    let first = class.split('/').next()?;
    if first.contains('*') {
        return None;
    }
    let letter = first
        .strip_prefix('A')?
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .chars()
        .next()?;
    let stem_and_long = |stem: &str| match letter {
        'k' | 'g' => Some(format!("{stem}ій")),
        't' | 'a' | 'n' => Some(format!("{stem}ый")),
        'j' | 's' => Some(format!("{stem}ій")),
        _ => None,
    };
    match (wanted, letter) {
        (_, 'i') => Some(lemma.to_string()),
        (Series::Short, 't' | 'k' | 'g' | 'a' | 'n') => {
            let stem = strip_long_ending(lemma)?;
            Some(format!("{stem}ъ"))
        }
        (Series::Short, 'j' | 's') => {
            let stem = strip_long_ending(lemma)?;
            Some(format!("{stem}ь"))
        }
        (Series::Long, 't' | 'k' | 'g' | 'a' | 'n') => stem_and_long(lemma.strip_suffix('ъ')?),
        (Series::Long, 'j' | 's') => stem_and_long(lemma.strip_suffix('ь')?),
        _ => None,
    }
}

fn strip_long_ending(lemma: &str) -> Option<&str> {
    ["ый", "ій", "ой"]
        .iter()
        .find_map(|e| lemma.strip_suffix(e))
}

fn polyakov_noun_cells(f: &Features) -> Result<Vec<usize>, &'static str> {
    let number = f.number.ok_or("noun: no number")?;
    if f.cases.is_empty() {
        return Err("noun: no case");
    }
    Ok(f.cases.iter().map(|c| noun_cell(c, &number)).collect())
}

fn polyakov_adj_cells(
    f: &Features,
    series_lemmas: &BTreeMap<Series, String>,
) -> Result<Attestation, &'static str> {
    let series = f.series.ok_or("adjective: no series tag")?;
    let lemma = series_lemmas.get(&series).ok_or(match series {
        Series::Short => "adjective: short series without an attested masculine nominative",
        Series::Long => "adjective: long series without an attested masculine nominative",
    })?;
    let number = f.number.ok_or("adjective: no number")?;
    if f.cases.is_empty() {
        return Err("adjective: no case");
    }
    let degree = if f.comparative {
        Degree::Comparative
    } else {
        Degree::Positive
    };
    let genders: Vec<Gender> = f.gender.map_or_else(|| GENDERS.to_vec(), |g| vec![g]);
    let mut cells = Vec::new();
    for case in &f.cases {
        for gender in &genders {
            cells.extend(adj_cell(case, &number, gender, &degree));
        }
    }
    Ok((lemma.clone(), cells))
}

fn polyakov_verb_cells(f: &Features, perfective: bool) -> Result<Vec<usize>, &'static str> {
    if f.infinitive {
        return Err("verb: infinitive (the lemma itself)");
    }
    if f.tense == Some(TenseTag::Perfect) {
        return Err("verb: perfect (the l-participle)");
    }
    if f.participle {
        if f.voice == Some(Voice::Passive) {
            return Err("verb: passive participle");
        }
        if f.series == Some(Series::Long) {
            return Err("verb: long-series participle");
        }
        let citation = f.number == Some(Number::Singular)
            && f.gender == Some(Gender::Masculine)
            && f.cases.contains(&Case::Nominative);
        if !citation {
            return Err("verb: participle declension");
        }
        return match f.tense {
            Some(TenseTag::Present) => Ok(vec![36]),
            Some(TenseTag::Future) if perfective => Ok(vec![36]),
            Some(TenseTag::Past) => Ok(vec![37]),
            _ => Err("verb: participle outside the two citation cells"),
        };
    }
    let (tense, form) = match (f.mood, f.tense) {
        (Some(Mood::Imperative), _) => (Tense::Present, Form::Imperative),
        (_, Some(TenseTag::Present)) => (Tense::Present, Form::Finite),
        (_, Some(TenseTag::Future)) if perfective => (Tense::Present, Form::Finite),
        (_, Some(TenseTag::Future)) => return Err("verb: future of an imperfective"),
        (_, Some(TenseTag::Aorist)) => (Tense::Aorist, Form::Finite),
        (_, Some(TenseTag::Imperfect)) => (Tense::Imperfect, Form::Finite),
        _ => return Err("verb: finite form without a tense"),
    };
    let number = f.number.ok_or("verb: no number")?;
    let person = f.person.ok_or("verb: no person")?;
    verb_cell(&person, &number, &tense, &form)
        .map(|c| vec![c])
        .ok_or("verb: cell outside the schema")
}

fn polyakov_pronoun_cells(f: &Features, person: Person) -> Result<Vec<usize>, &'static str> {
    let number = f.number.ok_or("pronoun: no number")?;
    let genders: Vec<Gender> = match (person, f.gender) {
        (Person::Third, Some(g)) => vec![g],
        _ => GENDERS.to_vec(),
    };
    let cells: Vec<usize> = f
        .cases
        .iter()
        .filter(|c| **c != Case::Vocative)
        .flat_map(|case| {
            genders
                .iter()
                .map(move |g| pronoun_cell(&person, &number, g, case))
        })
        .collect();
    if cells.is_empty() {
        return Err("pronoun: no case");
    }
    Ok(cells)
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

    fn polyakov_entry(
        lemma: &str,
        tags: &str,
        class: &str,
        forms: &[(&str, &str, u64)],
    ) -> polyakov::Entry {
        polyakov::Entry {
            lemma: lemma.to_string(),
            tags: tags.split(',').map(str::to_string).collect(),
            class: class.to_string(),
            count: forms.iter().map(|f| f.2).sum(),
            forms: forms
                .iter()
                .map(|(form, tags, count)| polyakov::FormEntry {
                    form: form.to_string(),
                    tags: tags.to_string(),
                    count: *count,
                    cells: polyakov::expand(tags),
                })
                .collect(),
        }
    }

    #[test]
    fn polyakov_frequency_orders_a_cell_and_expansions_attest_every_cell() {
        let e = polyakov_entry(
            "аарѡ́нъ",
            "S,m,anim,persn",
            "N1t",
            &[
                ("аарѡ́на", "sg,gen/acc", 90),
                ("аарѡ́нови", "sg,dat", 1),
                ("аарѡ́ну", "sg,dat", 86),
                ("аарѡ́нѣхъ", "", 2),
                ("другъдру́га", "gen/acc", 3),
            ],
        );
        let mut lexemes = Lexemes::new();
        let mut skips = Skips::default();
        gather_polyakov_entry(&e, &mut lexemes, &mut skips);
        let (key, obs) = only(&lexemes);
        assert_eq!(
            (key.tag, key.pos, key.lemma.as_str()),
            ("syn", Pos::Noun, "аарѡнъ")
        );
        let cell = |c: Case| obs[0].cells[noun_cell(&c, &Number::Singular)].clone();
        assert_eq!(cell(Case::Genitive), ["аарѡ́на"]);
        assert_eq!(cell(Case::Accusative), ["аарѡ́на"]);
        assert_eq!(cell(Case::Dative), ["аарѡ́ну", "аарѡ́нови"]);
        assert_eq!(skips.by_reason.get("form: unanalysed"), Some(&1));
        assert_eq!(skips.by_reason.get("noun: no number"), Some(&2));
        assert_eq!(skips.mapped, 4);
    }

    #[test]
    fn polyakov_adjective_series_are_two_lemmas_and_the_legend_spells_a_missing_nominative() {
        // The headword is long; the short series has an attested nominative.
        let e = polyakov_entry(
            "багря́ный",
            "A",
            "A1t",
            &[
                ("багря́ный", "plen,sg,m,nom/acc", 2),
                ("багря́нѣй", "plen,sg,f,dat/loc|comp,brev,sg,m,nom/acc", 1),
                ("багря́ныхъ", "plen/brev,pl,gen/loc", 1),
                ("багря́нъ", "brev,sg,m,nom/acc", 1),
            ],
        );
        let mut lexemes = Lexemes::new();
        gather_polyakov_entry(&e, &mut lexemes, &mut Skips::default());
        let lemmas: Vec<&str> = lexemes.keys().map(|k| k.lemma.as_str()).collect();
        assert_eq!(lemmas, ["багрянъ", "багряный"]);
        let short = &lexemes.values().next().expect("short")[0];
        let long = &lexemes.values().nth(1).expect("long")[0];
        let gen_pl = |g: Gender| {
            adj_cell(&Case::Genitive, &Number::Plural, &g, &Degree::Positive).expect("cell")
        };
        assert_eq!(short.cells[gen_pl(Gender::Neuter)], ["багря́ныхъ"]);
        assert_eq!(long.cells[gen_pl(Gender::Feminine)], ["багря́ныхъ"]);
        let comp = adj_cell(
            &Case::Nominative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Comparative,
        )
        .expect("cell");
        assert_eq!(short.cells[comp], ["багря́нѣй"]);
        assert!(long.cells[comp].is_empty());

        // No attested short nominative: the class legend spells it (A1k: -ій ~ -ъ);
        // a starred class (fleeting vowel) spells nothing and the series is skipped.
        for (class, expect) in [("A1k", Some("великъ")), ("A1t*", None)] {
            let e = polyakov_entry(
                "вели́кій",
                "A",
                class,
                &[
                    ("вели́ка", "brev,sg,m/n,gen/acc", 5),
                    ("вели́кій", "plen,sg,m,nom/acc", 9),
                ],
            );
            let mut lexemes = Lexemes::new();
            let mut skips = Skips::default();
            gather_polyakov_entry(&e, &mut lexemes, &mut skips);
            let lemmas: Vec<&str> = lexemes.keys().map(|k| k.lemma.as_str()).collect();
            match expect {
                Some(short) => assert_eq!(lemmas, [short, "великій"]),
                None => {
                    assert_eq!(lemmas, ["великій"]);
                    assert_eq!(
                        skips.by_reason.get(
                            "adjective: short series without an attested masculine nominative"
                        ),
                        Some(&4)
                    );
                }
            }
        }
        assert_eq!(
            legend_nominative("A2t", "аарѡновъ", Series::Long).as_deref(),
            Some("аарѡновый")
        );
        assert_eq!(
            legend_nominative("A2j", "аарѡнь", Series::Long).as_deref(),
            Some("аарѡній")
        );
        assert_eq!(
            legend_nominative("A1i", "божій", Series::Long).as_deref(),
            Some("божій")
        );
    }

    #[test]
    fn polyakov_verb_cells_follow_aspect_and_only_citation_participles_map() {
        let forms: &[(&str, &str, u64)] = &[
            ("да́ти", "inf", 10),
            ("да́мъ", "indic,fut,sg,1p", 7),
            ("да́сть", "indic,fut,sg,2p/3p", 9),
            ("да́хъ", "indic,aor,sg,1p", 3),
            ("да́лъ", "partcp,perf,sg,m", 4),
            ("да́вый", "partcp,praet,act,plen,sg,m/n,nom", 2),
            ("да́въ", "partcp,praet,act,brev,sg,m/n,nom", 6),
            ("да́вша", "partcp,praet,act,brev,sg,m/n,gen/acc", 1),
            ("да́ный", "partcp,praet,pass,plen,sg,m,nom/acc", 1),
            ("да́ждь", "imper,sg,2p/3p", 5),
        ];
        let mut lexemes = Lexemes::new();
        let mut skips = Skips::default();
        gather_polyakov_entry(
            &polyakov_entry("да́ти", "V,pf,tran", "Vdat", forms),
            &mut lexemes,
            &mut skips,
        );
        let (key, obs) = only(&lexemes);
        assert_eq!(key.lemma, "дати");
        let cell = |p: Person, n: Number, t: Tense, f: Form| {
            obs[0].cells[verb_cell(&p, &n, &t, &f).expect("cell")].clone()
        };
        assert_eq!(
            cell(
                Person::First,
                Number::Singular,
                Tense::Present,
                Form::Finite
            ),
            ["да́мъ"]
        );
        assert_eq!(
            cell(
                Person::Third,
                Number::Singular,
                Tense::Present,
                Form::Finite
            ),
            ["да́сть"]
        );
        assert_eq!(
            cell(Person::First, Number::Singular, Tense::Aorist, Form::Finite),
            ["да́хъ"]
        );
        assert_eq!(
            cell(
                Person::Second,
                Number::Singular,
                Tense::Present,
                Form::Imperative
            ),
            ["да́ждь"]
        );
        assert_eq!(obs[0].cells[37], ["да́въ"]);
        assert!(obs[0].cells[36].is_empty());
        for (reason, n) in [
            ("verb: infinitive (the lemma itself)", 1),
            ("verb: perfect (the l-participle)", 1),
            ("verb: long-series participle", 2),
            ("verb: participle declension", 5),
            ("verb: passive participle", 2),
        ] {
            assert_eq!(skips.by_reason.get(reason), Some(&n), "{reason}");
        }
        // An imperfective's `fut` is periphrastic and stays out.
        let mut lexemes = Lexemes::new();
        let mut skips = Skips::default();
        gather_polyakov_entry(
            &polyakov_entry(
                "бы́ти",
                "V,ipf,intr",
                "Vbyt",
                &[("бу́ду", "indic,fut,sg,1p", 1)],
            ),
            &mut lexemes,
            &mut skips,
        );
        assert!(lexemes.is_empty());
        assert_eq!(
            skips.by_reason.get("verb: future of an imperfective"),
            Some(&1)
        );
    }

    #[test]
    fn polyakov_pronouns_fill_the_personal_matrix_and_others_are_skipped() {
        let mut lexemes = Lexemes::new();
        let mut skips = Skips::default();
        gather_polyakov_entry(
            &polyakov_entry(
                "а́зъ",
                "SPRO",
                "PNja",
                &[("мене́", "sg,acc", 2690), ("мя́", "sg,acc,clit", 9226)],
            ),
            &mut lexemes,
            &mut skips,
        );
        gather_polyakov_entry(
            &polyakov_entry(
                "и́",
                "SPRO",
                "PA2i",
                &[("єгѡ́", "sg,m/n,gen", 16171), ("єя́", "sg,f,gen", 2344)],
            ),
            &mut lexemes,
            &mut skips,
        );
        gather_polyakov_entry(
            &polyakov_entry("кто́", "SPRO", "PNkto", &[("кого́", "sg,acc", 199)]),
            &mut lexemes,
            &mut skips,
        );
        let (key, obs) = only(&lexemes);
        assert_eq!(key.lemma, PRONOUN_KEY);
        assert_eq!(obs.len(), 1);
        let cell = |p: Person, g: Gender, c: Case| {
            obs[0].cells[pronoun_cell(&p, &Number::Singular, &g, &c)].clone()
        };
        assert_eq!(
            cell(Person::First, Gender::Feminine, Case::Accusative),
            ["мене́", "мя́"]
        );
        assert_eq!(cell(Person::Third, Gender::Neuter, Case::Genitive), ["єгѡ́"]);
        assert_eq!(
            cell(Person::Third, Gender::Feminine, Case::Genitive),
            ["єя́"]
        );
        assert_eq!(
            skips.by_reason.get("pronoun: outside the personal matrix"),
            Some(&1)
        );
    }

    #[test]
    fn alypy_merges_into_the_polyakov_observation_and_disagreements_are_counted() {
        let dat = noun_cell(&Case::Dative, &Number::Singular);
        let genitive = noun_cell(&Case::Genitive, &Number::Singular);
        let polyakov = lexeme(
            "syn",
            Pos::Noun,
            "рабъ",
            &[(dat, &["рабу́"]), (genitive, &["раба́"])],
        );
        let mut both = polyakov.clone();
        let alypy_obs = {
            let l = lexeme(
                "syn",
                Pos::Noun,
                "рабъ",
                &[(dat, &["рабꙋ̀"]), (genitive, &["раба́"])],
            );
            l.into_values().next().expect("obs").remove(0)
        };
        let key = both.keys().next().expect("key").clone();
        push_observation(&mut both, key, alypy_obs.clone(), true);
        let obs = &both.values().next().expect("obs");
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].cells[dat], ["рабу́", "рабꙋ̀"]);
        let mut alypy = Lexemes::new();
        alypy.insert(
            polyakov.keys().next().expect("key").clone(),
            vec![alypy_obs],
        );
        // The dative differs only by the print's letter and accent conventions.
        assert_eq!(disagreements(&alypy, &polyakov), (1, 0));
        let t = finalize(&both);
        let keys: Vec<&str> = t.noun.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["syn:рабъ", "syn:рабъ_2"]);
    }
}
