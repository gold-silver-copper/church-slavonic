//! Church Slavonic morphology, lexicon-first.
//!
//! A form is produced by four independent stages — the lexeme (from the
//! [`lexicon`], or the [`guess`]er), its letters (the class table,
//! [`paradigm`]), its stress (the [`stress`] paradigm) and the typography
//! ([`form::Form::print`]). The inherited paradigm data has not been fully
//! linguistically validated. The [`analyze`]r retrieves candidate lexemes and
//! cells; a match does not attest the candidate analysis. [`witness`] preserves
//! observed text independently of generated forms.

// a test asserts with unwrap; the workspace denies it in the code it ships
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod analyze;
pub mod cell;
pub mod document;
pub mod error;
pub mod form;
pub mod grammar;
pub mod guess;
pub mod inflect;
pub mod lexicon;
pub mod matching;
pub mod morphology;
pub mod orthography;
pub mod paradigm;
pub mod prosody;
pub mod sentence;
pub mod stress;
pub mod titlo;
pub mod witness;

pub use cell::*;
pub use error::{CellError, InflectError, LexiconError, LexiconBuildError, LexiconBuildProblem};
pub use form::Form;
pub use grammar::*;
pub use analyze::{Analysis, Reading};
pub use lexicon::{Lexeme, Lexicon, Provenance};
