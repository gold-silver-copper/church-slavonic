#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    Gender, GrammarCell, ImperativeCell, Inflector, LParticipleCell, LexemeId, LexemeSummary,
    LexicalMetadataSummary, MetadataField, Number, NumeralCell, NumeralKind, PartOfSpeech,
    ParticipleCell, ParticipleTense, ParticipleVoice, Person, PronounCell, Result, abbreviation,
    capabilities_by_id, grammar_cell_registry_keys, lexemes, lexical_metadata,
    missing_metadata_by_id,
};
pub(crate) use synodal_church_slavonic_core::{
    Confidence, FormSource, RecensionMappingId, RuleTrace, SynodalWord, normalize_lookup_accentless,
};

pub mod coverage;
pub mod prediction;

pub use synodal_church_slavonic as morphology;
pub use synodal_church_slavonic_core as core;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawSense(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawExample(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawSemanticAlignment(pub [&'static str; 6]);

include!("../generated/registry.rs");


mod analysis;
mod cells;
mod entry;
mod families;
mod search;
#[cfg(test)]
mod tests;
mod vocabulary;

pub use self::analysis::*;
pub use self::cells::*;
pub use self::entry::*;
pub use self::families::*;
pub use self::search::*;
pub use self::vocabulary::*;
