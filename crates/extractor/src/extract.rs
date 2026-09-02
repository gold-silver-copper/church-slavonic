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
//!    runtime; its presence reserves the bare key), numbers the survivors
//!    into deterministic `_<n>` keys via [`crate::assign`], and blanks every
//!    `_n` cell the bare row already holds (the runtime reads a `_n` blank
//!    from the bare row, then the rule).
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
//!   at `Comparative`; an `A,comp` headword (`бо́льшій`) is its own lemma; a
//!   form spelled under a titlo belongs to the lemma of the abbreviation
//!   (its citation form under the titlo: `бг҃ъ`, `гл҃ати`), a spelling lemma
//!   of its own that the rules inflect like any other — a titlo form whose
//!   citation form the entry never abbreviates is skipped and counted;
//! - the personal pronoun is one lemma-less row per recension (`personal`)
//!   merging the person entries (`азъ`, `тꙑ`, `и` in Kaikki; §47 in Alypy;
//!   `а́зъ`, `ты́`, `мы́`, `вы́`, `и́` in Polyakov);
//!   only the first-listed alternative is reachable through the lemma-less API;
//! - a sense tagged `Old-East-Church-Slavonic` is soft: it sorts after standard
//!   siblings and never takes the bare key from one.

use crate::alypy::{self, Defaults, TenseWord};
use crate::assign::{Candidate, assign, forms_sig};
use crate::cells::{
    clitic_cell, reflexive_cell, reflexive_clitic_cell,
    l_participle_cell,
    npron_cell,
    CASES, Conj, GENDERS, NUMBERS, PRESENT_STEM_CELL, PRONOUN_KEY, Pos, VERB_CLASS_CELL, adj_cell,
    noun_cell, participle_cell, participle_stem_cell, predict_verb_override, pronoun_cell,
    recension_of_tag, rule_matches, tag, verb_cell,
};
use crate::kaikki::{self, Entry, has};
use crate::polyakov::{self, Features, Mood, Series, TenseTag, Voice};
use crate::ruwiktionary;
use church_slavonic_core::ChurchSlavonicCore;
use church_slavonic_core::grammar::*;
use church_slavonic_core::grammar::{Series as GSeries, Voice as GVoice};
use church_slavonic_core::orthography::{
    comparison_key, realise, stress, strip_marks, transliteration_equivalent,
};
use std::collections::{BTreeMap, BTreeSet};
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
    pub npron: Table,
}

impl Tables {
    pub fn get(&self, pos: Pos) -> &Table {
        match pos {
            Pos::Noun => &self.noun,
            Pos::Adj => &self.adj,
            Pos::Verb => &self.verb,
            Pos::Pronoun => &self.pronoun,
            Pos::NPron => &self.npron,
        }
    }

    fn get_mut(&mut self, pos: Pos) -> &mut Table {
        match pos {
            Pos::Noun => &mut self.noun,
            Pos::Adj => &mut self.adj,
            Pos::Verb => &mut self.verb,
            Pos::Pronoun => &mut self.pronoun,
            Pos::NPron => &mut self.npron,
        }
    }
}

/// One attested paradigm: per schema cell, the forms listed for it, primary
/// first, deduplicated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observation {
    pub cells: Vec<Vec<String>>,
    pub soft: bool,
    /// The source spells the print's letters exactly
    /// ([`Source::letters_exact`]); a transliterated observation cannot.
    /// Every form such an observation attests is exact; a merge records
    /// the exact forms it receives in `exact`.
    pub precise: bool,
    /// The `(cell, form)` pairs attested by a print-exact source. A form
    /// outside this set is a transliteration: where it differs from the
    /// rule's prediction only in what a transliteration cannot encode
    /// ([`transliteration_equivalent`]), the rule decides the letters and
    /// the cell is not stored.
    pub exact: BTreeSet<(usize, String)>,
}

impl Observation {
    pub fn new(arity: usize) -> Self {
        Observation {
            cells: vec![Vec::new(); arity],
            soft: false,
            precise: false,
            exact: BTreeSet::new(),
        }
    }

    pub fn attest(&mut self, cell: usize, form: &str) {
        let slot = &mut self.cells[cell];
        if !slot.iter().any(|f| f == form) {
            slot.push(form.to_string());
        }
        if self.precise {
            self.exact.insert((cell, form.to_string()));
        }
    }

    /// Attest `form` as the cell's PRIMARY: the print outranks the
    /// transliteration (part 0 of v1.2, decision 4). A form already listed
    /// moves to the front; nothing is deleted.
    pub fn attest_primary(&mut self, cell: usize, form: &str) {
        let slot = &mut self.cells[cell];
        slot.retain(|f| f != form);
        slot.insert(0, form.to_string());
        self.exact.insert((cell, form.to_string()));
    }

    /// Is the attested form the print's own spelling (an exact source, or a
    /// witness), as opposed to a transliteration's?
    pub fn is_exact(&self, cell: usize, form: &str) -> bool {
        self.precise || self.exact.contains(&(cell, form.to_string()))
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_empty())
    }

    /// Merge another observation of the same lexeme: its forms join each
    /// cell's alternatives after the ones already there — except that a
    /// print-exact observation merging into a transliterated one decides
    /// the letters a transliteration cannot: a form that differs from the
    /// cell's primary only by ꙗ/ѧ or by the oxia/varia on a monosyllable
    /// ([`orthography::transliteration_equivalent`]) takes the primary
    /// slot, and the transliterated spelling stays as a variant.
    fn merge(&mut self, other: &Observation) {
        let decides_letters = other.precise && !self.precise;
        for (i, forms) in other.cells.iter().enumerate() {
            for f in forms {
                let equivalent = decides_letters
                    && self.cells[i]
                        .first()
                        .is_some_and(|primary| transliteration_equivalent(primary, f));
                if equivalent {
                    self.attest_primary(i, f);
                } else {
                    self.attest(i, f);
                }
                if other.is_exact(i, f) {
                    self.exact.insert((i, f.clone()));
                }
            }
        }
        self.soft = self.soft && other.soft;
    }

    /// Merge a witness observation: every witnessed form becomes its
    /// cell's primary unconditionally — a quoted, verified line of running
    /// print is the strongest evidence a cell can have.
    fn merge_as_primary(&mut self, other: &Observation) {
        for (i, forms) in other.cells.iter().enumerate() {
            for f in forms.iter().rev() {
                self.attest_primary(i, f);
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

/// The four labelled full-form sources, each with its filtered intermediate
/// under `data/intermediate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Kaikki,
    UdProiel,
    Alypy,
    Polyakov,
    RuWiktionary,
    /// `data/witnesses.tsv`: curated single cells from running Synodal
    /// print, each citing a verbatim line of a pinned text (v1.1;
    /// verified by `cargo xtask check-witnesses`).
    Witness,
}

impl Source {
    /// Reading order: Polyakov before Alypy and ru.wiktionary, so their
    /// paradigms merge into the corpus observation of the same lemma (see the
    /// module docs).
    pub const ALL: [Source; 6] = [
        Source::Kaikki,
        Source::UdProiel,
        Source::Polyakov,
        Source::Alypy,
        Source::RuWiktionary,
        Source::Witness,
    ];

    pub fn intermediate(self) -> &'static str {
        match self {
            Source::Kaikki => "kaikki.jsonl",
            Source::UdProiel => "ud_proiel.jsonl",
            Source::Alypy => "alypy.jsonl",
            Source::Polyakov => "polyakov.jsonl",
            Source::RuWiktionary => "ruwiktionary.jsonl",
            Source::Witness => "witnesses.tsv",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Source::Kaikki => "kaikki",
            Source::UdProiel => "ud_proiel_train",
            Source::Alypy => "alypy",
            Source::Polyakov => "polyakov",
            Source::RuWiktionary => "ruwiktionary",
                    Source::Witness => "witness",
}
    }

    /// Does the source spell the print's letters exactly? The Alypy grammar
    /// and the witness file reproduce the print (breathing, oxia/varia,
    /// ꙗ/ѧ); Polyakov's dictionary and ru.wiktionary are civil
    /// transliterations («я» for ꙗ and ѧ, one acute for both stresses),
    /// and the OCS sources are unaccented. See [`Observation::merge`].
    pub fn letters_exact(self) -> bool {
        matches!(self, Source::Alypy | Source::Witness)
    }

    /// The README's "Recension" column: the recension, and the source where
    /// a recension has more than one.
    pub fn recension_label(self) -> &'static str {
        match self {
            Source::Kaikki => "OCS",
            Source::UdProiel => "OCS (UD PROIEL train)",
            Source::Alypy => "Synodal (Alypy)",
            Source::Polyakov => "Synodal (Polyakov)",
            Source::RuWiktionary => "Synodal (ru.wiktionary)",
            Source::Witness => "Synodal (witnessed print)",
        }
    }
}

/// Read every attested paradigm out of the filtered intermediates, reporting
/// the Polyakov and ru.wiktionary mapping coverage.
pub fn gather(intermediate_dir: &Path) -> Result<Lexemes, Box<dyn Error>> {
    let mut skips = Skips::default();
    let lexemes = gather_with(intermediate_dir, &Source::ALL, &mut skips)?;
    println!("Polyakov and ru.wiktionary mapping: {skips}");
    Ok(lexemes)
}

/// Read `witnesses.tsv`: one attested cell per line, grouped per lemma
/// into a single observation (recension TAB pos TAB lemma TAB cell TAB
/// form TAB file TAB quote; `#` comments). The cell is a schema index or a
/// symbolic name ([`crate::cells::parse_cell`]: `3.f.sg.dat`, `m.pl.gen`,
/// `pl.acc`). A witnessed lexeme the other sources already observe ONCE
/// takes its forms as primaries ([`Observation::merge_as_primary`]); a
/// lexeme with several observations (homograph senses the witness cannot
/// choose between) or none gets its own, as in v1.1.
fn gather_witnesses(path: &Path, lexemes: &mut Lexemes) -> Result<(), Box<dyn Error>> {
    let text = std::fs::read_to_string(path)?;
    let mut grouped: BTreeMap<LexemeKey, Observation> = BTreeMap::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            return Err(format!("witnesses.tsv: malformed line: {line}").into());
        }
        let tag = match cols[0] {
            "syn" => "syn",
            "ocs" => "ocs",
            other => return Err(format!("witnesses.tsv: unknown recension {other}").into()),
        };
        let pos = match cols[1] {
            "noun" => Pos::Noun,
            "adj" => Pos::Adj,
            "verb" => Pos::Verb,
            "pronoun" => Pos::Pronoun,
            "npron" => Pos::NPron,
            other => return Err(format!("witnesses.tsv: unsupported pos {other}").into()),
        };
        let cell = crate::cells::parse_cell(pos, cols[3])
            .ok_or_else(|| format!("witnesses.tsv: bad cell in: {line}"))?;
        let lemma = if pos == Pos::Pronoun {
            PRONOUN_KEY.to_string()
        } else {
            cols[2].to_string()
        };
        let key = LexemeKey { tag, pos, lemma };
        let obs = grouped
            .entry(key)
            .or_insert_with(|| Observation::new(pos.arity()));
        obs.precise = true;
        obs.attest(cell, cols[4]);
    }
    for (key, obs) in grouped {
        let list = lexemes.entry(key).or_default();
        match list.as_mut_slice() {
            [only] => only.merge_as_primary(&obs),
            _ => list.push(obs),
        }
    }
    Ok(())
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
    // ru.wiktionary's unaccented headwords are keyed by the other Synodal
    // sources' accented lexemes: when read on its own (the accuracy
    // harness), those are gathered for the keys alone.
    let context = if sources.contains(&Source::RuWiktionary) && !sources.contains(&Source::Polyakov)
    {
        gather_with(
            intermediate_dir,
            &[Source::Polyakov, Source::Alypy],
            &mut Skips::default(),
        )?
    } else {
        Lexemes::new()
    };
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
            Source::Witness => {
                gather_witnesses(&path, &mut lexemes)?;
            }
            Source::Kaikki => {
                // Lemma pages first, then the standalone `form-of` pages,
                // whose attestations merge into the lemmas' observations.
                let entries = kaikki::read(&path)?;
                for entry in &entries {
                    gather_kaikki_entry(entry, &mut lexemes);
                }
                for entry in &entries {
                    gather_kaikki_form_of(entry, &mut lexemes, skips);
                }
            }
            Source::UdProiel => {
                gather_ud_proiel(&crate::treebank::read_train(&path)?, &mut lexemes, skips);
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
            Source::RuWiktionary => {
                for entry in ruwiktionary::read(&path)? {
                    gather_ruwiktionary_entry(&entry, &mut lexemes, &context, skips);
                }
            }
        }
    }
    Ok(lexemes)
}

