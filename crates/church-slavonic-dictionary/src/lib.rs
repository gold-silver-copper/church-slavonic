//! Optional Church Slavonic dictionary for the rewrite pilot: Wiktionary
//! sense lookup and gloss access re-keyed onto the `church-slavonic` facade's
//! lemma keys, plus form -> lemma lookup ([`lemmatize`]) built by inverting
//! the facade's paradigm enumeration.
//!
//! # Data
//!
//! The generated sorted-slice tables in `generated/entries.rs` are emitted by
//! `cargo xtask rewrite-dictionary` from `data/dictionary/senses.json` — the
//! same pinned Kaikki/Wiktionary extraction the old
//! `old-church-slavonic-dictionary` generated table came from — in the same
//! style as the facade's residue tables (sorted static slices, binary
//! search, no runtime I/O). Examples are not carried into this pilot table
//! (the old crate keeps them); glosses, raw glosses, tags, and topics are.
//!
//! # Lemma keys and homographs
//!
//! Every sense that the source pins to one extracted lexeme
//! (`inflection_lexeme_id`) carries [`Sense::lemma_key`]: the exact key the
//! `church-slavonic` facade serves that lexeme under, including the
//! deterministic numeric homograph suffixes (`градъ`, `градъ_2`, ...). The
//! mapping rule is: the generator replays the facade emitter's own homograph
//! sort — lexemes sharing a lemma ordered by their emitted form inventories
//! (the sorted `(cell code, variant list)` sequence), with the encoded
//! lexeme metadata and finally the extracted lexeme id as tie-breaks — so a
//! lexeme id lands on exactly the suffix the residue tables assigned it.
//! Senses the source leaves unpinned (no lexeme id, e.g. particles or
//! undisambiguated homograph pages) have `lemma_key() == None`; they remain
//! reachable through [`lookup`] by spelling.
//!
//! # Lemmatization index
//!
//! [`lemmatize`] inverts the facade's paradigm enumeration
//! (`noun_paradigm`, `adjective_paradigm`, `verb_paradigm`, and the closed
//! `*_form_paradigm` functions, plus the person-indexed personal table and
//! the reflexive) over the generated lemma-key inventory (`LEMMAS`). The
//! inverted index is built **lazily on first use** rather than emitted at
//! generation time: enumerating every paradigm takes ~0.16 s (release) /
//! ~1.6 s (debug) once per process (warm lookups are then ~0.3 us each),
//! while a generated index would add roughly 101k surface rows / several
//! megabytes of `.rs` source — over the rewrite plan's 5k-line
//! generated-file cap and dwarfing the sense table itself — to save that
//! one-time cost. The gate (`cargo xtask check-structure`) replays the
//! full round-trip either way.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;
use std::sync::OnceLock;

pub use church_slavonic::VerbCellKind;
pub use church_slavonic_core::grammar::{AdjectiveForm, Case, Gender, Number, Person};
pub use church_slavonic_core::recension::Recension;
use church_slavonic_orthography::projection;
use old_church_slavonic_core::orthography;

/// One generated sense row. Field layout mirrors the emitter in
/// `xtask/src/rewrite_dictionary.rs`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SenseRecord {
    pub id: &'static str,
    pub source_sense_id: &'static str,
    pub lemma: &'static str,
    pub page_word: &'static str,
    pub key: &'static str,
    pub page_key: &'static str,
    pub part_of_speech: &'static str,
    pub lemma_key: Option<&'static str>,
    pub glosses: &'static [&'static str],
    pub raw_glosses: &'static [&'static str],
    pub tags: &'static [&'static str],
    pub topics: &'static [&'static str],
}

mod generated {
    use super::SenseRecord;
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/entries.rs"));
}

/// The exact machine-readable source represented by this release (identical
/// provenance to `old-church-slavonic-dictionary`).
pub const SOURCE_NAME: &str = "English Wiktionary Old Church Slavonic via Kaikki";
pub const SOURCE_LICENSE: &str = "CC BY-SA 4.0";

/// Dictionary query errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictionaryError {
    EmptyQuery,
    InvalidQuery(String),
    /// The requested recension is not one the dictionary serves (only the
    /// two attested recensions are; docs/UNIFIED_FACADE.md).
    UnsupportedRecension(Recension),
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuery => formatter.write_str("the dictionary query is empty"),
            Self::InvalidQuery(reason) => write!(formatter, "invalid dictionary query: {reason}"),
            Self::UnsupportedRecension(recension) => {
                write!(formatter, "unsupported recension {recension:?}")
            }
        }
    }
}

