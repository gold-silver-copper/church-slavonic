//! Church Slavonic morphology, lexicon-first.
//!
//! A form is produced by four independent stages — the lexeme (from the
//! [`lexicon`], or the [`guess`]er), its letters (the class table,
//! [`paradigm`]), its stress (the [`stress`] paradigm) and the typography
//! ([`form::Form::print`]). There is no fallback ladder: a lexeme is
//! complete by construction. The [`analyze`]r reads a printed word back to
//! a lexeme and a cell. See `docs/DESIGN.md`.

pub mod analyze;
pub mod cell;
pub mod form;
pub mod grammar;
pub mod guess;
pub mod inflect;
pub mod lexicon;
pub mod orthography;
pub mod paradigm;
pub mod stress;

pub use cell::*;
pub use form::Form;
pub use grammar::*;
pub use lexicon::{Lexeme, Lexicon, Provenance};