// ---------------------------------------------------------------------------
// ru.wiktionary (Synodal)
// ---------------------------------------------------------------------------

/// An accented Synodal key for an unaccented dump headword: the lexeme the
/// other Synodal sources already hold under the same letters.
fn accented_synodal_key(lexemes: &Lexemes, pos: Pos, skeleton: &str) -> Option<String> {
    lexemes
        .keys()
        .find(|k| k.tag == tag(&SYN) && k.pos == pos && strip_marks(&k.lemma) == skeleton)
        .map(|k| k.lemma.clone())
}

/// One ru.wiktionary entry: its `forms` list read onto the schema. The dump's
/// `word` is unaccented, so the lemma is the accented nominative singular
/// the table prints (nouns), else the accented key Polyakov or Alypy gave
/// the same letters; an entry with neither is skipped and counted.
fn gather_ruwiktionary_entry(
    entry: &Entry,
    lexemes: &mut Lexemes,
    context: &Lexemes,
    skips: &mut Skips,
) {
    let pos = match entry.pos.as_str() {
        "noun" => Pos::Noun,
        "verb" => Pos::Verb,
        "adj" => Pos::Adj,
        _ => {
            skips.skip("ru.wiktionary: part of speech outside the tables");
            return;
        }
    };
    let Some(skeleton) = kaikki_form(&entry.word) else {
        skips.skip("ru.wiktionary: headword is not one word");
        return;
    };
    let skeleton = strip_marks(&realise(&skeleton, &SYN));
    let citation = entry.forms.iter().find(|f| {
        pos == Pos::Noun
            && has(&f.tags, "singular")
            && has(&f.tags, "nominative")
            && ruwiktionary::alternatives(f).len() == 1
    });
    let lemma = citation
        .and_then(|f| polyakov_key(&ruwiktionary::alternatives(f)[0]))
        .filter(|l| strip_marks(l) == skeleton)
        .or_else(|| accented_synodal_key(lexemes, pos, &skeleton))
        .or_else(|| accented_synodal_key(context, pos, &skeleton));
    let Some(lemma) = lemma else {
        skips.skip("ru.wiktionary: headword unaccented and unknown to the other Synodal sources");
        return;
    };
    let mut obs = Observation::new(pos.arity());
    for f in &entry.forms {
        let forms: Vec<String> = ruwiktionary::alternatives(f)
            .iter()
            .filter_map(|a| polyakov_surface(a))
            .collect();
        if forms.is_empty() {
            skips.skip("ru.wiktionary: periphrastic or unreadable form");
            continue;
        }
        let number = if ruwiktionary::is_dual_note(f) {
            Some(Number::Dual)
        } else {
            kaikki::number(&f.tags)
        };
        let cells: Vec<usize> = match pos {
            Pos::Noun => match (kaikki::case(&f.tags), number) {
                (Some(case), Some(number)) => vec![noun_cell(&case, &number)],
                _ => Vec::new(),
            },
            Pos::Verb => {
                if has(&f.tags, "perfect") {
                    skips.skip("ru.wiktionary: verb: perfect (the l-participle)");
                    continue;
                }
                let form = if has(&f.tags, "imperative") {
                    Form::Imperative
                } else {
                    Form::Finite
                };
                // A plural without a number tag: the third plural row.
                let number = number.unwrap_or(Number::Plural);
                let mut cells = Vec::new();
                for (name, person) in &PERSON_TAGS {
                    if has(&f.tags, name) {
                        cells.extend(verb_cell(person, &number, &Tense::Present, &form));
                    }
                }
                cells
            }
            _ => Vec::new(),
        };
        if cells.is_empty() {
            skips.skip("ru.wiktionary: form outside the schema");
            continue;
        }
        skips.mapped += 1;
        for cell in cells {
            for form in &forms {
                obs.attest(cell, form);
            }
        }
    }
    skips.entries += 1;
    push_observation(
        lexemes,
        LexemeKey {
            tag: tag(&SYN),
            pos,
            lemma,
        },
        obs,
        true,
    );
}

const PERSON_TAGS: [(&str, Person); 3] = [
    ("first-person", Person::First),
    ("second-person", Person::Second),
    ("third-person", Person::Third),
];

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

/// Does an attested form count as the rule's own answer? An exact
/// spelling must match the rule under the recension's policy
/// ([`rule_matches`]); a transliterated one also matches when it differs
/// only in what the transliteration cannot encode
/// ([`transliteration_equivalent`]: civil «я» for ꙗ/ѧ, one acute for the
/// monosyllable's oxia/varia) — the rule decides those letters.
pub fn attested_matches(exact: bool, recension: &Recension, attested: &str, predicted: &str) -> bool {
    rule_matches(recension, attested, predicted)
        || (!exact
            && *recension == Recension::Synodal
            && transliteration_equivalent(attested, predicted))
}

/// [`attested_matches`] for a pronoun row: the dictionary's tag bundles
/// (`sg,f,nom|pl,m,acc|pl,f,nom/acc`) give one spelling to a singular and
/// the plurals it looks like, so a bundled «всѧ̀» stands in the plural
/// cells where the print's number mark is the kamora («всѧ̑»); the
/// pronoun rule writes that mark per cell, and a transliterated form that
/// differs from it only by the stress mark on the same vowel is the rule's
/// form. Exact sources (the grammar, a witness) keep their marks.
pub fn pronoun_attested_matches(
    exact: bool,
    recension: &Recension,
    attested: &str,
    predicted: &str,
) -> bool {
    attested_matches(exact, recension, attested, predicted)
        || (!exact
            && *recension == Recension::Synodal
            && church_slavonic_core::orthography::number_mark_equivalent(attested, predicted))
}

/// The comparison a part of speech's attestations are subtracted under.
pub fn matches_for(pos: Pos) -> fn(bool, &Recension, &str, &str) -> bool {
    match pos {
        Pos::Pronoun | Pos::NPron => pronoun_attested_matches,
        _ => attested_matches,
    }
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
                _ => {
                    // A non-personal pronoun's own declension table:
                    // lemma-keyed gender/number/case cells.
                    let mut obs = Observation::new(Pos::NPron.arity());
                    obs.soft = soft;
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
                            let genders = if genders.is_empty() {
                                GENDERS.to_vec()
                            } else {
                                genders
                            };
                            for gender in genders {
                                obs.attest(npron_cell(&gender, &number, &case), &form);
                            }
                        }
                    }
                    push_observation(lexemes, key(Pos::NPron, lemma), obs, false);
                    return;
                }
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
/// The UD PROIEL train split (institutional grant — references/TERMS.md):
/// aggregated, normalised attestations from `treebank::filter_train`. The
/// frequency GATE lives here as policy: a (lemma, cell, form) enters only
/// with at least [`TRAIN_MIN_COUNT`] train-split attestations — hapaxes and
/// doubles in a hand-annotated corpus are where typos and annotation errors
/// live. Within a cell the corpus majority is attested first, so it becomes
/// the primary among the split's own variants. The whole observation is
/// SOFT and separate: a corpus is a secondary witness, so its variants sort
/// after every dictionary source's and can never take a bare key's primary
/// from them.
pub const TRAIN_MIN_COUNT: u64 = 3;

/// The corpus-frequency gate on Polyakov's declined-participle cells (the
/// citation cells 36/37 stay ungated, as before the schema widening).
pub const POLYAKOV_PARTICIPLE_MIN_COUNT: u64 = 5;

fn gather_ud_proiel(
    records: &[crate::treebank::TrainRecord],
    lexemes: &mut Lexemes,
    skips: &mut Skips,
) {
    use std::collections::HashMap;
    let mut by_lemma: BTreeMap<(Pos, String), Vec<&crate::treebank::TrainRecord>> = BTreeMap::new();
    let pos_of: HashMap<&str, Pos> = [
        ("noun", Pos::Noun),
        ("adj", Pos::Adj),
        ("verb", Pos::Verb),
        ("pronoun", Pos::Pronoun),
        ("npron", Pos::NPron),
    ]
    .into_iter()
    .collect();
    for r in records {
        skips.forms += 1;
        let Some(pos) = pos_of.get(r.pos.as_str()) else {
            skips.skip("train: unknown part of speech");
            continue;
        };
        if r.cell >= pos.arity() {
            skips.skip("train: cell outside the schema");
            continue;
        }
        if r.count < TRAIN_MIN_COUNT {
            skips.skip("train: below the frequency gate");
            continue;
        }
        by_lemma.entry((*pos, r.lemma.clone())).or_default().push(r);
    }
    for ((pos, lemma), mut records) in by_lemma {
        // Majority first within each cell: descending count, then the form.
        records.sort_by(|a, b| {
            (a.cell, std::cmp::Reverse(a.count), &a.form).cmp(&(
                b.cell,
                std::cmp::Reverse(b.count),
                &b.form,
            ))
        });
        let mut obs = Observation::new(pos.arity());
        obs.soft = true;
        // A scribal token is not automatically a form: an aphaeretic scrap
        // (`го` where `ѥго` dominates the cell), a one-letter remnant, or a
        // spelling that folds onto a form the cell already holds would be
        // noise, not attestation.
        let fold = |form: &str| comparison_key(form).replace('ъ', "ь");
        let mut kept: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for r in &records {
            if r.form.chars().count() < 2 {
                skips.skip("train: one-letter token");
                continue;
            }
            let cell = kept.entry(r.cell).or_default();
            let folded = fold(&r.form);
            if cell.iter().any(|k| fold(k) == folded) {
                skips.skip("train: spelling variant of an attested form");
                continue;
            }
            let subsequence = |short: &str, long: &str| {
                let mut rest = long;
                short.chars().all(|c| match rest.find(c) {
                    Some(at) => {
                        rest = &rest[at + c.len_utf8()..];
                        true
                    }
                    None => false,
                })
            };
            if cell.iter().any(|k| k != &r.form && subsequence(&r.form, k)) {
                skips.skip("train: elided writing of an attested form");
                continue;
            }
            cell.push(r.form.clone());
            obs.attest(r.cell, &r.form);
            skips.mapped += 1;
        }
        skips.entries += 1;
        push_observation(
            lexemes,
            LexemeKey {
                tag: tag(&OCS),
                pos,
                lemma,
            },
            obs,
            false,
        );
    }
}

