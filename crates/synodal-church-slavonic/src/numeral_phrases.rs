//! Structured Synodal Church Slavonic numeral composition (Alypy §§61–70).
//!
//! Components remain typed words with independent form sets. Historically
//! fused spellings are represented as one composed token whose evidence and
//! trace retain every input component; genuinely multiword expressions remain
//! multiple tokens.

use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope, AnalyticConstruction,
    Animacy, AuthorityRole, Case, Confidence, EpistemicRole, Error, Evidence, EvidenceId,
    EvidenceKind, FormSet, FormSource, FormVariant, Gender, GrammarCell, InitialPresentation,
    LetterOccurrence, LexemeId, MetadataField, NounCell, Number, NumeralCell, NumeralDeclension,
    NumeralKind, NumeralLexeme, OrthographyProfile, PhraseRole, PhraseToken, PositionalOperation,
    PositionalParadigm, PositionalReplacement, PositionalRule, RealizedPhrase, Recension, Result,
    RuleId, RuleTrace, SourceId, SynodalWord, TraceStep, apply_initial_presentation,
    decline_numeral, normalize_lookup_accentless,
};
use unicode_normalization::UnicodeNormalization;

use crate::Inflector;

mod api;
mod cardinal;
mod forms;
mod fuse;
mod ordinal;
mod support;
#[cfg(test)]
mod tests;
mod types;

pub use api::*;
use cardinal::*;
use forms::*;
use fuse::*;
use ordinal::*;
use support::*;
pub use types::*;