impl StdError for DictionaryError {}

/// Part of speech of a facade lemma key, following the facade's paradigm
/// families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pos {
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Numeral,
    Determiner,
}

impl Pos {
    fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::Noun,
            2 => Self::Adjective,
            3 => Self::Verb,
            4 => Self::Pronoun,
            5 => Self::Numeral,
            6 => Self::Determiner,
            _ => return None,
        })
    }
}

/// One dictionary sense (a single Wiktionary sense, kept independent).
#[derive(Debug, Clone, Copy)]
pub struct Sense {
    record: &'static SenseRecord,
}

fn nonempty(value: &'static str) -> Option<&'static str> {
    (!value.is_empty()).then_some(value)
}

impl Sense {
    /// Stable sense id (source sense id plus content hash).
    pub fn id(self) -> &'static str {
        self.record.id
    }

    pub fn source_sense_id(self) -> Option<&'static str> {
        nonempty(self.record.source_sense_id)
    }

    /// Canonical lemma spelling as the source gives it (may carry accents).
    pub fn lemma(self) -> &'static str {
        self.record.lemma
    }

    /// The spelling used as the source page title.
    pub fn source_spelling(self) -> &'static str {
        self.record.page_word
    }

    pub fn part_of_speech(self) -> &'static str {
        self.record.part_of_speech
    }

    /// The `church-slavonic` facade lemma key serving this sense's lexeme
    /// (numeric homograph suffix included), when the source pins one. This
    /// key resolves through the facade's paradigm functions and is the same
    /// key [`lemmatize`] reports.
    pub fn lemma_key(self) -> Option<&'static str> {
        self.record.lemma_key
    }

    /// Normalized lookup key of the canonical lemma.
    pub fn lookup_key(self) -> &'static str {
        self.record.key
    }

    /// Normalized lookup key of the source page title.
    pub fn page_key(self) -> &'static str {
        self.record.page_key
    }

    pub fn glosses(self) -> &'static [&'static str] {
        self.record.glosses
    }

    pub fn raw_glosses(self) -> &'static [&'static str] {
        self.record.raw_glosses
    }

    pub fn tags(self) -> &'static [&'static str] {
        self.record.tags
    }

    pub fn topics(self) -> &'static [&'static str] {
        self.record.topics
    }
}

fn normalize_query(query: &str) -> Result<String, DictionaryError> {
    if query.trim().is_empty() {
        return Err(DictionaryError::EmptyQuery);
    }
    orthography::lookup_key(query).map_err(|error| DictionaryError::InvalidQuery(error.to_string()))
}

/// Range of `SENSES` rows whose `key` equals `key` (the slice is sorted by
/// `(key, id)`).
fn key_range(key: &str) -> &'static [SenseRecord] {
    let senses = generated::SENSES;
    let start = senses.partition_point(|record| record.key < key);
    let end = senses.partition_point(|record| record.key <= key);
    &senses[start..end]
}

/// Look up every sense filed under a lemma: the query is normalized to the
/// dictionary lookup key and matched against both the canonical lemma key
/// and the source page spelling (so accentless page titles keep working).
pub fn lookup(query: &str) -> Result<Vec<Sense>, DictionaryError> {
    let key = normalize_query(query)?;
    let mut indices: Vec<usize> = Vec::new();
    let senses = generated::SENSES;
    let base = senses.as_ptr() as usize;
    for record in key_range(&key) {
        indices.push((record as *const SenseRecord as usize - base) / size_of::<SenseRecord>());
    }
    let start = generated::PAGE_INDEX.partition_point(|(page, _)| *page < key.as_str());
    for (page, index) in &generated::PAGE_INDEX[start..] {
        if *page != key {
            break;
        }
        indices.push(*index as usize);
    }
    indices.sort_unstable();
    indices.dedup();
    Ok(indices
        .into_iter()
        .map(|index| Sense {
            record: &senses[index],
        })
        .collect())
}

/// Every sense whose [`Sense::lemma_key`] is exactly this facade lemma key
/// (homograph suffix included).
pub fn senses_for_lemma_key(lemma_key: &str) -> Vec<Sense> {
    let start = generated::LEMMA_KEY_INDEX.partition_point(|(key, _)| *key < lemma_key);
    generated::LEMMA_KEY_INDEX[start..]
        .iter()
        .take_while(|(key, _)| *key == lemma_key)
        .map(|(_, index)| Sense {
            record: &generated::SENSES[*index as usize],
        })
        .collect()
}