/// A standalone `form-of` page: the headword is an attested form of the
/// sense's target lemma, its cells named by the sense tags — the same
/// vocabulary as the tables, but one sense may cover several cells
/// ("nominative/accusative dual"). Forms outside the schema (a passive or
/// l-participle, a future, a pronoun outside the personal matrix) are
/// skipped and counted. A sense tagged as Old East Church Slavonic joins as
/// a soft observation, like the East lemma pages do.
fn gather_kaikki_form_of(entry: &Entry, lexemes: &mut Lexemes, skips: &mut Skips) {
    let form_of: Vec<&kaikki::Sense> = entry
        .senses
        .iter()
        .filter(|s| kaikki::is_form_of(s))
        .collect();
    if form_of.is_empty() {
        return;
    }
    let Some(form) = kaikki_form(&entry.word) else {
        return skips.skip("form-of: non-Cyrillic headword");
    };
    for sense in form_of {
        let Some(lemma) = sense.form_of.first().and_then(|t| kaikki_form(&t.word)) else {
            skips.skip("form-of: non-Cyrillic target");
            continue;
        };
        let tags = &sense.tags;
        let soft = kaikki::has(tags, "Old-East-Church-Slavonic") || kaikki::has(tags, "East");
        let cases = kaikki::cases(tags);
        let numbers = kaikki::numbers(tags);
        let persons = kaikki::persons(tags);
        let (pos, cells): (Pos, Vec<usize>) = match entry.pos.as_str() {
            "noun" => {
                if cases.is_empty() || numbers.is_empty() {
                    skips.skip("form-of: noun without a case and number");
                    continue;
                }
                let mut cells = Vec::new();
                for case in &cases {
                    for number in &numbers {
                        cells.push(noun_cell(case, number));
                    }
                }
                (Pos::Noun, cells)
            }
            "pron" => {
                let person = match lemma.as_str() {
                    "азъ" => Some(Person::First),
                    "тꙑ" => Some(Person::Second),
                    "и" | "ѥ" | "ꙗ" => Some(Person::Third),
                    _ => None,
                };
                if cases.is_empty() || numbers.is_empty() {
                    skips.skip("form-of: pronoun without a case and number");
                    continue;
                }
                let genders = kaikki::genders(tags);
                let genders = if genders.is_empty() || person.is_some_and(|p| p != Person::Third)
                {
                    GENDERS.to_vec()
                } else {
                    genders
                };
                let mut cells = Vec::new();
                for case in cases.iter().filter(|c| **c != Case::Vocative) {
                    for number in &numbers {
                        for gender in &genders {
                            cells.push(match &person {
                                // A non-personal pronoun's form-of page.
                                None => npron_cell(gender, number, case),
                                Some(p) => pronoun_cell(p, number, gender, case),
                            });
                        }
                    }
                }
                match person {
                    None => (Pos::NPron, cells),
                    Some(_) => (Pos::Pronoun, cells),
                }
            }
            "adj" => {
                // The bare comparative pointer attests the comparative
                // citation cell; a case-tagged sense its named cells.
                let comparative = kaikki::has(tags, "comparative");
                if kaikki::has(tags, "superlative") || kaikki::has(tags, "definite") {
                    skips.skip("form-of: adjective outside the schema");
                    continue;
                }
                let degree = if comparative {
                    Degree::Comparative
                } else {
                    Degree::Positive
                };
                let (cases, numbers) = if comparative && cases.is_empty() {
                    (vec![Case::Nominative], vec![Number::Singular])
                } else {
                    (cases, numbers)
                };
                if cases.is_empty() || numbers.is_empty() {
                    skips.skip("form-of: adjective without a case and number");
                    continue;
                }
                let genders = kaikki::genders(tags);
                let genders = if genders.is_empty() {
                    if comparative {
                        vec![Gender::Masculine]
                    } else {
                        GENDERS.to_vec()
                    }
                } else {
                    genders
                };
                let mut cells = Vec::new();
                for case in &cases {
                    for number in &numbers {
                        for gender in &genders {
                            if let Some(i) = adj_cell(case, number, gender, &degree) {
                                cells.push(i);
                            }
                        }
                    }
                }
                (Pos::Adj, cells)
            }
            "verb" => {
                const OUTSIDE: [&str; 6] = [
                    "passive",
                    "supine",
                    "future",
                    "conditional",
                    "optative",
                    "negative",
                ];
                if OUTSIDE.iter().any(|t| kaikki::has(tags, t)) {
                    skips.skip("form-of: verb form outside the schema");
                    continue;
                }
                if kaikki::has(tags, "l-participle") {
                    // Nominative-only gender/number cells.
                    let numbers = if numbers.is_empty() {
                        vec![Number::Singular]
                    } else {
                        numbers.clone()
                    };
                    let genders = kaikki::genders(tags);
                    let genders = if genders.is_empty() {
                        GENDERS.to_vec()
                    } else {
                        genders
                    };
                    let mut cells = Vec::new();
                    for number in &numbers {
                        for gender in &genders {
                            cells.push(l_participle_cell(gender, number));
                        }
                    }
                    (Pos::Verb, cells)
                } else 
                if kaikki::has(tags, "participle") {
                    let cell = if kaikki::has(tags, "present") {
                        36
                    } else if kaikki::has(tags, "past") {
                        37
                    } else {
                        skips.skip("form-of: participle without a tense");
                        continue;
                    };
                    (Pos::Verb, vec![cell])
                } else {
                    let (tense, verb_form) = if kaikki::has(tags, "imperative") {
                        (Tense::Present, Form::Imperative)
                    } else if let Some(tense) = kaikki::tense(tags) {
                        (tense, Form::Finite)
                    } else {
                        skips.skip("form-of: finite verb without a tense");
                        continue;
                    };
                    if persons.is_empty() || numbers.is_empty() {
                        skips.skip("form-of: finite verb without a person and number");
                        continue;
                    }
                    let mut cells = Vec::new();
                    for person in &persons {
                        for number in &numbers {
                            if let Some(i) = verb_cell(person, number, &tense, &verb_form) {
                                cells.push(i);
                            }
                        }
                    }
                    (Pos::Verb, cells)
                }
            }
            _ => continue,
        };
        if cells.is_empty() {
            skips.skip("form-of: no cell resolved");
            continue;
        }
        let mut obs = Observation::new(pos.arity());
        obs.soft = soft;
        for cell in cells {
            obs.attest(cell, &form);
        }
        skips.mapped += 1;
        let key = LexemeKey {
            tag: tag(&OCS),
            pos,
            // The personal pronoun is the one shared, lemma-less row.
            lemma: if pos == Pos::Pronoun {
                PRONOUN_KEY.to_string()
            } else {
                lemma
            },
        };
        push_observation(lexemes, key, obs, !soft);
    }
}

