//! Indexed text analysis and deterministic corpus-coverage reporting.

pub(crate) use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, OnceLock},
};

pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use synodal_church_slavonic::{
    Animacy, Case, CompoundNumeralCell, Gender, GenerationPolicy, GrammarCell, Inflector, LexemeId,
    LexemeSummary, MetadataField, NumeralComposition, OrthographyProfile, PartOfSpeech, Result,
    SynodalWord, capabilities_by_id, lexemes, lexical_metadata, missing_metadata_by_id,
    numeral_phrases::cardinal_with,
};
pub(crate) use synodal_church_slavonic_core::{
    CyrillicNumeral, Recension, RuleId, RuleTrace, TraceStep, normalize_lookup,
    normalize_lookup_accentless,
};
pub(crate) use unicode_normalization::char::is_combining_mark;

#[cfg(test)]
pub(crate) use crate::candidate_cells;
pub(crate) use crate::{
    Analysis, AnalysisSource, FamilyId, analysis_cells_for_lexeme, analysis_source,
};

mod analyzer;
mod classify;
mod marginal;
mod projection;
mod report;
#[cfg(test)]
mod tests;
mod types;

pub use self::analyzer::*;
pub use self::classify::*;
pub use self::marginal::*;
pub use self::projection::*;
pub use self::report::*;
pub use self::types::*;