/// Fetch one sense by its stable id.
pub fn sense_by_id(id: &str) -> Option<Sense> {
    generated::SENSES
        .iter()
        .find(|record| record.id == id)
        .map(|record| Sense { record })
}

/// Iterate every sense in the dictionary (sorted by lookup key, then id).
pub fn senses() -> impl Iterator<Item = Sense> {
    generated::SENSES.iter().map(|record| Sense { record })
}

/// Iterate every facade lemma key the lemmatization index covers, with its
/// part of speech. These are exactly the `church-slavonic` pilot's lemma
/// keys (numeric homograph suffixes included).
pub fn lemmas() -> impl Iterator<Item = (&'static str, Pos)> {
    generated::LEMMAS
        .iter()
        .filter_map(|(lemma, code)| Pos::from_code(*code).map(|pos| (*lemma, pos)))
}

/// The typed paradigm cell that produces a surface form — the public
/// counterpart of the facade's paradigm enumeration keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    /// A noun declension cell.
    Noun { case: Case, number: Number },
    /// An adjective cell (long or short declension).
    Adjective {
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
    },
    /// A verb cell (finite, imperative, l-participle, or citation).
    Verb(VerbCellKind),
    /// A lemma-keyed closed-class cell (pronoun / numeral / determiner);
    /// `gender` is `Some` for gender-indexed lexemes, `None` for bare ones.
    Closed {
        case: Case,
        number: Number,
        gender: Option<Gender>,
    },
    /// A person-indexed personal-pronoun cell (canonical lemmas `азъ`, `тꙑ`).
    Personal {
        person: Person,
        number: Number,
        case: Case,
    },
    /// The numberless reflexive `сѧ`.
    Reflexive { case: Case },
}

/// One lemmatization reading: the facade lemma key (homograph suffix
/// included), its part of speech, and the typed cell producing the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reading {
    pub lemma: &'static str,
    pub pos: Pos,
    pub cell: Cell,
}

type Index = BTreeMap<String, Vec<Reading>>;

fn insert_surface(index: &mut Index, surface: &str, reading: Reading) {
    let Ok(key) = orthography::lookup_key(surface) else {
        return;
    };
    let readings = index.entry(key).or_default();
    if !readings.contains(&reading) {
        readings.push(reading);
    }
}