fn gather_kaikki_participle(forms: &[&kaikki::FormEntry], obs: &mut Observation) {
    // The sub-table's tense and voice are read off the short feminine
    // nominative singular: `-щи` present active, `-ши` past active, `-ма`
    // present passive, `-на`/`-та` past passive.
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
    let (tense, voice, citation) = match feminine.as_deref() {
        Some(f) if f.ends_with("щи") => (Tense::Present, GVoice::Active, Some(36)),
        Some(f) if f.ends_with("ши") => (Tense::Aorist, GVoice::Active, Some(37)),
        Some(f) if f.ends_with("ма") => (Tense::Present, GVoice::Passive, None),
        Some(f) if f.ends_with("на") || f.ends_with("та") => {
            (Tense::Aorist, GVoice::Passive, None)
        }
        _ => return,
    };
    for f in forms {
        let Some(form) = kaikki_form(&f.form) else {
            continue;
        };
        let series = if has(&f.tags, "short-form") {
            GSeries::Short
        } else if has(&f.tags, "long-form") {
            GSeries::Long
        } else {
            continue;
        };
        let (Some(case), Some(number)) = (kaikki::case(&f.tags), kaikki::number(&f.tags)) else {
            continue;
        };
        let genders = kaikki::genders(&f.tags);
        for gender in &genders {
            obs.attest(
                participle_cell(&voice, &series, &tense, gender, &number, &case),
                &form,
            );
        }
        // The masculine nominative singular short form is also the
        // citation cell of the finite block.
        if let Some(citation) = citation
            && series == GSeries::Short
            && case == Case::Nominative
            && number == Number::Singular
            && genders.contains(&Gender::Masculine)
        {
            obs.attest(citation, &form);
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
    /// The data columns (by rank among the table's columns, `start..end`)
    /// this paradigm reads, when a table prints two paradigms side by side
    /// (§47's third person beside мо́й, §48's ѻ҆́въ beside ѻ҆́вый); every
    /// column otherwise.
    columns: Option<(usize, usize)>,
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
        columns: None,
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
        columns: None,
        defaults: Defaults {
            number: None,
            tense,
        },
        block,
    }
}

/// The paradigm tables of the grammar, keyed by (artifact, index among that
/// artifact's `Decline` tables). Every other `Decline` table is deliberately
/// not a source: §37 (a collective with no number dimension), §48.3 (the
/// толи́цы fragment, a grid without its singular), §56 (a two-word phrase),
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
    // §47's second table prints the possessive мо́й beside the third
    // person: the first three columns are the matrix, the rest the
    // non-personal pronoun's row (v1.2 part 2).
    Paradigm {
        columns: Some((0, 3)),
        ..declension("p047.htm", 1, Pos::Pronoun)
    },
    Paradigm {
        columns: Some((3, 6)),
        ..declension("p047.htm", 1, Pos::NPron)
    },
    declension("p047.htm", 2, Pos::NPron),
    // §48: the interrogatives (singular only; the third column lists
    // что̀'s alternatives), кі́й, на́шъ, and ѻ҆́въ's short series beside its
    // long one — the long series is the adjective ѻ҆́вый's row.
    Paradigm {
        column_lemmas: &["кто̀", "что̀", "что̀"],
        defaults: Defaults {
            number: Some(Number::Singular),
            tense: None,
        },
        ..declension("p048.htm", 0, Pos::NPron)
    },
    declension("p048.htm", 1, Pos::NPron),
    declension("p048.htm", 2, Pos::NPron),
    Paradigm {
        columns: Some((0, 3)),
        ..declension("p048.htm", 4, Pos::NPron)
    },
    Paradigm {
        columns: Some((3, 6)),
        ..declension("p048.htm", 4, Pos::Adj)
    },
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
        lemma: Some("бы́ти"),
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
        lemma: Some("бы́ти"),
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
        column_lemmas: &["да́ти", "ꙗ҆́сти", "вѣ́дѣти", "и҆мѣ́ти"],
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
        column_lemmas: &["да́ти", "ꙗ҆́сти", "вѣ́дѣти", "и҆мѣ́ти"],
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
    let all_rows = alypy::rows(table, paradigm.defaults)?;
    let mut columns: Vec<usize> = all_rows.iter().map(|r| r.column).collect();
    columns.sort_unstable();
    columns.dedup();
    let rank = |column: usize| columns.iter().position(|c| *c == column).unwrap_or(0);
    let rows: Vec<&alypy::Row> = all_rows
        .iter()
        .filter(|r| {
            paradigm
                .columns
                .is_none_or(|(start, end)| (start..end).contains(&rank(r.column)))
        })
        .collect();

    // The lemma of the table's masculine nominative singular (adjectives
    // and non-personal pronouns).
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
    for row in rows {
        let lemma = match paradigm.pos {
            Pos::Pronoun => PRONOUN_KEY.to_string(),
            Pos::NPron if paradigm.column_lemmas.is_empty() => match &masculine_lemma {
                Some(l) => l.clone(),
                None => continue,
            },
            Pos::Adj => match (paradigm.block, &masculine_lemma) {
                // §60 declines the short comparative (`мꙋдрѣ́й`): its cells
                // belong to the positive lemma, stressed on its stem.
                (Block::Comparative, Some(l)) => match strip_marks(l).strip_suffix("ѣй") {
                    Some(stem) => restress(l, &format!("{stem}ъ")),
                    None => continue,
                },
                (_, Some(l)) => l.clone(),
                (_, None) => continue,
            },
            _ => {
                if let Some(l) = paradigm.lemma {
                    l.to_string()
                } else if !paradigm.column_lemmas.is_empty() {
                    match paradigm.column_lemmas.get(rank(row.column)) {
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
            .map(|f| realise(&f, &SYN))
            .collect();
        if forms.is_empty() {
            continue;
        }
        let obs = observations
            .entry(lemma)
            .or_insert_with(|| Observation::new(paradigm.pos.arity()));
        obs.precise = Source::Alypy.letters_exact();
        let mut cells: Vec<usize> = Vec::new();
        match (paradigm.pos, paradigm.block) {
            (Pos::NPron, _) => {
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
                            cells.push(npron_cell(gender, &number, case));
                        }
                    }
                }
            }
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
                // §47's third column is the reflexive: no exemplar heads it
                // (себѐ has no nominative), its forms name it.
                let reflexive = forms
                    .iter()
                    .any(|f| matches!(strip_marks(f).as_str(), "себе" | "себє" | "себѣ" | "собою"));
                if reflexive {
                    for case in &row.cases {
                        if matches!(*case, Case::Vocative | Case::Nominative) {
                            continue;
                        }
                        for form in &forms {
                            // «себѣ̀, сѝ»: the alternative that spells the
                            // clitic goes to the clitic cell.
                            let clitic = ChurchSlavonicCore::reflexive_clitic(case, &SYN)
                                .filter(|c| comparison_key(c) == comparison_key(form))
                                .and_then(|_| reflexive_clitic_cell(case));
                            obs.attest(clitic.unwrap_or_else(|| reflexive_cell(case)), form);
                        }
                    }
                    continue;
                }
                let person = if paradigm.index == 1 {
                    Person::Third
                } else {
                    match alypy::lemma_key(&row.headword)
                        .map(|l| strip_marks(&l))
                        .as_deref()
                    {
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
                            for form in &forms {
                                // The grammar prints the clitic as the
                                // alternative («мнѣ̀, мѝ»; «є҆го̀, и҆̀»; «ѧ҆̀,
                                // и҆̀хъ»; the nominative's «ѻ҆́нъ (и҆̀)»): a
                                // form spelling the rule's accusative or
                                // dative clitic for the cell's person, number
                                // and gender attests that clitic cell.
                                let clitic = [Case::Accusative, Case::Dative]
                                    .into_iter()
                                    .find_map(|c| {
                                        ChurchSlavonicCore::clitic(&person, &number, gender, &c, &SYN)
                                            .filter(|k| comparison_key(k) == comparison_key(form))
                                            .and_then(|k| clitic_cell(&person, &number, gender, &c).map(|i| (i, k)))
                                    });
                                let full = pronoun_cell(&person, &number, gender, case);
                                match clitic {
                                    Some((i, k)) => {
                                        obs.attest(i, form);
                                        // «ю҆̀» is the accusative and its clitic
                                        if comparison_key(ChurchSlavonicCore::pronoun(&person, &number, gender, case, &SYN))
                                            == comparison_key(k)
                                        {
                                            obs.attest(full, form);
                                        }
                                    }
                                    None => obs.attest(full, form),
                                }
                            }
                        }
                    }
                }
                continue;
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

/// A Polyakov headword or surface in the canonical Synodal typography
/// ([`realise`]: the print's letters for the edition's `у`/`я`, the
/// breathing, the oxia/varia by position — the accent kept), when it is a
/// proper single word under its marks. The headword is the table key.
fn polyakov_key(printed: &str) -> Option<String> {
    let key = realise(printed, &SYN);
    word_is_proper(&strip_marks(&key)).then_some(key)
}

/// A headword the dictionary writes with `+` before a solid enclitic
/// (`то́й+же`, `что́+же`, `то́й+жде`) is the print's one word (то́йже); every
/// other `+` headword (и́+на, что́+либо) is not one word.
fn polyakov_headword(printed: &str) -> Option<String> {
    match printed.rsplit_once('+') {
        Some((host, enclitic @ ("же" | "жде" | "ждо"))) if !host.contains('+') => {
            polyakov_key(&format!("{host}{enclitic}"))
        }
        Some(_) => None,
        None => polyakov_key(printed),
    }
}

fn polyakov_surface(printed: &str) -> Option<String> {
    polyakov_key(printed)
}

/// Carry `accented`'s stress onto `skeleton` (a lemma derived from another
/// lemma's letters): the same stem vowel, or the last vowel when the stress
/// sat on an ending the derived lemma no longer has (`свѧты́й` -> `свѧ́тъ`).
fn restress(accented: &str, skeleton: &str) -> String {
    let vowel = |c: char| "аеєиіїоѻѡуꙋыѣюꙗѧѵя".contains(c);
    let mut index = None;
    let mut seen = 0usize;
    for c in accented.nfd() {
        if vowel(c) {
            seen += 1;
        } else if matches!(c, '\u{301}' | '\u{300}' | '\u{311}') && index.is_none() {
            index = Some(seen.saturating_sub(1));
        }
    }
    let Some(index) = index else {
        return realise(skeleton, &SYN);
    };
    let vowels = skeleton.chars().filter(|c| vowel(*c)).count();
    realise(
        &stress(skeleton, index.min(vowels.saturating_sub(1)), false),
        &SYN,
    )
}

/// The cells one analysis attests, with the lemma that owns them.
type Attestation = (String, Vec<usize>);

/// Per `(part of speech, lemma, cell)`: the attesting forms as `(clitic,
/// frequency, print order, surface)`.
type Attested = BTreeMap<(Pos, String, usize), Vec<(bool, u64, usize, String)>>;

fn gather_polyakov_entry(entry: &polyakov::Entry, lexemes: &mut Lexemes, skips: &mut Skips) {
    // `SPRO` is the substantive pronoun (the personal matrix, the
    // reflexive, the кто̀/что̀ family), `APRO` the adjectival one (the
    // non-personal pronoun's short series; its long series is the
    // adjective's). The adverbial `ADVPRO` does not decline.
    let pos = match entry.tags.first().map(String::as_str) {
        Some("S" | "N") => Pos::Noun,
        Some("A") => Pos::Adj,
        Some("V") => Pos::Verb,
        Some("SPRO") => Pos::Pronoun,
        Some("APRO") => Pos::NPron,
        _ => {
            skips.skip("entry: part of speech outside the four tables");
            return;
        }
    };
    let Some(lemma) = polyakov_headword(&entry.lemma) else {
        skips.skip("entry: headword is not one word");
        return;
    };
    skips.entries += 1;
    let perfective = entry.tags.iter().any(|t| t == "pf" || t == "pf/ipf");
    let pronoun_person = match strip_marks(&lemma).as_str() {
        "азъ" | "мы" => Some(Person::First),
        "ты" | "вы" => Some(Person::Second),
        "и" => Some(Person::Third),
        _ => None,
    };
    // A substantive pronoun outside the personal matrix: the reflexive
    // себѐ (its own cells of the shared row) and the singular-only
    // кто̀/что̀ family (classes `PNkto`/`PNcto`), which declines as a
    // non-personal pronoun.
    let reflexive = pos == Pos::Pronoun && strip_marks(&lemma) == "себе";
    let substantive_npron = pos == Pos::Pronoun
        && pronoun_person.is_none()
        && (entry.class.starts_with("PNkto") || entry.class.starts_with("PNcto"));
    let series_lemmas = if pos == Pos::Adj || pos == Pos::NPron {
        adjective_series_lemmas(entry, &lemma)
    } else {
        BTreeMap::new()
    };
    let titlo_lemmas = titlo_lemmas(entry, pos);

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
        // The dictionary transcribes the print's erok — the jer-replacing
        // mark over a final consonant («нас̑» for «на́съ») — and its
        // consonant-borne abbreviation marks («нбс̑нѣй») with the kamora
        // (U+0311) on a consonant. Such a spelling is an abbreviation of
        // a form, not a form: the full spelling is attested beside it.
        if has_consonant_kamora(&surface) {
            skips.skip("form: erok or abbreviation mark on a consonant");
            continue;
        }
        skips.forms += 1;
        for set in &form.cells {
            let f = polyakov::features(set);
            let result: Result<(Pos, String, Vec<usize>), &'static str> = match pos {
                Pos::Noun => polyakov_noun_cells(&f).map(|c| (pos, lemma.clone(), c)),
                Pos::Adj => polyakov_adj_cells(&f, &series_lemmas).map(|(l, c)| (pos, l, c)),
                Pos::Verb => polyakov_verb_cells(&f, perfective, form.count)
                    .map(|c| (pos, lemma.clone(), c)),
                // The short series is the pronoun's row; the long series
                // declines as the adjective it is (part 0, decision 1).
                Pos::NPron => match f.series {
                    Some(Series::Long) => {
                        polyakov_adj_cells(&f, &series_lemmas).map(|(l, c)| (Pos::Adj, l, c))
                    }
                    Some(Series::Short) => match series_lemmas.get(&Series::Short) {
                        Some(l) => polyakov_npron_cells(&f, false).map(|c| (pos, l.clone(), c)),
                        None => Err("pronoun: short series without an attested masculine nominative"),
                    },
                    None => polyakov_npron_cells(&f, false).map(|c| (pos, lemma.clone(), c)),
                },
                Pos::Pronoun => match pronoun_person {
                    Some(person) => polyakov_pronoun_cells(&f, person, &surface)
                        .map(|c| (pos, PRONOUN_KEY.to_string(), c)),
                    None if reflexive => {
                        polyakov_reflexive_cells(&f).map(|c| (pos, PRONOUN_KEY.to_string(), c))
                    }
                    None if substantive_npron => {
                        polyakov_npron_cells(&f, true).map(|c| (Pos::NPron, lemma.clone(), c))
                    }
                    None => Err("pronoun: outside the personal matrix"),
                },
            };
            // A spelling under a titlo is its own lemma, keyed by the
            // abbreviation's citation form (`бг҃ъ`, `гл҃ати`).
            let result = match result {
                Ok((owner_pos, owner, cells)) if under_titlo(&surface) && owner_pos != Pos::Pronoun => {
                    match titlo_owner(&titlo_lemmas, f.series, &surface) {
                        Some(titlo) => Ok((owner_pos, titlo, cells)),
                        None => {
                            drop(owner);
                            Err("form: titlo spelling without a titlo citation form")
                        }
                    }
                }
                other => other,
            };
            match result {
                Ok((owner_pos, owner, cells)) => {
                    skips.mapped += 1;
                    for cell in cells {
                        attested
                            .entry((owner_pos, owner.clone(), cell))
                            .or_default()
                            .push((f.clitic, form.count, order, surface.clone()));
                    }
                }
                Err(reason) => skips.skip(reason),
            }
        }
    }

    let mut observations: BTreeMap<(Pos, String), Observation> = BTreeMap::new();
    for ((owner_pos, owner, cell), mut forms) in attested {
        // Corpus frequency decides the primary and print order breaks ties;
        // an enclitic (`мя`) never outranks the full form (`мене́`), as in
        // the grammar, where the clitics are the alternatives.
        forms.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        let obs = observations
            .entry((owner_pos, owner))
            .or_insert_with(|| Observation::new(owner_pos.arity()));
        for (_, _, _, surface) in forms {
            obs.attest(cell, &surface);
        }
    }
    for ((owner_pos, owner), obs) in observations {
        let key = LexemeKey {
            tag: tag(&SYN),
            pos: owner_pos,
            lemma: owner,
        };
        // Each entry is one lexeme (a homograph gets its own observation);
        // the personal pronoun is the one shared row.
        push_observation(lexemes, key, obs, owner_pos == Pos::Pronoun);
    }
}

/// A kamora (U+0311) carried by a consonant: Polyakov's transcription of
/// the erok and of a consonant-borne abbreviation mark; a vowel's kamora
/// is the print's plural mark and stays.
fn has_consonant_kamora(surface: &str) -> bool {
    let mut previous: Option<char> = None;
    for c in surface.nfd() {
        if c == '\u{311}' && previous.is_some_and(|p| !church_slavonic_core::orthography::is_vowel_letter(p)) {
            return true;
        }
        if c as u32 >= 0x300 && (c as u32) < 0x370 || (0x483..=0x489).contains(&(c as u32)) {
            continue;
        }
        previous = Some(c);
    }
    false
}

/// A spelling under a titlo (or a letter-titlo): the print's abbreviation of
/// a nomen sacrum (`бг҃ъ`, `гдⷭ҇ь`, `бл҃гослови́ти`).
fn under_titlo(surface: &str) -> bool {
    surface
        .nfd()
        .any(|c| matches!(c as u32, 0x0483 | 0x0487 | 0x2de0..=0x2dff))
}

/// The citation forms an entry attests under a titlo, per adjective series:
/// `(series, skeleton, canonical form, count)`. Each is the lemma of the
/// abbreviated spelling's paradigm — the rule engine carries the titlo over
/// the stem like any other mark of the citation form.
fn titlo_lemmas(entry: &polyakov::Entry, pos: Pos) -> Vec<(Option<Series>, String, String, u64)> {
    let mut out = Vec::new();
    for form in &entry.forms {
        if !under_titlo(&form.form) {
            continue;
        }
        let Some(canonical) = polyakov_key(&form.form) else {
            continue;
        };
        let series = form
            .cells
            .iter()
            .map(|set| polyakov::features(set))
            .filter(|f| match pos {
                Pos::Noun => {
                    f.number == Some(Number::Singular) && f.cases.contains(&Case::Nominative)
                }
                Pos::Verb => f.infinitive,
                Pos::Adj => {
                    !f.comparative
                        && f.number == Some(Number::Singular)
                        && f.gender == Some(Gender::Masculine)
                        && f.cases.contains(&Case::Nominative)
                }
                Pos::Pronoun | Pos::NPron => false,
            })
            .map(|f| f.series)
            .next();
        if let Some(series) = series {
            out.push((series, strip_marks(&canonical), canonical, form.count));
        }
    }
    out
}

/// The titlo lemma a titlo form belongs to: the same series, the longest
/// shared skeleton prefix, the more frequent on a tie (`гл҃ю` -> `гл҃ати`,
/// `гл҃го́лаше` -> `гл҃го́лати`).
fn titlo_owner(
    lemmas: &[(Option<Series>, String, String, u64)],
    series: Option<Series>,
    surface: &str,
) -> Option<String> {
    let skeleton = strip_marks(surface);
    lemmas
        .iter()
        .filter(|(s, ..)| *s == series || series.is_none())
        .map(|(_, candidate, canonical, count)| {
            let shared = skeleton
                .chars()
                .zip(candidate.chars())
                .take_while(|(a, b)| a == b)
                .count();
            (shared, *count, canonical)
        })
        .filter(|(shared, ..)| *shared > 0)
        .max_by_key(|(shared, count, _)| (*shared, *count))
        .map(|(_, _, canonical)| canonical.clone())
}

/// The lemma of each adjective series in an entry: the headword for its own
/// series; for the other, the most frequent attested masculine nominative
/// singular, else the nominative the paradigm class's legend gives it
/// ([`legend_nominative`]). A series with neither has no lemma and its forms
/// are skipped.
fn adjective_series_lemmas(entry: &polyakov::Entry, lemma: &str) -> BTreeMap<Series, String> {
    let headword: String = entry.lemma.nfc().collect();
    // A pronominal class (`PA2a`: мо́й, то́й) cites its SHORT form even
    // when it ends in -ой; an adjective class reads the ending.
    let pronominal = entry.class.starts_with('P');
    let headword_series = entry
        .forms
        .iter()
        .filter(|f| f.form.nfc().collect::<String>() == headword)
        .flat_map(|f| &f.cells)
        .map(|set| polyakov::features(set))
        .filter(|f| !f.comparative)
        .find_map(|f| f.series)
        .unwrap_or(if !pronominal && has_long_ending(lemma) {
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
    let lemma = strip_marks(lemma);
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
    legend_skeleton(class, &strip_marks(lemma), wanted).map(|s| realise(&restress(lemma, &s), &SYN))
}

fn legend_skeleton(class: &str, lemma: &str, wanted: Series) -> Option<String> {
    let first = class.split('/').next()?;
    if first.contains('*') {
        return None;
    }
    // The pronominal classes (`PA1k`, `PA2j`) pair their series like the
    // adjective classes they are named after.
    let first = first.strip_prefix('P').unwrap_or(first);
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

fn polyakov_verb_cells(
    f: &Features,
    perfective: bool,
    count: u64,
) -> Result<Vec<usize>, &'static str> {
    if f.infinitive {
        return Err("verb: infinitive (the lemma itself)");
    }
    if f.tense == Some(TenseTag::Perfect) {
        // The l-participle: nominative-only gender/number cells.
        let number = f.number.ok_or("verb: perfect without a number")?;
        let genders: Vec<Gender> = f.gender.map_or_else(|| GENDERS.to_vec(), |g| vec![g]);
        return Ok(genders
            .iter()
            .map(|g| l_participle_cell(g, &number))
            .collect());
    }
    if f.participle {
        // The declined-participle admission is corpus-frequency gated, like
        // the treebank train split: the dictionary is corpus-derived, and a
        // hapax declension cell is where its OCR and analysis noise lives —
        // and an ungated admission would balloon the tables past what CI
        // builds (~500 exception cells per misclassed verb).
        let voice = if f.voice == Some(Voice::Passive) {
            GVoice::Passive
        } else {
            GVoice::Active
        };
        let series = if f.series == Some(Series::Long) {
            GSeries::Long
        } else {
            GSeries::Short
        };
        let tense = match f.tense {
            Some(TenseTag::Present) => Tense::Present,
            Some(TenseTag::Future) if perfective => Tense::Present,
            Some(TenseTag::Past) => Tense::Aorist,
            _ => return Err("verb: participle without a tense"),
        };
        let number = f.number.ok_or("verb: participle without a number")?;
        let genders: Vec<Gender> = f.gender.map_or_else(|| GENDERS.to_vec(), |g| vec![g]);
        if f.cases.is_empty() {
            return Err("verb: participle without a case");
        }
        let citation = voice == GVoice::Active
            && series == GSeries::Short
            && number == Number::Singular
            && f.gender == Some(Gender::Masculine)
            && f.cases.contains(&Case::Nominative);
        let mut cells = Vec::new();
        // The declined cells are corpus-frequency gated: a hapax declension
        // reading is where the dictionary's OCR and analysis noise lives,
        // and ungated admission balloons the tables (~500 exception cells
        // per misclassed verb). The citation cells stay ungated, as before
        // the schema widening.
        if count >= POLYAKOV_PARTICIPLE_MIN_COUNT {
            for case in &f.cases {
                for gender in &genders {
                    cells.push(participle_cell(
                        &voice, &series, &tense, gender, &number, case,
                    ));
                }
            }
        }
        if citation {
            cells.push(if tense == Tense::Present { 36 } else { 37 });
        }
        if cells.is_empty() {
            return Err("verb: participle below the frequency gate");
        }
        return Ok(cells);
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

/// The non-personal pronoun cells of one analysis: an unspecified gender
/// attests every gender; the singular-only кто̀/что̀ family carries no
/// number tag and attests the singular of every gender (the rule answers
/// every cell the same six forms).
fn polyakov_npron_cells(f: &Features, singular_only: bool) -> Result<Vec<usize>, &'static str> {
    let number = match f.number {
        Some(n) => n,
        None if singular_only => Number::Singular,
        None => return Err("pronoun: no number"),
    };
    let genders: Vec<Gender> = f.gender.map_or_else(|| GENDERS.to_vec(), |g| vec![g]);
    let cells: Vec<usize> = f
        .cases
        .iter()
        .filter(|c| **c != Case::Vocative)
        .flat_map(|case| genders.iter().map(move |g| npron_cell(g, &number, case)))
        .collect();
    if cells.is_empty() {
        return Err("pronoun: no case");
    }
    Ok(cells)
}

/// The personal-matrix cells of one analysis. A `clit`-tagged form (мѧ̀,
/// мѝ, ны̀) attests the clitic cell of its person, number and case; a
/// third-person form that spells the rule's clitic for its cell («и҆̀»,
/// «ѧ҆̀», compared through [`comparison_key`] so civil «я» reaches it)
/// attests the clitic cell too — the dictionary carries no `clit` tag on
/// the anaphor.
fn polyakov_pronoun_cells(
    f: &Features,
    person: Person,
    surface: &str,
) -> Result<Vec<usize>, &'static str> {
    let number = f.number.ok_or("pronoun: no number")?;
    let genders: Vec<Gender> = match (person, f.gender) {
        (Person::Third, Some(g)) => vec![g],
        _ => GENDERS.to_vec(),
    };
    // The third-person headword is the anaphor «и҆̀», whose nominative the
    // language does not use — the ѻ҆́нъ series owns those cells (Alypy §47)
    // — and the dictionary's `nom/acc` bundles would otherwise attest
    // «и҆̀», «ꙗ҆̀», «є҆̀», «и҆́хъ» as nominatives.
    if person == Person::Third && f.cases.iter().all(|c| *c == Case::Nominative) {
        return Err("pronoun: the anaphor's nominative");
    }
    if f.clitic {
        let cells: Vec<usize> = f
            .cases
            .iter()
            .filter_map(|case| clitic_cell(&person, &number, &Gender::Masculine, case))
            .collect();
        if cells.is_empty() {
            return Err("pronoun: clitic outside the schema");
        }
        return Ok(cells);
    }
    let key = comparison_key(surface);
    let key = key.as_str();
    let cells: Vec<usize> = f
        .cases
        .iter()
        .filter(|c| **c != Case::Vocative)
        .filter(|c| !(person == Person::Third && **c == Case::Nominative))
        .flat_map(|case| {
            genders.iter().flat_map(move |g| {
                // A form spelling the rule's clitic attests the clitic
                // cell — and the full cell too where the clitic IS the full
                // form (ю҆̀, є҆̀, the dual and neuter-plural ѧ҆̀).
                let full = pronoun_cell(&person, &number, g, case);
                let clitic = ChurchSlavonicCore::clitic(&person, &number, g, case, &SYN)
                    .filter(|c| comparison_key(c) == key)
                    .and_then(|c| clitic_cell(&person, &number, g, case).map(|i| (i, c)));
                match clitic {
                    Some((i, c))
                        if comparison_key(ChurchSlavonicCore::pronoun(&person, &number, g, case, &SYN))
                            == comparison_key(c) =>
                    {
                        vec![i, full]
                    }
                    Some((i, _)) => vec![i],
                    None => vec![full],
                }
            })
        })
        .collect();
    if cells.is_empty() {
        return Err("pronoun: no case");
    }
    Ok(cells)
}

/// The reflexive's cells: its cases (no number), the `clit`-tagged forms
/// (сѧ̀, сѝ) at the reflexive clitic cells.
fn polyakov_reflexive_cells(f: &Features) -> Result<Vec<usize>, &'static str> {
    let cells: Vec<usize> = f
        .cases
        .iter()
        .filter(|c| !matches!(**c, Case::Vocative | Case::Nominative))
        .filter_map(|case| {
            if f.clitic {
                reflexive_clitic_cell(case)
            } else {
                Some(reflexive_cell(case))
            }
        })
        .collect();
    if cells.is_empty() {
        return Err("pronoun: reflexive outside the schema");
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
    fn observe(&mut self, forms: Vec<String>, raw: Vec<String>, soft: bool, primary: bool) {
        let sig = forms_sig(&forms);
        let is_new = !self.by_sig.contains_key(&sig);
        let c = self.by_sig.entry(sig).or_insert_with(|| Candidate {
            forms,
            raw: Vec::new(),
            soft_sense: false,
            primary: false,
            noise: 0,
        });
        c.primary |= primary;
        // Same-signature candidates are one candidate, but each may have
        // attested different raw cells (two rule-equal witnesses of
        // different cells): union the raws so the bare-shadow pass can
        // re-materialise every attested form. A conflicting cell keeps the
        // first witness, as before.
        if c.raw.is_empty() {
            c.raw = raw;
        } else {
            for (kept, new) in c.raw.iter_mut().zip(raw) {
                if kept.is_empty() {
                    *kept = new;
                }
            }
        }
        c.soft_sense = if is_new { soft } else { c.soft_sense && soft };
    }

    /// Handle the pattern with nothing left after subtracting the rules.
    /// A STANDARD regular observation is dropped and reserves the bare key
    /// for the rule engine. A SOFT one (an East spelling, the treebank train
    /// split) is a secondary witness: it must not demote a dictionary's
    /// exceptional row off the bare key, but it stays a candidate — where
    /// the bare row overrides the rule, the shadow pass re-materialises the
    /// rule-equal forms it attested, and an entirely redundant row is
    /// dropped at emission.
    fn drop_regular(&mut self, arity: usize) {
        let sig = forms_sig(&vec![String::new(); arity]);
        if self
            .by_sig
            .get(&sig)
            .is_some_and(|candidate| !candidate.soft_sense)
        {
            self.by_sig.remove(&sig);
            self.had_regular = true;
        }
    }
}

/// Derive the four participle stems of a verb candidate from its attested
/// declension, blank every cell the stem-expanded rule reproduces, and
/// store the stem in its reserved cell (542..546): a regular declension of
/// an irregular stem then costs four cells instead of five hundred. A stem
/// is accepted only when it explains at least two attested cells; the
/// cells it cannot explain stay stored individually.
fn infer_participle_stems(recension: &Recension, forms: &mut [String], raw: &mut [String]) {
    use church_slavonic_core::ChurchSlavonicCore;
    for (voice, past, marker) in [
        (GVoice::Active, false, 'щ'),
        (GVoice::Active, true, 'ш'),
        (GVoice::Passive, false, 'м'),
        (GVoice::Passive, true, 'н'),
    ] {
        let tense = if past { Tense::Aorist } else { Tense::Present };
        let stem_cell = participle_stem_cell(&voice, &tense);
        let block: Vec<(usize, GSeries, Gender, Number, Case)> = [GSeries::Short, GSeries::Long]
            .iter()
            .flat_map(|sr| {
                GENDERS.iter().flat_map(move |g| {
                    NUMBERS.iter().flat_map(move |n| {
                        CASES.iter().map(move |c| {
                            (
                                participle_cell(&voice, sr, &tense, g, n, c),
                                *sr,
                                *g,
                                *n,
                                *c,
                            )
                        })
                    })
                })
            })
            .collect();
        // Candidate stems: every prefix of an attested form ending at the
        // block's marker letter (`щ`, `ш`, `м`, `н` — plus `т` for the
        // t-participles).
        let mut candidates: Vec<String> = Vec::new();
        for (cell, ..) in &block {
            let f = &raw[*cell];
            if f.is_empty() {
                continue;
            }
            for (at, c) in f.char_indices() {
                if c == marker || (past && voice == GVoice::Passive && c == 'т') {
                    let stem = &f[..at + c.len_utf8()];
                    if !candidates.iter().any(|s| s == stem) {
                        candidates.push(stem.to_string());
                    }
                }
            }
            if candidates.len() >= 12 {
                break;
            }
        }
        let mut best: Option<(String, Vec<usize>)> = None;
        for stem in candidates {
            let covered: Vec<usize> = block
                .iter()
                .filter(|(cell, sr, g, n, c)| {
                    !raw[*cell].is_empty()
                        && ChurchSlavonicCore::participle_from_stem(
                            &stem, past, &voice, sr, c, n, g, recension,
                        )
                        .is_some_and(|d| rule_matches(recension, &raw[*cell], &d))
                })
                .map(|(cell, ..)| *cell)
                .collect();
            if covered.len() >= 2 && best.as_ref().is_none_or(|(_, b)| covered.len() > b.len()) {
                best = Some((stem, covered));
            }
        }
        if let Some((stem, covered)) = best {
            for cell in covered {
                forms[cell] = String::new();
            }
            // The runtime consults the stem BEFORE the rule, so a cell the
            // per-cell subtraction blanked as rule-predicted must be
            // re-stored when the stem would derive something else there —
            // otherwise the stem shadows the rule's correct answer.
            for (cell, sr, g, n, c) in &block {
                if raw[*cell].is_empty() || !forms[*cell].is_empty() {
                    continue;
                }
                if let Some(derived) = ChurchSlavonicCore::participle_from_stem(
                    &stem, past, &voice, sr, c, n, g, recension,
                ) && !rule_matches(recension, &raw[*cell], &derived)
                {
                    forms[*cell] = raw[*cell].clone();
                }
            }
            // The stem is part of the row's identity: the bare-shadow pass
            // compares raw against the bare row, so a variant's stem must
            // live in raw too or the pass would erase it.
            raw[stem_cell] = stem.clone();
            forms[stem_cell] = stem;
        }
    }
}

/// Infer a conjugation-class/present-stem override for a verb candidate:
/// candidate stems are read off the attested present cells by stripping
/// each class's endings, every hypothesis is scored by how many attested
/// finite/imperative/citation cells the re-run rule reproduces, and the
/// winner is adopted only when it beats the plain rule. Returns the
/// override-aware prediction row the caller must re-subtract against.
fn infer_verb_override(
    recension: &Recension,
    lemma: &str,
    predicted: &[String],
    raw: &[String],
) -> Option<(&'static str, String, Vec<String>)> {
    let synodal = *recension == Recension::Synodal;
    // (class, donor cell, ending) — 2sg, 3sg, 3pl per class.
    let donors: &[(Conj, usize, &str)] = if synodal {
        &[
            (Conj::Hard, 1, "еши"),
            (Conj::Hard, 2, "етъ"),
            (Conj::Hard, 8, "ꙋтъ"),
            (Conj::Iotated, 1, "еши"),
            (Conj::Iotated, 2, "етъ"),
            (Conj::Iotated, 8, "ꙋтъ"),
            (Conj::Vowel, 1, "еши"),
            (Conj::Vowel, 2, "етъ"),
            (Conj::Vowel, 8, "ютъ"),
            (Conj::Second, 1, "иши"),
            (Conj::Second, 2, "итъ"),
            (Conj::Second, 8, "ѧтъ"),
        ]
    } else {
        &[
            (Conj::Hard, 1, "еши"),
            (Conj::Hard, 2, "етъ"),
            (Conj::Hard, 8, "ѫтъ"),
            (Conj::Iotated, 1, "еши"),
            (Conj::Iotated, 2, "етъ"),
            (Conj::Iotated, 8, "ѫтъ"),
            (Conj::Vowel, 1, "ѥши"),
            (Conj::Vowel, 2, "ѥтъ"),
            (Conj::Vowel, 8, "ѭтъ"),
            (Conj::Second, 1, "иши"),
            (Conj::Second, 2, "итъ"),
            (Conj::Second, 8, "ѧтъ"),
        ]
    };
    let mut hypotheses: Vec<(Conj, String)> = Vec::new();
    for (conj, cell, ending) in donors {
        let form = &raw[*cell];
        if form.is_empty() {
            continue;
        }
        // An END-STRESSED donor spells the acute inside the ending
        // («стриже́ши», «дои́тъ») — strip that spelling too, or verbs whose
        // present is attested only with ending stress never hypothesize
        // (the стрищѝ/дои́ти defect of the v1.1 ledger). The scorer keeps
        // the guard: a hypothesis is adopted only when the re-run rule
        // reproduces MORE attested cells than the plain rule.
        let accented: String = {
            let mut out = String::new();
            let mut done = false;
            for c in ending.chars() {
                out.push(c);
                if !done && matches!(c, 'а' | 'е' | 'и' | 'о' | 'ꙋ' | 'ы' | 'ѣ' | 'ю' | 'ѧ') {
                    out.push('\u{0301}');
                    done = true;
                }
            }
            out
        };
        for suffix in [ending, accented.as_str()] {
            if let Some(stem) = form.strip_suffix(suffix)
                && !stem.is_empty()
                && !hypotheses.iter().any(|(c, st)| c == conj && st == stem)
            {
                hypotheses.push((*conj, stem.to_string()));
            }
        }
    }
    if hypotheses.is_empty() {
        return None;
    }
    let attested: Vec<usize> = (0..38).filter(|i| !raw[*i].is_empty()).collect();
    if attested.len() < 2 {
        return None;
    }
    // Score on the attested finite cells only — through the one resolution
    // engine; the full prediction row is computed once, for the winner.
    let realised = church_slavonic_core::orthography::realise(lemma, recension);
    let finite = |class: &str, present: &str, i: usize| {
        let fact = |cell: usize| -> Option<String> {
            if cell == VERB_CLASS_CELL {
                Some(class.to_string())
            } else if cell == PRESENT_STEM_CELL {
                Some(present.to_string())
            } else {
                None
            }
        };
        church_slavonic_core::orthography::realise(
            &church_slavonic_core::resolution::verb_fact_fallback(&realised, recension, i, &fact),
            recension,
        )
    };
    let plain = attested
        .iter()
        .filter(|i| rule_matches(recension, &raw[**i], &predicted[**i]))
        .count();
    let mut best: Option<(&'static str, String, usize)> = None;
    for (conj, stem) in hypotheses {
        let token = conj.token();
        let n = attested
            .iter()
            .filter(|i| rule_matches(recension, &raw[**i], &finite(token, &stem, **i)))
            .count();
        if n > plain && best.as_ref().is_none_or(|(.., b)| n > *b) {
            best = Some((token, stem, n));
        }
    }
    best.map(|(token, stem, _)| {
        let pred = predict_verb_override(lemma, Some(token), Some(&stem), recension);
        (token, stem, pred)
    })
}

/// The one resolution engine, dispatched by POS and wrapped in the
/// extractor's realise convention.
fn resolved_cell(
    pos: Pos,
    lemma_realised: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
) -> String {
    use church_slavonic_core::resolution as res;
    let raw = match pos {
        Pos::Noun => res::noun_fact_fallback(lemma_realised, recension, cell, fact),
        Pos::Adj => res::adj_fact_fallback(lemma_realised, recension, cell, fact),
        Pos::Verb => res::verb_fact_fallback(lemma_realised, recension, cell, fact),
        Pos::Pronoun => String::new(),
        Pos::NPron => res::npron_fact_fallback(lemma_realised, recension, cell, fact),
    };
    church_slavonic_core::orthography::realise(&raw, recension)
}

/// The stored FORM cells the resolution engine may read as facts for this
/// POS — the noun accusative-shape sources; empty elsewhere.
fn shape_sources(pos: Pos) -> &'static [usize] {
    use church_slavonic_core::schema as sch;
    match pos {
        Pos::Noun => &sch::NOUN_SHAPE_SOURCE_CELLS,
        _ => &[],
    }
}

/// The form-cell range and fact-cell range of a POS row.
fn fact_geometry(pos: Pos) -> Option<(usize, std::ops::Range<usize>)> {
    use church_slavonic_core::schema as sch;
    match pos {
        Pos::Noun => Some((sch::NOUN_ACCENT_CELL, 21..22)),
        Pos::Adj => Some((sch::ADJ_ACCENT_CELL, 126..127)),
        Pos::Verb => Some((sch::VERB_ACCENT_CELL, 542..549)),
        Pos::Pronoun | Pos::NPron => None,
    }
}

/// Infer a Synodal accent-pattern token for one candidate: every attested
/// accented form must stress the same stem vowel (`s<N>`) or its last
/// vowel (`e`), and the re-accented resolution must reproduce at least two
/// cells the current one does not. Mobile paradigms fit neither and stay
/// stored. OCS never adopts: its comparison is accent-blind, so a token
/// would be dead weight.
fn infer_accent_pattern(
    pos: Pos,
    recension: &Recension,
    lemma: &str,
    forms: &mut [String],
    raw: &mut [String],
    exact: &dyn Fn(usize) -> bool,
) {
    let matches = matches_for(pos);
    use church_slavonic_core::orthography::{realise, stressed_vowel_index, vowel_count};
    if *recension != Recension::Synodal {
        return;
    }
    let Some((accent_cell, fact_range)) = fact_geometry(pos) else {
        return;
    };
    let form_end = fact_range.start.min(accent_cell);
    let mut stem_n: Option<usize> = None;
    let (mut all_stem, mut all_end, mut seen) = (true, true, 0usize);
    for cell in 0..form_end {
        let f = &raw[cell];
        if f.is_empty() {
            continue;
        }
        let Some(k) = stressed_vowel_index(f) else {
            continue;
        };
        seen += 1;
        match stem_n {
            None => stem_n = Some(k),
            Some(n) if n == k => {}
            _ => all_stem = false,
        }
        if k + 1 != vowel_count(f) {
            all_end = false;
        }
    }
    if seen < 2 {
        return;
    }
    let mut candidates: Vec<String> = Vec::new();
    if all_stem && let Some(n) = stem_n {
        candidates.push(format!("s{n}"));
    }
    if all_end {
        candidates.push("e".to_string());
    }
    let realised = realise(lemma, recension);
    // Snapshot for the shape-source reads: the adoption below rewrites
    // `forms`, and the engine must see the pre-adoption stored row.
    let stored = forms.to_vec();
    let mut best: Option<(String, Vec<usize>)> = None;
    for token in candidates {
        let fact = |i: usize| -> Option<String> {
            if i == accent_cell {
                return Some(token.clone());
            }
            if shape_sources(pos).contains(&i) {
                return Some(stored[i].clone()).filter(|f| !f.is_empty());
            }
            fact_range
                .contains(&i)
                .then(|| raw[i].clone())
                .filter(|f| !f.is_empty())
        };
        let covered: Vec<usize> = (0..form_end)
            .filter(|cell| {
                !forms[*cell].is_empty()
                    && matches(
                        exact(*cell),
                        recension,
                        &raw[*cell],
                        &resolved_cell(pos, &realised, recension, *cell, &fact),
                    )
            })
            .collect();
        if covered.len() >= 2 && best.as_ref().is_none_or(|(_, b)| covered.len() > b.len()) {
            best = Some((token, covered));
        }
    }
    if let Some((token, _)) = best {
        // Adoption changes the resolution for EVERY cell, so the whole row
        // re-subtracts against it: a cell the accented resolution now
        // reproduces goes blank, and one it no longer reproduces (an
        // unaccented variant spelling a blanket token would orphan) is
        // stored.
        let fact = |i: usize| -> Option<String> {
            if i == accent_cell {
                return Some(token.clone());
            }
            if shape_sources(pos).contains(&i) {
                return Some(stored[i].clone()).filter(|f| !f.is_empty());
            }
            fact_range
                .contains(&i)
                .then(|| raw[i].clone())
                .filter(|f| !f.is_empty())
        };
        for cell in 0..form_end {
            forms[cell] = if raw[cell].is_empty()
                || matches(
                    exact(cell),
                    recension,
                    &raw[cell],
                    &resolved_cell(pos, &realised, recension, cell, &fact),
                ) {
                String::new()
            } else {
                raw[cell].clone()
            };
        }
        forms[accent_cell] = token.clone();
        raw[accent_cell] = token;
    }
}

/// Re-subtract the accusative cells the accusative-shape derivation now
/// reproduces (see `resolution::noun_fact_fallback` and
/// `schema::NOUN_SHAPE_SOURCE_CELLS`): a stored nominative-shaped lower
/// accusative makes the higher ones derivable, so a derivable cell whose
/// attestation the derived resolution reproduces goes blank. The anchor
/// (lowest stored accusative) never derives from itself and stays.
fn subtract_derived_accusatives(
    pos: Pos,
    recension: &Recension,
    lemma: &str,
    forms: &mut [String],
    raw: &[String],
) {
    if shape_sources(pos).is_empty() || *recension != Recension::Synodal {
        return;
    }
    let Some((accent_cell, fact_range)) = fact_geometry(pos) else {
        return;
    };
    let realised = church_slavonic_core::orthography::realise(lemma, recension);
    // The shape fact teaches in BOTH directions now (v1.1: an attested
    // nominative-shaped plural accusative teaches the singular), so any
    // two stored accusatives may predict each other. Fixpoint, highest
    // cell first: blank any stored accusative the REMAINING stored ones
    // reproduce, until stable — at least one always survives as the row's
    // anchor, and the survivor set is exactly what the runtime (and the
    // rule_table_sync audit) cannot predict from itself.
    loop {
        let stored: Vec<usize> = shape_sources(pos)
            .iter()
            .copied()
            .filter(|&c| !forms[c].is_empty() && !raw[c].is_empty())
            .collect();
        if stored.len() < 2 {
            return;
        }
        let mut blanked = false;
        for &cell in stored.iter().rev() {
            let snapshot = forms.to_vec();
            let fact = |i: usize| -> Option<String> {
                if shape_sources(pos).contains(&i) && i != cell {
                    return Some(snapshot[i].clone()).filter(|f| !f.is_empty());
                }
                (fact_range.contains(&i) || i == accent_cell)
                    .then(|| snapshot[i].clone())
                    .filter(|f| !f.is_empty())
            };
            if rule_matches(
                recension,
                &raw[cell],
                &resolved_cell(pos, &realised, recension, cell, &fact),
            ) {
                forms[cell] = String::new();
                blanked = true;
                break;
            }
        }
        if !blanked {
            return;
        }
    }
}

/// The audit's view of a row's full fact-aware resolution (0..form-cells),
/// realised — `None` when the POS has no fact cells or none are set.
pub fn audit_fact_resolution(
    pos: Pos,
    recension: &Recension,
    lemma: &str,
    cells: &[String],
    bare: Option<&[String]>,
) -> Option<Vec<String>> {
    let (_, fact_range) = fact_geometry(pos)?;
    let form_end = fact_range.start;
    // The noun engine also reads the stored lower accusatives as facts
    // (the accusative-shape derivation) — the audit's view must match the
    // runtime's.
    let sources = shape_sources(pos);
    let any = fact_range
        .clone()
        .chain(sources.iter().copied())
        .any(|i| !cells[i].is_empty() || bare.is_some_and(|b| !b[i].is_empty()));
    if !any {
        return None;
    }
    let fact = |i: usize| -> Option<String> {
        if !fact_range.contains(&i) && !sources.contains(&i) {
            return None;
        }
        if !cells[i].is_empty() {
            return Some(cells[i].clone());
        }
        bare.map(|b| b[i].clone()).filter(|c| !c.is_empty())
    };
    let realised = church_slavonic_core::orthography::realise(lemma, recension);
    Some(
        (0..form_end)
            .map(|i| resolved_cell(pos, &realised, recension, i, &fact))
            .collect(),
    )
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
            let variants = obs.cells.iter().map(Vec::len).max().unwrap_or(0);
            for k in 0..variants {
                let raw: Vec<String> = (0..arity)
                    .map(|i| {
                        let alts = &obs.cells[i];
                        alts.get(k).or(alts.first()).cloned().unwrap_or_default()
                    })
                    .collect();
                let mut forms: Vec<String> = raw
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if f.is_empty() || matches_for(key.pos)(obs.is_exact(i, f), &recension, f, &predicted[i]) {
                            String::new()
                        } else {
                            f.clone()
                        }
                    })
                    .collect();
                let mut raw = raw;
                if key.pos == Pos::Verb {
                    if let Some((class, stem, predicted_ov)) =
                        infer_verb_override(&recension, &key.lemma, &predicted, &raw)
                    {
                        // Re-subtract against what the overridden rule will
                        // actually answer, and carry the override in raw so
                        // the bare-shadow pass keeps the row's identity.
                        for i in 0..542 {
                            forms[i] = if raw[i].is_empty()
                                || rule_matches(&recension, &raw[i], &predicted_ov[i])
                            {
                                String::new()
                            } else {
                                raw[i].clone()
                            };
                        }
                        forms[VERB_CLASS_CELL] = class.to_string();
                        forms[PRESENT_STEM_CELL] = stem.clone();
                        raw[VERB_CLASS_CELL] = class.to_string();
                        raw[PRESENT_STEM_CELL] = stem;
                    }
                    infer_participle_stems(&recension, &mut forms, &mut raw);
                }
                let exact_cells: Vec<bool> =
                    (0..arity).map(|i| obs.is_exact(i, &raw[i])).collect();
                infer_accent_pattern(key.pos, &recension, &key.lemma, &mut forms, &mut raw, &|i| {
                    exact_cells[i]
                });
                subtract_derived_accusatives(key.pos, &recension, &key.lemma, &mut forms, &raw);
                // The personal pronoun is ONE shared row whose primaries
                // the print arbitrates (Alypy's order, the witnesses); the
                // sort must not hand its bare key to a row of second
                // choices (v1.2 part 1). A lemma-keyed row keeps the plain
                // form sort: Polyakov's counts are per form, not per cell,
                // so at a tag-bundled cell (`pl,nom/acc`) the "primary" is
                // the nominative's frequency wearing the accusative's tag
                // (га́ды over гадѡ́въ) — the Bible-as-source design is the
                // place to measure a cell's true primary.
                acc.observe(forms, raw, obs.soft, k == 0 && key.pos == Pos::Pronoun);
            }
        }
        acc.drop_regular(arity);
        let had_regular = acc.had_regular;
        let mut candidates: Vec<Candidate> = acc.by_sig.into_values().collect();
        // A verb whose forms arrived as single-cell observations (Polyakov
        // form-of entries) never had two attested cells IN ONE observation,
        // so the per-observation override inference above could not fire —
        // the стрищѝ/дои́ти defect of the v1.1 ledger. Re-infer over each
        // candidate's UNIONED raw cells; adopting the override re-subtracts
        // the forms exactly as the in-observation path does.
        // The same single-cell-observation gap holds for the accusative
        // shape: the two accusatives that teach each other may arrive in
        // separate observations, so the per-observation subtraction above
        // never saw them together — re-subtract over the unioned rows.
        for cand in &mut candidates {
            let raw = cand.raw.clone();
            subtract_derived_accusatives(key.pos, &recension, &key.lemma, &mut cand.forms, &raw);
        }
        if key.pos == Pos::Verb {
            for cand in &mut candidates {
                if !cand.forms[VERB_CLASS_CELL].is_empty()
                    || !cand.forms[PRESENT_STEM_CELL].is_empty()
                {
                    continue;
                }
                if let Some((class, stem, predicted_ov)) =
                    infer_verb_override(&recension, &key.lemma, &predicted, &cand.raw)
                {
                    for i in 0..542 {
                        cand.forms[i] = if cand.raw[i].is_empty()
                            || rule_matches(&recension, &cand.raw[i], &predicted_ov[i])
                        {
                            String::new()
                        } else {
                            cand.raw[i].clone()
                        };
                    }
                    cand.forms[VERB_CLASS_CELL] = class.to_string();
                    cand.forms[PRESENT_STEM_CELL] = stem.clone();
                    cand.raw[VERB_CLASS_CELL] = class.to_string();
                    cand.raw[PRESENT_STEM_CELL] = stem;
                }
            }
        }
        // A stored Synodal form with no stress mark is a transliteration's
        // dropped accent (the lemma is accented; a titlo abbreviation is
        // not counted): the row that carries it must not outrank the clean
        // rows in the bare-key sort.
        if recension == Recension::Synodal
            && church_slavonic_core::orthography::is_accented(&key.lemma)
        {
            use church_slavonic_core::orthography::{is_accented, vowel_count};
            for cand in &mut candidates {
                cand.noise = cand
                    .forms
                    .iter()
                    .enumerate()
                    .filter(|(i, f)| {
                        !f.is_empty()
                            && fact_geometry(key.pos).is_none_or(|(a, r)| *i != a && !r.contains(i))
                            && vowel_count(f) > 0
                            && !is_accented(f)
                            && !f.contains('\u{483}')
                    })
                    .count();
            }
        }
        let mut assigned = assign(&key.lemma, candidates, had_regular);
        // The runtime reads a `_n` blank from the bare row before the rule,
        // so where the bare row holds a cell, a variant row carries exactly
        // its own attested form when that differs — even one the rule would
        // predict, which the bare row would otherwise shadow — and nothing
        // when it is the same.
        if let Some((bare, rest)) = assigned.split_first_mut()
            && bare.key == key.lemma
        {
            for row in rest {
                for cell in 0..arity {
                    if bare.forms[cell].is_empty() {
                        continue;
                    }
                    row.forms[cell] = if row.raw[cell] == bare.forms[cell] {
                        String::new()
                    } else {
                        row.raw[cell].clone()
                    };
                }
            }
        }
        // A participle cell blanked as rule-predicted stays reachable only
        // through a published row whose stem does not shadow the rule; when
        // this row would vanish (or its stem derives something else),
        // re-store the attested form.
        if let Some((_, fact_range)) = fact_geometry(key.pos) {
            let form_end = fact_range.start;
            let realised = church_slavonic_core::orthography::realise(&key.lemma, &recension);
            // A transliterated spelling is judged under what it can
            // encode here too (see `attested_matches`).
            let matches = matches_for(key.pos);
            let is_exact = |i: usize, f: &str| observations.iter().any(|o| o.is_exact(i, f));
            // The bare row is resolved FIRST and the snapshots taken after:
            // the forward pass below can re-store bare cells, and every
            // variant must reason from the bare row the runtime will see.
            let snapshot = |assigned: &[crate::assign::Assignment]| {
                let bare = assigned.first().filter(|b| b.key == key.lemma);
                (
                    bare.map(|b| b.forms[fact_range.clone()].to_vec())
                        .unwrap_or_else(|| vec![String::new(); fact_range.len()]),
                    bare.map(|b| b.forms[..form_end].to_vec())
                        .unwrap_or_else(|| vec![String::new(); form_end]),
                )
            };
            let (mut bare_facts, mut bare_exact) = snapshot(&assigned);
            let row_count = assigned.len();
            for index in 0..row_count {
                if index == 1 {
                    // The bare row just settled; variants see its final state.
                    (bare_facts, bare_exact) = snapshot(&assigned);
                }
                let row = &mut assigned[index];
                let is_bare = row.key == key.lemma;
                let facts: Vec<String> = fact_range
                    .clone()
                    .map(|i| {
                        if row.forms[i].is_empty() {
                            bare_facts[i - fact_range.start].clone()
                        } else {
                            row.forms[i].clone()
                        }
                    })
                    .collect();
                // The shape-source cells the engine may read, own-else-bare
                // — read LIVE from the row so a cell this pass re-stores
                // immediately teaches the next (the bidirectional shape
                // fact makes restoring one accusative derive the others;
                // a stale snapshot would re-store them all, v1.1).
                let source_of = |forms: &[String], i: usize| -> String {
                    let own = &forms[i];
                    if !own.is_empty() {
                        own.clone()
                    } else if !is_bare {
                        bare_exact[i].clone()
                    } else {
                        String::new()
                    }
                };
                let resolved = |forms: &[String], cell: usize| {
                    let sources: Vec<(usize, String)> = shape_sources(key.pos)
                        .iter()
                        .map(|i| (*i, source_of(forms, *i)))
                        .collect();
                    let fact = |i: usize| -> Option<String> {
                        if let Some((_, f)) = sources.iter().find(|(c, _)| *c == i) {
                            return Some(f.clone()).filter(|f| !f.is_empty());
                        }
                        facts
                            .get(i.checked_sub(fact_range.start)?)
                            .filter(|f| !f.is_empty())
                            .cloned()
                    };
                    resolved_cell(key.pos, &realised, &recension, cell, &fact)
                };
                // Forward: a blank cell is re-stored when the runtime's
                // resolution for this key will NOT reproduce its
                // attestation — a differing bare exact cell shadows the
                // facts, and the facts must otherwise derive it.
                for cell in 0..form_end {
                    if row.forms[cell].is_empty() && !row.raw[cell].is_empty() {
                        let reproduced = if !is_bare && !bare_exact[cell].is_empty() {
                            bare_exact[cell] == row.raw[cell]
                        } else {
                            matches(is_exact(cell, &row.raw[cell]), &recension, &row.raw[cell], &resolved(&row.forms, cell))
                        };
                        if !reproduced {
                            row.forms[cell] = row.raw[cell].clone();
                        }
                    }
                }
                // Reverse: a stored cell the fallback already reproduces
                // (and the bare row does not shadow) is dead weight.
                for cell in 0..form_end {
                    if !is_bare
                        && !row.forms[cell].is_empty()
                        && bare_exact[cell].is_empty()
                        && matches(is_exact(cell, &row.forms[cell]), &recension, &row.forms[cell], &resolved(&row.forms, cell))
                    {
                        row.forms[cell] = String::new();
                    }
                }
            }
        }
        for a in assigned {
            // A row can subtract and shadow down to nothing (every variant
            // of every cell matched the rule or the bare row); an empty row
            // says nothing and fails the registry audit.
            if a.forms.iter().all(String::is_empty) {
                continue;
            }
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
        assert_eq!(row.len(), Pos::Noun.arity());
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
            ("syn", Pos::Noun, "а҆арѡ́нъ")
        );
        let cell = |c: Case| obs[0].cells[noun_cell(&c, &Number::Singular)].clone();
        assert_eq!(cell(Case::Genitive), ["а҆арѡ́на"]);
        assert_eq!(cell(Case::Accusative), ["а҆арѡ́на"]);
        assert_eq!(cell(Case::Dative), ["а҆арѡ́нꙋ", "а҆арѡ́нови"]);
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
        assert_eq!(lemmas, ["багрѧ́нъ", "багрѧ́ный"]);
        let short = &lexemes.values().next().expect("short")[0];
        let long = &lexemes.values().nth(1).expect("long")[0];
        let gen_pl = |g: Gender| {
            adj_cell(&Case::Genitive, &Number::Plural, &g, &Degree::Positive).expect("cell")
        };
        assert_eq!(short.cells[gen_pl(Gender::Neuter)], ["багрѧ́ныхъ"]);
        assert_eq!(long.cells[gen_pl(Gender::Feminine)], ["багрѧ́ныхъ"]);
        let comp = adj_cell(
            &Case::Nominative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Comparative,
        )
        .expect("cell");
        assert_eq!(short.cells[comp], ["багрѧ́нѣй"]);
        assert!(long.cells[comp].is_empty());

        // No attested short nominative: the class legend spells it (A1k: -ій ~ -ъ);
        // a starred class (fleeting vowel) spells nothing and the series is skipped.
        for (class, expect) in [("A1k", Some("вели́къ")), ("A1t*", None)] {
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
                Some(short) => assert_eq!(lemmas, [short, "вели́кій"]),
                None => {
                    assert_eq!(lemmas, ["вели́кій"]);
                    assert_eq!(
                        skips.by_reason.get(
                            "adjective: short series without an attested masculine nominative"
                        ),
                        Some(&3)
                    );
                }
            }
        }
        assert_eq!(
            legend_nominative("A2t", "а҆арѡ́новъ", Series::Long).as_deref(),
            Some("а҆арѡ́новый")
        );
        assert_eq!(
            legend_nominative("A2j", "а҆арѡ́нь", Series::Long).as_deref(),
            Some("а҆арѡ́ній")
        );
        assert_eq!(
            legend_nominative("A1i", "бо́жій", Series::Long).as_deref(),
            Some("бо́жій")
        );
        // A long lemma stressed on its ending gives a stem-stressed short one.
        assert_eq!(
            legend_nominative("A1t", "свѧты́й", Series::Short).as_deref(),
            Some("свѧ́тъ")
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
            ("да́вый", "partcp,praet,act,plen,sg,m/n,nom", 5),
            ("да́въ", "partcp,praet,act,brev,sg,m/n,nom", 6),
            ("да́вша", "partcp,praet,act,brev,sg,m/n,gen/acc", 5),
            ("да́ный", "partcp,praet,pass,plen,sg,m,nom/acc", 5),
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
        assert_eq!(key.lemma, "да́ти");
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
        // The declined participle cells: `да́вый` long active m nom sg,
        // `да́вша` short active m/n gen (and m acc) sg, `да́ный` long
        // passive m nom/acc sg.
        let pcell = |v: GVoice, sr: GSeries, g: Gender, n: Number, c: Case| {
            obs[0].cells[participle_cell(&v, &sr, &Tense::Aorist, &g, &n, &c)].clone()
        };
        use Case::*;
        assert_eq!(
            pcell(
                GVoice::Active,
                GSeries::Long,
                Gender::Masculine,
                Number::Singular,
                Nominative
            ),
            ["да́вый"]
        );
        assert_eq!(
            pcell(
                GVoice::Active,
                GSeries::Short,
                Gender::Neuter,
                Number::Singular,
                Genitive
            ),
            ["да́вша"]
        );
        assert_eq!(
            pcell(
                GVoice::Passive,
                GSeries::Long,
                Gender::Masculine,
                Number::Singular,
                Accusative
            ),
            ["да́ный"]
        );
        for (reason, n) in [("verb: infinitive (the lemma itself)", 1)] {
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
        let kto = LexemeKey { tag: "syn", pos: Pos::NPron, lemma: "кто̀".into() };
        assert_eq!(
            lexemes[&kto][0].cells[npron_cell(&Gender::Feminine, &Number::Singular, &Case::Accusative)],
            ["кого̀"]
        );
        let (key, obs) = lexemes.iter().find(|(k, _)| k.pos == Pos::Pronoun).expect("row");
        assert_eq!(key.lemma, PRONOUN_KEY);
        assert_eq!(obs.len(), 1);
        let cell = |p: Person, g: Gender, c: Case| {
            obs[0].cells[pronoun_cell(&p, &Number::Singular, &g, &c)].clone()
        };
        assert_eq!(cell(Person::First, Gender::Feminine, Case::Accusative), ["менѐ"]);
        let clitic = clitic_cell(&Person::First, &Number::Singular, &Gender::Feminine, &Case::Accusative);
        assert_eq!(obs[0].cells[clitic.expect("cell")], ["мѧ̀"]);
        assert_eq!(cell(Person::Third, Gender::Neuter, Case::Genitive), ["є҆гѡ̀"]);
        assert_eq!(
            cell(Person::Third, Gender::Feminine, Case::Genitive),
            ["є҆ѧ̀"]
        );
        assert_eq!(skips.by_reason.get("pronoun: outside the personal matrix"), None);
    }

    #[test]
    fn alypy_merges_into_the_polyakov_observation_and_disagreements_are_counted() {
        let dat = noun_cell(&Case::Dative, &Number::Singular);
        let genitive = noun_cell(&Case::Genitive, &Number::Singular);
        let polyakov = lexeme(
            "syn",
            Pos::Noun,
            "рабъ",
            &[(dat, &["рабꙋ̀"]), (genitive, &["раба̀"])],
        );
        let mut both = polyakov.clone();
        let alypy_obs = {
            let l = lexeme(
                "syn",
                Pos::Noun,
                "рабъ",
                &[(dat, &["рабꙋ̀"]), (genitive, &["ра́ба"])],
            );
            l.into_values().next().expect("obs").remove(0)
        };
        let key = both.keys().next().expect("key").clone();
        push_observation(&mut both, key, alypy_obs.clone(), true);
        let obs = &both.values().next().expect("obs");
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].cells[dat], ["рабꙋ̀"]);
        assert_eq!(obs[0].cells[genitive], ["раба̀", "ра́ба"]);
        let mut alypy = Lexemes::new();
        alypy.insert(
            polyakov.keys().next().expect("key").clone(),
            vec![alypy_obs],
        );
        // The genitives differ only by their accent.
        assert_eq!(disagreements(&alypy, &polyakov), (1, 0));
        let t = finalize(&both);
        let keys: Vec<&str> = t.noun.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["syn:рабъ", "syn:рабъ_2"]);
        // The ending-stressed cells collapsed onto an accent-pattern token;
        // the variant row repeats nothing the bare row resolves.
        assert_eq!(
            t.noun[0].1[church_slavonic_core::schema::NOUN_ACCENT_CELL],
            "s1"
        );
        assert_eq!(t.noun[0].1[dat], "");
        assert_eq!(t.noun[1].1[dat], "");
        assert_eq!(t.noun[1].1[genitive], "ра́ба");
    }
}