fn build_index() -> Index {
    let mut index = Index::new();
    for (lemma, pos) in lemmas() {
        match pos {
            Pos::Noun => {
                if let Ok(paradigm) = church_slavonic::noun_paradigm(lemma) {
                    for (case, number, variants) in paradigm {
                        for surface in &variants {
                            insert_surface(
                                &mut index,
                                surface,
                                Reading {
                                    lemma,
                                    pos,
                                    cell: Cell::Noun { case, number },
                                },
                            );
                        }
                    }
                }
            }
            Pos::Adjective => {
                for form in [AdjectiveForm::Long, AdjectiveForm::Short] {
                    if let Ok(paradigm) = church_slavonic::adjective_paradigm(lemma, form) {
                        for (case, number, gender, variants) in paradigm {
                            for surface in &variants {
                                insert_surface(
                                    &mut index,
                                    surface,
                                    Reading {
                                        lemma,
                                        pos,
                                        cell: Cell::Adjective {
                                            form,
                                            case,
                                            number,
                                            gender,
                                        },
                                    },
                                );
                            }
                        }
                    }
                }
            }
            Pos::Verb => {
                if let Ok(paradigm) = church_slavonic::verb_paradigm(lemma) {
                    for (kind, variants) in paradigm {
                        for surface in &variants {
                            insert_surface(
                                &mut index,
                                surface,
                                Reading {
                                    lemma,
                                    pos,
                                    cell: Cell::Verb(kind),
                                },
                            );
                        }
                    }
                }
            }
            Pos::Pronoun | Pos::Numeral | Pos::Determiner => {
                let paradigm = match pos {
                    Pos::Pronoun => church_slavonic::pronoun_form_paradigm(lemma),
                    Pos::Numeral => church_slavonic::numeral_form_paradigm(lemma),
                    _ => church_slavonic::determiner_form_paradigm(lemma),
                };
                if let Ok(paradigm) = paradigm {
                    for (case, number, gender, variants) in paradigm {
                        for surface in &variants {
                            insert_surface(
                                &mut index,
                                surface,
                                Reading {
                                    lemma,
                                    pos,
                                    cell: Cell::Closed {
                                        case,
                                        number,
                                        gender,
                                    },
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    // Person-indexed personal cells (canonical lemmas азъ / тꙑ) and the
    // numberless reflexive: these are not part of any lemma-keyed paradigm.
    for (person, lemma) in [(Person::First, "азъ"), (Person::Second, "тꙑ")] {
        if generated::LEMMAS
            .binary_search_by(|(key, _)| (*key).cmp(lemma))
            .is_err()
        {
            continue;
        }
        for number in Number::ALL {
            for case in Case::ALL {
                if let Ok(variants) = church_slavonic::pronoun_variants(person, number, case) {
                    for surface in &variants {
                        insert_surface(
                            &mut index,
                            surface,
                            Reading {
                                lemma,
                                pos: Pos::Pronoun,
                                cell: Cell::Personal {
                                    person,
                                    number,
                                    case,
                                },
                            },
                        );
                    }
                }
            }
        }
    }
    if generated::LEMMAS
        .binary_search_by(|(key, _)| (*key).cmp("сѧ"))
        .is_ok()
    {
        for case in Case::ALL {
            if let Ok(variants) = church_slavonic::reflexive_variants(case) {
                for surface in &variants {
                    insert_surface(
                        &mut index,
                        surface,
                        Reading {
                            lemma: "сѧ",
                            pos: Pos::Pronoun,
                            cell: Cell::Reflexive { case },
                        },
                    );
                }
            }
        }
    }
    index
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(build_index)
}

/// Every (lemma key, typed cell) reading that produces this surface form,
/// according to the facade's own paradigm enumeration. The form is
/// normalized with the same orthographic lookup key as [`lookup`], so
/// accentless and case-folded spellings match. Unknown or invalid forms
/// return an empty list.
#[must_use]
pub fn lemmatize(form: &str) -> Vec<Reading> {
    let Ok(key) = orthography::lookup_key(form) else {
        return Vec::new();
    };
    index().get(&key).cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------
// Recension-aware readings and cross-recension senses (merge phase 5,
// docs/UNIFIED_FACADE.md §4). The identity table is the authority: a sense
// attached to an OCS lexeme is reachable from the Synodal citation surface
// of the same abstract identity, marked with its provenance recension.
// ---------------------------------------------------------------------

/// One sense with its provenance recension: the recension of the evidence
/// the sense is attached to (today always
/// [`Recension::OldChurchSlavonic`], the Wiktionary source), plus the
/// abstract identity key where the sense's lexeme is identified
/// (docs/UNIFIED_IDENTITY.md).
#[derive(Debug, Clone, Copy)]
pub struct RecensionSense {
    /// The sense itself.
    pub sense: Sense,
    /// The recension of the evidence this sense is attached to.
    pub provenance: Recension,
    /// The abstract identity key of the sense's lexeme, when identified.
    pub abstract_key: Option<&'static str>,
}

/// One recension-tagged lemmatization reading: the lemma the reading is
/// served under (the abstract identity key where the lexeme is identified,
/// the native key otherwise), the abstract key when known, and the typed
/// cell, tagged with the recension whose realization produced the surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecensionReading {
    /// The lemma key the reading resolves under in the tagged recension's
    /// scope (`church_slavonic::recension(...)`).
    pub lemma: String,
    /// The abstract identity key, when the lexeme is in the identity table.
    pub abstract_key: Option<String>,
    /// Part of speech.
    pub pos: Pos,
    /// The typed cell that produces the surface.
    pub cell: Cell,
    /// The recension whose realization produced the surface.
    pub recension: Recension,
}

fn pos_from_identity(code: &str) -> Option<Pos> {
    Some(match code {
        "noun" => Pos::Noun,
        "adj" => Pos::Adjective,
        "verb" => Pos::Verb,
        "pron" => Pos::Pronoun,
        "num" => Pos::Numeral,
        "det" => Pos::Determiner,
        _ => return None,
    })
}

/// OCS facade lemma key -> abstract identity key, from the identity table
/// the facade ships.
fn abstract_by_ocs_lemma_key() -> &'static BTreeMap<&'static str, &'static str> {
    static MAP: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        church_slavonic::identity_registry()
            .entries()
            .iter()
            .filter_map(|entry| {
                entry
                    .ocs_lemma_key
                    .as_deref()
                    .map(|key| (key, entry.abstract_key.as_str()))
            })
            .collect()
    })
}

/// Projection-normal comparison key of a Synodal citation surface ->
/// identity entries carrying it.
fn identity_by_synodal_citation() -> &'static BTreeMap<String, Vec<usize>> {
    static MAP: OnceLock<BTreeMap<String, Vec<usize>>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, entry) in church_slavonic::identity_registry()
            .entries()
            .iter()
            .enumerate()
        {
            map.entry(projection::comparison_key(&entry.synodal_citation))
                .or_default()
                .push(index);
        }
        map
    })
}

/// Recension-aware sense lookup (docs/UNIFIED_FACADE.md §4). Under
/// [`Recension::OldChurchSlavonic`] this is [`lookup`], tagged. Under
/// [`Recension::SynodalRussian`] the query is folded to the
/// projection-normal comparison key and matched against the identity
/// table's Synodal citation surfaces: the senses of the same abstract
/// identity's OCS lexeme are returned, marked with their OCS provenance.
/// Any other recension is [`DictionaryError::UnsupportedRecension`].
pub fn lookup_in(
    query: &str,
    recension: Recension,
) -> Result<Vec<RecensionSense>, DictionaryError> {
    match recension {
        Recension::OldChurchSlavonic => Ok(lookup(query)?
            .into_iter()
            .map(|sense| RecensionSense {
                sense,
                provenance: Recension::OldChurchSlavonic,
                abstract_key: sense
                    .lemma_key()
                    .and_then(|key| abstract_by_ocs_lemma_key().get(key).copied()),
            })
            .collect()),
        Recension::SynodalRussian => {
            if query.trim().is_empty() {
                return Err(DictionaryError::EmptyQuery);
            }
            let key = projection::comparison_key(query);
            let registry = church_slavonic::identity_registry();
            let mut senses = Vec::new();
            for &index in identity_by_synodal_citation()
                .get(&key)
                .map_or(&[][..], Vec::as_slice)
            {
                let entry = &registry.entries()[index];
                let Some(lemma_key) = entry.ocs_lemma_key.as_deref() else {
                    continue;
                };
                for sense in senses_for_lemma_key(lemma_key) {
                    senses.push(RecensionSense {
                        sense,
                        provenance: Recension::OldChurchSlavonic,
                        abstract_key: Some(entry.abstract_key.as_str()),
                    });
                }
            }
            Ok(senses)
        }
        other => Err(DictionaryError::UnsupportedRecension(other)),
    }
}

/// Builds the Synodal lemmatization index by inverting the facade's
/// Synodal-scope paradigm enumeration over the identity table's abstract
/// keys — the same single-resolution-path rule as the OCS index. Surfaces
/// are folded accent-blind by the projection comparison key (the accent
/// asymmetry: OCS-side evidence can never key an accent-exact match).
fn synodal_index() -> &'static BTreeMap<String, Vec<RecensionReading>> {
    static INDEX: OnceLock<BTreeMap<String, Vec<RecensionReading>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut index: BTreeMap<String, Vec<RecensionReading>> = BTreeMap::new();
        let scope = church_slavonic::recension(Recension::SynodalRussian);
        let mut insert = |surface: &str, reading: RecensionReading| {
            let readings = index
                .entry(projection::comparison_key(surface))
                .or_default();
            if !readings.contains(&reading) {
                readings.push(reading);
            }
        };
        for entry in church_slavonic::identity_registry().entries() {
            let Some(pos) = pos_from_identity(&entry.pos) else {
                continue;
            };
            let key = entry.abstract_key.as_str();
            let reading = |cell: Cell| RecensionReading {
                lemma: key.to_owned(),
                abstract_key: Some(key.to_owned()),
                pos,
                cell,
                recension: Recension::SynodalRussian,
            };
            match pos {
                Pos::Noun => {
                    if let Ok(paradigm) = scope.noun_paradigm(key) {
                        for (case, number, variants) in paradigm {
                            for surface in &variants {
                                insert(surface, reading(Cell::Noun { case, number }));
                            }
                        }
                    }
                }
                Pos::Adjective => {
                    for form in [AdjectiveForm::Long, AdjectiveForm::Short] {
                        if let Ok(paradigm) = scope.adjective_paradigm(key, form) {
                            for (case, number, gender, variants) in paradigm {
                                for surface in &variants {
                                    insert(
                                        surface,
                                        reading(Cell::Adjective {
                                            form,
                                            case,
                                            number,
                                            gender,
                                        }),
                                    );
                                }
                            }
                        }
                    }
                }
                Pos::Verb => {
                    if let Ok(paradigm) = scope.verb_paradigm(key) {
                        for (kind, variants) in paradigm {
                            for surface in &variants {
                                insert(surface, reading(Cell::Verb(kind)));
                            }
                        }
                    }
                }
                // The closed classes are not on the recension scope in this
                // slice (docs/UNIFIED_FACADE.md §3).
                Pos::Pronoun | Pos::Numeral | Pos::Determiner => {}
            }
        }
        index
    })
}

/// Recension-tagged lemmatization (docs/UNIFIED_FACADE.md §4). Under
/// [`Recension::OldChurchSlavonic`] this is [`lemmatize`], each reading
/// tagged and carrying its abstract identity key where the lexeme is
/// identified. Under [`Recension::SynodalRussian`] it inverts the facade's
/// Synodal-scope paradigm enumeration over the identity table (surfaces
/// matched accent-blind). Unknown forms — and recensions the facade does
/// not serve — return an empty list.
#[must_use]
pub fn lemmatize_in(form: &str, recension: Recension) -> Vec<RecensionReading> {
    match recension {
        Recension::OldChurchSlavonic => lemmatize(form)
            .into_iter()
            .map(|reading| RecensionReading {
                lemma: reading.lemma.to_owned(),
                abstract_key: abstract_by_ocs_lemma_key()
                    .get(reading.lemma)
                    .map(|key| (*key).to_owned()),
                pos: reading.pos,
                cell: reading.cell,
                recension: Recension::OldChurchSlavonic,
            })
            .collect(),
        Recension::SynodalRussian => synodal_index()
            .get(&projection::comparison_key(form))
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_glosses() {
        let senses = lookup("аблъко").expect("valid query");
        assert!(!senses.is_empty());
        assert!(
            senses
                .iter()
                .any(|sense| sense.glosses().contains(&"apple"))
        );
        // The sense is pinned to a facade lemma key that resolves.
        let key = senses[0].lemma_key().expect("pinned lexeme");
        assert!(church_slavonic::noun_paradigm(key).is_ok());
    }

    #[test]
    fn lookup_by_accentless_page_spelling() {
        // The page title lacks the accent the canonical lemma carries.
        let senses = lookup("Аарѡнъ").expect("valid query");
        assert!(!senses.is_empty());
        assert_eq!(senses[0].lemma(), "Аарѡ\u{301}нъ");
    }

    #[test]
    fn empty_query_is_an_error() {
        assert!(matches!(lookup("  "), Err(DictionaryError::EmptyQuery)));
    }

    #[test]
    fn sense_by_id_round_trips() {
        let sense = senses().next().expect("nonempty dictionary");
        assert_eq!(sense_by_id(sense.id()).map(Sense::id), Some(sense.id()));
    }

    #[test]
    fn homograph_senses_land_on_facade_keys() {
        // сꙑнъ has two extracted lexemes (proper noun + common noun); the
        // one sense the source pins must land on one of the two facade keys,
        // and both keys must resolve in the facade.
        for key in ["сꙑнъ", "сꙑнъ_2"] {
            assert!(church_slavonic::noun_paradigm(key).is_ok(), "{key}");
        }
        let pinned: Vec<&str> = senses()
            .filter_map(Sense::lemma_key)
            .filter(|key| key.starts_with("сꙑнъ"))
            .collect();
        assert!(!pinned.is_empty());
        for key in pinned {
            assert!(key == "сꙑнъ" || key == "сꙑнъ_2", "{key}");
        }
    }

    #[test]
    fn every_pinned_lemma_key_is_in_the_lemma_inventory() {
        let inventory: std::collections::BTreeSet<&str> =
            lemmas().map(|(lemma, _)| lemma).collect();
        for sense in senses() {
            if let Some(key) = sense.lemma_key() {
                assert!(inventory.contains(key), "{} -> {key}", sense.id());
            }
        }
    }

    #[test]
    fn lemmatize_round_trips_a_noun_paradigm() {
        let paradigm = church_slavonic::noun_paradigm("аблъко").expect("known lemma");
        assert!(!paradigm.is_empty());
        for (case, number, variants) in paradigm {
            for surface in variants {
                let readings = lemmatize(&surface);
                assert!(
                    readings.contains(&Reading {
                        lemma: "аблъко",
                        pos: Pos::Noun,
                        cell: Cell::Noun { case, number },
                    }),
                    "{surface}: {readings:?}"
                );
            }
        }
    }

    #[test]
    fn lemmatize_distinguishes_homograph_keys() {
        // Both сꙑнъ lexemes share u-stem surfaces; the reading list for the
        // nominative singular must name both keys.
        let readings = lemmatize("сꙑнъ");
        let lemmas: std::collections::BTreeSet<&str> =
            readings.iter().map(|reading| reading.lemma).collect();
        assert!(
            lemmas.contains("сꙑнъ") && lemmas.contains("сꙑнъ_2"),
            "{readings:?}"
        );
    }

    #[test]
    fn lookup_in_ocs_matches_lookup() {
        let plain = lookup("аблъко").expect("valid query");
        let tagged = lookup_in("аблъко", Recension::OldChurchSlavonic).expect("valid query");
        assert_eq!(plain.len(), tagged.len());
        for wrapped in &tagged {
            assert_eq!(wrapped.provenance, Recension::OldChurchSlavonic);
        }
    }

    #[test]
    fn synodal_citation_reaches_ocs_senses() {
        // Find an identity entry whose OCS lexeme carries senses; its
        // Synodal citation surface must reach them, marked OCS-provenance.
        let entry = church_slavonic::identity_registry()
            .entries()
            .iter()
            .find(|entry| {
                entry
                    .ocs_lemma_key
                    .as_deref()
                    .is_some_and(|key| !senses_for_lemma_key(key).is_empty())
            })
            .expect("some identified lexeme carries senses");
        let senses =
            lookup_in(&entry.synodal_citation, Recension::SynodalRussian).expect("valid query");
        assert!(!senses.is_empty(), "{}", entry.abstract_key);
        for sense in &senses {
            assert_eq!(sense.provenance, Recension::OldChurchSlavonic);
        }
        assert!(
            senses
                .iter()
                .any(|sense| sense.abstract_key == Some(entry.abstract_key.as_str()))
        );
    }

    #[test]
    fn lemmatize_in_ocs_tags_and_identifies() {
        let readings = lemmatize_in("аблъко", Recension::OldChurchSlavonic);
        assert!(!readings.is_empty());
        for reading in &readings {
            assert_eq!(reading.recension, Recension::OldChurchSlavonic);
        }
    }

    #[test]
    fn lemmatize_in_synodal_round_trips_a_paradigm() {
        let scope = church_slavonic::recension(Recension::SynodalRussian);
        let (key, paradigm) = church_slavonic::identity_registry()
            .entries()
            .iter()
            .filter(|entry| entry.pos == "noun")
            .filter_map(|entry| {
                scope
                    .noun_paradigm(&entry.abstract_key)
                    .ok()
                    .filter(|paradigm| !paradigm.is_empty())
                    .map(|paradigm| (entry.abstract_key.clone(), paradigm))
            })
            .next()
            .expect("some identified noun inflects on the Synodal side");
        for (case, number, variants) in paradigm {
            for surface in variants {
                let readings = lemmatize_in(&surface, Recension::SynodalRussian);
                assert!(
                    readings.contains(&RecensionReading {
                        lemma: key.clone(),
                        abstract_key: Some(key.clone()),
                        pos: Pos::Noun,
                        cell: Cell::Noun { case, number },
                        recension: Recension::SynodalRussian,
                    }),
                    "{surface}: {readings:?}"
                );
            }
        }
    }

    #[test]
    fn unsupported_recension_is_typed() {
        assert!(matches!(
            lookup_in("аблъко", Recension::ModernRussian),
            Err(DictionaryError::UnsupportedRecension(
                Recension::ModernRussian
            )),
        ));
        assert!(lemmatize_in("аблъко", Recension::ModernRussian).is_empty());
    }

    #[test]
    fn lemmatize_unknown_form_is_empty() {
        assert!(lemmatize("xyz-not-a-form").is_empty());
        assert!(lemmatize("").is_empty());
    }
}
